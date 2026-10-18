//! The service-based interfaces between NFs, as EO messages.
//!
//! Each service operation is a pair of fixed-layout bodies, request and
//! response, paired by `corr_id` and marked with the `REQUEST` / `RESPONSE`
//! flags of `librecn_common::msg::flags`. The bodies carry what the 3GPP
//! operation carries, in the same terms; an HTTP/2 binding can come later
//! without changing the NFs' logic.
//!
//! - [`ue_authentication`]: Nausf_UEAuthentication (TS 29.509) and
//!   Nudm_UEAuthentication (TS 29.503), 5G AKA and EAP-AKA';
//! - [`policy`]: Npcf_AMPolicyControl (TS 29.507) and Npcf_SMPolicyControl
//!   (TS 29.512), between the AMF, the SMF and the PCF;
//! - [`pdu_session`]: Nsmf_PDUSession (TS 29.502) between AMF and SMF, and
//!   N4 between SMF and UPF. N4 is PFCP (TS 29.244) in 3GPP; here it is EO
//!   messages too, by the owner's choice (small deployments, one process).

pub mod pdu_session;
pub mod policy;
pub mod ue_authentication;

use std::fmt;

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// Result of an operation: success, or the cause of its ProblemDetails.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Outcome {
    #[default]
    Ok = 0,
    /// USER_NOT_FOUND.
    UserNotFound = 1,
    /// SERVING_NETWORK_NOT_AUTHORIZED.
    ServingNetworkNotAuthorized = 2,
    /// AUTHENTICATION_REJECTED (also a resynchronisation that failed).
    AuthenticationRejected = 3,
    /// UNSUPPORTED_PROTECTION_SCHEME (SUCI).
    UnsupportedProtectionScheme = 4,
    /// The authentication context is unknown or expired (HTTP 404).
    ContextNotFound = 5,
    /// The next NF could not be reached (HTTP 504).
    Unavailable = 6,
    /// A container or IE could not be decoded (HTTP 400).
    InvalidRequest = 7,
}

impl Outcome {
    /// From a body's byte; unknown values read as `Unavailable`.
    pub const fn from_u8(v: u8) -> Self {
        match v {
            0 => Outcome::Ok,
            1 => Outcome::UserNotFound,
            2 => Outcome::ServingNetworkNotAuthorized,
            3 => Outcome::AuthenticationRejected,
            4 => Outcome::UnsupportedProtectionScheme,
            5 => Outcome::ContextNotFound,
            7 => Outcome::InvalidRequest,
            _ => Outcome::Unavailable,
        }
    }
}

/// Text of bounded length in a body.
macro_rules! text {
    ($(#[$doc:meta])* $name:ident, $cap:literal) => {
        $(#[$doc])*
        #[derive(Clone, Copy, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
        #[repr(C)]
        pub struct $name {
            len: u8,
            bytes: [u8; $cap],
        }

        impl $name {
            /// Longest text held.
            pub const CAPACITY: usize = $cap;

            /// `None` if `s` is longer than [`CAPACITY`](Self::CAPACITY).
            pub fn new(s: &str) -> Option<Self> {
                let mut t = $name { len: 0, bytes: [0; $cap] };
                t.bytes.get_mut(..s.len())?.copy_from_slice(s.as_bytes());
                t.len = s.len() as u8;
                Some(t)
            }

            /// Formats `args`; `None` if the text is too long.
            pub fn format(args: fmt::Arguments<'_>) -> Option<Self> {
                let mut t = $name { len: 0, bytes: [0; $cap] };
                let mut w = Fixed { buf: &mut t.bytes, len: 0 };
                fmt::write(&mut w, args).ok()?;
                t.len = w.len as u8;
                Some(t)
            }

            /// The text (its valid UTF-8 prefix: bodies come from other EOs).
            pub fn as_str(&self) -> &str {
                let bytes = &self.bytes[..usize::from(self.len).min($cap)];
                match std::str::from_utf8(bytes) {
                    Ok(s) => s,
                    Err(e) => std::str::from_utf8(&bytes[..e.valid_up_to()]).unwrap_or_default(),
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(self.as_str(), f)
            }
        }
    };
}

/// `fmt::Write` into a fixed buffer, failing when full.
struct Fixed<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl fmt::Write for Fixed<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let dst = self.buf.get_mut(self.len..self.len + s.len()).ok_or(fmt::Error)?;
        dst.copy_from_slice(s.as_bytes());
        self.len += s.len();
        Ok(())
    }
}

text!(
    /// A SUPI, `imsi-<digits>` or `nai-<nai>` (TS 29.571 §5.3.2).
    Supi,
    63
);
text!(
    /// A SUPI or a SUCI, `suci-0-<mcc>-<mnc>-...` (TS 29.503 §6.1.6.3.2).
    SupiOrSuci,
    159
);
text!(
    /// A data network name, dotted (`internet`).
    DnnText,
    63
);
text!(
    /// A serving network name, `5G:mnc<mnc>.mcc<mcc>.3gppnetwork.org` (TS 24.501 §9.12.1).
    ServingNetwork,
    63
);

impl Supi {
    /// The digits of an IMSI SUPI.
    pub fn imsi(&self) -> Option<&str> {
        self.as_str().strip_prefix("imsi-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texts_are_bounded_and_formatted_in_place() {
        let supi = Supi::new("imsi-999700000000001").unwrap();
        assert_eq!((supi.as_str(), supi.imsi()), ("imsi-999700000000001", Some("999700000000001")));
        assert!(Supi::new(&"1".repeat(64)).is_none());
        let snn = ServingNetwork::format(format_args!("5G:mnc{:03}.mcc{}.3gppnetwork.org", 70, 999)).unwrap();
        assert_eq!(snn.to_string(), "5G:mnc070.mcc999.3gppnetwork.org");
        assert!(ServingNetwork::format(format_args!("{}", "x".repeat(64))).is_none());
        assert_eq!(std::mem::size_of::<SupiOrSuci>(), 160);
    }
}
