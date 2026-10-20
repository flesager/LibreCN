//! SMF configuration, from the deployment's `params`.
//!
//! ```toml
//! [[nf]]
//! kind = "smf"
//! name = "smf"
//! [nf.params]
//! upf = "upf"                    # the UPF's EO name
//! # The data networks served, the first by default; each with its UE
//! # addresses (its first one is the UPF's N6 side; pools do not overlap),
//! # and what a UE asking for them (ePCO) gets: DNS servers (1 to 4) and
//! # the IPv4 link MTU (room for the tunnel's 44 bytes on a 1500 N3).
//! dnns = [{ dnn = "internet", pool = "10.45.0.0/16", dns = ["8.8.8.8", "1.1.1.1"], mtu = 1400 }]
//! slices = [{ sst = 1 }]
//! pcf = "pcf"                    # the PCF's EO name: each session's QoS (SM policy)
//! ```
//!
//! Every field is optional; the defaults are these values. Each session's
//! QoS (session AMBR, QoS flows) is the PCF's (its SM policy).

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
struct DnnParams {
    dnn: String,
    pool: String,
    #[serde(default = "public_dns")]
    dns: Vec<Ipv4Addr>,
    #[serde(default = "link_mtu")]
    mtu: u16,
}

fn public_dns() -> Vec<Ipv4Addr> {
    vec![Ipv4Addr::new(8, 8, 8, 8), Ipv4Addr::new(1, 1, 1, 1)]
}

fn link_mtu() -> u16 {
    1400
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Params {
    upf: String,
    dnns: Vec<DnnParams>,
    slices: Vec<SliceParams>,
    pcf: String,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            upf: "upf".into(),
            dnns: vec![DnnParams {
                dnn: "internet".into(),
                pool: "10.45.0.0/16".into(),
                dns: public_dns(),
                mtu: link_mtu(),
            }],
            slices: vec![SliceParams { sst: 1, sd: None }],
            pcf: "pcf".into(),
        }
    }
}

/// A data network served, its UE addresses, and what its UEs are told.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DnnConfig {
    pub dnn: String,
    /// Network and prefix length.
    pub pool: (Ipv4Addr, u8),
    /// DNS servers, and the IPv4 link MTU, for the UEs that ask (ePCO).
    pub dns: Vec<[u8; 4]>,
    pub mtu: u16,
}

/// What the SMF serves.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmfConfig {
    pub upf: String,
    /// The first is the default DNN.
    pub dnns: Vec<DnnConfig>,
    pub slices: Vec<Snssai>,
    /// The PCF's EO name.
    pub pcf: String,
}

