//! Bounded lock-free ring, preallocated at construction.
//!
//! Dmitry Vyukov's bounded queue: every slot carries a sequence number that
//! tells producers and consumers whose turn it is, so a slot is owned by
//! exactly one thread between a successful position claim and the sequence
//! store that hands it over. Safe for any number of producers and consumers;
//! mailboxes use it as MPSC, free lists as MPMC.
//!
//! Sequences count in half steps so that capacity 1 works: a slot is free for
//! position `p` when `seq == 2p` and holds the value pushed at `p` when
//! `seq == 2p + 1`. With the textbook `p` / `p + 1` encoding, "published at
//! `p`" and "free for `p + capacity`" are the same value when capacity is 1.

use std::mem::MaybeUninit;

use crate::sync::{AtomicUsize, Ordering, UnsafeCell};

/// Keeps the producer and consumer cursors on separate cache lines
/// (128 bytes covers the adjacent-line prefetcher on x86_64).
#[repr(align(128))]
struct CachePadded<T>(T);

const fn free_for(pos: usize) -> usize {
    pos.wrapping_mul(2)
}

const fn published_at(pos: usize) -> usize {
    pos.wrapping_mul(2).wrapping_add(1)
}

struct Slot<T> {
    seq: AtomicUsize,
    value: UnsafeCell<MaybeUninit<T>>,
}

/// Bounded ring of exactly `capacity` slots.
pub struct Ring<T> {
    slots: Box<[Slot<T>]>,
    /// Next position to pop.
    head: CachePadded<AtomicUsize>,
    /// Next position to push.
    tail: CachePadded<AtomicUsize>,
}

// SAFETY: a slot's value is only touched by the thread that claimed its position
// with the cursor CAS; the Release store / Acquire load of `seq` hands the slot
// over between producer and consumer, so values only move between threads.
unsafe impl<T: Send> Send for Ring<T> {}
// SAFETY: see `Send`; shared access never yields `&T` to two threads.
unsafe impl<T: Send> Sync for Ring<T> {}

impl<T> Ring<T> {
    /// Allocates and initialises every slot up front.
    ///
    /// # Panics
    /// If `capacity` is 0.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "ring capacity must be at least 1");
        let slots = (0..capacity)
            .map(|i| Slot { seq: AtomicUsize::new(free_for(i)), value: UnsafeCell::new(MaybeUninit::uninit()) })
            .collect();
        Ring { slots, head: CachePadded(AtomicUsize::new(0)), tail: CachePadded(AtomicUsize::new(0)) }
    }

    /// Number of slots.
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    /// Pushes `value`, or hands it back if the ring is full.
    pub fn push(&self, value: T) -> Result<(), T> {
        let cap = self.slots.len();
        let mut pos = self.tail.0.load(Ordering::Relaxed);
        loop {
            let slot = &self.slots[pos % cap];
            let seq = slot.seq.load(Ordering::Acquire);
            let diff = seq.wrapping_sub(free_for(pos)) as isize;
            if diff == 0 {
                match self.tail.0.compare_exchange_weak(pos, pos.wrapping_add(1), Ordering::Relaxed, Ordering::Relaxed)
                {
                    Ok(_) => {
                        // SAFETY: the CAS made this thread the only owner of the slot
                        // until the `seq` store below publishes it.
                        slot.value.with_mut(|p| unsafe { (*p).write(value) });
                        slot.seq.store(published_at(pos), Ordering::Release);
                        return Ok(());
                    }
                    Err(current) => pos = current,
                }
            } else if diff < 0 {
                // The slot still holds the value from the previous lap.
                return Err(value);
            } else {
                pos = self.tail.0.load(Ordering::Relaxed);
            }
        }
    }

    /// Pops the oldest value, if any.
    pub fn pop(&self) -> Option<T> {
        let cap = self.slots.len();
        let mut pos = self.head.0.load(Ordering::Relaxed);
        loop {
            let slot = &self.slots[pos % cap];
            let seq = slot.seq.load(Ordering::Acquire);
            let diff = seq.wrapping_sub(published_at(pos)) as isize;
            if diff == 0 {
                match self.head.0.compare_exchange_weak(pos, pos.wrapping_add(1), Ordering::Relaxed, Ordering::Relaxed)
                {
                    Ok(_) => {
                        // SAFETY: `seq == published_at(pos)` means a producer wrote the value and
                        // published it; the CAS made this thread its only consumer.
                        let value = slot.value.with(|p| unsafe { (*p).assume_init_read() });
                        slot.seq.store(free_for(pos.wrapping_add(cap)), Ordering::Release);
                        return Some(value);
                    }
                    Err(current) => pos = current,
                }
            } else if diff < 0 {
                // Empty, or the producer that claimed this slot has not published yet.
                return None;
            } else {
                pos = self.head.0.load(Ordering::Relaxed);
            }
        }
    }

    /// Number of claimed slots. Exact when quiescent, approximate under concurrency.
    pub fn len(&self) -> usize {
        // Head first: head never overtakes a tail read after it.
        let head = self.head.0.load(Ordering::Relaxed);
        let tail = self.tail.0.load(Ordering::Relaxed);
        tail.wrapping_sub(head).min(self.slots.len())
    }

    /// Memory held by the slots.
    pub fn bytes_reserved(&self) -> usize {
        std::mem::size_of_val(&*self.slots)
    }

    /// Whether [`len`](Self::len) is 0.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Drop for Ring<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

