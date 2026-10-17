//! UDM configuration, from the deployment's `params`: the subscribers.
//!
//! ```toml
//! [[nf]]
//! kind = "udm"
//! name = "udm"
//! [[nf.params.subscribers]]
//! supi = "imsi-999700000000001"
//! k = "465B5CE8B199B49FAA5F0A2EE238A6BC"
//! opc = "E8ED289DEBA952E4283B54E88E6183CA"
//! amf = "8000"          # optional: the default
//! sqn = "000000000000"  # optional: the last SQN used
//! ```

use std::collections::HashMap;

use serde::Deserialize;

/// What the UDM knows of a subscriber.
#[derive(Clone, PartialEq, Eq)]
pub struct Subscriber {
    pub k: [u8; 16],
    pub opc: [u8; 16],
    /// Authentication management field; its separation bit (the first) must be set for 5G.
    pub amf: [u8; 2],
    /// The last sequence number used, 48 bits.
    pub sqn: u64,
}

impl std::fmt::Debug for Subscriber {
    /// Without the keys.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Subscriber {{ amf: {:02x}{:02x}, sqn: {:#x} }}", self.amf[0], self.amf[1], self.sqn)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SubscriberParams {
    supi: String,
    k: String,
    opc: String,
    #[serde(default = "default_amf")]
    amf: String,
    #[serde(default)]
    sqn: Option<String>,
}

fn default_amf() -> String {
    "8000".into()
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct Params {
    subscribers: Vec<SubscriberParams>,
}

/// Subscribers by SUPI.
pub fn from_params(params: &serde_json::Value) -> Result<HashMap<String, Subscriber>, String> {
    let p: Params = if params.is_null() {
        Params::default()
    } else {
        serde_json::from_value(params.clone()).map_err(|e| format!("invalid UDM parameters: {e}"))?
    };
    let mut subscribers = HashMap::with_capacity(p.subscribers.len());
    for s in p.subscribers {
        let what = |field: &str| format!("subscriber {}: {field}", s.supi);
        if !s.supi.starts_with("imsi-") || !s.supi[5..].bytes().all(|b| b.is_ascii_digit()) {
            return Err(what("the SUPI must be imsi-<digits>"));
        }
        let sqn = match &s.sqn {
            Some(h) => {
                hex::<6>(h).ok_or_else(|| what("sqn: 12 hex digits"))?.iter().fold(0, |a, &b| a << 8 | u64::from(b))
            }
            None => 0,
        };
        let subscriber = Subscriber {
            k: hex(&s.k).ok_or_else(|| what("k: 32 hex digits"))?,
            opc: hex(&s.opc).ok_or_else(|| what("opc: 32 hex digits"))?,
            amf: hex(&s.amf).ok_or_else(|| what("amf: 4 hex digits"))?,
            sqn,
        };
        if subscribers.insert(s.supi.clone(), subscriber).is_some() {
            return Err(what("listed twice"));
        }
    }
    Ok(subscribers)
}

fn hex<const N: usize>(s: &str) -> Option<[u8; N]> {
    if s.len() != 2 * N {
        return None;
    }
    let mut out = [0; N];
    for (i, o) in out.iter_mut().enumerate() {
        *o = u8::from_str_radix(s.get(2 * i..2 * i + 2)?, 16).ok()?;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscribers_are_read_and_checked() {
        let params = serde_json::json!({ "subscribers": [{
            "supi": "imsi-999700000000001",
            "k": "465B5CE8B199B49FAA5F0A2EE238A6BC",
            "opc": "E8ED289DEBA952E4283B54E88E6183CA",
            "sqn": "000000000040"
        }]});
        let subs = from_params(&params).unwrap();
        let s = &subs["imsi-999700000000001"];
        assert_eq!((s.k[0], s.opc[15], s.amf, s.sqn), (0x46, 0xca, [0x80, 0], 0x40));
        assert_eq!(format!("{s:?}"), "Subscriber { amf: 8000, sqn: 0x40 }", "no keys in logs");
        assert!(from_params(&serde_json::Value::Null).unwrap().is_empty());

        let bad = |f: &str, v: &str| {
            let mut p = params.clone();
            p["subscribers"][0][f] = v.into();
            from_params(&p).unwrap_err()
        };
        assert!(bad("k", "00").contains("k: 32 hex digits"));
        assert!(bad("supi", "nai-x@y").contains("imsi-<digits>"));
        assert!(bad("sqn", "zz0000000000").contains("sqn"));
    }
}
