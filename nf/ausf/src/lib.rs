//! AUSF, the authentication server function, hosted in one EO.
//!
//! This release serves Nausf_UEAuthentication (TS 33.501 §6.1.3): the AMF's
//! request goes to the UDM for a vector, of the method the UDM chose for the
//! subscriber.
//! - 5G AKA (§6.1.3.2): the AMF gets the challenge with HXRES*; XRES*,
//!   K_SEAF and the SUPI stay here until the AMF confirms with the UE's RES*.
//! - EAP-AKA' (§6.1.3.1): the AMF gets an EAP-Request/AKA'-Challenge for the
//!   UE, and confirms with the UE's EAP-Response; checked (AT_MAC, RES), it
//!   gets K_SEAF, the SUPI and EAP-Success, else EAP-Failure.
//!
//! Contexts expire [`LIFETIME`] after the challenge, and are bounded by
//! `max_contexts`.

use std::collections::HashMap;
use std::time::Duration;

use librecn_common::Service;
use librecn_common::log;
use librecn_common::log::Logger;
use librecn_common::msg::flags;
use librecn_sbi::ue_authentication::{
    AuthData, Authenticate, AuthenticateResult, Confirm, ConfirmResult, GetAuthData, method,
};
use librecn_sbi::{Outcome, ServingNetwork, Supi};
use librecn_scheduler::{Context, EoConfig, EoId, ExecutionObject, Message, Time};
use librecn_security::{eap_aka, kdf};
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
    check: Check,
    kseaf: [u8; 32],
    since: Time,
}

/// What the UE's answer is checked against.
enum Check {
    FiveGAka {
        xres_star: [u8; 16],
    },
    /// The EAP identifier of the exchange, K_aut for AT_MAC, XRES.
    EapAkaPrime {
        id: u8,
        k_aut: [u8; 32],
        xres: [u8; 16],
        xres_len: usize,
    },
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
        let id = msg.corr_id();
        if data.method == method::EAP_AKA_PRIME {
            // The SUPI's digits, the identity in MK (as the UE takes it).
            let identity = data.supi.imsi().unwrap_or_default();
            let keys = eap_aka::keys(&data.ck_prime, &data.ik_prime, identity);
            let eap_id = id as u8;
            let mut request = [0u8; eap_aka::MAX];
            let Some(n) = eap_aka::challenge(eap_id, &data.rand, &data.autn, snn, &keys.k_aut, &mut request) else {
                log!(self.log, ctx, Error, "serving network name {snn:?} too long for EAP-AKA'");
                return self.challenge(ctx, p.amf, p.corr_id, Outcome::Unavailable, |_| {});
            };
            let xres_len = usize::from(data.xres_len).min(16);
            let check = Check::EapAkaPrime { id: eap_id, k_aut: keys.k_aut, xres: data.xres, xres_len };
            let kseaf = kdf::kseaf(&keys.kausf, snn);
            self.challenges.insert(id, Challenge { amf: p.amf, supi: data.supi, check, kseaf, since: ctx.now() });
            return self.challenge(ctx, p.amf, p.corr_id, Outcome::Ok, |r| {
                (r.auth_ctx, r.method) = (id, method::EAP_AKA_PRIME);
                r.eap.set(&request[..n]);
            });
        }
        let hxres_star = kdf::hres_star(&data.rand, &data.xres_star);
        let challenge = Challenge {
            amf: p.amf,
            supi: data.supi,
            check: Check::FiveGAka { xres_star: data.xres_star },
            kseaf: kdf::kseaf(&data.kausf, snn),
            since: ctx.now(),
        };
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
        // The UE's answer checked: None if it passed, else why not; the EAP result.
        let (failure, eap) = match challenge.as_ref().map(|c| &c.check) {
            None => (Some("unknown context".to_string()), [0; 4]),
            Some(Check::FiveGAka { xres_star }) if *xres_star != confirm.res_star => {
                (Some("RES* differs".into()), [0; 4])
            }
            Some(Check::FiveGAka { .. }) => (None, [0; 4]),
            Some(Check::EapAkaPrime { id, k_aut, xres, xres_len }) => {
                match eap_aka::response(confirm.eap.get(), *id, k_aut) {
                    Ok(eap_aka::Response::Challenge { res }) if res == &xres[..*xres_len] => {
                        (None, eap_aka::success(*id))
                    }
                    Ok(eap_aka::Response::Challenge { .. }) => (Some("RES differs".into()), eap_aka::failure(*id)),
                    Ok(other) => (Some(format!("the UE answered {other:?}")), eap_aka::failure(*id)),
                    Err(e) => (Some(format!("EAP-AKA' response refused: {e:?}")), eap_aka::failure(*id)),
                }
            }
        };
        let outcome = match (&challenge, &failure) {
            (None, _) => Outcome::ContextNotFound,
            (Some(_), Some(_)) => Outcome::AuthenticationRejected,
            (Some(_), None) => Outcome::Ok,
        };
        match (&challenge, &failure) {
            (Some(c), None) => log!(self.log, ctx, Info, "{} authenticated", c.supi),
            (Some(c), Some(why)) => log!(self.log, ctx, Info, "{} failed authentication: {why}", c.supi),
            (None, _) => log!(self.log, ctx, Info, "confirmation for unknown context {}", confirm.auth_ctx),
        }
        let sent = ctx.alloc::<ConfirmResult>(msg.from()).and_then(|r| {
            let mut r = r.with_corr_id(msg.corr_id()).with_flags(flags::RESPONSE);
            r.set_outcome(outcome);
            r.eap = eap;
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
