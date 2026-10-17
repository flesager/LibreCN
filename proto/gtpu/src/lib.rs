//! GTP-U (TS 29.281) as N3 needs it, without allocation: decoding gives a
//! view of the payload in the received datagram; encoding writes the
//! header in front of a payload already in place.
//!
//! A G-PDU on N3 carries the PDU Session Container extension header
//! (TS 38.415): the QoS flow of the packet.

use std::fmt;

/// GTP-U's UDP port.
pub const PORT: u16 = 2152;

/// Message types (TS 29.281 §6.1).
pub mod kind {
    pub const ECHO_REQUEST: u8 = 1;
    pub const ECHO_RESPONSE: u8 = 2;
    pub const ERROR_INDICATION: u8 = 26;
    pub const END_MARKER: u8 = 254;
    pub const G_PDU: u8 = 255;
}

/// Version 1, protocol type GTP.
const FLAGS: u8 = 0x30;
const E: u8 = 0x04;
const S: u8 = 0x02;
const PN: u8 = 0x01;
/// PDU Session Container extension header type.
const PDU_SESSION_CONTAINER: u8 = 0x85;
/// Recovery IE of Echo Response, restart counter 0.
const RECOVERY: [u8; 2] = [14, 0];

/// Bytes of a G-PDU header written by [`write_g_pdu_header`]: the
/// mandatory 8, the optional 4, one 4-byte PDU Session Container.
pub const G_PDU_HEADER: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Truncated,
    /// Not GTP-U version 1.
    Version,
    /// The length field disagrees with the datagram.
    Length,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Error::Truncated => "truncated",
            Error::Version => "not GTP-U version 1",
            Error::Length => "length field disagrees with the datagram",
        })
    }
}

/// A decoded GTP-U message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Message<'a> {
    GPdu {
        teid: u32,
        /// From the PDU Session Container, if any.
        qfi: Option<u8>,
        payload: &'a [u8],
    },
    EchoRequest {
        seq: Option<u16>,
    },
    /// Any other type: echo response, error indication, end marker...
    Other {
        kind: u8,
        teid: u32,
    },
}

/// Decodes a datagram.
pub fn decode(datagram: &[u8]) -> Result<Message<'_>, Error> {
    let [flags, kind, l0, l1, t0, t1, t2, t3, rest @ ..] = datagram else { return Err(Error::Truncated) };
    if flags & 0xf0 != FLAGS {
        return Err(Error::Version);
    }
    let len = usize::from(u16::from_be_bytes([*l0, *l1]));
    let mut body = rest.get(..len).ok_or(Error::Length)?;
    let teid = u32::from_be_bytes([*t0, *t1, *t2, *t3]);
    let (mut seq, mut qfi) = (None, None);
    if flags & (E | S | PN) != 0 {
        let [s0, s1, _npdu, next, tail @ ..] = body else { return Err(Error::Truncated) };
        if flags & S != 0 {
            seq = Some(u16::from_be_bytes([*s0, *s1]));
        }
        let mut next = if flags & E != 0 { *next } else { 0 };
        body = tail;
        // Extension headers: length in 4-octet units, content, next type.
        while next != 0 {
            let units = usize::from(*body.first().ok_or(Error::Truncated)?);
            let header = body.get(..units * 4).filter(|_| units > 0).ok_or(Error::Truncated)?;
            if next == PDU_SESSION_CONTAINER && header.len() >= 4 {
                qfi = Some(header[2] & 0x3f);
            }
            next = header[header.len() - 1];
            body = &body[header.len()..];
        }
    }
    Ok(match *kind {
        kind::G_PDU => Message::GPdu { teid, qfi, payload: body },
        kind::ECHO_REQUEST => Message::EchoRequest { seq },
        kind => Message::Other { kind, teid },
    })
}

