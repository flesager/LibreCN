//! The multi-core runtime: load balancing, overload, execution time, timers,
//! I/O, faults and lifecycle, on real threads and real time.

mod common;

use std::collections::{HashMap, HashSet};
use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::{KICK, Ping, Pong, UdpEcho, spin, wait_until};
use librecn_scheduler::runtime::{PoolConfig, Runtime, RuntimeConfig};
use librecn_scheduler::{
    Context, EoConfig, EoId, EoStats, Error, ExecutionObject, FaultPolicy, Message, MetricCollector, MsgBody, Time,
    TimerId, histogram_bucket,
};

const SECOND: Duration = Duration::from_secs(1);
const MS: Duration = Duration::from_millis(1);

/// Workers to use: up to `wanted`, never more than the machine has.
fn workers(wanted: usize) -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get()).clamp(1, wanted)
}

fn runtime(pools: Vec<PoolConfig>) -> Runtime {
    Runtime::new(RuntimeConfig::new(pools)).unwrap()
}

fn runtime_with(pools: Vec<PoolConfig>, edit: impl FnOnce(&mut RuntimeConfig)) -> Runtime {
    let mut cfg = RuntimeConfig::new(pools);
    edit(&mut cfg);
    Runtime::new(cfg).unwrap()
}

// ---------------------------------------------------------------- basics

/// Answers `Ping{seq}` with `Pong{seq + 1}` to the sender, same corr_id.
struct Echo;

impl ExecutionObject for Echo {
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if let Some(p) = msg.body::<Ping>() {
            let mut pong = ctx.alloc::<Pong>(msg.from()).unwrap().with_corr_id(msg.corr_id());
            pong.seq = p.seq + 1;
            ctx.send(pong).unwrap();
        }
    }
}

/// Starts a ping-pong of `rounds` exchanges with `peer` and counts them.
struct Pinger {
    peer: EoId,
    rounds: u64,
    done: Arc<AtomicU64>,
}

impl ExecutionObject for Pinger {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        ctx.send_value(self.peer, &Ping { seq: 0 }).unwrap();
    }
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if let Some(p) = msg.body::<Pong>() {
            self.done.store(p.seq, Ordering::Relaxed);
            if p.seq < self.rounds {
                ctx.send_value(self.peer, &Ping { seq: p.seq }).unwrap();
            }
        }
    }
}

#[test]
fn eos_exchange_messages_across_workers() {
    let rt = runtime(vec![PoolConfig::new("cp", workers(4))]);
    let echo = rt.add(EoConfig::new("echo"), Echo).unwrap();
    let done = Arc::new(AtomicU64::new(0));
    rt.add(EoConfig::new("pinger"), Pinger { peer: echo, rounds: 10_000, done: done.clone() }).unwrap();
    assert!(wait_until(10 * SECOND, || done.load(Ordering::Relaxed) == 10_000));
}

#[test]
fn eos_in_different_pools_talk_to_each_other() {
    let rt = runtime(vec![PoolConfig::new("cp", 1), PoolConfig::new("up", 1)]);
    let echo = rt.add(EoConfig::new("echo").pool("up"), Echo).unwrap();
    let done = Arc::new(AtomicU64::new(0));
    rt.add(EoConfig::new("pinger").pool("cp"), Pinger { peer: echo, rounds: 100, done: done.clone() }).unwrap();
    assert!(wait_until(5 * SECOND, || done.load(Ordering::Relaxed) == 100));
    let stats = rt.worker_stats();
    assert!(stats.iter().all(|w| w.messages > 0), "both pools worked: {stats:?}");
    assert_eq!(rt.add(EoConfig::new("x").pool("nope"), Echo), Err(Error::InvalidConfig("unknown pool")));
}

