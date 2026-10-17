//! EAP-AKA' (RFC 9048) as 5G uses it (TS 33.501 §6.1.3.1, Annex F), for the
//! AUSF: the keys, the challenge (EAP-Request/AKA'-Challenge), the check of
//! the UE's answer, EAP-Success and EAP-Failure.
//!
//! ```text
//! CK' || IK' = KDF(CK || IK, 0x20, serving network name, SQN ⊕ AK)
//! MK = PRF'(IK' || CK', "EAP-AKA'" || identity): K_encr, K_aut, K_re, MSK, EMSK
//! K_AUSF = the first 256 bits of EMSK
//! AT_MAC = HMAC-SHA-256-128(K_aut, the EAP packet, AT_MAC zeroed)
//! ```
//!
//! The identity is the SUPI's digits (as UERANSIM's UE takes it).

use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::kdf;

/// EAP codes.
pub mod code {
    pub const REQUEST: u8 = 1;
    pub const RESPONSE: u8 = 2;
    pub const SUCCESS: u8 = 3;
    pub const FAILURE: u8 = 4;
}

/// The EAP method type of EAP-AKA'.
pub const TYPE: u8 = 50;

/// EAP-AKA' subtypes.
pub mod subtype {
    pub const CHALLENGE: u8 = 1;
    pub const AUTHENTICATION_REJECT: u8 = 2;
    pub const SYNCHRONIZATION_FAILURE: u8 = 4;
    pub const CLIENT_ERROR: u8 = 14;
}

const AT_RAND: u8 = 1;
const AT_AUTN: u8 = 2;
const AT_RES: u8 = 3;
const AT_AUTS: u8 = 4;
const AT_MAC: u8 = 11;
const AT_KDF_INPUT: u8 = 23;
const AT_KDF: u8 = 24;

/// Longest EAP packet built or read here.
pub const MAX: usize = 128;

/// CK' and IK' (RFC 9048 §3.3, TS 33.501 Annex A.3).
pub fn ck_ik_prime(ck: &[u8; 16], ik: &[u8; 16], serving_network: &str, sqn_xor_ak: &[u8; 6]) -> ([u8; 16], [u8; 16]) {
    let key = [&ck[..], &ik[..]].concat();
    let out = kdf::kdf(&key, 0x20, &[serving_network.as_bytes(), sqn_xor_ak]);
    (out[..16].try_into().unwrap(), out[16..].try_into().unwrap())
}

/// PRF' (RFC 9048 §3.4.1): HMAC-SHA-256 chained, into `out`.
fn prf_prime(key: &[u8], s: &[u8], out: &mut [u8]) {
    let mut previous: [u8; 32] = [0; 32];
    for (i, chunk) in out.chunks_mut(32).enumerate() {
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(key).expect("any key length");
        if i > 0 {
            mac.update(&previous);
        }
        mac.update(s);
        mac.update(&[i as u8 + 1]);
        previous = mac.finalize().into_bytes().into();
        chunk.copy_from_slice(&previous[..chunk.len()]);
    }
}

/// The keys the AUSF needs from MK.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Keys {
    /// Authenticates the EAP-AKA' packets.
    pub k_aut: [u8; 32],
    /// K_AUSF, the first half of EMSK.
    pub kausf: [u8; 32],
}

/// MK = PRF'(IK' || CK', "EAP-AKA'" || identity): K_encr (16 octets), K_aut
/// (32), K_re (32), MSK (64), EMSK (64).
pub fn keys(ck_prime: &[u8; 16], ik_prime: &[u8; 16], identity: &str) -> Keys {
    let key = [&ik_prime[..], &ck_prime[..]].concat();
    let s = [b"EAP-AKA'", identity.as_bytes()].concat();
    let mut mk = [0u8; 208];
    prf_prime(&key, &s, &mut mk);
    Keys { k_aut: mk[16..48].try_into().unwrap(), kausf: mk[144..176].try_into().unwrap() }
}

fn mac(k_aut: &[u8; 32], packet: &[u8]) -> [u8; 16] {
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(k_aut).expect("any key length");
    mac.update(packet);
    mac.finalize().into_bytes()[..16].try_into().unwrap()
}

