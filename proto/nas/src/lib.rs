//! 5GS NAS (TS 24.501): the 5GMM messages of registration, decoded and
//! encoded without allocation, and NAS security (§4.4).
//!
//! - Decoding borrows from the PDU: [`mm::Uplink::decode`] gives views into
//!   the received bytes.
//! - Encoding writes into a buffer given by the caller:
//!   [`mm::Downlink::encode`] returns the length written.
//! - [`security`] protects and unprotects PDUs in place, with 128-NIA2 and
//!   128-NEA2 (or NEA0).
//!
//! Only the messages the AMF needs are decoded (uplink) or encoded
//! (downlink); unknown optional IEs are skipped, as TS 24.007 §11.2.4 lets
//! a receiver do.

pub mod ie;
pub mod mm;
pub mod security;

use std::fmt;

pub use ie::{Plmn, Snssai};

/// Extended protocol discriminators (TS 24.007 §11.2.3.1.1).
pub const EPD_5GMM: u8 = 0x7e;
pub const EPD_5GSM: u8 = 0x2e;

/// Why a PDU could not be decoded or encoded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// Shorter than its contents say.
    Truncated,
    /// A mandatory IE is missing or invalid (5GMM cause #96).
    Invalid(&'static str),
    /// Not a 5GMM message this codec decodes (5GMM cause #97).
    UnknownMessage(u8),
    /// The integrity check failed: the PDU must be discarded.
    Mac,
    /// The output buffer is too small.
    BufferTooSmall,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Truncated => f.write_str("truncated"),
            Error::Invalid(what) => write!(f, "invalid {what}"),
            Error::UnknownMessage(kind) => write!(f, "message type {kind:#04x} not handled"),
            Error::Mac => f.write_str("integrity check failed"),
            Error::BufferTooSmall => f.write_str("output buffer too small"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

/// Reads a PDU front to back.
#[derive(Clone, Copy)]
pub(crate) struct Reader<'a> {
    buf: &'a [u8],
}

impl<'a> Reader<'a> {
    pub(crate) fn new(buf: &'a [u8]) -> Self {
        Reader { buf }
    }

    pub(crate) fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.buf.len() < n {
            return Err(Error::Truncated);
        }
        let (head, tail) = self.buf.split_at(n);
        self.buf = tail;
        Ok(head)
    }

    pub(crate) fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    pub(crate) fn array<const N: usize>(&mut self) -> Result<[u8; N]> {
        Ok(self.take(N)?.try_into().expect("N octets taken"))
    }

    /// LV: one length octet.
    pub(crate) fn lv(&mut self) -> Result<&'a [u8]> {
        let n = self.u8()?;
        self.take(n.into())
    }

    /// LV-E: two length octets.
    pub(crate) fn lve(&mut self) -> Result<&'a [u8]> {
        let n = u16::from_be_bytes(self.array()?);
        self.take(n.into())
    }

    /// The next optional IE as `(IEI, value)`, `None` at the end.
    ///
    /// Format from the IEI (TS 24.007 §11.2.4): bit 8 set, one octet (type
    /// 1, keyed by the high nibble, the value is the whole octet, or type
    /// 2); `0x7x`, TLV-E; an IEI of `tv` (IEI, value octets), TV; else TLV.
    pub(crate) fn optional(&mut self, tv: &[(u8, usize)]) -> Result<Option<(u8, &'a [u8])>> {
        let Some(&iei) = self.buf.first() else { return Ok(None) };
        if iei & 0x80 != 0 {
            return Ok(Some((iei & 0xf0, self.take(1)?)));
        }
        self.take(1)?;
        let value = if iei & 0xf0 == 0x70 {
            self.lve()?
        } else if let Some(&(_, n)) = tv.iter().find(|(t, _)| *t == iei) {
            self.take(n)?
        } else {
            self.lv()?
        };
        Ok(Some((iei, value)))
    }
}

/// Writes a PDU into a caller's buffer. An overflow is remembered and
/// reported by [`finish`](Self::finish), so encoders need no error checks.
pub(crate) struct Writer<'a> {
    buf: &'a mut [u8],
    len: usize,
    overflow: bool,
}

impl<'a> Writer<'a> {
    pub(crate) fn new(buf: &'a mut [u8]) -> Self {
        Writer { buf, len: 0, overflow: false }
    }

    pub(crate) fn bytes(&mut self, bytes: &[u8]) {
        match self.buf.get_mut(self.len..self.len + bytes.len()) {
            Some(dst) => {
                dst.copy_from_slice(bytes);
                self.len += bytes.len();
            }
            None => self.overflow = true,
        }
    }

    pub(crate) fn u8(&mut self, v: u8) {
        self.bytes(&[v]);
    }

