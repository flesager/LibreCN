//! AUSF, the authentication server function, hosted in one EO.
//!
//! This release serves Nausf_UEAuthentication for 5G AKA (TS 33.501
//! §6.1.3.2): the AMF's request goes to the UDM for a vector; the AMF gets
//! the challenge with HXRES*; XRES*, K_SEAF and the SUPI stay here until the
//! AMF confirms with the UE's RES*.
//!
//! Contexts expire [`LIFETIME`] after the challenge, and are bounded by
//! `max_contexts`.

use std::collections::HashMap;
use std::time::Duration;

use librecn_common::Service;
use librecn_common::log;
use librecn_common::log::Logger;
use librecn_common::msg::flags;
use librecn_sbi::ue_authentication::{AuthData, Authenticate, AuthenticateResult, Confirm, ConfirmResult, GetAuthData};
use librecn_sbi::{Outcome, ServingNetwork, Supi};
use librecn_scheduler::{Context, EoConfig, EoId, ExecutionObject, Message, Time};
use librecn_security::kdf;
use serde::Deserialize;

/// How long a challenge waits for its confirmation: past the UE's
/// authentication timers (T3560 × 5 retransmissions, TS 24.501 §10.2).
pub const LIFETIME: Duration = Duration::from_secs(30);
/// Expired contexts are swept at this period.
const SWEEP: Duration = Duration::from_secs(5);

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Params {
    /// The UDM's EO name.
    udm: String,
    max_contexts: usize,
}

impl Default for Params {
    fn default() -> Self {
        Params { udm: "udm".into(), max_contexts: 1024 }
    }
}

/// A request waiting for the UDM.
struct Pending {
    amf: EoId,
    corr_id: u32,
    serving_network: ServingNetwork,
    since: Time,
}

/// A challenge waiting for the AMF's confirmation.
struct Challenge {
    amf: EoId,
    supi: Supi,
    xres_star: [u8; 16],
    kseaf: [u8; 32],
    since: Time,
}

/// The AUSF EO.
pub struct Ausf {
    name: String,
    udm: Service,
    max_contexts: usize,
    next_id: u32,
    pending: HashMap<u32, Pending>,
    challenges: HashMap<u32, Challenge>,
    log: Logger,
}

impl Ausf {
    /// From deployment `params`: `udm` (the UDM's name, default "udm") and
    /// `max_contexts` (default 1024).
    pub fn new(name: &str, params: &serde_json::Value) -> Result<Self, String> {
        let p: Params = if params.is_null() {
            Params::default()
        } else {
            serde_json::from_value(params.clone()).map_err(|e| format!("invalid AUSF parameters: {e}"))?
        };
        Ok(Ausf {
            name: name.to_string(),
            udm: Service::named(p.udm),
            max_contexts: p.max_contexts,
            next_id: 1,
            pending: HashMap::with_capacity(p.max_contexts),
            challenges: HashMap::with_capacity(p.max_contexts),
            log: Logger::default(),
        })
    }

    /// For the control EO's factory.
    pub fn from_params(name: &str, params: &serde_json::Value) -> Result<(EoConfig, Self), String> {
        Ok((Self::eo_config(name), Self::new(name, params)?))
    }

    /// Requests are at most 256 bytes; one sweep timer.
    pub fn eo_config(name: &str) -> EoConfig {
        EoConfig::new(name).max_timers(1).buffers(&[(256, 1024)])
    }

    fn next_id(&mut self) -> u32 {
        loop {
            let id = self.next_id;
            self.next_id = self.next_id.wrapping_add(1).max(1);
            if !self.pending.contains_key(&id) && !self.challenges.contains_key(&id) {
                return id;
            }
        }
    }

