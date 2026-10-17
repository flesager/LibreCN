//! Platform time, per-EO timer tables and the hashed timer wheel.
//!
//! The wheel only schedules expiries; the EO's timer table is the truth about
//! which timers are armed. An expiry for a timer that was cancelled or already
//! delivered is dropped before it reaches the handler.

use std::ops::{Add, Sub};
use std::time::Duration;

use crate::{EoId, Error, Result};

/// Platform time: nanoseconds since the platform started. Virtual on the test bed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Time(u64);

impl Time {
    /// Platform start.
    pub const ZERO: Time = Time(0);

    /// Time `ns` nanoseconds after start.
    pub const fn from_nanos(ns: u64) -> Self {
        Time(ns)
    }

    /// Nanoseconds since start.
    pub const fn as_nanos(self) -> u64 {
        self.0
    }
}

impl Add<Duration> for Time {
    type Output = Time;

    fn add(self, d: Duration) -> Time {
        Time(self.0.saturating_add(d.as_nanos().min(u128::from(u64::MAX)) as u64))
    }
}

impl Sub for Time {
    type Output = Duration;

    /// Saturates at zero.
    fn sub(self, earlier: Time) -> Duration {
        Duration::from_nanos(self.0.saturating_sub(earlier.0))
    }
}

/// Handle on an armed timer: slot index in the EO's timer table + generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimerId(u64);

impl TimerId {
    const fn new(index: u32, generation: u32) -> Self {
        TimerId(((generation as u64) << 32) | index as u64)
    }

    pub(crate) const fn from_raw(raw: u64) -> Self {
        TimerId(raw)
    }

    /// Raw value, as carried in the `reserved` field of `TIMER_EXPIRED`.
    pub(crate) const fn raw(self) -> u64 {
        self.0
    }

    const fn index(self) -> usize {
        self.0 as u32 as usize
    }