#[test]
fn config_is_validated() {
    let bad = |edit: fn(&mut RuntimeConfig)| {
        let mut cfg = RuntimeConfig::new(vec![PoolConfig::new("cp", 1)]);
        edit(&mut cfg);
        Runtime::new(cfg).err()
    };
    assert!(bad(|c| c.pools.clear()).is_some());
    assert!(bad(|c| c.pools[0].workers = 65).is_some());
    assert!(bad(|c| c.pools.push(PoolConfig::new("cp", 1))).is_some());
    assert!(bad(|c| c.fault_policy = FaultPolicy::Propagate).is_some());
    assert!(bad(|_| {}).is_none());
}

// ---------------------------------------------------------- load balancing

#[derive(Clone, Copy, Debug, zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::KnownLayout, zerocopy::Immutable)]
#[repr(C)]
struct Work {
    sender: u64,
    seq: u64,
}

impl MsgBody for Work {
    const MSG_ID: u32 = 0x0002_0001;
}

#[derive(Default)]
struct Checks {
    handled: AtomicU64,
    overlaps: AtomicU64,
    out_of_order: AtomicU64,
    threads: Mutex<HashMap<usize, HashSet<String>>>,
}

/// Spins on each `Work`, checks run-to-completion exclusivity and per-sender
/// FIFO order, then forwards a `Work` of its own to the next EO.
struct Balanced {
    index: usize,
    next: Option<EoId>,
    forwarded: u64,
    last_seq: HashMap<u64, u64>,
    in_handler: Arc<AtomicBool>,
    checks: Arc<Checks>,
    work: Duration,
}

impl ExecutionObject for Balanced {
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        let Some(w) = msg.body::<Work>() else { return };
        if self.in_handler.swap(true, Ordering::SeqCst) {
            self.checks.overlaps.fetch_add(1, Ordering::Relaxed);
        }
        let last = self.last_seq.entry(w.sender).or_insert(0);
        if w.seq != *last + 1 {
            self.checks.out_of_order.fetch_add(1, Ordering::Relaxed);
        }
        *last = w.seq;
        let thread = std::thread::current().name().unwrap_or("?").to_string();
        self.checks.threads.lock().unwrap().entry(self.index).or_default().insert(thread);
        spin(self.work);
        // Forward only work coming from the injectors (senders below 1000).
        if w.sender < 1000
            && let Some(next) = self.next
        {
            self.forwarded += 1;
            let fwd = Work { sender: 1000 + self.index as u64, seq: self.forwarded };
            // Room for every in-flight message is configured, so this never spins long.
            while ctx.send_value(next, &fwd).is_err() {}
        }
        self.checks.handled.fetch_add(1, Ordering::Relaxed);
        self.in_handler.store(false, Ordering::SeqCst);
    }
}

#[test]
fn load_is_balanced_with_exclusivity_and_ordering() {
    const EOS: usize = 16;
    const INJECTORS: u64 = 4;
    const PER_EO: u64 = 50;
    let n = workers(4);
    let rt = runtime(vec![PoolConfig::new("cp", n)]);
    let checks = Arc::new(Checks::default());

    // Build the ring back to front so each EO knows its successor.
    let mut ids = vec![EoId::SYSTEM; EOS];
    for index in (0..EOS).rev() {
        let next = (index + 1 < EOS).then(|| ids[index + 1]);
        let eo = Balanced {
            index,
            next,
            forwarded: 0,
            last_seq: HashMap::new(),
            in_handler: Arc::default(),
            checks: checks.clone(),
            work: Duration::from_micros(50),
        };
        let cfg = EoConfig::new(format!("eo{index}")).data_capacity(1024).buffers(&[(64, 1024)]);
        ids[index] = rt.add(cfg, eo).unwrap();
    }

    std::thread::scope(|s| {
        for sender in 0..INJECTORS {
            let (rt, ids) = (&rt, &ids);
            s.spawn(move || {
                for seq in 1..=PER_EO {
                    for &id in ids {
                        while rt.inject(id, &Work { sender, seq }).is_err() {
                            std::thread::yield_now();
                        }
                    }
                }
            });
        }
    });
    let injected = INJECTORS * PER_EO * EOS as u64;
    let expected = injected + injected / EOS as u64 * (EOS as u64 - 1);
    assert!(wait_until(30 * SECOND, || checks.handled.load(Ordering::Relaxed) == expected));

    assert_eq!(checks.overlaps.load(Ordering::Relaxed), 0, "an EO ran on two workers at once");
    assert_eq!(checks.out_of_order.load(Ordering::Relaxed), 0, "per-sender FIFO broken");
    if n > 1 {
        let stats = rt.worker_stats();
        let total: Duration = stats.iter().map(|w| w.busy).sum();
        for w in &stats {
            let share = w.busy.as_secs_f64() / total.as_secs_f64();
            assert!(share > 0.5 / n as f64, "worker {} got {:.0}% of the work: {stats:?}", w.index, share * 100.0);
        }
        let threads = checks.threads.lock().unwrap();
        let migrated = threads.values().filter(|t| t.len() > 1).count();
        assert!(migrated >= EOS / 2, "EOs should move between workers: {threads:?}");
    }
}

