//! The execution object trait.

use crate::{Context, EoId, Message};

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

impl ExecutionObject for Box<dyn ExecutionObject> {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        (**self).on_start(ctx);
    }

    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        (**self).on_message(msg, ctx);
    }

    fn on_stop(&mut self, ctx: &mut Context<'_>) {
        (**self).on_stop(ctx);
    }
}

/// An EO as listed by [`Context::eos`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EoInfo {
    /// Its id.
    pub id: EoId,
    /// Name from its `EoConfig`.
    pub name: String,
    /// Pool it runs on (empty on the test bed).
    pub pool: String,
    /// False once it faulted (isolated by `FaultPolicy::Isolate`).
    pub alive: bool,
}
