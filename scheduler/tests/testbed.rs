//! Behaviour of EOs on the single-threaded test bed.

mod common;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::{Big, KICK, Ping, Pong};
use librecn_common::flags;
use librecn_scheduler::testbed::{Probe, TestBed, TestBedConfig};
use librecn_scheduler::{Context, EoConfig, EoId, Error, ExecutionObject, Message, Time, TimerId};

const MS: Duration = Duration::from_millis(1);

type Log<T> = Arc<Mutex<Vec<T>>>;

fn log<T>() -> Log<T> {
    Arc::new(Mutex::new(Vec::new()))
}

/// Answers every `Ping{seq}` with `Pong{seq + 1}`, same correlation id.
struct Echo;

impl ExecutionObject for Echo {
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if let Some(ping) = msg.body::<Ping>() {
            let mut pong =
                ctx.alloc::<Pong>(msg.from()).unwrap().with_corr_id(msg.corr_id()).with_flags(flags::RESPONSE);
            pong.seq = ping.seq + 1;
            ctx.send(pong).unwrap();
        }
    }
}

#[test]
fn request_response_keeps_correlation() {
    let mut tb = TestBed::new();
    let (probe, received) = Probe::new();
    let client = tb.add(EoConfig::new("client"), probe).unwrap();
    let echo = tb.add(EoConfig::new("echo"), Echo).unwrap();
    tb.run_until_idle();

    tb.inject_from(client, echo, 7, &Ping { seq: 41 }).unwrap();
    tb.run_until_idle();

    let got = received.take();
    assert_eq!(got.len(), 1);
    let r = &got[0];
    assert_eq!(r.body::<Pong>(), Some(Pong { seq: 42 }));
    assert_eq!(r.body::<Ping>(), None, "wrong type is refused");
    assert_eq!((r.header.from, r.header.to, r.header.corr_id, r.header.flags), (echo, client, 7, flags::RESPONSE));
    assert_eq!(r.header.body_len as usize, std::mem::size_of::<Pong>());
}

/// Arms the given `(tag, delay)` timers on start; logs `(tag, now)` on expiry,
/// re-arming the tags listed in `periodic`.
struct Timers {
    arm: Vec<(u32, Duration)>,
    cancel_on_start: Vec<u32>,
    cancel_on_fire: Vec<(u32, u32)>,
    periodic: Vec<u32>,
    ids: Vec<(u32, TimerId)>,
    fired: Log<(u32, Time)>,
    cancel_results: Log<bool>,
}

impl Timers {
    fn new(arm: &[(u32, Duration)]) -> (Self, Log<(u32, Time)>, Log<bool>) {
        let (fired, cancels) = (log(), log());
        let eo = Timers {
            arm: arm.to_vec(),
            cancel_on_start: vec![],
            cancel_on_fire: vec![],
            periodic: vec![],
            ids: vec![],
            fired: fired.clone(),
            cancel_results: cancels.clone(),
        };
        (eo, fired, cancels)
    }

    fn id(&self, tag: u32) -> TimerId {
        self.ids.iter().find(|(t, _)| *t == tag).unwrap().1
    }
}

impl ExecutionObject for Timers {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        for &(tag, after) in &self.arm {
            self.ids.push((tag, ctx.arm_timer(after, tag).unwrap()));
        }
        for tag in self.cancel_on_start.clone() {
            let ok = ctx.cancel_timer(self.id(tag));
            self.cancel_results.lock().unwrap().push(ok);
        }
    }

    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        let Some(t) = msg.timer() else { return };
        self.fired.lock().unwrap().push((t.tag, ctx.now()));
        for &(when, cancel) in &self.cancel_on_fire.clone() {
            if when == t.tag {
                let ok = ctx.cancel_timer(self.id(cancel));
                self.cancel_results.lock().unwrap().push(ok);
            }
        }
        if self.periodic.contains(&t.tag) {
            let after = self.arm.iter().find(|(tag, _)| *tag == t.tag).unwrap().1;
            ctx.arm_timer(after, t.tag).unwrap();
        }
    }
}

fn at(ms: u64) -> Time {
    Time::ZERO + Duration::from_millis(ms)
}

#[test]
fn timers_fire_at_their_virtual_deadline_in_order() {
    let mut tb = TestBed::new();
    let (eo, fired, _) = Timers::new(&[(1, 30 * MS), (2, 10 * MS), (3, 20 * MS)]);
    tb.add(EoConfig::new("timers"), eo).unwrap();
    tb.advance(25 * MS);
    assert_eq!(*fired.lock().unwrap(), [(2, at(10)), (3, at(20))]);
    assert_eq!(tb.now(), at(25));
    tb.advance(10 * MS);
    assert_eq!(fired.lock().unwrap()[2], (1, at(30)));
    assert_eq!(tb.pending_timers(), 0);
}