impl SmfConfig {
    /// From deployment parameters (`null` or `{}`: all defaults).
    pub fn from_params(params: &serde_json::Value) -> Result<Self, String> {
        let p: Params = if params.is_null() {
            Params::default()
        } else {
            serde_json::from_value(params.clone()).map_err(|e| format!("invalid SMF parameters: {e}"))?
        };
        if p.dnns.is_empty() {
            return Err("dnns: one at least".into());
        }
        let dnns = p.dnns.iter().map(dnn).collect::<Result<Vec<_>, String>>()?;
        for (i, a) in dnns.iter().enumerate() {
            for b in &dnns[i + 1..] {
                if a.dnn.eq_ignore_ascii_case(&b.dnn) {
                    return Err(format!("dnn {:?}: listed twice", a.dnn));
                }
                let mask = u32::MAX << (32 - a.pool.1.min(b.pool.1));
                if u32::from(a.pool.0) & mask == u32::from(b.pool.0) & mask {
                    return Err(format!("pools of {:?} and {:?} overlap", a.dnn, b.dnn));
                }
            }
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
        Ok(SmfConfig { upf: p.upf, dnns, slices, pcf: p.pcf })
    }
}

fn dnn(d: &DnnParams) -> Result<DnnConfig, String> {
    if d.dnn.is_empty() || d.dnn.split('.').any(|l| l.is_empty() || l.len() > 63) {
        return Err(format!("dnn {:?}: dotted labels of 1 to 63 characters", d.dnn));
    }
    let bad_pool = || format!("pool {:?} of {:?}: expected a.b.c.d/n, n from 8 to 30", d.pool, d.dnn);
    let (net, len) = d.pool.split_once('/').ok_or_else(bad_pool)?;
    let (net, len): (Ipv4Addr, u8) = (net.parse().map_err(|_| bad_pool())?, len.parse().map_err(|_| bad_pool())?);
    if !(8..=30).contains(&len) || u32::from(net) & (u32::MAX >> len) != 0 {
        return Err(bad_pool());
    }
    if d.dns.is_empty() || d.dns.len() > 4 {
        return Err(format!("dns of {:?}: 1 to 4 IPv4 addresses", d.dnn));
    }
    if !(576..=9000).contains(&d.mtu) {
        return Err(format!("mtu of {:?}: 576 to 9000", d.dnn));
    }
    Ok(DnnConfig { dnn: d.dnn.clone(), pool: (net, len), dns: d.dns.iter().map(|a| a.octets()).collect(), mtu: d.mtu })
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
        let internet = DnnConfig {
            dnn: "internet".into(),
            pool: (Ipv4Addr::new(10, 45, 0, 0), 16),
            dns: vec![[8, 8, 8, 8], [1, 1, 1, 1]],
            mtu: 1400,
        };
        assert_eq!((&c.dnns[..], c.pcf.as_str()), (&[internet][..], "pcf"));
        let two = serde_json::json!({ "dnns": [{ "dnn": "internet", "pool": "10.45.0.0/16" },
                                               { "dnn": "ims", "pool": "10.46.0.0/24" }] });
        assert_eq!(SmfConfig::from_params(&two).unwrap().dnns[1].pool, (Ipv4Addr::new(10, 46, 0, 0), 24));
        let own = serde_json::json!({ "dnns": [{ "dnn": "internet", "pool": "10.45.0.0/16", "dns": ["9.9.9.9"], "mtu": 1450 }] });
        let own = &SmfConfig::from_params(&own).unwrap().dnns[0];
        assert_eq!((&own.dns[..], own.mtu), (&[[9, 9, 9, 9]][..], 1450));
        let c = SmfConfig::from_params(&serde_json::json!({ "slices": [{ "sst": 1, "sd": "010203" }] })).unwrap();
        assert_eq!(c.slices, [Snssai { sst: 1, sd: Some([1, 2, 3]) }]);
        let dnns = |list: serde_json::Value| serde_json::json!({ "dnns": list });
        for bad in [
            dnns(serde_json::json!([{ "dnn": "internet", "pool": "10.45.0.1/16" }])),
            dnns(serde_json::json!([{ "dnn": "internet", "pool": "10.45.0.0/31" }])),
            dnns(serde_json::json!([{ "dnn": "internet", "pool": "10.45.0.0" }])),
            dnns(serde_json::json!([{ "dnn": "a..b", "pool": "10.45.0.0/16" }])),
            dnns(serde_json::json!([{ "dnn": "a", "pool": "10.45.0.0/16", "dns": [] }])),
            dnns(serde_json::json!([{ "dnn": "a", "pool": "10.45.0.0/16", "dns": ["dns.example"] }])),
            dnns(serde_json::json!([{ "dnn": "a", "pool": "10.45.0.0/16", "mtu": 100 }])),
            dnns(serde_json::json!([])),
            dnns(serde_json::json!([{ "dnn": "a", "pool": "10.45.0.0/16" }, { "dnn": "b", "pool": "10.45.1.0/24" }])),
            dnns(serde_json::json!([{ "dnn": "a", "pool": "10.45.0.0/16" }, { "dnn": "A", "pool": "10.46.0.0/16" }])),
            serde_json::json!({ "dnn": "internet" }),
            serde_json::json!({ "slices": [{ "sst": 1, "sd": "01" }] }),
            serde_json::json!({ "upff": "x" }),
        ] {
            assert!(SmfConfig::from_params(&bad).is_err(), "{bad}");
        }
    }
}
