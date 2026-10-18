//! The UPF's sessions, found by uplink TEID (N3) and by UE address (N6).

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};

use librecn_sbi::policy::SessionRule;
use librecn_scheduler::Time;

use crate::qos::Qos;

/// One PDU session's forwarding state.
#[derive(Debug, PartialEq, Eq)]
pub struct Session {
    pub ue_ip: Ipv4Addr,
    pub ul_teid: u32,
    /// The PCF's policy: the extra QoS flows, the rates.
    pub qos: Qos,
    /// Its cell's slot in the UPF's traffic per cell, once known.
    pub cell: Option<u16>,
    /// The gNB's GTP-U endpoint and TEID, once known.
    pub dl: Option<(SocketAddrV4, u32)>,
    /// When downlink data without a tunnel was last reported to the SMF.
    pub reported: Option<Time>,
}

/// Why a session was not created.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Full,
    /// Another session has this UE address.
    AddressInUse,
}

/// Sessions by SEID, with both indexes; sized at creation.
pub struct Sessions {
    by_seid: HashMap<u64, Session>,
    by_teid: HashMap<u32, u64>,
    by_ue: HashMap<Ipv4Addr, u64>,
    capacity: usize,
    next_teid: u32,
}

impl Sessions {
    pub fn new(capacity: usize) -> Self {
        Sessions {
            by_seid: HashMap::with_capacity(capacity),
            by_teid: HashMap::with_capacity(capacity),
            by_ue: HashMap::with_capacity(capacity),
            capacity,
            next_teid: 1,
        }
    }

    pub fn len(&self) -> usize {
        self.by_seid.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_seid.is_empty()
    }

    /// The session `seid` for `ue_ip`, with a new uplink TEID; the existing
    /// one if `seid` is known (a retransmission).
    pub fn establish(&mut self, seid: u64, ue_ip: Ipv4Addr, rule: &SessionRule) -> Result<&Session, Refusal> {
        if self.by_seid.contains_key(&seid) {
            return Ok(&self.by_seid[&seid]);
        }
        if self.by_seid.len() >= self.capacity {
            return Err(Refusal::Full);
        }
        if self.by_ue.contains_key(&ue_ip) {
            return Err(Refusal::AddressInUse);
        }
        let ul_teid = loop {
            let teid = self.next_teid;
            self.next_teid = self.next_teid.wrapping_add(1).max(1);
            if !self.by_teid.contains_key(&teid) {
                break teid;
            }
        };
        self.by_teid.insert(ul_teid, seid);
        self.by_ue.insert(ue_ip, seid);
        Ok(self.by_seid.entry(seid).or_insert(Session {
            ue_ip,
            ul_teid,
            qos: Qos::new(rule),
            cell: None,
            dl: None,
            reported: None,
        }))
    }

    /// Sets the downlink tunnel (its data reported no longer), or clears it
    /// (the UE is idle); false if `seid` is unknown.
    pub fn modify(&mut self, seid: u64, dl: Option<(SocketAddrV4, u32)>) -> bool {
        self.by_seid.get_mut(&seid).map(|s| (s.dl, s.reported) = (dl, None)).is_some()
    }

    pub fn delete(&mut self, seid: u64) -> Option<Session> {
        let session = self.by_seid.remove(&seid)?;
        self.by_teid.remove(&session.ul_teid);
        self.by_ue.remove(&session.ue_ip);
        Some(session)
    }

    pub fn get(&self, seid: u64) -> Option<&Session> {
        self.by_seid.get(&seid)
    }

    pub fn get_mut(&mut self, seid: u64) -> Option<&mut Session> {
        self.by_seid.get_mut(&seid)
    }

    /// The session an uplink G-PDU with `teid` belongs to.
    pub fn by_teid(&mut self, teid: u32) -> Option<&mut Session> {
        self.by_teid.get(&teid).and_then(|seid| self.by_seid.get_mut(seid))
    }

    /// The session of the UE with address `ip` (downlink), with its SEID.
    pub fn by_ue(&mut self, ip: Ipv4Addr) -> Option<(u64, &mut Session)> {
        let seid = *self.by_ue.get(&ip)?;
        self.by_seid.get_mut(&seid).map(|s| (seid, s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zerocopy::FromZeros;

    #[test]
    fn sessions_are_found_both_ways_and_bounded() {
        let mut s = Sessions::new(2);
        let rule = SessionRule::new_zeroed();
        let ue = Ipv4Addr::new(10, 45, 0, 2);
        let a = s.establish(1, ue, &rule).unwrap().ul_teid;
        assert_eq!(s.establish(1, ue, &rule).unwrap().ul_teid, a, "retransmission");
        assert_eq!(s.establish(2, ue, &rule), Err(Refusal::AddressInUse));
        let b = s.establish(2, Ipv4Addr::new(10, 45, 0, 3), &rule).unwrap().ul_teid;
        assert_ne!(a, b);
        assert_eq!(s.establish(3, Ipv4Addr::new(10, 45, 0, 4), &rule), Err(Refusal::Full));
        let gnb = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 2152);
        assert!(s.modify(1, Some((gnb, 9))) && !s.modify(7, Some((gnb, 9))));
        assert_eq!(s.by_teid(a).unwrap().dl, Some((gnb, 9)));
        assert!(s.modify(1, None) && s.by_ue(ue).unwrap().1.dl.is_none());
        s.modify(1, Some((gnb, 9)));
        assert_eq!(s.by_ue(ue).unwrap().1.ul_teid, a);
        assert!(s.delete(1).is_some());
        assert!(s.by_teid(a).is_none() && s.by_ue(ue).is_none() && s.len() == 1);
    }
}
