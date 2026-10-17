//! The alarm EO: keeps the active alarms of every EO.

use std::time::{SystemTime, UNIX_EPOCH};

use librecn_common::alarm::{AlarmClear, AlarmRaise, Severity};
use librecn_common::log::{Level, Logger};
use librecn_common::{log, names};
use librecn_scheduler::{Context, EoConfig, EoId, ExecutionObject, Message};
use serde::Serialize;

use crate::proto;

/// Active alarms kept at most; fits one [`Reply`].
pub const MAX_ACTIVE: usize = 64;

/// One active alarm, as listed by the web API.
#[derive(Clone, Debug, Serialize)]
struct Active {
    source: String,
    source_id: u64,
    alarm_id: u32,
    severity: &'static str,
    text: String,
    raised_unix_ms: u64,
}

/// Alarm manager EO (name [`names::ALARM`]). Every raise and clear is logged;
/// [`proto::ALARMS`] lists the active ones.
#[derive(Default)]
pub struct AlarmManager {
    active: Vec<Active>,
    log: Logger,
}

impl AlarmManager {
    /// No alarm active.
    pub fn new() -> Self {
        AlarmManager { active: Vec::with_capacity(MAX_ACTIVE), log: Logger::default() }
    }

    /// Name `alarm`, OAM pool.
    pub fn config() -> EoConfig {
        EoConfig::new(names::ALARM).pool(crate::POOL).buffers(&[(128, 256)]).max_io(0)
    }

    fn raise(&mut self, ctx: &mut Context<'_>, from: EoId, raise: &AlarmRaise) {
        let severity = raise.severity();
        let source = ctx.eos().into_iter().find(|e| e.id == from).map_or_else(|| from.to_string(), |e| e.name);
        let level = if severity <= Severity::Major { Level::Error } else { Level::Info };
        let alarm = Active {
            source,
            source_id: from.raw(),
            alarm_id: raise.alarm_id,
            severity: severity.as_str(),
            text: raise.text().to_string(),
            raised_unix_ms: SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64),
        };
        let existing = self.active.iter().position(|a| a.source_id == alarm.source_id && a.alarm_id == alarm.alarm_id);
        let verb = match existing {
            Some(i) => {
                self.active[i] = alarm.clone();
                "updated"
            }
            None if self.active.len() < MAX_ACTIVE => {
                self.active.push(alarm.clone());
                "raised"
            }
            None => {
                log!(self.log, ctx, Error, "alarm table full, dropped {} {}", alarm.source, alarm.alarm_id);
                return;
            }
        };
        self.log.log(
            ctx,
            level,
            format_args!("alarm {verb} {} {}/{}: {}", alarm.severity, alarm.source, alarm.alarm_id, alarm.text),
        );
    }

    fn clear(&mut self, ctx: &mut Context<'_>, from: EoId, alarm_id: u32) {
        let before = self.active.len();
        self.active.retain(|a| !(a.source_id == from.raw() && a.alarm_id == alarm_id));
        if self.active.len() < before {
            log!(self.log, ctx, Info, "alarm cleared {from}/{alarm_id}");
        }
    }
}

impl ExecutionObject for AlarmManager {
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if let Some(raise) = msg.body::<AlarmRaise>() {
            self.raise(ctx, msg.from(), raise);
        } else if let Some(clear) = msg.body::<AlarmClear>() {
            self.clear(ctx, msg.from(), clear.alarm_id);
        } else if msg.id() == proto::ALARMS {
            let json = serde_json::to_string(&self.active).unwrap_or_default();
            if let Err(e) = proto::send_reply(ctx, msg, 200, &json) {
                log!(self.log, ctx, Error, "cannot reply to {}: {e}", msg.from());
            }
        }
    }
}
