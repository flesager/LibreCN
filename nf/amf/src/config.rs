//! AMF configuration, from the deployment's `params`.
//!
//! ```toml
//! [[nf]]
//! kind = "amf"
//! name = "amf-1"
//! [nf.params]
//! n2 = "127.0.0.1:38412"
//! plmn = { mcc = "999", mnc = "70" }
//! amf_region = 1
//! amf_set = 1
//! amf_pointer = 0
//! relative_capacity = 255
//! slices = [{ sst = 1 }, { sst = 1, sd = "010203" }]
//! max_ues = 1024
//! ```
//!
//! Every field is optional; the defaults are test values matching UERANSIM's
//! sample gNB configuration (PLMN 999/70, SST 1).

use std::net::SocketAddr;

use serde::Deserialize;

/// A PLMN, as the 3 BCD octets of 3GPP TS 38.413 §9.3.3.5.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Plmn(pub [u8; 3]);

impl Plmn {
    /// From MCC (3 digits) and MNC (2 or 3 digits).
    pub fn new(mcc: &str, mnc: &str) -> Result<Self, String> {
        let digits = |s: &str, n: &[usize]| -> Result<Vec<u8>, String> {
            if !n.contains(&s.len()) || !s.bytes().all(|b| b.is_ascii_digit()) {
                return Err(format!("{s:?}: expected {n:?} digits"));
            }
            Ok(s.bytes().map(|b| b - b'0').collect())
        };
        let (c, n) = (digits(mcc, &[3])?, digits(mnc, &[2, 3])?);
        let mnc3 = n.get(2).copied().unwrap_or(0xF);
        Ok(Plmn([c[1] << 4 | c[0], mnc3 << 4 | c[2], n[1] << 4 | n[0]]))
    }
}

impl std::fmt::Display for Plmn {
    /// `mcc/mnc`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [a, b, c] = self.0;
        write!(f, "{}{}{}/{}{}", a & 0xF, a >> 4, b & 0xF, c & 0xF, c >> 4)?;
        if b >> 4 != 0xF {
            write!(f, "{}", b >> 4)?;
        }
        Ok(())
    }
}

/// A network slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Snssai {
    /// Slice/service type.
    pub sst: u8,
    /// Slice differentiator.
    pub sd: Option<[u8; 3]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlmnParams {
    mcc: String,
    mnc: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SliceParams {
    sst: u8,
    sd: Option<String>,
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Params {
    n2: SocketAddr,
    plmn: PlmnParams,
    amf_region: u8,
    amf_set: u16,
    amf_pointer: u8,
    relative_capacity: u8,
    slices: Vec<SliceParams>,
    max_ues: usize,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            n2: SocketAddr::from(([127, 0, 0, 1], 38412)),
            plmn: PlmnParams { mcc: "999".into(), mnc: "70".into() },
            amf_region: 1,
            amf_set: 1,
            amf_pointer: 0,
            relative_capacity: 255,
            slices: vec![SliceParams { sst: 1, sd: None }],
            max_ues: 1024,
        }
    }
}

/// What the AMF announces in NG Setup, and where it listens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AmfConfig {
    /// AMF Name (the EO name).
    pub name: String,
    /// N2 address (SCTP).
    pub n2: SocketAddr,
    /// Served PLMN.
    pub plmn: Plmn,
    /// AMF Region ID (8 bits).
    pub amf_region: u8,
    /// AMF Set ID (10 bits).
    pub amf_set: u16,
    /// AMF Pointer (6 bits).
    pub amf_pointer: u8,
    /// Relative AMF Capacity.
    pub relative_capacity: u8,
    /// Slices supported in the PLMN.
    pub slices: Vec<Snssai>,
    /// UE contexts kept at most.
    pub max_ues: usize,
}

impl AmfConfig {
    /// From deployment parameters (`null` or `{}`: all defaults).
    pub fn from_params(name: &str, params: &serde_json::Value) -> Result<Self, String> {
        let p: Params = if params.is_null() {
            Params::default()
        } else {
            serde_json::from_value(params.clone()).map_err(|e| format!("invalid AMF parameters: {e}"))?
        };
        if p.amf_set >= 1 << 10 || p.amf_pointer >= 1 << 6 {
            return Err("amf_set is 10 bits and amf_pointer 6 bits".into());
        }
        let slices = p
            .slices
            .iter()
            .map(|s| {
                let sd = s.sd.as_deref().map(parse_sd).transpose()?;
                Ok(Snssai { sst: s.sst, sd })
            })
            .collect::<Result<_, String>>()?;
        Ok(AmfConfig {
            name: name.to_string(),
            n2: p.n2,
            plmn: Plmn::new(&p.plmn.mcc, &p.plmn.mnc)?,
            amf_region: p.amf_region,
            amf_set: p.amf_set,
            amf_pointer: p.amf_pointer,
            relative_capacity: p.relative_capacity,
            slices,
            max_ues: p.max_ues,
        })
    }
}

/// `"010203"` → `[1, 2, 3]`.
fn parse_sd(hex: &str) -> Result<[u8; 3], String> {
    let bad = || format!("sd {hex:?}: expected 6 hexadecimal digits");
    if hex.len() != 6 {
        return Err(bad());
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| bad());
    Ok([byte(0)?, byte(2)?, byte(4)?])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plmn_is_bcd_encoded() {
        assert_eq!(Plmn::new("208", "93").unwrap().0, [0x02, 0xf8, 0x39], "free5GC's test PLMN");
        assert_eq!(Plmn::new("999", "70").unwrap().0, [0x99, 0xf9, 0x07]);
        assert_eq!(Plmn::new("310", "410").unwrap().0, [0x13, 0x00, 0x14], "3-digit MNC");
        assert_eq!(Plmn::new("310", "410").unwrap().to_string(), "310/410");
        assert_eq!(Plmn::new("999", "70").unwrap().to_string(), "999/70");
        assert!(Plmn::new("99", "70").is_err() && Plmn::new("999", "7a").is_err());
    }

    #[test]
    fn defaults_are_the_test_values_and_params_override_them() {
        let d = AmfConfig::from_params("amf", &serde_json::Value::Null).unwrap();
        assert_eq!((d.plmn.to_string(), d.n2.port(), d.slices.len()), ("999/70".into(), 38412, 1));
        let params =
            serde_json::json!({ "plmn": { "mcc": "001", "mnc": "01" }, "slices": [{ "sst": 1, "sd": "010203" }] });
        let c = AmfConfig::from_params("amf", &params).unwrap();
        assert_eq!(c.plmn.to_string(), "001/01");
        assert_eq!(c.slices, [Snssai { sst: 1, sd: Some([1, 2, 3]) }]);
        assert!(AmfConfig::from_params("amf", &serde_json::json!({ "amf_set": 1024 })).is_err());
        assert!(AmfConfig::from_params("amf", &serde_json::json!({ "plmm": {} })).is_err(), "typos are refused");
    }
}