    const fn generation(self) -> u32 {
        (self.0 >> 32) as u32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WheelHandle {
    index: u32,
    generation: u32,
    /// Worker owning the wheel the entry lives in.
    pub(crate) worker: u16,
}

#[derive(Clone, Copy)]
struct TimerSlot {
    generation: u32,
    armed: bool,
    wheel: WheelHandle,
}

/// Timers of one EO; only touched by the thread running that EO.
pub(crate) struct TimerTable {
    slots: Box<[TimerSlot]>,
    free: Box<[u32]>,
    free_len: usize,
}

impl TimerTable {
    pub(crate) fn new(capacity: u32) -> Self {
        let unattached = WheelHandle { index: u32::MAX, generation: 0, worker: 0 };
        TimerTable {
            slots: vec![TimerSlot { generation: 0, armed: false, wheel: unattached }; capacity as usize]
                .into_boxed_slice(),
            free: (0..capacity).rev().collect(),
            free_len: capacity as usize,
        }
    }

    /// Takes a free slot; attach the wheel entry with [`attach`](Self::attach).
    pub(crate) fn reserve(&mut self) -> Result<TimerId> {
        if self.free_len == 0 {
            return Err(Error::TimersExhausted);
        }
        self.free_len -= 1;
        let index = self.free[self.free_len];
        let slot = &mut self.slots[index as usize];
        slot.armed = true;
        Ok(TimerId::new(index, slot.generation))
    }

    pub(crate) fn attach(&mut self, id: TimerId, handle: WheelHandle) {
        self.slots[id.index()].wheel = handle;
    }

    fn is_armed(&self, id: TimerId) -> bool {
        self.slots.get(id.index()).is_some_and(|s| s.armed && s.generation == id.generation())
    }

    /// Frees the slot; the id becomes stale.
    pub(crate) fn release(&mut self, id: TimerId) {
        let slot = &mut self.slots[id.index()];
        slot.armed = false;
        slot.generation = slot.generation.wrapping_add(1);
        self.free[self.free_len] = id.index() as u32;
        self.free_len += 1;
    }

    /// Disarms `id`; returns its wheel entry if it was armed.
    pub(crate) fn cancel(&mut self, id: TimerId) -> Option<WheelHandle> {
        self.is_armed(id).then(|| {
            let handle = self.slots[id.index()].wheel;
            self.release(id);
            handle
        })
    }

    /// Called when an expiry is about to be delivered: true if `id` is still
    /// armed (and disarms it), false if the expiry is stale.
    pub(crate) fn take_fired(&mut self, id: TimerId) -> bool {
        let armed = self.is_armed(id);
        if armed {
            self.release(id);
        }
        armed
    }

    /// Disarms every timer, handing each wheel entry to `f`.
    pub(crate) fn drain(&mut self, mut f: impl FnMut(WheelHandle)) {
        for index in 0..self.slots.len() {
            let slot = self.slots[index];
            if slot.armed {
                self.release(TimerId::new(index as u32, slot.generation));
                f(slot.wheel);
            }
        }
    }

    pub(crate) fn bytes_reserved(&self) -> usize {
        std::mem::size_of_val(&*self.slots) + std::mem::size_of_val(&*self.free)
    }
}

const NIL: u32 = u32::MAX;

#[derive(Clone, Copy)]
struct WheelEntry {
    deadline: u64,
    seq: u64,
    eo: EoId,
    timer: TimerId,
    tag: u32,
    generation: u32,
    prev: u32,
    next: u32,
    live: bool,
}

/// One expiry produced by [`TimerWheel::expire`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Expired {
    pub(crate) eo: EoId,
    pub(crate) timer: TimerId,
    pub(crate) tag: u32,
}

/// Hashed wheel: `buckets` lists indexed by `deadline_tick % buckets`, entries
/// from a preallocated slab. Deadlines further than one turn stay in their
/// bucket until their tick comes.
pub(crate) struct TimerWheel {
    tick_ns: u64,
    worker: u16,
    /// Last tick processed by `expire`.
    current: u64,
    buckets: Box<[u32]>,
    entries: Box<[WheelEntry]>,
    free_head: u32,
    len: usize,
    next_seq: u64,
    /// Expired entries of one `expire` call, preallocated to the slab size.
    scratch: Vec<u32>,
}

impl TimerWheel {
    pub(crate) fn new(capacity: u32, buckets: usize, tick: Duration, worker: u16) -> Result<Self> {
        let tick_ns = u64::try_from(tick.as_nanos()).unwrap_or(0);
        if tick_ns == 0 || buckets == 0 || capacity == 0 || capacity == NIL {
            return Err(Error::InvalidConfig("timer wheel needs a tick, buckets and capacity > 0"));
        }
        let entries = (0..capacity)
            .map(|i| WheelEntry {
                deadline: 0,
                seq: 0,
                eo: EoId::SYSTEM,
                timer: TimerId(0),
                tag: 0,
                generation: 0,
                prev: NIL,
                next: if i + 1 < capacity { i + 1 } else { NIL },
                live: false,
            })
            .collect();
        Ok(TimerWheel {
            tick_ns,
            worker,
            current: 0,
            buckets: vec![NIL; buckets].into_boxed_slice(),
            entries,
            free_head: 0,
            len: 0,
            next_seq: 0,
            scratch: Vec::with_capacity(capacity as usize),
        })
    }

    fn bucket(&self, deadline: u64) -> usize {
        (deadline % self.buckets.len() as u64) as usize
    }

    /// Schedules an expiry no earlier than `now + after`, rounded up to the next tick.
    pub(crate) fn insert(
        &mut self,
        now: Time,
        after: Duration,
        eo: EoId,
        timer: TimerId,
        tag: u32,
    ) -> Result<WheelHandle> {
        let index = self.free_head;
        if index == NIL {
            return Err(Error::TimersExhausted);
        }
        let deadline = (now + after).as_nanos().div_ceil(self.tick_ns).max(self.current + 1);
        let bucket = self.bucket(deadline);
        let head = self.buckets[bucket];
        let seq = self.next_seq;
        self.next_seq += 1;
        let e = &mut self.entries[index as usize];
        self.free_head = e.next;
        *e = WheelEntry { deadline, seq, eo, timer, tag, generation: e.generation, prev: NIL, next: head, live: true };
        let generation = e.generation;
        if head != NIL {
            self.entries[head as usize].prev = index;
        }
        self.buckets[bucket] = index;
        self.len += 1;
        Ok(WheelHandle { index, generation, worker: self.worker })
    }

