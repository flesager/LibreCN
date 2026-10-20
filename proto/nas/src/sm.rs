//! 5GSM messages (TS 24.501 §8.3) of PDU session establishment and release,
//! for the SMF.
//!
//! A 5GSM message is the extended protocol discriminator, the PDU session
//! ID, the procedure transaction identity (PTI), the message type, then the
//! IEs. It travels inside UL / DL NAS Transport.

use crate::ie::Dnn;
use crate::{EPD_5GSM, Error, Reader, Result, Snssai, Writer};

/// Message types (§9.7).
pub mod kind {
    pub const ESTABLISHMENT_REQUEST: u8 = 0xc1;
    pub const ESTABLISHMENT_ACCEPT: u8 = 0xc2;
    pub const ESTABLISHMENT_REJECT: u8 = 0xc3;
    pub const MODIFICATION_REQUEST: u8 = 0xc9;
    pub const MODIFICATION_REJECT: u8 = 0xca;
    pub const RELEASE_REQUEST: u8 = 0xd1;
    pub const RELEASE_COMMAND: u8 = 0xd3;
    pub const RELEASE_COMPLETE: u8 = 0xd4;
    pub const STATUS: u8 = 0xd6;
}

/// The name of 5GSM message type `kind`.
pub fn name(kind: u8) -> &'static str {
    match kind {
        kind::ESTABLISHMENT_REQUEST => "PDU Session Establishment Request",
        kind::ESTABLISHMENT_ACCEPT => "PDU Session Establishment Accept",
        kind::ESTABLISHMENT_REJECT => "PDU Session Establishment Reject",
        kind::MODIFICATION_REQUEST => "PDU Session Modification Request",
        kind::MODIFICATION_REJECT => "PDU Session Modification Reject",
        kind::RELEASE_REQUEST => "PDU Session Release Request",
        kind::RELEASE_COMMAND => "PDU Session Release Command",
        kind::RELEASE_COMPLETE => "PDU Session Release Complete",
        kind::STATUS => "5GSM Status",
        _ => "5GSM message",
    }
}

/// 5GSM causes (§9.11.4.2) the SMF sends.
pub mod cause {
    pub const INSUFFICIENT_RESOURCES: u8 = 26;
    pub const MISSING_OR_UNKNOWN_DNN: u8 = 27;
    pub const UNKNOWN_PDU_SESSION_TYPE: u8 = 28;
    pub const REQUEST_REJECTED: u8 = 31;
    pub const REGULAR_DEACTIVATION: u8 = 36;
    pub const SERVICE_OPTION_NOT_SUPPORTED: u8 = 32;
    pub const IPV4_ONLY_ALLOWED: u8 = 50;
    pub const INVALID_PDU_SESSION_IDENTITY: u8 = 43;
    pub const INVALID_MANDATORY_INFORMATION: u8 = 96;
    pub const MESSAGE_TYPE_NOT_IMPLEMENTED: u8 = 97;
}

/// PDU session types (§9.11.4.11).
pub mod session_type {
    pub const IPV4: u8 = 1;
    pub const IPV6: u8 = 2;
    pub const IPV4V6: u8 = 3;
}

/// The 5GSM header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub pdu_session_id: u8,
    pub pti: u8,
    pub kind: u8,
}

/// What a UE asks for in its extended protocol configuration options
/// (ePCO, TS 24.008 §10.5.6.3: containers of the request, without content).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PcoRequests {
    /// DNS server IPv4 addresses (container 000DH).
    pub dns: bool,
    /// IPv4 link MTU (container 0010H).
    pub mtu: bool,
}

/// The containers an ePCO's `value` asks for; a malformed tail is ignored.
fn pco_requests(value: &[u8]) -> PcoRequests {
    let mut asked = PcoRequests::default();
    // The configuration protocol octet, then containers: id (2), length (1), content.
    let mut rest = value.get(1..).unwrap_or_default();
    while let [id0, id1, len, tail @ ..] = rest {
        match u16::from_be_bytes([*id0, *id1]) {
            0x000d => asked.dns = true,
            0x0010 => asked.mtu = true,
            _ => {}
        }
        rest = tail.get(usize::from(*len)..).unwrap_or_default();
    }
    asked
}

