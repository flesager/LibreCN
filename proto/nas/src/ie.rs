//! Information elements (TS 24.501 §9.11) used by registration.

use std::fmt;

use crate::{Error, Reader, Result, Writer};

/// A PLMN as its 3 BCD octets, the encoding shared by NAS and NGAP
/// (TS 24.008 §10.5.1.13, TS 38.413 §9.3.3.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Plmn(pub [u8; 3]);

/// Up to 3 decimal digits, displayed as they are (padding honoured).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Digits {
    ascii: [u8; 3],
    len: u8,
}

impl fmt::Display for Digits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(std::str::from_utf8(&self.ascii[..usize::from(self.len)]).unwrap_or("?"))
    }
}

impl Plmn {
    /// From MCC (3 digits) and MNC (2 or 3 digits).
    pub fn new(mcc: &str, mnc: &str) -> std::result::Result<Self, String> {
        let digits = |s: &str, n: &[usize]| -> std::result::Result<[u8; 3], String> {
            if !n.contains(&s.len()) || !s.bytes().all(|b| b.is_ascii_digit()) {
                return Err(format!("{s:?}: expected {n:?} digits"));
            }
            let mut d = [0xf; 3];
            for (d, b) in d.iter_mut().zip(s.bytes()) {
                *d = b - b'0';
            }
            Ok(d)
        };
        let (c, n) = (digits(mcc, &[3])?, digits(mnc, &[2, 3])?);
        Ok(Plmn([c[1] << 4 | c[0], n[2] << 4 | c[2], n[1] << 4 | n[0]]))
    }

    fn digits(nibbles: [u8; 3]) -> Digits {
        let len = nibbles.iter().take_while(|&&d| d <= 9).count();
        Digits { ascii: nibbles.map(|d| b'0' + d.min(9)), len: len as u8 }
    }

    /// Mobile country code.
    pub fn mcc(&self) -> Digits {
        let [a, b, _] = self.0;
        Self::digits([a & 0xf, a >> 4, b & 0xf])
    }

    /// Mobile network code, 2 or 3 digits.
    pub fn mnc(&self) -> Digits {
        let [_, b, c] = self.0;
        Self::digits([c & 0xf, c >> 4, b >> 4])
    }
}

impl fmt::Display for Plmn {
    /// `mcc/mnc`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.mcc(), self.mnc())
    }
}

/// A network slice (§9.11.2.8, mapped values ignored).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Snssai {
    /// Slice/service type.
    pub sst: u8,
    /// Slice differentiator.
    pub sd: Option<[u8; 3]>,
}

impl Snssai {
    /// From an S-NSSAI value without its length.
    fn decode(v: &[u8]) -> Result<Self> {
        match v.len() {
            1 | 2 => Ok(Snssai { sst: v[0], sd: None }),
            4 | 5 | 8 => Ok(Snssai { sst: v[0], sd: Some([v[1], v[2], v[3]]) }),
            _ => Err(Error::Invalid("S-NSSAI")),
        }
    }

    fn encode(&self, w: &mut Writer<'_>) {
        w.lv(|w| {
            w.u8(self.sst);
            if let Some(sd) = self.sd {
                w.bytes(&sd);
            }
        });
    }
}

/// The S-NSSAIs of an NSSAI IE (§9.11.3.37), as they are decoded.
#[derive(Clone, Copy, Debug)]
pub struct Nssai<'a>(pub(crate) &'a [u8]);

impl Iterator for Nssai<'_> {
    type Item = Result<Snssai>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }
        let mut r = Reader::new(self.0);
        let item = r.lv().and_then(Snssai::decode);
        self.0 = if item.is_ok() { r.buf } else { &[] };
        Some(item)
    }
}

/// At most 8 slices, the bound of an allowed NSSAI (§9.11.3.37).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SnssaiList {
    items: [Option<Snssai>; 8],
}