    fn unlink(&mut self, index: u32) {
        let WheelEntry { prev, next, deadline, .. } = self.entries[index as usize];
        if prev == NIL {
            let bucket = self.bucket(deadline);
            self.buckets[bucket] = next;
        } else {
            self.entries[prev as usize].next = next;
        }
        if next != NIL {
            self.entries[next as usize].prev = prev;
        }
    }

    fn release(&mut self, index: u32) {
        let e = &mut self.entries[index as usize];
        e.live = false;
        e.generation = e.generation.wrapping_add(1);
        e.next = self.free_head;
        self.free_head = index;
        self.len -= 1;
    }

    /// Removes a pending entry; false if it already expired or was removed.
    pub(crate) fn remove(&mut self, handle: WheelHandle) -> bool {
        let live = self.entries.get(handle.index as usize).is_some_and(|e| e.live && e.generation == handle.generation);
        if live {
            self.unlink(handle.index);
            self.release(handle.index);
        }
        live
    }

    /// Calls `on_expired` for every entry whose deadline tick is at or before
    /// `now`, ordered by deadline then insertion.
    pub(crate) fn expire(&mut self, now: Time, mut on_expired: impl FnMut(Expired)) {
        let target = now.as_nanos() / self.tick_ns;
        if target <= self.current {
            return;
        }
        let span = (target - self.current).min(self.buckets.len() as u64);
        let mut scratch = std::mem::take(&mut self.scratch);
        for tick in self.current + 1..=self.current + span {
            let mut index = self.buckets[self.bucket(tick)];
            while index != NIL {
                let e = self.entries[index as usize];
                if e.deadline <= target {
                    self.unlink(index);
                    scratch.push(index);
                }
                index = e.next;
            }
        }
        self.current = target;
        scratch.sort_unstable_by_key(|&i| (self.entries[i as usize].deadline, self.entries[i as usize].seq));
        for &index in &scratch {
            let e = self.entries[index as usize];
            self.release(index);
            on_expired(Expired { eo: e.eo, timer: e.timer, tag: e.tag });
        }
        scratch.clear();
        self.scratch = scratch;
    }

    /// Time of the earliest pending deadline.
    ///
    /// Walks the buckets in tick order from the next tick: the first entry
    /// whose deadline is the tick being visited is the earliest. If none is
    /// due within one turn of the wheel, every entry was seen once and the
    /// minimum is returned. Cost: at most one turn of buckets plus the entries.
    pub(crate) fn next_deadline(&self) -> Option<Time> {
        if self.len == 0 {
            return None;
        }
        let turn = self.buckets.len() as u64;
        let mut earliest = u64::MAX;
        for tick in self.current + 1..=self.current + turn {
            let mut index = self.buckets[self.bucket(tick)];
            while index != NIL {
                let e = &self.entries[index as usize];
                if e.deadline == tick {
                    earliest = tick;
                    break;
                }
                earliest = earliest.min(e.deadline);
                index = e.next;
            }
            if earliest == tick {
                break;
            }
        }
        Some(Time::from_nanos(earliest.saturating_mul(self.tick_ns)))
    }

    /// Timer resolution.
    pub(crate) fn tick(&self) -> Duration {
        Duration::from_nanos(self.tick_ns)
    }

