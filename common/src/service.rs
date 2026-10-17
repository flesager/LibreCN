//! Finding well-known service EOs by name, cheaply.

use std::time::Duration;

use librecn_scheduler::{Context, EoId, Time};

/// Names of the OAM service EOs.
pub mod names {
    /// Log sink.
    pub const LOG: &str = "log";
    /// Alarm manager.
    pub const ALARM: &str = "alarm";
    /// Control (lifecycle of the EOs).
    pub const CONTROL: &str = "control";
}

/// How long an absent service is not looked for again.
const RETRY: Duration = Duration::from_secs(1);

/// Cached id of a service EO. Looks it up once, and again only after
/// [`forget`](Self::forget) or, while absent, once per second.
#[derive(Clone, Debug)]
pub struct Service {
    name: &'static str,
    id: Option<EoId>,
    retry_at: Time,
}

impl Service {
    /// A service known by `name`, not looked up yet.
    pub const fn new(name: &'static str) -> Self {
        Service { name, id: None, retry_at: Time::ZERO }
    }

    /// The service's id, if it runs.
    pub fn get(&mut self, ctx: &Context<'_>) -> Option<EoId> {
        if self.id.is_none() && ctx.now() >= self.retry_at {
            self.id = ctx.find(self.name);
            self.retry_at = ctx.now() + RETRY;
        }
        self.id
    }

    /// The cached id went stale (the service stopped): look it up again.
    pub fn forget(&mut self) {
        self.id = None;
        self.retry_at = Time::ZERO;
    }
}
