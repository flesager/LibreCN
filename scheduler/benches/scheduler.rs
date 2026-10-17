//! Cost of the hot paths: ring, one message on the test bed, a cross-worker
//! round trip on the runtime.

use std::hint::black_box;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use criterion::{Criterion, criterion_group, criterion_main};
use librecn_scheduler::queue::Ring;
use librecn_scheduler::runtime::{PoolConfig, Runtime, RuntimeConfig};
use librecn_scheduler::testbed::TestBed;
use librecn_scheduler::{Context, EoConfig, EoId, ExecutionObject, Message, MsgBody};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
struct Ping {
    seq: u64,
}

impl MsgBody for Ping {
    const MSG_ID: u32 = 0x0001_0001;
}

#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
struct Go {
    rounds: u64,
}

impl MsgBody for Go {
    const MSG_ID: u32 = 0x0001_0002;
}

/// Bounces every `Ping{seq}` back to its sender as `Ping{seq + 1}`.
struct Echo;

impl ExecutionObject for Echo {
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if let Some(p) = msg.body::<Ping>() {
            let mut reply = ctx.alloc::<Ping>(msg.from()).unwrap();
            reply.seq = p.seq + 1;
            ctx.send(reply).unwrap();
        }
    }
}

/// On `Go{rounds}`, plays `rounds` round trips with the echo, then publishes the count.
struct Driver {
    echo: EoId,
    rounds: u64,
    done: Arc<AtomicU64>,
}

impl ExecutionObject for Driver {
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if let Some(go) = msg.body::<Go>() {
            self.rounds = go.rounds;
            ctx.send_value(self.echo, &Ping { seq: 0 }).unwrap();
        } else if let Some(p) = msg.body::<Ping>() {
            if p.seq < self.rounds {
                ctx.send_value(self.echo, &Ping { seq: p.seq }).unwrap();
            } else {
                self.done.store(p.seq, Ordering::Release);
            }
        }
    }
}

fn ring(c: &mut Criterion) {
    let ring = Ring::new(1024);
    c.bench_function("ring/push+pop", |b| {
        b.iter(|| {
            ring.push(black_box(7u64)).unwrap();
            black_box(ring.pop())
        })
    });
}

fn testbed(c: &mut Criterion) {
    let mut tb = TestBed::new();
    let echo = tb.add(EoConfig::new("echo"), Echo).unwrap();
    let a = tb.add(EoConfig::new("a"), Echo).unwrap();
    tb.run_until_idle();
    // Two echoes bounce one message forever: each step handles exactly one.
    tb.inject_from(a, echo, 0, &Ping { seq: 0 }).unwrap();
    c.bench_function("testbed/one message (alloc+send+schedule+handle)", |b| b.iter(|| black_box(tb.step())));
}

fn runtime(c: &mut Criterion) {
    let rt = Runtime::new(RuntimeConfig::new(vec![PoolConfig::new("cp", 2)])).unwrap();
    let echo = rt.add(EoConfig::new("echo"), Echo).unwrap();
    let done = Arc::new(AtomicU64::new(0));
    let driver = rt.add(EoConfig::new("driver"), Driver { echo, rounds: 0, done: done.clone() }).unwrap();
    c.bench_function("runtime/round trip, 2 workers", |b| {
        b.iter_custom(|iters| {
            done.store(0, Ordering::Release);
            let started = Instant::now();
            rt.inject(driver, &Go { rounds: iters }).unwrap();
            while done.load(Ordering::Acquire) < iters {
                std::hint::spin_loop();
            }
            started.elapsed()
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(3)).warm_up_time(Duration::from_secs(1));
    targets = ring, testbed, runtime
}
criterion_main!(benches);