/// What the network answers to a UE's ePCO requests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pco<'a> {
    pub dns: &'a [[u8; 4]],
    pub mtu: Option<u16>,
}

/// The uplink 5GSM messages the SMF handles, decoded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmUplink {
    EstablishmentRequest {
        /// Requested PDU session type, if any.
        session_type: Option<u8>,
        ssc_mode: Option<u8>,
        pco: PcoRequests,
    },
    ReleaseRequest,
    ReleaseComplete,
    ModificationRequest,
    Status {
        cause: u8,
    },
    /// Not a message the SMF handles.
    Other(u8),
}

impl SmUplink {
    /// Decodes a 5GSM message.
    pub fn decode(pdu: &[u8]) -> Result<(Header, Self)> {
        let mut r = Reader::new(pdu);
        if r.u8()? != EPD_5GSM {
            return Err(Error::Invalid("5GSM header"));
        }
        let header = Header { pdu_session_id: r.u8()?, pti: r.u8()?, kind: r.u8()? };
        let message = match header.kind {
            kind::ESTABLISHMENT_REQUEST => {
                r.take(2)?; // integrity protection maximum data rate
                let (mut session_type, mut ssc_mode, mut pco) = (None, None, PcoRequests::default());
                // Maximum number of supported packet filters (0x55) is TV.
                while let Some((iei, v)) = r.optional(&[(0x55, 2)])? {
                    match iei {
                        0x90 => session_type = Some(v[0] & 0x07),
                        0xa0 => ssc_mode = Some(v[0] & 0x07),
                        0x7b => pco = pco_requests(v),
                        _ => {}
                    }
                }
                SmUplink::EstablishmentRequest { session_type, ssc_mode, pco }
            }
            kind::RELEASE_REQUEST => SmUplink::ReleaseRequest,
            kind::RELEASE_COMPLETE => SmUplink::ReleaseComplete,
            kind::MODIFICATION_REQUEST => SmUplink::ModificationRequest,
            kind::STATUS => SmUplink::Status { cause: r.u8()? },
            other => SmUplink::Other(other),
        };
        Ok((header, message))
    }
}

/// A packet filter of a QoS rule (§9.11.4.13), seen from the UE ("remote"
/// is the other end). Zeros match anything.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PacketFilter {
    /// 1 downlink, 2 uplink, 3 both.
    pub direction: u8,
    /// IP protocol; 0: any.
    pub protocol: u8,
    /// Remote IPv4 address and prefix length; length 0: any.
    pub remote: ([u8; 4], u8),
    /// First and last port; 0, 0: any.
    pub remote_ports: [u16; 2],
    pub local_ports: [u16; 2],
}

/// Guaranteed and maximum flow bit rates of a GBR flow, in bit/s: uplink, downlink.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlowRates {
    pub gfbr: (u64, u64),
    pub mfbr: (u64, u64),
}

/// A QoS flow of an accepted session, with its QoS rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QosFlow<'a> {
    pub qfi: u8,
    pub five_qi: u8,
    /// GBR flows only.
    pub gbr: Option<FlowRates>,
    /// The rule's precedence (lowest first) and packet filters.
    pub precedence: u8,
    pub filters: &'a [PacketFilter],
}

/// The downlink 5GSM messages the SMF sends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmDownlink<'a> {
    EstablishmentAccept {
        /// IPv4 address of the UE.
        address: [u8; 4],
        ssc_mode: u8,
        /// The first is the default flow: its rule is the default QoS
        /// rule (match-all, precedence 255), its filters and precedence
        /// unused.
        flows: &'a [QosFlow<'a>],
        /// Session AMBR in bit/s: uplink, downlink.
        ambr: (u64, u64),
        snssai: Snssai,
        dnn: &'a str,
        /// The answers to the UE's ePCO requests (none: no ePCO).
        pco: Pco<'a>,
    },
    EstablishmentReject {
        cause: u8,
    },
    /// PTI 0 when the network releases on its own.
    ReleaseCommand {
        cause: u8,
    },
    ModificationReject {
        cause: u8,
    },
    Status {
        cause: u8,
    },
}

