//! PCF, the policy control function, hosted in one EO.
//!
//! Policies are per subscriber, with a default one ([`config`]):
//! - AM policy (Npcf_AMPolicyControl, TS 29.507): the AMF asks at
//!   registration, and gets the UE-AMBR; when the subscriber's policy
//!   changes, the AMF is told (`AmPolicyUpdate`) and modifies the UE's
//!   context at its gNB.
//! - SM policy (Npcf_SMPolicyControl, TS 29.512): the SMF asks at each
//!   session establishment, and gets the session rule (session AMBR, the
//!   default flow's 5QI and ARP) and the PCC rules (extra QoS flows, their
//!   packet filters and bit rates) of the session's DNN. A changed policy
//!   applies to the sessions set up after it.
//!
//! - [`config`], the policies;
//! - [`manage`], the policies on the web API;
//! - [`persist`], the web API's changes kept across restarts (`[oam] state_dir`).

pub mod config;
pub mod manage;
pub mod persist;

use std::collections::{BTreeSet, HashMap};

use librecn_common::log;
use librecn_common::log::Logger;
use librecn_common::manage::{Manage, send_reply};
use librecn_common::msg::flags;
use librecn_common::state::{SNAPSHOT_PERIOD, Store};
use librecn_sbi::Supi;
use librecn_sbi::policy::{AmPolicyCreate, AmPolicyResult, AmPolicyUpdate, Rates, SmPolicyCreate, SmPolicyResult};
use librecn_scheduler::{Context, EoConfig, EoId, ExecutionObject, Message};

use config::Policies;

fn mbps(r: Rates) -> (u64, u64) {
    (r.uplink / 1_000_000, r.downlink / 1_000_000)
}

/// The PCF EO.
pub struct Pcf {
    name: String,
    policies: Policies,
    /// The configuration's policies the web API removed.
    removed: BTreeSet<String>,
    /// The AMF of each subscriber with an AM policy: told when it changes.
    amfs: HashMap<String, EoId>,
    store: Option<(Store, persist::Restored)>,
    dirty: bool,
    log: Logger,
}

impl Pcf {
    pub fn new(name: &str, policies: Policies) -> Self {
        Pcf {
            name: name.to_string(),
            policies,
            removed: BTreeSet::new(),
            amfs: HashMap::new(),
            store: None,
            dirty: false,
            log: Logger::default(),
        }
    }

    /// Keeps the web API's changes in `store`, restoring what it holds.
    pub fn with_store(mut self, store: Store) -> Result<Self, String> {
        let where_ = || store.path().display().to_string();
        let restored = match store.load().map_err(|e| format!("state {}: {e}", where_()))? {
            Some(text) => persist::restore(&text, &mut self.policies, &mut self.removed)
                .map_err(|e| format!("state {}: {e}", where_()))?,
            None => persist::Restored::default(),
        };
        self.store = Some((store, restored));
        Ok(self)
    }

    /// For the control EO's factory: EO sizes and the PCF from deployment
    /// `params`, its state restored from the state directory if there is one.
    pub fn from_params(name: &str, params: &serde_json::Value) -> Result<(EoConfig, Self), String> {
        let pcf = Self::new(name, Policies::from_params(params)?);
        let pcf = match Store::for_nf(name) {
            Some(store) => pcf.with_store(store)?,
            None => pcf,
        };
        Ok((Self::eo_config(name), pcf))
    }

    /// Requests are small; management requests 8 KiB; the snapshot timer.
    pub fn eo_config(name: &str) -> EoConfig {
        EoConfig::new(name).max_timers(1).buffers(&[(256, 1024), (8 * 1024, 4)])
    }

    fn on_am_policy(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &AmPolicyCreate) {
        let supi = request.supi.as_str();
        let ue_ambr = self.policies.of(supi).ue_ambr;
        self.amfs.insert(supi.to_string(), msg.from());
        let (ul, dl) = mbps(ue_ambr);
        log!(self.log, ctx, Info, "{supi}: AM policy, UE-AMBR {ul} / {dl} Mbit/s (up / down)");
        let sent = ctx.alloc::<AmPolicyResult>(msg.from()).and_then(|r| {
            let mut r = r.with_corr_id(msg.corr_id()).with_flags(flags::RESPONSE);
            r.ue_ambr = ue_ambr;
            ctx.send(r)
        });
        if let Err(e) = sent {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", msg.from());
        }
    }