impl SnssaiList {
    /// Adds `s` unless full or already there.
    pub fn push(&mut self, s: Snssai) {
        if self.iter().any(|x| x == s) {
            return;
        }
        if let Some(slot) = self.items.iter_mut().find(|i| i.is_none()) {
            *slot = Some(s);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Snssai> + '_ {
        self.items.iter().map_while(|s| *s)
    }

    pub fn is_empty(&self) -> bool {
        self.items[0].is_none()
    }

    pub(crate) fn encode(&self, w: &mut Writer<'_>) {
        for s in self.iter() {
            s.encode(w);
        }
    }
}

/// NAS key set identifier (§9.11.3.32): a native KSI 0–6, or 7 for "no key".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NgKsi {
    /// Type of security context: mapped (from EPS) rather than native.
    pub mapped: bool,
    pub value: u8,
}

impl NgKsi {
    /// No key is available.
    pub const NONE: u8 = 7;

    pub(crate) fn from_nibble(n: u8) -> Self {
        NgKsi { mapped: n & 0x8 != 0, value: n & 0x7 }
    }

    pub(crate) fn nibble(self) -> u8 {
        u8::from(self.mapped) << 3 | self.value & 0x7
    }
}

/// UE security capability (§9.11.3.54), kept as received: the AMF replays it
/// unchanged in the Security Mode Command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UeSecurityCapability {
    octets: [u8; 8],
    len: u8,
}

impl UeSecurityCapability {
    pub fn decode(v: &[u8]) -> Result<Self> {
        if !(2..=8).contains(&v.len()) {
            return Err(Error::Invalid("UE security capability"));
        }
        let mut octets = [0; 8];
        octets[..v.len()].copy_from_slice(v);
        Ok(UeSecurityCapability { octets, len: v.len() as u8 })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.octets[..usize::from(self.len)]
    }

    /// Whether 5G-EA`n` (ciphering, 0–7) is supported. EA0 always is.
    pub fn ciphering(&self, n: u8) -> bool {
        n == 0 || n < 8 && self.octets[0] & (0x80 >> n) != 0
    }

    /// Whether 5G-IA`n` (integrity, 0–7) is supported.
    pub fn integrity(&self, n: u8) -> bool {
        n < 8 && self.octets[1] & (0x80 >> n) != 0
    }
}

/// A 5G-GUTI (§9.11.3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Guti {
    pub plmn: Plmn,
    pub amf_region: u8,
    /// 10 bits.
    pub amf_set: u16,
    /// 6 bits.
    pub amf_pointer: u8,
    pub tmsi: u32,
}

impl Guti {
    fn decode(v: &[u8]) -> Result<Self> {
        let v: &[u8; 11] = v.try_into().map_err(|_| Error::Invalid("5G-GUTI"))?;
        Ok(Guti {
            plmn: Plmn([v[1], v[2], v[3]]),
            amf_region: v[4],
            amf_set: u16::from(v[5]) << 2 | u16::from(v[6] >> 6),
            amf_pointer: v[6] & 0x3f,
            tmsi: u32::from_be_bytes([v[7], v[8], v[9], v[10]]),
        })
    }

    pub(crate) fn encode(&self, w: &mut Writer<'_>) {
        w.u8(0xf0 | IdentityType::Guti as u8);
        w.bytes(&self.plmn.0);
        w.u8(self.amf_region);
        w.u8((self.amf_set >> 2) as u8);
        w.u8(((self.amf_set & 0x3) as u8) << 6 | self.amf_pointer & 0x3f);
        w.bytes(&self.tmsi.to_be_bytes());
    }
}

impl fmt::Display for Guti {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let g = self;
        write!(f, "5G-GUTI {} {}:{}:{} {:08x}", g.plmn, g.amf_region, g.amf_set, g.amf_pointer, g.tmsi)
    }
}

