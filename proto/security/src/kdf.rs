//! The key derivation function of TS 33.220 Annex B.2 (HMAC-SHA-256 over
//! `FC || P0 || L0 || P1 || L1 ...`) and the 5G keys of TS 33.501 Annex A.
//! The key chain is checked against UERANSIM; K_gNB, which UERANSIM does
//! not derive, follows the same KDF with the parameters of A.9.

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

/// Algorithm type distinguishers (TS 33.501 Annex A.8).
const N_NAS_ENC_ALG: u8 = 0x01;
const N_NAS_INT_ALG: u8 = 0x02;

/// KDF(key, FC, P0..Pn).
pub fn kdf(key: &[u8], fc: u8, params: &[&[u8]]) -> [u8; 32] {
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(key).expect("HMAC takes any key length");
    mac.update(&[fc]);
    for p in params {
        mac.update(p);
        mac.update(&(p.len() as u16).to_be_bytes());
    }
    mac.finalize().into_bytes().into()
}

fn ck_ik(ck: &[u8; 16], ik: &[u8; 16]) -> [u8; 32] {
    let mut key = [0u8; 32];
    key[..16].copy_from_slice(ck);
    key[16..].copy_from_slice(ik);
    key
}

fn low_128(bytes: [u8; 32]) -> [u8; 16] {
    bytes[16..].try_into().unwrap()
}

/// K_AUSF for 5G AKA (A.2), from `SQN ⊕ AK` as sent in AUTN.
pub fn kausf(ck: &[u8; 16], ik: &[u8; 16], serving_network: &str, sqn_xor_ak: &[u8; 6]) -> [u8; 32] {
    kdf(&ck_ik(ck, ik), 0x6a, &[serving_network.as_bytes(), sqn_xor_ak])
}

/// RES* (UE) or XRES* (home network) (A.4).
pub fn res_star(ck: &[u8; 16], ik: &[u8; 16], serving_network: &str, rand: &[u8; 16], res: &[u8]) -> [u8; 16] {
    low_128(kdf(&ck_ik(ck, ik), 0x6b, &[serving_network.as_bytes(), rand, res]))
}

/// HRES* or HXRES* (A.5): the 128 least significant bits of SHA-256(RAND || RES*).
pub fn hres_star(rand: &[u8; 16], res_star: &[u8; 16]) -> [u8; 16] {
    low_128(Sha256::new().chain_update(rand).chain_update(res_star).finalize().into())
}

/// K_SEAF (A.6).
pub fn kseaf(kausf: &[u8; 32], serving_network: &str) -> [u8; 32] {
    kdf(kausf, 0x6c, &[serving_network.as_bytes()])
}

/// K_AMF (A.7), for a SUPI of type IMSI given as its digits.
pub fn kamf(kseaf: &[u8; 32], imsi: &str, abba: &[u8]) -> [u8; 32] {
    kdf(kseaf, 0x6d, &[imsi.as_bytes(), abba])
}

/// K_gNB (A.9), from the uplink NAS COUNT of the message that triggered it,
/// for 3GPP access.
pub fn kgnb(kamf: &[u8; 32], ul_count: u32) -> [u8; 32] {
    const ACCESS_3GPP: u8 = 0x01;
    kdf(kamf, 0x6e, &[&ul_count.to_be_bytes(), &[ACCESS_3GPP]])
}

/// K_NASenc and K_NASint (A.8) for algorithm identities `ciphering` and
/// `integrity` (2 for NEA2 / NIA2).
pub fn nas_keys(kamf: &[u8; 32], ciphering: u8, integrity: u8) -> ([u8; 16], [u8; 16]) {
    (
        low_128(kdf(kamf, 0x69, &[&[N_NAS_ENC_ALG], &[ciphering]])),
        low_128(kdf(kamf, 0x69, &[&[N_NAS_INT_ALG], &[integrity]])),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex;
    use crate::milenage::Milenage;

    /// The whole 5G AKA key chain for UERANSIM's sample subscriber; every value
    /// from UERANSIM's implementation.
    #[test]
    fn the_5g_aka_key_chain_matches_ueransim() {
        let snn = "5G:mnc070.mcc999.3gppnetwork.org";
        let m = Milenage::new(&hex("465b5ce8b199b49faa5f0a2ee238a6bc"), &hex("e8ed289deba952e4283b54e88e6183ca"));
        let rand = hex("23553cbe9637a89d218ae64dae47bf35");
        let v = m.f2345(&rand);
        let sqn_xor_ak: [u8; 6] = std::array::from_fn(|i| hex::<6>("000000000020")[i] ^ v.ak[i]);

        let kausf = kausf(&v.ck, &v.ik, snn, &sqn_xor_ak);
        assert_eq!(kausf, hex("4f2a3c0449084e104f39d0fe2af0c40ebc59ffbbd1eae27f314324b192f4e841"));
        assert_eq!(res_star(&v.ck, &v.ik, snn, &rand, &v.res), hex("494971b7c0300cf47df7412c1889aee1"));
        let kseaf = kseaf(&kausf, snn);
        assert_eq!(kseaf, hex("2532d4c43e0ad4a3e3b2c125fe145a64b092e8f123ff1c52db5f20e2e3205ef6"));
        let kamf = kamf(&kseaf, "999700000000001", &[0, 0]);
        assert_eq!(kamf, hex("3b342433dd37dc9ad17e14fd47f1df93417befff787891fa710c7b1c37801052"));
        assert_eq!(
            nas_keys(&kamf, 2, 2),
            (hex("1feed6bfc79973d28c52be1028e56b03"), hex("68c1b91f3eb481f9f4694de8d740c3df"))
        );
    }

    #[test]
    fn hres_star_is_the_low_half_of_sha256() {
        let (rand, res) = ([1u8; 16], [2u8; 16]);
        let digest: [u8; 32] = Sha256::digest([rand, res].concat()).into();
        assert_eq!(hres_star(&rand, &res), digest[16..]);
    }
}
