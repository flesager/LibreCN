//! 5GSM messages (TS 24.501 §8.3) of PDU session establishment, for the SMF.
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
    pub const RELEASE_REQUEST: u8 = 0xd1;
    pub const STATUS: u8 = 0xd6;
}

/// The name of 5GSM message type `kind`.
pub fn name(kind: u8) -> &'static str {
    match kind {
        kind::ESTABLISHMENT_REQUEST => "PDU Session Establishment Request",
        kind::ESTABLISHMENT_ACCEPT => "PDU Session Establishment Accept",
        kind::ESTABLISHMENT_REJECT => "PDU Session Establishment Reject",
        kind::RELEASE_REQUEST => "PDU Session Release Request",
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

/// The uplink 5GSM messages the SMF handles, decoded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmUplink {
    EstablishmentRequest {
        /// Requested PDU session type, if any.
        session_type: Option<u8>,
        ssc_mode: Option<u8>,
    },
    Status {
        cause: u8,
    },
    /// Understood but not decoded: release, modification...
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
                let (mut session_type, mut ssc_mode) = (None, None);
                // Maximum number of supported packet filters (0x55) is TV.
                while let Some((iei, v)) = r.optional(&[(0x55, 2)])? {
                    match iei {
                        0x90 => session_type = Some(v[0] & 0x07),
                        0xa0 => ssc_mode = Some(v[0] & 0x07),
                        _ => {}
                    }
                }
                SmUplink::EstablishmentRequest { session_type, ssc_mode }
            }
            kind::STATUS => SmUplink::Status { cause: r.u8()? },
            other => SmUplink::Other(other),
        };
        Ok((header, message))
    }
}

/// The default QoS flow of an accepted session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QosFlow {
    pub qfi: u8,
    pub five_qi: u8,
}

/// The downlink 5GSM messages the SMF sends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmDownlink<'a> {
    EstablishmentAccept {
        /// IPv4 address of the UE.
        address: [u8; 4],
        ssc_mode: u8,
        flow: QosFlow,
        /// Session AMBR in Mbit/s: uplink, downlink.
        ambr_mbps: (u16, u16),
        snssai: Snssai,
        dnn: &'a str,
    },
    EstablishmentReject {
        cause: u8,
    },
    Status {
        cause: u8,
    },
}

/// Unit of the session AMBR values (§9.11.4.14): 1 Mbit/s.
const AMBR_UNIT_MBPS: u8 = 0x06;

impl SmDownlink<'_> {
    pub fn kind(&self) -> u8 {
        match self {
            SmDownlink::EstablishmentAccept { .. } => kind::ESTABLISHMENT_ACCEPT,
            SmDownlink::EstablishmentReject { .. } => kind::ESTABLISHMENT_REJECT,
            SmDownlink::Status { .. } => kind::STATUS,
        }
    }

    /// Encodes the message for PDU session `pdu_session_id`, answering
    /// procedure `pti`, into `out`; returns its length.
    pub fn encode(&self, pdu_session_id: u8, pti: u8, out: &mut [u8]) -> Result<usize> {
        let mut w = Writer::new(out);
        w.bytes(&[EPD_5GSM, pdu_session_id, pti, self.kind()]);
        match *self {
            SmDownlink::EstablishmentAccept { address, ssc_mode, flow, ambr_mbps, snssai, dnn } => {
                w.u8(ssc_mode << 4 | crate::sm::session_type::IPV4);
                // Authorized QoS rules (§9.11.4.13): the default rule, one
                // match-all packet filter, to the default flow.
                w.lve(|w| {
                    w.u8(1); // QoS rule identifier
                    w.bytes(&6u16.to_be_bytes());
                    w.u8(0x31); // create new rule, default (DQR), one packet filter
                    w.bytes(&[0x31, 0x01, 0x01]); // bidirectional filter 1: match-all
                    w.u8(0xff); // precedence
                    w.u8(flow.qfi & 0x3f);
                });
                let [ul_hi, ul_lo] = ambr_mbps.0.to_be_bytes();
                let [dl_hi, dl_lo] = ambr_mbps.1.to_be_bytes();
                w.lv(|w| w.bytes(&[AMBR_UNIT_MBPS, dl_hi, dl_lo, AMBR_UNIT_MBPS, ul_hi, ul_lo]));
                w.tlv(0x29, |w| {
                    w.u8(crate::sm::session_type::IPV4);
                    w.bytes(&address);
                });
                w.u8(0x22);
                snssai.encode(&mut w);
                // Authorized QoS flow descriptions (§9.11.4.12): the flow's 5QI.
                w.tlve(0x79, |w| w.bytes(&[flow.qfi & 0x3f, 0x20, 0x41, 0x01, 0x01, flow.five_qi]));
                w.tlv(0x25, |w| Dnn::encode(dnn, w));
            }
            SmDownlink::EstablishmentReject { cause } | SmDownlink::Status { cause } => w.u8(cause),
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
        assert_eq!(message, SmUplink::EstablishmentRequest { session_type: Some(1), ssc_mode: Some(1) });
        assert_eq!(SmUplink::decode(&hex("2e0101d1")).unwrap().1, SmUplink::Other(kind::RELEASE_REQUEST));
        assert_eq!(SmUplink::decode(&hex("2e01")), Err(Error::Truncated));
        assert!(SmUplink::decode(&hex("7e0001c1")).is_err());
    }

    /// The accept, byte for byte (TS 24.501 §8.3.2).
    #[test]
    fn an_establishment_accept_is_encoded() {
        let accept = SmDownlink::EstablishmentAccept {
            address: [10, 45, 0, 2],
            ssc_mode: 1,
            flow: QosFlow { qfi: 1, five_qi: 9 },
            ambr_mbps: (100, 1000),
            snssai: Snssai { sst: 1, sd: None },
            dnn: "internet",
        };
        let mut buf = [0u8; 128];
        let n = accept.encode(1, 1, &mut buf).unwrap();
        let expected = hex("2e 01 01 c2 11 0009 01 0006 31 310101 ff 01 06 0603e8060064 29 05 01 0a2d0002 22 01 01 \
             79 0006 012041010109 25 09 08696e7465726e6574");
        assert_eq!(buf[..n], expected[..]);
        let reject = SmDownlink::EstablishmentReject { cause: cause::MISSING_OR_UNKNOWN_DNN };
        let n = reject.encode(5, 3, &mut buf).unwrap();
        assert_eq!(buf[..n], hex("2e0503c31b")[..]);
    }
}