#[test]
fn rearmed_timer_fires_repeatedly_within_one_advance() {
    let mut tb = TestBed::new();
    let (mut eo, fired, _) = Timers::new(&[(1, 10 * MS)]);
    eo.periodic = vec![1];
    let id = tb.add(EoConfig::new("periodic"), eo).unwrap();
    tb.advance(100 * MS);
    let times: Vec<_> = fired.lock().unwrap().iter().map(|&(_, t)| t).collect();
    assert_eq!(times, (1..=10).map(|i| at(i * 10)).collect::<Vec<_>>());
    assert_eq!(tb.stats(id).unwrap().timers_fired, 10);
}

#[test]
fn cancelled_timer_never_fires() {
    let mut tb = TestBed::new();
    let (mut eo, fired, cancels) = Timers::new(&[(1, 10 * MS), (2, 10 * MS)]);
    eo.cancel_on_start = vec![2];
    let id = tb.add(EoConfig::new("cancel"), eo).unwrap();
    tb.advance(50 * MS);
    assert_eq!(*cancels.lock().unwrap(), [true]);
    assert_eq!(*fired.lock().unwrap(), [(1, at(10))]);
    assert_eq!(tb.stats(id).unwrap().timers_fired, 1);
}

#[test]
fn timer_cancelled_after_expiry_but_before_delivery_is_dropped() {
    let mut tb = TestBed::new();
    // Both expire on the same tick; handling tag 1 cancels tag 2, whose
    // expiry is already waiting in the system lane.
    let (mut eo, fired, cancels) = Timers::new(&[(1, 10 * MS), (2, 10 * MS)]);
    eo.cancel_on_fire = vec![(1, 2)];
    let id = tb.add(EoConfig::new("stale"), eo).unwrap();
    tb.advance(20 * MS);
    assert_eq!(*cancels.lock().unwrap(), [true]);
    assert_eq!(*fired.lock().unwrap(), [(1, at(10))]);
    let stats = tb.stats(id).unwrap();
    assert_eq!((stats.timers_fired, stats.timers_stale), (1, 1));
}

#[test]
fn timers_are_bounded_per_eo() {
    let mut tb = TestBed::new();
    let ok: Log<Result<TimerId, Error>> = log();
    struct Greedy(Log<Result<TimerId, Error>>);
    impl ExecutionObject for Greedy {
        fn on_start(&mut self, ctx: &mut Context<'_>) {
            for _ in 0..3 {
                let r = ctx.arm_timer(MS, 0);
                self.0.lock().unwrap().push(r);
            }
        }
        fn on_message(&mut self, _: &Message<'_>, _: &mut Context<'_>) {}
    }
    tb.add(EoConfig::new("greedy").max_timers(2), Greedy(ok.clone())).unwrap();
    tb.run_until_idle();
    let results = ok.lock().unwrap();
    assert!(results[0].is_ok() && results[1].is_ok());
    assert_eq!(results[2], Err(Error::TimersExhausted));
}

/// Logs lifecycle and messages, stops itself on `KICK`.
struct Lifecycle(Log<String>);

impl ExecutionObject for Lifecycle {
    fn on_start(&mut self, _: &mut Context<'_>) {
        self.0.lock().unwrap().push("start".into());
    }
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        self.0.lock().unwrap().push(format!("msg {:#x}", msg.id()));
        if msg.id() == KICK {
            ctx.stop_self();
        }
    }
    fn on_stop(&mut self, _: &mut Context<'_>) {
        self.0.lock().unwrap().push("stop".into());
    }
}

#[test]
fn stop_runs_on_stop_then_ids_go_stale() {
    let mut tb = TestBed::with_config(TestBedConfig { max_eos: 1, ..Default::default() });
    let events = log();
    let old = tb.add(EoConfig::new("lifecycle"), Lifecycle(events.clone())).unwrap();
    tb.run_until_idle();
    // Stop comes first (system lane): the Ping queued after the kick is dropped.
    tb.signal(old, KICK).unwrap();
    tb.inject(old, &Ping { seq: 1 }).unwrap();
    tb.run_until_idle();
    assert_eq!(*events.lock().unwrap(), ["start", &format!("msg {KICK:#x}"), "stop"]);
    assert!(!tb.is_alive(old));
    assert_eq!(tb.inject(old, &Ping { seq: 2 }), Err(Error::UnknownEo));
    assert!(tb.stats(old).is_none());

    let new = tb.add(EoConfig::new("next"), Echo).unwrap();
    assert_eq!(new.index(), old.index(), "slot reused");
    assert_ne!(new, old, "with a new generation");
    assert_eq!(tb.inject(old, &Ping { seq: 3 }), Err(Error::UnknownEo));
    assert_eq!(tb.stop(old), Err(Error::UnknownEo));
}

