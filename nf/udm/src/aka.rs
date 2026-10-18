//! The UE authentication service, on domain values: SUCI de-concealment
//! and 5G home environment authentication vectors (TS 33.501 §6.1.3.2).

use librecn_sbi::{Outcome, ServingNetwork, Supi};
use librecn_security::milenage::Milenage;
use librecn_security::{ecies, kdf};

use crate::config::{HomeNetworkKeys, Subscriber};

/// The SUPI of `supi_or_suci` (the SIDF, TS 33.501 §6.12.2): a SUPI as it
/// is; a SUCI of the null scheme read, of Profile A (scheme 1) deciphered
/// with the home network key it names. Profile B is not supported.
pub fn deconceal(supi_or_suci: &str, keys: &HomeNetworkKeys) -> Result<Supi, Outcome> {
    if supi_or_suci.starts_with("imsi-") {
        return Supi::new(supi_or_suci).ok_or(Outcome::UserNotFound);
    }
    let mut parts = supi_or_suci.split('-');
    let mut next = || parts.next().ok_or(Outcome::UserNotFound);
    let (prefix, kind, mcc, mnc, _routing, scheme, key, output) =
        (next()?, next()?, next()?, next()?, next()?, next()?, next()?, next()?);
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if prefix != "suci" || kind != "0" || !digits(mcc) || !digits(mnc) {
        return Err(Outcome::UserNotFound);
    }
    let mut msin = [0u8; 20];
    let msin = match scheme {
        "0" => output,
        "1" => {
            let private =
                key.parse().ok().and_then(|k: u8| keys.get(&k)).ok_or(Outcome::UnsupportedProtectionScheme)?;
            let mut bytes = [0u8; 64];
            let n = unhex(output, &mut bytes).ok_or(Outcome::UserNotFound)?;
            let plain = ecies::deconceal(private, &mut bytes[..n]).map_err(|_| Outcome::UserNotFound)?;
            bcd_digits(&bytes[plain], &mut msin).ok_or(Outcome::UserNotFound)?
        }
        _ => return Err(Outcome::UnsupportedProtectionScheme),
    };
    if !digits(msin) {
        return Err(Outcome::UserNotFound);
    }
    Supi::format(format_args!("imsi-{mcc}{mnc}{msin}")).ok_or(Outcome::UserNotFound)
}

/// Hexadecimal `s` into `out`: its length, `None` if it is not hex or too long.
fn unhex(s: &str, out: &mut [u8]) -> Option<usize> {
    let n = s.len() / 2;
    if !s.len().is_multiple_of(2) || n > out.len() {
        return None;
    }
    for (i, o) in out[..n].iter_mut().enumerate() {
        *o = u8::from_str_radix(s.get(2 * i..2 * i + 2)?, 16).ok()?;
    }
    Some(n)
}

/// The digits of BCD `bcd` (low nibble first, 0xF filling the last one), in `out`.
fn bcd_digits<'a>(bcd: &[u8], out: &'a mut [u8; 20]) -> Option<&'a str> {
    let mut n = 0;
    for nibble in bcd.iter().flat_map(|b| [b & 0x0f, b >> 4]) {
        match nibble {
            0..=9 if n < out.len() => {
                out[n] = b'0' + nibble;
                n += 1;
            }
            0x0f => break,
            _ => return None,
        }
    }
    std::str::from_utf8(&out[..n]).ok()
}

/// SQN = SEQ || IND with a 5-bit IND (TS 33.102 Annex C.3.2), IND always 0:
/// each vector takes the next SEQ.
const IND_BITS: u32 = 5;
const SQN_MASK: u64 = (1 << 48) - 1;

/// An authentication vector: RAND, AUTN, and what the method needs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vector {
    pub rand: [u8; 16],
    pub autn: [u8; 16],
    pub method: Method,
}

/// The method's part of a vector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    /// A 5G HE AV (TS 33.501 §6.1.3.2): XRES*, K_AUSF.
    FiveGAka { xres_star: [u8; 16], kausf: [u8; 32] },
    /// An EAP-AKA' AV (§6.1.3.1): XRES, CK', IK'.
    EapAkaPrime { xres: [u8; 8], ck_prime: [u8; 16], ik_prime: [u8; 16] },
}

