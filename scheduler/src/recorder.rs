//! Flight recorder: the last headers an EO processed, for failure reports.

use crate::{EoId, MsgHeader};

/// Fixed ring of headers, written only by the thread running the EO.
pub(crate) struct FlightRecorder {
    ring: Box<[MsgHeader]>,
    next: usize,
    len: usize,
}

impl FlightRecorder {
    pub(crate) fn new(depth: usize) -> Self {
        let empty = MsgHeader::new(0, EoId::SYSTEM, EoId::SYSTEM);
        FlightRecorder { ring: vec![empty; depth].into_boxed_slice(), next: 0, len: 0 }
    }

    pub(crate) fn record(&mut self, header: &MsgHeader) {
        self.ring[self.next] = *header;
        self.next = (self.next + 1) % self.ring.len();
        self.len = (self.len + 1).min(self.ring.len());
    }

    /// Recorded headers, oldest first.
    pub(crate) fn history(&self) -> impl Iterator<Item = &MsgHeader> {
        let cap = self.ring.len();
        let start = (self.next + cap - self.len) % cap;
        (0..self.len).map(move |i| &self.ring[(start + i) % cap])
    }

    pub(crate) fn bytes_reserved(&self) -> usize {
        std::mem::size_of_val(&*self.ring)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_last_headers_in_order() {
        let mut r = FlightRecorder::new(3);
        assert_eq!(r.history().count(), 0);
        for id in 1..=5 {
            r.record(&MsgHeader::new(id, EoId::SYSTEM, EoId::SYSTEM));
        }
        assert_eq!(r.history().map(|h| h.msg_id).collect::<Vec<_>>(), [3, 4, 5]);
    }
}
