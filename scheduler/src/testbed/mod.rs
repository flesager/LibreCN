//! Single-threaded fake platform with a virtual clock.
//!
//! Runs real execution objects, unchanged, deterministically: messages are
//! processed in scheduling order, timers fire only when [`TestBed::advance`]
//! moves the clock, readiness is collected only by [`TestBed::poll_io`], and
//! nothing depends on wall-clock time (processing time is still measured for
//! real, for the metrics).

mod probe;

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

pub use probe::{Probe, ProbeLog, Received};
use rustix::event::epoll;

use crate::context::{Platform, Poster, check_user_msg_id, post_signal, post_stop, post_value};
use crate::core::EoCore;
use crate::exec::{self, TurnEnd};
use crate::failure::{self, FailureReport, FaultPolicy};
use crate::io::{self, Poller};
use crate::registry::Registry;
use crate::timer::{Expired, Time, TimerId, TimerWheel, WheelHandle};
use crate::{EoConfig, EoId, EoStats, Error, ExecutionObject, MsgBody, Result, WorkerStats};

/// Sizes and policies of a [`TestBed`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TestBedConfig {
    /// Registry capacity.
    pub max_eos: u32,
    /// Timers pending at once, all EOs together.
    pub max_timers: u32,
    /// Buckets of the timer wheel.
    pub wheel_buckets: usize,
    /// Timer resolution.
    pub tick: Duration,
    /// Messages an EO handles per turn before the next EO gets its turn.
    pub budget: usize,
    /// What a handler panic does.
    pub fault_policy: FaultPolicy,
}

impl Default for TestBedConfig {
    fn default() -> Self {
        TestBedConfig {
            max_eos: 64,
            max_timers: 4096,
            wheel_buckets: 1024,
            tick: Duration::from_millis(1),
            budget: 16,
            fault_policy: FaultPolicy::Propagate,
        }
    }
}

/// Single-threaded platform hosting real EOs, with a virtual clock.
pub struct TestBed {
    config: TestBedConfig,
    clock: Time,
    registry: Registry,
    run_queue: VecDeque<u32>,
    wheel: TimerWheel,
    /// Expiries refused by a full system lane, re-armed for the next tick.
    retry: Vec<Expired>,
    poller: Poller,
    events: Vec<epoll::Event>,
}

impl Default for TestBed {
    fn default() -> Self {
        TestBed::new()
    }
}

impl TestBed {
    /// Test bed with [`TestBedConfig::default`].
    pub fn new() -> Self {
        TestBed::with_config(TestBedConfig::default())
    }

    /// # Panics
    /// If the configuration is invalid or epoll is unavailable.
    pub fn with_config(config: TestBedConfig) -> Self {
        assert!(config.max_eos > 0 && config.budget > 0, "max_eos and budget must be > 0");
        failure::install_panic_hook();
        let wheel =
            TimerWheel::new(config.max_timers, config.wheel_buckets, config.tick, 0).expect("invalid timer config");
        TestBed {
            config,
            clock: Time::ZERO,
            registry: Registry::new(config.max_eos),
            run_queue: VecDeque::with_capacity(config.max_eos as usize),
            wheel,
            retry: Vec::with_capacity(config.max_timers as usize),
            poller: Poller::new().expect("epoll"),
            events: Vec::with_capacity(64),
        }
    }

    /// Creates an EO; its `on_start` runs on the next `run_until_idle`.
    pub fn add(&mut self, cfg: EoConfig, eo: impl ExecutionObject) -> Result<EoId> {
        self.spawn(&cfg, Box::new(eo))
    }

    /// Sends `value` to `to` from [`EoId::SYSTEM`].
    pub fn inject<T: MsgBody>(&mut self, to: EoId, value: &T) -> Result<()> {
        self.inject_from(EoId::SYSTEM, to, 0, value)
    }

    /// Sends `value` to `to` as if `from` had sent it.
    pub fn inject_from<T: MsgBody>(&mut self, from: EoId, to: EoId, corr_id: u32, value: &T) -> Result<()> {
        post_value(self, from, to, corr_id, value)
    }

    /// Sends a header-only message to `to` from [`EoId::SYSTEM`].
    pub fn signal(&mut self, to: EoId, msg_id: u32) -> Result<()> {
        check_user_msg_id(msg_id)?;
        post_signal(self, EoId::SYSTEM, to, msg_id, 0)
    }

    /// Asks `id` to stop, like [`Context::stop`](crate::Context::stop).
    pub fn stop(&mut self, id: EoId) -> Result<()> {
        post_stop(self, id)
    }

    /// Runs one scheduling turn (one EO, up to `budget` messages). Returns
    /// the messages handled, `None` if nothing was scheduled.
    pub fn step(&mut self) -> Option<usize> {
        let index = self.run_queue.pop_front()?;
        let Some(core) = self.registry.by_index(index) else { return Some(0) };
        let TestBedConfig { budget, fault_policy, .. } = self.config;
        let (handled, end) = exec::run_turn(self, &core, budget, fault_policy);
        if let TurnEnd::Requeue = end {
            self.schedule(&core);
        }
        Some(handled)
    }

    /// Runs turns until no EO has a message waiting. Returns the messages handled.
    /// Never returns if EOs keep messaging each other forever.
    pub fn run_until_idle(&mut self) -> usize {
        let mut handled = 0;
        while let Some(n) = self.step() {
            handled += n;
        }
        handled
    }

    /// Moves the virtual clock forward by `by`, firing timers in deadline
    /// order and running EOs at each deadline, so timers armed on the way
    /// fire too if they fall inside the window. Returns the messages handled.
    pub fn advance(&mut self, by: Duration) -> usize {
        let target = self.clock + by;
        let mut handled = self.run_until_idle();
        while let Some(deadline) = self.wheel.next_deadline().filter(|&d| d <= target) {
            self.clock = self.clock.max(deadline);
            self.fire_timers();
            handled += self.run_until_idle();
        }
        self.clock = target;
        self.fire_timers();
        handled + self.run_until_idle()
    }

