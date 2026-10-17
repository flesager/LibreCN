//! Logging from EOs.
//!
//! A log line is formatted on the stack into a fixed-size [`LogRecord`] and
//! sent to the log EO ([`names::LOG`](crate::names::LOG)), which writes it as
//! a JSON line. Nothing is allocated, and a slow log output never blocks the
//! EO. If no log EO runs, the line is printed to stderr directly, in the same
//! format. If the log EO's mailbox is full, the record is dropped (and
//! counted in its stats), except `Vip` and `Fatal`, which are then printed.
//!
//! ```ignore
//! use librecn_common::log::{Level, Logger};
//!
//! struct Amf { log: Logger }
//! // in a handler:
//! librecn_common::log!(self.log, ctx, Info, "ue {} registered", ue_id);
//! ```

use std::fmt::{self, Write as _};
use std::io::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};

use librecn_scheduler::{Context, Error, MsgBody};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::msg::{family, id};
use crate::service::{Service, names};
use crate::text::{FixedWriter, as_str};

/// Severity, most important first. `Vip` is never filtered out: keep it for
/// the few lines that must always appear (an NF started successfully).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Level {
    /// Always logged.
    Vip = 0,
    /// The EO or the system cannot go on.
    Fatal = 1,
    /// An operation failed.
    Error = 2,
    /// Normal operation worth knowing.
    Info = 3,
    /// Detail for troubleshooting.
    Debug = 4,
}

impl Level {
    /// Upper-case name, as written in log lines.
    pub const fn as_str(self) -> &'static str {
        match self {
            Level::Vip => "VIP",
            Level::Fatal => "FATAL",
            Level::Error => "ERROR",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
        }
    }

    /// From a record's byte; unknown values read as `Debug`.
    pub const fn from_u8(v: u8) -> Level {
        match v {
            0 => Level::Vip,
            1 => Level::Fatal,
            2 => Level::Error,
            3 => Level::Info,
            _ => Level::Debug,
        }
    }

    /// From a name, case-insensitive (for configuration).
    pub fn parse(s: &str) -> Option<Level> {
        [Level::Vip, Level::Fatal, Level::Error, Level::Info, Level::Debug]
            .into_iter()
            .find(|l| l.as_str().eq_ignore_ascii_case(s))
    }
}

/// Text bytes in a record.
pub const LOG_TEXT: usize = 240;

/// One log line on its way to the log EO. 256 bytes.
#[derive(Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct LogRecord {
    /// Wall-clock time, milliseconds since the Unix epoch.
    pub unix_ms: u64,
    level: u8,
    truncated: u8,
    len: u16,
    _pad: u32,
    text: [u8; LOG_TEXT],
}

impl MsgBody for LogRecord {
    const MSG_ID: u32 = id(family::OAM_LOG, 1);
}

impl LogRecord {
    /// A record stamped now, with the formatted text (cut at [`LOG_TEXT`] bytes).
    pub fn new(level: Level, args: fmt::Arguments<'_>) -> Self {
        let mut record = LogRecord {
            unix_ms: unix_ms_now(),
            level: level as u8,
            truncated: 0,
            len: 0,
            _pad: 0,
            text: [0; LOG_TEXT],
        };
        let mut w = FixedWriter { buf: &mut record.text, len: 0, truncated: false };
        let _ = w.write_fmt(args);
        (record.len, record.truncated) = (w.len as u16, u8::from(w.truncated));
        record
    }

    /// Severity.
    pub fn level(&self) -> Level {
        Level::from_u8(self.level)
    }

    /// The text.
    pub fn text(&self) -> &str {
        as_str(&self.text[..usize::from(self.len).min(LOG_TEXT)])
    }

    /// Whether the text was cut to fit.
    pub fn truncated(&self) -> bool {
        self.truncated != 0
    }
}

fn unix_ms_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

/// Per-EO logging handle: a severity threshold and the cached log EO.
#[derive(Clone, Debug)]
pub struct Logger {
    level: Level,
    sink: Service,
}

/// The threshold of `Logger::default()`, process-wide (`[oam] log_level`).
static DEFAULT_LEVEL: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(Level::Info as u8);

/// Sets the threshold of the loggers created from now on (`Info` until then).
pub fn set_default_level(level: Level) {
    DEFAULT_LEVEL.store(level as u8, std::sync::atomic::Ordering::Relaxed);
}

impl Default for Logger {
    /// The default threshold ([`set_default_level`]).
    fn default() -> Self {
        Logger::new(Level::from_u8(DEFAULT_LEVEL.load(std::sync::atomic::Ordering::Relaxed)))
    }
}

impl Logger {
    /// Logs `level` and more important ones.
    pub const fn new(level: Level) -> Self {
        Logger { level, sink: Service::new(names::LOG) }
    }

    /// Changes the threshold.
    pub fn set_level(&mut self, level: Level) {
        self.level = level;
    }

    /// Whether `level` passes the threshold (`Vip` always does).
    pub fn enabled(&self, level: Level) -> bool {
        level <= self.level
    }

