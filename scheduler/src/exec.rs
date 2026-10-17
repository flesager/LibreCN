//! One scheduling turn of an EO, shared by the test bed and runtime workers.

use std::os::fd::AsFd;
use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::Ordering;
use std::time::Instant;

use crate::context::Platform;
use crate::core::EoCore;
use crate::failure::{self, FailureReport, FaultPolicy};
use crate::header::system;
use crate::io::{self, IoToken};
use crate::mailbox::Item;
use crate::timer::{Expired, TimerId};
use crate::{Context, Error, ExecutionObject, Message, MsgHeader};

/// How a turn ended.
pub(crate) enum TurnEnd {
    /// Nothing left to do.
    Idle,
    /// Messages are waiting: queue the EO again.
    Requeue,
    /// The EO stopped or faulted.
    Retired,
}

enum Outcome {
    Continue,
    Stop,
    Fault(Box<FailureReport>),
}

fn deliver(eo: &mut dyn ExecutionObject, msg: &Message<'_>, ctx: &mut Context<'_>) {
    match msg.id() {
        system::EO_START => eo.on_start(ctx),
        system::EO_STOP => eo.on_stop(ctx),
        _ => eo.on_message(msg, ctx),
    }
}

/// Runs up to `budget` messages of `core`, which the caller popped from a run
/// queue. Returns the messages handled.
pub(crate) fn run_turn(
    platform: &mut dyn Platform,
    core: &EoCore,
    budget: usize,
    policy: FaultPolicy,
) -> (usize, TurnEnd) {
    core.begin_run();
    let counters = &core.counters;
    counters.runs.fetch_add(1, Ordering::Relaxed);

    let mut handled = 0;
    let mut outcome = Outcome::Continue;
    while handled < budget {
        let Some(item) = core.mailbox.pop() else { break };
        let (header, buf) = match item {
            Item::System(header) => (header, None),
            Item::Data(item) => (item.header, item.buf),
        };
        let io_token = (header.msg_id == system::IO_READY).then(|| IoToken::from_raw(header.reserved as u32));
        if header.msg_id == system::TIMER_EXPIRED {
            // SAFETY: this thread runs `core` (begin_run), no other borrow of the table is alive.
            if !unsafe { core.timers_mut() }.take_fired(TimerId::from_raw(header.reserved)) {
                counters.timers_stale.fetch_add(1, Ordering::Relaxed);
                continue;
            }
            counters.timers_fired.fetch_add(1, Ordering::Relaxed);
        } else if let Some(token) = io_token {
            // SAFETY: as above, for the I/O table.
            if unsafe { core.io_mut() }.get(token).is_none() {
                counters.io_stale.fetch_add(1, Ordering::Relaxed);
                continue;
            }
            counters.io_events.fetch_add(1, Ordering::Relaxed);
        }
        handled += 1;
        if header.msg_id == system::EO_POLL {
            // Polls would flush the real history out of the recorder.
            core.poll_queued.store(false, Ordering::Release);
        } else {
            // SAFETY: runner; the recorder is not borrowed anywhere else right now.
            unsafe { core.recorder_mut() }.record(&header);
        }
        let body = buf.as_ref().map_or(&[][..], |b| core.mailbox.pool.bytes(b, header.body_len as usize));
        outcome = dispatch(platform, core, Message::new(header, body), policy);
        if let Some(buf) = buf {
            core.mailbox.pool.free(buf);
        }
        if !matches!(outcome, Outcome::Continue) {
            break;
        }
        if let Some(token) = io_token {
            // SAFETY: runner, no other borrow of the I/O table alive.
            unsafe { core.io_mut() }.mark_handled(token);
        }
    }

    match outcome {
        Outcome::Continue => {
            // Re-armed once per turn, not after each handler: a socket ready
            // again at once would otherwise fill the system lane, which goes
            // first, and starve the data lane (DESIGN.md §7).
            rearm_handled(platform, core);
            if core.io_lost.swap(false, Ordering::Relaxed) {
                rearm_all(platform, core);
            }
            if handled == budget && !core.mailbox.is_empty() {
                counters.budget_exhausted.fetch_add(1, Ordering::Relaxed);
            }
            let end = if core.end_run() { TurnEnd::Requeue } else { TurnEnd::Idle };
            (handled, end)
        }
        Outcome::Stop => {
            retire(platform, core, None);
            (handled, TurnEnd::Retired)
        }
        Outcome::Fault(report) => {
            retire(platform, core, Some(report));
            (handled, TurnEnd::Retired)
        }
    }
}