/// A SUCI (§9.11.3.4, TS 23.003 §2.2B) whose SUPI is an IMSI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Suci<'a> {
    pub plmn: Plmn,
    /// Routing indicator, 4 BCD digits (`F` filled).
    pub routing: [u8; 2],
    /// 0: null scheme, 1: ECIES profile A, 2: profile B.
    pub protection_scheme: u8,
    pub key_id: u8,
    /// The MSIN in BCD for the null scheme, else the ciphered MSIN.
    pub output: &'a [u8],
}

impl Suci<'_> {
    pub const NULL_SCHEME: u8 = 0;
}

impl fmt::Display for Suci<'_> {
    /// The SBI form (TS 29.503 §6.1.6.3.2):
    /// `suci-0-<mcc>-<mnc>-<routing>-<scheme>-<key id>-<output>`, the output
    /// as digits for the null scheme, else in hexadecimal.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "suci-0-{}-{}-", self.plmn.mcc(), self.plmn.mnc())?;
        bcd(f, &self.routing)?;
        write!(f, "-{}-{}-", self.protection_scheme, self.key_id)?;
        if self.protection_scheme == Self::NULL_SCHEME {
            bcd(f, self.output)
        } else {
            self.output.iter().try_for_each(|b| write!(f, "{b:02x}"))
        }
    }
}

/// BCD digits, low nibble first, up to the first filler.
fn bcd(f: &mut fmt::Formatter<'_>, octets: &[u8]) -> fmt::Result {
    for d in octets.iter().flat_map(|b| [b & 0xf, b >> 4]).take_while(|&d| d <= 9) {
        write!(f, "{d}")?;
    }
    Ok(())
}

/// Identity types (§9.11.3.3, §9.11.3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum IdentityType {
    Suci = 1,
    Guti = 2,
    Imei = 3,
    STmsi = 4,
    Imeisv = 5,
}

/// 5GS mobile identity (§9.11.3.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MobileIdentity<'a> {
    NoIdentity,
    Suci(Suci<'a>),
    /// A SUCI whose SUPI is a network access identifier: not served.
    SuciNai,
    Guti(Guti),
    /// IMEI or IMEISV digits (BCD, first digit in the high nibble of the
    /// first octet).
    Imei(&'a [u8]),
    Imeisv(&'a [u8]),
    /// 5G-S-TMSI, MAC address, EUI-64, ...
    Other(u8),
}

impl<'a> MobileIdentity<'a> {
    pub(crate) fn decode(v: &'a [u8]) -> Result<Self> {
        let Some(&first) = v.first() else { return Err(Error::Invalid("5GS mobile identity")) };
        Ok(match first & 0x7 {
            0 => MobileIdentity::NoIdentity,
            1 if (first >> 4) & 0x7 != 0 => MobileIdentity::SuciNai,
            1 => {
                let mut r = Reader::new(&v[1..]);
                let suci = Suci {
                    plmn: Plmn(r.array()?),
                    routing: r.array()?,
                    protection_scheme: r.u8()? & 0xf,
                    key_id: r.u8()?,
                    output: r.buf,
                };
                MobileIdentity::Suci(suci)
            }
            2 => MobileIdentity::Guti(Guti::decode(v)?),
            3 => MobileIdentity::Imei(v),
            5 => MobileIdentity::Imeisv(v),
            other => MobileIdentity::Other(other),
        })
    }
}

impl fmt::Display for MobileIdentity<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let digits = |f: &mut fmt::Formatter<'_>, v: &[u8]| {
            write!(f, "{}", v[0] >> 4)?;
            bcd(f, &v[1..])
        };
        match self {
            MobileIdentity::NoIdentity => f.write_str("no identity"),
            MobileIdentity::Suci(suci) => suci.fmt(f),
            MobileIdentity::SuciNai => f.write_str("SUCI of a NAI"),
            MobileIdentity::Guti(guti) => guti.fmt(f),
            MobileIdentity::Imei(v) => {
                f.write_str("imei-")?;
                digits(f, v)
            }
            MobileIdentity::Imeisv(v) => {
                f.write_str("imeisv-")?;
                digits(f, v)
            }
            MobileIdentity::Other(kind) => write!(f, "identity of type {kind}"),
        }
    }
}

