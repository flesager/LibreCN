//! UDM, the unified data management, hosted in one EO.
//!
//! This release serves the UE authentication service to the AUSF
//! (Nudm_UEAuthentication_Get, 5G AKA): SUCI de-concealment (null scheme,
//! ECIES Profile A with the home network keys of the deployment) and
//! authentication vectors from Milenage, with SQN resynchronisation.
//! Subscribers come from the deployment `params` ([`config`]) and from the
//! web API while it runs ([`manage`]); the web API's changes and the SQNs
//! survive restarts in the state file ([`persist`], `[oam] state_dir`).
//!
//! - [`aka`], the service on domain values;
//! - [`config`], the subscribers;
//! - [`manage`], subscribers on the web API;
//! - the EO, thin glue.

pub mod aka;
pub mod config;
pub mod manage;
pub mod persist;

use std::collections::{BTreeSet, HashMap};

use librecn_common::log;
use librecn_common::log::Logger;
use librecn_common::manage::{Manage, send_reply};
use librecn_common::msg::flags;
use librecn_common::state::{SNAPSHOT_PERIOD, Store};
use librecn_sbi::Outcome;
use librecn_sbi::ue_authentication::{AuthData, AuthInfo, GetAuthData};
use librecn_scheduler::{Context, EoConfig, ExecutionObject, Message, TimerId};

use config::{HomeNetworkKeys, Subscriber};

/// The UDM EO.
pub struct Udm {
    name: String,
    subscribers: HashMap<String, Subscriber>,
    keys: HomeNetworkKeys,
    /// The configuration's subscribers the web API removed.
    removed: BTreeSet<String>,
    /// The state file, what it restored, and whether there is news for it.
    store: Option<(Store, persist::Restored)>,
    dirty: bool,
    snapshot_timer: Option<TimerId>,
    log: Logger,
}

impl Udm {
    pub fn new(name: &str, subscribers: HashMap<String, Subscriber>, keys: HomeNetworkKeys) -> Self {
        Udm {
            name: name.to_string(),
            subscribers,
            keys,
            removed: BTreeSet::new(),
            store: None,
            dirty: false,
            snapshot_timer: None,
            log: Logger::default(),
        }
    }

    /// Keeps the state in `store`, restoring what it holds.
    pub fn with_store(mut self, store: Store) -> Result<Self, String> {
        let restored = match store.load().map_err(|e| format!("state {}: {e}", store.path().display()))? {
            Some(text) => persist::restore(&text, &mut self.subscribers, &mut self.removed)
                .map_err(|e| format!("state {}: {e}", store.path().display()))?,
            None => persist::Restored { provisioned: 0, removed: 0, sqns: 0 },
        };
        self.store = Some((store, restored));
        Ok(self)
    }

    /// For the control EO's factory: EO sizes and the UDM from deployment
    /// `params`, its state restored from the state directory if there is one.
    pub fn from_params(name: &str, params: &serde_json::Value) -> Result<(EoConfig, Self), String> {
        let udm = Self::new(name, config::from_params(params)?, config::keys_from_params(params)?);
        let udm = match Store::for_nf(name) {
            Some(store) => udm.with_store(store)?,
            None => udm,
        };
        Ok((Self::eo_config(name), udm))
    }

    /// Requests are 256 bytes; management requests 8 KiB; the snapshot timer.
    pub fn eo_config(name: &str) -> EoConfig {
        EoConfig::new(name).max_timers(1).buffers(&[(256, 1024), (8 * 1024, 4)])
    }

    /// Writes the state file if something changed.
    fn snapshot(&mut self, ctx: &mut Context<'_>) {
        let Some((store, _)) = &self.store else { return };
        if !std::mem::take(&mut self.dirty) {
            return;
        }
        if let Err(e) = store.save(&persist::snapshot(&self.subscribers, &self.removed)) {
            self.dirty = true;
            log!(self.log, ctx, Error, "state {} not written: {e}", store.path().display());
        }
    }

    fn on_manage(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &Manage) {
        let Some(method) = request.method() else { return };
        let answer = manage::handle(&mut self.subscribers, &mut self.removed, method, request.target(), request.body());
        if let Some(change) = &answer.change {
            self.dirty = true;
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
        self.dirty = true;
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
                    (reply.supi, reply.rand, reply.autn) = (supi, v.rand, v.autn);
                    match v.method {
                        aka::Method::FiveGAka { xres_star, kausf } => {
                            (reply.xres_star, reply.kausf) = (xres_star, kausf)
                        }
                        aka::Method::EapAkaPrime { xres, ck_prime, ik_prime } => {
                            reply.method = librecn_sbi::ue_authentication::method::EAP_AKA_PRIME;
                            reply.xres[..8].copy_from_slice(&xres);
                            (reply.xres_len, reply.ck_prime, reply.ik_prime) = (8, ck_prime, ik_prime);
                        }
                    }
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
        if let Some((store, r)) = &self.store {
            let path = store.path().display();
            log!(
                self.log,
                ctx,
                Info,
                "state in {path}: {} subscriber(s) added and {} removed by the web API, {} SQN(s) restored",
                r.provisioned,
                r.removed,
                r.sqns
            );
            self.snapshot_timer = ctx.arm_timer(SNAPSHOT_PERIOD, 0).ok();
        }
    }

    fn on_stop(&mut self, ctx: &mut Context<'_>) {
        self.snapshot(ctx);
    }

    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if msg.timer().is_some() {
            self.snapshot(ctx);
            self.snapshot_timer = ctx.arm_timer(SNAPSHOT_PERIOD, 0).ok();
        } else if let Some(request) = msg.body::<GetAuthData>() {
            self.on_get_auth_data(ctx, msg, request);
        } else if let Some(request) = msg.body::<Manage>() {
            self.on_manage(ctx, msg, request);
        }
    }
}
