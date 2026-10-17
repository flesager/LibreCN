//! What an EO can do while handling a message, and the platform contract
//! behind it.

use std::os::fd::AsFd;
use std::sync::Arc;
use std::time::Duration;

use crate::core::EoCore;
use crate::header::system;
use crate::io::{self, Interest, IoToken, Poller};
use crate::mailbox::DataItem;
use crate::registry::Registry;
use crate::timer::{Time, TimerId, WheelHandle};
use crate::{EoConfig, EoId, EoInfo, EoStats, Error, ExecutionObject, MsgBody, MsgHeader, OutMsg, Result, WorkerStats};

/// Enough to post messages: test bed, runtime workers, and the runtime's
/// control handle all implement it.
pub(crate) trait Poster {
    fn now(&self) -> Time;
    /// A live EO, or why it is not.
    fn lookup(&mut self, id: EoId) -> Result<Arc<EoCore>>;
    /// Queues an EO whose post returned "must schedule".
    fn schedule(&mut self, core: &EoCore);
}

/// Everything a platform gives to the EO it runs.
pub(crate) trait Platform: Poster {
    /// Inserts an expiry in this thread's timer wheel.
    fn arm_wheel(&mut self, eo: EoId, after: Duration, timer: TimerId, tag: u32) -> Result<WheelHandle>;
    /// Removes an expiry from whichever wheel holds it (best effort: the
    /// timer table already makes a cancelled expiry stale).
    fn cancel_wheel(&mut self, handle: WheelHandle);
    /// The epoll instance of a pool.
    fn poller(&self, pool: u16) -> &Poller;
    /// The EO retired; free its registry slot unless it faulted.
    fn retired(&mut self, core: &EoCore, faulted: bool);
    /// Creates and schedules an EO.
    fn spawn(&mut self, cfg: &EoConfig, eo: Box<dyn ExecutionObject>) -> Result<EoId>;
    fn registry(&self) -> &Registry;
    /// Name of a pool (empty on the test bed).
    fn pool_name(&self, pool: u16) -> &str;
    /// Activity of the workers (none on the test bed).
    fn worker_stats(&self) -> Vec<WorkerStats>;
}

pub(crate) fn check_user_msg_id(msg_id: u32) -> Result<()> {
    if system::is_system(msg_id) { Err(Error::ReservedMsgId(msg_id)) } else { Ok(()) }
}

fn post_item<P: Poster + ?Sized>(platform: &mut P, core: &EoCore, item: DataItem) -> Result<()> {
    if core.post_data(item)? {
        platform.schedule(core);
    }
    Ok(())
}

/// Copies `value` into `to`'s pool and queues it.
pub(crate) fn post_value<P: Poster + ?Sized, T: MsgBody>(
    platform: &mut P,
    from: EoId,
    to: EoId,
    corr_id: u32,
    value: &T,
) -> Result<()> {
    check_user_msg_id(T::MSG_ID)?;
    let core = platform.lookup(to)?;
    let mut msg = OutMsg::<T>::new(core, from)?.with_corr_id(corr_id);
    msg.as_mut_bytes().copy_from_slice(value.as_bytes());
    let (core, item) = msg.into_parts();
    post_item(platform, &core, item)
}

/// Queues a header-only message on `to`'s data lane.
pub(crate) fn post_signal<P: Poster + ?Sized>(
    platform: &mut P,
    from: EoId,
    to: EoId,
    msg_id: u32,
    corr_id: u32,
) -> Result<()> {
    check_user_msg_id(msg_id)?;
    let core = platform.lookup(to)?;
    let mut header = MsgHeader::new(msg_id, from, to);
    header.corr_id = corr_id;
    post_item(platform, &core, DataItem { header, buf: None })
}

/// Queues `EO_STOP` on `to`'s system lane.
pub(crate) fn post_stop<P: Poster + ?Sized>(platform: &mut P, to: EoId) -> Result<()> {
    let core = platform.lookup(to)?;
    if core.post_system(MsgHeader::system(system::EO_STOP, to, 0))? {
        platform.schedule(&core);
    }
    Ok(())
}

/// The EO's only way to reach the outside world. Same API on the runtime and
/// on the test bed.
pub struct Context<'a> {
    platform: &'a mut dyn Platform,
    core: &'a EoCore,
}

impl<'a> Context<'a> {
    /// `core` must be the EO this thread is running.
    pub(crate) fn new(platform: &'a mut dyn Platform, core: &'a EoCore) -> Self {
        Context { platform, core }
    }

