//! Hot path makes no heap allocation: send, receive, timers, scheduling.

mod common;

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use common::{Ping, Pong};
use librecn_scheduler::runtime::{PoolConfig, Runtime, RuntimeConfig};
use librecn_scheduler::testbed::TestBed;
use librecn_scheduler::{Context, EoConfig, EoId, ExecutionObject, Message, TimerId};

thread_local! {
    static ALLOCATIONS: Cell<u64> = const { Cell::new(0) };
}

struct Counting;

fn count() {
    let _ = ALLOCATIONS.try_with(|c| c.set(c.get() + 1));
}

// SAFETY: forwards to the system allocator, only counting calls.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count();
        // SAFETY: same contract as the caller's.
        unsafe { System.alloc(layout) }
    }
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        count();
        // SAFETY: same contract as the caller's.
        unsafe { System.alloc_zeroed(layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        count();
        // SAFETY: same contract as the caller's.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: same contract as the caller's.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

fn allocations() -> u64 {
    ALLOCATIONS.with(Cell::get)
}

/// Replies to pings, arming a guard timer per request and cancelling it on
/// the next one; a 1 ms periodic timer keeps the wheel busy.
struct Server {
    guard: Option<TimerId>,
}

impl ExecutionObject for Server {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        ctx.arm_timer(Duration::from_millis(1), 1).unwrap();
    }
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if msg.timer().is_some_and(|t| t.tag == 1) {
            ctx.arm_timer(Duration::from_millis(1), 1).unwrap();
        } else if let Some(ping) = msg.body::<Ping>() {
            if let Some(guard) = self.guard.take() {
                ctx.cancel_timer(guard);
            }
            self.guard = Some(ctx.arm_timer(Duration::from_secs(5), 2).unwrap());
            let mut pong = ctx.alloc::<Pong>(msg.from()).unwrap().with_corr_id(msg.corr_id());
            pong.seq = ping.seq;
            ctx.send(pong).unwrap();
        }
    }
}

/// Pings the server again on every pong, until `remaining` runs out.
struct Client {
    server: EoId,
    remaining: u64,
}

impl ExecutionObject for Client {
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if let Some(pong) = msg.body::<Pong>()
            && self.remaining > 0
        {
            self.remaining -= 1;
            ctx.send_value(self.server, &Ping { seq: pong.seq + 1 }).unwrap();
        }
    }
}

#[test]
fn message_and_timer_paths_do_not_allocate() {
    let mut tb = TestBed::new();
    let server = tb.add(EoConfig::new("server"), Server { guard: None }).unwrap();
    let client = tb.add(EoConfig::new("client"), Client { server, remaining: u64::MAX }).unwrap();
    // Warm up: first use of thread locals, lazily initialised statics, etc.
    // The ping-pong never goes idle, so drive it by steps, never `run_until_idle`.
    tb.inject_from(client, server, 0, &Ping { seq: 0 }).unwrap();
    for _ in 0..10 {
        for _ in 0..20 {
            tb.step();
        }
        tb.advance_clock(Duration::from_millis(1));
    }

    let before = allocations();
    let mut handled = 0;
    for _ in 0..1_000 {
        for _ in 0..20 {
            handled += tb.step().unwrap_or(0);
        }
        handled += usize::from(tb.step().is_some());
        tb.advance_clock(Duration::from_millis(1));
    }
    let after = allocations();
    assert!(handled > 20_000, "the scenario actually ran: {handled}");
    let stats = tb.stats(server).unwrap();
    assert!(stats.timers_fired >= 1_000, "{stats:?}");
    assert_eq!(after - before, 0, "allocations on the hot path");
}

/// Allocations made on the current thread since its previous measurement,
/// summed over every message after the warm-up.
struct Meter {
    last: Option<(std::thread::ThreadId, u64)>,
    total: Arc<AtomicU64>,
}

impl Meter {
    fn measure(&mut self, counting: bool) {
        let now = (std::thread::current().id(), allocations());
        if counting && let Some((_, count)) = self.last.filter(|(t, _)| *t == now.0) {
            self.total.fetch_add(now.1 - count, Ordering::Relaxed);
        }
        self.last = Some(now);
    }
}

/// Echo with a 1 ms periodic timer, measuring its worker thread.
struct MeteredServer {
    meter: Meter,
    seen: u64,
}

impl ExecutionObject for MeteredServer {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        ctx.arm_timer(Duration::from_millis(1), 0).unwrap();
    }
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        self.seen += 1;
        self.meter.measure(self.seen > 1_000);
        if msg.timer().is_some() {
            ctx.arm_timer(Duration::from_millis(1), 0).unwrap();
        } else if let Some(ping) = msg.body::<Ping>() {
            let mut pong = ctx.alloc::<Pong>(msg.from()).unwrap();
            pong.seq = ping.seq;
            ctx.send(pong).unwrap();
        }
    }
}

/// Plays `rounds` round trips, measuring its worker thread.
struct MeteredClient {
    server: EoId,
    rounds: u64,
    meter: Meter,
    done: Arc<AtomicU64>,
}

impl ExecutionObject for MeteredClient {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        ctx.send_value(self.server, &Ping { seq: 0 }).unwrap();
    }
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        let Some(pong) = msg.body::<Pong>() else { return };
        self.meter.measure(pong.seq > 1_000);
        if pong.seq < self.rounds {
            ctx.send_value(self.server, &Ping { seq: pong.seq + 1 }).unwrap();
        } else {
            self.done.store(1, Ordering::Release);
        }
    }
}

#[test]
fn runtime_workers_do_not_allocate() {
    let rt = Runtime::new(RuntimeConfig::new(vec![PoolConfig::new("cp", 2)])).unwrap();
    let total = Arc::new(AtomicU64::new(0));
    let done = Arc::new(AtomicU64::new(0));
    let meter = || Meter { last: None, total: total.clone() };
    let server = rt.add(EoConfig::new("server"), MeteredServer { meter: meter(), seen: 0 }).unwrap();
    let client = MeteredClient { server, rounds: 50_000, meter: meter(), done: done.clone() };
    rt.add(EoConfig::new("client"), client).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    while done.load(Ordering::Acquire) == 0 && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(done.load(Ordering::Acquire), 1, "the scenario ran to the end");
    assert!(rt.stats(server).unwrap().timers_fired > 10);
    assert_eq!(total.load(Ordering::Relaxed), 0, "allocations on worker threads between messages");
}