// ---------------------------------------------------------------- overload

/// Slow consumer; on each timer, records how deep its data lane was.
struct SlowSink {
    handled: Arc<AtomicU64>,
    depth_at_timer: Arc<AtomicUsize>,
}

impl ExecutionObject for SlowSink {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        ctx.arm_timer(5 * MS, 0).unwrap();
    }
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if msg.timer().is_some() {
            let depth = ctx.stats(ctx.self_id()).unwrap().data_lane.depth;
            self.depth_at_timer.fetch_max(depth, Ordering::Relaxed);
            ctx.arm_timer(5 * MS, 0).unwrap();
        } else if msg.body::<Ping>().is_some() {
            spin(Duration::from_micros(200));
            self.handled.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[test]
fn overload_from_many_threads_is_counted_exactly_and_recovers() {
    let rt = runtime(vec![PoolConfig::new("cp", workers(2))]);
    let handled = Arc::new(AtomicU64::new(0));
    let depth_at_timer = Arc::new(AtomicUsize::new(0));
    let cfg = EoConfig::new("sink").data_capacity(64).buffers(&[(64, 32), (128, 16)]);
    let sink = rt.add(cfg, SlowSink { handled: handled.clone(), depth_at_timer: depth_at_timer.clone() }).unwrap();

    let (ok, full, exhausted) = (AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0));
    let flood_until = Instant::now() + Duration::from_millis(150);
    std::thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(|| {
                let mut seq = 0;
                while Instant::now() < flood_until {
                    seq += 1;
                    let counter = match rt.inject(sink, &Ping { seq }) {
                        Ok(()) => &ok,
                        Err(Error::QueueFull) => &full,
                        Err(Error::PoolExhausted) => &exhausted,
                        Err(e) => panic!("{e}"),
                    };
                    counter.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });
    let ok = ok.load(Ordering::Relaxed);
    assert!(wait_until(10 * SECOND, || handled.load(Ordering::Relaxed) == ok), "every accepted message delivered");

    let s = rt.stats(sink).unwrap();
    assert_eq!(s.data_lane.rejected, full.load(Ordering::Relaxed));
    assert_eq!(s.buffers[0].alloc_failures, exhausted.load(Ordering::Relaxed), "failures counted on the 64 B class");
    assert!(exhausted.load(Ordering::Relaxed) > 0, "the flood exhausted the pool");
    assert_eq!(s.buffers[0].high_water, 32, "the requested class filled up");
    assert!(s.buffers[0].fallbacks > 0, "then spilled into the 128 B class");
    assert_eq!((s.data_lane.depth, s.buffers[0].in_use, s.buffers[1].in_use), (0, 0, 0), "drained, no leak");
    assert!(s.timers_fired > 5, "timers kept firing during the flood");
    assert!(depth_at_timer.load(Ordering::Relaxed) > 0, "a timer overtook queued data");
}

// ---------------------------------------------------------- execution time

struct Spinner(Duration);

impl ExecutionObject for Spinner {
    fn on_message(&mut self, msg: &Message<'_>, _: &mut Context<'_>) {
        if msg.body::<Ping>().is_some() {
            spin(self.0);
        }
    }
}

type Collection = (Time, Vec<(EoId, EoStats)>);

#[test]
fn execution_time_and_metric_loop_on_real_time() {
    let rt = runtime(vec![PoolConfig::new("cp", workers(2))]);
    let work = Duration::from_millis(1);
    let spinner = rt.add(EoConfig::new("spinner"), Spinner(work)).unwrap();
    let collected: Arc<Mutex<Vec<Collection>>> = Arc::default();
    let sink = collected.clone();
    let period = Duration::from_millis(20);
    let collector = MetricCollector::new(
        period,
        vec![spinner],
        true,
        Box::new(move |now, snapshot| sink.lock().unwrap().push((now, snapshot.to_vec()))),
    );
    rt.add(EoConfig::new("metrics"), collector).unwrap();
    for seq in 0..10 {
        rt.inject(spinner, &Ping { seq }).unwrap();
    }
    assert!(wait_until(5 * SECOND, || collected.lock().unwrap().len() >= 5));

    let p = rt.stats(spinner).unwrap().processing;
    assert_eq!(p.count, 11, "on_start + 10 pings");
    assert!(p.total >= 10 * work);
    let slow: u64 = p.histogram[histogram_bucket(work)..].iter().sum();
    assert_eq!(slow, 10);

    let collected = collected.lock().unwrap();
    let times: Vec<Time> = collected.iter().map(|(t, _)| *t).collect();
    assert!(times.windows(2).all(|w| w[1] - w[0] >= period), "one collection per period at most: {times:?}");
    let peaks: Vec<Duration> = collected.iter().map(|(_, s)| s[0].1.processing.max).collect();
    assert!(peaks[0] >= work, "first window saw the pings: {peaks:?}");
    assert!(peaks[2..].iter().all(|&m| m == Duration::ZERO), "later windows start from reset peaks: {peaks:?}");
}

// ------------------------------------------------------------------ timers

/// Arms `(tag, delay)` timers on start and records when each one fires.
struct TimerLog {
    arm: Vec<(u32, Duration)>,
    armed_at: Time,
    fired: Arc<Mutex<Vec<(u32, Duration)>>>,
}

impl ExecutionObject for TimerLog {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        self.armed_at = ctx.now();
        for &(tag, after) in &self.arm {
            ctx.arm_timer(after, tag).unwrap();
        }
    }
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if let Some(t) = msg.timer() {
            self.fired.lock().unwrap().push((t.tag, ctx.now() - self.armed_at));
        }
    }
}

