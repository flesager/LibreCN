//! One worker thread: a reactor loop over its pool's run queue, its own timer
//! wheel and the pool's readiness events.

use std::sync::Arc;
use std::sync::atomic::{Ordering, fence};
use std::time::{Duration, Instant};

use rustix::event::epoll;
use rustix::thread::{CpuSet, sched_setaffinity};

use super::{Pool, Shared};
use crate::context::{Platform, Poster};
use crate::core::EoCore;
use crate::exec::{self, TurnEnd};
use crate::io::{self, Poller};
use crate::stats::WorkerCounters;
use crate::timer::{Expired, Time, TimerId, TimerWheel, WheelHandle};
use crate::{EoConfig, EoId, Error, ExecutionObject, Result};

/// epoll data of the worker's own descriptors.
const WAKE: u64 = 0;
const POOL_IO: u64 = 1;

// One sleeping worker per pool, the I/O leader, also watches the pool's
// poller; the others only their wake-up eventfd, so a readiness event wakes
// one worker, not all of them. An awake worker always polls the pool before
// sleeping, so readiness is never left unwatched: either a worker is awake
// and will poll, or the last one to fall asleep takes the leadership.

pub(crate) struct Worker {
    shared: Arc<Shared>,
    id: usize,
    pool: usize,
    bit: u64,
    /// Sleeps here when another worker leads I/O: the wake eventfd.
    sleep_plain: Poller,
    /// Sleeps here as I/O leader: the wake eventfd and the pool's poller.
    sleep_io: Poller,
    wheel: TimerWheel,
    retry: Vec<Expired>,
    /// EOs by registry index, so the message path skips the registry lock.
    cache: Vec<Option<Arc<EoCore>>>,
    io_events: Vec<epoll::Event>,
    wake_events: Vec<epoll::Event>,
}

/// A live-or-not EO by exact id, through the cache.
fn cached(shared: &Shared, cache: &mut [Option<Arc<EoCore>>], id: EoId) -> Option<Arc<EoCore>> {
    let slot = cache.get_mut(id.index() as usize)?;
    if let Some(core) = slot.as_ref().filter(|c| c.id == id) {
        return Some(core.clone());
    }
    let core = shared.registry.get(id);
    slot.clone_from(&core);
    core
}

/// The EO occupying registry slot `index`, through the cache.
fn cached_by_index(shared: &Shared, cache: &mut [Option<Arc<EoCore>>], index: u32) -> Option<Arc<EoCore>> {
    let slot = cache.get_mut(index as usize)?;
    // A closed EO's slot may have been reused: ask the registry again.
    if let Some(core) = slot.as_ref().filter(|c| !c.is_closed()) {
        return Some(core.clone());
    }
    let core = shared.registry.by_index(index);
    slot.clone_from(&core);
    core
}

impl Worker {
    pub(crate) fn new(shared: Arc<Shared>, id: usize) -> Result<Self> {
        let me = &shared.workers[id];
        if let Some(core) = me.core {
            let mut set = CpuSet::new();
            set.set(core);
            sched_setaffinity(None, &set)?;
        }
        let pool = usize::from(me.pool);
        let config = &shared.config;
        let wake = rustix::fd::AsFd::as_fd(&me.wake);
        let sleep_plain = Poller::new()?;
        sleep_plain.add_level(wake, WAKE)?;
        let sleep_io = Poller::new()?;
        sleep_io.add_level(wake, WAKE)?;
        sleep_io.add_level(shared.pools[pool].poller.as_fd(), POOL_IO)?;
        let wheel = TimerWheel::new(config.timers_per_worker, config.wheel_buckets, config.tick, id as u16)?;
        let bit = 1u64 << (id - shared.pools[pool].first_worker);
        Ok(Worker {
            id,
            pool,
            bit,
            sleep_plain,
            sleep_io,
            wheel,
            retry: Vec::with_capacity(config.timers_per_worker as usize),
            cache: vec![None; config.max_eos as usize],
            io_events: Vec::with_capacity(config.io_batch),
            wake_events: Vec::with_capacity(4),
            shared,
        })
    }

    fn counters(&self) -> &WorkerCounters {
        &self.shared.workers[self.id].counters
    }

    pub(crate) fn run(mut self) {
        let shared = self.shared.clone();
        let pool = &shared.pools[self.pool];
        // While the run queue stays busy, readiness is polled once per tick.
        let tick = self.wheel.tick();
        let mut last_poll = Time::ZERO;
        while !shared.shutdown.load(Ordering::Acquire) {
            // Timers are checked before every turn: a long turn on another
            // EO must not delay the expiries this worker's wheel holds.
            self.apply_cancels();
            let now = self.fire_timers();
            if now - last_poll >= tick {
                self.harvest_io(pool);
                last_poll = now;
            }
            if let Some(index) = pool.run_queue.pop() {
                self.run_turn(index, pool.budget);
                continue;
            }
            // Always poll right before sleeping (see the I/O leader note).
            if self.harvest_io(pool) == 0 {
                self.sleep(pool);
            }
            last_poll = self.shared.now();
        }
    }

    fn run_turn(&mut self, index: u32, budget: usize) {
        let Some(core) = cached_by_index(&self.shared, &mut self.cache, index) else { return };
        let started = Instant::now();
        let policy = self.shared.config.fault_policy;
        let (handled, end) = exec::run_turn(self, &core, budget, policy);
        let counters = self.counters();
        WorkerCounters::add_time(&counters.busy_ns, started.elapsed());
        WorkerCounters::add(&counters.turns, 1);
        WorkerCounters::add(&counters.messages, handled as u64);
        // A polling EO gets its next `EO_POLL`, which keeps it runnable.
        let polling = !matches!(end, TurnEnd::Retired) && core.is_polling();
        let woken = polling && exec::post_poll(&core);
        if matches!(end, TurnEnd::Requeue) || woken {
            self.shared.requeue(&core);
        }
    }

