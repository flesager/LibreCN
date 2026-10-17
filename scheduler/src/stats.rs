//! Per-EO counters and the snapshots exposed to the metric loop.
//!
//! Counters are atomics written by whoever runs the EO (or by senders, for
//! lane rejections) and read by anyone without locking. A snapshot taken
//! while the EO runs is consistent per counter, not across counters.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Number of buckets in [`ProcessingStats::histogram`].
pub const HIST_BUCKETS: usize = 24;

/// Bucket of a processing time: 0 is below 1 µs, bucket `i` in `1..HIST_BUCKETS-1`
/// covers `[2^(i-1), 2^i)` µs, the last bucket everything from 2^22 µs (~4.2 s).
pub fn histogram_bucket(elapsed: Duration) -> usize {
    let us = elapsed.as_micros().min(u128::from(u64::MAX)) as u64;
    if us == 0 { 0 } else { ((u64::BITS - us.leading_zeros()) as usize).min(HIST_BUCKETS - 1) }
}

/// Lower bound of histogram bucket `i`.
pub fn histogram_bucket_floor(i: usize) -> Duration {
    if i == 0 { Duration::ZERO } else { Duration::from_micros(1 << (i - 1)) }
}

pub(crate) struct EoCounters {
    processed: AtomicU64,
    total_ns: AtomicU64,
    min_ns: AtomicU64,
    max_ns: AtomicU64,
    histogram: [AtomicU64; HIST_BUCKETS],
    pub(crate) runs: AtomicU64,
    pub(crate) budget_exhausted: AtomicU64,
    pub(crate) timers_fired: AtomicU64,
    pub(crate) timers_stale: AtomicU64,
    pub(crate) io_events: AtomicU64,
    pub(crate) io_stale: AtomicU64,
}

impl EoCounters {
    pub(crate) fn new() -> Self {
        EoCounters {
            processed: AtomicU64::new(0),
            total_ns: AtomicU64::new(0),
            min_ns: AtomicU64::new(u64::MAX),
            max_ns: AtomicU64::new(0),
            histogram: std::array::from_fn(|_| AtomicU64::new(0)),
            runs: AtomicU64::new(0),
            budget_exhausted: AtomicU64::new(0),
            timers_fired: AtomicU64::new(0),
            timers_stale: AtomicU64::new(0),
            io_events: AtomicU64::new(0),
            io_stale: AtomicU64::new(0),
        }
    }

    /// Records the processing time of one message.
    pub(crate) fn record(&self, elapsed: Duration) {
        let ns = elapsed.as_nanos().min(u128::from(u64::MAX)) as u64;
        self.processed.fetch_add(1, Ordering::Relaxed);
        self.total_ns.fetch_add(ns, Ordering::Relaxed);
        self.min_ns.fetch_min(ns, Ordering::Relaxed);
        self.max_ns.fetch_max(ns, Ordering::Relaxed);
        self.histogram[histogram_bucket(elapsed)].fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn processing(&self, reset_peaks: bool) -> ProcessingStats {
        let (min, max) = if reset_peaks {
            (self.min_ns.swap(u64::MAX, Ordering::Relaxed), self.max_ns.swap(0, Ordering::Relaxed))
        } else {
            (self.min_ns.load(Ordering::Relaxed), self.max_ns.load(Ordering::Relaxed))
        };
        ProcessingStats {
            count: self.processed.load(Ordering::Relaxed),
            total: Duration::from_nanos(self.total_ns.load(Ordering::Relaxed)),
            min: (min != u64::MAX).then(|| Duration::from_nanos(min)),
            max: Duration::from_nanos(max),
            histogram: std::array::from_fn(|i| self.histogram[i].load(Ordering::Relaxed)),
        }
    }
}

/// Processing time of the messages handled by one EO.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessingStats {
    /// Messages handled, system messages included. Never reset.
    pub count: u64,
    /// Sum of all processing times. Never reset.
    pub total: Duration,
    /// Shortest processing time since the last peak reset; `None` if nothing ran since.
    pub min: Option<Duration>,
    /// Longest processing time since the last peak reset.
    pub max: Duration,
    /// Count per bucket, see [`histogram_bucket`]. Never reset.
    pub histogram: [u64; HIST_BUCKETS],
}

impl ProcessingStats {
    /// Average processing time.
    pub fn mean(&self) -> Option<Duration> {
        (self.count > 0).then(|| Duration::from_nanos((self.total.as_nanos() / u128::from(self.count)) as u64))
    }
}

/// One lane of a mailbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LaneStats {
    /// Number of slots.
    pub capacity: usize,
    /// Messages waiting now.
    pub depth: usize,
    /// Highest depth since the last peak reset.
    pub high_water: usize,
    /// Pushes refused because the lane was full. Never reset.
    pub rejected: u64,
}

/// One buffer size class of a mailbox's pool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferClassStats {
    /// Buffer size in bytes.
    pub size: usize,
    /// Number of buffers.
    pub count: u32,
    /// Buffers held now (queued or being processed).
    pub in_use: u32,
    /// Highest `in_use` since the last peak reset. Equal to `count` means the
    /// class ran out at least once.
    pub high_water: u32,
    /// Bodies sized for this class that got a buffer from a larger class
    /// because this one was empty. Never reset.
    pub fallbacks: u64,
    /// Bodies sized for this class refused because it and every larger class
    /// were empty. Never reset.
    pub alloc_failures: u64,
}

/// Cost of creating the EO.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreationStats {
    /// Allocation and prefault of every preallocated region.
    pub duration: Duration,
    /// Bytes held by the EO's mailbox, pool, timer table and flight recorder.
    pub bytes_reserved: usize,
}