/// EAP-Request/AKA'-Challenge with identifier `id` (RFC 9048 §3.1): AT_RAND,
/// AT_AUTN, AT_KDF (1), AT_KDF_INPUT (the serving network name), AT_MAC;
/// its length in `out`, `None` if the name makes it too long.
pub fn challenge(
    id: u8,
    rand: &[u8; 16],
    autn: &[u8; 16],
    serving_network: &str,
    k_aut: &[u8; 32],
    out: &mut [u8; MAX],
) -> Option<usize> {
    let name = serving_network.as_bytes();
    let padded = name.len().div_ceil(4) * 4;
    let len = 8 + 20 + 20 + 4 + 4 + padded + 20;
    if len > MAX {
        return None;
    }
    let mut at = 0;
    let mut put = |bytes: &[u8]| {
        out[at..at + bytes.len()].copy_from_slice(bytes);
        at += bytes.len();
    };
    put(&[code::REQUEST, id]);
    put(&(len as u16).to_be_bytes());
    put(&[TYPE, subtype::CHALLENGE, 0, 0, AT_RAND, 5, 0, 0]);
    put(rand);
    put(&[AT_AUTN, 5, 0, 0]);
    put(autn);
    put(&[AT_KDF, 1, 0, 1, AT_KDF_INPUT, (1 + padded / 4) as u8]);
    put(&(name.len() as u16).to_be_bytes());
    put(name);
    put(&[0; 3][..padded - name.len()]);
    put(&[AT_MAC, 5, 0, 0]);
    out[at..at + 16].fill(0);
    let mac = mac(k_aut, &out[..len]);
    out[at..at + 16].copy_from_slice(&mac);
    Some(len)
}

/// What the UE answered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response<'a> {
    /// AT_MAC checked: its RES.
    Challenge {
        res: &'a [u8],
    },
    AuthenticationReject,
    SynchronizationFailure {
        auts: [u8; 14],
    },
    ClientError,
}

/// Why an answer is refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// Not an EAP-AKA' Response to the request with this identifier.
    Malformed,
    /// AT_MAC missing or wrong.
    Mac,
}

/// The UE's EAP-AKA' Response to the request with identifier `id`.
pub fn response<'a>(packet: &'a [u8], id: u8, k_aut: &[u8; 32]) -> Result<Response<'a>, Error> {
    let len = packet.get(2..4).map_or(0, |l| usize::from(u16::from_be_bytes([l[0], l[1]])));
    if packet.len() < 8 || len != packet.len() || packet[..2] != [code::RESPONSE, id] || packet[4] != TYPE {
        return Err(Error::Malformed);
    }
    let (mut res, mut auts, mut at_mac) = (None, None, None);
    let mut at = 8;
    while at < packet.len() {
        let size = usize::from(*packet.get(at + 1).ok_or(Error::Malformed)?) * 4;
        let attribute = packet.get(at..at + size).filter(|_| size >= 4).ok_or(Error::Malformed)?;
        match attribute[0] {
            AT_RES => {
                let bytes = usize::from(u16::from_be_bytes([attribute[2], attribute[3]])).div_ceil(8);
                res = Some(attribute.get(4..4 + bytes).ok_or(Error::Malformed)?);
            }
            AT_AUTS => auts = attribute.get(2..16).and_then(|a| a.try_into().ok()),
            AT_MAC if size == 20 => at_mac = Some(at + 4),
            _ => {}
        }
        at += size;
    }
    match packet[5] {
        subtype::CHALLENGE => {
            let at_mac = at_mac.ok_or(Error::Mac)?;
            let mut zeroed = [0u8; MAX];
            let zeroed = zeroed.get_mut(..packet.len()).ok_or(Error::Malformed)?;
            zeroed.copy_from_slice(packet);
            zeroed[at_mac..at_mac + 16].fill(0);
            if mac(k_aut, zeroed) != packet[at_mac..at_mac + 16] {
                return Err(Error::Mac);
            }
            Ok(Response::Challenge { res: res.ok_or(Error::Malformed)? })
        }
        subtype::AUTHENTICATION_REJECT => Ok(Response::AuthenticationReject),
        subtype::SYNCHRONIZATION_FAILURE => {
            Ok(Response::SynchronizationFailure { auts: auts.ok_or(Error::Malformed)? })
        }
        subtype::CLIENT_ERROR => Ok(Response::ClientError),
        _ => Err(Error::Malformed),
    }
}

