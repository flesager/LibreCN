//! SMF configuration, from the deployment's `params`.
//!
//! ```toml
//! [[nf]]
//! kind = "smf"
//! name = "smf"
//! [nf.params]
//! upf = "upf"                    # the UPF's EO name
//! dnn = "internet"               # the data network served
//! pool = "10.45.0.0/16"          # UE addresses; the first one is the UPF's (N6)
//! slices = [{ sst = 1 }]
//! five_qi = 9                    # default QoS flow
//! ambr_mbps = { uplink = 1000, downlink = 1000 }   # session AMBR
//! ```
//!
//! Every field is optional; the defaults are these values. Without
//! subscription data yet, they apply to every UE.

use std::net::Ipv4Addr;

use librecn_nas::Snssai;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SliceParams {
    sst: u8,
    sd: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Ambr {
    uplink: u16,
    downlink: u16,
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Params {
    upf: String,
    dnn: String,
    pool: String,
    slices: Vec<SliceParams>,
    five_qi: u8,
    ambr_mbps: Ambr,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            upf: "upf".into(),
            dnn: "internet".into(),
            pool: "10.45.0.0/16".into(),
            slices: vec![SliceParams { sst: 1, sd: None }],
            five_qi: 9,
            ambr_mbps: Ambr { uplink: 1000, downlink: 1000 },
        }
    }
}

/// What the SMF serves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmfConfig {
    pub upf: String,
    pub dnn: String,
    /// Network and prefix length.
    pub pool: (Ipv4Addr, u8),
    pub slices: Vec<Snssai>,
    pub five_qi: u8,
    /// Uplink, downlink.
    pub ambr_mbps: (u16, u16),
}

impl SmfConfig {
    /// From deployment parameters (`null` or `{}`: all defaults).
    pub fn from_params(params: &serde_json::Value) -> Result<Self, String> {
        let p: Params = if params.is_null() {
            Params::default()
        } else {
            serde_json::from_value(params.clone()).map_err(|e| format!("invalid SMF parameters: {e}"))?
        };
        let bad_pool = || format!("pool {:?}: expected a.b.c.d/n, n from 8 to 30", p.pool);
        let (net, len) = p.pool.split_once('/').ok_or_else(bad_pool)?;
        let (net, len): (Ipv4Addr, u8) = (net.parse().map_err(|_| bad_pool())?, len.parse().map_err(|_| bad_pool())?);
        if !(8..=30).contains(&len) || u32::from(net) & (u32::MAX >> len) != 0 {
            return Err(bad_pool());
        }
        let slices = p
            .slices
            .iter()
            .map(|s| {
                let sd = match &s.sd {
                    None => None,
                    Some(h) => Some(sd(h).ok_or_else(|| format!("sd {h:?}: expected 6 hexadecimal digits"))?),
                };
                Ok(Snssai { sst: s.sst, sd })
            })
            .collect::<Result<_, String>>()?;
        if p.dnn.is_empty() || p.dnn.split('.').any(|l| l.is_empty() || l.len() > 63) {
            return Err(format!("dnn {:?}: dotted labels of 1 to 63 characters", p.dnn));
        }
        Ok(SmfConfig {
            upf: p.upf,
            dnn: p.dnn,
            pool: (net, len),
            slices,
            five_qi: p.five_qi,
            ambr_mbps: (p.ambr_mbps.uplink, p.ambr_mbps.downlink),
        })
    }
}

fn sd(hex: &str) -> Option<[u8; 3]> {
    let byte = |i: usize| hex.get(i..i + 2).and_then(|b| u8::from_str_radix(b, 16).ok());
    (hex.len() == 6).then(|| Some([byte(0)?, byte(2)?, byte(4)?])).flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_checks() {
        let c = SmfConfig::from_params(&serde_json::Value::Null).unwrap();
        assert_eq!((c.pool, c.dnn.as_str(), c.five_qi), ((Ipv4Addr::new(10, 45, 0, 0), 16), "internet", 9));
        let c = SmfConfig::from_params(&serde_json::json!({ "slices": [{ "sst": 1, "sd": "010203" }] })).unwrap();
        assert_eq!(c.slices, [Snssai { sst: 1, sd: Some([1, 2, 3]) }]);
        for bad in [
            serde_json::json!({ "pool": "10.45.0.1/16" }),
            serde_json::json!({ "pool": "10.45.0.0/31" }),
            serde_json::json!({ "pool": "10.45.0.0" }),
            serde_json::json!({ "dnn": "a..b" }),
            serde_json::json!({ "slices": [{ "sst": 1, "sd": "01" }] }),
            serde_json::json!({ "upff": "x" }),
        ] {
            assert!(SmfConfig::from_params(&bad).is_err(), "{bad}");
        }
    }
}
