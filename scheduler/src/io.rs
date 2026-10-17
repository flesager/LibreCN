//! I/O readiness: per-EO registrations and a thin epoll wrapper.
//!
//! Registrations are one-shot and level-triggered: the platform delivers one
//! `IO_READY`, then re-arms the descriptor at the end of the turn that handled
//! it. A socket that still has data is reported again; the handler can read as
//! much or as little as it likes, and an EO never has more than one `IO_READY`
//! per registration waiting, which bounds its system lane, nor handles more
//! than one per turn, which leaves the rest of the turn to its data lane.
//!
//! The platform keeps its own duplicate of every registered descriptor, so it
//! can always re-arm and remove it safely. The socket therefore stays open
//! until it is deregistered (or the EO stops), even if the EO drops its handle.

use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::time::Duration;

use rustix::buffer::spare_capacity;
use rustix::event::{Timespec, epoll};

use crate::{Error, Result};

/// What to watch on a descriptor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Interest(u8);

impl Interest {
    /// Data to read, or peer closed.
    pub const READABLE: Interest = Interest(1);
    /// Room to write.
    pub const WRITABLE: Interest = Interest(2);

    const fn contains(self, other: Interest) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for Interest {
    type Output = Interest;

    fn bitor(self, rhs: Interest) -> Interest {
        Interest(self.0 | rhs.0)
    }
}

/// Bits of the `corr_id` of an `IO_READY` message.
mod readiness {
    /// Data to read (or end of stream).
    pub const READABLE: u32 = 1 << 0;
    /// Room to write.
    pub const WRITABLE: u32 = 1 << 1;
    /// Peer hung up.
    pub const HANGUP: u32 = 1 << 2;
    /// Error pending on the descriptor.
    pub const ERROR: u32 = 1 << 3;
}

/// Handle on a registration: slot index (16 bits) + generation (16 bits).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IoToken(u32);

impl IoToken {
    const fn new(index: u16, generation: u16) -> Self {
        IoToken(((generation as u32) << 16) | index as u32)
    }

    pub(crate) const fn from_raw(raw: u32) -> Self {
        IoToken(raw)
    }

    /// Raw value, as carried in the `reserved` field of `IO_READY`.
    pub(crate) const fn raw(self) -> u32 {
        self.0
    }

    const fn index(self) -> usize {
        (self.0 & 0xFFFF) as usize
    }

    const fn generation(self) -> u16 {
        (self.0 >> 16) as u16
    }
}

/// A delivered readiness, see [`Message::io`](crate::Message::io).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IoReady {
    /// Token returned by `register_io`.
    pub token: IoToken,
    pub(crate) readiness: u32,
}

impl IoReady {
    /// Data to read.
    pub fn readable(&self) -> bool {
        self.readiness & readiness::READABLE != 0
    }

    /// Room to write.
    pub fn writable(&self) -> bool {
        self.readiness & readiness::WRITABLE != 0
    }

    /// Peer hung up.
    pub fn hangup(&self) -> bool {
        self.readiness & readiness::HANGUP != 0
    }

    /// Error pending.
    pub fn error(&self) -> bool {
        self.readiness & readiness::ERROR != 0
    }
}

struct IoSlot {
    generation: u16,
    fd: Option<OwnedFd>,
    interest: Interest,
    /// Its readiness was handled this turn: re-armed at the turn's end.
    handled: bool,
}

/// Registrations of one EO; only touched by the thread running that EO.
pub(crate) struct IoTable {
    slots: Box<[IoSlot]>,
    /// Marks set since the last [`take_handled`](Self::take_handled); 0
    /// lets a turn that handled no readiness skip the scan.
    handled: u16,
}

impl IoTable {
    pub(crate) fn new(capacity: u16) -> Self {
        let slot = || IoSlot { generation: 0, fd: None, interest: Interest::READABLE, handled: false };
        IoTable { slots: (0..capacity).map(|_| slot()).collect(), handled: 0 }
    }

    /// Stores `fd` in a free slot.
    pub(crate) fn insert(&mut self, fd: OwnedFd, interest: Interest) -> Result<IoToken> {
        let (index, slot) =
            self.slots.iter_mut().enumerate().find(|(_, s)| s.fd.is_none()).ok_or(Error::IoExhausted)?;
        slot.fd = Some(fd);
        slot.interest = interest;
        slot.handled = false;
        Ok(IoToken::new(index as u16, slot.generation))
    }

    /// Notes that the readiness of `token` was handled, if it is current.
    pub(crate) fn mark_handled(&mut self, token: IoToken) {
        if let Some(slot) = self.slots.get_mut(token.index())
            && slot.generation == token.generation()
            && slot.fd.is_some()
            && !slot.handled
        {
            slot.handled = true;
            self.handled += 1;
        }
    }