impl<T> std::fmt::Debug for Ring<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ring").field("capacity", &self.capacity()).field("len", &self.len()).finish()
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Arc;

    use proptest::prelude::*;

    use super::*;

    #[test]
    fn fifo_full_and_empty() {
        let ring = Ring::new(3);
        assert_eq!(ring.pop(), None);
        for i in 0..3 {
            ring.push(i).unwrap();
        }
        assert_eq!(ring.push(9), Err(9));
        assert_eq!(ring.len(), 3);
        assert_eq!((ring.pop(), ring.pop(), ring.pop(), ring.pop()), (Some(0), Some(1), Some(2), None));
    }

    #[test]
    fn wraps_around_many_laps() {
        let ring = Ring::new(5);
        for i in 0..1_000 {
            ring.push(i).unwrap();
            ring.push(i + 1).unwrap();
            assert_eq!(ring.pop(), Some(i));
            assert_eq!(ring.pop(), Some(i + 1));
        }
        assert!(ring.is_empty());
    }

    #[test]
    fn drops_remaining_values() {
        let marker = Arc::new(());
        let ring = Ring::new(4);
        ring.push(marker.clone()).unwrap();
        ring.push(marker.clone()).unwrap();
        drop(ring);
        assert_eq!(Arc::strong_count(&marker), 1);
    }

    #[test]
    fn concurrent_producers_keep_per_producer_order() {
        const PRODUCERS: u64 = 4;
        const PER: u64 = 50_000;
        let ring = Arc::new(Ring::new(64));
        let handles: Vec<_> = (0..PRODUCERS)
            .map(|p| {
                let ring = ring.clone();
                std::thread::spawn(move || {
                    for i in 0..PER {
                        let mut v = (p << 32) | i;
                        while let Err(back) = ring.push(v) {
                            v = back;
                            std::hint::spin_loop();
                        }
                    }
                })
            })
            .collect();
        let mut next = [0u64; PRODUCERS as usize];
        let mut received = 0;
        while received < PRODUCERS * PER {
            if let Some(v) = ring.pop() {
                let (p, i) = ((v >> 32) as usize, v & 0xFFFF_FFFF);
                assert_eq!(i, next[p], "producer {p} out of order");
                next[p] += 1;
                received += 1;
            }
        }
        handles.into_iter().for_each(|h| h.join().unwrap());
        assert!(ring.is_empty());
    }

    proptest! {
        #[test]
        fn behaves_like_a_bounded_deque(cap in 1usize..8, ops in prop::collection::vec(any::<Option<u8>>(), 0..200)) {
            let ring = Ring::new(cap);
            let mut model = VecDeque::new();
            for op in ops {
                match op {
                    Some(v) => {
                        let expected = if model.len() < cap { model.push_back(v); Ok(()) } else { Err(v) };
                        prop_assert_eq!(ring.push(v), expected);
                    }
                    None => prop_assert_eq!(ring.pop(), model.pop_front()),
                }
                prop_assert_eq!(ring.len(), model.len());
            }
        }
    }
}

/// Run with `RUSTFLAGS="--cfg loom" cargo test -p librecn-scheduler --release --lib queue::loom_tests`.
#[cfg(all(test, loom))]
mod loom_tests {
    use loom::sync::Arc;
    use loom::thread;

    use super::*;

    /// Pops concurrently with two producers: nothing lost or duplicated.
    #[test]
    fn two_producers_one_consumer() {
        loom::model(|| {
            let ring = Arc::new(Ring::new(2));
            let producers: Vec<_> = (0..2u32)
                .map(|p| {
                    let ring = ring.clone();
                    thread::spawn(move || ring.push(p).unwrap())
                })
                .collect();
            let mut got = Vec::new();
            for _ in 0..2 {
                got.extend(ring.pop());
            }
            producers.into_iter().for_each(|h| h.join().unwrap());
            while let Some(v) = ring.pop() {
                got.push(v);
            }
            got.sort();
            assert_eq!(got, [0, 1]);
        });
    }

    /// One producer laps a 2-slot ring while the consumer pops: exactly the
    /// accepted values come out, in order.
    #[test]
    fn fifo_across_wrap_around() {
        loom::model(|| {
            let ring = Arc::new(Ring::new(2));
            let producer = {
                let ring = ring.clone();
                thread::spawn(move || (0..3u32).filter(|&v| ring.push(v).is_ok()).collect::<Vec<_>>())
            };
            let mut got = Vec::new();
            for _ in 0..3 {
                got.extend(ring.pop());
            }
            let pushed = producer.join().unwrap();
            while let Some(v) = ring.pop() {
                got.push(v);
            }
            assert_eq!(got, pushed);
        });
    }

    #[test]
    fn full_ring_hands_value_back() {
        loom::model(|| {
            let ring = Arc::new(Ring::new(1));
            ring.push(1).unwrap();
            let consumer = {
                let ring = ring.clone();
                thread::spawn(move || ring.pop())
            };
            let pushed = ring.push(2);
            let popped = consumer.join().unwrap();
            assert_eq!(popped, Some(1));
            match pushed {
                Ok(()) => assert_eq!(ring.pop(), Some(2)),
                Err(v) => assert_eq!(v, 2),
            }
        });
    }
}
