//! SMF, the session management function, hosted in one EO.
//!
//! This release establishes IPv4 PDU sessions (TS 23.502 §4.3.2.2.1): the
//! AMF hands over the UE's PDU Session Establishment Request; the SMF
//! checks it, gives the UE an address, asks the PCF for the session's policy
//! (QoS, AMBR), sets the session up on the UPF (N4), and answers with the
//! Accept for the UE (N1) and the setup for the gNB (N2). The gNB's answer, through the AMF, gives the UPF its downlink
//! tunnel.
//!
//! Sessions are released by the UE (PDU Session Release Request) or by the
//! network (`DELETE /api/nfs/{smf}/sessions/{id}`, the AMF telling the UE);
//! a UE's modification request is rejected (no QoS change is offered).
//! Downlink data for an idle UE, reported by the UPF, goes to the AMF with
//! the session's N2 setup (TS 23.502 §4.2.3.3): the AMF pages the UE.
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
use librecn_common::manage::{Manage, Method, send_reply};
use librecn_common::msg::flags;
use librecn_nas::Snssai;
use librecn_nas::sm::{SmUplink, cause};
use librecn_sbi::pdu_session::{
    CreateSmContext, CreateSmContextResult, N4Delete, N4Establish, N4EstablishResult, N4Modify, N4ModifyResult,
    N4Report, ReleaseSmContext, SmNotify, SmNotifyResult, UpdateSmContext, UpdateSmContextResult, n2, update,
};
use librecn_sbi::policy::{Rates, SessionRule, SmPolicyCreate, SmPolicyResult};
use librecn_sbi::{Outcome, Supi};
use librecn_scheduler::{Context, EoConfig, EoId, ExecutionObject, Message};
use serde_json::json;

use config::SmfConfig;
use pool::Pool;
use session::Refused;

/// Sessions listed by `GET sessions` (the answer fits in 8 KiB).
const LISTED: usize = 64;

/// What an UpdateSmContextResult carries.
#[derive(Default)]
struct Answer<'a> {
    outcome: Outcome,
    released: bool,
    n1: &'a [u8],
    n2_kind: u8,
    n2: &'a [u8],
}

impl Answer<'_> {
    fn outcome(outcome: Outcome) -> Self {
        Answer { outcome, ..Answer::default() }
    }
}

/// The PCF's rule of a session, kept small: whole (boxed) only with PCC
/// rules, which most sessions lack (the rule takes 440 bytes).
enum Policy {
    Plain { ambr: Rates, five_qi: u8, arp: u8 },
    Rules(Box<SessionRule>),
}

impl Policy {
    fn new(rule: &SessionRule) -> Self {
        match rule.rules() {
            [] => Policy::Plain { ambr: rule.ambr, five_qi: rule.five_qi, arp: rule.arp },
            _ => Policy::Rules(Box::new(*rule)),
        }
    }

    fn rule(&self) -> SessionRule {
        match self {
            Policy::Rules(rule) => **rule,
            Policy::Plain { ambr, five_qi, arp } => {
                let mut rule = SessionRule::default();
                (rule.ambr, rule.five_qi, rule.arp) = (*ambr, *five_qi, *arp);
                rule
            }
        }
    }
}

/// One PDU session.
struct SmContext {
    supi: Supi,
    pdu_session_id: u8,
    pti: u8,
    slice: Snssai,
    /// Its DNN, an index in the configuration's (and in the pools).
    dnn: usize,
    ue_ip: Ipv4Addr,
    amf: EoId,
    /// The AMF's id of the UE, the `corr_id` of its last request.
    amf_ue: u32,
    /// A Release Command was sent: waiting for the UE's Release Complete.
    releasing: bool,
    /// The AMF's request waiting for the PCF or the UPF (its `corr_id`).
    waiting: Option<u32>,
    /// The PCF's policy, once it answered.
    rule: Option<Policy>,
    /// The UPF's uplink tunnel endpoint, once it answered N4Establish.
    n3: Option<(Ipv4Addr, u32)>,
    /// The downlink tunnel is set on the UPF (false while the UE is idle).
    active: bool,
}

