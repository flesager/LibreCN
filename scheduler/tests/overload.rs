//! Overload: bounded lanes and pools refuse at the sender, are counted
//! exactly, keep system messages flowing, and recover without leaks.

mod common;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use common::{KICK, Ping};
use librecn_scheduler::testbed::TestBed;
use librecn_scheduler::{Context, EoConfig, EoId, Error, ExecutionObject, Message};

const MS: Duration = Duration::from_millis(1);

/// Logs what it handles: "timer", "ping <seq>", "kick", "stop".
struct Sink(Arc<Mutex<Vec<String>>>);

impl ExecutionObject for Sink {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        ctx.arm_timer(5 * MS, 0).unwrap();
    }
    fn on_message(&mut self, msg: &Message<'_>, _: &mut Context<'_>) {
        let entry = if msg.timer().is_some() {
            "timer".to_string()
        } else if let Some(p) = msg.body::<Ping>() {
            format!("ping {}", p.seq)
        } else {
            "kick".to_string()
        };
        self.0.lock().unwrap().push(entry);
    }
    fn on_stop(&mut self, _: &mut Context<'_>) {
        self.0.lock().unwrap().push("stop".into());
    }
}

/// 8 data slots but only 4 body buffers: bodies run out first, header-only
/// messages then fill the lane.
fn sink_config() -> EoConfig {
    EoConfig::new("sink").data_capacity(8).buffers(&[(64, 4)]).max_timers(4).max_io(0).system_capacity(8)
}

fn flood(tb: &mut TestBed, sink: EoId) {
    for seq in 0..4 {
        tb.inject(sink, &Ping { seq }).unwrap();
    }
    assert_eq!(tb.inject(sink, &Ping { seq: 99 }), Err(Error::PoolExhausted));
    for _ in 0..4 {
        tb.signal(sink, KICK).unwrap();
    }
    assert_eq!(tb.signal(sink, KICK), Err(Error::QueueFull));
}

#[test]
fn flood_is_refused_counted_and_recovered() {
    let mut tb = TestBed::new();
    let log = Arc::new(Mutex::new(Vec::new()));
    let sink = tb.add(sink_config(), Sink(log.clone())).unwrap();
    tb.run_until_idle();

    flood(&mut tb, sink);
    let s = tb.stats(sink).unwrap();
    assert_eq!((s.data_lane.depth, s.data_lane.high_water, s.data_lane.rejected), (8, 8, 1));
    assert_eq!(s.data_lane.high_water, s.data_lane.capacity);
    let pool = s.buffers[0];
    assert_eq!((pool.in_use, pool.high_water, pool.alloc_failures), (4, 4, 1));

    // The timer expires while the data lane is full: it still gets in, and
    // is handled before the flood.
    tb.advance_clock(5 * MS);
    tb.run_until_idle();
    let handled = log.lock().unwrap().clone();
    assert_eq!(handled[0], "timer");
    assert_eq!(&handled[1..5], ["ping 0", "ping 1", "ping 2", "ping 3"]);
    assert_eq!(handled[5..].len(), 4);

    // Recovered: empty, every buffer back, peaks still visible until reset.
    let s = tb.stats_and_reset_peaks(sink).unwrap();
    assert_eq!((s.data_lane.depth, s.buffers[0].in_use, s.data_lane.high_water), (0, 0, 8));
    let s = tb.stats(sink).unwrap();
    assert_eq!((s.data_lane.high_water, s.buffers[0].high_water), (0, 0));
    assert_eq!((s.data_lane.rejected, s.buffers[0].alloc_failures), (1, 1), "failure counters are never reset");
    for seq in 0..4 {
        tb.inject(sink, &Ping { seq }).unwrap();
    }
}

#[test]
fn stop_gets_through_a_full_mailbox() {
    let mut tb = TestBed::new();
    let log = Arc::new(Mutex::new(Vec::new()));
    let sink = tb.add(sink_config(), Sink(log.clone())).unwrap();
    tb.run_until_idle();

    flood(&mut tb, sink);
    tb.stop(sink).unwrap();
    tb.run_until_idle();
    assert_eq!(*log.lock().unwrap(), ["stop"], "stop handled first, flood dropped");
    assert!(!tb.is_alive(sink));
}

/// Sends `count` pings to `target` from its `on_start` and tallies the outcomes.
struct Flooder {
    target: EoId,
    count: u64,
    outcomes: Arc<Mutex<Vec<Result<(), Error>>>>,
}

impl ExecutionObject for Flooder {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        for seq in 0..self.count {
            let r = ctx.send_value(self.target, &Ping { seq });
            self.outcomes.lock().unwrap().push(r);
        }
    }
    fn on_message(&mut self, _: &Message<'_>, _: &mut Context<'_>) {}
}

#[test]
fn eo_sender_sees_overload_and_receiver_catches_up() {
    let mut tb = TestBed::new();
    let log = Arc::new(Mutex::new(Vec::new()));
    let sink = tb.add(sink_config(), Sink(log.clone())).unwrap();
    let outcomes = Arc::new(Mutex::new(Vec::new()));
    tb.add(EoConfig::new("flooder"), Flooder { target: sink, count: 20, outcomes: outcomes.clone() }).unwrap();
    tb.run_until_idle();

    let outcomes = outcomes.lock().unwrap();
    assert_eq!(outcomes.iter().filter(|r| r.is_ok()).count(), 4);
    assert_eq!(outcomes.iter().filter(|r| **r == Err(Error::PoolExhausted)).count(), 16);
    // Everything accepted was delivered, in order.
    assert_eq!(*log.lock().unwrap(), ["ping 0", "ping 1", "ping 2", "ping 3"]);
    let s = tb.stats(sink).unwrap();
    assert_eq!((s.buffers[0].in_use, s.buffers[0].alloc_failures, s.data_lane.depth), (0, 16, 0));
}

#[test]
fn bodies_fall_back_to_larger_classes() {
    let mut tb = TestBed::new();
    let log = Arc::new(Mutex::new(Vec::new()));
    let cfg = EoConfig::new("sink").buffers(&[(64, 2), (128, 2)]);
    let sink = tb.add(cfg, Sink(log.clone())).unwrap();
    tb.run_until_idle();

    for seq in 0..4 {
        tb.inject(sink, &Ping { seq }).unwrap();
    }
    assert_eq!(tb.inject(sink, &Ping { seq: 4 }), Err(Error::PoolExhausted));
    let b = tb.stats(sink).unwrap().buffers;
    assert_eq!((b[0].high_water, b[0].count), (2, 2), "the 64-byte class ran out");
    assert_eq!((b[0].fallbacks, b[0].alloc_failures), (2, 1), "counted on the class the body asked for");
    assert_eq!((b[1].in_use, b[1].fallbacks, b[1].alloc_failures), (2, 0, 0));

    tb.run_until_idle();
    assert_eq!(log.lock().unwrap().len(), 4, "fallback buffers deliver like any other");
    let b = tb.stats(sink).unwrap().buffers;
    assert_eq!((b[0].in_use, b[1].in_use), (0, 0));
}