/// The UE's side, for tests and tools: EAP-Response/AKA'-Challenge with
/// `res`, as UERANSIM builds it (AT_RES, AT_MAC, AT_KDF).
pub fn answer(id: u8, res: &[u8], k_aut: &[u8; 32]) -> Vec<u8> {
    let mut p = vec![code::RESPONSE, id, 0, 0, TYPE, subtype::CHALLENGE, 0, 0];
    p.extend_from_slice(&[AT_RES, (1 + res.len().div_ceil(4)) as u8]);
    p.extend_from_slice(&((res.len() * 8) as u16).to_be_bytes());
    p.extend_from_slice(res);
    p.resize(p.len() + res.len().div_ceil(4) * 4 - res.len(), 0);
    p.extend_from_slice(&[AT_MAC, 5, 0, 0]);
    let at = p.len();
    p.extend_from_slice(&[0; 16]);
    p.extend_from_slice(&[AT_KDF, 1, 0, 1]);
    let len = p.len() as u16;
    p[2..4].copy_from_slice(&len.to_be_bytes());
    let m = mac(k_aut, &p);
    p[at..at + 16].copy_from_slice(&m);
    p
}

/// EAP-Success for the exchange `id`.
pub fn success(id: u8) -> [u8; 4] {
    [code::SUCCESS, id, 0, 4]
}

/// EAP-Failure for the exchange `id`.
pub fn failure(id: u8) -> [u8; 4] {
    [code::FAILURE, id, 0, 4]
}

#[cfg(test)]
mod tests {
    use super::*;

    const SNN: &str = "5G:mnc070.mcc999.3gppnetwork.org";

    #[test]
    fn a_challenge_is_answered_and_checked() {
        let (ck_p, ik_p) = ck_ik_prime(&[1; 16], &[2; 16], SNN, &[3; 6]);
        assert_ne!(ck_p, ik_p);
        let keys = keys(&ck_p, &ik_p, "999700000000001");
        assert_ne!(keys.k_aut[..], keys.kausf[..]);
        let mut out = [0u8; MAX];
        let n = challenge(7, &[4; 16], &[5; 16], SNN, &keys.k_aut, &mut out).unwrap();
        let request = &out[..n];
        assert_eq!(request[..8], [1, 7, 0, n as u8, 50, 1, 0, 0], "Request, AKA'-Challenge");
        assert_eq!(n, 8 + 20 + 20 + 4 + 4 + 32 + 20, "the name padded to 32 octets");
        let at_mac = n - 16;
        let mut zeroed = request.to_vec();
        zeroed[at_mac..].fill(0);
        assert_eq!(mac(&keys.k_aut, &zeroed), request[at_mac..], "AT_MAC over the packet, AT_MAC zeroed");
        assert!(challenge(7, &[4; 16], &[5; 16], &"x".repeat(80), &keys.k_aut, &mut out).is_none());

        let res = [6u8; 8];
        let answer = answer(7, &res, &keys.k_aut);
        assert_eq!(response(&answer, 7, &keys.k_aut), Ok(Response::Challenge { res: &res }));
        assert_eq!(response(&answer, 8, &keys.k_aut), Err(Error::Malformed), "another exchange");
        let mut forged = answer.clone();
        forged[12] ^= 1;
        assert_eq!(response(&forged, 7, &keys.k_aut), Err(Error::Mac));
        assert_eq!(response(&answer, 7, &[0; 32]), Err(Error::Mac), "another K_aut");
        assert_eq!(response(&answer[..10], 7, &keys.k_aut), Err(Error::Malformed));
        let reject = [code::RESPONSE, 7, 0, 8, TYPE, subtype::AUTHENTICATION_REJECT, 0, 0];
        assert_eq!(response(&reject, 7, &keys.k_aut), Ok(Response::AuthenticationReject));
        assert_eq!((success(7), failure(7)), ([3, 7, 0, 4], [4, 7, 0, 4]));
    }

    #[test]
    fn prf_prime_chains_its_blocks() {
        let mut long = [0u8; 64];
        prf_prime(b"key", b"seed", &mut long);
        let mut short = [0u8; 32];
        prf_prime(b"key", b"seed", &mut short);
        assert_eq!(long[..32], short, "T1 first");
        assert_ne!(long[..32], long[32..], "T2 differs");
    }
}
