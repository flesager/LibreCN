//! PDU sessions: AMF ↔ SMF (Nsmf_PDUSession, TS 29.502) and SMF ↔ UPF (N4).
//!
//! ```text
//! AMF ── CreateSmContext (N1: establishment request) ──▶ SMF ── N4Establish ──▶ UPF
//! AMF ◀─ CreateSmContextResult (N1 accept + N2 setup) ── SMF ◀─ N4EstablishResult (N3 tunnel)
//! AMF ── UpdateSmContext (N2: the gNB's tunnel) ───────▶ SMF ── N4Modify ─────▶ UPF
//! AMF ◀─ UpdateSmContextResult ───────────────────────── SMF ◀─ N4ModifyResult
//! AMF ── UpdateSmContext (deactivate: the UE is idle) ─▶ SMF ── N4Modify ─────▶ UPF (no downlink)
//! AMF ── UpdateSmContext (activate: the UE is back) ───▶ SMF
//! AMF ◀─ UpdateSmContextResult (N2 setup again) ──────── SMF
//! AMF ── UpdateSmContext (N1: release request...) ─────▶ SMF ── N4Delete ─────▶ UPF
//! AMF ◀─ UpdateSmContextResult (N1 command, N2 release) SMF
//! AMF ◀─ SmNotify (network-requested: N1, N2) ────────── SMF
//! AMF ── SmNotifyResult (delivered, or the UE is idle) ▶ SMF
//! AMF ── ReleaseSmContext ─────────────────────────────▶ SMF ── N4Delete ─────▶ UPF
//! ```
//!
//! N1 is a 5GSM message (NAS), N2 an NGAP transfer (PDU Session Resource
//! Setup Request / Response Transfer): the SMF encodes and decodes both, the
//! AMF only carries them. The SMF's answer to CreateSmContext carries what
//! Namf_Communication_N1N2MessageTransfer would: one message instead of two.
//! A reject is an N1 message without N2.
//!
//! N4 carries the session's outcome, not PFCP's rules (PDR, FAR, QER): one
//! uplink tunnel to N6, one downlink tunnel to the gNB, per session.

use librecn_common::msg::{family, id};
use librecn_scheduler::MsgBody;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::{DnnText, Outcome, Supi};

/// Largest N1 or N2 container carried.
pub const CONTAINER: usize = 512;

/// An N1 or N2 container: length and bytes.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Container {
    len: u16,
    bytes: [u8; CONTAINER],
}

impl Container {
    /// False (and empty) if `bytes` is longer than [`CONTAINER`].
    pub fn set(&mut self, bytes: &[u8]) -> bool {
        let fits = bytes.len() <= CONTAINER;
        let n = if fits { bytes.len() } else { 0 };
        self.bytes[..n].copy_from_slice(&bytes[..n]);
        self.len = n as u16;
        fits
    }

    pub fn get(&self) -> &[u8] {
        &self.bytes[..usize::from(self.len).min(CONTAINER)]
    }
}

/// A network slice: SST, and SD when `has_sd`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Slice {
    pub sst: u8,
    pub has_sd: u8,
    pub sd: [u8; 3],
}

/// AMF → SMF: Nsmf_PDUSession_CreateSMContext.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct CreateSmContext {
    pub supi: Supi,
    pub pdu_session_id: u8,
    pub slice: Slice,
    pub dnn: DnnText,
    pub n1: Container,
}

impl MsgBody for CreateSmContext {
    const MSG_ID: u32 = id(family::PDU_SESSION, 1);
}

/// SMF → AMF: the SM context, N1 for the UE and, when accepted, N2 for the gNB.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct CreateSmContextResult {
    outcome: u8,
    pub pdu_session_id: u8,
    _pad: [u8; 2],
    pub sm_ctx: u32,
    pub n1: Container,
    pub n2: Container,
}

impl MsgBody for CreateSmContextResult {
    const MSG_ID: u32 = id(family::PDU_SESSION, 2);
}

/// What an [`UpdateSmContext`] is about.
pub mod update {
    /// N2: PDU Session Resource Setup Response Transfer.
    pub const SETUP_RESPONSE: u8 = 1;
    /// N2: PDU Session Resource Setup Unsuccessful Transfer.
    pub const SETUP_FAILED: u8 = 2;
    /// The UE went idle: no downlink tunnel (upCnxState DEACTIVATED).
    pub const DEACTIVATE: u8 = 3;
    /// The UE is back: N2 to set the session up again (upCnxState ACTIVATING).
    pub const ACTIVATE: u8 = 4;
    /// N1: a 5GSM message of the UE for this session.
    pub const N1: u8 = 5;
}

/// What the N2 container of an answer is, for the NGAP procedure to run.
pub mod n2 {
    pub const NONE: u8 = 0;
    /// PDU Session Resource Setup Request Transfer.
    pub const SETUP: u8 = 1;
    /// PDU Session Resource Release Command Transfer.
    pub const RELEASE: u8 = 2;
}