    /// Moves the virtual clock forward by `by` and queues every expired timer,
    /// **without running any EO**. Lower-level than [`advance`](Self::advance):
    /// use [`step`](Self::step) or [`run_until_idle`](Self::run_until_idle) to
    /// process what was queued.
    pub fn advance_clock(&mut self, by: Duration) {
        self.clock = self.clock + by;
        self.fire_timers();
    }

    /// Queues `IO_READY` for every registered descriptor that is ready now,
    /// without running any EO. Returns the readiness events collected.
    pub fn poll_io(&mut self) -> usize {
        self.wait_io(Duration::ZERO)
    }

    /// Like [`poll_io`](Self::poll_io), waiting up to `timeout` for a first event.
    pub fn wait_io(&mut self, timeout: Duration) -> usize {
        let registry = &self.registry;
        let run_queue = &mut self.run_queue;
        self.poller
            .wait(&mut self.events, Some(timeout), |data, readiness| {
                let (eo_index, token) = io::decode(data);
                if let Some(core) = registry.by_index(eo_index)
                    && exec::post_readiness(&core, token, readiness)
                {
                    run_queue.push_back(eo_index);
                }
            })
            .expect("epoll_wait")
    }

    /// Queues one `EO_POLL` for every polling EO, without running any EO.
    /// The runtime does this after each of their turns; the test bed leaves
    /// the cadence to the test. Returns the EOs that poll.
    pub fn poll(&mut self) -> usize {
        let mut polling = 0;
        for index in 0..self.registry.capacity() {
            let Some(core) = self.registry.by_index(index) else { continue };
            if core.is_polling() && core.check_alive().is_ok() {
                polling += 1;
                if exec::post_poll(&core) {
                    self.schedule(&core);
                }
            }
        }
        polling
    }

    /// Virtual time.
    pub fn now(&self) -> Time {
        self.clock
    }

    /// Whether `id` names a live (not stopped, not faulted) EO.
    pub fn is_alive(&self, id: EoId) -> bool {
        self.registry.get(id).is_some_and(|core| core.check_alive().is_ok())
    }

    /// Metrics of a live or faulted EO.
    pub fn stats(&self, id: EoId) -> Option<EoStats> {
        self.registry.get(id).map(|core| core.stats(false))
    }

    /// Metrics of a live or faulted EO, then resets its peaks.
    pub fn stats_and_reset_peaks(&self, id: EoId) -> Option<EoStats> {
        self.registry.get(id).map(|core| core.stats(true))
    }

    /// Report of an EO isolated by [`FaultPolicy::Isolate`].
    pub fn failure(&self, id: EoId) -> Option<FailureReport> {
        self.registry.get(id)?.fault().cloned()
    }

    /// Timers pending in the wheel.
    pub fn pending_timers(&self) -> usize {
        self.wheel.len()
    }

    fn fire_timers(&mut self) {
        let registry = &self.registry;
        let run_queue = &mut self.run_queue;
        let retry = &mut self.retry;
        self.wheel.expire(self.clock, |expired| {
            let Some(core) = registry.get(expired.eo) else { return };
            match core.post_system(exec::expiry_header(&expired)) {
                Ok(true) => run_queue.push_back(expired.eo.index()),
                Ok(false) => {}
                Err(Error::QueueFull) => retry.push(expired),
                // The EO is gone: so is the timer.
                Err(_) => {}
            }
        });
        let tick = self.wheel.tick();
        for e in self.retry.drain(..) {
            let _ = self.wheel.insert(self.clock, tick, e.eo, e.timer, e.tag);
        }
    }
}

impl Poster for TestBed {
    fn now(&self) -> Time {
        self.clock
    }

    fn lookup(&mut self, id: EoId) -> Result<Arc<EoCore>> {
        let core = self.registry.get(id).ok_or(Error::UnknownEo)?;
        core.check_alive()?;
        Ok(core)
    }

    fn schedule(&mut self, core: &EoCore) {
        debug_assert!(self.run_queue.len() < self.run_queue.capacity(), "an EO is queued at most once");
        self.run_queue.push_back(core.id.index());
    }
}

impl Platform for TestBed {
    fn arm_wheel(&mut self, eo: EoId, after: Duration, timer: TimerId, tag: u32) -> Result<WheelHandle> {
        self.wheel.insert(self.clock, after, eo, timer, tag)
    }

    fn cancel_wheel(&mut self, handle: WheelHandle) {
        self.wheel.remove(handle);
    }

    fn poller(&self, _pool: u16) -> &Poller {
        &self.poller
    }

    fn retired(&mut self, core: &EoCore, faulted: bool) {
        self.registry.retired();
        if !faulted {
            self.registry.remove(core.id);
        }
    }

    fn spawn(&mut self, cfg: &EoConfig, eo: Box<dyn ExecutionObject>) -> Result<EoId> {
        let (core, must_schedule) = self.registry.create(cfg, 0, eo)?;
        if must_schedule {
            self.schedule(&core);
        }
        Ok(core.id)
    }

    fn registry(&self) -> &Registry {
        &self.registry
    }

    fn pool_name(&self, _pool: u16) -> &str {
        ""
    }

    fn worker_stats(&self) -> Vec<WorkerStats> {
        Vec::new()
    }
}

impl std::fmt::Debug for TestBed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TestBed")
            .field("clock", &self.clock)
            .field("alive", &self.registry.alive())
            .field("scheduled", &self.run_queue.len())
            .field("pending_timers", &self.wheel.len())
            .finish()
    }
}
