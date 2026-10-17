//! Downlink packets for idle UEs (TS 23.502 §4.2.3.3: the UPF buffers),
//! kept until their session's tunnel is back: a few packets in all,
//! preallocated, the oldest dropped first.

use std::collections::VecDeque;

/// Packets kept, all sessions together.
pub const SLOTS: usize = 64;
/// Largest packet kept; a larger one is dropped.
pub const SLOT: usize = 2048;

/// What became of a packet offered to [`Parked::park`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Parking {
    Kept,
    /// Kept, the oldest packet dropped to make room.
    KeptOldestDropped,
    TooLarge,
}

pub struct Parked {
    slots: Box<[u8]>,
    /// Kept packets in arrival order: SEID, slot, length.
    queue: VecDeque<(u64, u16, u16)>,
    free: Vec<u16>,
}

impl Default for Parked {
    fn default() -> Self {
        Parked {
            slots: vec![0; SLOTS * SLOT].into_boxed_slice(),
            queue: VecDeque::with_capacity(SLOTS),
            free: (0..SLOTS as u16).rev().collect(),
        }
    }
}

impl Parked {
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Keeps `packet` for session `seid`.
    pub fn park(&mut self, seid: u64, packet: &[u8]) -> Parking {
        if packet.len() > SLOT {
            return Parking::TooLarge;
        }
        let mut outcome = Parking::Kept;
        let slot = match self.free.pop() {
            Some(slot) => slot,
            None => {
                outcome = Parking::KeptOldestDropped;
                self.queue.pop_front().expect("no free slot: some are queued").1
            }
        };
        let at = usize::from(slot) * SLOT;
        self.slots[at..at + packet.len()].copy_from_slice(packet);
        self.queue.push_back((seid, slot, packet.len() as u16));
        outcome
    }

    /// Hands the packets of `seid` to `f`, in arrival order, and frees them.
    pub fn release(&mut self, seid: u64, mut f: impl FnMut(&[u8])) {
        let (slots, free) = (&self.slots, &mut self.free);
        self.queue.retain(|&(s, slot, len)| {
            if s != seid {
                return true;
            }
            let at = usize::from(slot) * SLOT;
            f(&slots[at..at + usize::from(len)]);
            free.push(slot);
            false
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packets_are_kept_per_session_in_order_the_oldest_dropped_when_full() {
        let mut p = Parked::default();
        assert_eq!(p.park(1, b"a1"), Parking::Kept);
        assert_eq!(p.park(2, b"b1"), Parking::Kept);
        assert_eq!(p.park(1, b"a2"), Parking::Kept);
        assert_eq!(p.park(1, &[0; SLOT + 1]), Parking::TooLarge);
        let mut got = Vec::new();
        p.release(1, |packet| got.push(packet.to_vec()));
        assert_eq!(got, [b"a1".to_vec(), b"a2".to_vec()]);
        assert_eq!(p.len(), 1, "session 2's packet stays");
        for n in 0..SLOTS - 1 {
            assert_eq!(p.park(3, &[n as u8]), Parking::Kept);
        }
        assert_eq!(p.park(3, b"x"), Parking::KeptOldestDropped, "full: b1 goes");
        p.release(2, |_| panic!("dropped"));
        let mut n = 0;
        p.release(3, |_| n += 1);
        assert_eq!((n, p.len()), (SLOTS, 0));
    }
}
