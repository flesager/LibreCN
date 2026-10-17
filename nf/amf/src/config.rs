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
//! t3512_s = 3240                # periodic registration (54 min: sent as 1 h)
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
    t3512_s: u32,
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
            t3512_s: 3240,
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
    /// T3512, the periodic registration timer given to the UEs, as sent (a
    /// GPRS timer 3 value: the parameter rounded up to what it can say).
    pub t3512: u8,
}

impl AmfConfig {
    /// T3512 as the UEs run it.
    pub fn t3512(&self) -> std::time::Duration {
        let seconds = librecn_nas::ie::gprs_timer_3_seconds(self.t3512).unwrap_or(u32::MAX);
        std::time::Duration::from_secs(u64::from(seconds))
    }

    /// The mobile reachable timer: T3512 and 4 minutes (TS 24.501 §5.3.7);
    /// an idle UE silent for longer is implicitly deregistered.
    pub fn mobile_reachable(&self) -> std::time::Duration {
        self.t3512() + std::time::Duration::from_secs(4 * 60)
    }
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
        if p.t3512_s < 2 {
            return Err("t3512_s: 2 s at least".into());
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
            t3512: librecn_nas::ie::gprs_timer_3(p.t3512_s),
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
    fn t3512_is_sent_as_it_can_be_said_and_the_mobile_reachable_timer_follows() {
        let d = AmfConfig::from_params("amf", &serde_json::Value::Null).unwrap();
        assert_eq!((d.t3512().as_secs(), d.mobile_reachable().as_secs()), (3600, 3840), "54 min: sent as 1 h");
        let short = AmfConfig::from_params("amf", &serde_json::json!({ "t3512_s": 30 })).unwrap();
        assert_eq!((short.t3512, short.mobile_reachable().as_secs()), (0b011 << 5 | 15, 270));
        assert!(AmfConfig::from_params("amf", &serde_json::json!({ "t3512_s": 1 })).is_err());
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
        assert_eq!((d.ciphering.as_slice(), d.ausf.as_str()), (&[NEA2, NEA0][..], "ausf"));
        assert_eq!(d.serving_network().as_str(), "5G:mnc070.mcc999.3gppnetwork.org");
        assert!(AmfConfig::from_params("amf", &serde_json::json!({ "ciphering": ["NEA1"] })).is_err());
    }
}
