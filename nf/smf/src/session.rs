//! PDU session establishment, on domain values: the SMF's decision on a
//! PDU Session Establishment Request (TS 23.502 §4.3.2.2.1), and the N1
//! answers.

use std::net::Ipv4Addr;

use librecn_nas::Snssai;
use librecn_nas::sm::{QosFlow, SmDownlink, SmUplink, cause, session_type};

use librecn_sbi::policy::SessionRule;

use crate::config::SmfConfig;

/// The default QoS flow's identifier.
pub const DEFAULT_QFI: u8 = 1;
/// Largest N1 answer.
pub const N1_MAX: usize = 128;

/// An accepted request: what the answer needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Accepted {
    pub pti: u8,
    /// The DNN chosen, an index in the configuration's.
    pub dnn: usize,
}

/// A refused request: its PTI and 5GSM cause.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Refused {
    pub pti: u8,
    pub cause: u8,
}

/// Checks the UE's request (`n1`) for PDU session `pdu_session_id` on
/// `slice`, to `dnn` (empty: the UE asked none, the first one served).
pub fn decide(
    config: &SmfConfig,
    pdu_session_id: u8,
    slice: Snssai,
    dnn: &str,
    n1: &[u8],
) -> Result<Accepted, Refused> {
    let (header, message) =
        SmUplink::decode(n1).map_err(|_| Refused { pti: 0, cause: cause::INVALID_MANDATORY_INFORMATION })?;
    let refuse = |cause| Err(Refused { pti: header.pti, cause });
    let SmUplink::EstablishmentRequest { session_type, ssc_mode } = message else {
        return refuse(cause::MESSAGE_TYPE_NOT_IMPLEMENTED);
    };
    if header.pdu_session_id != pdu_session_id || !(1..=15).contains(&pdu_session_id) {
        return refuse(cause::INVALID_PDU_SESSION_IDENTITY);
    }
    let dnn = if dnn.is_empty() { Some(0) } else { config.dnns.iter().position(|d| dnn.eq_ignore_ascii_case(&d.dnn)) };
    let Some(dnn) = dnn else { return refuse(cause::MISSING_OR_UNKNOWN_DNN) };
    if !config.slices.contains(&slice) {
        return refuse(cause::REQUEST_REJECTED);
    }
    match session_type {
        None | Some(session_type::IPV4 | session_type::IPV4V6) => {}
        Some(session_type::IPV6) => return refuse(cause::IPV4_ONLY_ALLOWED),
        Some(_) => return refuse(cause::UNKNOWN_PDU_SESSION_TYPE),
    }
    if ssc_mode.is_some_and(|m| m != 1) {
        return refuse(cause::SERVICE_OPTION_NOT_SUPPORTED);
    }
    Ok(Accepted { pti: header.pti, dnn })
}

/// A bit rate in Mbit/s as NAS carries it (16 bits, 1 at least).
fn mbps(bps: u64) -> u16 {
    (bps / 1_000_000).clamp(1, u64::from(u16::MAX)) as u16
}

/// PDU Session Establishment Accept: IPv4 `address` on DNN `dnn` (an index
/// in the configuration's), SSC mode 1, the default flow and the session
/// AMBR of the PCF's `rule`.
pub fn accept(
    config: &SmfConfig,
    pdu_session_id: u8,
    pti: u8,
    slice: Snssai,
    dnn: usize,
    address: Ipv4Addr,
    rule: &SessionRule,
) -> Vec<u8> {
    let mut out = [0u8; N1_MAX];
    let accept = SmDownlink::EstablishmentAccept {
        address: address.octets(),
        ssc_mode: 1,
        flow: QosFlow { qfi: DEFAULT_QFI, five_qi: rule.five_qi },
        ambr_mbps: (mbps(rule.ambr.uplink), mbps(rule.ambr.downlink)),
        snssai: slice,
        dnn: &config.dnns[dnn].dnn,
    };
    let n = accept.encode(pdu_session_id, pti, &mut out).expect("an accept fits in 128 bytes");
    out[..n].to_vec()
}

/// PDU Session Release Command, cause #36 (regular deactivation).
pub fn release_command(pdu_session_id: u8, pti: u8) -> Vec<u8> {
    encode(SmDownlink::ReleaseCommand { cause: cause::REGULAR_DEACTIVATION }, pdu_session_id, pti)
}

