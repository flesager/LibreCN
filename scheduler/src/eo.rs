//! The execution object trait.

use crate::{Context, Message};

/// A run-to-completion processing unit; hosts one network function.
///
/// Handlers never block: they react to one message, talk to the outside only
/// through the [`Context`], and return. The scheduler guarantees that an EO is
/// never run by two threads at once, so its state needs no lock.
pub trait ExecutionObject: Send + 'static {
    /// First message of the EO.
    fn on_start(&mut self, _ctx: &mut Context<'_>) {}

    /// Every message other than start and stop, timer expiries included
    /// (see [`Message::timer`]).
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>);

    /// Last message of the EO; it is removed afterwards and waiting messages are dropped.
    fn on_stop(&mut self, _ctx: &mut Context<'_>) {}
}
