//! An EO that records everything it receives.

use std::sync::{Arc, Mutex};

use crate::timer::Time;
use crate::{Context, ExecutionObject, Message, MsgBody, MsgHeader};

/// One message seen by a [`Probe`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Received {
    /// Its header.
    pub header: MsgHeader,
    /// Copy of its body.
    pub body: Vec<u8>,
    /// Platform time at delivery.
    pub at: Time,
}

impl Received {
    /// Copy of the body as a `T`, if the id and length match.
    pub fn body<T: MsgBody>(&self) -> Option<T> {
        (self.header.msg_id == T::MSG_ID).then(|| T::read_from_bytes(&self.body).ok()).flatten()
    }
}

/// Shared log of a [`Probe`]; clone it freely.
#[derive(Clone, Debug, Default)]
pub struct ProbeLog(Arc<Mutex<Vec<Received>>>);

impl ProbeLog {
    /// Messages received so far.
    pub fn len(&self) -> usize {
        self.0.lock().unwrap().len()
    }

    /// Whether nothing was received.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Copy of the log.
    pub fn messages(&self) -> Vec<Received> {
        self.0.lock().unwrap().clone()
    }

    /// Empties the log, returning its content.
    pub fn take(&self) -> Vec<Received> {
        std::mem::take(&mut *self.0.lock().unwrap())
    }
}

/// Test EO that records every message it handles (allocates: test use only).
#[derive(Debug)]
pub struct Probe {
    log: ProbeLog,
}

impl Probe {
    /// A probe and the handle to read what it receives.
    pub fn new() -> (Probe, ProbeLog) {
        let log = ProbeLog::default();
        (Probe { log: log.clone() }, log)
    }
}

impl ExecutionObject for Probe {
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        let received = Received { header: *msg.header(), body: msg.body_bytes().to_vec(), at: ctx.now() };
        self.log.0.lock().unwrap().push(received);
    }
}
