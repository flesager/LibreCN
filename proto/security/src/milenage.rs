//! Milenage, the authentication and key agreement functions f1, f1*, f2,
//! f3, f4, f5 and f5* (TS 35.206), from K and OPc.

use aes::Aes128;
use aes::cipher::{BlockEncrypt, KeyInit};

/// Milenage for one subscriber.
pub struct Milenage {
    cipher: Aes128,
    opc: [u8; 16],
}

/// Outputs of f2–f5 for one RAND.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vector {
    /// f2.
    pub res: [u8; 8],
    /// f3.
    pub ck: [u8; 16],
    /// f4.
    pub ik: [u8; 16],
    /// f5.
    pub ak: [u8; 6],
}

impl Milenage {
    pub fn new(k: &[u8; 16], opc: &[u8; 16]) -> Self {
        Milenage { cipher: Aes128::new(k.into()), opc: *opc }
    }

    fn encrypt(&self, mut block: [u8; 16]) -> [u8; 16] {
        let b = (&mut block).into();
        self.cipher.encrypt_block(b);
        block
    }

    fn xor(a: [u8; 16], b: &[u8; 16]) -> [u8; 16] {
        std::array::from_fn(|i| a[i] ^ b[i])
    }

    /// E_K(RAND ⊕ OPc).
    fn temp(&self, rand: &[u8; 16]) -> [u8; 16] {
        self.encrypt(Self::xor(*rand, &self.opc))
    }

    /// OUT_n = E_K(rot(x ⊕ OPc, r) ⊕ c) ⊕ OPc, with `r` in octets and `c`
    /// the last octet of the constant.
    fn out(&self, x: [u8; 16], r: usize, c: u8) -> [u8; 16] {
        let x = Self::xor(x, &self.opc);
        let mut block: [u8; 16] = std::array::from_fn(|i| x[(i + r) % 16]);
        block[15] ^= c;
        Self::xor(self.encrypt(block), &self.opc)
    }

    /// f1 and f1*: MAC-A (network authentication) and MAC-S (resynchronisation).
    pub fn f1(&self, rand: &[u8; 16], sqn: &[u8; 6], amf: &[u8; 2]) -> ([u8; 8], [u8; 8]) {
        let temp = self.temp(rand);
        let mut in1 = [0u8; 16];
        for half in in1.chunks_mut(8) {
            half[..6].copy_from_slice(sqn);
            half[6..].copy_from_slice(amf);
        }
        // OUT1 = E_K(TEMP ⊕ rot(IN1 ⊕ OPc, r1) ⊕ c1) ⊕ OPc: TEMP enters after the rotation.
        let x = Self::xor(in1, &self.opc);
        let block: [u8; 16] = std::array::from_fn(|i| x[(i + 8) % 16] ^ temp[i]);
        let out = Self::xor(self.encrypt(block), &self.opc);
        (out[..8].try_into().unwrap(), out[8..].try_into().unwrap())
    }

    /// f2, f3, f4 and f5.
    pub fn f2345(&self, rand: &[u8; 16]) -> Vector {
        let temp = self.temp(rand);
        let out2 = self.out(temp, 0, 1);
        Vector {
            res: out2[8..].try_into().unwrap(),
            ck: self.out(temp, 4, 2),
            ik: self.out(temp, 8, 4),
            ak: out2[..6].try_into().unwrap(),
        }
    }

    /// f5*: the anonymity key of resynchronisation.
    pub fn f5_star(&self, rand: &[u8; 16]) -> [u8; 6] {
        self.out(self.temp(rand), 12, 8)[..6].try_into().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex;

    #[test]
    fn matches_ts_35208_test_set_1() {
        let m = Milenage::new(&hex("465b5ce8b199b49faa5f0a2ee238a6bc"), &hex("cd63cb71954a9f4e48a5994e37a02baf"));
        let rand = hex("23553cbe9637a89d218ae64dae47bf35");
        assert_eq!(m.f1(&rand, &hex("ff9bb4d0b607"), &hex("b9b9")), (hex("4a9ffac354dfafb3"), hex("01cfaf9ec4e871e9")));
        let v = m.f2345(&rand);
        assert_eq!(v.res, hex("a54211d5e3ba50bf"));
        assert_eq!(v.ck, hex("b40ba9a3c58b2a05bbf0d987b21bf8cb"));
        assert_eq!(v.ik, hex("f769bcd751044604127672711c6d3441"));
        assert_eq!(v.ak, hex("aa689c648370"));
        assert_eq!(m.f5_star(&rand), hex("451e8beca43b"));
    }

    /// UERANSIM's sample subscriber, values from UERANSIM's implementation.
    #[test]
    fn matches_ueransim() {
        let m = Milenage::new(&hex("465b5ce8b199b49faa5f0a2ee238a6bc"), &hex("e8ed289deba952e4283b54e88e6183ca"));
        let rand = hex("23553cbe9637a89d218ae64dae47bf35");
        let sqn = hex("000000000020");
        assert_eq!(m.f1(&rand, &sqn, &hex("8000")), (hex("0cf661619e1a630c"), hex("b3694959a8a0bd12")));
        assert_eq!(m.f1(&rand, &sqn, &[0, 0]).1, hex("5edc8f5b471fa16f"), "MAC-S of resynchronisation");
        let v = m.f2345(&rand);
        assert_eq!(
            v,
            Vector {
                res: hex("7930e456c8677f0d"),
                ck: hex("e3e2b91467b15119fbd8ae0710a0688e"),
                ik: hex("e25a0d267e9f1dcd83e56256797696cb"),
                ak: hex("95ea6c443143"),
            }
        );
        assert_eq!(m.f5_star(&rand), hex("42e43b09e7fc"));
    }
}
