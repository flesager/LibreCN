//! Poll and hybrid modes: EOs that ask for an `EO_POLL` every turn.

mod common;

use std::io::ErrorKind;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::{KICK, Ping, Pong, wait_until};
use librecn_scheduler::runtime::{PoolConfig, Runtime, RuntimeConfig};
use librecn_scheduler::testbed::{TestBed, TestBedConfig};
use librecn_scheduler::{
    Context, EoConfig, EoId, ExecutionObject, FaultPolicy, Interest, IoToken, Message, MsgBody, system,
};

const SECOND: Duration = Duration::from_secs(1);

/// Polls from start; counts polls; stops polling on `KICK`; panics on the
/// poll following a `Ping{seq: 99}`.
struct Poller {
    polls: Arc<AtomicU64>,
    armed: bool,
}

impl ExecutionObject for Poller {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        ctx.set_polling(true);
    }
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        match msg.id() {
            system::EO_POLL => {
                assert!(!self.armed, "poll after 99");
                self.polls.fetch_add(1, Ordering::Relaxed);
            }
            KICK => ctx.set_polling(false),
            _ => self.armed = msg.body::<Ping>().is_some_and(|p| p.seq == 99),
        }
    }
}

#[test]
fn test_bed_polls_only_when_asked_and_never_twice_at_once() {
    let mut tb = TestBed::new();
    let polls = Arc::new(AtomicU64::new(0));
    let id = tb.add(EoConfig::new("poller"), Poller { polls: polls.clone(), armed: false }).unwrap();
    tb.run_until_idle();
    assert_eq!(polls.load(Ordering::Relaxed), 0, "no poll without TestBed::poll");

    assert_eq!(tb.poll(), 1);
    assert_eq!(tb.poll(), 1, "still polling, but one EO_POLL waiting at most");
    tb.run_until_idle();
    assert_eq!(polls.load(Ordering::Relaxed), 1);

    tb.signal(id, KICK).unwrap();
    tb.run_until_idle();
    assert_eq!(tb.poll(), 0, "polling turned off");
}

#[test]
fn polls_stay_out_of_the_flight_recorder() {
    let mut tb = TestBed::with_config(TestBedConfig { fault_policy: FaultPolicy::Isolate, ..Default::default() });
    let id = tb.add(EoConfig::new("poller"), Poller { polls: Arc::default(), armed: false }).unwrap();
    for _ in 0..5 {
        tb.poll();
        tb.run_until_idle();
    }
    // Ping first: a poll queued with it would run before it (system lane first).
    tb.inject(id, &Ping { seq: 99 }).unwrap();
    tb.run_until_idle();
    tb.poll();
    tb.run_until_idle();

    let r = tb.failure(id).expect("panicked on the poll");
    assert_eq!(r.message.msg_id, system::EO_POLL);
    let history: Vec<_> = r.history.iter().map(|h| h.msg_id).collect();
    assert_eq!(history, [system::EO_START, Ping::MSG_ID], "five polls and the faulty one not recorded");
}

/// Answers `Ping{seq}` with `Pong{seq + 1}`.
struct Echo;

impl ExecutionObject for Echo {
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if let Some(p) = msg.body::<Ping>() {
            ctx.send_value(msg.from(), &Pong { seq: p.seq + 1 }).unwrap();
        }
    }
}

/// Plays `rounds` round trips with the echo.
struct Pinger {
    echo: EoId,
    rounds: u64,
    done: Arc<AtomicU64>,
}

impl ExecutionObject for Pinger {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        ctx.send_value(self.echo, &Ping { seq: 0 }).unwrap();
    }
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if let Some(p) = msg.body::<Pong>() {
            self.done.store(p.seq, Ordering::Relaxed);
            if p.seq < self.rounds {
                ctx.send_value(self.echo, &Ping { seq: p.seq }).unwrap();
            }
        }
    }
}

