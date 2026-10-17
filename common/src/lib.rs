//! Conventions shared by the LibreCN EOs (network functions and OAM), on top
//! of the scheduler: message-id families, header flags, well-known services,
//! logging and alarms.

pub mod alarm;
pub mod log;
pub mod msg;
mod service;
mod text;

pub use service::{Service, names};
