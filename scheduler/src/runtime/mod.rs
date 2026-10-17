//! Multi-core runtime: pools of worker threads, each worker a reactor.
//!
//! An EO belongs to one pool and runs on any worker of that pool, never on
//! two at once. Each pool has a shared run queue of EO indices and one epoll
//! instance for the sockets of its EOs; each worker has its own timer wheel
//! and sleeps in `epoll_wait` when there is nothing to run.

mod worker;

use std::os::fd::OwnedFd;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering, fence};
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use rustix::event::{EventfdFlags, eventfd};

use crate::context::{Poster, check_user_msg_id, post_signal, post_stop, post_value};
use crate::core::EoCore;
use crate::failure::{self, FailureReport, FaultPolicy};
use crate::io::Poller;
use crate::queue::Ring;
use crate::registry::Registry;
use crate::stats::{WorkerCounters, WorkerStats};
use crate::timer::{Time, WheelHandle};
use crate::{EoConfig, EoId, EoStats, Error, ExecutionObject, MsgBody, Result};

/// Workers of one pool.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolConfig {
    /// Name, referenced by [`EoConfig::pool`].
    pub name: String,
    /// Number of worker threads, at most 64.
    pub workers: usize,
    /// Core each worker is pinned to; empty for unpinned workers.
    pub cores: Vec<usize>,
    /// Messages an EO handles per turn before the worker moves to the next EO.
    pub budget: usize,
}

impl PoolConfig {
    /// `workers` unpinned workers, budget 16.
    pub fn new(name: impl Into<String>, workers: usize) -> Self {
        PoolConfig { name: name.into(), workers, cores: Vec::new(), budget: 16 }
    }

    /// One worker pinned to each of `cores`, budget 16.
    pub fn pinned(name: impl Into<String>, cores: &[usize]) -> Self {
        PoolConfig { name: name.into(), workers: cores.len(), cores: cores.to_vec(), budget: 16 }
    }

    /// Sets [`budget`](Self::budget).
    pub fn budget(mut self, messages: usize) -> Self {
        self.budget = messages;
        self
    }
}

/// Sizes and policies of a [`Runtime`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeConfig {
    /// Pools; EOs without a pool go to the first one.
    pub pools: Vec<PoolConfig>,
    /// Registry capacity.
    pub max_eos: u32,
    /// Timer resolution.
    pub tick: Duration,
    /// Buckets of each worker's timer wheel.
    pub wheel_buckets: usize,
    /// Timers pending at once in each worker's wheel.
    pub timers_per_worker: u32,
    /// Cancellations from other workers waiting to be applied to a wheel.
    pub cancel_queue: usize,
    /// Readiness events collected per `epoll_wait`.
    pub io_batch: usize,
    /// What a handler panic does; `Propagate` is for the test bed only.
    pub fault_policy: FaultPolicy,
}

impl RuntimeConfig {
    /// Defaults: 256 EOs, 1 ms tick, 1024 buckets, 16384 timers and 1024
    /// queued cancellations per worker, 64 events per poll, abort on panic.
    pub fn new(pools: Vec<PoolConfig>) -> Self {
        RuntimeConfig {
            pools,
            max_eos: 256,
            tick: Duration::from_millis(1),
            wheel_buckets: 1024,
            timers_per_worker: 16384,
            cancel_queue: 1024,
            io_batch: 64,
            fault_policy: FaultPolicy::Abort,
        }
    }

