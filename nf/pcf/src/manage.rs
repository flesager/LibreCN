//! The PCF's resources on the web API (`/api/nfs/{pcf}/...`).
//!
//! | Request | Answer |
//! |---|---|
//! | `GET policies` | `{"default", "policies": [...]}`: the default policy, the first [`LISTED`] subscribers' |
//! | `GET policies/{supi}` | that subscriber's policy (its own, else the default) |
//! | `POST policies` | sets one or an array (fields of `[[nf.params.policies]]`), replacing any: 200 |
//! | `DELETE policies/{supi}` | the subscriber falls back to the default: 200, or 404 |
//! | `DELETE policies/{supi}?count=N` | the same for the `N` subscribers from `supi` on: 200 (those that had one), or 404 |
//!
//! A subscriber's new UE-AMBR reaches its AMF, which modifies the UE's
//! context; session policies apply to the sessions set up after.

use std::collections::BTreeSet;

use librecn_common::manage::Method;
use librecn_sbi::policy::{PacketFilter, Rates, direction};
use serde_json::{Value, json};

use crate::config::{Entry, Policies, Policy};

/// Subscribers listed by `GET policies`.
pub const LISTED: usize = 64;

/// Status, JSON answer, what changed (for the log), and whose policy.
pub struct Answer {
    pub status: u16,
    pub json: String,
    pub change: Option<String>,
    pub changed: Vec<String>,
}

fn answer(status: u16, json: Value) -> Answer {
    Answer { status, json: json.to_string(), change: None, changed: vec![] }
}

fn mbps(r: Rates) -> Value {
    json!({ "uplink": r.uplink / 1_000_000, "downlink": r.downlink / 1_000_000 })
}

fn filter(f: &PacketFilter) -> Value {
    let dir = match f.direction {
        direction::UPLINK => "uplink",
        direction::DOWNLINK => "downlink",
        _ => "both",
    };
    let ports =
        |p: [u16; 2]| (p != [0, 0]).then(|| if p[0] == p[1] { p[0].to_string() } else { format!("{}-{}", p[0], p[1]) });
    let remote = (f.remote_prefix > 0).then(|| format!("{}/{}", std::net::Ipv4Addr::from(f.remote), f.remote_prefix));
    json!({ "direction": dir, "remote": remote, "protocol": f.protocol,
            "remote_ports": ports(f.remote_ports), "local_ports": ports(f.local_ports) })
}

/// A policy as the web API shows it.
pub fn view(p: &Policy) -> Value {
    let sessions: Vec<Value> = p
        .sessions
        .iter()
        .map(|s| {
            let r = &s.rule;
            let flows: Vec<Value> = r
                .rules()
                .iter()
                .map(|f| {
                    json!({ "five_qi": f.five_qi, "arp": f.arp, "precedence": f.precedence,
                            "mbr_mbps": (!f.mbr.is_zero()).then(|| mbps(f.mbr)),
                            "gbr_mbps": (!f.gbr.is_zero()).then(|| mbps(f.gbr)),
                            "filters": f.filters().iter().map(filter).collect::<Vec<_>>() })
                })
                .collect();
            json!({ "dnn": s.dnn, "five_qi": r.five_qi, "arp": r.arp, "ambr_mbps": mbps(r.ambr), "flows": flows })
        })
        .collect();
    json!({ "ue_ambr_mbps": mbps(p.ue_ambr), "sessions": sessions })
}

/// `removed`: the configuration's policies removed here (kept in the state file).
/// The `count` SUPIs from `supi` on, its digits kept (`imsi-001` to `imsi-003`).
fn range(supi: &str, count: u64) -> Option<Vec<String>> {
    let digits = supi.strip_prefix("imsi-").filter(|d| !d.is_empty() && d.len() <= 19)?;
    let first: u64 = digits.parse().ok()?;
    let last = first.checked_add(count.checked_sub(1)?)?;
    (last.to_string().len() <= digits.len())
        .then(|| (first..=last).map(|n| format!("imsi-{n:0width$}", width = digits.len())).collect())
}

