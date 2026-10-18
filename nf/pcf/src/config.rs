//! The PCF's policies, from the deployment's `params` (and the web API, in
//! the same form): a `default` one, and one per subscriber.
//!
//! ```toml
//! [nf.params.default]                       # subscribers without their own
//! ue_ambr_mbps = { uplink = 1000, downlink = 1000 }
//! [[nf.params.default.sessions]]
//! five_qi = 9
//! arp = 8
//! ambr_mbps = { uplink = 1000, downlink = 1000 }
//!
//! [[nf.params.policies]]
//! supi = "imsi-999700000000001"
//! count = 1                                 # optional: this many from this SUPI on
//! ue_ambr_mbps = { uplink = 50, downlink = 200 }
//! [[nf.params.policies.sessions]]           # by DNN; no `dnn`: any DNN
//! dnn = "internet"
//! five_qi = 9
//! arp = 8
//! ambr_mbps = { uplink = 20, downlink = 100 }
//! [[nf.params.policies.sessions.flows]]     # PCC rules: extra QoS flows, 4 at most
//! five_qi = 1
//! arp = 2
//! precedence = 10                           # lowest first
//! mbr_mbps = { uplink = 1, downlink = 1 }
//! gbr_mbps = { uplink = 1, downlink = 1 }   # GBR 5QIs only, and then required
//! filters = [{ direction = "both", remote = "10.10.0.0/16", protocol = 17, remote_ports = "5060-5061" }]
//! ```
//!
//! What an entry leaves out comes from the `default` one, then from the
//! built-in values: 5QI 9, ARP 8, 1000 Mbit/s each way. Flows are kept in
//! precedence order (unique, below the default rule's 255); each becomes a
//! QoS flow of the session (QFI 2 on), the UPF taking the first whose
//! filters match a downlink packet.

use std::collections::HashMap;
use std::net::Ipv4Addr;

use librecn_sbi::policy::{MAX_FILTERS, MAX_RULES, PacketFilter, PccRule, Rates, SessionRule, direction};
use serde::Deserialize;
use zerocopy::FromZeros;

const DEFAULT_FIVE_QI: u8 = 9;
const DEFAULT_ARP: u8 = 8;
const DEFAULT_MBPS: u32 = 1000;

#[derive(Deserialize, Clone, Copy)]
#[serde(deny_unknown_fields)]
struct Mbps {
    uplink: u32,
    downlink: u32,
}

