//! State of an EO: the part shared with its senders, and the part only the
//! thread running it may touch.

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering, fence};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use crate::failure::FailureReport;
use crate::io::IoTable;
use crate::mailbox::{DataItem, Mailbox};
use crate::recorder::FlightRecorder;
use crate::stats::{CreationStats, EoCounters, EoStats};
use crate::timer::TimerTable;
use crate::{EoConfig, EoId, Error, ExecutionObject, MsgHeader, Result};

const IDLE: u8 = 0;
const SCHEDULED: u8 = 1;
const RUNNING: u8 = 2;
const CLOSED: u8 = 3;

/// One EO.
///
/// Run state: `Idle → Scheduled` (by the poster that finds it idle, who must
/// then queue the EO), `Scheduled → Running` (by the runner), and back to
/// `Scheduled` or `Idle` at the end of the turn, or `Closed` when it retires.
/// An EO is therefore queued at most once and run by one thread at a time;
/// that thread alone may use the runner-only cells (EO object, flight
/// recorder, timer and I/O tables).
pub(crate) struct EoCore {
    pub(crate) id: EoId,
    pub(crate) name: Box<str>,
    /// Runtime pool index (0 on the test bed).
    pub(crate) pool: u16,
    pub(crate) mailbox: Mailbox,
    pub(crate) counters: EoCounters,
    creation: CreationStats,
    state: AtomicU8,
    fault: OnceLock<Box<FailureReport>>,
    /// An `IO_READY` could not be queued: re-arm every registration at the end of the next turn.
    pub(crate) io_lost: AtomicBool,
    eo: UnsafeCell<Option<Box<dyn ExecutionObject>>>,
    recorder: UnsafeCell<FlightRecorder>,
    timers: UnsafeCell<TimerTable>,
    io: UnsafeCell<IoTable>,
}

// SAFETY: the `UnsafeCell`s are only reached through the `unsafe` accessors,
// whose callers hold the `Running` state (exclusive across threads, ordered by
// the Acquire/Release run-state transitions). Everything else is atomic,
// immutable or internally synchronised.
unsafe impl Sync for EoCore {}

impl EoCore {
    /// Allocates and prefaults every region, measuring how long it takes.
    pub(crate) fn create(id: EoId, cfg: &EoConfig, pool: u16, eo: Box<dyn ExecutionObject>) -> Result<Arc<Self>> {
        cfg.validate()?;
        let started = Instant::now();
        let mailbox = Mailbox::new(cfg)?;
        let timers = TimerTable::new(cfg.max_timers);
        let io = IoTable::new(cfg.max_io);
        let recorder = FlightRecorder::new(cfg.flight_recorder_depth);
        let bytes_reserved =
            mailbox.bytes_reserved() + timers.bytes_reserved() + io.bytes_reserved() + recorder.bytes_reserved();
        Ok(Arc::new(EoCore {
            id,
            name: cfg.name.as_str().into(),
            pool,
            mailbox,
            counters: EoCounters::new(),
            creation: CreationStats { duration: started.elapsed(), bytes_reserved },
            state: AtomicU8::new(IDLE),
            fault: OnceLock::new(),
            io_lost: AtomicBool::new(false),
            eo: UnsafeCell::new(Some(eo)),
            recorder: UnsafeCell::new(recorder),
            timers: UnsafeCell::new(timers),
            io: UnsafeCell::new(io),
        }))
    }

    /// Queues a system message. `Ok(true)`: the caller must schedule the EO.
    pub(crate) fn post_system(&self, header: MsgHeader) -> Result<bool> {
        if self.is_closed() {
            return Err(Error::UnknownEo);
        }
        self.mailbox.push_system(header)?;
        Ok(self.after_post())
    }

    /// Queues a data message; its buffer is released on failure.
    /// `Ok(true)`: the caller must schedule the EO.
    pub(crate) fn post_data(&self, item: DataItem) -> Result<bool> {
        if self.is_closed() {
            self.mailbox.release(item);
            return Err(Error::UnknownEo);
        }
        self.mailbox.push_data(item)?;
        Ok(self.after_post())
    }

