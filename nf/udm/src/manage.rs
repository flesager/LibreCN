//! The UDM's resources on the web API (`/api/nfs/{udm}/...`): subscribers
//! listed, added and removed while the UDM runs. Keys never leave the UDM.
//!
//! | Request | Answer |
//! |---|---|
//! | `GET subscribers` | `[{"supi", "amf", "sqn"}]`, by SUPI |
//! | `POST subscribers` | adds one subscriber or an array (fields of `[[nf.params.subscribers]]`): 201, or 409 if one exists |
//! | `DELETE subscribers/{supi}` | removes it: 200, or 404 |

use std::collections::HashMap;

use librecn_common::manage::Method;
use serde_json::json;

use crate::config::{self, Subscriber};

/// Status, JSON answer, and what changed (for the log).
pub struct Answer {
    pub status: u16,
    pub json: String,
    pub change: Option<String>,
}

fn answer(status: u16, json: serde_json::Value) -> Answer {
    Answer { status, json: json.to_string(), change: None }
}

pub fn handle(subscribers: &mut HashMap<String, Subscriber>, method: Method, path: &str, body: &str) -> Answer {
    match (method, path.split_once('/').unwrap_or((path, ""))) {
        (Method::Get, ("subscribers", "")) => {
            let mut list: Vec<_> = subscribers.iter().collect();
            list.sort_by(|a, b| a.0.cmp(b.0));
            let list: Vec<_> = list
                .into_iter()
                .map(|(supi, s)| {
                    json!({ "supi": supi, "amf": format!("{:02X}{:02X}", s.amf[0], s.amf[1]), "sqn": format!("{:012X}", s.sqn) })
                })
                .collect();
            answer(200, json!(list))
        }
        (Method::Post, ("subscribers", "")) => {
            let new = match config::subscribers(body) {
                Ok(new) => new,
                Err(e) => return answer(400, json!({ "error": e })),
            };
            let mut supis: Vec<&str> = new.iter().map(|(supi, _)| supi.as_str()).collect();
            supis.sort_unstable();
            if let Some(twice) = supis.windows(2).find(|w| w[0] == w[1]) {
                return answer(400, json!({ "error": format!("subscriber {}: listed twice", twice[0]) }));
            }
            let existing: Vec<_> = supis.iter().filter(|s| subscribers.contains_key(**s)).collect();
            if !existing.is_empty() {
                return answer(409, json!({ "error": "subscribers exist already", "supis": existing }));
            }
            let added = supis.join(", ");
            let reply = json!({ "added": supis });
            subscribers.extend(new);
            Answer { status: 201, json: reply.to_string(), change: Some(format!("subscriber(s) added: {added}")) }
        }
        (Method::Delete, ("subscribers", supi)) if !supi.is_empty() => match subscribers.remove(supi) {
            Some(_) => Answer {
                status: 200,
                json: json!({ "removed": supi }).to_string(),
                change: Some(format!("subscriber removed: {supi}")),
            },
            None => answer(404, json!({ "error": format!("no subscriber {supi}") })),
        },
        _ => answer(404, json!({ "error": format!("no resource {path:?}") })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const K: &str = r#""k": "465B5CE8B199B49FAA5F0A2EE238A6BC", "opc": "E8ED289DEBA952E4283B54E88E6183CA""#;

    #[test]
    fn subscribers_are_listed_added_and_removed() {
        let mut subs = HashMap::new();
        let list = |subs: &mut HashMap<_, _>| handle(subs, Method::Get, "subscribers", "").json;
        assert_eq!(list(&mut subs), "[]");

        let added = handle(
            &mut subs,
            Method::Post,
            "subscribers",
            &format!(r#"{{"supi": "imsi-2", {K}, "sqn": "000000000020"}}"#),
        );
        assert_eq!((added.status, added.change.as_deref()), (201, Some("subscriber(s) added: imsi-2")));
        let two = format!(r#"[{{"supi": "imsi-1", {K}}}, {{"supi": "imsi-3", {K}, "amf": "9000"}}]"#);
        assert_eq!(handle(&mut subs, Method::Post, "subscribers", &two).status, 201);
        assert_eq!(
            list(&mut subs),
            r#"[{"amf":"8000","sqn":"000000000000","supi":"imsi-1"},{"amf":"8000","sqn":"000000000020","supi":"imsi-2"},{"amf":"9000","sqn":"000000000000","supi":"imsi-3"}]"#,
            "by SUPI, without keys"
        );

        let again = handle(&mut subs, Method::Post, "subscribers", &format!(r#"{{"supi": "imsi-1", {K}}}"#));
        assert_eq!((again.status, again.change), (409, None));
        let twice = format!(r#"[{{"supi": "imsi-9", {K}}}, {{"supi": "imsi-9", {K}}}]"#);
        assert_eq!(handle(&mut subs, Method::Post, "subscribers", &twice).status, 400);
        assert_eq!(handle(&mut subs, Method::Post, "subscribers", r#"{"supi": "imsi-4"}"#).status, 400);
        assert_eq!(subs.len(), 3, "a refused request adds nothing");

        assert_eq!(handle(&mut subs, Method::Delete, "subscribers/imsi-2", "").status, 200);
        assert_eq!(handle(&mut subs, Method::Delete, "subscribers/imsi-2", "").status, 404);
        assert_eq!(handle(&mut subs, Method::Get, "keys", "").status, 404);
        assert_eq!(subs.len(), 2);
    }
}