impl Mbps {
    fn rates(self) -> Rates {
        Rates { uplink: u64::from(self.uplink) * 1_000_000, downlink: u64::from(self.downlink) * 1_000_000 }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FilterParams {
    #[serde(default)]
    direction: Option<String>,
    #[serde(default)]
    remote: Option<String>,
    #[serde(default)]
    protocol: u8,
    #[serde(default)]
    remote_ports: Option<String>,
    #[serde(default)]
    local_ports: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FlowParams {
    five_qi: u8,
    arp: Option<u8>,
    precedence: Option<u8>,
    mbr_mbps: Option<Mbps>,
    gbr_mbps: Option<Mbps>,
    filters: Vec<FilterParams>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionParams {
    dnn: Option<String>,
    five_qi: Option<u8>,
    arp: Option<u8>,
    ambr_mbps: Option<Mbps>,
    #[serde(default)]
    flows: Vec<FlowParams>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct DefaultParams {
    ue_ambr_mbps: Option<Mbps>,
    #[serde(default)]
    sessions: Vec<SessionParams>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PolicyParams {
    supi: String,
    #[serde(default = "one")]
    count: u32,
    ue_ambr_mbps: Option<Mbps>,
    #[serde(default)]
    sessions: Vec<SessionParams>,
}

fn one() -> u32 {
    1
}

#[derive(Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct Params {
    default: Option<DefaultParams>,
    policies: Vec<serde_json::Value>,
}

/// A session's policy, for the DNN it names (none: any).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionPolicy {
    pub dnn: Option<String>,
    pub rule: SessionRule,
}

/// One subscriber's policy, or the default one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Policy {
    pub ue_ambr: Rates,
    pub sessions: Vec<SessionPolicy>,
}

impl Policy {
    /// The rule of a session to `dnn`: the first matching session policy.
    pub fn session(&self, dnn: &str) -> Option<&SessionRule> {
        let matches = |s: &&SessionPolicy| s.dnn.as_deref().is_none_or(|d| d.eq_ignore_ascii_case(dnn));
        self.sessions.iter().find(matches).map(|s| &s.rule)
    }
}

/// Whether `five_qi` is a GBR 5QI (TS 23.501 Table 5.7.4-1).
pub fn is_gbr(five_qi: u8) -> bool {
    matches!(five_qi, 1..=4 | 65..=67 | 71..=76 | 82..=86)
}

fn ports(text: &Option<String>, what: &str) -> Result<[u16; 2], String> {
    let Some(text) = text else { return Ok([0, 0]) };
    let bad = || format!("{what} {text:?}: a port or first-last");
    let (first, last) = text.split_once('-').unwrap_or((text, text));
    let (first, last): (u16, u16) = (first.trim().parse().map_err(|_| bad())?, last.trim().parse().map_err(|_| bad())?);
    if first == 0 || last < first {
        return Err(bad());
    }
    Ok([first, last])
}

fn filter(f: &FilterParams) -> Result<PacketFilter, String> {
    let direction = match f.direction.as_deref() {
        None | Some("both") => direction::BOTH,
        Some("uplink") => direction::UPLINK,
        Some("downlink") => direction::DOWNLINK,
        Some(other) => return Err(format!("filter direction {other:?}: uplink, downlink or both")),
    };
    let remote = match &f.remote {
        None => ([0; 4], 0),
        Some(text) => {
            let bad = || format!("filter remote {text:?}: a.b.c.d or a.b.c.d/n");
            let (addr, len) = text.split_once('/').unwrap_or((text, "32"));
            let addr: Ipv4Addr = addr.parse().map_err(|_| bad())?;
            let len: u8 = len.parse().ok().filter(|l| (1..=32).contains(l)).ok_or_else(bad)?;
            (addr.octets(), len)
        }
    };
    Ok(PacketFilter::new(
        direction,
        f.protocol,
        remote,
        ports(&f.remote_ports, "remote_ports")?,
        ports(&f.local_ports, "local_ports")?,
    ))
}

fn session(s: &SessionParams, base: Option<&SessionRule>) -> Result<SessionPolicy, String> {
    let mut rule = SessionRule::new_zeroed();
    rule.five_qi = s.five_qi.or(base.map(|b| b.five_qi)).unwrap_or(DEFAULT_FIVE_QI);
    rule.arp = s.arp.or(base.map(|b| b.arp)).unwrap_or(DEFAULT_ARP);
    let default_ambr = Mbps { uplink: DEFAULT_MBPS, downlink: DEFAULT_MBPS }.rates();
    rule.ambr = s.ambr_mbps.map(Mbps::rates).or(base.map(|b| b.ambr)).unwrap_or(default_ambr);
    if is_gbr(rule.five_qi) {
        return Err(format!("5QI {} is a GBR 5QI: the default flow is non-GBR", rule.five_qi));
    }
    if !(1..=15).contains(&rule.arp) {
        return Err(format!("arp {}: 1 to 15", rule.arp));
    }
    if s.flows.len() > MAX_RULES {
        return Err(format!("{} flows: {MAX_RULES} at most", s.flows.len()));
    }
    let mut rules = Vec::with_capacity(s.flows.len());
    for (i, f) in s.flows.iter().enumerate() {
        let mut r = PccRule::new_zeroed();
        r.five_qi = f.five_qi;
        r.arp = f.arp.unwrap_or(rule.arp);
        r.precedence = f.precedence.unwrap_or(10 * (i as u8 + 1));
        r.mbr = f.mbr_mbps.map(Mbps::rates).unwrap_or_default();
        r.gbr = f.gbr_mbps.map(Mbps::rates).unwrap_or_default();
        let what = format!("flow with 5QI {}", f.five_qi);
        match (is_gbr(f.five_qi), f.gbr_mbps.is_some(), f.mbr_mbps.is_some()) {
            (true, true, true) if r.mbr.uplink >= r.gbr.uplink && r.mbr.downlink >= r.gbr.downlink => {}
            (true, true, true) => return Err(format!("{what}: mbr_mbps below gbr_mbps")),
            (true, _, _) => return Err(format!("{what}: a GBR 5QI needs gbr_mbps and mbr_mbps")),
            (false, true, _) => return Err(format!("{what}: gbr_mbps only for a GBR 5QI")),
            (false, false, _) => {}
        }
        if !(1..=15).contains(&r.arp) {
            return Err(format!("{what}: arp {}: 1 to 15", r.arp));
        }
        if f.filters.is_empty() || f.filters.len() > MAX_FILTERS {
            return Err(format!("{what}: 1 to {MAX_FILTERS} filters"));
        }
        let filters = f.filters.iter().map(filter).collect::<Result<Vec<_>, _>>()?;
        r.set_filters(&filters);
        if r.precedence == u8::MAX {
            return Err(format!("{what}: precedence 255 is the default rule's"));
        }
        rules.push(r);
    }
    // In evaluation order: the UPF takes the first rule matching a packet.
    rules.sort_by_key(|r| r.precedence);
    if let Some(twice) = rules.windows(2).find(|w| w[0].precedence == w[1].precedence) {
        return Err(format!("two flows with precedence {}", twice[0].precedence));
    }
    rule.set_rules(&rules);
    Ok(SessionPolicy { dnn: s.dnn.clone(), rule })
}

/// A policy from its sessions and UE-AMBR, what they leave out from `base`.
fn policy(ue_ambr: Option<Mbps>, sessions: &[SessionParams], base: &Policy) -> Result<Policy, String> {
    let base_session = base.sessions.first().map(|s| &s.rule);
    let sessions = if sessions.is_empty() {
        base.sessions.clone()
    } else {
        sessions
            .iter()
            .map(|s| session(s, s.dnn.as_deref().and_then(|d| base.session(d)).or(base_session)))
            .collect::<Result<_, _>>()?
    };
    Ok(Policy { ue_ambr: ue_ambr.map(Mbps::rates).unwrap_or(base.ue_ambr), sessions })
}

/// The built-in policy: 5QI 9, ARP 8, 1000 Mbit/s each way.
fn built_in() -> Policy {
    let any = SessionParams { dnn: None, five_qi: None, arp: None, ambr_mbps: None, flows: vec![] };
    let ue_ambr = Mbps { uplink: DEFAULT_MBPS, downlink: DEFAULT_MBPS }.rates();
    Policy { ue_ambr, sessions: vec![session(&any, None).expect("valid")] }
}

/// A subscriber's policy, and where it came from.
#[derive(Clone, Debug)]
pub struct Entry {
    pub policy: Policy,
    /// Its `[[nf.params.policies]]` entry, for this SUPI alone.
    pub source: serde_json::Value,
    /// Set through the web API, not the configuration: kept in the state file.
    pub provisioned: bool,
}

/// Every policy.
#[derive(Clone, Debug)]
pub struct Policies {
    pub default: Policy,
    pub by_supi: HashMap<String, Entry>,
}

impl Policies {
    /// The policy of `supi`: its own, else the default one.
    pub fn of(&self, supi: &str) -> &Policy {
        self.by_supi.get(supi).map_or(&self.default, |e| &e.policy)
    }

    /// Policies given as one entry or an array of them, with the fields of
    /// `[[nf.params.policies]]` (the web API's `POST policies`).
    pub fn entries(&self, json: &str) -> Result<Vec<(String, Entry)>, String> {
        let value: serde_json::Value = serde_json::from_str(json).map_err(|e| format!("invalid policy: {e}"))?;
        let list = match value {
            serde_json::Value::Array(list) => list,
            one => vec![one],
        };
        let mut out = Vec::new();
        for entry in list {
            out.extend(self.entry(entry)?);
        }
        Ok(out)
    }

    /// One `[[nf.params.policies]]` entry: `count` subscribers from its SUPI on.
    fn entry(&self, value: serde_json::Value) -> Result<Vec<(String, Entry)>, String> {
        let p: PolicyParams = serde_json::from_value(value.clone()).map_err(|e| format!("invalid policy: {e}"))?;
        let what = |field: &str| format!("policy of {}: {field}", p.supi);
        if !p.supi.starts_with("imsi-") || p.supi.len() < 6 || !p.supi[5..].bytes().all(|b| b.is_ascii_digit()) {
            return Err(what("the SUPI must be imsi-<digits>"));
        }
        let policy = policy(p.ue_ambr_mbps, &p.sessions, &self.default).map_err(|e| what(&e))?;
        let digits = &p.supi[5..];
        let (first, width): (u64, usize) = (digits.parse().map_err(|_| what("imsi-<digits>"))?, digits.len());
        if p.count == 0 || u64::from(p.count - 1) > 10u64.pow(width.min(19) as u32) - 1 - first {
            return Err(what("count: 1 or more, within the SUPI's digits"));
        }
        Ok((0..u64::from(p.count))
            .map(|i| {
                let supi = format!("imsi-{:0width$}", first + i);
                let mut source = value.clone();
                if let Some(o) = source.as_object_mut() {
                    o.remove("count");
                    o.insert("supi".into(), supi.clone().into());
                }
                (supi, Entry { policy: policy.clone(), source, provisioned: false })
            })
            .collect())
    }

    /// From deployment parameters (`null`: the built-in default alone).
    pub fn from_params(params: &serde_json::Value) -> Result<Self, String> {
        let p: Params = if params.is_null() {
            Params::default()
        } else {
            serde_json::from_value(params.clone()).map_err(|e| format!("invalid PCF parameters: {e}"))?
        };
        let built_in = built_in();
        let default = match p.default {
            Some(d) => policy(d.ue_ambr_mbps, &d.sessions, &built_in).map_err(|e| format!("default policy: {e}"))?,
            None => built_in,
        };
        let mut policies = Policies { default, by_supi: HashMap::new() };
        for value in p.policies {
            for (supi, entry) in policies.entry(value)? {
                if policies.by_supi.insert(supi.clone(), entry).is_some() {
                    return Err(format!("policy of {supi}: listed twice"));
                }
            }
        }
        Ok(policies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(toml_like: serde_json::Value) -> Result<Policies, String> {
        Policies::from_params(&toml_like)
    }

    #[test]
    fn subscribers_get_their_policy_the_others_the_default() {
        let p = parse(serde_json::json!({
            "default": { "ue_ambr_mbps": { "uplink": 100, "downlink": 500 } },
            "policies": [{
                "supi": "imsi-999700000000001", "count": 2,
                "ue_ambr_mbps": { "uplink": 50, "downlink": 200 },
                "sessions": [
                    { "dnn": "ims", "five_qi": 5, "ambr_mbps": { "uplink": 1, "downlink": 2 },
                      "flows": [{ "five_qi": 1, "mbr_mbps": { "uplink": 1, "downlink": 1 },
                                  "gbr_mbps": { "uplink": 1, "downlink": 1 },
                                  "filters": [{ "remote": "10.10.0.0/16", "protocol": 17, "remote_ports": "5060-5061" }] }] },
                    { "ambr_mbps": { "uplink": 20, "downlink": 100 } }
                ]
            }]
        }))
        .unwrap();
        let mine = p.of("imsi-999700000000002");
        assert_eq!(mine.ue_ambr, Rates { uplink: 50_000_000, downlink: 200_000_000 }, "the range covers 002");
        let ims = mine.session("IMS").unwrap();
        assert_eq!((ims.five_qi, ims.arp, ims.ambr.downlink), (5, 8, 2_000_000));
        let flow = ims.rules()[0];
        assert_eq!((flow.five_qi, flow.precedence, flow.gbr.uplink), (1, 10, 1_000_000));
        let f = flow.filters()[0];
        assert_eq!(
            (f.direction, f.protocol, f.remote, f.remote_prefix, f.remote_ports),
            (3, 17, [10, 10, 0, 0], 16, [5060, 5061])
        );
        let internet = mine.session("internet").unwrap();
        assert_eq!((internet.five_qi, internet.ambr.uplink, internet.rules().len()), (9, 20_000_000, 0), "any DNN");
        let other = p.of("imsi-999700000000009");
        assert_eq!(other.ue_ambr.downlink, 500_000_000, "the default");
        assert_eq!(other.session("internet").unwrap().ambr.downlink, 1_000_000_000, "built in");
        assert_eq!(p.by_supi["imsi-999700000000002"].source["supi"], "imsi-999700000000002");
    }

    #[test]
    fn bad_policies_are_refused() {
        let bad = |session: serde_json::Value| {
            parse(serde_json::json!({ "policies": [{ "supi": "imsi-1", "sessions": [session] }] })).unwrap_err()
        };
        let filters = serde_json::json!([{ "protocol": 6 }]);
        assert!(bad(serde_json::json!({ "five_qi": 1 })).contains("GBR 5QI"));
        assert!(bad(serde_json::json!({ "flows": [{ "five_qi": 1, "filters": filters }] })).contains("gbr_mbps"));
        let gbr_on_non_gbr = serde_json::json!({ "flows": [{ "five_qi": 9, "gbr_mbps": { "uplink": 1, "downlink": 1 }, "filters": filters }] });
        assert!(bad(gbr_on_non_gbr).contains("only for a GBR 5QI"));
        assert!(bad(serde_json::json!({ "flows": [{ "five_qi": 9, "filters": [] }] })).contains("1 to 4 filters"));
        let port = serde_json::json!({ "flows": [{ "five_qi": 9, "filters": [{ "remote_ports": "9-1" }] }] });
        assert!(bad(port).contains("remote_ports"));
        let five = serde_json::json!({ "five_qi": 9, "filters": filters });
        assert!(
            bad(serde_json::json!({ "flows": [five.clone(), five.clone(), five.clone(), five.clone(), five] }))
                .contains("4 at most")
        );
        let same = serde_json::json!({ "five_qi": 9, "precedence": 5, "filters": filters });
        assert!(bad(serde_json::json!({ "flows": [same.clone(), same] })).contains("two flows with precedence 5"));
        let last = serde_json::json!({ "five_qi": 9, "precedence": 255, "filters": filters });
        assert!(bad(serde_json::json!({ "flows": [last] })).contains("the default rule's"));
        assert!(parse(serde_json::json!({ "policies": [{ "supi": "nai-x" }] })).unwrap_err().contains("imsi-"));
    }

    #[test]
    fn flows_are_kept_in_precedence_order() {
        let flow = |precedence: u8, protocol: u8| serde_json::json!({ "five_qi": 9, "precedence": precedence, "filters": [{ "protocol": protocol }] });
        let p = parse(
            serde_json::json!({ "default": { "sessions": [{ "flows": [flow(30, 6), flow(10, 17), flow(20, 1)] }] } }),
        )
        .unwrap();
        let rules = p.default.session("internet").unwrap().rules();
        let order: Vec<_> = rules.iter().map(|r| (r.precedence, r.filters()[0].protocol)).collect();
        assert_eq!(order, [(10, 17), (20, 1), (30, 6)]);
    }
}