    fn validate(&self) -> Result<()> {
        if self.pools.is_empty() || self.pools.len() > usize::from(u16::MAX) {
            return Err(Error::InvalidConfig("at least one pool is needed"));
        }
        for (i, p) in self.pools.iter().enumerate() {
            if p.workers == 0 || p.workers > 64 {
                return Err(Error::InvalidConfig("a pool has 1 to 64 workers"));
            }
            if !p.cores.is_empty() && p.cores.len() != p.workers {
                return Err(Error::InvalidConfig("pinned pools give one core per worker"));
            }
            if p.budget == 0 {
                return Err(Error::InvalidConfig("budget must be > 0"));
            }
            if self.pools[..i].iter().any(|q| q.name == p.name) {
                return Err(Error::InvalidConfig("pool names must be unique"));
            }
        }
        if self.pools.iter().map(|p| p.workers).sum::<usize>() > usize::from(u16::MAX) {
            return Err(Error::InvalidConfig("too many workers"));
        }
        if self.max_eos == 0 || self.cancel_queue == 0 || self.io_batch == 0 {
            return Err(Error::InvalidConfig("max_eos, cancel_queue and io_batch must be > 0"));
        }
        if self.fault_policy == FaultPolicy::Propagate {
            return Err(Error::InvalidConfig("FaultPolicy::Propagate is for the test bed"));
        }
        Ok(())
    }
}

pub(crate) struct Pool {
    name: String,
    budget: usize,
    run_queue: Ring<u32>,
    /// Bit `i` set: worker `first_worker + i` is (about to be) asleep.
    idle: AtomicU64,
    first_worker: usize,
    poller: Poller,
    /// One worker at a time collects the pool's readiness events.
    harvesting: AtomicBool,
    /// A sleeping worker watches the pool's poller.
    io_leader: AtomicBool,
}

pub(crate) struct WorkerShared {
    pool: u16,
    index_in_pool: usize,
    core: Option<usize>,
    wake: OwnedFd,
    cancels: Ring<WheelHandle>,
    counters: WorkerCounters,
}

pub(crate) struct Shared {
    registry: Registry,
    pools: Box<[Pool]>,
    workers: Box<[WorkerShared]>,
    start: Instant,
    shutdown: AtomicBool,
    config: RuntimeConfig,
}

impl Shared {
    fn now(&self) -> Time {
        Time::from_nanos(self.start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64)
    }

    /// Queues an EO whose post returned "must schedule" and wakes a sleeping
    /// worker of its pool, if any.
    fn schedule(&self, core: &EoCore) {
        let pool = self.enqueue(core);
        self.wake_one(pool);
    }

    /// Queues an EO again at the end of its turn. The worker doing it pops the
    /// next EO right away, so another worker is only woken if more than one
    /// EO is waiting.
    fn requeue(&self, core: &EoCore) {
        let pool = self.enqueue(core);
        if pool.run_queue.len() > 1 {
            self.wake_one(pool);
        }
    }

    fn enqueue(&self, core: &EoCore) -> &Pool {
        let pool = &self.pools[usize::from(core.pool)];
        let queued = pool.run_queue.push(core.id.index());
        debug_assert!(queued.is_ok(), "an EO is queued at most once; the run queue holds them all");
        // Pairs with the fence in `Worker::sleep`: either the worker sees the
        // queued EO, or we see its idle bit.
        fence(Ordering::SeqCst);
        pool
    }

    fn wake_one(&self, pool: &Pool) {
        loop {
            let idle = pool.idle.load(Ordering::Relaxed);
            if idle == 0 {
                return;
            }
            let bit = 1u64 << idle.trailing_zeros();
            if pool.idle.fetch_and(!bit, Ordering::AcqRel) & bit != 0 {
                self.wake(pool.first_worker + idle.trailing_zeros() as usize);
                return;
            }
        }
    }

    fn spawn(&self, cfg: &EoConfig, eo: Box<dyn ExecutionObject>) -> Result<EoId> {
        let pool = match &cfg.pool {
            None => 0,
            Some(name) => {
                self.pools.iter().position(|p| p.name == *name).ok_or(Error::InvalidConfig("unknown pool"))?
            }
        };
        let (core, must_schedule) = self.registry.create(cfg, pool as u16, eo)?;
        if must_schedule {
            self.schedule(&core);
        }
        Ok(core.id)
    }

    fn worker_stats(&self) -> Vec<WorkerStats> {
        let now = self.now().as_nanos();
        self.workers
            .iter()
            .map(|w| w.counters.snapshot(&self.pools[usize::from(w.pool)].name, w.index_in_pool, w.core, now))
            .collect()
    }

    fn pool_name(&self, pool: u16) -> &str {
        &self.pools[usize::from(pool)].name
    }

