//! Configuration of one execution object.

use crate::pool::BufferClassConfig;
use crate::{Error, Result};

/// Sizes of everything preallocated for one EO.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EoConfig {
    /// Name used in metrics and failure reports.
    pub name: String,
    /// Runtime pool the EO runs on; `None` = the first pool. Ignored by the test bed.
    pub pool: Option<String>,
    /// Slots of the data lane (messages from other EOs).
    pub data_capacity: usize,
    /// Slots of the system lane (timers, I/O, lifecycle). Must hold every timer,
    /// one readiness message per I/O registration, plus start and stop.
    pub system_capacity: usize,
    /// Buffer pool size classes, for message bodies.
    pub buffers: Vec<BufferClassConfig>,
    /// Timers that can be armed at the same time.
    pub max_timers: u32,
    /// File descriptors that can be registered at the same time.
    pub max_io: u16,
    /// Headers kept for failure reports.
    pub flight_recorder_depth: usize,
}

impl EoConfig {
    /// Defaults: first pool, 1024 data slots, 64 system slots, 256 × 256 B +
    /// 32 × 2 KiB buffers, 32 timers, 8 I/O registrations, 32 recorded headers.
    pub fn new(name: impl Into<String>) -> Self {
        EoConfig {
            name: name.into(),
            pool: None,
            data_capacity: 1024,
            system_capacity: 64,
            buffers: vec![BufferClassConfig { size: 256, count: 256 }, BufferClassConfig { size: 2048, count: 32 }],
            max_timers: 32,
            max_io: 8,
            flight_recorder_depth: 32,
        }
    }

    /// Sets [`pool`](Self::pool).
    pub fn pool(mut self, pool: impl Into<String>) -> Self {
        self.pool = Some(pool.into());
        self
    }

    /// Sets [`max_io`](Self::max_io).
    pub fn max_io(mut self, registrations: u16) -> Self {
        self.max_io = registrations;
        self
    }

    /// Sets [`data_capacity`](Self::data_capacity).
    pub fn data_capacity(mut self, slots: usize) -> Self {
        self.data_capacity = slots;
        self
    }

    /// Sets [`system_capacity`](Self::system_capacity).
    pub fn system_capacity(mut self, slots: usize) -> Self {
        self.system_capacity = slots;
        self
    }

    /// Replaces the buffer classes with `(size, count)` pairs.
    pub fn buffers(mut self, classes: &[(usize, u32)]) -> Self {
        self.buffers = classes.iter().map(|&(size, count)| BufferClassConfig { size, count }).collect();
        self
    }

    /// Sets [`max_timers`](Self::max_timers).
    pub fn max_timers(mut self, timers: u32) -> Self {
        self.max_timers = timers;
        self
    }

    /// Sets [`flight_recorder_depth`](Self::flight_recorder_depth).
    pub fn flight_recorder_depth(mut self, headers: usize) -> Self {
        self.flight_recorder_depth = headers;
        self
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if self.data_capacity == 0 {
            return Err(Error::InvalidConfig("data_capacity must be > 0"));
        }
        if self.system_capacity < self.max_timers as usize + usize::from(self.max_io) + 2 {
            return Err(Error::InvalidConfig("system_capacity must be >= max_timers + max_io + 2"));
        }
        if self.flight_recorder_depth == 0 {
            return Err(Error::InvalidConfig("flight_recorder_depth must be > 0"));
        }
        Ok(())
    }
}