    /// Calls `f` for every registration marked handled, and clears the marks.
    pub(crate) fn take_handled(&mut self, mut f: impl FnMut(IoToken, BorrowedFd<'_>, Interest)) {
        if self.handled == 0 {
            return;
        }
        self.handled = 0;
        for (i, s) in self.slots.iter_mut().enumerate() {
            if std::mem::take(&mut s.handled)
                && let Some(fd) = &s.fd
            {
                f(IoToken::new(i as u16, s.generation), fd.as_fd(), s.interest);
            }
        }
    }

    /// The registered descriptor and interest, if `token` is current.
    pub(crate) fn get(&self, token: IoToken) -> Option<(BorrowedFd<'_>, Interest)> {
        let slot = self.slots.get(token.index())?;
        if slot.generation != token.generation() {
            return None;
        }
        slot.fd.as_ref().map(|fd| (fd.as_fd(), slot.interest))
    }

    /// Frees the slot, handing back the platform's duplicate; the token goes stale.
    pub(crate) fn remove(&mut self, token: IoToken) -> Option<OwnedFd> {
        let slot = self.slots.get_mut(token.index())?;
        if slot.generation != token.generation() {
            return None;
        }
        let fd = slot.fd.take()?;
        slot.generation = slot.generation.wrapping_add(1);
        Some(fd)
    }

    /// Calls `f` for every registration.
    pub(crate) fn for_each(&self, mut f: impl FnMut(IoToken, BorrowedFd<'_>, Interest)) {
        for (i, s) in self.slots.iter().enumerate() {
            if let Some(fd) = &s.fd {
                f(IoToken::new(i as u16, s.generation), fd.as_fd(), s.interest);
            }
        }
    }

    /// Removes every registration, handing each duplicate to `f`.
    pub(crate) fn drain(&mut self, mut f: impl FnMut(OwnedFd)) {
        for s in self.slots.iter_mut() {
            if let Some(fd) = s.fd.take() {
                s.generation = s.generation.wrapping_add(1);
                f(fd);
            }
        }
    }

    /// Tokens of every registration.
    #[cfg(test)]
    pub(crate) fn tokens(&self) -> impl Iterator<Item = IoToken> + '_ {
        self.slots.iter().enumerate().filter(|(_, s)| s.fd.is_some()).map(|(i, s)| IoToken::new(i as u16, s.generation))
    }

    pub(crate) fn bytes_reserved(&self) -> usize {
        std::mem::size_of_val(&*self.slots)
    }
}

/// epoll user data of a registration: EO registry index + token.
pub(crate) fn encode(eo_index: u32, token: IoToken) -> u64 {
    (u64::from(eo_index) << 32) | u64::from(token.raw())
}

pub(crate) fn decode(data: u64) -> (u32, IoToken) {
    ((data >> 32) as u32, IoToken::from_raw(data as u32))
}

fn event_flags(interest: Interest) -> epoll::EventFlags {
    let mut flags = epoll::EventFlags::ONESHOT | epoll::EventFlags::RDHUP;
    if interest.contains(Interest::READABLE) {
        flags |= epoll::EventFlags::IN;
    }
    if interest.contains(Interest::WRITABLE) {
        flags |= epoll::EventFlags::OUT;
    }
    flags
}

fn readiness_of(flags: epoll::EventFlags) -> u32 {
    use epoll::EventFlags as F;
    let mut r = 0;
    if flags.intersects(F::IN | F::PRI | F::RDHUP) {
        r |= readiness::READABLE;
    }
    if flags.contains(F::OUT) {
        r |= readiness::WRITABLE;
    }
    if flags.intersects(F::HUP | F::RDHUP) {
        r |= readiness::HANGUP;
    }
    if flags.contains(F::ERR) {
        r |= readiness::ERROR;
    }
    r
}

/// An epoll instance. Every method takes `&self`: epoll is thread-safe.
pub(crate) struct Poller {
    epoll: OwnedFd,
}

impl Poller {
    pub(crate) fn new() -> Result<Self> {
        Ok(Poller { epoll: epoll::create(epoll::CreateFlags::CLOEXEC)? })
    }

    pub(crate) fn as_fd(&self) -> BorrowedFd<'_> {
        self.epoll.as_fd()
    }

    pub(crate) fn add(&self, fd: BorrowedFd<'_>, data: u64, interest: Interest) -> Result<()> {
        Ok(epoll::add(&self.epoll, fd, epoll::EventData::new_u64(data), event_flags(interest))?)
    }

    /// Level-triggered, for the platform's own descriptors (wake-up eventfd,
    /// a pool's poller nested in a worker's).
    pub(crate) fn add_level(&self, fd: BorrowedFd<'_>, data: u64) -> Result<()> {
        Ok(epoll::add(&self.epoll, fd, epoll::EventData::new_u64(data), epoll::EventFlags::IN)?)
    }

