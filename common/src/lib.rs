//! Conventions shared by the LibreCN EOs (network functions and OAM), on top
//! of the scheduler: message-id families, header flags, well-known services,
//! logging, alarms, the management requests of the web API, and the state
//! NFs keep across restarts.

pub mod alarm;
pub mod log;
pub mod manage;
pub mod msg;
mod service;
pub mod state;
mod text;

pub use service::{Service, names};
