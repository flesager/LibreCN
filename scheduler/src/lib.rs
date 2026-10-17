//! Reactor scheduler hosting run-to-completion execution objects (EOs).
//!
//! See `DESIGN.md` for the full design. EOs run on the multi-core
//! [`runtime::Runtime`], or unchanged on the single-threaded
//! [`testbed::TestBed`].
//!
//! - Everything an EO receives is a message: other EOs' messages on the data
//!   lane, timers and lifecycle on the system lane (drained first).
//! - All memory is preallocated when an EO is created: mailbox lanes, a
//!   buffer pool per mailbox for message bodies, timers, flight recorder.
//! - Bodies are fixed-layout structs written in place in the receiver's
//!   buffer ([`MsgBody`], [`Context::alloc`]), never serialised.

mod config;
mod context;
mod core;
mod eo;
mod error;
mod exec;
mod failure;
mod header;
mod id;
mod io;
mod mailbox;
mod message;
mod pool;
pub mod queue;
mod recorder;
mod registry;
pub mod runtime;
mod stats;
mod sync;
pub mod testbed;
mod timer;

pub use config::EoConfig;
pub use context::Context;
pub use eo::{EoInfo, ExecutionObject};
pub use error::{Error, Result};
pub use failure::{FailureReport, FaultPolicy, install_panic_hook};
pub use header::{MsgHeader, system};
pub use id::EoId;
pub use io::{Interest, IoReady, IoToken};
pub use message::{Message, MsgBody, OutMsg, TimerFired};
pub use pool::{BUFFER_ALIGN, BufferClassConfig};
pub use stats::{
    BufferClassStats, CreationStats, EoStats, HIST_BUCKETS, LaneStats, ProcessingStats, WorkerStats, histogram_bucket,
    histogram_bucket_floor,
};
pub use timer::{Time, TimerId};