    fn on_sm_policy(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &SmPolicyCreate) {
        let (supi, dnn) = (request.supi.as_str(), request.dnn.as_str());
        let policy = self.policies.of(supi);
        let rule = policy.session(dnn).or_else(|| self.policies.default.session(dnn)).copied();
        let sent = ctx.alloc::<SmPolicyResult>(msg.from()).and_then(|r| {
            let mut r = r.with_corr_id(msg.corr_id()).with_flags(flags::RESPONSE);
            match rule {
                Some(rule) => r.rule = rule,
                None => r.set_outcome(librecn_sbi::Outcome::InvalidRequest),
            }
            ctx.send(r)
        });
        match rule {
            Some(rule) => {
                let (ul, dl) = mbps(rule.ambr);
                let psi = request.pdu_session_id;
                let flows = rule.rules().len();
                log!(
                    self.log,
                    ctx,
                    Info,
                    "{supi}: SM policy for PDU session {psi} to {dnn}: 5QI {}, ARP {}, AMBR {ul} / {dl} Mbit/s, {flows} extra flow(s)",
                    rule.five_qi,
                    rule.arp
                );
            }
            None => log!(self.log, ctx, Info, "{supi}: no SM policy for DNN {dnn:?}"),
        }
        if let Err(e) = sent {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", msg.from());
        }
    }

    /// The AM policy of `supi` changed: its AMF is told.
    fn notify(&mut self, ctx: &mut Context<'_>, supi: &str) {
        let Some(&amf) = self.amfs.get(supi) else { return };
        let Some(supi_text) = Supi::new(supi) else { return };
        let update = AmPolicyUpdate { ue_ambr: self.policies.of(supi).ue_ambr, supi: supi_text };
        let sent = ctx.alloc::<AmPolicyUpdate>(amf).and_then(|mut m| {
            *m = update;
            ctx.send(m.with_flags(flags::NOTIFICATION))
        });
        if let Err(e) = sent {
            log!(self.log, ctx, Error, "{supi}: the AMF not told of its new policy: {e}");
        }
    }

    fn on_manage(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &Manage) {
        let Some(method) = request.method() else { return };
        let answer = manage::handle(&mut self.policies, &mut self.removed, method, request.path(), request.body());
        if let Some(change) = &answer.change {
            self.dirty = true;
            log!(self.log, ctx, Info, "{change}");
        }
        for supi in &answer.changed {
            self.notify(ctx, supi);
        }
        if let Err(e) = send_reply(ctx, msg, answer.status, &answer.json) {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", msg.from());
        }
    }

    fn snapshot(&mut self, ctx: &mut Context<'_>) {
        let Some((store, _)) = &self.store else { return };
        if !std::mem::take(&mut self.dirty) {
            return;
        }
        if let Err(e) = store.save(&persist::snapshot(&self.policies, &self.removed)) {
            self.dirty = true;
            log!(self.log, ctx, Error, "state {} not written: {e}", store.path().display());
        }
    }
}

impl ExecutionObject for Pcf {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        let (ul, dl) = mbps(self.policies.default.ue_ambr);
        log!(
            self.log,
            ctx,
            Vip,
            "PCF {} ready, {} subscriber policy(ies), default UE-AMBR {ul} / {dl} Mbit/s",
            self.name,
            self.policies.by_supi.len()
        );
        if let Some((store, r)) = &self.store {
            let path = store.path().display();
            log!(
                self.log,
                ctx,
                Info,
                "state in {path}: {} policy(ies) set and {} removed by the web API",
                r.set,
                r.removed
            );
            let _ = ctx.arm_timer(SNAPSHOT_PERIOD, 0);
        }
    }

    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if msg.timer().is_some() {
            self.snapshot(ctx);
            let _ = ctx.arm_timer(SNAPSHOT_PERIOD, 0);
        } else if let Some(request) = msg.body::<AmPolicyCreate>() {
            self.on_am_policy(ctx, msg, request);
        } else if let Some(request) = msg.body::<SmPolicyCreate>() {
            self.on_sm_policy(ctx, msg, request);
        } else if let Some(request) = msg.body::<Manage>() {
            self.on_manage(ctx, msg, request);
        }
    }

    fn on_stop(&mut self, ctx: &mut Context<'_>) {
        self.snapshot(ctx);
    }
}
