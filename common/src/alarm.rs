//! Raising and clearing alarms (ITU-T X.733 severities).
//!
//! An alarm is identified by its source EO and an `alarm_id` chosen by that
//! EO. Raising an alarm already active updates it; clearing removes it. The
//! alarm EO ([`names::ALARM`](crate::names::ALARM)) keeps the active list and
//! logs every change.

use librecn_scheduler::{Context, Error, MsgBody};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::msg::{family, id};
use crate::service::{Service, names};
use crate::text::{as_str, copy_truncated};

/// Severity, most severe first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Severity {
    /// Service is down; immediate action.
    Critical = 1,
    /// Service is degraded; urgent action.
    Major = 2,
    /// Not service affecting yet; action to prevent worse.
    Minor = 3,
    /// Potential problem detected.
    Warning = 4,
}

impl Severity {
    /// Upper-case name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::Major => "MAJOR",
            Severity::Minor => "MINOR",
            Severity::Warning => "WARNING",
        }
    }

    /// From a body's byte; unknown values read as `Warning`.
    pub const fn from_u8(v: u8) -> Severity {
        match v {
            1 => Severity::Critical,
            2 => Severity::Major,
            3 => Severity::Minor,
            _ => Severity::Warning,
        }
    }
}

/// Text bytes of an alarm.
pub const ALARM_TEXT: usize = 120;

/// Raise (or update) an alarm. 128 bytes.
#[derive(Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct AlarmRaise {
    /// Chosen by the source EO.
    pub alarm_id: u32,
    severity: u8,
    len: u8,
    _pad: [u8; 2],
    text: [u8; ALARM_TEXT],
}

impl MsgBody for AlarmRaise {
    const MSG_ID: u32 = id(family::OAM_ALARM, 1);
}

impl AlarmRaise {
    /// Alarm `alarm_id` with its text (cut at [`ALARM_TEXT`] bytes).
    pub fn new(alarm_id: u32, severity: Severity, text: &str) -> Self {
        let mut raise = AlarmRaise { alarm_id, severity: severity as u8, len: 0, _pad: [0; 2], text: [0; ALARM_TEXT] };
        raise.len = copy_truncated(&mut raise.text, text) as u8;
        raise
    }

    /// Severity.
    pub fn severity(&self) -> Severity {
        Severity::from_u8(self.severity)
    }

    /// The text.
    pub fn text(&self) -> &str {
        as_str(&self.text[..usize::from(self.len).min(ALARM_TEXT)])
    }
}

/// Clear an alarm.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct AlarmClear {
    /// As raised.
    pub alarm_id: u32,
}

impl MsgBody for AlarmClear {
    const MSG_ID: u32 = id(family::OAM_ALARM, 2);
}

/// Per-EO alarm handle (caches the alarm EO).
#[derive(Clone, Debug)]
pub struct Alarms {
    sink: Service,
}

impl Default for Alarms {
    fn default() -> Self {
        Alarms::new()
    }
}

impl Alarms {
    /// Handle, not looked up yet.
    pub const fn new() -> Self {
        Alarms { sink: Service::new(names::ALARM) }
    }

    /// Raises or updates alarm `alarm_id`. False if no alarm EO took it
    /// (none running, or overloaded): the caller may log instead.
    pub fn raise(&mut self, ctx: &mut Context<'_>, alarm_id: u32, severity: Severity, text: &str) -> bool {
        self.send(ctx, &AlarmRaise::new(alarm_id, severity, text))
    }

    /// Clears alarm `alarm_id`. False if no alarm EO took it.
    pub fn clear(&mut self, ctx: &mut Context<'_>, alarm_id: u32) -> bool {
        self.send(ctx, &AlarmClear { alarm_id })
    }

    fn send<T: MsgBody>(&mut self, ctx: &mut Context<'_>, body: &T) -> bool {
        let Some(sink) = self.sink.get(ctx) else { return false };
        match ctx.send_value(sink, body) {
            Ok(()) => true,
            Err(Error::UnknownEo | Error::EoFaulted) => {
                self.sink.forget();
                false
            }
            Err(_) => false,
        }
    }
}