/// Resynchronises the subscriber's SQN on the UE's (TS 33.102 §6.3.5): the
/// UE sent AUTS = SQN_MS ⊕ AK* || MAC-S for the challenge with `rand`.
/// False if MAC-S is wrong (the SQN is then left as it was).
pub fn resynchronise(sub: &mut Subscriber, rand: &[u8; 16], auts: &[u8; 14]) -> bool {
    let m = Milenage::new(&sub.k, &sub.opc);
    let ak = m.f5_star(rand);
    let sqn_ms: [u8; 6] = std::array::from_fn(|i| auts[i] ^ ak[i]);
    // MAC-S is computed with a dummy AMF of zeros (TS 33.102 §6.3.3).
    if m.f1(rand, &sqn_ms, &[0, 0]).1 != auts[6..] {
        return false;
    }
    sub.sqn = sqn_ms.iter().fold(0, |a, &b| a << 8 | u64::from(b));
    true
}

/// A new vector with the subscriber's next SQN.
pub fn vector(sub: &mut Subscriber, serving_network: &ServingNetwork, rand: [u8; 16]) -> Vector {
    sub.sqn = ((sub.sqn >> IND_BITS) + 1) << IND_BITS & SQN_MASK;
    let sqn: [u8; 6] = sub.sqn.to_be_bytes()[2..].try_into().expect("6 octets");
    let m = Milenage::new(&sub.k, &sub.opc);
    let (mac_a, _) = m.f1(&rand, &sqn, &sub.amf);
    let v = m.f2345(&rand);
    let sqn_xor_ak: [u8; 6] = std::array::from_fn(|i| sqn[i] ^ v.ak[i]);
    let mut autn = [0u8; 16];
    autn[..6].copy_from_slice(&sqn_xor_ak);
    autn[6..8].copy_from_slice(&sub.amf);
    autn[8..].copy_from_slice(&mac_a);
    let snn = serving_network.as_str();
    let method = if sub.eap_aka_prime {
        let (ck_prime, ik_prime) = librecn_security::eap_aka::ck_ik_prime(&v.ck, &v.ik, snn, &sqn_xor_ak);
        Method::EapAkaPrime { xres: v.res, ck_prime, ik_prime }
    } else {
        Method::FiveGAka {
            xres_star: kdf::res_star(&v.ck, &v.ik, snn, &rand, &v.res),
            kausf: kdf::kausf(&v.ck, &v.ik, snn, &sqn_xor_ak),
        }
    };
    Vector { rand, autn, method }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex<const N: usize>(s: &str) -> [u8; N] {
        std::array::from_fn(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap())
    }

    fn subscriber() -> Subscriber {
        Subscriber {
            k: hex("465b5ce8b199b49faa5f0a2ee238a6bc"),
            opc: hex("e8ed289deba952e4283b54e88e6183ca"),
            amf: [0x80, 0],
            sqn: 0,
            eap_aka_prime: false,
            provisioned: false,
        }
    }

    #[test]
    fn null_scheme_sucis_are_deconcealed() {
        let deconceal = |s: &str| deconceal(s, &HomeNetworkKeys::new());
        assert_eq!(deconceal("suci-0-999-70-0000-0-0-0000000001").unwrap().as_str(), "imsi-999700000000001");
        assert_eq!(deconceal("imsi-208930000000003").unwrap().as_str(), "imsi-208930000000003");
        assert_eq!(deconceal("suci-0-999-70-0000-1-1-abcdef"), Err(Outcome::UnsupportedProtectionScheme), "no key 1");
        assert_eq!(deconceal("suci-0-999-70-0000-2-1-abcdef"), Err(Outcome::UnsupportedProtectionScheme), "profile B");
        assert_eq!(deconceal("suci-1-999-70-0000-0-0-a@b"), Err(Outcome::UserNotFound), "NAI");
        assert_eq!(deconceal("suci-0-999-70"), Err(Outcome::UserNotFound));
        assert_eq!(deconceal(""), Err(Outcome::UserNotFound));
    }

    /// The home network key of TS 33.501 Annex C.4.3 (and of UERANSIM's sample UE).
    fn keys() -> HomeNetworkKeys {
        HomeNetworkKeys::from([(1, hex("c53c22208b61860b06c62e5406a7b330c2b577aa5558981510d128247d38bd1d"))])
    }

    #[test]
    fn profile_a_sucis_are_deciphered() {
        // TS 33.501 Annex C.4.3: plaintext block 00012080f6, the MSIN 001002086 in BCD.
        let output = "b2e92f836055a255837debf850b528997ce0201cb82adfe4be1f587d07d8457d cb02352410 cddd9e730ef3fa87";
        let suci = format!("suci-0-001-01-0000-1-1-{}", output.replace(' ', ""));
        assert_eq!(deconceal(&suci, &keys()).unwrap().as_str(), "imsi-00101001002086");
        // The UE's side: UERANSIM's sample subscriber, MSIN 0000000001.
        let public = ecies::public_key(&keys()[&1]);
        let mut concealed = ecies::conceal(&public, &[9; 32], &[0x00, 0x00, 0x00, 0x00, 0x10]);
        let suci = |output: &[u8]| {
            let hex: String = output.iter().map(|b| format!("{b:02x}")).collect();
            format!("suci-0-999-70-0000-1-1-{hex}")
        };
        assert_eq!(deconceal(&suci(&concealed), &keys()).unwrap().as_str(), "imsi-999700000000001");
        let other_key = suci(&concealed).replace("-1-1-", "-1-2-");
        assert_eq!(deconceal(&other_key, &keys()), Err(Outcome::UnsupportedProtectionScheme));
        concealed[ecies::PUBLIC_KEY] ^= 1;
        assert_eq!(deconceal(&suci(&concealed), &keys()), Err(Outcome::UserNotFound), "the tag does not match");
    }

    /// UERANSIM's sample subscriber; the expected values are UERANSIM's (see librecn-security).
    #[test]
    fn vectors_take_the_next_sqn_and_match_the_ue() {
        let mut sub = subscriber();
        let snn = ServingNetwork::new("5G:mnc070.mcc999.3gppnetwork.org").unwrap();
        let v = vector(&mut sub, &snn, hex("23553cbe9637a89d218ae64dae47bf35"));
        assert_eq!(sub.sqn, 0x20, "SEQ 1, IND 0");
        let ak = hex::<6>("95ea6c443143");
        let sqn_xor_ak: [u8; 6] = std::array::from_fn(|i| [0, 0, 0, 0, 0, 0x20][i] ^ ak[i]);
        assert_eq!(v.autn[..6], sqn_xor_ak);
        assert_eq!(v.autn[6..], hex::<10>("80000cf661619e1a630c"));
        let Method::FiveGAka { xres_star, kausf } = v.method else { panic!("5G AKA") };
        assert_eq!(xres_star, hex("494971b7c0300cf47df7412c1889aee1"));
        assert_eq!(kausf, hex("4f2a3c0449084e104f39d0fe2af0c40ebc59ffbbd1eae27f314324b192f4e841"));
        vector(&mut sub, &snn, [0; 16]);
        assert_eq!(sub.sqn, 0x40);
        sub.eap_aka_prime = true;
        let v = vector(&mut sub, &snn, hex("23553cbe9637a89d218ae64dae47bf35"));
        let Method::EapAkaPrime { xres, .. } = v.method else { panic!("the subscriber's method") };
        assert_eq!(xres.len(), 8, "Milenage's RES");
    }

    #[test]
    fn resynchronisation_takes_the_ue_sqn_when_mac_s_is_right() {
        let mut sub = subscriber();
        let rand = hex("23553cbe9637a89d218ae64dae47bf35");
        // UERANSIM's AUTS for SQN_MS 0x20: SQN_MS ⊕ AK* || MAC-S (AMF 0000).
        let ak_star = hex::<6>("42e43b09e7fc");
        let mut auts = [0u8; 14];
        for i in 0..6 {
            auts[i] = [0, 0, 0, 0, 0, 0x20][i] ^ ak_star[i];
        }
        auts[6..].copy_from_slice(&hex::<8>("5edc8f5b471fa16f"));
        let mut tampered = auts;
        tampered[13] ^= 1;
        assert!(!resynchronise(&mut sub, &rand, &tampered));
        assert_eq!(sub.sqn, 0);
        assert!(resynchronise(&mut sub, &rand, &auts));
        assert_eq!(sub.sqn, 0x20);
        let snn = ServingNetwork::new("5G:mnc070.mcc999.3gppnetwork.org").unwrap();
        vector(&mut sub, &snn, rand);
        assert_eq!(sub.sqn, 0x40, "the next vector is fresh for the UE");
    }
}
