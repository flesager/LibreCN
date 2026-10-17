//! NAS security (TS 24.501 §4.4, TS 33.501 §6.4): the security protected
//! 5GMM message, and the AMF side of a 5G NAS security context.
//!
//! A protected message is: EPD, security header type, MAC (4 octets),
//! sequence number (1 octet), then the plain message, ciphered for the
//! "ciphered" header types. The MAC covers the sequence number and the
//! (ciphered) message.

use librecn_security::{Direction, kdf, nea2, nia2};

use crate::{EPD_5GMM, Error, Result};

/// Security header types (§9.3.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Header {
    Plain = 0,
    Integrity = 1,
    IntegrityCiphered = 2,
    /// Security Mode Command only.
    IntegrityNewContext = 3,
    /// Security Mode Complete only.
    IntegrityCipheredNewContext = 4,
}

impl Header {
    /// For logs.
    pub fn name(header: u8) -> &'static str {
        match header {
            0 => "plain",
            1 => "integrity protected",
            2 => "integrity protected and ciphered",
            3 => "integrity protected, new security context",
            4 => "integrity protected and ciphered, new security context",
            _ => "unknown security header",
        }
    }
}

/// The security header of a 5GMM PDU.
pub fn header(pdu: &[u8]) -> Option<u8> {
    match pdu {
        [EPD_5GMM, h, ..] => Some(h & 0x0f),
        _ => None,
    }
}

/// The inner message of an integrity protected (not ciphered) PDU, read
/// without checking the MAC: for an initial message whose security context
/// the AMF does not have (§4.4.6), which may only carry cleartext IEs.
pub fn skip_header(pdu: &[u8]) -> Result<&[u8]> {
    match pdu {
        [EPD_5GMM, h, _, _, _, _, _, inner @ ..] if h & 0x0f == Header::Integrity as u8 => Ok(inner),
        [EPD_5GMM, ..] => Err(Error::Invalid("security header")),
        _ => Err(Error::Truncated),
    }
}

/// Algorithm identities (TS 33.501 §5.11.1.1): only these two run here.
pub const NEA0: u8 = 0;
pub const NEA2: u8 = 2;
pub const NIA2: u8 = 2;

/// BEARER of the NAS algorithms: the NAS connection identifier, 1 for
/// 3GPP access (as UERANSIM checks).
const BEARER: u8 = 1;

/// A current 5G NAS security context, AMF side.
#[derive(Clone)]
pub struct SecurityContext {
    pub ngksi: u8,
    /// NEA0 or NEA2.
    pub ciphering: u8,
    /// NIA2.
    pub integrity: u8,
    k_enc: [u8; 16],
    k_int: [u8; 16],
    /// Next uplink NAS COUNT expected (overflow << 8 | sequence number).
    ul_count: u32,
    /// Next downlink NAS COUNT.
    dl_count: u32,
}

impl std::fmt::Debug for SecurityContext {
    /// Without the keys.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (k, c, i, ul, dl) = (self.ngksi, self.ciphering, self.integrity, self.ul_count, self.dl_count);
        write!(f, "SecurityContext {{ ngksi: {k}, NEA{c}, NIA{i}, ul: {ul}, dl: {dl} }}")
    }
}

impl SecurityContext {
    /// A new context from K_AMF, counts at 0.
    pub fn new(kamf: &[u8; 32], ngksi: u8, ciphering: u8, integrity: u8) -> Self {
        let (k_enc, k_int) = kdf::nas_keys(kamf, ciphering, integrity);
        SecurityContext { ngksi, ciphering, integrity, k_enc, k_int, ul_count: 0, dl_count: 0 }
    }

    /// The NAS COUNT of the last uplink message accepted (0 before any).
    pub fn last_uplink_count(&self) -> u32 {
        self.ul_count.saturating_sub(1)
    }

    fn cipher(&self, count: u32, direction: Direction, data: &mut [u8]) {
        if self.ciphering == NEA2 {
            nea2(&self.k_enc, count, BEARER, direction, data);
        }
    }

    /// Protects `plain` into `out` with header type `header` (1 to 4);
    /// returns the length. Ciphers for types 2 and 4.
    pub fn protect(&mut self, plain: &[u8], header: Header, out: &mut [u8]) -> Result<usize> {
        let len = 7 + plain.len();
        let out = out.get_mut(..len).ok_or(Error::BufferTooSmall)?;
        let count = self.dl_count;
        out[0] = EPD_5GMM;
        out[1] = header as u8;
        out[6] = count as u8;
        out[7..].copy_from_slice(plain);
        if matches!(header, Header::IntegrityCiphered | Header::IntegrityCipheredNewContext) {
            self.cipher(count, Direction::Downlink, &mut out[7..]);
        }
        let mac = nia2(&self.k_int, count, BEARER, Direction::Downlink, &out[6..]);
        out[2..6].copy_from_slice(&mac);
        self.dl_count = (count + 1) & 0xff_ffff;
        Ok(len)
    }