/// Everything known about one EO.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EoStats {
    /// Name given in the `EoConfig`.
    pub name: String,
    /// Per-message processing time.
    pub processing: ProcessingStats,
    /// Times the EO was picked by a scheduler turn.
    pub runs: u64,
    /// Turns that ended because the budget ran out with messages still waiting.
    pub budget_exhausted: u64,
    /// Timers delivered.
    pub timers_fired: u64,
    /// Expired timers dropped because they were cancelled before delivery.
    pub timers_stale: u64,
    /// Readiness messages delivered.
    pub io_events: u64,
    /// Readiness dropped because the registration was removed before delivery.
    pub io_stale: u64,
    /// System lane (timers, I/O, lifecycle).
    pub system_lane: LaneStats,
    /// Data lane (messages from other EOs).
    pub data_lane: LaneStats,
    /// Buffer pool, one entry per size class, smallest first.
    pub buffers: Vec<BufferClassStats>,
    /// Creation cost.
    pub creation: CreationStats,
}

/// Activity of one runtime worker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerStats {
    /// Pool name.
    pub pool: String,
    /// Index of the worker in its pool.
    pub index: usize,
    /// Core it is pinned to, if any.
    pub core: Option<usize>,
    /// Scheduling turns run.
    pub turns: u64,
    /// Messages handled.
    pub messages: u64,
    /// Time spent running turns.
    pub busy: Duration,
    /// Time spent sleeping in `epoll_wait`, the current sleep included.
    pub idle: Duration,
    /// Times it woke up from sleep.
    pub wakeups: u64,
    /// Readiness events harvested.
    pub io_events: u64,
    /// Timer expiries posted from its wheel.
    pub timers_fired: u64,
    /// Cancellations sent to another worker's wheel.
    pub remote_cancels: u64,
    /// Cancellations left to expire because the owner's cancel queue was full.
    pub lazy_cancels: u64,
}

pub(crate) struct WorkerCounters {
    pub(crate) turns: AtomicU64,
    pub(crate) messages: AtomicU64,
    pub(crate) busy_ns: AtomicU64,
    pub(crate) idle_ns: AtomicU64,
    /// Platform time + 1 when the current sleep began, 0 when awake.
    pub(crate) asleep_since: AtomicU64,
    pub(crate) wakeups: AtomicU64,
    pub(crate) io_events: AtomicU64,
    pub(crate) timers_fired: AtomicU64,
    pub(crate) remote_cancels: AtomicU64,
    pub(crate) lazy_cancels: AtomicU64,
}

impl WorkerCounters {
    pub(crate) fn new() -> Self {
        WorkerCounters {
            turns: AtomicU64::new(0),
            messages: AtomicU64::new(0),
            busy_ns: AtomicU64::new(0),
            idle_ns: AtomicU64::new(0),
            asleep_since: AtomicU64::new(0),
            wakeups: AtomicU64::new(0),
            io_events: AtomicU64::new(0),
            timers_fired: AtomicU64::new(0),
            remote_cancels: AtomicU64::new(0),
            lazy_cancels: AtomicU64::new(0),
        }
    }

    pub(crate) fn add(counter: &AtomicU64, n: u64) {
        counter.fetch_add(n, Ordering::Relaxed);
    }

    pub(crate) fn add_time(counter: &AtomicU64, d: Duration) {
        counter.fetch_add(d.as_nanos().min(u128::from(u64::MAX)) as u64, Ordering::Relaxed);
    }

    /// `now`: platform time, to count the sleep in progress.
    pub(crate) fn snapshot(&self, pool: &str, index: usize, core: Option<usize>, now_ns: u64) -> WorkerStats {
        let load = |c: &AtomicU64| c.load(Ordering::Relaxed);
        let asleep = match load(&self.asleep_since) {
            0 => 0,
            since => now_ns.saturating_sub(since - 1),
        };
        WorkerStats {
            pool: pool.to_string(),
            index,
            core,
            turns: load(&self.turns),
            messages: load(&self.messages),
            busy: Duration::from_nanos(load(&self.busy_ns)),
            idle: Duration::from_nanos(load(&self.idle_ns) + asleep),
            wakeups: load(&self.wakeups),
            io_events: load(&self.io_events),
            timers_fired: load(&self.timers_fired),
            remote_cancels: load(&self.remote_cancels),
            lazy_cancels: load(&self.lazy_cancels),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histogram_buckets() {
        assert_eq!(histogram_bucket(Duration::from_nanos(999)), 0);
        assert_eq!(histogram_bucket(Duration::from_micros(1)), 1);
        assert_eq!(histogram_bucket(Duration::from_micros(3)), 2);
        assert_eq!(histogram_bucket(Duration::from_micros(1024)), 11);
        assert_eq!(histogram_bucket(Duration::from_secs(3600)), HIST_BUCKETS - 1);
        for i in 1..HIST_BUCKETS {
            assert_eq!(histogram_bucket(histogram_bucket_floor(i)), i);
        }
    }

    #[test]
    fn records_and_resets_peaks() {
        let c = EoCounters::new();
        assert_eq!(c.processing(false).min, None);
        c.record(Duration::from_micros(5));
        c.record(Duration::from_micros(50));
        let p = c.processing(true);
        assert_eq!((p.count, p.min, p.max), (2, Some(Duration::from_micros(5)), Duration::from_micros(50)));
        assert_eq!(p.mean(), Some(Duration::from_nanos(27_500)));
        let p = c.processing(false);
        assert_eq!((p.count, p.min, p.max), (2, None, Duration::ZERO));
    }
}
