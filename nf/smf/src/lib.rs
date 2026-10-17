//! SMF, the session management function, hosted in one EO.
//!
//! This release establishes IPv4 PDU sessions (TS 23.502 §4.3.2.2.1): the
//! AMF hands over the UE's PDU Session Establishment Request; the SMF
//! checks it, gives the UE an address, sets the session up on the UPF (N4),
//! and answers with the Accept for the UE (N1) and the setup for the gNB
//! (N2). The gNB's answer, through the AMF, gives the UPF its downlink
//! tunnel.
//!
//! - [`session`], the decision and the N1 answers, on domain values;
//! - [`n2sm`], the NGAP transfers (the only module seeing ASN.1);
//! - [`pool`], UE addresses;
//! - [`config`], what is served, from the deployment `params`.

pub mod config;
pub mod n2sm;
pub mod pool;
pub mod session;

use std::collections::HashMap;
use std::net::Ipv4Addr;

use librecn_common::Service;
use librecn_common::log;
use librecn_common::log::Logger;
use librecn_common::msg::flags;
use librecn_nas::Snssai;
use librecn_nas::sm::cause;
use librecn_sbi::pdu_session::{
    CreateSmContext, CreateSmContextResult, N4Delete, N4Establish, N4EstablishResult, N4Modify, N4ModifyResult,
    ReleaseSmContext, UpdateSmContext, UpdateSmContextResult, update,
};
use librecn_sbi::{Outcome, Supi};
use librecn_scheduler::{Context, EoConfig, EoId, ExecutionObject, Message};

use config::SmfConfig;
use pool::Pool;
use session::Refused;

/// One PDU session.
struct SmContext {
    supi: Supi,
    pdu_session_id: u8,
    pti: u8,
    slice: Snssai,
    ue_ip: Ipv4Addr,
    amf: EoId,
    /// The AMF's request waiting for the UPF (its `corr_id`).
    waiting: Option<u32>,
    /// The UPF's uplink tunnel endpoint, once it answered N4Establish.
    n3: Option<(Ipv4Addr, u32)>,
}

/// The SMF EO.
pub struct Smf {
    name: String,
    config: SmfConfig,
    upf: Service,
    pool: Pool,
    contexts: HashMap<u32, SmContext>,
    next_ctx: u32,
    log: Logger,
}

impl Smf {
    pub fn new(name: &str, config: SmfConfig) -> Self {
        Smf {
            name: name.to_string(),
            upf: Service::named(config.upf.clone()),
            pool: Pool::new(config.pool.0, config.pool.1),
            config,
            contexts: HashMap::new(),
            next_ctx: 1,
            log: Logger::default(),
        }
    }

    /// For the control EO's factory.
    pub fn from_params(name: &str, params: &serde_json::Value) -> Result<(EoConfig, Self), String> {
        Ok((Self::eo_config(name), Self::new(name, SmfConfig::from_params(params)?)))
    }

    /// The AMF's requests carry N1/N2 containers (1 KiB); the UPF's answers are small.
    pub fn eo_config(name: &str) -> EoConfig {
        EoConfig::new(name).max_timers(0).buffers(&[(256, 64), (1024, 32)])
    }

    fn next_ctx(&mut self) -> u32 {
        loop {
            let id = self.next_ctx;
            self.next_ctx = self.next_ctx.wrapping_add(1).max(1);
            if !self.contexts.contains_key(&id) {
                return id;
            }
        }
    }

    /// Answers a CreateSmContext; an empty `n2` is a reject.
    fn create_result(&mut self, ctx: &mut Context<'_>, to: (EoId, u32), psi: u8, sm_ctx: u32, n1: &[u8], n2: &[u8]) {
        let sent = ctx.alloc::<CreateSmContextResult>(to.0).and_then(|r| {
            let mut r = r.with_corr_id(to.1).with_flags(flags::RESPONSE);
            (r.pdu_session_id, r.sm_ctx) = (psi, sm_ctx);
            r.n1.set(n1);
            r.n2.set(n2);
            ctx.send(r)
        });
        if let Err(e) = sent {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", to.0);
        }
    }

    fn refuse(&mut self, ctx: &mut Context<'_>, to: (EoId, u32), supi: &Supi, psi: u8, refused: Refused) {
        log!(self.log, ctx, Info, "{supi}: PDU session {psi} refused, 5GSM cause #{}", refused.cause);
        self.create_result(ctx, to, psi, 0, &session::reject(psi, refused), &[]);
    }