#[test]
fn external_stop_works_too() {
    let mut tb = TestBed::new();
    let events = log();
    let id = tb.add(EoConfig::new("lifecycle"), Lifecycle(events.clone())).unwrap();
    tb.stop(id).unwrap();
    tb.run_until_idle();
    assert_eq!(*events.lock().unwrap(), ["start", "stop"]);
    assert!(!tb.is_alive(id));
}

#[test]
fn registry_is_bounded() {
    let mut tb = TestBed::with_config(TestBedConfig { max_eos: 2, ..Default::default() });
    tb.add(EoConfig::new("a"), Echo).unwrap();
    tb.add(EoConfig::new("b"), Echo).unwrap();
    assert_eq!(tb.add(EoConfig::new("c"), Echo).unwrap_err(), Error::RegistryFull);
}

#[test]
fn invalid_config_is_refused_and_frees_the_slot() {
    let mut tb = TestBed::with_config(TestBedConfig { max_eos: 1, ..Default::default() });
    let bad = EoConfig::new("bad").max_timers(100).system_capacity(10);
    assert!(matches!(tb.add(bad, Echo), Err(Error::InvalidConfig(_))));
    assert!(tb.add(EoConfig::new("good"), Echo).is_ok());
}

#[test]
fn reserved_ids_are_refused() {
    let mut tb = TestBed::new();
    let id = tb.add(EoConfig::new("echo"), Echo).unwrap();
    assert_eq!(tb.signal(id, 0x0003), Err(Error::ReservedMsgId(0x0003)));
}

#[test]
fn body_larger_than_largest_class_is_refused() {
    let mut tb = TestBed::new();
    let small = tb.add(EoConfig::new("small").buffers(&[(256, 4)]), Echo).unwrap();
    let big = Big { data: [0; 1024] };
    assert_eq!(tb.inject(small, &big), Err(Error::BodyTooLarge(1024)));
    let roomy = tb.add(EoConfig::new("roomy").buffers(&[(256, 4), (1024, 1)]), Echo).unwrap();
    assert_eq!(tb.inject(roomy, &big), Ok(()));
}

#[test]
fn unsent_message_returns_its_buffer() {
    struct Dropper(EoId);
    impl ExecutionObject for Dropper {
        fn on_start(&mut self, ctx: &mut Context<'_>) {
            let mut msg = ctx.alloc::<Ping>(self.0).unwrap();
            msg.seq = 1;
        }
        fn on_message(&mut self, _: &Message<'_>, _: &mut Context<'_>) {}
    }
    let mut tb = TestBed::new();
    let target = tb.add(EoConfig::new("target").buffers(&[(64, 1)]), Echo).unwrap();
    tb.add(EoConfig::new("dropper"), Dropper(target)).unwrap();
    tb.run_until_idle();
    let pool = tb.stats(target).unwrap().buffers[0];
    assert_eq!((pool.in_use, pool.high_water), (0, 1));
}

#[test]
fn budget_interleaves_busy_eos() {
    struct Tagged(u8, Log<u8>);
    impl ExecutionObject for Tagged {
        fn on_message(&mut self, _: &Message<'_>, _: &mut Context<'_>) {
            self.1.lock().unwrap().push(self.0);
        }
    }
    let mut tb = TestBed::with_config(TestBedConfig { budget: 4, ..Default::default() });
    let order = log();
    let a = tb.add(EoConfig::new("a"), Tagged(b'a', order.clone())).unwrap();
    let b = tb.add(EoConfig::new("b"), Tagged(b'b', order.clone())).unwrap();
    tb.run_until_idle();
    for _ in 0..8 {
        tb.signal(a, KICK).unwrap();
        tb.signal(b, KICK).unwrap();
    }
    tb.run_until_idle();
    assert_eq!(String::from_utf8(order.lock().unwrap().clone()).unwrap(), "aaaabbbbaaaabbbb");
    let stats = tb.stats(a).unwrap();
    // Turns: start, then 4 + 4; the first of those two turns left messages waiting.
    assert_eq!((stats.runs, stats.budget_exhausted), (3, 1));
}

/// Spawns an `Echo` child on start and pings it; logs the answer.
struct Parent(Log<u64>);

impl ExecutionObject for Parent {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        let child = ctx.spawn(EoConfig::new("child"), Echo).unwrap();
        ctx.send_value(child, &Ping { seq: 1 }).unwrap();
    }
    fn on_message(&mut self, msg: &Message<'_>, _: &mut Context<'_>) {
        if let Some(p) = msg.body::<Pong>() {
            self.0.lock().unwrap().push(p.seq);
        }
    }
}

#[test]
fn eos_spawn_eos() {
    let mut tb = TestBed::new();
    let got = log();
    tb.add(EoConfig::new("parent"), Parent(got.clone())).unwrap();
    tb.run_until_idle();
    assert_eq!(*got.lock().unwrap(), [2]);
}