    fn after_post(&self) -> bool {
        // Pairs with the fence in `end_run`: either the runner sees our message,
        // or we see its `Idle` and schedule.
        fence(Ordering::SeqCst);
        self.state.compare_exchange(IDLE, SCHEDULED, Ordering::AcqRel, Ordering::Relaxed).is_ok()
    }

    pub(crate) fn begin_run(&self) {
        let previous = self.state.swap(RUNNING, Ordering::Acquire);
        assert_eq!(previous, SCHEDULED, "EO {} run while not scheduled", self.id);
    }

    /// Ends a turn. `true`: messages are waiting and the caller must queue the EO again.
    pub(crate) fn end_run(&self) -> bool {
        if !self.mailbox.is_empty() {
            self.state.store(SCHEDULED, Ordering::Release);
            return true;
        }
        self.state.store(IDLE, Ordering::Release);
        fence(Ordering::SeqCst);
        !self.mailbox.is_empty()
            && self.state.compare_exchange(IDLE, SCHEDULED, Ordering::AcqRel, Ordering::Relaxed).is_ok()
    }

    /// Refuses every further message and drops the waiting ones. Runner only.
    pub(crate) fn close(&self) {
        self.state.store(CLOSED, Ordering::Release);
        self.mailbox.drain();
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.state.load(Ordering::Acquire) == CLOSED
    }

    pub(crate) fn set_fault(&self, report: Box<FailureReport>) {
        let _ = self.fault.set(report);
    }

    pub(crate) fn fault(&self) -> Option<&FailureReport> {
        self.fault.get().map(|r| &**r)
    }

    /// Ok if the EO accepts messages.
    pub(crate) fn check_alive(&self) -> Result<()> {
        if self.fault.get().is_some() {
            Err(Error::EoFaulted)
        } else if self.is_closed() {
            Err(Error::UnknownEo)
        } else {
            Ok(())
        }
    }

    /// # Safety
    /// Runner only, and no other reference into this cell may be alive.
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn eo_mut(&self) -> &mut Option<Box<dyn ExecutionObject>> {
        // SAFETY: guaranteed by the caller.
        unsafe { &mut *self.eo.get() }
    }

    /// # Safety
    /// Runner only, and no other reference into this cell may be alive.
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn recorder_mut(&self) -> &mut FlightRecorder {
        // SAFETY: guaranteed by the caller.
        unsafe { &mut *self.recorder.get() }
    }

    /// # Safety
    /// Runner only, and no other reference into this cell may be alive.
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn timers_mut(&self) -> &mut TimerTable {
        // SAFETY: guaranteed by the caller.
        unsafe { &mut *self.timers.get() }
    }

    /// # Safety
    /// Runner only, and no other reference into this cell may be alive.
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn io_mut(&self) -> &mut IoTable {
        // SAFETY: guaranteed by the caller.
        unsafe { &mut *self.io.get() }
    }

    pub(crate) fn stats(&self, reset_peaks: bool) -> EoStats {
        let (system_lane, data_lane) = self.mailbox.lane_stats(reset_peaks);
        let c = &self.counters;
        EoStats {
            name: self.name.to_string(),
            processing: c.processing(reset_peaks),
            runs: c.runs.load(Ordering::Relaxed),
            budget_exhausted: c.budget_exhausted.load(Ordering::Relaxed),
            timers_fired: c.timers_fired.load(Ordering::Relaxed),
            timers_stale: c.timers_stale.load(Ordering::Relaxed),
            io_events: c.io_events.load(Ordering::Relaxed),
            io_stale: c.io_stale.load(Ordering::Relaxed),
            system_lane,
            data_lane,
            buffers: self.mailbox.pool.stats(reset_peaks),
            creation: self.creation,
        }
    }
}
