//! The metric loop, as an execution object.

use std::time::Duration;

use crate::timer::Time;
use crate::{Context, EoId, EoStats, ExecutionObject, Message};

/// Callback receiving each collection: platform time and one entry per live target.
pub type MetricSink = Box<dyn FnMut(Time, &[(EoId, EoStats)]) + Send>;

/// Periodically snapshots the metrics of a set of EOs and hands them to a sink.
///
/// Collection allocates (names, buffer class lists): it runs on the metric
/// path, never on the message path of the EOs it observes.
pub struct MetricCollector {
    period: Duration,
    targets: Vec<EoId>,
    reset_peaks: bool,
    sink: MetricSink,
}

impl MetricCollector {
    /// Collects every `period`; with `reset_peaks`, each snapshot covers the
    /// peaks since the previous one.
    pub fn new(period: Duration, targets: Vec<EoId>, reset_peaks: bool, sink: MetricSink) -> Self {
        MetricCollector { period, targets, reset_peaks, sink }
    }

    fn arm(&self, ctx: &mut Context<'_>) {
        // Out of timers means a misconfigured collector; nothing better to do than stop collecting.
        let _ = ctx.arm_timer(self.period, 0);
    }
}

impl ExecutionObject for MetricCollector {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        self.arm(ctx);
    }

    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if msg.timer().is_none() {
            return;
        }
        let snapshots: Vec<_> = self
            .targets
            .iter()
            .filter_map(|&id| {
                let stats = if self.reset_peaks { ctx.stats_and_reset_peaks(id) } else { ctx.stats(id) };
                stats.map(|s| (id, s))
            })
            .collect();
        (self.sink)(ctx.now(), &snapshots);
        self.arm(ctx);
    }
}

impl std::fmt::Debug for MetricCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricCollector")
            .field("period", &self.period)
            .field("targets", &self.targets)
            .finish_non_exhaustive()
    }
}