/// A bit rate as session AMBR (§9.11.4.14) and flow bit rates (§9.11.4.12)
/// carry it: a unit, then 16 bits. In Mbit/s from 1 Mbit/s on, else kbit/s.
fn bit_rate(bps: u64) -> [u8; 3] {
    const KBPS: u8 = 0x01;
    const MBPS: u8 = 0x06;
    let (unit, value) = if bps >= 1_000_000 { (MBPS, bps / 1_000_000) } else { (KBPS, (bps / 1000).max(1)) };
    let [hi, lo] = (value.min(u64::from(u16::MAX)) as u16).to_be_bytes();
    [unit, hi, lo]
}

/// A QoS rule's packet filter `id` (§9.11.4.13): its components.
fn encode_filter(w: &mut Writer<'_>, id: u8, f: &PacketFilter) {
    w.u8((f.direction & 0x03) << 4 | id & 0x0f);
    w.lv(|w| {
        let any = f.protocol == 0 && f.remote.1 == 0 && f.remote_ports == [0, 0] && f.local_ports == [0, 0];
        if any {
            return w.u8(0x01); // match-all
        }
        if f.remote.1 > 0 {
            let mask = u32::MAX.checked_shl(32 - u32::from(f.remote.1.min(32))).unwrap_or(0);
            w.u8(0x10); // IPv4 remote address
            w.bytes(&f.remote.0);
            w.bytes(&mask.to_be_bytes());
        }
        if f.protocol != 0 {
            w.bytes(&[0x30, f.protocol]);
        }
        // Single port (0x40, 0x50) or range (0x41, 0x51).
        for (ports, single) in [(f.local_ports, 0x40), (f.remote_ports, 0x50)] {
            match ports {
                [0, 0] => {}
                [first, last] if first == last => {
                    w.u8(single);
                    w.bytes(&first.to_be_bytes());
                }
                [first, last] => {
                    w.u8(single + 1);
                    w.bytes(&first.to_be_bytes());
                    w.bytes(&last.to_be_bytes());
                }
            }
        }
    });
}