/// A tracking area: PLMN and 24-bit TAC.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tai {
    pub plmn: Plmn,
    pub tac: [u8; 3],
}

impl Tai {
    /// As a TAI list (§9.11.3.9) of one element, type "00".
    pub(crate) fn encode_list(&self, w: &mut Writer<'_>) {
        w.u8(0x00);
        w.bytes(&self.plmn.0);
        w.bytes(&self.tac);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex;

    #[test]
    fn plmn_is_bcd_with_its_digits() {
        assert_eq!(Plmn::new("208", "93").unwrap().0, [0x02, 0xf8, 0x39], "free5GC's test PLMN");
        assert_eq!(Plmn::new("999", "70").unwrap().0, [0x99, 0xf9, 0x07]);
        assert_eq!(Plmn::new("310", "410").unwrap().0, [0x13, 0x00, 0x14], "3-digit MNC");
        assert_eq!(Plmn::new("310", "410").unwrap().to_string(), "310/410");
        let p = Plmn::new("999", "70").unwrap();
        assert_eq!((p.to_string(), format!("{:0>3}", p.mnc())), ("999/70".into(), "070".into()));
        assert!(Plmn::new("99", "70").is_err() && Plmn::new("999", "7a").is_err());
    }

    #[test]
    fn a_null_scheme_suci_reads_as_its_sbi_form() {
        // UERANSIM's sample UE: imsi-999700000000001, routing indicator 0000.
        let v = hex("01 99f907 0000 00 00 0000000010");
        let MobileIdentity::Suci(suci) = MobileIdentity::decode(&v).unwrap() else { panic!() };
        assert_eq!(suci.to_string(), "suci-0-999-70-0000-0-0-0000000001");
        let v = hex("01 99f907 21f3 01 05 abcd");
        assert_eq!(MobileIdentity::decode(&v).unwrap().to_string(), "suci-0-999-70-123-1-5-abcd");
        assert_eq!(MobileIdentity::decode(&hex("11 00")).unwrap(), MobileIdentity::SuciNai);
        assert!(MobileIdentity::decode(&hex("01 99f9")).is_err());
    }

    #[test]
    fn guti_round_trips_and_other_identities_display() {
        let guti =
            Guti { plmn: Plmn::new("999", "70").unwrap(), amf_region: 1, amf_set: 0x3ff, amf_pointer: 5, tmsi: 7 };
        let mut buf = [0u8; 11];
        let mut w = Writer::new(&mut buf);
        guti.encode(&mut w);
        assert_eq!(w.finish(), Ok(11));
        assert_eq!(buf[..], hex("f2 99f907 01 ffc5 00000007")[..]);
        assert_eq!(MobileIdentity::decode(&buf).unwrap(), MobileIdentity::Guti(guti));
        assert_eq!(guti.to_string(), "5G-GUTI 999/70 1:1023:5 00000007");
        // UERANSIM's IMEISV 4370816125816151 (even: filler in the last octet).
        let imeisv = hex("45 73 80 61 21 85 61 51 f1");
        assert_eq!(MobileIdentity::decode(&imeisv).unwrap().to_string(), "imeisv-4370816125816151");
    }

    #[test]
    fn nssai_and_capabilities() {
        let items: Vec<_> = Nssai(&hex("0101 04 01000001 0501")).collect();
        assert_eq!(items[0], Ok(Snssai { sst: 1, sd: None }));
        assert_eq!(items[1], Ok(Snssai { sst: 1, sd: Some([0, 0, 1]) }));
        assert_eq!(items[2], Err(Error::Truncated));
        let caps = UeSecurityCapability::decode(&hex("f0f0")).unwrap();
        assert!(caps.ciphering(2) && caps.integrity(2) && !caps.integrity(4));
        assert!(UeSecurityCapability::decode(&[0xf0]).is_err());
    }
}
