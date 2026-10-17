//! The NAS layer, as N2 sees it: an uplink NAS PDU in, an answer out.
//!
//! This release has no NAS codec yet (TS 24.501 comes next): every uplink
//! PDU is answered by a plain Registration Reject, cause #111 "protocol
//! error, unspecified", and the UE is released. Replacing [`on_uplink`] is
//! the whole of the NAS work; N2 does not change.

/// What N2 must do after an uplink NAS PDU.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// Send `nas` down to the UE, then release its context.
    SendAndRelease { nas: Vec<u8> },
}

/// 5GS mobility management (TS 24.007 §11.2.3.1.1).
const EPD_5GMM: u8 = 0x7e;
/// 5GS session management.
const EPD_5GSM: u8 = 0x2e;
const PLAIN: u8 = 0x00;
const REGISTRATION_REJECT: u8 = 0x44;
/// 5GMM cause #111 (TS 24.501 §9.11.3.2).
const PROTOCOL_ERROR_UNSPECIFIED: u8 = 111;

pub fn on_uplink(_pdu: &[u8]) -> Answer {
    // TS 24.501 §8.2.9: plain 5GMM header, message type, 5GMM cause.
    Answer::SendAndRelease { nas: vec![EPD_5GMM, PLAIN, REGISTRATION_REJECT, PROTOCOL_ERROR_UNSPECIFIED] }
}

/// A short description for logs: protocol, security, message type.
pub fn describe(pdu: &[u8]) -> String {
    let name = |epd: u8, kind: u8| match (epd, kind) {
        (EPD_5GMM, 0x41) => "Registration Request",
        (EPD_5GMM, 0x43) => "Registration Complete",
        (EPD_5GMM, 0x44) => "Registration Reject",
        (EPD_5GMM, 0x45) => "Deregistration Request",
        (EPD_5GMM, 0x4c) => "Service Request",
        (EPD_5GMM, 0x56) => "Authentication Request",
        (EPD_5GMM, 0x57) => "Authentication Response",
        (EPD_5GMM, 0x5d) => "Security Mode Command",
        (EPD_5GMM, 0x5e) => "Security Mode Complete",
        (EPD_5GMM, 0x67) => "UL NAS Transport",
        (EPD_5GMM, 0x68) => "DL NAS Transport",
        _ => "message",
    };
    match pdu {
        [EPD_5GMM, PLAIN, kind, ..] => format!("5GMM {} (plain)", name(EPD_5GMM, *kind)),
        // Security protected: the inner message is ciphered or follows 6 octets later.
        [EPD_5GMM, security, ..] => format!("5GMM security protected (header type {})", security & 0x0f),
        [EPD_5GSM, ..] => "5GSM message".into(),
        _ => format!("unknown NAS ({} bytes)", pdu.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_uplink_gets_a_plain_registration_reject() {
        assert_eq!(on_uplink(&[0x7e, 0x00, 0x41]), Answer::SendAndRelease { nas: vec![0x7e, 0x00, 0x44, 0x6f] });
        assert_eq!(describe(&[0x7e, 0x00, 0x44, 0x6f]), "5GMM Registration Reject (plain)");
        assert_eq!(describe(&[0x7e, 0x02, 0x11]), "5GMM security protected (header type 2)");
        assert_eq!(describe(&[]), "unknown NAS (0 bytes)");
    }
}