impl SmDownlink<'_> {
    pub fn kind(&self) -> u8 {
        match self {
            SmDownlink::EstablishmentAccept { .. } => kind::ESTABLISHMENT_ACCEPT,
            SmDownlink::EstablishmentReject { .. } => kind::ESTABLISHMENT_REJECT,
            SmDownlink::ReleaseCommand { .. } => kind::RELEASE_COMMAND,
            SmDownlink::ModificationReject { .. } => kind::MODIFICATION_REJECT,
            SmDownlink::Status { .. } => kind::STATUS,
        }
    }

    /// Encodes the message for PDU session `pdu_session_id`, answering
    /// procedure `pti`, into `out`; returns its length.
    pub fn encode(&self, pdu_session_id: u8, pti: u8, out: &mut [u8]) -> Result<usize> {
        let mut w = Writer::new(out);
        w.bytes(&[EPD_5GSM, pdu_session_id, pti, self.kind()]);
        match *self {
            SmDownlink::EstablishmentAccept { address, ssc_mode, flows, ambr, snssai, dnn, pco } => {
                w.u8(ssc_mode << 4 | crate::sm::session_type::IPV4);
                // Authorized QoS rules (§9.11.4.13), one per flow, rule
                // identifiers from 1: the default rule (DQR) with one
                // match-all filter, then the flows' own.
                w.lve(|w| {
                    for (i, flow) in flows.iter().enumerate() {
                        w.u8(i as u8 + 1);
                        w.lve(|w| {
                            const CREATE: u8 = 0x20;
                            const DQR: u8 = 0x10;
                            if i == 0 {
                                w.u8(CREATE | DQR | 1);
                                encode_filter(w, 1, &PacketFilter { direction: 3, ..PacketFilter::default() });
                                w.u8(0xff);
                            } else {
                                w.u8(CREATE | flow.filters.len() as u8 & 0x0f);
                                for (id, f) in flow.filters.iter().enumerate() {
                                    encode_filter(w, id as u8 + 1, f);
                                }
                                w.u8(flow.precedence);
                            }
                            w.u8(flow.qfi & 0x3f);
                        });
                    }
                });
                w.lv(|w| {
                    w.bytes(&bit_rate(ambr.1));
                    w.bytes(&bit_rate(ambr.0));
                });
                w.tlv(0x29, |w| {
                    w.u8(crate::sm::session_type::IPV4);
                    w.bytes(&address);
                });
                w.u8(0x22);
                snssai.encode(&mut w);
                // Authorized QoS flow descriptions (§9.11.4.12): each
                // flow's 5QI, and a GBR flow's GFBR and MFBR.
                w.tlve(0x79, |w| {
                    for flow in flows {
                        let parameters = if flow.gbr.is_some() { 5 } else { 1 };
                        // Create new, E bit (parameters list included).
                        w.bytes(&[flow.qfi & 0x3f, 0x20, 0x40 | parameters, 0x01, 0x01, flow.five_qi]);
                        if let Some(FlowRates { gfbr, mfbr }) = flow.gbr {
                            for (id, bps) in [(0x02, gfbr.0), (0x03, gfbr.1), (0x04, mfbr.0), (0x05, mfbr.1)] {
                                w.bytes(&[id, 3]);
                                w.bytes(&bit_rate(bps));
                            }
                        }
                    }
                });
                // ePCO: the configuration protocol octet (PPP, ext bit),
                // then one container per DNS server, the MTU's.
                if !pco.dns.is_empty() || pco.mtu.is_some() {
                    w.tlve(0x7b, |w| {
                        w.u8(0x80);
                        for dns in pco.dns {
                            w.bytes(&[0x00, 0x0d, 4]);
                            w.bytes(dns);
                        }
                        if let Some(mtu) = pco.mtu {
                            w.bytes(&[0x00, 0x10, 2]);
                            w.bytes(&mtu.to_be_bytes());
                        }
                    });
                }
                w.tlv(0x25, |w| Dnn::encode(dnn, w));
            }
            SmDownlink::EstablishmentReject { cause }
            | SmDownlink::ReleaseCommand { cause }
            | SmDownlink::ModificationReject { cause }
            | SmDownlink::Status { cause } => w.u8(cause),
        }
        w.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex;

    /// UERANSIM's request: IPv4, SSC mode 1, 5GSM capability and ePCO.
    #[test]
    fn an_establishment_request_is_decoded() {
        let pdu = hex("2e 01 01 c1 ffff 91 a1 28 01 00 7b 0007 80000a00000d00");
        let (header, message) = SmUplink::decode(&pdu).unwrap();
        assert_eq!(header, Header { pdu_session_id: 1, pti: 1, kind: kind::ESTABLISHMENT_REQUEST });
        let pco = PcoRequests { dns: true, mtu: false };
        assert_eq!(message, SmUplink::EstablishmentRequest { session_type: Some(1), ssc_mode: Some(1), pco });
        // UERANSIM's release request carries a 5GSM cause (#36): ignored.
        assert_eq!(SmUplink::decode(&hex("2e0102d1 5924")).unwrap().1, SmUplink::ReleaseRequest);
        assert_eq!(SmUplink::decode(&hex("2e0100d4")).unwrap().1, SmUplink::ReleaseComplete);
        assert_eq!(SmUplink::decode(&hex("2e0103c9")).unwrap().1, SmUplink::ModificationRequest);
        assert_eq!(SmUplink::decode(&hex("2e0101c5")).unwrap().1, SmUplink::Other(0xc5));
        assert_eq!(SmUplink::decode(&hex("2e01")), Err(Error::Truncated));
        assert!(SmUplink::decode(&hex("7e0001c1")).is_err());
    }

    /// The accept, byte for byte (TS 24.501 §8.3.2).
    #[test]
    fn an_establishment_accept_is_encoded() {
        let accept = SmDownlink::EstablishmentAccept {
            address: [10, 45, 0, 2],
            ssc_mode: 1,
            flows: &[QosFlow { qfi: 1, five_qi: 9, gbr: None, precedence: 0, filters: &[] }],
            ambr: (100_000_000, 1_000_000_000),
            snssai: Snssai { sst: 1, sd: None },
            dnn: "internet",
            pco: Pco::default(),
        };
        let mut buf = [0u8; 128];
        let n = accept.encode(1, 1, &mut buf).unwrap();
        let expected = hex("2e 01 01 c2 11 0009 01 0006 31 310101 ff 01 06 0603e8060064 29 05 01 0a2d0002 22 01 01 \
             79 0006 012041010109 25 09 08696e7465726e6574");
        assert_eq!(buf[..n], expected[..]);
        let reject = SmDownlink::EstablishmentReject { cause: cause::MISSING_OR_UNKNOWN_DNN };
        let n = reject.encode(5, 3, &mut buf).unwrap();
        assert_eq!(buf[..n], hex("2e0503c31b")[..]);
        let n = SmDownlink::ReleaseCommand { cause: cause::REGULAR_DEACTIVATION }.encode(1, 0, &mut buf).unwrap();
        assert_eq!(buf[..n], hex("2e0100d324")[..], "network-requested: PTI 0");
    }

    /// A UE's ePCO requests (a Quectel's: P-CSCF, DNS, MTU, IP address
    /// allocation), and the network's answer before the DNN.
    #[test]
    fn dns_servers_and_the_mtu_answer_the_ues_requests() {
        assert_eq!(pco_requests(&hex("80 000c00 000d00 001000 000a00")), PcoRequests { dns: true, mtu: true });
        assert_eq!(pco_requests(&hex("80 000a00")), PcoRequests::default());
        assert_eq!(pco_requests(&hex("80 000d")), PcoRequests::default(), "truncated: ignored");
        assert_eq!(pco_requests(&[]), PcoRequests::default());
        let accept = SmDownlink::EstablishmentAccept {
            address: [10, 45, 0, 2],
            ssc_mode: 1,
            flows: &[QosFlow { qfi: 1, five_qi: 9, gbr: None, precedence: 0, filters: &[] }],
            ambr: (100_000_000, 1_000_000_000),
            snssai: Snssai { sst: 1, sd: None },
            dnn: "internet",
            pco: Pco { dns: &[[8, 8, 8, 8], [1, 1, 1, 1]], mtu: Some(1400) },
        };
        let mut buf = [0u8; 128];
        let n = accept.encode(1, 1, &mut buf).unwrap();
        let tail =
            hex("79 0006 012041010109 7b 0014 80 000d04 08080808 000d04 01010101 001002 0578 25 09 08696e7465726e6574");
        assert_eq!(buf[n - tail.len()..n], tail[..]);
    }

    /// An extra GBR flow: its rule's filter components (§9.11.4.13) and its
    /// GFBR and MFBR (§9.11.4.12).
    #[test]
    fn an_extra_flow_gets_its_rule_and_description() {
        let sip = PacketFilter {
            direction: 3,
            protocol: 17,
            remote: ([10, 10, 0, 0], 16),
            remote_ports: [5060, 5061],
            ..PacketFilter::default()
        };
        let voice = QosFlow {
            qfi: 2,
            five_qi: 1,
            gbr: Some(FlowRates { gfbr: (1_000_000, 1_000_000), mfbr: (2_000_000, 64_000) }),
            precedence: 10,
            filters: &[sip],
        };
        let flows = [QosFlow { qfi: 1, five_qi: 9, gbr: None, precedence: 0, filters: &[] }, voice];
        let accept = SmDownlink::EstablishmentAccept {
            address: [10, 45, 0, 2],
            ssc_mode: 1,
            flows: &flows,
            ambr: (20_000_000, 100_000_000),
            snssai: Snssai { sst: 1, sd: None },
            dnn: "internet",
            pco: Pco::default(),
        };
        let mut buf = [0u8; 256];
        let n = accept.encode(1, 1, &mut buf).unwrap();
        let expected = hex("2e 01 01 c2 11 0021 \
             01 0006 31 310101 ff 01 \
             02 0015 21 31 10 100a0a0000ffff0000 3011 5113c413c5 0a 02 \
             06 060064060014 29 05 01 0a2d0002 22 01 01 \
             79 0020 012041010109 \
             022045010101 0203060001 0303060001 0403060002 0503010040 \
             25 09 08696e7465726e6574");
        assert_eq!(buf[..n], expected[..]);

        // Single ports, a whole address, and a filter matching anything.
        let mut out = [0u8; 32];
        let mut w = Writer::new(&mut out);
        let one =
            PacketFilter { direction: 1, remote: ([8, 8, 8, 8], 32), local_ports: [80, 80], ..PacketFilter::default() };
        encode_filter(&mut w, 2, &one);
        encode_filter(&mut w, 3, &PacketFilter { direction: 2, ..PacketFilter::default() });
        let n = w.finish().unwrap();
        assert_eq!(out[..n], hex("12 0c 10 08080808 ffffffff 400050  23 01 01")[..]);
    }
}