    /// Removes the wheel entries other workers cancelled.
    fn apply_cancels(&mut self) {
        while let Some(handle) = self.shared.workers[self.id].cancels.pop() {
            self.wheel.remove(handle);
        }
    }

    /// Posts the expiries due now; returns the time it used.
    fn fire_timers(&mut self) -> Time {
        let now = self.shared.now();
        let shared = &self.shared;
        let cache = &mut self.cache;
        let retry = &mut self.retry;
        let mut fired = 0;
        self.wheel.expire(now, |expired| {
            fired += 1;
            let Some(core) = cached(shared, cache, expired.eo) else { return };
            match core.post_system(exec::expiry_header(&expired)) {
                Ok(true) => shared.schedule(&core),
                Ok(false) => {}
                Err(Error::QueueFull) => retry.push(expired),
                // The EO is gone: so is the timer.
                Err(_) => {}
            }
        });
        let tick = self.wheel.tick();
        for e in self.retry.drain(..) {
            let _ = self.wheel.insert(now, tick, e.eo, e.timer, e.tag);
        }
        if fired > 0 {
            WorkerCounters::add(&self.counters().timers_fired, fired);
        }
        now
    }

    /// Collects the pool's readiness events, unless another worker is at it.
    fn harvest_io(&mut self, pool: &Pool) -> usize {
        if pool.harvesting.swap(true, Ordering::Acquire) {
            return 0;
        }
        let shared = &self.shared;
        let cache = &mut self.cache;
        let events = pool
            .poller
            .wait(&mut self.io_events, Some(Duration::ZERO), |data, readiness| {
                let (eo_index, token) = io::decode(data);
                if let Some(core) = cached_by_index(shared, cache, eo_index)
                    && exec::post_readiness(&core, token, readiness)
                {
                    shared.schedule(&core);
                }
            })
            .unwrap_or(0);
        pool.harvesting.store(false, Ordering::Release);
        WorkerCounters::add(&self.counters().io_events, events as u64);
        events
    }

    /// Sleeps until woken by a scheduler, a readiness event or the next timer.
    fn sleep(&mut self, pool: &Pool) {
        pool.idle.fetch_or(self.bit, Ordering::SeqCst);
        // Pairs with the fence in `Shared::schedule`.
        fence(Ordering::SeqCst);
        if !pool.run_queue.is_empty() || self.shared.shutdown.load(Ordering::Acquire) {
            pool.idle.fetch_and(!self.bit, Ordering::SeqCst);
            return;
        }
        let timeout = self.wheel.next_deadline().map(|deadline| deadline - self.shared.now());
        let started = self.shared.now();
        // Platform time + 1 while asleep, so a snapshot can count the current sleep.
        self.counters().asleep_since.store(started.as_nanos() + 1, Ordering::Relaxed);
        let wake = &self.shared.workers[self.id].wake;
        let leader = !pool.io_leader.swap(true, Ordering::AcqRel);
        let epoll = if leader { &self.sleep_io } else { &self.sleep_plain };
        let _ = epoll.wait(&mut self.wake_events, timeout, |data, _| {
            if data == WAKE {
                let mut counter = [0u8; 8];
                let _ = rustix::io::read(wake, &mut counter);
            }
        });
        if leader {
            pool.io_leader.store(false, Ordering::Release);
        }
        pool.idle.fetch_and(!self.bit, Ordering::SeqCst);
        let slept = self.shared.now() - started;
        let counters = self.counters();
        counters.asleep_since.store(0, Ordering::Relaxed);
        WorkerCounters::add_time(&counters.idle_ns, slept);
        WorkerCounters::add(&counters.wakeups, 1);
    }
}

impl Poster for Worker {
    fn now(&self) -> Time {
        self.shared.now()
    }

    fn lookup(&mut self, id: EoId) -> Result<Arc<EoCore>> {
        let core = cached(&self.shared, &mut self.cache, id).ok_or(Error::UnknownEo)?;
        core.check_alive()?;
        Ok(core)
    }

    fn schedule(&mut self, core: &EoCore) {
        self.shared.schedule(core);
    }
}

impl Platform for Worker {
    fn arm_wheel(&mut self, eo: EoId, after: Duration, timer: TimerId, tag: u32) -> Result<WheelHandle> {
        self.wheel.insert(self.shared.now(), after, eo, timer, tag)
    }

    fn cancel_wheel(&mut self, handle: WheelHandle) {
        let owner = usize::from(handle.worker);
        if owner == self.id {
            self.wheel.remove(handle);
        } else if self.shared.workers[owner].cancels.push(handle).is_ok() {
            WorkerCounters::add(&self.counters().remote_cancels, 1);
        } else {
            // The entry stays until it expires; the timer table makes it stale.
            WorkerCounters::add(&self.counters().lazy_cancels, 1);
        }
    }

    fn poller(&self, pool: u16) -> &Poller {
        &self.shared.pools[usize::from(pool)].poller
    }

    fn retired(&mut self, core: &EoCore, faulted: bool) {
        self.shared.registry.retired();
        if !faulted {
            self.shared.registry.remove(core.id);
        }
    }

    fn spawn(&mut self, cfg: &EoConfig, eo: Box<dyn ExecutionObject>) -> Result<EoId> {
        self.shared.spawn(cfg, eo)
    }
}
