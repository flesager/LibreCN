//! 3GPP security algorithms for the core network, without allocation:
//!
//! - [`milenage`]: the authentication functions f1–f5* (TS 35.206), used by
//!   the UDM to compute authentication vectors;
//! - [`kdf`]: the key derivation function of TS 33.220 Annex B and the 5G
//!   keys built on it (TS 33.501 Annex A): K_AUSF, RES*, HRES*, K_SEAF,
//!   K_AMF, NAS keys;
//! - [`nia2`] and [`nea2`]: 128-NIA2 (AES-CMAC) and 128-NEA2 (AES-CTR),
//!   TS 33.501 Annex D, the same algorithms as 128-EIA2/EEA2 (TS 33.401);
//! - [`random`]: random values (RAND, 5G-TMSI) from the kernel.
//!
//! NIA1/NEA1 (SNOW 3G) and NIA3/NEA3 (ZUC) are not implemented: every UE
//! must support NIA2 and NEA2 (TS 33.501 §5.11.1).

pub mod kdf;
pub mod milenage;

use aes::Aes128;
use aes::cipher::{KeyIvInit, StreamCipher};
use cmac::{Cmac, Mac};

/// Transfer direction of a protected message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Uplink = 0,
    Downlink = 1,
}

/// COUNT, BEARER and DIRECTION as the first 64 bits of the NIA2 input and
/// of the NEA2 counter block (TS 33.401 Annex B.1.3, B.2.3).
fn iv(count: u32, bearer: u8, direction: Direction) -> [u8; 8] {
    let c = count.to_be_bytes();
    [c[0], c[1], c[2], c[3], (bearer & 0x1f) << 3 | (direction as u8) << 2, 0, 0, 0]
}

/// 128-NIA2: the 32-bit MAC of `message` (whole octets).
pub fn nia2(key: &[u8; 16], count: u32, bearer: u8, direction: Direction, message: &[u8]) -> [u8; 4] {
    let mut mac = <Cmac<Aes128> as Mac>::new(key.into());
    mac.update(&iv(count, bearer, direction));
    mac.update(message);
    let tag = mac.finalize().into_bytes();
    [tag[0], tag[1], tag[2], tag[3]]
}

/// 128-NEA2: ciphers or deciphers `data` in place.
pub fn nea2(key: &[u8; 16], count: u32, bearer: u8, direction: Direction, data: &mut [u8]) {
    let mut block = [0u8; 16];
    block[..8].copy_from_slice(&iv(count, bearer, direction));
    ctr::Ctr128BE::<Aes128>::new(key.into(), &block.into()).apply_keystream(data);
}

/// `N` random octets from the kernel's CSPRNG (`getrandom(2)`).
pub fn random<const N: usize>() -> std::io::Result<[u8; N]> {
    let mut out = [0u8; N];
    let n = rustix::rand::getrandom(&mut out[..], rustix::rand::GetRandomFlags::empty())?;
    if n != N {
        return Err(std::io::Error::other("short getrandom"));
    }
    Ok(out)
}

#[cfg(test)]
pub(crate) fn hex<const N: usize>(s: &str) -> [u8; N] {
    let v: Vec<u8> = (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect();
    v.try_into().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_values_differ() {
        assert_ne!(random::<16>().unwrap(), random::<16>().unwrap());
    }

    #[test]
    fn nia2_matches_ts_33401_test_set_1() {
        let key = hex("d3c5d592327fb11c4035c6680af8c6d1");
        assert_eq!(nia2(&key, 0x398a59b4, 0x1a, Direction::Downlink, &hex::<8>("484583d5afe082ae")), hex("b93787e6"));
    }

    // Reference values from UERANSIM's implementation (the NAS keys of
    // `kdf::tests`, a Security Mode Command with its sequence number).
    #[test]
    fn nia2_and_nea2_match_ueransim() {
        let message = hex::<15>("007e005d020004f0f0f0f0e1360100");
        let int = hex("68c1b91f3eb481f9f4694de8d740c3df");
        assert_eq!(nia2(&int, 0x102, 1, Direction::Downlink, &message), hex("33f928da"));
        let enc = hex("1feed6bfc79973d28c52be1028e56b03");
        let mut data = message;
        nea2(&enc, 0x102, 1, Direction::Uplink, &mut data);
        assert_eq!(data, hex("d358cef4b8007d35fb93a39ae091f0"));
        nea2(&enc, 0x102, 1, Direction::Uplink, &mut data);
        assert_eq!(data, message, "deciphering is ciphering");
    }
}
