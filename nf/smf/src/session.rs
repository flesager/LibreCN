//! PDU session establishment, on domain values: the SMF's decision on a
//! PDU Session Establishment Request (TS 23.502 §4.3.2.2.1), and the N1
//! answers.

use std::net::Ipv4Addr;

use librecn_nas::Snssai;
use librecn_nas::sm::{FlowRates, PacketFilter, QosFlow, SmDownlink, SmUplink, cause, session_type};
use librecn_sbi::pdu_session::CONTAINER;
use librecn_sbi::policy::{DEFAULT_QFI, MAX_FILTERS, MAX_RULES, SessionRule, rule_qfi};

use crate::config::SmfConfig;

/// Largest N1 answer: what a container carries.
pub const N1_MAX: usize = CONTAINER;

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

/// PDU Session Establishment Accept: IPv4 `address` on DNN `dnn` (an index
/// in the configuration's), SSC mode 1, the PCF's `rule`: the session AMBR,
/// the default flow, and a flow per PCC rule. None if it does not fit in
/// [`N1_MAX`] (the most filters with every field).
pub fn accept(
    config: &SmfConfig,
    pdu_session_id: u8,
    pti: u8,
    slice: Snssai,
    dnn: usize,
    address: Ipv4Addr,
    rule: &SessionRule,
) -> Option<Vec<u8>> {
    let mut filters = [[PacketFilter::default(); MAX_FILTERS]; MAX_RULES];
    for (mine, pcc) in filters.iter_mut().zip(rule.rules()) {
        for (f, p) in mine.iter_mut().zip(pcc.filters()) {
            *f = PacketFilter {
                direction: p.direction,
                protocol: p.protocol,
                remote: (p.remote, p.remote_prefix),
                remote_ports: p.remote_ports,
                local_ports: p.local_ports,
            };
        }
    }
    let default = QosFlow { qfi: DEFAULT_QFI, five_qi: rule.five_qi, gbr: None, precedence: 0, filters: &[] };
    let mut flows = [default; 1 + MAX_RULES];
    for (i, pcc) in rule.rules().iter().enumerate() {
        flows[1 + i] = QosFlow {
            qfi: rule_qfi(i),
            five_qi: pcc.five_qi,
            gbr: (!pcc.gbr.is_zero()).then_some(FlowRates {
                gfbr: (pcc.gbr.uplink, pcc.gbr.downlink),
                mfbr: (pcc.mbr.uplink, pcc.mbr.downlink),
            }),
            precedence: pcc.precedence,
            filters: &filters[i][..pcc.filters().len()],
        };
    }
    let mut out = [0u8; N1_MAX];
    let accept = SmDownlink::EstablishmentAccept {
        address: address.octets(),
        ssc_mode: 1,
        flows: &flows[..1 + rule.rules().len()],
        ambr: (rule.ambr.uplink, rule.ambr.downlink),
        snssai: slice,
        dnn: &config.dnns[dnn].dnn,
    };
    let n = accept.encode(pdu_session_id, pti, &mut out).ok()?;
    Some(out[..n].to_vec())
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

    use librecn_sbi::policy::{self, PccRule};
    use zerocopy::FromZeros;

    const SST1: Snssai = Snssai { sst: 1, sd: None };

    /// The PCF's rule: 5QI 9, 100 Mbit/s up, 1000 down.
    fn rule() -> SessionRule {
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
        let accept = |dnn, rule: &SessionRule| accept(&c, 1, 7, SST1, dnn, Ipv4Addr::new(10, 45, 0, 2), rule);
        assert_eq!(accept(0, &rule()).unwrap()[..5], hex("2e0107c211")[..]);
        let ims = accept(1, &rule()).unwrap();
        assert!(ims.ends_with(&hex("25 04 03696d73")), "DNN ims in the accept");

        // A PCC rule: QoS rule 2 to flow 2, its filter, its description.
        let mut with_flow = rule();
        let mut pcc = PccRule::new_zeroed();
        (pcc.five_qi, pcc.arp, pcc.precedence) = (7, 8, 10);
        pcc.set_filters(&[policy::PacketFilter::new(1, 1, ([10, 45, 0, 1], 32), [0, 0], [0, 0])]);
        with_flow.set_rules(&[pcc]);
        let n1 = accept(0, &with_flow).unwrap();
        let rule2 = hex("02 0010 21 11 0b 10 0a2d0001 ffffffff 3001 0a 02");
        assert!(n1.windows(rule2.len()).any(|w| w == rule2), "the rule: ICMP from 10.45.0.1, downlink");
        assert!(n1.windows(6).any(|w| w == hex("022041010107")), "flow 2, 5QI 7");

        // Every filter with every field, on GBR flows: too big for a container.
        let mut big = PccRule::new_zeroed();
        (big.five_qi, big.precedence) = (1, 1);
        (big.gbr, big.mbr) = (rule().ambr, rule().ambr);
        big.set_filters(&[policy::PacketFilter::new(3, 6, ([10, 0, 0, 0], 8), [1, 2], [3, 4]); MAX_FILTERS]);
        let mut huge = rule();
        huge.set_rules(&[big; MAX_RULES]);
        assert_eq!(accept(0, &huge), None);
    }
}