/// `target`: the path, with its query.
pub fn handle(
    policies: &mut Policies,
    removed: &mut BTreeSet<String>,
    method: Method,
    target: &str,
    body: &str,
) -> Answer {
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let count = query.split('&').find_map(|pair| pair.strip_prefix("count=")).map_or(Some(1), |n| n.parse().ok());
    match (method, path.split_once('/').unwrap_or((path, ""))) {
        (Method::Get, ("policies", "")) => {
            let mut supis: Vec<_> = policies.by_supi.keys().collect();
            supis.sort();
            let listed: Vec<Value> = supis
                .into_iter()
                .take(LISTED)
                .map(|supi| {
                    let e = &policies.by_supi[supi];
                    let mut v = view(&e.policy);
                    v["supi"] = supi.as_str().into();
                    v["provisioned"] = e.provisioned.into();
                    v
                })
                .collect();
            answer(
                200,
                json!({ "default": view(&policies.default), "count": policies.by_supi.len(), "policies": listed }),
            )
        }
        (Method::Get, ("policies", supi)) => {
            let mut v = view(policies.of(supi));
            v["supi"] = supi.into();
            v["own"] = policies.by_supi.contains_key(supi).into();
            answer(200, v)
        }
        (Method::Post, ("policies", "")) => {
            let entries = match policies.entries(body) {
                Ok(entries) => entries,
                Err(e) => return answer(400, json!({ "error": e })),
            };
            let supis: Vec<String> = entries.iter().map(|(s, _)| s.clone()).collect();
            for (supi, entry) in entries {
                policies.by_supi.insert(supi, Entry { provisioned: true, ..entry });
            }
            let change = Some(format!("policy(ies) set: {}", supis.join(", ")));
            Answer { status: 200, json: json!({ "set": supis }).to_string(), change, changed: supis }
        }
        (Method::Delete, ("policies", supi)) if !supi.is_empty() => {
            let Some(supis) = count.and_then(|n| range(supi, n)) else {
                return answer(400, json!({ "error": "count: 1 or more, within the SUPI's digits" }));
            };
            let gone: Vec<String> = supis
                .into_iter()
                .filter(|s| {
                    let Some(entry) = policies.by_supi.remove(s) else { return false };
                    if !entry.provisioned {
                        removed.insert(s.clone());
                    }
                    true
                })
                .collect();
            if gone.is_empty() {
                return answer(404, json!({ "error": format!("no policy of {supi}") }));
            }
            Answer {
                status: 200,
                json: json!({ "removed": gone }).to_string(),
                change: Some(format!("policy(ies) removed: {}, the default applies", gone.join(", "))),
                changed: gone,
            }
        }
        _ => answer(404, json!({ "error": format!("no resource {path:?}") })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The GUI edits a policy as `GET policies` shows it, `provisioned` left out.
    #[test]
    fn a_listed_policy_can_be_posted_back_unchanged() {
        let source = json!({ "supi": "imsi-1", "ue_ambr_mbps": { "uplink": 5, "downlink": 10 },
            "sessions": [{ "dnn": "internet", "ambr_mbps": { "uplink": 2, "downlink": 4 },
                "flows": [{ "five_qi": 1, "mbr_mbps": { "uplink": 1, "downlink": 1 }, "gbr_mbps": { "uplink": 1, "downlink": 1 },
                            "filters": [{ "remote": "10.10.0.0/16", "protocol": 17, "remote_ports": "5060" }] },
                          { "five_qi": 9, "filters": [{ "direction": "downlink", "local_ports": "80-81" }] }] }] });
        let mut policies = Policies::from_params(&json!({ "policies": [source] })).unwrap();
        let mut removed = BTreeSet::new();
        let listed: Value =
            serde_json::from_str(&handle(&mut policies, &mut removed, Method::Get, "policies", "").json).unwrap();
        let mut entry = listed["policies"][0].clone();
        entry.as_object_mut().unwrap().remove("provisioned");
        let before = policies.of("imsi-1").clone();
        let posted = handle(&mut policies, &mut removed, Method::Post, "policies", &entry.to_string());
        assert_eq!(posted.status, 200, "{}", posted.json);
        assert_eq!(policies.of("imsi-1"), &before);
        assert!(policies.by_supi["imsi-1"].provisioned);
    }

    #[test]
    fn a_range_of_policies_is_removed() {
        let mut policies = Policies::from_params(&json!({ "policies": [] })).unwrap();
        let mut removed = BTreeSet::new();
        let three = json!({ "supi": "imsi-0098", "count": 3, "ue_ambr_mbps": { "uplink": 5, "downlink": 10 } });
        assert_eq!(handle(&mut policies, &mut removed, Method::Post, "policies", &three.to_string()).status, 200);
        let gone = handle(&mut policies, &mut removed, Method::Delete, "policies/imsi-0099?count=5", "");
        assert_eq!((gone.status, gone.json.as_str()), (200, r#"{"removed":["imsi-0099","imsi-0100"]}"#));
        assert_eq!(policies.by_supi.keys().collect::<Vec<_>>(), ["imsi-0098"]);
        let none = handle(&mut policies, &mut removed, Method::Delete, "policies/imsi-0099?count=5", "");
        assert_eq!(none.status, 404);
        let wide = handle(&mut policies, &mut removed, Method::Delete, "policies/imsi-9999?count=2", "");
        assert_eq!(wide.status, 400, "beyond the SUPI's digits");
        assert_eq!(range("imsi-1", 1).unwrap(), ["imsi-1"]);
        assert!(range("imsi-1", 0).is_none() && range("x-1", 1).is_none());
    }
}