    /// Writes `content` after a length field of `octets` (1 or 2) octets.
    fn length_prefixed(&mut self, octets: usize, content: impl FnOnce(&mut Self)) {
        let at = self.len;
        self.bytes(&[0; 2][..octets]);
        content(self);
        if self.overflow {
            return;
        }
        let n = self.len - at - octets;
        match octets {
            1 if n <= 0xff => self.buf[at] = n as u8,
            2 if n <= 0xffff => self.buf[at..at + 2].copy_from_slice(&(n as u16).to_be_bytes()),
            _ => self.overflow = true,
        }
    }

    pub(crate) fn lv(&mut self, content: impl FnOnce(&mut Self)) {
        self.length_prefixed(1, content);
    }

    pub(crate) fn lve(&mut self, content: impl FnOnce(&mut Self)) {
        self.length_prefixed(2, content);
    }

    pub(crate) fn tlv(&mut self, iei: u8, content: impl FnOnce(&mut Self)) {
        self.u8(iei);
        self.lv(content);
    }

    pub(crate) fn tlve(&mut self, iei: u8, content: impl FnOnce(&mut Self)) {
        self.u8(iei);
        self.lve(content);
    }

    pub(crate) fn finish(self) -> Result<usize> {
        if self.overflow { Err(Error::BufferTooSmall) } else { Ok(self.len) }
    }
}

/// A short description of a NAS PDU, for logs: protocol, message, security.
pub fn describe(pdu: &[u8]) -> Describe<'_> {
    Describe(pdu)
}

/// See [`describe`].
pub struct Describe<'a>(&'a [u8]);

impl fmt::Display for Describe<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            [EPD_5GMM, 0, kind, ..] => write!(f, "5GMM {} (plain)", mm::name(*kind)),
            // Protected: MAC (4) and sequence number (1), then the inner
            // message, readable unless ciphered.
            [EPD_5GMM, header @ (1 | 3), _, _, _, _, _, EPD_5GMM, 0, kind, ..] => {
                write!(f, "5GMM {} ({})", mm::name(*kind), security::Header::name(*header))
            }
            [EPD_5GMM, header, ..] => write!(f, "5GMM message ({})", security::Header::name(*header)),
            [EPD_5GSM, ..] => f.write_str("5GSM message"),
            _ => write!(f, "unknown NAS ({} bytes)", self.0.len()),
        }
    }
}

#[cfg(test)]
pub(crate) fn hex(s: &str) -> Vec<u8> {
    let s: String = s.split_whitespace().collect();
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_ies_are_walked_by_format() {
        // Type 1 (0xE1), TLV (0x2D), TLV-E (0x71), TV (0x52, 6 octets).
        let pdu = hex("e1 2d0201ff 7100020a0b 52 999f07000001");
        let mut r = Reader::new(&pdu);
        let tv = [(0x52, 6)];
        assert_eq!(r.optional(&tv).unwrap(), Some((0xe0, &[0xe1][..])));
        assert_eq!(r.optional(&tv).unwrap(), Some((0x2d, &[0x01, 0xff][..])));
        assert_eq!(r.optional(&tv).unwrap(), Some((0x71, &[0x0a, 0x0b][..])));
        assert_eq!(r.optional(&tv).unwrap(), Some((0x52, &hex("999f07000001")[..])));
        assert_eq!(r.optional(&tv).unwrap(), None);
        assert_eq!(Reader::new(&hex("2d05ff")).optional(&[]), Err(Error::Truncated));
    }

    #[test]
    fn the_writer_reports_overflow_once_at_the_end() {
        let mut buf = [0u8; 4];
        let mut w = Writer::new(&mut buf);
        w.tlv(0x2d, |w| w.bytes(&[1, 2]));
        assert_eq!(w.finish(), Ok(4));
        assert_eq!(buf, [0x2d, 2, 1, 2]);
        let mut w = Writer::new(&mut buf);
        w.lve(|w| w.bytes(&[1, 2, 3]));
        assert_eq!(w.finish(), Err(Error::BufferTooSmall));
    }

    #[test]
    fn pdus_are_described_for_logs() {
        assert_eq!(describe(&[0x7e, 0x00, 0x41]).to_string(), "5GMM Registration Request (plain)");
        let protected = hex("7e01 11223344 00 7e0043");
        assert_eq!(describe(&protected).to_string(), "5GMM Registration Complete (integrity protected)");
        assert_eq!(
            describe(&hex("7e02 11223344 00 ffff")).to_string(),
            "5GMM message (integrity protected and ciphered)"
        );
        assert_eq!(describe(&[0x2e, 1]).to_string(), "5GSM message");
        assert_eq!(describe(&[]).to_string(), "unknown NAS (0 bytes)");
    }
}
