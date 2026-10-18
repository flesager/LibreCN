//! What the PCF keeps across restarts (`[oam] state_dir`): the policies the
//! web API set, as `[[nf.params.policies]]` entries, and the configuration's
//! it removed. The configuration stays the source of what it lists.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::config::{Entry, Policies};

#[derive(Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct Saved {
    set: Vec<serde_json::Value>,
    removed: Vec<String>,
}

/// What a restore changed.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Restored {
    pub set: usize,
    pub removed: usize,
}

pub fn snapshot(policies: &Policies, removed: &BTreeSet<String>) -> String {
    let mut set: Vec<_> = policies.by_supi.iter().filter(|(_, e)| e.provisioned).collect();
    set.sort_by(|a, b| a.0.cmp(b.0));
    let saved = Saved {
        set: set.into_iter().map(|(_, e)| e.source.clone()).collect(),
        removed: removed.iter().cloned().collect(),
    };
    serde_json::to_string_pretty(&saved).expect("plain values")
}

pub fn restore(text: &str, policies: &mut Policies, removed: &mut BTreeSet<String>) -> Result<Restored, String> {
    let saved: Saved = serde_json::from_str(text).map_err(|e| format!("unreadable: {e}"))?;
    for supi in &saved.removed {
        policies.by_supi.remove(supi);
        removed.insert(supi.clone());
    }
    let entries = policies.entries(&serde_json::Value::Array(saved.set).to_string())?;
    let set = entries.len();
    for (supi, entry) in entries {
        policies.by_supi.insert(supi, Entry { provisioned: true, ..entry });
    }
    Ok(Restored { set, removed: saved.removed.len() })
}