    /// Pending entries.
    pub(crate) fn len(&self) -> usize {
        self.len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: Duration = Duration::from_millis(1);

    fn wheel(capacity: u32, buckets: usize) -> TimerWheel {
        TimerWheel::new(capacity, buckets, MS, 0).unwrap()
    }

    fn at_ms(ms: u64) -> Time {
        Time::from_nanos(ms * 1_000_000)
    }

    fn fired(w: &mut TimerWheel, now: Time) -> Vec<u32> {
        let mut out = Vec::new();
        w.expire(now, |e| out.push(e.tag));
        out
    }

    #[test]
    fn fires_in_deadline_then_insertion_order_never_early() {
        let mut w = wheel(8, 4);
        let eo = EoId::new(0, 0);
        for (tag, ms) in [(1, 5), (2, 3), (3, 5), (4, 1)] {
            w.insert(Time::ZERO, ms * MS, eo, TimerId(0), tag).unwrap();
        }
        assert!(fired(&mut w, at_ms(0)).is_empty());
        assert_eq!(fired(&mut w, at_ms(3)), [4, 2]);
        assert_eq!(fired(&mut w, Time::from_nanos(4_999_999)), [] as [u32; 0]);
        assert_eq!(fired(&mut w, at_ms(5)), [1, 3]);
        assert_eq!(w.len(), 0);
    }

    #[test]
    fn rounds_partial_ticks_up() {
        let mut w = wheel(2, 16);
        w.insert(Time::from_nanos(500_000), MS, EoId::new(0, 0), TimerId(0), 7).unwrap();
        assert!(fired(&mut w, Time::from_nanos(1_500_000)).is_empty());
        assert_eq!(fired(&mut w, at_ms(2)), [7]);
    }

    #[test]
    fn long_delays_survive_many_turns_and_big_jumps() {
        let mut w = wheel(4, 8);
        let eo = EoId::new(0, 0);
        w.insert(Time::ZERO, 100 * MS, eo, TimerId(0), 1).unwrap();
        w.insert(Time::ZERO, 20 * MS, eo, TimerId(0), 2).unwrap();
        assert_eq!(w.next_deadline(), Some(at_ms(20)));
        for ms in 1..100 {
            let f = fired(&mut w, at_ms(ms));
            assert_eq!(f, if ms == 20 { vec![2] } else { vec![] }, "at {ms} ms");
        }
        assert_eq!(fired(&mut w, at_ms(10_000)), [1]);
    }

    #[test]
    fn next_deadline_matches_a_full_scan() {
        let mut w = wheel(16, 8);
        let eo = EoId::new(0, 0);
        assert_eq!(w.next_deadline(), None);
        for ms in [50, 9, 3, 17, 3, 26] {
            w.insert(Time::ZERO, ms * MS, eo, TimerId(0), 0).unwrap();
        }
        let mut now = 0;
        while let Some(next) = w.next_deadline() {
            let scanned = w.entries.iter().filter(|e| e.live).map(|e| e.deadline).min().unwrap();
            assert_eq!(next, at_ms(scanned));
            now = now.max(scanned);
            fired(&mut w, at_ms(now));
        }
        assert_eq!(now, 50);
    }

    #[test]
    fn remove_is_generation_checked_and_capacity_bounded() {
        let mut w = wheel(1, 4);
        let eo = EoId::new(0, 0);
        let h = w.insert(Time::ZERO, MS, eo, TimerId(0), 1).unwrap();
        assert_eq!(w.insert(Time::ZERO, MS, eo, TimerId(0), 2), Err(Error::TimersExhausted));
        assert!(w.remove(h));
        assert!(!w.remove(h));
        let h2 = w.insert(Time::ZERO, MS, eo, TimerId(0), 3).unwrap();
        assert!(!w.remove(h), "stale handle must not remove the reused entry");
        assert_eq!(fired(&mut w, at_ms(1)), [3]);
        assert!(!w.remove(h2));
    }

    #[test]
    fn timer_table_ids_go_stale() {
        let mut t = TimerTable::new(1);
        let a = t.reserve().unwrap();
        assert_eq!(t.reserve(), Err(Error::TimersExhausted));
        assert!(t.cancel(a).is_some());
        assert!(t.cancel(a).is_none());
        let b = t.reserve().unwrap();
        assert_ne!(a, b);
        assert!(!t.take_fired(a));
        assert!(t.take_fired(b));
        assert!(!t.take_fired(b));
    }
}
