//! Decoding what a UE sends never panics, whatever it sends.

use librecn_nas::mm::{Uplink, kind};
use librecn_nas::security::{self, NEA2, NIA2, SecurityContext};
use librecn_nas::sm::SmUplink;
use proptest::prelude::*;

/// Every uplink message type, so that random tails reach the IE decoders.
const KINDS: [u8; 12] = [
    kind::REGISTRATION_REQUEST,
    kind::REGISTRATION_COMPLETE,
    kind::DEREGISTRATION_REQUEST_UE,
    kind::SERVICE_REQUEST,
    kind::AUTHENTICATION_RESPONSE,
    kind::AUTHENTICATION_FAILURE,
    kind::IDENTITY_RESPONSE,
    kind::SECURITY_MODE_COMPLETE,
    kind::SECURITY_MODE_REJECT,
    kind::STATUS,
    kind::UL_NAS_TRANSPORT,
    0xff,
];

fn decode_all(pdu: &[u8]) {
    if let Ok(Uplink::RegistrationRequest(r)) = Uplink::decode(pdu) {
        let _ = (r.identity.to_string(), r.requested().count());
    }
    let _ = librecn_nas::describe(pdu).to_string();
    let _ = security::skip_header(pdu).map(Uplink::decode);
    if let Ok(Uplink::UlNasTransport(t)) = Uplink::decode(pdu) {
        let _ = (t.dnn.map(|d| d.to_string()), SmUplink::decode(t.container));
    }
    let _ = SmUplink::decode(pdu);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn random_bytes(pdu in proptest::collection::vec(any::<u8>(), 0..64)) {
        decode_all(&pdu);
        let mut pdu = pdu;
        let _ = SecurityContext::new(&[1; 32], 0, NEA2, NIA2).unprotect(&mut pdu);
    }

    #[test]
    fn plain_5gmm_with_random_ies(k in 0..KINDS.len(), ies in proptest::collection::vec(any::<u8>(), 0..48)) {
        decode_all(&[&[0x7e, 0x00, KINDS[k]][..], &ies].concat());
        // Mostly well-formed: a SUCI or 5G-GUTI identity of random content.
        let id = [&[0x7e, 0x00, 0x41, 0x01, 0x00, ies.len().min(20) as u8][..], &ies].concat();
        decode_all(&id);
        // 5GSM, and UL NAS Transport around it.
        decode_all(&[&[0x2e, 0x01, 0x01, 0xc1][..], &ies].concat());
        let ul = [&[0x7e, 0x00, 0x67, 0x01, 0x00, ies.len().min(30) as u8][..], &ies].concat();
        decode_all(&ul);
    }
}