    /// Checks an uplink protected PDU and deciphers it in place; returns
    /// the plain message. A failed check leaves the counts unchanged.
    pub fn unprotect<'a>(&mut self, pdu: &'a mut [u8]) -> Result<&'a [u8]> {
        if pdu.len() < 8 || pdu[0] != EPD_5GMM {
            return Err(Error::Truncated);
        }
        let header = pdu[1] & 0x0f;
        if !(1..=4).contains(&header) {
            return Err(Error::Invalid("security header"));
        }
        // NAS COUNT estimate (TS 33.501 §6.4.3.1): the overflow counter
        // moves on when the sequence number wraps.
        let mut count = (self.ul_count & !0xff) | u32::from(pdu[6]);
        if count < self.ul_count {
            count += 0x100;
        }
        let mac = nia2(&self.k_int, count, BEARER, Direction::Uplink, &pdu[6..]);
        if mac != pdu[2..6] {
            return Err(Error::Mac);
        }
        self.ul_count = (count + 1) & 0xff_ffff;
        if header == 2 || header == 4 {
            self.cipher(count, Direction::Uplink, &mut pdu[7..]);
        }
        Ok(&pdu[7..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex;

    /// A UE side for the tests: the same context with the directions swapped.
    fn ue_protect(ctx: &SecurityContext, count: u32, header: Header, plain: &[u8]) -> Vec<u8> {
        let mut pdu = [&[EPD_5GMM, header as u8, 0, 0, 0, 0, count as u8][..], plain].concat();
        if header == Header::IntegrityCiphered || header == Header::IntegrityCipheredNewContext {
            ctx.cipher(count, Direction::Uplink, &mut pdu[7..]);
        }
        let mac = nia2(&ctx.k_int, count, BEARER, Direction::Uplink, &pdu[6..]);
        pdu[2..6].copy_from_slice(&mac);
        pdu
    }

    fn context() -> SecurityContext {
        SecurityContext::new(&[7; 32], 0, NEA2, NIA2)
    }

    /// A Security Mode Command protected with the NAS keys of librecn-security's
    /// tests; the MAC is UERANSIM's.
    #[test]
    fn a_downlink_pdu_carries_the_mac_of_its_count() {
        let kamf = hex("3b342433dd37dc9ad17e14fd47f1df93417befff787891fa710c7b1c37801052");
        let mut ctx = SecurityContext::new(&kamf.try_into().unwrap(), 0, NEA2, NIA2);
        ctx.dl_count = 0x102;
        let plain = hex("7e005d020004f0f0f0f0e1360100");
        let mut out = [0u8; 64];
        let n = ctx.protect(&plain, Header::IntegrityNewContext, &mut out).unwrap();
        assert_eq!(out[..n], [&hex("7e03 77fcdb45 02")[..], &plain].concat()[..]);
        assert_eq!(ctx.dl_count, 0x103);
        assert!(ctx.protect(&plain, Header::Integrity, &mut [0u8; 8]).is_err());
    }

    #[test]
    fn uplink_pdus_are_checked_deciphered_and_counted() {
        let mut amf = context();
        let ue = context();
        let plain = hex("7e0043");
        let mut pdu = ue_protect(&ue, 0, Header::IntegrityCiphered, &plain);
        assert_ne!(pdu[7..], plain[..], "ciphered");
        assert_eq!(amf.unprotect(&mut pdu).unwrap(), &plain[..]);

        let mut replay = ue_protect(&ue, 0, Header::Integrity, &plain);
        assert_eq!(amf.unprotect(&mut replay), Err(Error::Mac), "count 0 is past: read as 256");
        let mut tampered = ue_protect(&ue, 1, Header::Integrity, &plain);
        tampered[9] ^= 1;
        assert_eq!(amf.unprotect(&mut tampered), Err(Error::Mac));
        assert_eq!(amf.ul_count, 1, "a failed check changes nothing");
        assert_eq!(amf.unprotect(&mut ue_protect(&ue, 1, Header::Integrity, &plain)).unwrap(), &plain[..]);
    }

    #[test]
    fn the_sequence_number_wraps_into_the_overflow_counter() {
        let mut amf = context();
        let ue = context();
        amf.ul_count = 0xff;
        let mut pdu = ue_protect(&ue, 0x100, Header::Integrity, &[0x7e, 0, 0x43]);
        assert_eq!(pdu[6], 0, "sequence number 0 on the wire");
        let mut early = ue_protect(&ue, 0xff, Header::Integrity, &[0x7e, 0, 0x43]);
        assert!(amf.unprotect(&mut early).is_ok());
        assert!(amf.unprotect(&mut pdu).is_ok(), "estimated as 0x100");
        assert_eq!(amf.ul_count, 0x101);
    }

    #[test]
    fn an_unchecked_initial_message_is_read_past_its_header() {
        assert_eq!(skip_header(&hex("7e01 00000000 00 7e0041")), Ok(&hex("7e0041")[..]));
        assert!(skip_header(&hex("7e02 00000000 00 ffff")).is_err(), "ciphered: unreadable");
        assert_eq!(header(&[0x7e, 0x04]), Some(4));
    }
}
