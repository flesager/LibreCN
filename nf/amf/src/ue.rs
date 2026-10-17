//! UE contexts: the binding between a UE's AMF-UE-NGAP-ID (allocated here)
//! and its gNB, RAN-UE-NGAP-ID and SCTP stream (TS 38.413 §9.3.3.1–2).

use std::collections::HashMap;

use crate::config::{Plmn, Snssai};
use crate::mm::Mm;
use crate::sctp::AssocId;

/// AMF-UE-NGAP-ID: 40 bits in NGAP, unique in this AMF. Allocated within 32
/// bits, so that it also serves as the `corr_id` of the UE's SBI requests.
pub type AmfUeId = u64;
/// RAN-UE-NGAP-ID: 32 bits, unique in its gNB.
pub type RanUeId = u32;

const AMF_UE_ID_MAX: AmfUeId = u32::MAX as AmfUeId;

/// Where a UE is, from the last User Location Information.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    /// Tracking area: PLMN and TAC.
    pub tai: (Plmn, [u8; 3]),
    /// NR cell: PLMN and 36-bit NR Cell Identity.
    pub cell: (Plmn, u64),
}

/// A PDU session of the UE, as the AMF knows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PduSession {
    pub snssai: Snssai,
    /// The SMF's context, once it answered.
    pub sm_ctx: Option<u32>,
}

/// One UE context.
#[derive(Clone, Debug)]
pub struct UeContext {
    pub amf_id: AmfUeId,
    pub ran_id: RanUeId,
    /// gNB association.
    pub assoc: AssocId,
    /// SCTP stream for this UE's signalling (never 0, kept for non-UE signalling).
    pub stream: u16,
    pub location: Option<Location>,
    /// A UE Context Release Command was sent; waiting for the Complete.
    pub releasing: bool,
    /// Its 5GMM context, which lives as long as the N2 context for now.
    pub mm: Mm,
    /// PDU sessions by PDU session ID (1 to 15).
    pub sessions: [Option<PduSession>; 16],
}

/// Why an id pair does not match a context (TS 38.413 §10.6 causes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mismatch {
    /// No context with this AMF-UE-NGAP-ID.
    UnknownAmfId,
    /// The context exists but its RAN-UE-NGAP-ID or gNB differ.
    InconsistentRanId,
}

/// All UE contexts, bounded.
#[derive(Debug)]
pub struct UeTable {
    ues: HashMap<AmfUeId, UeContext>,
    next: AmfUeId,
    capacity: usize,
}

impl UeTable {
    pub fn new(capacity: usize) -> Self {
        UeTable { ues: HashMap::with_capacity(capacity), next: 1, capacity }
    }

    pub fn len(&self) -> usize {
        self.ues.len()
    }

    /// UEs in state 5GMM-REGISTERED.
    pub fn registered(&self) -> usize {
        self.ues.values().filter(|ue| ue.mm.state() == crate::mm::State::Registered).count()
    }

    /// New context for a UE arriving on `assoc` (`streams`: outbound streams
    /// of that association). `None` when the table is full.
    pub fn create(&mut self, assoc: AssocId, ran_id: RanUeId, streams: u16) -> Option<&UeContext> {
        if self.ues.len() >= self.capacity {
            return None;
        }
        while self.ues.contains_key(&self.next) {
            self.next = if self.next == AMF_UE_ID_MAX { 1 } else { self.next + 1 };
        }
        let amf_id = self.next;
        self.next = if self.next == AMF_UE_ID_MAX { 1 } else { self.next + 1 };
        // Spread UEs over streams 1.. (stream 0 is for non-UE signalling).
        let stream = if streams > 1 { 1 + (amf_id % u64::from(streams - 1)) as u16 } else { 0 };
        let ue = UeContext {
            amf_id,
            ran_id,
            assoc,
            stream,
            location: None,
            releasing: false,
            mm: Mm::default(),
            sessions: [None; 16],
        };
        Some(self.ues.entry(amf_id).or_insert(ue))
    }

    /// The context these ids, received from `assoc`, designate.
    pub fn get_mut(&mut self, assoc: AssocId, amf_id: AmfUeId, ran_id: RanUeId) -> Result<&mut UeContext, Mismatch> {
        let ue = self.ues.get_mut(&amf_id).ok_or(Mismatch::UnknownAmfId)?;
        if ue.ran_id != ran_id || ue.assoc != assoc {
            return Err(Mismatch::InconsistentRanId);
        }
        Ok(ue)
    }

    pub fn get_mut_by_id(&mut self, amf_id: AmfUeId) -> Option<&mut UeContext> {
        self.ues.get_mut(&amf_id)
    }

    /// The context of RAN-UE-NGAP-ID `ran_id` on `assoc`, if any.
    pub fn find(&self, assoc: AssocId, ran_id: RanUeId) -> Option<AmfUeId> {
        self.ues.values().find(|ue| ue.assoc == assoc && ue.ran_id == ran_id).map(|ue| ue.amf_id)
    }

    pub fn remove(&mut self, amf_id: AmfUeId) -> Option<UeContext> {
        self.ues.remove(&amf_id)
    }

    /// Removes every context of a gNB (association lost, NG Reset).
    pub fn remove_gnb(&mut self, assoc: AssocId) -> Vec<UeContext> {
        self.ues.extract_if(|_, ue| ue.assoc == assoc).map(|(_, ue)| ue).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_streams_skip_zero_and_capacity_holds() {
        let mut t = UeTable::new(3);
        let a = t.create(7, 100, 4).unwrap().clone();
        let b = t.create(7, 101, 4).unwrap().clone();
        assert_ne!(a.amf_id, b.amf_id);
        assert!([a.stream, b.stream].iter().all(|&s| (1..4).contains(&s)));
        assert_eq!(t.create(8, 5, 1).unwrap().stream, 0, "single-stream association");
        assert!(t.create(8, 6, 4).is_none(), "full");
        assert_eq!(t.find(7, 101), Some(b.amf_id));
        assert_eq!(t.remove_gnb(7).len(), 2);
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn ids_are_checked_against_the_gnb_and_ran_id() {
        let mut t = UeTable::new(4);
        let ue = t.create(1, 42, 2).unwrap().amf_id;
        assert!(t.get_mut(1, ue, 42).is_ok());
        assert_eq!(t.get_mut(1, ue, 43).unwrap_err(), Mismatch::InconsistentRanId);
        assert_eq!(t.get_mut(2, ue, 42).unwrap_err(), Mismatch::InconsistentRanId, "other gNB");
        assert_eq!(t.get_mut(1, ue + 1, 42).unwrap_err(), Mismatch::UnknownAmfId);
    }

    #[test]
    fn allocation_wraps_within_32_bits_and_skips_live_ids() {
        let mut t = UeTable::new(4);
        t.next = AMF_UE_ID_MAX;
        assert_eq!(t.create(1, 1, 2).unwrap().amf_id, AMF_UE_ID_MAX);
        let kept = t.create(1, 2, 2).unwrap().amf_id;
        assert_eq!(kept, 1, "wrapped, 0 skipped");
        t.next = 1;
        assert_eq!(t.create(1, 3, 2).unwrap().amf_id, 2, "1 is in use");
    }
}
