//! Operations, administration and maintenance: the non-3GPP EOs.
//!
//! | EO | Name | Role |
//! |---|---|---|
//! | [`LogSink`] | `log` | writes log records as JSON lines |
//! | [`AlarmManager`] | `alarm` | active alarms, logged on every change |
//! | [`Control`] | `control` | starts NFs from a deployment (TOML or JSON), stops EOs, supervises |
//! | [`Server`] | `server` | web API and GUI |
//!
//! They run in their own pool, so OAM traffic never takes a worker from an NF.
//! Metrics need no EO: any EO reads them through its `Context`; the server
//! renders them ([`metrics`]).

mod alarm;
mod control;
mod log;
pub mod metrics;
pub mod proto;
mod server;

pub use alarm::AlarmManager;
pub use control::{Control, Deployment, Factory, NfSpec};
pub use log::{CapturedLine, CapturedLog, LogSink};
pub use server::Server;

/// Pool the OAM EOs run on.
pub const POOL: &str = "oam";
