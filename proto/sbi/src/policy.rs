//! Policy control between the AMF, the SMF and the PCF: AM policy
//! (Npcf_AMPolicyControl, TS 29.507) and SM policy (Npcf_SMPolicyControl,
//! TS 29.512).
//!
//! ```text
//! AMF ── AmPolicyCreate (SUPI, at registration) ──▶ PCF
//! AMF ◀─ AmPolicyResult (UE-AMBR) ──────────────── PCF
//! AMF ◀─ AmPolicyUpdate (the subscriber's policy changed) ─ PCF
//! SMF ── SmPolicyCreate (SUPI, DNN, slice, at session establishment) ──▶ PCF
//! SMF ◀─ SmPolicyResult (session rule, PCC rules) ─ PCF
//! ```
//!
//! A session rule gives the session AMBR and the default QoS flow's 5QI and
//! ARP; each PCC rule an extra QoS flow, its packet filters and bit rates.
//! A changed SM policy applies to the sessions set up after it.

use librecn_common::msg::{family, id};
use librecn_scheduler::MsgBody;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::pdu_session::Slice;
use crate::{DnnText, Outcome, Supi};

/// PCC rules per session, at most.
pub const MAX_RULES: usize = 4;
/// Packet filters per PCC rule, at most.
pub const MAX_FILTERS: usize = 4;

/// Bit rates in bit/s.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Rates {
    pub uplink: u64,
    pub downlink: u64,
}

impl Rates {
    pub fn is_zero(&self) -> bool {
        self.uplink == 0 && self.downlink == 0
    }
}

/// Packet filter directions (TS 24.501 §9.11.4.13).
pub mod direction {
    pub const DOWNLINK: u8 = 1;
    pub const UPLINK: u8 = 2;
    pub const BOTH: u8 = 3;
}

/// A packet filter: what a packet must match, seen from the UE ("remote"
/// is the other end). Zeros match anything.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct PacketFilter {
    /// [`direction`].
    pub direction: u8,
    /// IP protocol (6 TCP, 17 UDP...); 0: any.
    pub protocol: u8,
    /// Prefix length of `remote`; 0: any address.
    pub remote_prefix: u8,
    _pad: u8,
    pub remote: [u8; 4],
    /// First and last port; 0, 0: any.
    pub remote_ports: [u16; 2],
    pub local_ports: [u16; 2],
}

impl PacketFilter {
    pub fn new(
        direction: u8,
        protocol: u8,
        remote: ([u8; 4], u8),
        remote_ports: [u16; 2],
        local_ports: [u16; 2],
    ) -> Self {
        PacketFilter {
            direction,
            protocol,
            remote_prefix: remote.1,
            _pad: 0,
            remote: remote.0,
            remote_ports,
            local_ports,
        }
    }
}

/// A PCC rule: an extra QoS flow, the packets of its filters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct PccRule {
    /// Maximum bit rate; zeros: none.
    pub mbr: Rates,
    /// Guaranteed bit rate; zeros: a non-GBR flow.
    pub gbr: Rates,
    pub five_qi: u8,
    /// ARP priority level (1 highest, 15 lowest).
    pub arp: u8,
    /// Evaluation order among the session's rules, lowest first.
    pub precedence: u8,
    filters_len: u8,
    _pad: [u8; 4],
    filters: [PacketFilter; MAX_FILTERS],
}

impl PccRule {
    /// False if there are more than [`MAX_FILTERS`] filters.
    pub fn set_filters(&mut self, filters: &[PacketFilter]) -> bool {
        let Some(slots) = self.filters.get_mut(..filters.len()) else { return false };
        slots.copy_from_slice(filters);
        self.filters_len = filters.len() as u8;
        true
    }

    pub fn filters(&self) -> &[PacketFilter] {
        &self.filters[..usize::from(self.filters_len).min(MAX_FILTERS)]
    }
}

/// The policy of one session (SmPolicyDecision: session rule, PCC rules).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct SessionRule {
    /// Session AMBR.
    pub ambr: Rates,
    /// The default QoS flow.
    pub five_qi: u8,
    pub arp: u8,
    rules_len: u8,
    _pad: [u8; 5],
    rules: [PccRule; MAX_RULES],
}

impl SessionRule {
    /// False if there are more than [`MAX_RULES`] rules.
    pub fn set_rules(&mut self, rules: &[PccRule]) -> bool {
        let Some(slots) = self.rules.get_mut(..rules.len()) else { return false };
        slots.copy_from_slice(rules);
        self.rules_len = rules.len() as u8;
        true
    }

    pub fn rules(&self) -> &[PccRule] {
        &self.rules[..usize::from(self.rules_len).min(MAX_RULES)]
    }
}

/// AMF → PCF: Npcf_AMPolicyControl_Create, the UE registered.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct AmPolicyCreate {
    pub supi: Supi,
}

impl MsgBody for AmPolicyCreate {
    const MSG_ID: u32 = id(family::POLICY, 1);
}

/// PCF → AMF: the UE's access and mobility policy.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct AmPolicyResult {
    pub ue_ambr: Rates,
    outcome: u8,
    _pad: [u8; 7],
}

impl MsgBody for AmPolicyResult {
    const MSG_ID: u32 = id(family::POLICY, 2);
}

/// PCF → AMF, unsolicited (Npcf_AMPolicyControl_UpdateNotify): the
/// subscriber's policy changed.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct AmPolicyUpdate {
    pub ue_ambr: Rates,
    pub supi: Supi,
}

impl MsgBody for AmPolicyUpdate {
    const MSG_ID: u32 = id(family::POLICY, 3);
}

/// SMF → PCF: Npcf_SMPolicyControl_Create, a session is being set up.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct SmPolicyCreate {
    pub supi: Supi,
    pub dnn: DnnText,
    pub slice: Slice,
    pub pdu_session_id: u8,
}

impl MsgBody for SmPolicyCreate {
    const MSG_ID: u32 = id(family::POLICY, 4);
}

/// PCF → SMF: the session's policy.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct SmPolicyResult {
    pub rule: SessionRule,
    outcome: u8,
    _pad: [u8; 7],
}

impl MsgBody for SmPolicyResult {
    const MSG_ID: u32 = id(family::POLICY, 5);
}

macro_rules! outcome {
    ($($ty:ident),*) => {$(
        impl $ty {
            pub fn outcome(&self) -> Outcome {
                Outcome::from_u8(self.outcome)
            }

            pub fn set_outcome(&mut self, outcome: Outcome) {
                self.outcome = outcome as u8;
            }
        }
    )*};
}

outcome!(AmPolicyResult, SmPolicyResult);

#[cfg(test)]
mod tests {
    use super::*;
    use zerocopy::FromZeros;

    #[test]
    fn rules_and_filters_are_bounded_and_bodies_fit_their_classes() {
        let mut rule = PccRule::new_zeroed();
        let filter = PacketFilter::new(direction::BOTH, 17, ([10, 10, 0, 0], 16), [5060, 5061], [0, 0]);
        assert!(rule.set_filters(&[filter; 2]) && rule.filters() == [filter; 2]);
        assert!(!rule.set_filters(&[filter; MAX_FILTERS + 1]));
        let mut session = SessionRule::new_zeroed();
        assert!(session.set_rules(&[rule]) && session.rules().len() == 1);
        assert!(!session.set_rules(&[rule; MAX_RULES + 1]));
        assert!(std::mem::size_of::<SmPolicyResult>() <= 1024 && std::mem::size_of::<SmPolicyCreate>() <= 256);
        assert!(std::mem::size_of::<AmPolicyUpdate>() <= 256);
    }
}
