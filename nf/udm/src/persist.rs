//! What the UDM keeps across restarts, in its state file (`[oam]
//! state_dir`): the subscribers the web API added (with their keys, as the
//! configuration gives them), the configuration's subscribers it removed,
//! and every subscriber's SQN. The configuration stays the source of what it
//! lists; the state file only keeps what the web API changed, and the SQNs.
//!
//! An SQN restored is moved [`SQN_JUMP`] ahead: one used after the last
//! snapshot is never used again (TS 33.102 Annex C.3.2; the USIM accepts a
//! jump of up to 2^28 SEQ).

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config::{self, Subscriber};

/// SEQ steps added to a restored SQN (in SQN units: SEQ is above the 5 IND bits).
pub const SQN_JUMP: u64 = 1024 << 5;

#[derive(Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct Saved {
    /// Added through the web API, as `[[nf.params.subscribers]]` entries.
    provisioned: Vec<serde_json::Value>,
    /// The configuration's subscribers removed through the web API.
    removed: Vec<String>,
    /// SUPI → SQN, 12 hexadecimal digits.
    sqn: BTreeMap<String, String>,
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect()
}

/// The state file's text.
pub fn snapshot(subscribers: &HashMap<String, Subscriber>, removed: &BTreeSet<String>) -> String {
    let mut provisioned: Vec<_> = subscribers.iter().filter(|(_, s)| s.provisioned).collect();
    provisioned.sort_by(|a, b| a.0.cmp(b.0));
    let saved = Saved {
        provisioned: provisioned
            .into_iter()
            .map(|(supi, s)| {
                json!({ "supi": supi, "k": hex(&s.k), "opc": hex(&s.opc), "amf": hex(&s.amf),
                        "sqn": format!("{:012X}", s.sqn), "auth": s.auth() })
            })
            .collect(),
        removed: removed.iter().cloned().collect(),
        sqn: subscribers.iter().map(|(supi, s)| (supi.clone(), format!("{:012X}", s.sqn))).collect(),
    };
    serde_json::to_string_pretty(&saved).expect("plain values")
}

/// What a restore changed.
#[derive(Debug, PartialEq, Eq)]
pub struct Restored {
    pub provisioned: usize,
    pub removed: usize,
    pub sqns: usize,
}

/// Applies the state file's `text` to the configuration's `subscribers`.
pub fn restore(
    text: &str,
    subscribers: &mut HashMap<String, Subscriber>,
    removed: &mut BTreeSet<String>,
) -> Result<Restored, String> {
    let saved: Saved = serde_json::from_str(text).map_err(|e| format!("unreadable: {e}"))?;
    for supi in &saved.removed {
        subscribers.remove(supi);
        removed.insert(supi.clone());
    }
    let provisioned = config::subscribers(&serde_json::Value::Array(saved.provisioned).to_string())?;
    let count = provisioned.len();
    for (supi, s) in provisioned {
        subscribers.insert(supi, Subscriber { provisioned: true, ..s });
    }
    let mut sqns = 0;
    for (supi, sqn) in &saved.sqn {
        let sqn = u64::from_str_radix(sqn, 16).map_err(|_| format!("SQN of {supi}: {sqn:?}"))?;
        if let Some(s) = subscribers.get_mut(supi) {
            s.sqn = (s.sqn.max(sqn) + SQN_JUMP) & ((1 << 48) - 1);
            sqns += 1;
        }
    }
    Ok(Restored { provisioned: count, removed: saved.removed.len(), sqns })
}

#[cfg(test)]
mod tests {
    use super::*;

    const K: &str = r#""k": "465B5CE8B199B49FAA5F0A2EE238A6BC", "opc": "E8ED289DEBA952E4283B54E88E6183CA""#;

    fn table(json: &str) -> HashMap<String, Subscriber> {
        config::subscribers(json).unwrap().into_iter().collect()
    }

    #[test]
    fn the_web_api_changes_and_the_sqns_survive_a_restart() {
        // Running: two configured, one removed, one added, SQNs moved on.
        let mut running = table(&format!(r#"[{{"supi": "imsi-1", {K}}}, {{"supi": "imsi-2", {K}}}]"#));
        let mut removed = BTreeSet::new();
        running.remove("imsi-2");
        removed.insert("imsi-2".to_string());
        let added = table(&format!(r#"{{"supi": "imsi-3", {K}, "auth": "eap-aka-prime"}}"#));
        running.extend(added.into_iter().map(|(supi, s)| (supi, Subscriber { provisioned: true, ..s })));
        running.get_mut("imsi-1").unwrap().sqn = 0x40;
        running.get_mut("imsi-3").unwrap().sqn = 0x20;
        let text = snapshot(&running, &removed);
        assert!(!text.contains("imsi-1\", \"k"), "the configuration's keys are not copied");

        // Restarted from the same configuration.
        let mut subscribers = table(&format!(r#"[{{"supi": "imsi-1", {K}}}, {{"supi": "imsi-2", {K}}}]"#));
        let mut again = BTreeSet::new();
        let restored = restore(&text, &mut subscribers, &mut again).unwrap();
        assert_eq!(restored, Restored { provisioned: 1, removed: 1, sqns: 2 });
        let mut supis: Vec<_> = subscribers.keys().cloned().collect();
        supis.sort();
        assert_eq!(supis, ["imsi-1", "imsi-3"]);
        assert_eq!(subscribers["imsi-1"].sqn, 0x40 + SQN_JUMP, "ahead of any SQN used");
        assert!(subscribers["imsi-3"].provisioned && subscribers["imsi-3"].eap_aka_prime);
        assert_eq!(again, removed);
        assert!(restore("{", &mut subscribers, &mut again).is_err());
    }
}
