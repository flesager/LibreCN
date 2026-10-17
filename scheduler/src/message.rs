//! Zero-copy message bodies.

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::core::EoCore;
use crate::header::system;
use crate::io::{IoReady, IoToken};
use crate::mailbox::DataItem;
use crate::pool::BufRef;
use crate::timer::TimerId;
use crate::{EoId, MsgHeader, Result};

/// A fixed-layout body type, written in place in the receiver's buffer.
///
/// ```
/// use librecn_scheduler::MsgBody;
/// use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};
///
/// #[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
/// #[repr(C)]
/// struct Ping { seq: u64 }
///
/// impl MsgBody for Ping { const MSG_ID: u32 = 0x0001_0001; }
/// ```
pub trait MsgBody: FromBytes + IntoBytes + KnownLayout + Immutable + Sized {
    /// Message id; must be outside the system range.
    const MSG_ID: u32;
}

/// A received message: header plus a view of the body in the receiver's buffer.
/// The buffer returns to the pool once the handler returns.
#[derive(Clone, Copy, Debug)]
pub struct Message<'a> {
    header: MsgHeader,
    body: &'a [u8],
}

/// A delivered timer expiry, see [`Message::timer`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimerFired {
    /// Id returned by `arm_timer`.
    pub id: TimerId,
    /// Tag given to `arm_timer`.
    pub tag: u32,
}

impl<'a> Message<'a> {
    pub(crate) fn new(header: MsgHeader, body: &'a [u8]) -> Self {
        Message { header, body }
    }

    /// Full header.
    pub fn header(&self) -> &MsgHeader {
        &self.header
    }

    /// Message id.
    pub fn id(&self) -> u32 {
        self.header.msg_id
    }

    /// Sender.
    pub fn from(&self) -> EoId {
        self.header.from
    }

    /// Correlation id.
    pub fn corr_id(&self) -> u32 {
        self.header.corr_id
    }

    /// Raw body bytes; empty for header-only messages.
    pub fn body_bytes(&self) -> &'a [u8] {
        self.body
    }

    /// Typed view of the body, without copy. `None` if the id is not
    /// `T::MSG_ID` or the length does not match `T`.
    pub fn body<T: MsgBody>(&self) -> Option<&'a T> {
        if self.header.msg_id != T::MSG_ID {
            return None;
        }
        T::ref_from_bytes(self.body).ok()
    }

    /// The readiness, if this is an `IO_READY` message.
    pub fn io(&self) -> Option<IoReady> {
        (self.header.msg_id == system::IO_READY && self.header.from == EoId::SYSTEM)
            .then(|| IoReady { token: IoToken::from_raw(self.header.reserved as u32), readiness: self.header.corr_id })
    }

    /// The expired timer, if this is a `TIMER_EXPIRED` message.
    pub fn timer(&self) -> Option<TimerFired> {
        (self.header.msg_id == system::TIMER_EXPIRED && self.header.from == EoId::SYSTEM)
            .then(|| TimerFired { id: TimerId::from_raw(self.header.reserved), tag: self.header.corr_id })
    }
}

/// A message being written, in a buffer already taken from the receiver's
/// pool. Fill it through `Deref`, then hand it to `Context::send`. Dropping it
/// unsent returns the buffer.
pub struct OutMsg<T: MsgBody> {
    core: Arc<EoCore>,
    buf: Option<BufRef>,
    header: MsgHeader,
    _body: PhantomData<fn() -> T>,
}

impl<T: MsgBody> OutMsg<T> {
    pub(crate) fn new(core: Arc<EoCore>, from: EoId) -> Result<Self> {
        let len = std::mem::size_of::<T>();
        let mut buf = core.mailbox.pool.alloc(len)?;
        core.mailbox.pool.bytes_mut(&mut buf, len).fill(0);
        let mut header = MsgHeader::new(T::MSG_ID, from, core.id);
        header.body_len = len as u32;
        Ok(OutMsg { core, buf: Some(buf), header, _body: PhantomData })
    }

    /// Receiver.
    pub fn to(&self) -> EoId {
        self.header.to
    }

    /// Sets the correlation id.
    pub fn with_corr_id(mut self, corr_id: u32) -> Self {
        self.header.corr_id = corr_id;
        self
    }

    /// Sets the header flags (free for the application).
    pub fn with_flags(mut self, flags: u32) -> Self {
        self.header.flags = flags;
        self
    }

    pub(crate) fn into_parts(mut self) -> (Arc<EoCore>, DataItem) {
        let item = DataItem { header: self.header, buf: self.buf.take() };
        (self.core.clone(), item)
    }
}

impl<T: MsgBody> Deref for OutMsg<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let buf = self.buf.as_ref().expect("buffer held until sent");
        T::ref_from_bytes(self.core.mailbox.pool.bytes(buf, std::mem::size_of::<T>()))
            .expect("buffer sized and aligned for T at allocation")
    }
}

impl<T: MsgBody> DerefMut for OutMsg<T> {
    fn deref_mut(&mut self) -> &mut T {
        let buf = self.buf.as_mut().expect("buffer held until sent");
        T::mut_from_bytes(self.core.mailbox.pool.bytes_mut(buf, std::mem::size_of::<T>()))
            .expect("buffer sized and aligned for T at allocation")
    }
}

impl<T: MsgBody> Drop for OutMsg<T> {
    fn drop(&mut self) {
        if let Some(buf) = self.buf.take() {
            self.core.mailbox.pool.free(buf);
        }
    }
}

impl<T: MsgBody> std::fmt::Debug for OutMsg<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutMsg").field("header", &self.header).finish_non_exhaustive()
    }
}
