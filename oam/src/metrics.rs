//! Metrics in the Prometheus text format.
//!
//! No metric EO: every EO reads the scheduler's counters through its
//! `Context`, without locks; whoever scrapes keeps the history. Processing
//! time is exported as a histogram, so no reader has to reset peaks.

use std::fmt::Write;

use librecn_scheduler::{EoInfo, EoStats, HIST_BUCKETS, WorkerStats, histogram_bucket_floor};

/// A metric family: name, type, help, value.
type Family<T> = (&'static str, &'static str, &'static str, fn(&T) -> u64);
type WorkerValue = fn(&WorkerStats) -> f64;

/// Renders the EOs (with their stats) and the workers.
pub fn render(eos: &[(EoInfo, EoStats)], workers: &[WorkerStats]) -> String {
    let mut out = String::with_capacity(4096 + eos.len() * 4096);
    let eo_labels = |e: &EoInfo| format!("eo=\"{}\",pool=\"{}\"", escape(&e.name), escape(&e.pool));

    family(&mut out, "librecn_eo_alive", "gauge", "1 while the EO runs, 0 once it faulted");
    for (e, _) in eos {
        let _ = writeln!(out, "librecn_eo_alive{{{}}} {}", eo_labels(e), u8::from(e.alive));
    }

    family(&mut out, "librecn_eo_processing_seconds", "histogram", "Time to handle one message");
    for (e, s) in eos {
        let labels = eo_labels(e);
        let mut cumulative = 0;
        for (i, count) in s.processing.histogram.iter().enumerate() {
            cumulative += count;
            let le = if i + 1 == HIST_BUCKETS {
                "+Inf".to_string()
            } else {
                format!("{:e}", histogram_bucket_floor(i + 1).as_secs_f64())
            };
            let _ = writeln!(out, "librecn_eo_processing_seconds_bucket{{{labels},le=\"{le}\"}} {cumulative}");
        }
        let _ = writeln!(out, "librecn_eo_processing_seconds_sum{{{labels}}} {}", s.processing.total.as_secs_f64());
        let _ = writeln!(out, "librecn_eo_processing_seconds_count{{{labels}}} {}", s.processing.count);
    }

    // One loop per family: Prometheus wants the samples of a family together.
    type Lane = fn(&EoStats) -> &librecn_scheduler::LaneStats;
    let lanes: [(&str, Lane); 2] = [("system", |s| &s.system_lane), ("data", |s| &s.data_lane)];
    let lane_families: [Family<librecn_scheduler::LaneStats>; 3] = [
        ("librecn_eo_lane_depth", "gauge", "Messages waiting in a mailbox lane", |l| l.depth as u64),
        ("librecn_eo_lane_high_water", "gauge", "Highest depth of a mailbox lane", |l| l.high_water as u64),
        ("librecn_eo_lane_rejected_total", "counter", "Messages refused because a lane was full", |l| l.rejected),
    ];
    for (name, kind, help, value) in lane_families {
        family(&mut out, name, kind, help);
        for (e, s) in eos {
            for (lane, get) in lanes {
                let _ = writeln!(out, "{name}{{{},lane=\"{lane}\"}} {}", eo_labels(e), value(get(s)));
            }
        }
    }

    let buffer_families: [Family<librecn_scheduler::BufferClassStats>; 5] = [
        ("librecn_eo_buffers_in_use", "gauge", "Body buffers held, per size class", |b| u64::from(b.in_use)),
        ("librecn_eo_buffers_high_water", "gauge", "Most body buffers held, per size class", |b| {
            u64::from(b.high_water)
        }),
        ("librecn_eo_buffers_count", "gauge", "Body buffers of a size class", |b| u64::from(b.count)),
        ("librecn_eo_buffer_fallbacks_total", "counter", "Bodies served by a larger class", |b| b.fallbacks),
        ("librecn_eo_buffer_failures_total", "counter", "Bodies refused, pool exhausted", |b| b.alloc_failures),
    ];
    for (name, kind, help, value) in buffer_families {
        family(&mut out, name, kind, help);
        for (e, s) in eos {
            for b in &s.buffers {
                let _ = writeln!(out, "{name}{{{},size=\"{}\"}} {}", eo_labels(e), b.size, value(b));
            }
        }
    }

    let worker_families: [(&str, &str, WorkerValue); 3] = [
        ("librecn_worker_busy_seconds_total", "Time a worker spent running EOs", |w| w.busy.as_secs_f64()),
        ("librecn_worker_idle_seconds_total", "Time a worker spent asleep", |w| w.idle.as_secs_f64()),
        ("librecn_worker_messages_total", "Messages a worker handled", |w| w.messages as f64),
    ];
    for (name, help, value) in worker_families {
        family(&mut out, name, "counter", help);
        for w in workers {
            let _ = writeln!(out, "{name}{{pool=\"{}\",worker=\"{}\"}} {}", escape(&w.pool), w.index, value(w));
        }
    }
    out
}

fn family(out: &mut String, name: &str, kind: &str, help: &str) {
    let _ = writeln!(out, "# HELP {name} {help}\n# TYPE {name} {kind}");
}

fn escape(label: &str) -> String {
    label.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}