/// AMF → SMF: Nsmf_PDUSession_UpdateSMContext.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct UpdateSmContext {
    pub sm_ctx: u32,
    /// [`update`] kind.
    pub kind: u8,
    _pad: u8,
    /// N2 (the gNB's answer) or N1 (the UE's message), per `kind`.
    pub container: Container,
}

impl MsgBody for UpdateSmContext {
    const MSG_ID: u32 = id(family::PDU_SESSION, 3);
}

/// SMF → AMF: N1 for the UE, N2 for the gNB ([`n2`] kind), either may be
/// empty; `released` when the session is over.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct UpdateSmContextResult {
    outcome: u8,
    pub pdu_session_id: u8,
    pub released: u8,
    pub n2_kind: u8,
    pub n1: Container,
    pub n2: Container,
}

impl MsgBody for UpdateSmContextResult {
    const MSG_ID: u32 = id(family::PDU_SESSION, 4);
}

/// AMF → SMF: Nsmf_PDUSession_ReleaseSMContext (the UE is gone); answered
/// by an UpdateSmContextResult, `released`.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct ReleaseSmContext {
    pub sm_ctx: u32,
}

impl MsgBody for ReleaseSmContext {
    const MSG_ID: u32 = id(family::PDU_SESSION, 5);
}

/// SMF → UPF: a session, for the UE's address.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct N4Establish {
    pub seid: u64,
    pub ue_ip: [u8; 4],
    _pad: [u8; 4],
}

impl N4Establish {
    pub fn new(seid: u64, ue_ip: [u8; 4]) -> Self {
        N4Establish { seid, ue_ip, _pad: [0; 4] }
    }
}

impl MsgBody for N4Establish {
    const MSG_ID: u32 = id(family::PDU_SESSION, 6);
}

/// UPF → SMF: the uplink tunnel endpoint the gNB sends to (N3).
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct N4EstablishResult {
    pub seid: u64,
    outcome: u8,
    _pad: [u8; 3],
    pub n3_addr: [u8; 4],
    pub ul_teid: u32,
    _pad2: [u8; 4],
}

impl MsgBody for N4EstablishResult {
    const MSG_ID: u32 = id(family::PDU_SESSION, 7);
}

/// SMF → UPF: the downlink tunnel endpoint, the gNB's; TEID 0: none (the
/// UE is idle, downlink packets are dropped).
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct N4Modify {
    pub seid: u64,
    pub gnb_addr: [u8; 4],
    pub dl_teid: u32,
}

impl MsgBody for N4Modify {
    const MSG_ID: u32 = id(family::PDU_SESSION, 8);
}

/// UPF → SMF.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct N4ModifyResult {
    pub seid: u64,
    outcome: u8,
    _pad: [u8; 7],
}

impl MsgBody for N4ModifyResult {
    const MSG_ID: u32 = id(family::PDU_SESSION, 9);
}

/// SMF → UPF: the session is gone. No answer.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct N4Delete {
    pub seid: u64,
}

impl MsgBody for N4Delete {
    const MSG_ID: u32 = id(family::PDU_SESSION, 10);
}

/// SMF → AMF, unsolicited (Namf_Communication_N1N2MessageTransfer): the
/// network releases a session. `corr_id` is the AMF's id of the UE when it
/// last spoke to the SMF.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct SmNotify {
    pub sm_ctx: u32,
    pub pdu_session_id: u8,
    pub n2_kind: u8,
    pub n1: Container,
    pub n2: Container,
    _pad: [u8; 2],
}

impl MsgBody for SmNotify {
    const MSG_ID: u32 = id(family::PDU_SESSION, 11);
}

/// AMF → SMF: delivered, or `ContextNotFound` if the UE cannot be reached
/// (idle or gone): the SMF then ends the session alone.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct SmNotifyResult {
    outcome: u8,
    _pad: [u8; 3],
    pub sm_ctx: u32,
}

impl MsgBody for SmNotifyResult {
    const MSG_ID: u32 = id(family::PDU_SESSION, 12);
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

outcome!(CreateSmContextResult, UpdateSmContextResult, SmNotifyResult, N4EstablishResult, N4ModifyResult);

#[cfg(test)]
mod tests {
    use super::*;
    use zerocopy::FromZeros;

    #[test]
    fn containers_are_bounded_and_bodies_fit_their_buffer_classes() {
        let mut c = Container::new_zeroed();
        assert!(c.set(&[1, 2, 3]) && c.get() == [1, 2, 3]);
        assert!(!c.set(&[0; CONTAINER + 1]) && c.get().is_empty());
        assert!(std::mem::size_of::<CreateSmContext>() <= 1024);
        assert!(std::mem::size_of::<CreateSmContextResult>() <= 2048);
        assert!(std::mem::size_of::<N4EstablishResult>() <= 256);
    }
}