#[test]
fn timers_fire_in_order_never_early() {
    let rt = runtime(vec![PoolConfig::new("cp", workers(2))]);
    let fired = Arc::default();
    let arm = vec![(3, 50 * MS), (1, 10 * MS), (2, 30 * MS)];
    let eo = TimerLog { arm: arm.clone(), armed_at: Time::ZERO, fired: Arc::clone(&fired) };
    rt.add(EoConfig::new("timers"), eo).unwrap();
    assert!(wait_until(5 * SECOND, || fired.lock().unwrap().len() == 3));
    let fired = fired.lock().unwrap();
    assert_eq!(fired.iter().map(|f| f.0).collect::<Vec<_>>(), [1, 2, 3]);
    for &(tag, elapsed) in fired.iter() {
        let delay = arm.iter().find(|a| a.0 == tag).unwrap().1;
        assert!(elapsed >= delay, "timer {tag} fired early: {elapsed:?} < {delay:?}");
    }
}

/// Arms a timer on `ARM`, cancels it on `KICK`; counts what fires.
struct ArmCancel {
    timer: Option<TimerId>,
    fired: Arc<AtomicU64>,
    cancel_failed: Arc<AtomicU64>,
}

const ARM: u32 = 0x0002_0010;

impl ExecutionObject for ArmCancel {
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if msg.timer().is_some() {
            self.fired.fetch_add(1, Ordering::Relaxed);
        } else if msg.id() == ARM {
            self.timer = Some(ctx.arm_timer(30 * MS, 0).unwrap());
        } else if msg.id() == KICK && !ctx.cancel_timer(self.timer.take().unwrap()) {
            self.cancel_failed.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[test]
fn cancelled_timers_never_fire_whichever_worker_cancels() {
    let rt = runtime(vec![PoolConfig::new("cp", workers(4)).budget(1)]);
    let (fired, cancel_failed) = (Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
    let ids: Vec<_> = (0..8)
        .map(|i| {
            let eo = ArmCancel { timer: None, fired: fired.clone(), cancel_failed: cancel_failed.clone() };
            rt.add(EoConfig::new(format!("t{i}")), eo).unwrap()
        })
        .collect();
    for _ in 0..50 {
        ids.iter().for_each(|&id| rt.signal(id, ARM).unwrap());
        std::thread::sleep(Duration::from_micros(200));
        ids.iter().for_each(|&id| rt.signal(id, KICK).unwrap());
    }
    std::thread::sleep(100 * MS);
    assert_eq!(cancel_failed.load(Ordering::Relaxed), 0);
    assert_eq!(fired.load(Ordering::Relaxed), 0, "a cancelled timer fired");
    let remote: u64 = rt.worker_stats().iter().map(|w| w.remote_cancels + w.lazy_cancels).sum();
    println!("cross-worker cancellations: {remote}");
}

// --------------------------------------------------------------------- I/O

#[test]
fn udp_echo_over_the_pool_poller() {
    let rt = runtime(vec![PoolConfig::new("cp", workers(2))]);
    let (echo, addr) = UdpEcho::new();
    let id = rt.add(EoConfig::new("echo"), echo).unwrap();
    assert!(wait_until(5 * SECOND, || addr.lock().unwrap().is_some()));
    let server = addr.lock().unwrap().unwrap();

    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    client.set_read_timeout(Some(2 * SECOND)).unwrap();
    let mut buf = [0u8; 16];
    for i in 0..200u32 {
        client.send_to(&i.to_be_bytes(), server).unwrap();
        // Burst of 5 before reading back: several datagrams per readiness.
        if i % 5 == 4 {
            for j in i - 4..=i {
                let n = client.recv(&mut buf).unwrap();
                assert_eq!(&buf[..n], &j.to_be_bytes());
            }
        }
    }
    assert_eq!(rt.stats(id).unwrap().io_events, 200, "one datagram per readiness");
}

// --------------------------------------------------------- faults, lifecycle

struct Bomb;

impl ExecutionObject for Bomb {
    fn on_message(&mut self, msg: &Message<'_>, _: &mut Context<'_>) {
        if msg.body::<Ping>().is_some_and(|p| p.seq == 3) {
            panic!("boom on 3");
        }
    }
}

#[test]
fn isolated_fault_keeps_the_other_eos_running() {
    let rt = runtime_with(vec![PoolConfig::new("cp", workers(2))], |c| c.fault_policy = FaultPolicy::Isolate);
    let bomb = rt.add(EoConfig::new("bomb"), Bomb).unwrap();
    for seq in 1..=3 {
        rt.inject(bomb, &Ping { seq }).unwrap();
    }
    assert!(wait_until(5 * SECOND, || rt.failure(bomb).is_some()));
    let r = rt.failure(bomb).unwrap();
    assert!(r.panic_message.contains("boom on 3"));
    assert!(r.thread.starts_with("cp-"), "{}", r.thread);
    assert_eq!(r.history.len(), 3, "start + 2 pings before the faulty one");
    assert!(!rt.is_alive(bomb));
    assert_eq!(rt.inject(bomb, &Ping { seq: 4 }), Err(Error::EoFaulted));

    let echo = rt.add(EoConfig::new("echo"), Echo).unwrap();
    let done = Arc::new(AtomicU64::new(0));
    rt.add(EoConfig::new("pinger"), Pinger { peer: echo, rounds: 10, done: done.clone() }).unwrap();
    assert!(wait_until(5 * SECOND, || done.load(Ordering::Relaxed) == 10));
}

struct Stoppable(Arc<AtomicU64>);

impl ExecutionObject for Stoppable {
    fn on_message(&mut self, _: &Message<'_>, _: &mut Context<'_>) {}
    fn on_stop(&mut self, _: &mut Context<'_>) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

#[test]
fn stop_and_shutdown_run_on_stop() {
    let mut rt = runtime(vec![PoolConfig::new("cp", workers(2))]);
    let stopped = Arc::new(AtomicU64::new(0));
    let ids: Vec<_> =
        (0..10).map(|i| rt.add(EoConfig::new(format!("s{i}")), Stoppable(stopped.clone())).unwrap()).collect();
    rt.stop(ids[0]).unwrap();
    // `is_alive` turns false when the EO closes, `alive()` once it is fully retired.
    assert!(wait_until(5 * SECOND, || rt.alive() == 9));
    assert!(!rt.is_alive(ids[0]));
    assert_eq!(rt.inject(ids[0], &Ping { seq: 0 }), Err(Error::UnknownEo));
    assert!(rt.shutdown(5 * SECOND), "every EO stopped in time");
    assert_eq!(stopped.load(Ordering::Relaxed), 10);
}

#[test]
fn eos_added_while_traffic_flows() {
    let rt = runtime(vec![PoolConfig::new("cp", workers(2))]);
    let echo = rt.add(EoConfig::new("echo"), Echo).unwrap();
    let dones: Vec<_> = (0..20)
        .map(|i| {
            let done = Arc::new(AtomicU64::new(0));
            rt.add(EoConfig::new(format!("p{i}")), Pinger { peer: echo, rounds: 500, done: done.clone() }).unwrap();
            done
        })
        .collect();
    assert!(wait_until(10 * SECOND, || dones.iter().all(|d| d.load(Ordering::Relaxed) == 500)));
}

/// Records the CPU of every message.
struct CpuLog(Arc<Mutex<HashSet<usize>>>);

impl ExecutionObject for CpuLog {
    fn on_message(&mut self, _: &Message<'_>, _: &mut Context<'_>) {
        self.0.lock().unwrap().insert(rustix::thread::sched_getcpu());
    }
}

#[test]
fn pinned_workers_stay_on_their_cores() {
    let allowed = rustix::thread::sched_getaffinity(None).unwrap();
    let cores: Vec<usize> = (0..1024).filter(|&c| allowed.is_set(c)).take(2).collect();
    let rt = runtime(vec![PoolConfig::pinned("cp", &cores)]);
    let seen = Arc::default();
    let id = rt.add(EoConfig::new("cpu"), CpuLog(Arc::clone(&seen))).unwrap();
    for _ in 0..200 {
        rt.signal(id, KICK).unwrap();
        std::thread::sleep(Duration::from_micros(50));
    }
    assert!(wait_until(5 * SECOND, || rt.stats(id).unwrap().processing.count == 201));
    let seen = seen.lock().unwrap();
    assert!(seen.iter().all(|c| cores.contains(c)), "ran on {seen:?}, pinned to {cores:?}");
    assert!(rt.worker_stats().iter().all(|w| w.core.is_some()));
}

/// Spawns an `Echo` child in pool "up" on start and pings it.
struct Parent(Arc<AtomicU64>);

impl ExecutionObject for Parent {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        let child = ctx.spawn(EoConfig::new("child").pool("up"), Echo).unwrap();
        ctx.send_value(child, &Ping { seq: 41 }).unwrap();
    }
    fn on_message(&mut self, msg: &Message<'_>, _: &mut Context<'_>) {
        if let Some(p) = msg.body::<Pong>() {
            self.0.store(p.seq, Ordering::Relaxed);
        }
    }
}

#[test]
fn eos_spawn_eos_in_other_pools() {
    let rt = runtime(vec![PoolConfig::new("cp", 1), PoolConfig::new("up", 1)]);
    let got = Arc::new(AtomicU64::new(0));
    rt.add(EoConfig::new("parent").pool("cp"), Parent(got.clone())).unwrap();
    assert!(wait_until(5 * SECOND, || got.load(Ordering::Relaxed) == 42));
    assert_eq!(rt.alive(), 2);
}
