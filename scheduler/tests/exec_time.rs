//! Execution-time API: per-message processing time, peaks reset, metric loop.

mod common;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use common::Ping;
use librecn_scheduler::testbed::TestBed;
use librecn_scheduler::{
    Context, EoConfig, EoId, EoStats, ExecutionObject, HIST_BUCKETS, Message, MetricCollector, Time, histogram_bucket,
};

/// One call of the metric sink.
type Collection = (Time, Vec<(EoId, EoStats)>);

const WORK: Duration = Duration::from_micros(1500);

/// Busy-waits `WORK` per ping.
struct Spinner;

impl ExecutionObject for Spinner {
    fn on_message(&mut self, msg: &Message<'_>, _: &mut Context<'_>) {
        if msg.body::<Ping>().is_some() {
            let start = Instant::now();
            while start.elapsed() < WORK {
                std::hint::spin_loop();
            }
        }
    }
}

#[test]
fn processing_time_is_measured_per_message() {
    let mut tb = TestBed::new();
    let id = tb.add(EoConfig::new("spinner"), Spinner).unwrap();
    for seq in 0..5 {
        tb.inject(id, &Ping { seq }).unwrap();
    }
    tb.run_until_idle();

    let p = tb.stats(id).unwrap().processing;
    assert_eq!(p.count, 6, "on_start + 5 pings");
    assert!(p.max >= WORK, "max {:?}", p.max);
    assert!(p.min.unwrap() < WORK, "on_start is quick");
    assert!(p.total >= 5 * WORK);
    assert!(p.mean().unwrap() >= 5 * WORK / 6);
    assert_eq!(p.histogram.iter().sum::<u64>(), 6);
    let slow: u64 = p.histogram[histogram_bucket(WORK)..HIST_BUCKETS].iter().sum();
    assert_eq!(slow, 5, "histogram {:?}", p.histogram);
}

#[test]
fn reset_peaks_keeps_totals() {
    let mut tb = TestBed::new();
    let id = tb.add(EoConfig::new("spinner"), Spinner).unwrap();
    tb.inject(id, &Ping { seq: 0 }).unwrap();
    tb.run_until_idle();

    let before = tb.stats_and_reset_peaks(id).unwrap().processing;
    assert!(before.max >= WORK);
    let after = tb.stats(id).unwrap().processing;
    assert_eq!((after.min, after.max), (None, Duration::ZERO));
    assert_eq!((after.count, after.total, after.histogram), (before.count, before.total, before.histogram));
}

#[test]
fn creation_cost_is_reported() {
    let mut tb = TestBed::new();
    let id = tb.add(EoConfig::new("sized").buffers(&[(1024, 100)]).data_capacity(10), Spinner).unwrap();
    let c = tb.stats(id).unwrap().creation;
    assert!(c.bytes_reserved >= 1024 * 100, "{c:?}");
    assert!(c.duration > Duration::ZERO);
}

#[test]
fn metric_loop_collects_periodically() {
    let mut tb = TestBed::new();
    let spinner = tb.add(EoConfig::new("spinner"), Spinner).unwrap();
    let collected: Arc<Mutex<Vec<Collection>>> = Arc::default();
    let sink = collected.clone();
    let collector = MetricCollector::new(
        Duration::from_millis(10),
        vec![spinner],
        true,
        Box::new(move |now, snapshot| sink.lock().unwrap().push((now, snapshot.to_vec()))),
    );
    tb.add(EoConfig::new("metrics"), collector).unwrap();
    tb.inject(spinner, &Ping { seq: 0 }).unwrap();
    tb.advance(Duration::from_millis(35));

    let collected = collected.lock().unwrap();
    let times: Vec<_> = collected.iter().map(|(t, _)| *t - Time::ZERO).collect();
    assert_eq!(times, [10, 20, 30].map(Duration::from_millis));
    let max: Vec<_> = collected.iter().map(|(_, s)| s[0].1.processing.max).collect();
    assert!(max[0] >= WORK, "first window saw the ping");
    assert_eq!(max[1..], [Duration::ZERO; 2], "peaks reset at each collection");
    assert!(collected.iter().all(|(_, s)| s[0].0 == spinner && s[0].1.name == "spinner"));
}