/// PDU Session Modification Reject: no QoS change is offered (#31).
pub fn modification_reject(pdu_session_id: u8, pti: u8) -> Vec<u8> {
    encode(SmDownlink::ModificationReject { cause: cause::REQUEST_REJECTED }, pdu_session_id, pti)
}

/// 5GSM Status.
pub fn status(pdu_session_id: u8, pti: u8, cause: u8) -> Vec<u8> {
    encode(SmDownlink::Status { cause }, pdu_session_id, pti)
}

fn encode(message: SmDownlink<'_>, pdu_session_id: u8, pti: u8) -> Vec<u8> {
    let mut out = [0u8; 8];
    let n = message.encode(pdu_session_id, pti, &mut out).expect("5 bytes");
    out[..n].to_vec()
}

/// PDU Session Establishment Reject.
pub fn reject(pdu_session_id: u8, refused: Refused) -> Vec<u8> {
    encode(SmDownlink::EstablishmentReject { cause: refused.cause }, pdu_session_id, refused.pti)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(s: &str) -> Vec<u8> {
        let s: String = s.split_whitespace().collect();
        (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
    }

    const SST1: Snssai = Snssai { sst: 1, sd: None };

    /// The PCF's rule: 5QI 9, 100 Mbit/s up, 1000 down.
    fn rule() -> SessionRule {
        use zerocopy::FromZeros;
        let mut r = SessionRule::new_zeroed();
        (r.five_qi, r.arp) = (9, 8);
        r.ambr = librecn_sbi::policy::Rates { uplink: 100_000_000, downlink: 1_000_000_000 };
        r
    }

    #[test]
    fn requests_are_checked_against_what_the_smf_serves() {
        let params = serde_json::json!({ "dnns": [{ "dnn": "internet", "pool": "10.45.0.0/16" },
                                                  { "dnn": "ims", "pool": "10.46.0.0/16" }] });
        let c = SmfConfig::from_params(&params).unwrap();
        let request = hex("2e 01 07 c1 ffff 91 a1");
        assert_eq!(decide(&c, 1, SST1, "internet", &request), Ok(Accepted { pti: 7, dnn: 0 }));
        assert_eq!(decide(&c, 1, SST1, "IMS", &request), Ok(Accepted { pti: 7, dnn: 1 }), "DNNs ignore case");
        assert_eq!(decide(&c, 1, SST1, "", &request), Ok(Accepted { pti: 7, dnn: 0 }), "the first DNN by default");
        let refused = |cause| Err(Refused { pti: 7, cause });
        assert_eq!(decide(&c, 1, SST1, "mms", &request), refused(cause::MISSING_OR_UNKNOWN_DNN));
        assert_eq!(decide(&c, 2, SST1, "internet", &request), refused(cause::INVALID_PDU_SESSION_IDENTITY));
        assert_eq!(decide(&c, 1, Snssai { sst: 2, sd: None }, "", &request), refused(cause::REQUEST_REJECTED));
        assert_eq!(decide(&c, 1, SST1, "", &hex("2e 01 07 c1 ffff 92")), refused(cause::IPV4_ONLY_ALLOWED));
        assert_eq!(decide(&c, 1, SST1, "", &hex("2e 01 07 c1 ffff 95")), refused(cause::UNKNOWN_PDU_SESSION_TYPE));
        assert_eq!(decide(&c, 1, SST1, "", &hex("2e 01 07 c1 ffff a2")), refused(cause::SERVICE_OPTION_NOT_SUPPORTED));
        assert_eq!(decide(&c, 1, SST1, "", &hex("2e 01 07 d1")), refused(cause::MESSAGE_TYPE_NOT_IMPLEMENTED));
        assert_eq!(decide(&c, 1, SST1, "", &hex("2e 01")).unwrap_err().cause, cause::INVALID_MANDATORY_INFORMATION);
        assert_eq!(reject(1, Refused { pti: 7, cause: 27 }), hex("2e0107c31b"));
        assert_eq!(release_command(1, 0), hex("2e0100d324"));
        assert_eq!(modification_reject(1, 3), hex("2e0103ca1f"));
        assert_eq!(accept(&c, 1, 7, SST1, 0, Ipv4Addr::new(10, 45, 0, 2), &rule())[..5], hex("2e0107c211")[..]);
        let ims = accept(&c, 1, 7, SST1, 1, Ipv4Addr::new(10, 46, 0, 2), &rule());
        assert!(ims.ends_with(&hex("25 04 03696d73")), "DNN ims in the accept");
    }
}