#[test]
fn busy_poller_keeps_its_worker_without_starving_the_pool() {
    let rt = Runtime::new(RuntimeConfig::new(vec![PoolConfig::new("up", 2)])).unwrap();
    let polls = Arc::new(AtomicU64::new(0));
    let poller = rt.add(EoConfig::new("poller"), Poller { polls: polls.clone(), armed: false }).unwrap();
    let echo = rt.add(EoConfig::new("echo"), Echo).unwrap();
    let done = Arc::new(AtomicU64::new(0));
    rt.add(EoConfig::new("pinger"), Pinger { echo, rounds: 2_000, done: done.clone() }).unwrap();

    assert!(wait_until(10 * SECOND, || done.load(Ordering::Relaxed) == 2_000), "other EOs keep being served");
    std::thread::sleep(Duration::from_millis(50));
    let before = polls.load(Ordering::Relaxed);
    std::thread::sleep(Duration::from_millis(50));
    assert!(polls.load(Ordering::Relaxed) > before + 1_000, "polled continuously");

    rt.signal(poller, KICK).unwrap();
    std::thread::sleep(Duration::from_millis(20));
    let stopped = polls.load(Ordering::Relaxed);
    let idle_before: Duration = rt.worker_stats().iter().map(|w| w.idle).sum();
    std::thread::sleep(Duration::from_millis(100));
    assert!(polls.load(Ordering::Relaxed) <= stopped + 1, "no more polls once turned off");
    let idle_after: Duration = rt.worker_stats().iter().map(|w| w.idle).sum();
    assert!(idle_after - idle_before > Duration::from_millis(150), "both workers sleep again");
}

/// Hybrid UDP echo: polls the socket while traffic flows, falls back to
/// readiness after `IDLE_POLLS` empty polls, and polls again on readiness.
struct HybridEcho {
    addr: Arc<Mutex<Option<SocketAddr>>>,
    sock: Option<UdpSocket>,
    token: Option<IoToken>,
    empty_polls: u32,
    polls: Arc<AtomicU64>,
    echoed: Arc<AtomicU64>,
}

const IDLE_POLLS: u32 = 100;

impl HybridEcho {
    fn drain(&mut self) -> u64 {
        let sock = self.sock.as_ref().unwrap();
        let mut buf = [0u8; 1500];
        let mut n = 0;
        loop {
            match sock.recv_from(&mut buf) {
                Ok((len, peer)) => {
                    sock.send_to(&buf[..len], peer).unwrap();
                    n += 1;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) => panic!("recv: {e}"),
            }
        }
        self.echoed.fetch_add(n, Ordering::Relaxed);
        n
    }
}

impl ExecutionObject for HybridEcho {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
        sock.set_nonblocking(true).unwrap();
        self.token = Some(ctx.register_io(&sock, Interest::READABLE).unwrap());
        *self.addr.lock().unwrap() = Some(sock.local_addr().unwrap());
        self.sock = Some(sock);
        ctx.set_polling(true);
    }
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if msg.id() == system::EO_POLL {
            self.polls.fetch_add(1, Ordering::Relaxed);
            if self.drain() > 0 {
                self.empty_polls = 0;
            } else {
                self.empty_polls += 1;
                if self.empty_polls == IDLE_POLLS {
                    ctx.set_polling(false);
                }
            }
        } else if msg.io().is_some() {
            self.empty_polls = 0;
            ctx.set_polling(true);
            self.drain();
        }
    }
}

#[test]
fn hybrid_echo_polls_under_traffic_and_sleeps_when_idle() {
    let rt = Runtime::new(RuntimeConfig::new(vec![PoolConfig::new("up", 1)])).unwrap();
    let (addr, polls, echoed) = (Arc::default(), Arc::new(AtomicU64::new(0)), Arc::new(AtomicU64::new(0)));
    let eo = HybridEcho {
        addr: Arc::clone(&addr),
        sock: None,
        token: None,
        empty_polls: 0,
        polls: polls.clone(),
        echoed: echoed.clone(),
    };
    rt.add(EoConfig::new("hybrid"), eo).unwrap();
    assert!(wait_until(5 * SECOND, || addr.lock().unwrap().is_some()));
    let server = addr.lock().unwrap().unwrap();
    let client = UdpSocket::bind("127.0.0.1:0").unwrap();
    client.set_read_timeout(Some(2 * SECOND)).unwrap();

    let mut buf = [0u8; 8];
    for burst in 0..3u64 {
        for i in 0..50u64 {
            client.send_to(&(burst * 100 + i).to_be_bytes(), server).unwrap();
            assert_eq!(client.recv(&mut buf).unwrap(), 8);
            assert_eq!(u64::from_be_bytes(buf), burst * 100 + i);
        }
        // Idle: polling stops after IDLE_POLLS empty polls, the worker sleeps.
        std::thread::sleep(Duration::from_millis(30));
        let quiet = polls.load(Ordering::Relaxed);
        std::thread::sleep(Duration::from_millis(30));
        assert_eq!(polls.load(Ordering::Relaxed), quiet, "burst {burst}: no polling while idle");
    }
    assert_eq!(echoed.load(Ordering::Relaxed), 150);
}