fn dispatch(platform: &mut dyn Platform, core: &EoCore, msg: Message<'_>, policy: FaultPolicy) -> Outcome {
    // SAFETY: runner. The EO object and the recorder live in their own cells;
    // the handler can only reach the timer and I/O cells, through the Context.
    let eo = unsafe { core.eo_mut() }.as_mut().expect("a running EO has its object");
    // SAFETY: as above; only read until the handler returns.
    let recorder = &*unsafe { core.recorder_mut() };
    let guard = failure::enter(core, recorder, msg.header(), policy);
    let mut ctx = Context::new(platform, core);
    let started = Instant::now();
    let result = if policy == FaultPolicy::Isolate {
        panic::catch_unwind(AssertUnwindSafe(|| deliver(eo.as_mut(), &msg, &mut ctx)))
    } else {
        deliver(eo.as_mut(), &msg, &mut ctx);
        Ok(())
    };
    core.counters.record(started.elapsed());
    drop(guard);
    match result {
        Ok(()) if msg.id() == system::EO_STOP => Outcome::Stop,
        Ok(()) => Outcome::Continue,
        Err(payload) => {
            let report = failure::take_last_report()
                .unwrap_or_else(|| FailureReport::after_unwind(core, recorder, *msg.header(), &*payload));
            Outcome::Fault(Box::new(report))
        }
    }
}

fn rearm_handled(platform: &mut dyn Platform, core: &EoCore) {
    let poller = platform.poller(core.pool);
    // SAFETY: runner, no other borrow of the I/O table alive.
    unsafe { core.io_mut() }.take_handled(|token, fd, interest| {
        // Fails only if the descriptor is gone; the EO will deregister it.
        let _ = poller.rearm(fd, io::encode(core.id.index(), token), interest);
    });
}

fn rearm_all(platform: &mut dyn Platform, core: &EoCore) {
    let poller = platform.poller(core.pool);
    // SAFETY: runner, no other borrow of the I/O table alive.
    unsafe { core.io_mut() }.for_each(|token, fd, interest| {
        let _ = poller.rearm(fd, io::encode(core.id.index(), token), interest);
    });
}

/// Closes the EO and releases everything it holds. Runner only.
pub(crate) fn retire(platform: &mut dyn Platform, core: &EoCore, report: Option<Box<FailureReport>>) {
    core.close();
    let poller = platform.poller(core.pool);
    // SAFETY: still the runner: the EO is closed, nobody else will run it.
    unsafe { core.io_mut() }.drain(|fd| poller.delete(fd.as_fd()));
    // SAFETY: as above.
    unsafe { core.timers_mut() }.drain(|handle| platform.cancel_wheel(handle));
    // SAFETY: as above.
    let eo = unsafe { core.eo_mut() }.take();
    // A panicking destructor must not take the worker down with it.
    let _ = panic::catch_unwind(AssertUnwindSafe(move || drop(eo)));
    let faulted = report.is_some();
    if let Some(report) = report {
        core.set_fault(report);
    }
    platform.retired(core, faulted);
}

/// `TIMER_EXPIRED` for an expiry popped from a wheel.
pub(crate) fn expiry_header(expired: &Expired) -> MsgHeader {
    let mut header = MsgHeader::system(system::TIMER_EXPIRED, expired.eo, expired.timer.raw());
    header.corr_id = expired.tag;
    header
}

/// Queues the `EO_POLL` of a polling EO, unless one is already waiting.
/// Returns whether the EO must be scheduled.
pub(crate) fn post_poll(core: &EoCore) -> bool {
    if core.poll_queued.swap(true, Ordering::AcqRel) {
        return false;
    }
    match core.post_system(MsgHeader::system(system::EO_POLL, core.id, 0)) {
        Ok(must_schedule) => must_schedule,
        // Lane full (the EO has work and will run anyway) or EO closed.
        Err(_) => {
            core.poll_queued.store(false, Ordering::Release);
            false
        }
    }
}

/// Queues an `IO_READY`. Returns whether the EO must be scheduled. If the
/// system lane is full the readiness is lost, so every registration of the EO
/// is re-armed at the end of its next turn (the EO is necessarily scheduled:
/// its mailbox is not empty).
pub(crate) fn post_readiness(core: &EoCore, token: IoToken, readiness: u32) -> bool {
    let mut header = MsgHeader::system(system::IO_READY, core.id, u64::from(token.raw()));
    header.corr_id = readiness;
    match core.post_system(header) {
        Ok(must_schedule) => must_schedule,
        Err(Error::QueueFull) => {
            core.io_lost.store(true, Ordering::Relaxed);
            false
        }
        Err(_) => false,
    }
}
