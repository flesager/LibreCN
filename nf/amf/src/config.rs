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
//! ausf = "ausf"                 # the AUSF's EO name
//! smf = "smf"                   # the SMF's EO name
//! ciphering = ["NEA2", "NEA0"]  # NAS ciphering by preference
//! ```
//!
//! Every field is optional; the defaults are test values matching UERANSIM's
//! sample gNB configuration (PLMN 999/70, SST 1). NAS integrity is always
//! NIA2, the only integrity algorithm implemented.

use std::net::SocketAddr;

use librecn_nas::security::{NEA0, NEA2};
pub use librecn_nas::{Plmn, Snssai};
use librecn_sbi::ServingNetwork;
use serde::Deserialize;

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
    ausf: String,
    smf: String,
    ciphering: Vec<String>,
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
            ausf: "ausf".into(),
            smf: "smf".into(),
            ciphering: vec!["NEA2".into(), "NEA0".into()],
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
    /// The AUSF's EO name.
    pub ausf: String,
    /// The SMF's EO name.
    pub smf: String,
    /// NAS ciphering algorithm identities, by preference.
    pub ciphering: Vec<u8>,
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
        let ciphering = p
            .ciphering
            .iter()
            .map(|c| match c.as_str() {
                "NEA0" => Ok(NEA0),
                "NEA2" => Ok(NEA2),
                other => Err(format!("ciphering {other:?}: NEA0 or NEA2")),
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
            ausf: p.ausf,
            smf: p.smf,
            ciphering,
        })
    }

    /// `5G:mnc<mnc>.mcc<mcc>.3gppnetwork.org` (TS 24.501 §9.12.1).
    pub fn serving_network(&self) -> ServingNetwork {
        let (mcc, mnc) = (self.plmn.mcc(), self.plmn.mnc());
        ServingNetwork::format(format_args!("5G:mnc{mnc:0>3}.mcc{mcc}.3gppnetwork.org")).expect("32 characters")
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
        assert_eq!((d.ciphering.as_slice(), d.ausf.as_str()), (&[NEA2, NEA0][..], "ausf"));
        assert_eq!(d.serving_network().as_str(), "5G:mnc070.mcc999.3gppnetwork.org");
        assert!(AmfConfig::from_params("amf", &serde_json::json!({ "ciphering": ["NEA1"] })).is_err());
    }
}