    pub(crate) fn rearm(&self, fd: BorrowedFd<'_>, data: u64, interest: Interest) -> Result<()> {
        Ok(epoll::modify(&self.epoll, fd, epoll::EventData::new_u64(data), event_flags(interest))?)
    }

    pub(crate) fn delete(&self, fd: BorrowedFd<'_>) {
        // Already gone if the descriptor was closed elsewhere: nothing to undo.
        let _ = epoll::delete(&self.epoll, fd);
    }

    /// Waits up to `timeout` (`None`: forever) and calls `on_event(data,
    /// readiness)` for each event. `events` is the caller's preallocated
    /// buffer; its capacity bounds the events per call. Interrupted waits
    /// return 0 events.
    pub(crate) fn wait(
        &self,
        events: &mut Vec<epoll::Event>,
        timeout: Option<Duration>,
        mut on_event: impl FnMut(u64, u32),
    ) -> Result<usize> {
        let timespec = timeout.map(|t| Timespec {
            tv_sec: t.as_secs().min(i64::MAX as u64) as i64,
            tv_nsec: i64::from(t.subsec_nanos()),
        });
        events.clear();
        match epoll::wait(&self.epoll, spare_capacity(events), timespec.as_ref()) {
            Ok(_) | Err(rustix::io::Errno::INTR) => {}
            Err(e) => return Err(e.into()),
        }
        for e in events.iter() {
            on_event(e.data.u64(), readiness_of(e.flags));
        }
        Ok(events.len())
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use std::net::UdpSocket;

    use super::*;

    #[test]
    fn table_tokens_go_stale() {
        let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
        let mut t = IoTable::new(1);
        let a = t.insert(sock.as_fd().try_clone_to_owned().unwrap(), Interest::READABLE).unwrap();
        assert_eq!(t.insert(sock.as_fd().try_clone_to_owned().unwrap(), Interest::READABLE), Err(Error::IoExhausted));
        assert!(t.get(a).is_some());
        assert!(t.remove(a).is_some());
        assert!(t.get(a).is_none() && t.remove(a).is_none());
        let b = t.insert(sock.as_fd().try_clone_to_owned().unwrap(), Interest::READABLE).unwrap();
        assert_ne!(a, b);
        assert_eq!(t.tokens().collect::<Vec<_>>(), [b]);
    }

    #[test]
    fn handled_marks_are_taken_once_and_only_for_current_tokens() {
        let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
        let fd = || sock.as_fd().try_clone_to_owned().unwrap();
        let mut t = IoTable::new(3);
        let (a, b, c) = (
            t.insert(fd(), Interest::READABLE).unwrap(),
            t.insert(fd(), Interest::READABLE).unwrap(),
            t.insert(fd(), Interest::READABLE).unwrap(),
        );
        t.mark_handled(a);
        t.mark_handled(a);
        t.mark_handled(c);
        let taken = |t: &mut IoTable| {
            let mut v = Vec::new();
            t.take_handled(|token, _, _| v.push(token));
            v
        };
        assert_eq!(taken(&mut t), [a, c], "each marked registration once");
        assert!(taken(&mut t).is_empty(), "marks cleared");
        // Marked, then freed and reused in the same turn: neither is taken.
        t.mark_handled(b);
        t.remove(b);
        t.mark_handled(b);
        let d = t.insert(fd(), Interest::READABLE).unwrap();
        assert!(taken(&mut t).is_empty());
        t.mark_handled(d);
        assert_eq!(taken(&mut t), [d]);
    }

    #[test]
    fn one_shot_until_rearmed() {
        let rx = UdpSocket::bind("127.0.0.1:0").unwrap();
        let tx = UdpSocket::bind("127.0.0.1:0").unwrap();
        let poller = Poller::new().unwrap();
        let mut events = Vec::with_capacity(4);
        let data = encode(7, IoToken::new(2, 5));
        poller.add(rx.as_fd(), data, Interest::READABLE).unwrap();
        tx.send_to(b"x", rx.local_addr().unwrap()).unwrap();

        let mut seen = vec![];
        poller.wait(&mut events, Some(Duration::from_secs(1)), |d, r| seen.push((decode(d), r))).unwrap();
        assert_eq!(seen, [((7, IoToken::new(2, 5)), readiness::READABLE)]);
        // Not read yet, but one-shot: silent until re-armed.
        assert_eq!(poller.wait(&mut events, Some(Duration::ZERO), |_, _| {}).unwrap(), 0);
        poller.rearm(rx.as_fd(), data, Interest::READABLE).unwrap();
        assert_eq!(poller.wait(&mut events, Some(Duration::ZERO), |_, _| {}).unwrap(), 1);
    }
}