    fn wake(&self, worker: usize) {
        // Only fails if the counter would overflow, i.e. the worker is already awake.
        let _ = rustix::io::write(&self.workers[worker].wake, &1u64.to_ne_bytes());
    }
}

/// Posts from outside any worker (the thread owning the [`Runtime`]).
struct Control<'a>(&'a Shared);

impl Poster for Control<'_> {
    fn now(&self) -> Time {
        self.0.now()
    }

    fn lookup(&mut self, id: EoId) -> Result<Arc<EoCore>> {
        let core = self.0.registry.get(id).ok_or(Error::UnknownEo)?;
        core.check_alive()?;
        Ok(core)
    }

    fn schedule(&mut self, core: &EoCore) {
        self.0.schedule(core);
    }
}

/// The multi-core platform.
///
/// Every method takes `&self`: the handle can be shared between threads, for
/// example to inject traffic from several of them.
pub struct Runtime {
    shared: Arc<Shared>,
    threads: Vec<JoinHandle<()>>,
}

impl Runtime {
    /// Starts every worker; fails if a worker cannot be created or pinned.
    pub fn new(config: RuntimeConfig) -> Result<Self> {
        config.validate()?;
        failure::install_panic_hook();
        let mut pools = Vec::with_capacity(config.pools.len());
        let mut workers = Vec::new();
        for (p, pc) in config.pools.iter().enumerate() {
            pools.push(Pool {
                name: pc.name.clone(),
                budget: pc.budget,
                run_queue: Ring::new(config.max_eos as usize),
                idle: AtomicU64::new(0),
                first_worker: workers.len(),
                poller: Poller::new()?,
                harvesting: AtomicBool::new(false),
                io_leader: AtomicBool::new(false),
            });
            for i in 0..pc.workers {
                workers.push(WorkerShared {
                    pool: p as u16,
                    index_in_pool: i,
                    core: pc.cores.get(i).copied(),
                    wake: eventfd(0, EventfdFlags::CLOEXEC | EventfdFlags::NONBLOCK)?,
                    cancels: Ring::new(config.cancel_queue),
                    counters: WorkerCounters::new(),
                });
            }
        }
        let shared = Arc::new(Shared {
            registry: Registry::new(config.max_eos),
            pools: pools.into_boxed_slice(),
            workers: workers.into_boxed_slice(),
            start: Instant::now(),
            shutdown: AtomicBool::new(false),
            config,
        });

        let (ready_tx, ready_rx) = mpsc::channel();
        let mut runtime = Runtime { shared: shared.clone(), threads: Vec::new() };
        for id in 0..shared.workers.len() {
            let w = &shared.workers[id];
            let name = format!("{}-{}", shared.pools[usize::from(w.pool)].name, w.index_in_pool);
            let (shared, ready) = (shared.clone(), ready_tx.clone());
            let spawned = std::thread::Builder::new().name(name).spawn(move || match worker::Worker::new(shared, id) {
                Ok(worker) => {
                    let _ = ready.send(Ok(()));
                    worker.run();
                }
                Err(e) => {
                    let _ = ready.send(Err(e));
                }
            });
            match spawned {
                Ok(handle) => runtime.threads.push(handle),
                Err(e) => {
                    runtime.shutdown(Duration::ZERO);
                    return Err(e.into());
                }
            }
        }
        drop(ready_tx);
        for _ in 0..runtime.threads.len() {
            if let Err(e) = ready_rx.recv().unwrap_or(Err(Error::InvalidConfig("worker died during start"))) {
                runtime.shutdown(Duration::ZERO);
                return Err(e);
            }
        }
        Ok(runtime)
    }

    /// Creates an EO on the pool named in `cfg` (default: the first pool).
    /// The allocation happens on the calling thread; workers only see the EO
    /// once it is fully built. Works before and while traffic flows.
    pub fn add(&self, cfg: EoConfig, eo: impl ExecutionObject) -> Result<EoId> {
        self.shared.spawn(&cfg, Box::new(eo))
    }

