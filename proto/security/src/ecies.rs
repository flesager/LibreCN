//! SUCI concealment with ECIES Profile A (TS 33.501 Annex C.3.4.1): X25519,
//! the ANSI X9.63 KDF on SHA-256, AES-128-CTR and a 64-bit HMAC-SHA-256 tag.
//!
//! The scheme output is the UE's ephemeral public key (32 bytes), the
//! ciphered MSIN (its BCD digits), then the tag (8 bytes). The home network
//! (the UDM's SIDF) holds the private key.

use aes::Aes128;
use aes::cipher::{KeyIvInit, StreamCipher};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};

/// Ephemeral public key length.
pub const PUBLIC_KEY: usize = 32;
/// MAC tag length.
pub const TAG: usize = 8;

/// Why a scheme output was not de-concealed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// Shorter than a public key and a tag.
    Truncated,
    /// The tag does not match: another key, or a forged SUCI.
    Mac,
}

/// The public key of home network private key `private`, for the USIMs.
pub fn public_key(private: &[u8; 32]) -> [u8; 32] {
    PublicKey::from(&StaticSecret::from(*private)).to_bytes()
}

/// K_enc, ICB and K_mac from the shared secret and the ephemeral public key.
fn keys(shared: &[u8; 32], ephemeral: &[u8; 32]) -> ([u8; 16], [u8; 16], [u8; 32]) {
    let block = |counter: u32| {
        Sha256::new().chain_update(shared).chain_update(counter.to_be_bytes()).chain_update(ephemeral).finalize()
    };
    let (a, b) = (block(1), block(2));
    (a[..16].try_into().unwrap(), a[16..].try_into().unwrap(), b.into())
}

fn tag(kmac: &[u8; 32], ciphertext: &[u8]) -> [u8; TAG] {
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(kmac).expect("any key length");
    mac.update(ciphertext);
    mac.finalize().into_bytes()[..TAG].try_into().unwrap()
}

/// The plaintext (the MSIN in BCD) of `output`, in place in `output`: its
/// range there.
pub fn deconceal(private: &[u8; 32], output: &mut [u8]) -> Result<std::ops::Range<usize>, Error> {
    if output.len() < PUBLIC_KEY + TAG {
        return Err(Error::Truncated);
    }
    let ephemeral: [u8; 32] = output[..PUBLIC_KEY].try_into().unwrap();
    let shared = StaticSecret::from(*private).diffie_hellman(&PublicKey::from(ephemeral)).to_bytes();
    let (kenc, icb, kmac) = keys(&shared, &ephemeral);
    let end = output.len() - TAG;
    if tag(&kmac, &output[PUBLIC_KEY..end]) != output[end..] {
        return Err(Error::Mac);
    }
    ctr::Ctr128BE::<Aes128>::new(&kenc.into(), &icb.into()).apply_keystream(&mut output[PUBLIC_KEY..end]);
    Ok(PUBLIC_KEY..end)
}

/// The UE's side, for tests and tools: the scheme output of `plaintext`
/// for home network public key `public`, with ephemeral private key
/// `ephemeral` (random on a real UE).
pub fn conceal(public: &[u8; 32], ephemeral: &[u8; 32], plaintext: &[u8]) -> Vec<u8> {
    let secret = StaticSecret::from(*ephemeral);
    let ephemeral_public = PublicKey::from(&secret).to_bytes();
    let shared = secret.diffie_hellman(&PublicKey::from(*public)).to_bytes();
    let (kenc, icb, kmac) = keys(&shared, &ephemeral_public);
    let mut ciphertext = plaintext.to_vec();
    ctr::Ctr128BE::<Aes128>::new(&kenc.into(), &icb.into()).apply_keystream(&mut ciphertext);
    let tag = tag(&kmac, &ciphertext);
    [&ephemeral_public[..], &ciphertext, &tag].concat()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex<const N: usize>(s: &str) -> [u8; N] {
        let v: Vec<u8> = (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect();
        v.try_into().unwrap()
    }

    /// TS 33.501 Annex C.4.3: the home network key pair of Profile A, the
    /// one UERANSIM's sample UE and the free5GC and Open5GS samples use.
    const PRIVATE: &str = "c53c22208b61860b06c62e5406a7b330c2b577aa5558981510d128247d38bd1d";
    const PUBLIC: &str = "5a8d38864820197c3394b92613b20b91633cbd897119273bf8e4a6f4eec0a650";

    #[test]
    fn the_annex_key_pair_matches() {
        assert_eq!(public_key(&hex(PRIVATE)), hex::<32>(PUBLIC));
    }

    /// TS 33.501 Annex C.4.3, Profile A: the scheme output of the example.
    #[test]
    fn the_annex_example_is_deconcealed() {
        let mut output = [
            &hex::<32>("b2e92f836055a255837debf850b528997ce0201cb82adfe4be1f587d07d8457d")[..],
            &hex::<5>("cb02352410"),
            &hex::<8>("cddd9e730ef3fa87"),
        ]
        .concat();
        let range = deconceal(&hex(PRIVATE), &mut output).unwrap();
        assert_eq!(output[range], hex::<5>("00012080f6"), "the plaintext block of the example");
    }

    #[test]
    fn a_concealed_msin_comes_back_and_a_changed_one_does_not() {
        let msin = [0x00, 0x00, 0x00, 0x00, 0x10];
        let mut output = conceal(&hex(PUBLIC), &[7; 32], &msin);
        assert_eq!(output.len(), PUBLIC_KEY + msin.len() + TAG);
        assert_ne!(output[PUBLIC_KEY..PUBLIC_KEY + 5], msin, "ciphered");
        let mut forged = output.clone();
        forged[PUBLIC_KEY] ^= 1;
        let range = deconceal(&hex(PRIVATE), &mut output).unwrap();
        assert_eq!(output[range], msin);
        assert_eq!(deconceal(&hex(PRIVATE), &mut forged), Err(Error::Mac));
        assert_eq!(deconceal(&[1; 32], &mut conceal(&hex(PUBLIC), &[7; 32], &msin)), Err(Error::Mac), "another key");
        assert_eq!(deconceal(&hex(PRIVATE), &mut [0; 39]), Err(Error::Truncated));
    }
}
