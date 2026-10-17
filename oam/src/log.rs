//! The log EO: writes the records of every EO as JSON lines.

use std::io::Write;
use std::sync::{Arc, Mutex};

use librecn_common::log::{LogRecord, write_json_line};
use librecn_common::names;
use librecn_scheduler::{Context, EoConfig, EoId, ExecutionObject, Message};

/// Log sink EO (name [`names::LOG`]). Its writes are blocking: keep it in
/// the OAM pool. A record it has no room for is dropped by the sender and
/// counted in this EO's `data_lane.rejected` / `alloc_failures`.
pub struct LogSink {
    out: Box<dyn Write + Send>,
    line: String,
    /// Names of the EOs seen so far.
    names: Vec<(EoId, String)>,
}

impl LogSink {
    /// Writes to stdout (Docker and journald collect it).
    pub fn stdout() -> Self {
        LogSink::to(Box::new(std::io::stdout()))
    }

    /// Writes to `out`.
    pub fn to(out: Box<dyn Write + Send>) -> Self {
        LogSink { out, line: String::with_capacity(512), names: Vec::new() }
    }

    /// Writes into memory, for tests: put it in a test bed or runtime, then
    /// read what the EOs logged with [`CapturedLog::lines`].
    pub fn capture() -> (Self, CapturedLog) {
        let captured = CapturedLog::default();
        (LogSink::to(Box::new(captured.clone())), captured)
    }

    /// Name `log`, OAM pool, room for 4096 records.
    pub fn config() -> EoConfig {
        EoConfig::new(names::LOG).pool(crate::POOL).data_capacity(4096).buffers(&[(256, 4096)]).max_io(0)
    }

    fn name_of(&mut self, ctx: &Context<'_>, id: EoId) -> &str {
        if !self.names.iter().any(|(known, _)| *known == id) {
            self.names = ctx.eos().into_iter().map(|e| (e.id, e.name)).collect();
            if !self.names.iter().any(|(known, _)| *known == id) {
                self.names.push((id, if id == EoId::SYSTEM { "system".into() } else { id.to_string() }));
            }
        }
        &self.names.iter().find(|(known, _)| *known == id).expect("just added").1
    }
}

impl ExecutionObject for LogSink {
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        let Some(record) = msg.body::<LogRecord>() else { return };
        let mut line = std::mem::take(&mut self.line);
        line.clear();
        write_json_line(&mut line, record, self.name_of(ctx, msg.from()));
        let _ = self.out.write_all(line.as_bytes());
        self.line = line;
    }

    fn on_stop(&mut self, _: &mut Context<'_>) {
        let _ = self.out.flush();
    }
}

/// What a captured [`LogSink`] wrote.
#[derive(Clone, Default)]
pub struct CapturedLog(Arc<Mutex<Vec<u8>>>);

/// One captured log line.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
pub struct CapturedLine {
    /// RFC 3339 time.
    pub ts: String,
    /// `VIP`, `FATAL`, `ERROR`, `INFO` or `DEBUG`.
    pub level: String,
    /// Name of the EO that logged.
    pub eo: String,
    /// The text.
    pub msg: String,
}

impl CapturedLog {
    /// Every line so far, parsed.
    ///
    /// # Panics
    /// If a line is not valid JSON: the sink's output format is under test too.
    pub fn lines(&self) -> Vec<CapturedLine> {
        let bytes = self.0.lock().unwrap();
        std::str::from_utf8(&bytes)
            .expect("log output is UTF-8")
            .lines()
            .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("bad log line {l:?}: {e}")))
            .collect()
    }
}

impl Write for CapturedLog {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