    /// Sends `value` to `to` from [`EoId::SYSTEM`].
    pub fn inject<T: MsgBody>(&self, to: EoId, value: &T) -> Result<()> {
        self.inject_from(EoId::SYSTEM, to, 0, value)
    }

    /// Sends `value` to `to` as if `from` had sent it.
    pub fn inject_from<T: MsgBody>(&self, from: EoId, to: EoId, corr_id: u32, value: &T) -> Result<()> {
        post_value(&mut Control(&self.shared), from, to, corr_id, value)
    }

    /// Sends a header-only message to `to` from [`EoId::SYSTEM`].
    pub fn signal(&self, to: EoId, msg_id: u32) -> Result<()> {
        check_user_msg_id(msg_id)?;
        post_signal(&mut Control(&self.shared), EoId::SYSTEM, to, msg_id, 0)
    }

    /// Asks `id` to stop, like [`Context::stop`](crate::Context::stop).
    pub fn stop(&self, id: EoId) -> Result<()> {
        post_stop(&mut Control(&self.shared), id)
    }

    /// Time since the runtime started.
    pub fn now(&self) -> Time {
        self.shared.now()
    }

    /// Whether `id` names a live EO. False as soon as it stops accepting
    /// messages (stopping or faulted), possibly before [`alive`](Self::alive)
    /// stops counting it.
    pub fn is_alive(&self, id: EoId) -> bool {
        self.shared.registry.get(id).is_some_and(|core| core.check_alive().is_ok())
    }

    /// EOs not fully retired: an EO leaves the count once `on_stop` ran (or it
    /// faulted) and its timers, sockets and object are released.
    pub fn alive(&self) -> usize {
        self.shared.registry.alive()
    }

    /// Metrics of a live or faulted EO.
    pub fn stats(&self, id: EoId) -> Option<EoStats> {
        self.shared.registry.get(id).map(|core| core.stats(false))
    }

    /// Metrics of a live or faulted EO, then resets its peaks.
    pub fn stats_and_reset_peaks(&self, id: EoId) -> Option<EoStats> {
        self.shared.registry.get(id).map(|core| core.stats(true))
    }

    /// Report of an EO isolated by [`FaultPolicy::Isolate`].
    pub fn failure(&self, id: EoId) -> Option<FailureReport> {
        self.shared.registry.get(id)?.fault().cloned()
    }

    /// Activity of every worker, pool by pool.
    pub fn worker_stats(&self) -> Vec<WorkerStats> {
        self.shared.worker_stats()
    }

    /// Stops every EO in reverse creation order, each one finishing (its
    /// `on_stop` included) before the next is asked, within `timeout`
    /// overall; then stops the workers. Returns whether every EO stopped in
    /// time. Called by `Drop` with a 5 s timeout if not called before.
    pub fn shutdown(&mut self, timeout: Duration) -> bool {
        if self.threads.is_empty() {
            return self.shared.registry.alive() == 0;
        }
        let deadline = Instant::now() + timeout;
        let pause = || std::thread::sleep(Duration::from_micros(100));
        let registry = &self.shared.registry;
        // Last created, first stopped: services created first (logs, ...)
        // still serve the `on_stop` of those created after them.
        for id in registry.alive_ids().into_iter().rev() {
            while post_stop(&mut Control(&self.shared), id) == Err(Error::QueueFull) && Instant::now() < deadline {
                pause();
            }
            while registry.get(id).is_some_and(|core| core.fault().is_none()) && Instant::now() < deadline {
                pause();
            }
        }
        let clean = self.shared.registry.alive() == 0;
        self.shared.shutdown.store(true, Ordering::Release);
        for w in 0..self.shared.workers.len() {
            self.shared.wake(w);
        }
        for handle in self.threads.drain(..) {
            let _ = handle.join();
        }
        clean
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.shutdown(Duration::from_secs(5));
    }
}

impl std::fmt::Debug for Runtime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runtime")
            .field("pools", &self.shared.pools.iter().map(|p| &p.name).collect::<Vec<_>>())
            .field("workers", &self.shared.workers.len())
            .field("alive", &self.shared.registry.alive())
            .finish()
    }
}
