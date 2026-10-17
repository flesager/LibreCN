//! The AMF's resources on the web API (`/api/nfs/{amf}/...`, a `Manage`
//! request from the server):
//! - `GET ues`: how many UEs, connected and idle, and the first [`LISTED`];
//! - `GET ues/{supi}`: one UE;
//! - `DELETE ues/{supi}`, body `{"reregister": true}` optional: the network
//!   deregisters the UE (TS 23.502 §4.2.2.3.3). A connected UE is told
//!   (Deregistration Request); an idle one is deregistered locally;
//! - `POST ues/{supi}/guti`: a new 5G-GUTI for the connected UE
//!   (Configuration Update Command, TS 24.501 §5.4.4).

use librecn_common::log;
use librecn_common::manage::{Manage, Method, send_reply};
use librecn_scheduler::{Context, Message};
use serde_json::{Value, json};

use crate::Amf;
use crate::ue::UeContext;

/// UEs listed by `GET ues`.
const LISTED: usize = 64;

fn connected(ue: &UeContext) -> Value {
    json!({
        "supi": ue.mm.supi().map(|s| s.as_str()),
        "cm": "connected",
        "state": format!("{:?}", ue.mm.state()),
        "amf_ue_ngap_id": ue.amf_id,
        "ran_ue_ngap_id": ue.ran_id,
        "tmsi": ue.mm.tmsi().map(|t| format!("{t:08x}")),
        "sessions": ue.sessions.iter().enumerate().filter(|(_, s)| s.is_some()).map(|(psi, _)| psi).collect::<Vec<_>>(),
    })
}

impl Amf {
    pub(crate) fn on_manage(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &Manage) {
        let path = request.path();
        let (status, answer) = match (request.method(), path.split_once('/').unwrap_or((path, ""))) {
            (Some(Method::Get), ("ues", "")) => {
                let idle = self.idle.iter().map(|(tmsi, idle)| idle.describe(*tmsi, ctx.now()));
                let listed: Vec<Value> = self.ues.iter().map(connected).chain(idle).take(LISTED).collect();
                let counts = (self.ues.len(), self.ues.registered(), self.idle.len());
                (200, json!({ "connected": counts.0, "registered": counts.1, "idle": counts.2, "ues": listed }))
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