    fn on_authenticate(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &Authenticate) {
        let info = &request.info;
        let refuse = |this: &mut Self, ctx: &mut Context<'_>, outcome: Outcome| {
            log!(this.log, ctx, Info, "authentication of {} refused: {outcome:?}", info.supi_or_suci);
            this.challenge(ctx, msg.from(), msg.corr_id(), outcome, |_| {});
        };
        // A serving network name is "5G:" then the network identifier (TS 24.501 §9.12.1).
        if !info.serving_network.as_str().starts_with("5G:") {
            return refuse(self, ctx, Outcome::ServingNetworkNotAuthorized);
        }
        if self.pending.len() + self.challenges.len() >= self.max_contexts {
            return refuse(self, ctx, Outcome::Unavailable);
        }
        let Some(udm) = self.udm.get(ctx) else {
            log!(self.log, ctx, Error, "no UDM {:?}", self.udm.name());
            return refuse(self, ctx, Outcome::Unavailable);
        };
        let id = self.next_id();
        let sent = ctx.alloc::<GetAuthData>(udm).and_then(|m| {
            let mut m = m.with_corr_id(id).with_flags(flags::REQUEST);
            m.info = *info;
            ctx.send(m)
        });
        match sent {
            Ok(()) => {
                let pending = Pending {
                    amf: msg.from(),
                    corr_id: msg.corr_id(),
                    serving_network: info.serving_network,
                    since: ctx.now(),
                };
                self.pending.insert(id, pending);
            }
            Err(e) => {
                self.udm.forget();
                log!(self.log, ctx, Error, "cannot reach the UDM: {e}");
                refuse(self, ctx, Outcome::Unavailable);
            }
        }
    }

    fn on_auth_data(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, data: &AuthData) {
        let Some(p) = self.pending.remove(&msg.corr_id()) else {
            return log!(self.log, ctx, Debug, "late UDM answer {} dropped", msg.corr_id());
        };
        if data.outcome() != Outcome::Ok {
            return self.challenge(ctx, p.amf, p.corr_id, data.outcome(), |_| {});
        }
        let snn = p.serving_network.as_str();
        let hxres_star = kdf::hres_star(&data.rand, &data.xres_star);
        let challenge = Challenge {
            amf: p.amf,
            supi: data.supi,
            xres_star: data.xres_star,
            kseaf: kdf::kseaf(&data.kausf, snn),
            since: ctx.now(),
        };
        let id = msg.corr_id();
        self.challenges.insert(id, challenge);
        self.challenge(ctx, p.amf, p.corr_id, Outcome::Ok, |r| {
            (r.auth_ctx, r.rand, r.autn, r.hxres_star) = (id, data.rand, data.autn, hxres_star);
        });
    }

    fn challenge(
        &mut self,
        ctx: &mut Context<'_>,
        amf: EoId,
        corr_id: u32,
        outcome: Outcome,
        fill: impl FnOnce(&mut AuthenticateResult),
    ) {
        let sent = ctx.alloc::<AuthenticateResult>(amf).and_then(|r| {
            let mut r = r.with_corr_id(corr_id).with_flags(flags::RESPONSE);
            r.set_outcome(outcome);
            fill(&mut r);
            ctx.send(r)
        });
        if let Err(e) = sent {
            log!(self.log, ctx, Error, "cannot answer {amf}: {e}");
        }
    }

    fn on_confirm(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, confirm: &Confirm) {
        let challenge = self.challenges.remove(&confirm.auth_ctx).filter(|c| c.amf == msg.from());
        let outcome = match &challenge {
            None => Outcome::ContextNotFound,
            Some(c) if c.xres_star != confirm.res_star => Outcome::AuthenticationRejected,
            Some(_) => Outcome::Ok,
        };
        match &challenge {
            Some(c) if outcome == Outcome::Ok => log!(self.log, ctx, Info, "{} authenticated", c.supi),
            Some(c) => log!(self.log, ctx, Info, "{} failed authentication: RES* differs", c.supi),
            None => log!(self.log, ctx, Info, "confirmation for unknown context {}", confirm.auth_ctx),
        }
        let sent = ctx.alloc::<ConfirmResult>(msg.from()).and_then(|r| {
            let mut r = r.with_corr_id(msg.corr_id()).with_flags(flags::RESPONSE);
            r.set_outcome(outcome);
            if let Some(c) = challenge.filter(|_| outcome == Outcome::Ok) {
                (r.supi, r.kseaf) = (c.supi, c.kseaf);
            }
            ctx.send(r)
        });
        if let Err(e) = sent {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", msg.from());
        }
    }

    fn sweep(&mut self, now: Time) -> usize {
        let before = self.pending.len() + self.challenges.len();
        let live = |since: Time| now < since + LIFETIME;
        self.pending.retain(|_, p| live(p.since));
        self.challenges.retain(|_, c| live(c.since));
        before - self.pending.len() - self.challenges.len()
    }
}

impl ExecutionObject for Ausf {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        if let Err(e) = ctx.arm_timer(SWEEP, 0) {
            log!(self.log, ctx, Error, "no sweep timer, contexts will not expire: {e}");
        }
        log!(self.log, ctx, Vip, "AUSF {} ready, UDM {:?}", self.name, self.udm.name());
    }

    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if msg.timer().is_some() {
            let expired = self.sweep(ctx.now());
            if expired > 0 {
                log!(self.log, ctx, Info, "{expired} authentication context(s) expired");
            }
            let _ = ctx.arm_timer(SWEEP, 0);
        } else if let Some(request) = msg.body::<Authenticate>() {
            self.on_authenticate(ctx, msg, request);
        } else if let Some(data) = msg.body::<AuthData>() {
            self.on_auth_data(ctx, msg, data);
        } else if let Some(confirm) = msg.body::<Confirm>() {
            self.on_confirm(ctx, msg, confirm);
        }
    }
}