    /// Logs a line; prefer the [`log!`](crate::log!) macro, which skips the
    /// formatting when the level is filtered out.
    pub fn log(&mut self, ctx: &mut Context<'_>, level: Level, args: fmt::Arguments<'_>) {
        if !self.enabled(level) {
            return;
        }
        let record = LogRecord::new(level, args);
        if let Some(sink) = self.sink.get(ctx) {
            match ctx.send_value(sink, &record) {
                Ok(()) => return,
                Err(Error::UnknownEo | Error::EoFaulted) => self.sink.forget(),
                // Sink overloaded: drop, unless the line must not be lost.
                Err(_) if level > Level::Fatal => return,
                Err(_) => {}
            }
        }
        print_fallback(ctx.name(), &record);
    }
}

/// Logs through a [`Logger`] if the level passes its threshold.
///
/// `log!(self.log, ctx, Info, "ue {} registered", ue_id)`
#[macro_export]
macro_rules! log {
    ($logger:expr, $ctx:expr, $level:ident, $($arg:tt)+) => {{
        let level = $crate::log::Level::$level;
        if $logger.enabled(level) {
            $logger.log($ctx, level, format_args!($($arg)+));
        }
    }};
}

fn print_fallback(eo: &str, record: &LogRecord) {
    let mut line = String::with_capacity(LOG_TEXT + 96);
    write_json_line(&mut line, record, eo);
    let _ = std::io::stderr().lock().write_all(line.as_bytes());
}

/// Prints a line to stderr in the log format, for code outside any EO (the
/// application's `main`). EOs use a [`Logger`].
pub fn print(source: &str, level: Level, args: fmt::Arguments<'_>) {
    print_fallback(source, &LogRecord::new(level, args));
}

/// Appends `record` from EO `eo` as one JSON line:
/// `{"ts":"2026-10-17T09:30:00.123Z","level":"INFO","eo":"amf","msg":"..."}`
/// plus `"truncated":true` when the text was cut.
pub fn write_json_line(out: &mut String, record: &LogRecord, eo: &str) {
    out.push_str("{\"ts\":\"");
    write_rfc3339(out, record.unix_ms);
    let _ = write!(out, "\",\"level\":\"{}\",\"eo\":", record.level().as_str());
    write_json_string(out, eo);
    out.push_str(",\"msg\":");
    write_json_string(out, record.text());
    if record.truncated() {
        out.push_str(",\"truncated\":true");
    }
    out.push_str("}\n");
}

fn write_json_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// `YYYY-MM-DDTHH:MM:SS.mmmZ` (UTC) for milliseconds since the epoch.
fn write_rfc3339(out: &mut String, unix_ms: u64) {
    let (secs, ms) = (unix_ms / 1000, unix_ms % 1000);
    let (days, rem) = (secs / 86_400, secs % 86_400);
    // Civil date from days since 1970-01-01 (H. Hinnant's algorithm).
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    let _ =
        write!(out, "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{ms:03}Z", rem / 3600, rem % 3600 / 60, rem % 60);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_order_and_vip_always_passes() {
        let quiet = Logger::new(Level::Vip);
        assert!(quiet.enabled(Level::Vip) && !quiet.enabled(Level::Fatal));
        let normal = Logger::default();
        assert!(normal.enabled(Level::Error) && normal.enabled(Level::Info) && !normal.enabled(Level::Debug));
        assert_eq!(Level::parse("debug"), Some(Level::Debug));
        assert_eq!(Level::from_u8(Level::Fatal as u8), Level::Fatal);
    }

    #[test]
    fn json_line_is_escaped_and_dated() {
        let mut record = LogRecord::new(Level::Error, format_args!("bad \"value\"\n\tx\u{1}"));
        record.unix_ms = 1_760_693_400_123; // 2025-10-17T09:30:00.123Z
        let mut line = String::new();
        write_json_line(&mut line, &record, "amf-1");
        assert_eq!(
            line,
            "{\"ts\":\"2025-10-17T09:30:00.123Z\",\"level\":\"ERROR\",\"eo\":\"amf-1\",\
             \"msg\":\"bad \\\"value\\\"\\n\\tx\\u0001\"}\n"
        );
    }

    #[test]
    fn dates_across_leap_years() {
        let mut out = String::new();
        write_rfc3339(&mut out, 0);
        assert_eq!(out, "1970-01-01T00:00:00.000Z");
        out.clear();
        write_rfc3339(&mut out, 951_782_400_000); // leap day 2000
        assert_eq!(out, "2000-02-29T00:00:00.000Z");
    }

    #[test]
    fn long_text_is_cut_and_flagged() {
        let long = "x".repeat(LOG_TEXT + 10);
        let record = LogRecord::new(Level::Info, format_args!("{long}"));
        assert_eq!((record.text().len(), record.truncated()), (LOG_TEXT, true));
        assert_eq!(std::mem::size_of::<LogRecord>(), 256);
    }
}
