//! Failure reports: what an EO was doing when it panicked.

use std::cell::{Cell, RefCell};
use std::fmt;
use std::marker::PhantomData;
use std::panic::{self, PanicHookInfo};
use std::sync::Once;

use crate::core::EoCore;
use crate::recorder::FlightRecorder;
use crate::{EoId, EoStats, MsgHeader};

/// What happens when a handler panics. A report is written in every case.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FaultPolicy {
    /// Abort the process after the report (production default).
    #[default]
    Abort,
    /// Mark the EO faulted and keep the others running. Needs `panic = "unwind"`.
    Isolate,
    /// Let the panic reach the caller of the platform (test bed default).
    Propagate,
}

/// Everything needed to find the root cause of a handler panic.
#[derive(Clone, Debug)]
pub struct FailureReport {
    /// The EO that panicked.
    pub eo: EoId,
    /// Its configured name.
    pub eo_name: String,
    /// Thread that ran it.
    pub thread: String,
    /// CPU it was running on.
    pub cpu: usize,
    /// Header of the message being processed.
    pub message: MsgHeader,
    /// Headers processed before it, oldest first (the flight recorder, without `message`).
    pub history: Vec<MsgHeader>,
    /// Metrics at the time of the panic.
    pub stats: EoStats,
    /// Panic payload, if it was a string.
    pub panic_message: String,
    /// `file:line:column` of the panic.
    pub location: Option<String>,
    /// Backtrace from the panic point.
    pub backtrace: String,
}

#[derive(Clone, Copy)]
struct Running {
    core: *const EoCore,
    recorder: *const FlightRecorder,
    header: MsgHeader,
    policy: FaultPolicy,
}

thread_local! {
    static RUNNING: Cell<Option<Running>> = const { Cell::new(None) };
    static LAST_REPORT: RefCell<Option<FailureReport>> = const { RefCell::new(None) };
}

/// Marks the current thread as running a handler until dropped.
pub(crate) struct RunningGuard<'a>(PhantomData<&'a ()>);

/// Records which EO and message the thread is about to process. The
/// references must outlive the guard; the panic hook reads them.
pub(crate) fn enter<'a>(
    core: &'a EoCore,
    recorder: &'a FlightRecorder,
    header: &MsgHeader,
    policy: FaultPolicy,
) -> RunningGuard<'a> {
    RUNNING.set(Some(Running { core, recorder, header: *header, policy }));
    RunningGuard(PhantomData)
}

impl Drop for RunningGuard<'_> {
    fn drop(&mut self) {
        RUNNING.set(None);
    }
}

/// Report captured by the hook for the last panic on this thread.
pub(crate) fn take_last_report() -> Option<FailureReport> {
    LAST_REPORT.with_borrow_mut(Option::take)
}

/// Installs the process-wide hook that writes failure reports to stderr.
/// Idempotent; platforms call it when they are created. Panics outside a
/// handler go to the previously installed hook.
pub fn install_panic_hook() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let previous = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let Some(running) = RUNNING.try_with(|r| r.take()).ok().flatten() else {
                return previous(info);
            };
            // SAFETY: `running` was set by `enter`, whose guard is still alive: the
            // panic is being raised inside the handler that guard covers.
            let (core, recorder) = unsafe { (&*running.core, &*running.recorder) };
            let report = FailureReport::capture(core, recorder, running.header, info);
            eprintln!("{report}");
            if running.policy == FaultPolicy::Abort {
                std::process::abort();
            }
            let _ = LAST_REPORT.try_with(|last| *last.borrow_mut() = Some(report));
        }));
    });
}

fn payload_message(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|s| s.to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "<non-string panic payload>".to_string())
}

impl FailureReport {
    fn capture(core: &EoCore, recorder: &FlightRecorder, message: MsgHeader, info: &PanicHookInfo<'_>) -> Self {
        let mut report = Self::build(core, recorder, message, payload_message(info.payload()));
        report.location = info.location().map(|l| l.to_string());
        report.backtrace = std::backtrace::Backtrace::force_capture().to_string();
        report
    }

    /// Report built after the fact, when the hook was not ours (someone replaced it).
    pub(crate) fn after_unwind(
        core: &EoCore,
        recorder: &FlightRecorder,
        message: MsgHeader,
        payload: &(dyn std::any::Any + Send),
    ) -> Self {
        let mut report = Self::build(core, recorder, message, payload_message(payload));
        report.backtrace = "<unavailable: panic hook not installed>".to_string();
        report
    }

    fn build(core: &EoCore, recorder: &FlightRecorder, message: MsgHeader, panic_message: String) -> Self {
        let mut history: Vec<_> = recorder.history().copied().collect();
        if history.last() == Some(&message) {
            history.pop();
        }
        FailureReport {
            eo: core.id,
            eo_name: core.name.to_string(),
            thread: std::thread::current().name().unwrap_or("<unnamed>").to_string(),
            cpu: rustix::thread::sched_getcpu(),
            message,
            history,
            stats: core.stats(false),
            panic_message,
            location: None,
            backtrace: String::new(),
        }
    }
}

impl fmt::Display for FailureReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = &self.stats;
        writeln!(f, "=== execution object failure ===")?;
        writeln!(f, "eo:       {} {}", self.eo_name, self.eo)?;
        writeln!(f, "thread:   {} (cpu {})", self.thread, self.cpu)?;
        writeln!(f, "panic:    {}", self.panic_message)?;
        writeln!(f, "at:       {}", self.location.as_deref().unwrap_or("<unknown>"))?;
        writeln!(f, "message:  {}", self.message)?;
        writeln!(f, "history:  {} previous message(s), oldest first", self.history.len())?;
        for (i, h) in self.history.iter().enumerate() {
            writeln!(f, "  [{i:2}] {h}")?;
        }
        writeln!(
            f,
            "stats:    processed={} max={:?} system_lane={}/{} data_lane={}/{} (rejected {})",
            s.processing.count,
            s.processing.max,
            s.system_lane.depth,
            s.system_lane.capacity,
            s.data_lane.depth,
            s.data_lane.capacity,
            s.data_lane.rejected,
        )?;
        for b in &s.buffers {
            writeln!(
                f,
                "buffers:  {:>6} B  in_use={}/{} high_water={} alloc_failures={}",
                b.size, b.in_use, b.count, b.high_water, b.alloc_failures
            )?;
        }
        write!(f, "backtrace:\n{}", self.backtrace)
    }
}