/// The SMF EO.
pub struct Smf {
    name: String,
    config: SmfConfig,
    upf: Service,
    pcf: Service,
    /// UE addresses, one pool per DNN.
    pools: Vec<Pool>,
    contexts: HashMap<u32, SmContext>,
    next_ctx: u32,
    log: Logger,
}

impl Smf {
    pub fn new(name: &str, config: SmfConfig) -> Self {
        Smf {
            name: name.to_string(),
            upf: Service::named(config.upf.clone()),
            pcf: Service::named(config.pcf.clone()),
            pools: config.dnns.iter().map(|d| Pool::new(d.pool.0, d.pool.1)).collect(),
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

    /// The AMF's requests carry N1/N2 containers (1 KiB); the UPF's answers
    /// are small; management requests 8 KiB, four at once (the GUI asks for
    /// several resources together).
    pub fn eo_config(name: &str) -> EoConfig {
        EoConfig::new(name).max_timers(0).buffers(&[(256, 1024), (1024, 1024), (8 * 1024, 4)])
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
        let dnn = accepted.dnn;
        let Some(ue_ip) = self.pools[dnn].allocate() else {
            log!(self.log, ctx, Error, "address pool of {} exhausted", self.config.dnns[dnn].dnn);
            return self.refuse(ctx, to, &supi, psi, insufficient);
        };
        let id = self.next_ctx();
        let pcf = self.pcf.get(ctx);
        let sent = pcf.ok_or(librecn_scheduler::Error::UnknownEo).and_then(|pcf| {
            let mut m = ctx.alloc::<SmPolicyCreate>(pcf)?.with_corr_id(id).with_flags(flags::REQUEST);
            (m.supi, m.dnn, m.slice, m.pdu_session_id) = (supi, request.dnn, request.slice, psi);
            ctx.send(m)
        });
        if let Err(e) = sent {
            self.pcf.forget();
            self.pools[dnn].release(ue_ip);
            log!(self.log, ctx, Error, "cannot reach the PCF {:?}: {e}", self.pcf.name());
            return self.refuse(ctx, to, &supi, psi, insufficient);
        }
        let context = SmContext {
            supi,
            pdu_session_id: psi,
            pti: accepted.pti,
            slice,
            dnn,
            ue_ip,
            amf: to.0,
            amf_ue: to.1,
            releasing: false,
            waiting: Some(to.1),
            rule: None,
            n3: None,
            active: false,
        };
        self.contexts.insert(id, context);
    }

    /// The PCF's policy: the session is set up on the UPF.
    fn on_policy(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, result: &SmPolicyResult) {
        let id = msg.corr_id();
        let Some(c) = self.contexts.get_mut(&id).filter(|c| c.rule.is_none() && c.waiting.is_some()) else { return };
        let (supi, psi, pti, ue_ip) = (c.supi, c.pdu_session_id, c.pti, c.ue_ip);
        let insufficient = Refused { pti, cause: cause::INSUFFICIENT_RESOURCES };
        if result.outcome() != Outcome::Ok {
            log!(self.log, ctx, Info, "{supi}: no policy for PDU session {psi}: {:?}", result.outcome());
            let to = (c.amf, c.waiting.take().unwrap_or_default());
            self.remove(ctx, id);
            return self.refuse(ctx, to, &supi, psi, insufficient);
        }
        c.rule = Some(Policy::new(&result.rule));
        let sent = self.upf.get(ctx).ok_or(librecn_scheduler::Error::UnknownEo).and_then(|upf| {
            let mut m = ctx.alloc::<N4Establish>(upf)?.with_corr_id(id).with_flags(flags::REQUEST);
            *m = N4Establish::new(u64::from(id), ue_ip.octets(), result.rule);
            ctx.send(m)
        });
        if let Err(e) = sent {
            self.upf.forget();
            log!(self.log, ctx, Error, "cannot reach the UPF {:?}: {e}", self.upf.name());
            let Some(c) = self.contexts.get_mut(&id) else { return };
            let to = (c.amf, c.waiting.take().unwrap_or_default());
            self.remove(ctx, id);
            self.refuse(ctx, to, &supi, psi, insufficient);
        }
    }

    fn on_established(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, result: &N4EstablishResult) {
        let id = msg.corr_id();
        let Some(c) = self.contexts.get_mut(&id) else { return };
        let Some(rule) = c.rule.as_ref().map(Policy::rule) else { return };
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
        let (pti, slice, dnn, ue_ip) = (c.pti, c.slice, c.dnn, c.ue_ip);
        let n2 = match setup_transfer(n3, &rule) {
            Ok(n2) => n2,
            Err(e) => {
                log!(self.log, ctx, Error, "{supi}: {e}");
                self.remove(ctx, id);
                return self.refuse(ctx, to, &supi, psi, Refused { pti, cause: cause::INSUFFICIENT_RESOURCES });
            }
        };
        let Some(n1) = session::accept(&self.config, psi, pti, slice, dnn, ue_ip, &rule) else {
            log!(self.log, ctx, Error, "{supi}: the PCF's rules do not fit in a PDU Session Establishment Accept");
            self.remove(ctx, id);
            return self.refuse(ctx, to, &supi, psi, Refused { pti, cause: cause::INSUFFICIENT_RESOURCES });
        };
        log!(
            self.log,
            ctx,
            Info,
            "{supi}: PDU session {psi} to {}, UE {ue_ip}, N3 {} TEID {:#x}",
            self.config.dnns[dnn].dnn,
            n3.0,
            n3.1
        );
        self.create_result(ctx, to, psi, id, &n1, &n2);
    }

    /// N4Modify; `teid` 0 (deactivation) is a notification, not answered.
    fn n4_modify(&mut self, ctx: &mut Context<'_>, id: u32, gnb: Ipv4Addr, teid: u32) -> librecn_scheduler::Result<()> {
        let flag = if teid == 0 { flags::NOTIFICATION } else { flags::REQUEST };
        let sent = self.upf.get(ctx).ok_or(librecn_scheduler::Error::UnknownEo).and_then(|upf| {
            let mut m = ctx.alloc::<N4Modify>(upf)?.with_corr_id(id).with_flags(flag);
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
            return self.update_result(ctx, to, 0, Answer::outcome(Outcome::ContextNotFound));
        };
        (c.amf, c.amf_ue) = to;
        let (psi, supi, n3) = (c.pdu_session_id, c.supi, c.n3.zip(c.rule.as_ref().map(Policy::rule)));
        let container = request.container.get();
        match (request.kind, n3) {
            (update::DEACTIVATE, _) => {
                c.active = false;
                log!(self.log, ctx, Info, "{supi}: PDU session {psi} deactivated (the UE is idle)");
                let _ = self.n4_modify(ctx, id, Ipv4Addr::UNSPECIFIED, 0);
                // The AMF's acknowledgement: it paces its notices on them.
                self.update_result(ctx, to, psi, Answer::default());
            }
            (update::ACTIVATE, Some((n3, rule))) => match setup_transfer(n3, &rule) {
                Ok(n2) => {
                    log!(self.log, ctx, Info, "{supi}: PDU session {psi} activated");
                    self.update_result(ctx, to, psi, Answer { n2_kind: n2::SETUP, n2: &n2, ..Answer::default() });
                }
                Err(e) => {
                    log!(self.log, ctx, Error, "{supi}: {e}");
                    self.update_result(ctx, to, psi, Answer::outcome(Outcome::Unavailable));
                }
            },
            (update::ACTIVATE, None) => self.update_result(ctx, to, psi, Answer::outcome(Outcome::ContextNotFound)),
            (update::N1, _) => self.on_n1(ctx, to, id, container),
            (update::SETUP_FAILED, _) => {
                let why = n2sm::unsuccessful_transfer(container);
                log!(self.log, ctx, Info, "{supi}: the gNB could not set PDU session {psi} up: {why}");
                self.remove(ctx, id);
                self.update_result(ctx, to, psi, Answer { released: true, ..Answer::default() });
            }
            _ => self.on_setup_response(ctx, to, id, container),
        }
    }

    /// The gNB's tunnel, to the UPF; answered once the UPF is.
    fn on_setup_response(&mut self, ctx: &mut Context<'_>, to: (EoId, u32), id: u32, transfer: &[u8]) {
        let Some(c) = self.contexts.get_mut(&id) else { return };
        let (psi, supi) = (c.pdu_session_id, c.supi);
        let (gnb, teid) = match n2sm::setup_response_transfer(transfer) {
            Ok(tunnel) => tunnel,
            Err(e) => {
                log!(self.log, ctx, Error, "{supi}: PDU session {psi}: {e}");
                return self.update_result(ctx, to, psi, Answer::outcome(Outcome::InvalidRequest));
            }
        };
        c.waiting = Some(to.1);
        if let Err(e) = self.n4_modify(ctx, id, gnb, teid) {
            log!(self.log, ctx, Error, "cannot reach the UPF: {e}");
            if let Some(c) = self.contexts.get_mut(&id) {
                c.waiting = None;
            }
            return self.update_result(ctx, to, psi, Answer::outcome(Outcome::Unavailable));
        }
        log!(self.log, ctx, Info, "{supi}: PDU session {psi} downlink to {gnb} TEID {teid:#x}");
    }

    /// A 5GSM message of the UE for an existing session (TS 24.501 §6.3, §6.4).
    fn on_n1(&mut self, ctx: &mut Context<'_>, to: (EoId, u32), id: u32, n1: &[u8]) {
        let Some(c) = self.contexts.get(&id) else { return };
        let (psi, supi) = (c.pdu_session_id, c.supi);
        let Ok((header, message)) = SmUplink::decode(n1) else {
            let status = session::status(psi, 0, cause::INVALID_MANDATORY_INFORMATION);
            return self.update_result(ctx, to, psi, Answer { n1: &status, ..Answer::default() });
        };
        log!(self.log, ctx, Info, "{supi}: PDU session {psi}: {}", librecn_nas::sm::name(header.kind));
        match message {
            SmUplink::ReleaseRequest => {
                let (command, transfer) = self.release(ctx, id, header.pti);
                let answer = Answer { n1: &command, n2_kind: n2::RELEASE, n2: &transfer, ..Answer::default() };
                self.update_result(ctx, to, psi, answer);
            }
            SmUplink::ReleaseComplete => {
                self.remove(ctx, id);
                self.update_result(ctx, to, psi, Answer { released: true, ..Answer::default() });
            }
            // QoS changes are not offered: the session stays as it is.
            SmUplink::ModificationRequest => {
                let reject = session::modification_reject(psi, header.pti);
                self.update_result(ctx, to, psi, Answer { n1: &reject, ..Answer::default() });
            }
            SmUplink::Status { cause } => {
                log!(self.log, ctx, Info, "{supi}: PDU session {psi}: 5GSM Status #{cause}");
                self.update_result(ctx, to, psi, Answer::default());
            }
            SmUplink::EstablishmentRequest { .. } | SmUplink::Other(_) => {
                let status = session::status(psi, header.pti, cause::MESSAGE_TYPE_NOT_IMPLEMENTED);
                self.update_result(ctx, to, psi, Answer { n1: &status, ..Answer::default() });
            }
        }
    }

    /// Ends a session on the UPF; the Release Command (N1, answering `pti`,
    /// 0 for the network's own) and the gNB's release (N2). The context stays
    /// until the UE's Release Complete, or the UE's departure.
    fn release(&mut self, ctx: &mut Context<'_>, id: u32, pti: u8) -> (Vec<u8>, Vec<u8>) {
        let Some(c) = self.contexts.get_mut(&id) else { return (vec![], vec![]) };
        let psi = c.pdu_session_id;
        c.releasing = true;
        if c.n3.take().is_some()
            && let Some(upf) = self.upf.get(ctx)
        {
            let _ = ctx.send_value(upf, &N4Delete { seid: u64::from(id) });
        }
        let transfer = n2sm::release_command_transfer().unwrap_or_default();
        (session::release_command(psi, pti), transfer)
    }

    fn on_modified(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, result: &N4ModifyResult) {
        let Some(c) = self.contexts.get_mut(&msg.corr_id()) else { return };
        let Some(corr) = c.waiting.take() else { return };
        c.active = result.outcome() == Outcome::Ok;
        let (to, psi) = ((c.amf, corr), c.pdu_session_id);
        self.update_result(ctx, to, psi, Answer::outcome(result.outcome()));
    }

    fn update_result(&mut self, ctx: &mut Context<'_>, to: (EoId, u32), psi: u8, answer: Answer<'_>) {
        let sent = ctx.alloc::<UpdateSmContextResult>(to.0).and_then(|r| {
            let mut r = r.with_corr_id(to.1).with_flags(flags::RESPONSE);
            (r.pdu_session_id, r.released, r.n2_kind) = (psi, u8::from(answer.released), answer.n2_kind);
            r.set_outcome(answer.outcome);
            r.n1.set(answer.n1);
            r.n2.set(answer.n2);
            ctx.send(r)
        });
        if let Err(e) = sent {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", to.0);
        }
    }

    /// Downlink data for session `id` without its tunnel: the AMF gets the
    /// N2 setup, to page the UE if it is idle (TS 23.502 §4.2.3.3).
    fn on_report(&mut self, ctx: &mut Context<'_>, id: u32) {
        let Some(c) = self.contexts.get(&id).filter(|c| !c.active && !c.releasing) else { return };
        let (amf, amf_ue, psi, supi) = (c.amf, c.amf_ue, c.pdu_session_id, c.supi);
        let Some((n3, rule)) = c.n3.zip(c.rule.as_ref().map(Policy::rule)) else { return };
        let transfer = match setup_transfer(n3, &rule) {
            Ok(transfer) => transfer,
            Err(e) => return log!(self.log, ctx, Error, "{supi}: {e}"),
        };
        let sent = ctx.alloc::<SmNotify>(amf).and_then(|m| {
            let mut m = m.with_corr_id(amf_ue).with_flags(flags::REQUEST);
            (m.sm_ctx, m.pdu_session_id, m.n2_kind) = (id, psi, n2::SETUP);
            m.n2.set(&transfer);
            ctx.send(m)
        });
        match sent {
            Ok(()) => log!(self.log, ctx, Info, "{supi}: PDU session {psi}: downlink data, the AMF told"),
            Err(e) => log!(self.log, ctx, Error, "{supi}: PDU session {psi}: downlink data, the AMF not told: {e}"),
        }
    }

    /// The network releases session `id` (TS 23.502 §4.3.4.2): the UE told
    /// through its AMF.
    fn release_by_network(&mut self, ctx: &mut Context<'_>, id: u32) -> bool {
        let Some(c) = self.contexts.get(&id).filter(|c| !c.releasing) else { return false };
        let (amf, amf_ue, psi, supi) = (c.amf, c.amf_ue, c.pdu_session_id, c.supi);
        let (command, transfer) = self.release(ctx, id, 0);
        log!(self.log, ctx, Info, "{supi}: PDU session {psi} released by the network");
        let sent = ctx.alloc::<SmNotify>(amf).and_then(|m| {
            let mut m = m.with_corr_id(amf_ue).with_flags(flags::REQUEST);
            (m.sm_ctx, m.pdu_session_id, m.n2_kind) = (id, psi, n2::RELEASE);
            m.n1.set(&command);
            m.n2.set(&transfer);
            ctx.send(m)
        });
        if sent.is_err() {
            self.remove(ctx, id);
        }
        true
    }

    fn on_manage(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &Manage) {
        let (status, json) = match (request.method(), request.path().split_once('/').unwrap_or((request.path(), ""))) {
            (Some(Method::Get), ("sessions", "")) => {
                let supi = request.query("supi");
                let mut list: Vec<_> =
                    self.contexts.iter().filter(|(_, c)| supi.is_none_or(|s| c.supi.as_str() == s)).collect();
                list.sort_by_key(|(id, _)| **id);
                let count = list.len();
                let list: Vec<_> = list.into_iter().take(LISTED).map(|(id, c)| self.view(*id, c)).collect();
                (200, json!({ "count": count, "sessions": list }))
            }
            (Some(Method::Get), ("dnns", "")) => {
                let dnns: Vec<_> = self
                    .config
                    .dnns
                    .iter()
                    .enumerate()
                    .map(|(i, d)| {
                        json!({ "dnn": d.dnn, "pool": format!("{}/{}", d.pool.0, d.pool.1),
                                "sessions": self.contexts.values().filter(|c| c.dnn == i).count() })
                    })
                    .collect();
                (200, json!(dnns))
            }
            (Some(Method::Delete), ("sessions", id)) => match id.parse() {
                Ok(id) if self.release_by_network(ctx, id) => (200, json!({ "releasing": id })),
                _ => (404, json!({ "error": format!("no session {id:?}") })),
            },
            _ => (404, json!({ "error": format!("no resource {:?}", request.path()) })),
        };
        if let Err(e) = send_reply(ctx, msg, status, &json.to_string()) {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", msg.from());
        }
    }

    /// A session as the web API shows it.
    fn view(&self, id: u32, c: &SmContext) -> serde_json::Value {
        let state = match (c.releasing, c.active, c.n3.is_some()) {
            (true, _, _) => "releasing",
            (_, true, _) => "active",
            (_, false, true) => "idle",
            (_, false, false) => "setting up",
        };
        let rule = c.rule.as_ref().map(Policy::rule);
        let mbps = |bps: u64| bps / 1_000_000;
        json!({ "id": id, "supi": c.supi.as_str(), "pdu_session_id": c.pdu_session_id,
                "ue_ip": c.ue_ip.to_string(), "dnn": self.config.dnns[c.dnn].dnn, "state": state,
                "five_qi": rule.map(|r| r.five_qi),
                "ambr_mbps": rule.map(|r| json!({ "uplink": mbps(r.ambr.uplink), "downlink": mbps(r.ambr.downlink) })),
                "flows": rule.map(|r| r.rules().len()) })
    }

    /// Drops a session: the UPF's side, the address.
    fn remove(&mut self, ctx: &mut Context<'_>, id: u32) {
        let Some(c) = self.contexts.remove(&id) else { return };
        self.pools[c.dnn].release(c.ue_ip);
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

/// The PDU Session Resource Setup Request Transfer for the UPF's tunnel `n3`
/// and the PCF's `rule`.
fn setup_transfer(n3: (Ipv4Addr, u32), rule: &SessionRule) -> Result<Vec<u8>, String> {
    n2sm::setup_request_transfer(n3.0, n3.1, rule)
}

impl ExecutionObject for Smf {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        let dnns: Vec<String> =
            self.config.dnns.iter().map(|d| format!("{} ({}/{})", d.dnn, d.pool.0, d.pool.1)).collect();
        log!(
            self.log,
            ctx,
            Vip,
            "SMF {} ready: DNN {}, UPF {:?}, PCF {:?}",
            self.name,
            dnns.join(", "),
            self.upf.name(),
            self.pcf.name()
        );
    }

    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if let Some(request) = msg.body::<CreateSmContext>() {
            self.on_create(ctx, msg, request);
        } else if let Some(result) = msg.body::<SmPolicyResult>() {
            self.on_policy(ctx, msg, result);
        } else if let Some(result) = msg.body::<N4EstablishResult>() {
            self.on_established(ctx, msg, result);
        } else if let Some(request) = msg.body::<UpdateSmContext>() {
            self.on_update(ctx, msg, request);
        } else if let Some(result) = msg.body::<N4ModifyResult>() {
            self.on_modified(ctx, msg, result);
        } else if let Some(report) = msg.body::<N4Report>() {
            self.on_report(ctx, report.seid as u32);
        } else if let Some(result) = msg.body::<SmNotifyResult>() {
            // The UE could not be reached: the session ends here.
            if result.outcome() != Outcome::Ok {
                self.remove(ctx, result.sm_ctx);
            }
        } else if let Some(request) = msg.body::<Manage>() {
            self.on_manage(ctx, msg, request);
        } else if let Some(request) = msg.body::<ReleaseSmContext>() {
            self.remove(ctx, request.sm_ctx);
            let to = (msg.from(), msg.corr_id());
            self.update_result(ctx, to, 0, Answer { released: true, ..Answer::default() });
        }
    }

    fn on_stop(&mut self, ctx: &mut Context<'_>) {
        log!(self.log, ctx, Info, "SMF {} stopping, {} session(s)", self.name, self.contexts.len());
    }
}
