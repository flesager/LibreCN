//! UDM configuration, from the deployment's `params`: the subscribers and
//! the home network keys (SUCI Profile A).
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
//! count = 1             # optional: this many subscribers from this SUPI on,
//!                       # the same keys (test fleets)
//! [[nf.params.home_network_keys]]   # optional: SUCI Profile A (ECIES, X25519)
//! id = 1                # the SUCI's home network public key identifier
//! private_key = "c53c22208b61860b06c62e5406a7b330c2b577aa5558981510d128247d38bd1d"
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
    #[serde(default = "one")]
    count: u32,
}

fn one() -> u32 {
    1
}

fn default_amf() -> String {
    "8000".into()
}

/// A home network key pair for SUCI Profile A: its id (the SUCI's key
/// identifier) and its private key.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct KeyParams {
    id: u8,
    private_key: String,
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct Params {
    subscribers: Vec<SubscriberParams>,
    home_network_keys: Vec<KeyParams>,
}

fn params(params: &serde_json::Value) -> Result<Params, String> {
    if params.is_null() {
        return Ok(Params::default());
    }
    serde_json::from_value(params.clone()).map_err(|e| format!("invalid UDM parameters: {e}"))
}

/// Home network private keys for SUCI Profile A (ECIES, X25519), by key id.
pub type HomeNetworkKeys = HashMap<u8, [u8; 32]>;

/// The home network keys of the deployment `params`.
pub fn keys_from_params(params: &serde_json::Value) -> Result<HomeNetworkKeys, String> {
    let mut keys = HomeNetworkKeys::new();
    for k in self::params(params)?.home_network_keys {
        let private = hex(&k.private_key).ok_or_else(|| format!("home network key {}: 64 hex digits", k.id))?;
        if keys.insert(k.id, private).is_some() {
            return Err(format!("home network key {}: listed twice", k.id));
        }
    }
    Ok(keys)
}

/// Subscribers by SUPI.
pub fn from_params(params: &serde_json::Value) -> Result<HashMap<String, Subscriber>, String> {
    let p = self::params(params)?;
    let mut subscribers = HashMap::with_capacity(p.subscribers.iter().map(|s| s.count as usize).sum());
    for s in p.subscribers {
        for (supi, subscriber) in parse(s)? {
            if subscribers.insert(supi.clone(), subscriber).is_some() {
                return Err(format!("subscriber {supi}: listed twice"));
            }
        }
    }
    Ok(subscribers)
}

/// Subscribers given as one object or an array of them, with the fields of
/// `[[nf.params.subscribers]]` (the web API's `POST subscribers`).
pub fn subscribers(json: &str) -> Result<Vec<(String, Subscriber)>, String> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(SubscriberParams),
        Many(Vec<SubscriberParams>),
    }
    let list = match serde_json::from_str(json).map_err(|e| format!("invalid subscriber: {e}"))? {
        OneOrMany::One(s) => vec![s],
        OneOrMany::Many(list) => list,
    };
    Ok(list.into_iter().map(parse).collect::<Result<Vec<_>, _>>()?.into_iter().flatten().collect())
}

/// The subscriber, or `count` of them: the SUPI's digits counted up, width kept.
fn parse(s: SubscriberParams) -> Result<Vec<(String, Subscriber)>, String> {
    let what = |field: &str| format!("subscriber {}: {field}", s.supi);
    if !s.supi.starts_with("imsi-") || s.supi.len() < 6 || !s.supi[5..].bytes().all(|b| b.is_ascii_digit()) {
        return Err(what("the SUPI must be imsi-<digits>"));
    }
    let sqn = match &s.sqn {
        Some(h) => hex::<6>(h).ok_or_else(|| what("sqn: 12 hex digits"))?.iter().fold(0, |a, &b| a << 8 | u64::from(b)),
        None => 0,
    };
    let subscriber = Subscriber {
        k: hex(&s.k).ok_or_else(|| what("k: 32 hex digits"))?,
        opc: hex(&s.opc).ok_or_else(|| what("opc: 32 hex digits"))?,
        amf: hex(&s.amf).ok_or_else(|| what("amf: 4 hex digits"))?,
        sqn,
    };
    let digits = &s.supi[5..];
    let first: u64 = digits.parse().map_err(|_| what("the SUPI must be imsi-<digits>"))?;
    let width = digits.len();
    if s.count == 0 || u64::from(s.count - 1) > 10u64.pow(width.min(19) as u32) - 1 - first {
        return Err(what("count: 1 or more, within the SUPI's digits"));
    }
    Ok((0..u64::from(s.count)).map(|i| (format!("imsi-{:0width$}", first + i), subscriber.clone())).collect())
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
        assert!(bad("supi", "imsi-").contains("imsi-<digits>"));
    }

    #[test]
    fn home_network_keys_are_read_and_checked() {
        let key = "c53c22208b61860b06c62e5406a7b330c2b577aa5558981510d128247d38bd1d";
        let params = serde_json::json!({ "home_network_keys": [{ "id": 1, "private_key": key }] });
        assert_eq!(keys_from_params(&params).unwrap()[&1][..2], [0xc5, 0x3c]);
        assert!(keys_from_params(&serde_json::Value::Null).unwrap().is_empty());
        let short = serde_json::json!({ "home_network_keys": [{ "id": 2, "private_key": "c53c" }] });
        assert!(keys_from_params(&short).unwrap_err().contains("64 hex digits"));
    }

    #[test]
    fn a_range_counts_the_supi_up() {
        let params = serde_json::json!({ "subscribers": [{ "supi": "imsi-001010000000998", "count": 3,
            "k": "00000000000000000000000000000001", "opc": "00000000000000000000000000000002" }]});
        let mut supis: Vec<_> = from_params(&params).unwrap().into_keys().collect();
        supis.sort();
        assert_eq!(supis, ["imsi-001010000000998", "imsi-001010000000999", "imsi-001010000001000"]);
        let mut bad = params.clone();
        bad["subscribers"][0]["supi"] = "imsi-99".into();
        bad["subscribers"][0]["count"] = 5.into();
        assert!(from_params(&bad).unwrap_err().contains("count"), "past the digits");
    }

    #[test]
    fn subscribers_come_one_or_many() {
        let one =
            r#"{"supi": "imsi-1", "k": "00000000000000000000000000000001", "opc": "00000000000000000000000000000002"}"#;
        assert_eq!(subscribers(one).unwrap()[0].0, "imsi-1");
        assert_eq!(subscribers(&format!("[{one}, {}]", one.replace("imsi-1", "imsi-2"))).unwrap().len(), 2);
        assert!(subscribers(r#"{"supi": "imsi-1"}"#).unwrap_err().contains("invalid subscriber"));
    }
}
