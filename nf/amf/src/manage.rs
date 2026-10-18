//! The AMF's resources on the web API (`/api/nfs/{amf}/...`, a `Manage`
//! request from the server):
//! - `GET gnbs`: the gNBs connected (association, key, id, name, tracking
//!   areas), set up or not yet, with their cells and UEs: connected,
//!   inactive (RRC_INACTIVE), idle (their last cell);
//! - `GET ues`: how many UEs, connected and idle, and the first [`LISTED`],
//!   each with its RRC state and cell; `ues?gnb={assoc}`: that gNB's;
//! - `GET ues/{supi}`: one UE;
//! - `DELETE ues/{supi}`, body `{"reregister": true}` optional: the network
//!   deregisters the UE (TS 23.502 §4.2.2.3.3). A connected UE is told
//!   (Deregistration Request); an idle one is deregistered locally;
//! - `POST ues/{supi}/guti`: a new 5G-GUTI for the connected UE
//!   (Configuration Update Command, TS 24.501 §5.4.4);
//! - `POST capacity`, body `{"relative_capacity"}`: the AMF's relative
//!   capacity, announced to every gNB (AMF Configuration Update).

use librecn_common::log;
use librecn_common::manage::{Manage, Method, send_reply};
use librecn_scheduler::{Context, Message};
use serde_json::{Value, json};

use std::collections::{BTreeMap, HashMap};

use crate::sctp::AssocId;
use crate::setup::RanNodeId;
use crate::ue::{Location, UeContext};
use crate::{Amf, n2};

/// UEs listed by `GET ues`.
const LISTED: usize = 64;

/// A cell as the web API shows it: its NR Cell Identity, 9 hex digits.
fn cell(location: Option<Location>) -> Option<String> {
    location.map(|l| format!("{:09x}", l.cell.1))
}

fn connected(ue: &UeContext) -> Value {
    json!({
        "supi": ue.mm.supi().map(|s| s.as_str()),
        "cm": "connected",
        "rrc": if ue.inactive { "inactive" } else { "connected" },
        "cell": cell(ue.location),
        "state": format!("{:?}", ue.mm.state()),
        "amf_ue_ngap_id": ue.amf_id,
        "ran_ue_ngap_id": ue.ran_id,
        "assoc": ue.assoc,
        "tmsi": ue.mm.tmsi().map(|t| format!("{t:08x}")),
        "sessions": ue.sessions.iter().enumerate().filter(|(_, s)| s.is_some()).map(|(psi, _)| psi).collect::<Vec<_>>(),
    })
}

impl Amf {
    /// The gNBs, each with its cells and their UEs by RRC state.
    fn gnbs_view(&self) -> Value {
        // Per association, per cell (u64::MAX: not known): connected, inactive, idle.
        let mut cells: HashMap<AssocId, BTreeMap<u64, [usize; 3]>> = HashMap::new();
        for ue in self.ues.iter() {
            let cell = ue.location.map_or(u64::MAX, |l| l.cell.1);
            cells.entry(ue.assoc).or_default().entry(cell).or_default()[usize::from(ue.inactive)] += 1;
        }
        for location in self.idle.values().filter_map(|i| i.location) {
            let under = self.gnbs.iter().find(|(_, g)| g.node.as_ref().is_some_and(|(n, _)| n.has_cell(location.cell)));
            if let Some((assoc, _)) = under {
                cells.entry(*assoc).or_default().entry(location.cell.1).or_default()[2] += 1;
            }
        }
        let mut gnbs: Vec<_> = self.gnbs.iter().collect();
        gnbs.sort_by_key(|(assoc, _)| **assoc);
        let listed: Vec<Value> = gnbs
            .into_iter()
            .map(|(assoc, g)| {
                let tas: Vec<String> = g
                    .tas
                    .iter()
                    .map(|(plmn, tac)| format!("{plmn} TAC {}", u32::from_be_bytes([0, tac[0], tac[1], tac[2]])))
                    .collect();
                let mine = cells.remove(assoc).unwrap_or_default();
                let total = |i: usize| mine.values().map(|c| c[i]).sum::<usize>();
                let (connected, inactive, idle) = (total(0), total(1), total(2));
                let cells: Vec<Value> = mine
                    .iter()
                    .map(|(cell, c)| {
                        json!({ "cell": (*cell != u64::MAX).then(|| format!("{cell:09x}")),
                                "connected": c[0], "inactive": c[1], "idle": c[2] })
                    })
                    .collect();
                let bits = match g.node.as_ref().map(|(n, _)| *n) {
                    Some(RanNodeId::Gnb { bits, .. }) => Some(bits),
                    _ => None,
                };
                json!({ "assoc": assoc, "key": g.node.as_ref().map(|(n, _)| n.key()), "bits": bits,
                        "id": g.node.as_ref().map(|(id, _)| id.to_string()),
                        "name": g.node.as_ref().and_then(|(_, name)| name.as_deref()), "tas": tas,
                        "ues": connected + inactive, "connected": connected, "inactive": inactive, "idle": idle,
                        "cells": cells })
            })
            .collect();
        json!(listed)
    }

