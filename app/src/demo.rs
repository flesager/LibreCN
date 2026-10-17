//! `demo`: a stand-in NF until the real ones exist. Ticks, logs, and raises
//! a warning alarm if asked (`params.alarm = true`).

use std::time::Duration;

use librecn_common::alarm::{Alarms, Severity};
use librecn_common::log;
use librecn_common::log::Logger;
use librecn_oam::Factory;
use librecn_scheduler::{Context, EoConfig, ExecutionObject, Message};

struct Demo {
    period: Duration,
    alarm: bool,
    ticks: u64,
    log: Logger,
    alarms: Alarms,
}

/// Parameters: `period_ms` (default 1000), `alarm` (default false).
pub fn factory() -> Factory {
    Box::new(|name, params| {
        let period = Duration::from_millis(params.get("period_ms").and_then(|v| v.as_u64()).unwrap_or(1000).max(1));
        let alarm = params.get("alarm").and_then(|v| v.as_bool()).unwrap_or(false);
        let demo = Demo { period, alarm, ticks: 0, log: Logger::default(), alarms: Alarms::new() };
        Ok((EoConfig::new(name), Box::new(demo)))
    })
}

impl ExecutionObject for Demo {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        if let Err(e) = ctx.arm_timer(self.period, 0) {
            log!(self.log, ctx, Fatal, "cannot arm timer: {e}");
            return;
        }
        if self.alarm {
            self.alarms.raise(ctx, 1, Severity::Warning, "demo alarm");
        }
        log!(self.log, ctx, Vip, "demo NF started, period {:?}", self.period);
    }

    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if msg.timer().is_some() {
            self.ticks += 1;
            log!(self.log, ctx, Debug, "tick {}", self.ticks);
            let _ = ctx.arm_timer(self.period, 0);
        }
    }

    fn on_stop(&mut self, ctx: &mut Context<'_>) {
        if self.alarm {
            self.alarms.clear(ctx, 1);
        }
        log!(self.log, ctx, Info, "demo NF stopped after {} ticks", self.ticks);
    }
}