    /// Adds `n` to the EO's counter `counter` (an index into
    /// [`EoConfig::counters`](crate::EoConfig::counters); ignored if out of
    /// range). One relaxed atomic add.
    pub fn count(&self, counter: usize, n: u64) {
        if let Some(c) = self.core.own.get(counter) {
            c.fetch_add(n, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Id of the running EO.
    pub fn self_id(&self) -> EoId {
        self.core.id
    }

    /// Name of the running EO.
    pub fn name(&self) -> &str {
        &self.core.name
    }

    /// Platform time.
    pub fn now(&self) -> Time {
        self.platform.now()
    }

    /// Takes a zeroed buffer for a `T` from `to`'s pool. Fill it, then [`send`](Self::send) it.
    pub fn alloc<T: MsgBody>(&mut self, to: EoId) -> Result<OutMsg<T>> {
        check_user_msg_id(T::MSG_ID)?;
        let core = self.platform.lookup(to)?;
        OutMsg::new(core, self.core.id)
    }

    /// Queues a message written with [`alloc`](Self::alloc).
    pub fn send<T: MsgBody>(&mut self, msg: OutMsg<T>) -> Result<()> {
        let (core, item) = msg.into_parts();
        post_item(self.platform, &core, item)
    }

    /// Copies `value` into a buffer of `to`'s pool and queues it.
    pub fn send_value<T: MsgBody>(&mut self, to: EoId, value: &T) -> Result<()> {
        post_value(self.platform, self.core.id, to, 0, value)
    }

    /// Queues a header-only message on `to`'s data lane.
    pub fn signal(&mut self, to: EoId, msg_id: u32, corr_id: u32) -> Result<()> {
        post_signal(self.platform, self.core.id, to, msg_id, corr_id)
    }

    /// One-shot timer; delivered as `TIMER_EXPIRED` no earlier than `after`
    /// from now, with `tag` in [`TimerFired::tag`](crate::TimerFired::tag).
    pub fn arm_timer(&mut self, after: Duration, tag: u32) -> Result<TimerId> {
        // SAFETY: a Context only exists while this thread runs `core`, and this
        // borrow of the timer table ends before the call returns.
        let timers = unsafe { self.core.timers_mut() };
        let id = timers.reserve()?;
        match self.platform.arm_wheel(self.core.id, after, id, tag) {
            Ok(handle) => {
                timers.attach(id, handle);
                Ok(id)
            }
            Err(e) => {
                timers.release(id);
                Err(e)
            }
        }
    }

    /// Cancels an armed timer; false if it already fired or was cancelled.
    /// A cancelled timer is never delivered, even if it already expired.
    pub fn cancel_timer(&mut self, id: TimerId) -> bool {
        // SAFETY: as in `arm_timer`.
        let handle = unsafe { self.core.timers_mut() }.cancel(id);
        if let Some(handle) = handle {
            self.platform.cancel_wheel(handle);
        }
        handle.is_some()
    }

    /// Watches `fd`; readiness arrives as `IO_READY` (see
    /// [`Message::io`](crate::Message::io)). The platform keeps a duplicate
    /// of the descriptor until [`deregister_io`](Self::deregister_io) or
    /// the EO stops.
    pub fn register_io(&mut self, fd: &impl AsFd, interest: Interest) -> Result<IoToken> {
        let dup = fd.as_fd().try_clone_to_owned()?;
        // SAFETY: as in `arm_timer`, for the I/O table.
        let table = unsafe { self.core.io_mut() };
        let token = table.insert(dup, interest)?;
        let (fd, _) = table.get(token).expect("just inserted");
        if let Err(e) = self.platform.poller(self.core.pool).add(fd, io::encode(self.core.id.index(), token), interest)
        {
            table.remove(token);
            return Err(e);
        }
        Ok(token)
    }

    /// Stops watching; false if `token` is not registered. A readiness
    /// already queued for it is dropped.
    pub fn deregister_io(&mut self, token: IoToken) -> bool {
        // SAFETY: as in `register_io`.
        let Some(fd) = (unsafe { self.core.io_mut() }).remove(token) else { return false };
        self.platform.poller(self.core.pool).delete(fd.as_fd());
        true
    }

    /// Poll mode: while on, the EO receives one `EO_POLL` per turn and stays
    /// runnable, so its worker never sleeps (busy polling of rings, sockets in
    /// non-blocking mode, ...). Turning it off when there is nothing to poll,
    /// and back on at the next `IO_READY` or message, gives a hybrid mode.
    /// The test bed delivers polls only on [`TestBed::poll`](crate::testbed::TestBed::poll).
    pub fn set_polling(&mut self, on: bool) {
        self.core.set_polling(on);
    }

    /// Metrics of any live EO.
    pub fn stats(&mut self, eo: EoId) -> Option<EoStats> {
        self.platform.lookup(eo).ok().map(|core| core.stats(false))
    }

    /// Metrics of any live EO, then resets its peaks (min/max time, high-water marks).
    pub fn stats_and_reset_peaks(&mut self, eo: EoId) -> Option<EoStats> {
        self.platform.lookup(eo).ok().map(|core| core.stats(true))
    }

    /// Creates an EO, on the pool named in `cfg` (the test bed ignores pools).
    /// Its memory is allocated now, on this thread, and its creation cost is
    /// in its [`CreationStats`](crate::CreationStats).
    pub fn spawn(&mut self, cfg: EoConfig, eo: impl ExecutionObject) -> Result<EoId> {
        self.platform.spawn(&cfg, Box::new(eo))
    }

    /// Asks an EO, this one included, to stop: its `on_stop` runs next
    /// (system messages come first), then it is removed.
    pub fn stop(&mut self, id: EoId) -> Result<()> {
        post_stop(self.platform, id)
    }

    /// Live and faulted EOs. Allocates: for supervision, not for the message path.
    pub fn eos(&self) -> Vec<EoInfo> {
        let platform = &*self.platform;
        platform.registry().infos(|pool| platform.pool_name(pool))
    }

    /// First live EO with this name, e.g. a well-known service. Scans the
    /// registry: cache the result.
    pub fn find(&self, name: &str) -> Option<EoId> {
        self.platform.registry().find(name)
    }

    /// Activity of the workers (empty on the test bed). Allocates.
    pub fn worker_stats(&self) -> Vec<WorkerStats> {
        self.platform.worker_stats()
    }
}

impl std::fmt::Debug for Context<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context").field("self_id", &self.core.id).finish_non_exhaustive()
    }
}