    pub(crate) fn on_manage(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &Manage) {
        let path = request.path();
        let (status, answer) = match (request.method(), path.split_once('/').unwrap_or((path, ""))) {
            (Some(Method::Get), ("gnbs", "")) => (200, self.gnbs_view()),
            (Some(Method::Get), ("ues", "")) => {
                // All UEs, or one gNB's: connected through it, or idle last under it.
                let gnb = request.query("gnb").and_then(|a| a.parse::<AssocId>().ok());
                let node = gnb.and_then(|a| self.gnbs.get(&a)).and_then(|g| g.node.as_ref()).map(|(n, _)| *n);
                let under = |l: Option<Location>| l.is_some_and(|l| node.is_some_and(|n| n.has_cell(l.cell)));
                let idle = self
                    .idle
                    .iter()
                    .filter(|(_, i)| gnb.is_none() || under(i.location))
                    .map(|(tmsi, idle)| idle.describe(*tmsi, ctx.now()));
                let all: Vec<Value> =
                    self.ues.iter().filter(|ue| gnb.is_none_or(|a| ue.assoc == a)).map(connected).chain(idle).collect();
                let counts = (self.ues.len(), self.ues.registered(), self.idle.len());
                let count = all.len();
                let listed: Vec<Value> = all.into_iter().take(LISTED).collect();
                (
                    200,
                    json!({ "connected": counts.0, "registered": counts.1, "idle": counts.2,
                              "count": count, "ues": listed }),
                )
            }
            (Some(Method::Get), ("ues", supi)) => {
                let found = self.ues.iter().find(|ue| ue.mm.supi().is_some_and(|s| s.as_str() == supi)).map(connected);
                let found = found.or_else(|| {
                    let (tmsi, idle) = self.idle.iter().find(|(_, i)| i.supi().is_some_and(|s| s.as_str() == supi))?;
                    Some(idle.describe(*tmsi, ctx.now()))
                });
                match found {
                    Some(ue) => (200, ue),
                    None => (404, json!({ "error": format!("no UE {supi:?}") })),
                }
            }
            (Some(Method::Post), ("ues", rest)) if rest.ends_with("/guti") => {
                self.reallocate_guti(ctx, &rest[..rest.len() - "/guti".len()])
            }
            (Some(Method::Post), ("capacity", "")) => {
                let body: Value = serde_json::from_str(request.body()).unwrap_or(Value::Null);
                match body.get("relative_capacity").and_then(Value::as_u64).and_then(|c| u8::try_from(c).ok()) {
                    Some(capacity) => self.announce_capacity(ctx, capacity),
                    None => (400, json!({ "error": "relative_capacity: 0 to 255" })),
                }
            }
            (Some(Method::Delete), ("ues", supi)) => {
                let body: Value = serde_json::from_str(request.body()).unwrap_or(Value::Null);
                let reregister = body.get("reregister").and_then(Value::as_bool).unwrap_or(false);
                self.deregister_by_network(ctx, supi, reregister)
            }
            _ => (404, json!({ "error": format!("no resource {path:?}") })),
        };
        if let Err(e) = send_reply(ctx, msg, status, &answer.to_string()) {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", msg.from());
        }
    }

    /// The AMF's new relative capacity, announced to every gNB set up.
    fn announce_capacity(&mut self, ctx: &mut Context<'_>, capacity: u8) -> (u16, Value) {
        self.config.relative_capacity = capacity;
        let update = n2::amf_configuration_update(&self.config);
        let gnbs: Vec<_> = self.gnbs.iter().filter(|(_, g)| g.node.is_some()).map(|(a, _)| *a).collect();
        for &assoc in &gnbs {
            self.send(ctx, assoc, 0, update.clone(), "AMF Configuration Update");
        }
        log!(self.log, ctx, Info, "relative capacity {capacity}: AMF Configuration Update to {} gNB(s)", gnbs.len());
        (202, json!({ "relative_capacity": capacity, "gnbs": gnbs.len() }))
    }

    /// A new 5G-GUTI for connected UE `supi`: an HTTP status and answer.
    fn reallocate_guti(&mut self, ctx: &mut Context<'_>, supi: &str) -> (u16, Value) {
        let connected = self.ues.iter().find(|ue| ue.mm.supi().is_some_and(|s| s.as_str() == supi)).map(|ue| ue.amf_id);
        let Some(amf_id) = connected else {
            let idle = self.idle.values().any(|i| i.supi().is_some_and(|s| s.as_str() == supi));
            let why = if idle { "is idle" } else { "is not connected" };
            return (if idle { 409 } else { 404 }, json!({ "error": format!("{supi} {why}") }));
        };
        let mut started = false;
        self.with_mm(ctx, amf_id, |mm, _, _, port| started = mm.reallocate_guti(port));
        if started {
            (202, json!({ "reallocating": supi }))
        } else {
            (409, json!({ "error": format!("{supi} is in another procedure") }))
        }
    }

    /// The network deregisters UE `supi`: an HTTP status and answer.
    fn deregister_by_network(&mut self, ctx: &mut Context<'_>, supi: &str, reregister: bool) -> (u16, Value) {
        let connected = self.ues.iter().find(|ue| ue.mm.supi().is_some_and(|s| s.as_str() == supi)).map(|ue| ue.amf_id);
        if let Some(amf_id) = connected {
            let mut started = false;
            self.with_mm(ctx, amf_id, |mm, _, _, port| started = mm.deregister(reregister, port));
            return if started {
                (202, json!({ "deregistering": supi, "reregister": reregister }))
            } else {
                (409, json!({ "error": format!("{supi} is in another procedure") }))
            };
        }
        let idle = self.idle.iter().find(|(_, i)| i.supi().is_some_and(|s| s.as_str() == supi)).map(|(t, _)| *t);
        match idle {
            Some(tmsi) => {
                self.forget_idle(ctx, tmsi, "deregistered by the network, locally (idle)");
                (200, json!({ "deregistered": supi }))
            }
            None => (404, json!({ "error": format!("no UE {supi:?}") })),
        }
    }
}