    fn on_create(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &CreateSmContext) {
        let (to, psi, supi) = ((msg.from(), msg.corr_id()), request.pdu_session_id, request.supi);
        let slice = Snssai { sst: request.slice.sst, sd: (request.slice.has_sd != 0).then_some(request.slice.sd) };
        let accepted = match session::decide(&self.config, psi, slice, request.dnn.as_str(), request.n1.get()) {
            Ok(accepted) => accepted,
            Err(refused) => return self.refuse(ctx, to, &supi, psi, refused),
        };
        let insufficient = Refused { pti: accepted.pti, cause: cause::INSUFFICIENT_RESOURCES };
        let Some(ue_ip) = self.pool.allocate() else {
            log!(self.log, ctx, Error, "address pool exhausted");
            return self.refuse(ctx, to, &supi, psi, insufficient);
        };
        let id = self.next_ctx();
        let upf = self.upf.get(ctx);
        let sent = upf.ok_or(librecn_scheduler::Error::UnknownEo).and_then(|upf| {
            let mut m = ctx.alloc::<N4Establish>(upf)?.with_corr_id(id).with_flags(flags::REQUEST);
            *m = N4Establish::new(u64::from(id), ue_ip.octets());
            ctx.send(m)
        });
        if let Err(e) = sent {
            self.upf.forget();
            self.pool.release(ue_ip);
            log!(self.log, ctx, Error, "cannot reach the UPF {:?}: {e}", self.upf.name());
            return self.refuse(ctx, to, &supi, psi, insufficient);
        }
        let context = SmContext {
            supi,
            pdu_session_id: psi,
            pti: accepted.pti,
            slice,
            ue_ip,
            amf: to.0,
            waiting: Some(to.1),
            n3: None,
        };
        self.contexts.insert(id, context);
    }

    fn on_established(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, result: &N4EstablishResult) {
        let id = msg.corr_id();
        let Some(c) = self.contexts.get_mut(&id) else { return };
        let Some(corr) = c.waiting.take() else { return };
        let (to, psi, supi) = ((c.amf, corr), c.pdu_session_id, c.supi);
        if result.outcome() != Outcome::Ok {
            log!(self.log, ctx, Error, "{supi}: the UPF refused session {id}: {:?}", result.outcome());
            let pti = c.pti;
            self.remove(ctx, id);
            return self.refuse(ctx, to, &supi, psi, Refused { pti, cause: cause::INSUFFICIENT_RESOURCES });
        }
        let n3 = (Ipv4Addr::from(result.n3_addr), result.ul_teid);
        c.n3 = Some(n3);
        let (pti, slice, ue_ip) = (c.pti, c.slice, c.ue_ip);
        let n2 = match self.setup_transfer(n3) {
            Ok(n2) => n2,
            Err(e) => {
                log!(self.log, ctx, Error, "{supi}: {e}");
                self.remove(ctx, id);
                return self.refuse(ctx, to, &supi, psi, Refused { pti, cause: cause::INSUFFICIENT_RESOURCES });
            }
        };
        let n1 = session::accept(&self.config, psi, pti, slice, ue_ip);
        log!(
            self.log,
            ctx,
            Info,
            "{supi}: PDU session {psi} to {}, UE {ue_ip}, N3 {} TEID {:#x}",
            self.config.dnn,
            n3.0,
            n3.1
        );
        self.create_result(ctx, to, psi, id, &n1, &n2);
    }

    /// The PDU Session Resource Setup Request Transfer for the UPF's tunnel `n3`.
    fn setup_transfer(&self, n3: (Ipv4Addr, u32)) -> Result<Vec<u8>, String> {
        let (ul, dl) = self.config.ambr_mbps;
        n2sm::setup_request_transfer(&n2sm::Setup {
            upf: n3.0,
            teid: n3.1,
            qfi: session::DEFAULT_QFI,
            five_qi: self.config.five_qi,
            ambr_bps: (u64::from(ul) * 1_000_000, u64::from(dl) * 1_000_000),
        })
    }

    fn n4_modify(&mut self, ctx: &mut Context<'_>, id: u32, gnb: Ipv4Addr, teid: u32) -> librecn_scheduler::Result<()> {
        let sent = self.upf.get(ctx).ok_or(librecn_scheduler::Error::UnknownEo).and_then(|upf| {
            let mut m = ctx.alloc::<N4Modify>(upf)?.with_corr_id(id).with_flags(flags::REQUEST);
            *m = N4Modify { seid: u64::from(id), gnb_addr: gnb.octets(), dl_teid: teid };
            ctx.send(m)
        });
        if sent.is_err() {
            self.upf.forget();
        }
        sent
    }