/// Writes in `buf[..G_PDU_HEADER]` the header of a downlink G-PDU for
/// `teid` and QoS flow `qfi`, whose `payload_len` bytes follow in
/// `buf[G_PDU_HEADER..]`.
pub fn write_g_pdu_header(buf: &mut [u8; G_PDU_HEADER], teid: u32, qfi: u8, payload_len: usize) {
    let len = (payload_len + G_PDU_HEADER - 8) as u16;
    let [l0, l1] = len.to_be_bytes();
    let [t0, t1, t2, t3] = teid.to_be_bytes();
    // DL PDU SESSION INFORMATION: PDU type 0, then the QFI.
    *buf = [FLAGS | E, kind::G_PDU, l0, l1, t0, t1, t2, t3, 0, 0, 0, PDU_SESSION_CONTAINER, 1, 0x00, qfi & 0x3f, 0];
}

/// An Echo Response to a request with `seq`, in `out`; its length.
pub fn echo_response(seq: Option<u16>, out: &mut [u8; 14]) -> usize {
    let [s0, s1] = seq.unwrap_or(0).to_be_bytes();
    *out = [FLAGS | S, kind::ECHO_RESPONSE, 0, 6, 0, 0, 0, 0, s0, s1, 0, 0, RECOVERY[0], RECOVERY[1]];
    14
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(s: &str) -> Vec<u8> {
        let s: String = s.split_whitespace().collect();
        (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
    }

    /// UERANSIM's uplink: E flag, UL PDU SESSION INFORMATION (type 1), QFI 1.
    #[test]
    fn an_uplink_g_pdu_gives_its_payload_and_qfi() {
        let datagram = hex("34ff 000c 00000001 0000 0085 01 10 01 00 45000000");
        let Message::GPdu { teid, qfi, payload } = decode(&datagram).unwrap() else { panic!() };
        assert_eq!((teid, qfi, payload), (1, Some(1), &hex("45000000")[..]));
        assert_eq!(
            decode(&hex("30ff 0004 00000007 45000000")).unwrap(),
            Message::GPdu { teid: 7, qfi: None, payload: &hex("45000000") }
        );
    }

    #[test]
    fn bad_datagrams_are_refused() {
        assert_eq!(decode(&hex("30ff 00")), Err(Error::Truncated));
        assert_eq!(decode(&hex("50ff 0000 00000001")), Err(Error::Version));
        assert_eq!(decode(&hex("30ff 0008 00000001 4500")), Err(Error::Length));
        assert_eq!(
            decode(&hex("34ff 0007 00000001 0000 0085 00 00 00")),
            Err(Error::Truncated),
            "zero-length extension"
        );
        assert_eq!(decode(&hex("34ff 0005 00000001 0000 0085 02")), Err(Error::Truncated));
    }

    #[test]
    fn a_downlink_header_round_trips_and_echo_is_answered() {
        let mut buf = [0u8; G_PDU_HEADER + 4];
        buf[G_PDU_HEADER..].copy_from_slice(&[0x45, 0, 0, 0]);
        write_g_pdu_header((&mut buf[..G_PDU_HEADER]).try_into().unwrap(), 0x77, 1, 4);
        assert_eq!(decode(&buf).unwrap(), Message::GPdu { teid: 0x77, qfi: Some(1), payload: &[0x45, 0, 0, 0] });
        assert_eq!(decode(&hex("3201 0004 00000000 1234 0000")).unwrap(), Message::EchoRequest { seq: Some(0x1234) });
        let mut out = [0u8; 14];
        let n = echo_response(Some(0x1234), &mut out);
        assert_eq!(out[..n], hex("3202 0006 00000000 1234 0000 0e00")[..]);
        assert_eq!(decode(&out).unwrap(), Message::Other { kind: kind::ECHO_RESPONSE, teid: 0 });
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(2000))]

        #[test]
        fn decoding_never_panics(datagram in proptest::collection::vec(proptest::prelude::any::<u8>(), 0..48)) {
            let _ = decode(&datagram);
            let mut gtp = datagram.clone();
            if gtp.len() >= 4 {
                gtp[0] = 0x34;
                let len = (gtp.len().saturating_sub(8)) as u16;
                gtp[2..4].copy_from_slice(&len.to_be_bytes());
                let _ = decode(&gtp);
            }
        }
    }
}
