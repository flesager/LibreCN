//! UDM, the unified data management, hosted in one EO.
//!
//! This release serves the UE authentication service to the AUSF
//! (Nudm_UEAuthentication_Get, 5G AKA): SUCI de-concealment (null scheme,
//! ECIES Profile A with the home network keys of the deployment) and
//! authentication vectors from Milenage, with SQN resynchronisation.
//! Subscribers come from the deployment `params` ([`config`]) and from the
//! web API while it runs ([`manage`]).
//!
//! - [`aka`], the service on domain values;
//! - [`config`], the subscribers;
//! - [`manage`], subscribers on the web API;
//! - the EO, thin glue.

pub mod aka;
pub mod config;
pub mod manage;

use std::collections::HashMap;

use librecn_common::log;
use librecn_common::log::Logger;
use librecn_common::manage::{Manage, send_reply};
use librecn_common::msg::flags;
use librecn_sbi::Outcome;
use librecn_sbi::ue_authentication::{AuthData, AuthInfo, GetAuthData};
use librecn_scheduler::{Context, EoConfig, ExecutionObject, Message};

use config::{HomeNetworkKeys, Subscriber};

/// The UDM EO.
pub struct Udm {
    name: String,
    subscribers: HashMap<String, Subscriber>,
    keys: HomeNetworkKeys,
    log: Logger,
}

impl Udm {
    pub fn new(name: &str, subscribers: HashMap<String, Subscriber>, keys: HomeNetworkKeys) -> Self {
        Udm { name: name.to_string(), subscribers, keys, log: Logger::default() }
    }

    /// For the control EO's factory: EO sizes and the UDM from deployment `params`.
    pub fn from_params(name: &str, params: &serde_json::Value) -> Result<(EoConfig, Self), String> {
        let udm = Self::new(name, config::from_params(params)?, config::keys_from_params(params)?);
        Ok((Self::eo_config(name), udm))
    }

    /// Requests are 256 bytes; management requests 8 KiB.
    pub fn eo_config(name: &str) -> EoConfig {
        EoConfig::new(name).max_timers(0).buffers(&[(256, 1024), (8 * 1024, 4)])
    }

    fn on_manage(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &Manage) {
        let Some(method) = request.method() else { return };
        let answer = manage::handle(&mut self.subscribers, method, request.path(), request.body());
        if let Some(change) = &answer.change {
            log!(self.log, ctx, Info, "{change}, {} subscriber(s)", self.subscribers.len());
        }
        if let Err(e) = send_reply(ctx, msg, answer.status, &answer.json) {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", msg.from());
        }
    }

    fn get_auth_data(&mut self, info: &AuthInfo) -> Result<(librecn_sbi::Supi, aka::Vector), Outcome> {
        let supi = aka::deconceal(info.supi_or_suci.as_str(), &self.keys)?;
        let sub = self.subscribers.get_mut(supi.as_str()).ok_or(Outcome::UserNotFound)?;
        if let Some((rand, auts)) = info.resync()
            && !aka::resynchronise(sub, rand, auts)
        {
            return Err(Outcome::AuthenticationRejected);
        }
        let rand = librecn_security::random().map_err(|_| Outcome::Unavailable)?;
        Ok((supi, aka::vector(sub, &info.serving_network, rand)))
    }

    fn on_get_auth_data(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &GetAuthData) {
        let info = &request.info;
        let answer = self.get_auth_data(info);
        match &answer {
            Ok((supi, _)) => {
                let resync = if info.resync().is_some() { ", resynchronised" } else { "" };
                let sqn = self.subscribers.get(supi.as_str()).map_or(0, |s| s.sqn);
                // The SUCI's protection scheme: its sixth field.
                let concealed = match info.supi_or_suci.as_str().split('-').nth(5) {
                    Some("1") => ", SUCI of Profile A",
                    _ => "",
                };
                log!(self.log, ctx, Info, "authentication vector for {supi} (SQN {sqn:#x}{resync}{concealed})");
            }
            Err(outcome) => {
                log!(self.log, ctx, Info, "no authentication vector for {}: {outcome:?}", info.supi_or_suci)
            }
        }
        let sent = ctx.alloc::<AuthData>(msg.from()).and_then(|reply| {
            let mut reply = reply.with_corr_id(msg.corr_id()).with_flags(flags::RESPONSE);
            match answer {
                Ok((supi, v)) => {
                    (reply.supi, reply.rand, reply.autn, reply.xres_star, reply.kausf) =
                        (supi, v.rand, v.autn, v.xres_star, v.kausf);
                }
                Err(outcome) => reply.set_outcome(outcome),
            }
            ctx.send(reply)
        });
        if let Err(e) = sent {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", msg.from());
        }
    }
}

impl ExecutionObject for Udm {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        log!(self.log, ctx, Vip, "UDM {} ready, {} subscriber(s)", self.name, self.subscribers.len());
        let mut ids: Vec<_> = self.keys.keys().copied().collect();
        ids.sort();
        for id in ids {
            let public: String =
                librecn_security::ecies::public_key(&self.keys[&id]).iter().map(|b| format!("{b:02x}")).collect();
            log!(self.log, ctx, Info, "SUCI home network key {id} (Profile A), public key {public}");
        }
    }

    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if let Some(request) = msg.body::<GetAuthData>() {
            self.on_get_auth_data(ctx, msg, request);
        } else if let Some(request) = msg.body::<Manage>() {
            self.on_manage(ctx, msg, request);
        }
    }
}