    fn on_update(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &UpdateSmContext) {
        let to = (msg.from(), msg.corr_id());
        let id = request.sm_ctx;
        let Some(c) = self.contexts.get_mut(&id) else {
            if request.kind != update::DEACTIVATE {
                self.update_result(ctx, to, 0, Outcome::ContextNotFound, &[]);
            }
            return;
        };
        let (psi, supi, n3) = (c.pdu_session_id, c.supi, c.n3);
        match (request.kind, n3) {
            (update::DEACTIVATE, _) => {
                log!(self.log, ctx, Info, "{supi}: PDU session {psi} deactivated (the UE is idle)");
                c.amf = to.0;
                let _ = self.n4_modify(ctx, id, Ipv4Addr::UNSPECIFIED, 0);
                return;
            }
            (update::ACTIVATE, Some(n3)) => {
                c.amf = to.0;
                return match self.setup_transfer(n3) {
                    Ok(n2) => {
                        log!(self.log, ctx, Info, "{supi}: PDU session {psi} activated");
                        self.update_result(ctx, to, psi, Outcome::Ok, &n2)
                    }
                    Err(e) => {
                        log!(self.log, ctx, Error, "{supi}: {e}");
                        self.update_result(ctx, to, psi, Outcome::Unavailable, &[])
                    }
                };
            }
            (update::ACTIVATE, None) => return self.update_result(ctx, to, psi, Outcome::ContextNotFound, &[]),
            _ => {}
        }
        if request.kind == update::SETUP_FAILED {
            let why = n2sm::unsuccessful_transfer(request.n2.get());
            log!(self.log, ctx, Info, "{supi}: the gNB could not set PDU session {psi} up: {why}");
            self.remove(ctx, id);
            return self.update_result(ctx, to, psi, Outcome::Ok, &[]);
        }
        let (gnb, teid) = match n2sm::setup_response_transfer(request.n2.get()) {
            Ok(tunnel) => tunnel,
            Err(e) => {
                log!(self.log, ctx, Error, "{supi}: PDU session {psi}: {e}");
                return self.update_result(ctx, to, psi, Outcome::InvalidRequest, &[]);
            }
        };
        (c.waiting, c.amf) = (Some(to.1), to.0);
        if let Err(e) = self.n4_modify(ctx, id, gnb, teid) {
            log!(self.log, ctx, Error, "cannot reach the UPF: {e}");
            if let Some(c) = self.contexts.get_mut(&id) {
                c.waiting = None;
            }
            return self.update_result(ctx, to, psi, Outcome::Unavailable, &[]);
        }
        log!(self.log, ctx, Info, "{supi}: PDU session {psi} downlink to {gnb} TEID {teid:#x}");
    }

    fn on_modified(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, result: &N4ModifyResult) {
        let Some(c) = self.contexts.get_mut(&msg.corr_id()) else { return };
        let Some(corr) = c.waiting.take() else { return };
        let (to, psi) = ((c.amf, corr), c.pdu_session_id);
        self.update_result(ctx, to, psi, result.outcome(), &[]);
    }

    fn update_result(&mut self, ctx: &mut Context<'_>, to: (EoId, u32), psi: u8, outcome: Outcome, n2: &[u8]) {
        let sent = ctx.alloc::<UpdateSmContextResult>(to.0).and_then(|r| {
            let mut r = r.with_corr_id(to.1).with_flags(flags::RESPONSE);
            r.pdu_session_id = psi;
            r.set_outcome(outcome);
            r.n2.set(n2);
            ctx.send(r)
        });
        if let Err(e) = sent {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", to.0);
        }
    }

    /// Drops a session: the UPF's side, the address.
    fn remove(&mut self, ctx: &mut Context<'_>, id: u32) {
        let Some(c) = self.contexts.remove(&id) else { return };
        self.pool.release(c.ue_ip);
        if c.n3.is_some()
            && let Some(upf) = self.upf.get(ctx)
        {
            let _ = ctx.send_value(upf, &N4Delete { seid: u64::from(id) });
        }
        log!(
            self.log,
            ctx,
            Info,
            "{}: PDU session {} released, {} left",
            c.supi,
            c.pdu_session_id,
            self.contexts.len()
        );
    }
}

impl ExecutionObject for Smf {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        let (net, len) = self.config.pool;
        log!(
            self.log,
            ctx,
            Vip,
            "SMF {} ready: DNN {}, pool {net}/{len}, UPF {:?}",
            self.name,
            self.config.dnn,
            self.upf.name()
        );
    }

    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if let Some(request) = msg.body::<CreateSmContext>() {
            self.on_create(ctx, msg, request);
        } else if let Some(result) = msg.body::<N4EstablishResult>() {
            self.on_established(ctx, msg, result);
        } else if let Some(request) = msg.body::<UpdateSmContext>() {
            self.on_update(ctx, msg, request);
        } else if let Some(result) = msg.body::<N4ModifyResult>() {
            self.on_modified(ctx, msg, result);
        } else if let Some(request) = msg.body::<ReleaseSmContext>() {
            self.remove(ctx, request.sm_ctx);
        }
    }

    fn on_stop(&mut self, ctx: &mut Context<'_>) {
        log!(self.log, ctx, Info, "SMF {} stopping, {} session(s)", self.name, self.contexts.len());
    }
}
