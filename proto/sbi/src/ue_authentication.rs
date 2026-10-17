//! 5G AKA (TS 33.501 §6.1.3.2) between AMF (SEAF), AUSF and UDM.
//!
//! ```text
//! AMF ── Authenticate ──────▶ AUSF ── GetAuthData ──▶ UDM
//! AMF ◀─ AuthenticateResult ─ AUSF ◀─ AuthData ────── UDM
//! AMF ── Confirm ───────────▶ AUSF
//! AMF ◀─ ConfirmResult ────── AUSF
//! ```
//!
//! The AUSF keeps XRES* and K_SEAF until the AMF confirms with RES*: K_SEAF
//! and the SUPI reach the AMF only after the UE proved itself.

use librecn_common::msg::{family, id};
use librecn_scheduler::MsgBody;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::{Outcome, ServingNetwork, Supi, SupiOrSuci};

/// Who to authenticate, for which network (AuthenticationInfo, TS 29.509
/// §6.1.6.2.2; AuthenticationInfoRequest, TS 29.503 §6.3.6.2.2).
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct AuthInfo {
    pub supi_or_suci: SupiOrSuci,
    pub serving_network: ServingNetwork,
    resync: u8,
    /// Resynchronisation: the RAND of the failed challenge...
    pub rand: [u8; 16],
    /// ...and the AUTS the UE answered.
    pub auts: [u8; 14],
    _pad: u8,
}

impl AuthInfo {
    pub fn new(supi_or_suci: SupiOrSuci, serving_network: ServingNetwork) -> Self {
        AuthInfo { supi_or_suci, serving_network, resync: 0, rand: [0; 16], auts: [0; 14], _pad: 0 }
    }

    /// With resynchronisation info (after a synch failure).
    pub fn with_resync(mut self, rand: [u8; 16], auts: [u8; 14]) -> Self {
        (self.resync, self.rand, self.auts) = (1, rand, auts);
        self
    }

    /// RAND and AUTS, when resynchronising.
    pub fn resync(&self) -> Option<(&[u8; 16], &[u8; 14])> {
        (self.resync != 0).then_some((&self.rand, &self.auts))
    }
}

/// AMF → AUSF: Nausf_UEAuthentication_Authenticate.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Authenticate {
    pub info: AuthInfo,
}

impl MsgBody for Authenticate {
    const MSG_ID: u32 = id(family::UE_AUTHENTICATION, 1);
}

/// AUSF → AMF: the 5G AKA challenge (5gAuthData, TS 29.509 §6.1.6.2.3).
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct AuthenticateResult {
    outcome: u8,
    _pad: [u8; 3],
    /// The AUSF's context, to confirm.
    pub auth_ctx: u32,
    pub rand: [u8; 16],
    pub autn: [u8; 16],
    pub hxres_star: [u8; 16],
}

impl MsgBody for AuthenticateResult {
    const MSG_ID: u32 = id(family::UE_AUTHENTICATION, 2);
}

/// AMF → AUSF: 5G AKA confirmation with the UE's RES*.
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Confirm {
    pub auth_ctx: u32,
    pub res_star: [u8; 16],
}

impl MsgBody for Confirm {
    const MSG_ID: u32 = id(family::UE_AUTHENTICATION, 3);
}

/// AUSF → AMF: confirmation result (ConfirmationDataResponse, TS 29.509 §6.1.6.2.6).
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct ConfirmResult {
    outcome: u8,
    pub supi: Supi,
    pub kseaf: [u8; 32],
}

impl MsgBody for ConfirmResult {
    const MSG_ID: u32 = id(family::UE_AUTHENTICATION, 4);
}

/// AUSF → UDM: Nudm_UEAuthentication_Get (generate-auth-data).
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct GetAuthData {
    pub info: AuthInfo,
}

impl MsgBody for GetAuthData {
    const MSG_ID: u32 = id(family::UE_AUTHENTICATION, 5);
}

/// UDM → AUSF: a 5G home environment authentication vector and the SUPI
/// (AuthenticationInfoResult, TS 29.503 §6.3.6.2.3).
#[derive(Clone, Copy, Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct AuthData {
    outcome: u8,
    pub supi: Supi,
    pub rand: [u8; 16],
    pub autn: [u8; 16],
    pub xres_star: [u8; 16],
    pub kausf: [u8; 32],
}

impl MsgBody for AuthData {
    const MSG_ID: u32 = id(family::UE_AUTHENTICATION, 6);
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

outcome!(AuthenticateResult, ConfirmResult, AuthData);

#[cfg(test)]
mod tests {
    use super::*;
    use zerocopy::FromZeros;

    #[test]
    fn bodies_are_small_and_zeroed_bodies_read_as_ok_without_resync() {
        assert_eq!(std::mem::size_of::<AuthInfo>(), 256);
        assert!(std::mem::size_of::<ConfirmResult>() <= 128 && std::mem::size_of::<AuthData>() <= 256);
        let mut result = AuthenticateResult::new_zeroed();
        assert_eq!(result.outcome(), Outcome::Ok);
        result.set_outcome(Outcome::UserNotFound);
        assert_eq!(result.outcome(), Outcome::UserNotFound);
        let info = AuthInfo::new(SupiOrSuci::new("imsi-1").unwrap(), ServingNetwork::new("5G:x").unwrap());
        assert!(info.resync().is_none());
        assert_eq!(info.with_resync([1; 16], [2; 14]).resync(), Some((&[1; 16], &[2; 14])));
    }
}
