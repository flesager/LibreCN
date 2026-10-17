//! Two-lane mailbox owning the buffer pool of its EO.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use crate::pool::{BufRef, BufferPool};
use crate::queue::Ring;
use crate::stats::LaneStats;
use crate::{EoConfig, Error, MsgHeader, Result};

pub(crate) struct DataItem {
    pub(crate) header: MsgHeader,
    pub(crate) buf: Option<BufRef>,
}

pub(crate) enum Item {
    System(MsgHeader),
    Data(DataItem),
}

struct Lane<T> {
    ring: Ring<T>,
    high_water: AtomicUsize,
    rejected: AtomicU64,
}

impl<T> Lane<T> {
    fn new(capacity: usize) -> Self {
        Lane { ring: Ring::new(capacity), high_water: AtomicUsize::new(0), rejected: AtomicU64::new(0) }
    }

    fn push(&self, value: T) -> Result<(), T> {
        match self.ring.push(value) {
            Ok(()) => {
                self.high_water.fetch_max(self.ring.len(), Ordering::Relaxed);
                Ok(())
            }
            Err(value) => {
                self.rejected.fetch_add(1, Ordering::Relaxed);
                Err(value)
            }
        }
    }

    fn stats(&self, reset_peaks: bool) -> LaneStats {
        let depth = self.ring.len();
        let high_water = if reset_peaks {
            self.high_water.swap(depth, Ordering::Relaxed)
        } else {
            self.high_water.load(Ordering::Relaxed)
        };
        LaneStats { capacity: self.ring.capacity(), depth, high_water, rejected: self.rejected.load(Ordering::Relaxed) }
    }
}

/// System lane (header-only, drained first) + data lane + the buffers of
/// every body sent to this EO.
pub(crate) struct Mailbox {
    system: Lane<MsgHeader>,
    data: Lane<DataItem>,
    pub(crate) pool: BufferPool,
}

impl Mailbox {
    pub(crate) fn new(cfg: &EoConfig) -> Result<Self> {
        Ok(Mailbox {
            system: Lane::new(cfg.system_capacity),
            data: Lane::new(cfg.data_capacity),
            pool: BufferPool::new(&cfg.buffers)?,
        })
    }

    pub(crate) fn push_system(&self, header: MsgHeader) -> Result<()> {
        self.system.push(header).map_err(|_| Error::QueueFull)
    }

    /// On failure the body buffer goes back to the pool.
    pub(crate) fn push_data(&self, item: DataItem) -> Result<()> {
        self.data.push(item).map_err(|item| {
            self.release(item);
            Error::QueueFull
        })
    }

    pub(crate) fn pop(&self) -> Option<Item> {
        self.system.ring.pop().map(Item::System).or_else(|| self.data.ring.pop().map(Item::Data))
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.system.ring.is_empty() && self.data.ring.is_empty()
    }

    pub(crate) fn release(&self, item: DataItem) {
        if let Some(buf) = item.buf {
            self.pool.free(buf);
        }
    }

    /// Drops every waiting message, returning their buffers.
    pub(crate) fn drain(&self) {
        while let Some(item) = self.pop() {
            if let Item::Data(item) = item {
                self.release(item);
            }
        }
    }

    pub(crate) fn bytes_reserved(&self) -> usize {
        self.system.ring.bytes_reserved() + self.data.ring.bytes_reserved() + self.pool.bytes_reserved()
    }

    pub(crate) fn lane_stats(&self, reset_peaks: bool) -> (LaneStats, LaneStats) {
        (self.system.stats(reset_peaks), self.data.stats(reset_peaks))
    }
}
