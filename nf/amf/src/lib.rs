//! AMF, the access and mobility management function, hosted in one EO.
//!
//! This release covers N2 (SCTP one-to-many, NG Setup, NG Reset, Error
//! Indication, UE-associated signalling), registration (5G AKA through the
//! AUSF, NAS security, Registration Accept in Initial Context Setup,
//! UE-originating deregistration) and PDU session establishment through the
//! SMF (PDU Session Resource Setup).
//!
//! - [`sctp`], the transport;
//! - `n2`, NGAP ↔ domain values (the only module seeing ASN.1);
//! - `setup`, the NG Setup decision, on domain values;
//! - `ue`, UE contexts and AMF-UE-NGAP-ID allocation;
//! - `mm`, 5GMM for one UE, on domain values;
//! - `sessions`, PDU sessions between the UE, the SMF and the gNB;
//! - [`config`], identity and addresses, from the deployment `params`.
//!
//! The AUSF and the SMF are reached by EO messages (`librecn-sbi`); the
//! `corr_id` of a request is the UE's AMF-UE-NGAP-ID.

pub mod config;
mod mm;
mod n2;
pub mod sctp;
mod sessions;
mod setup;
mod ue;

use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use librecn_common::Service;
use librecn_common::alarm::{Alarms, Severity};
use librecn_common::log;
use librecn_common::log::{Level, Logger};
use librecn_common::msg::flags;
use librecn_nas::ie::Tai;
use librecn_ngap::{
    Cause, CauseNas, CauseProtocol, CauseRadioNetwork, Criticality, ID_INITIAL_UEMESSAGE, ID_NGRESET,
    ID_UECONTEXT_RELEASE_REQUEST, ID_UPLINK_NASTRANSPORT, procedure_name,
};
use librecn_sbi::pdu_session::{CreateSmContext, CreateSmContextResult, Slice, UpdateSmContextResult};
use librecn_sbi::ue_authentication::{AuthInfo, Authenticate, AuthenticateResult, Confirm, ConfirmResult};
use librecn_sbi::{DnnText, ServingNetwork, SupiOrSuci};
use librecn_scheduler::{Context, EoConfig, ExecutionObject, Interest, IoToken, Message, MsgBody};

use config::AmfConfig;
use mm::{AsContext, Effects, Release, SessionRequest};
use n2::Incoming;
use sctp::{AssocId, NGAP_PPID, Received, SctpSocket};
use setup::RanNodeId;
use ue::{AmfUeId, Location, Mismatch, PduSession, RanUeId, UeTable};

/// Largest NGAP message accepted (a larger one is dropped and logged).
const RECEIVE_BUFFER: usize = 64 * 1024;
/// Messages read per readiness, so one busy gNB cannot hold the worker.
const MESSAGES_PER_TURN: usize = 32;
/// SCTP streams offered each way; stream 0 carries non-UE signalling.
const STREAMS: u16 = 16;
/// Alarm raised while N2 cannot be served.
const ALARM_N2_DOWN: u32 = 0;

/// A gNB association; `node` is set once NG Setup succeeded.
struct Gnb {
    node: Option<(RanNodeId, Option<String>)>,
    /// Outbound streams of the association.
    streams: u16,
}

/// The AMF EO.
pub struct Amf {
    config: AmfConfig,
    serving_network: ServingNetwork,
    ausf: Service,
    smf: Service,
    socket: Option<(SctpSocket, IoToken)>,
    bound: Arc<Mutex<Option<SocketAddr>>>,
    gnbs: HashMap<AssocId, Gnb>,
    ues: UeTable,
    buffer: Box<[u8]>,
    log: Logger,
    alarms: Alarms,
}

impl Amf {
    /// An AMF with this configuration; N2 opens when the EO starts.
    pub fn new(config: AmfConfig) -> Self {
        Amf {
            ues: UeTable::new(config.max_ues),
            serving_network: config.serving_network(),
            ausf: Service::named(config.ausf.clone()),
            smf: Service::named(config.smf.clone()),
            config,
            socket: None,
            bound: Arc::default(),
            gnbs: HashMap::new(),
            buffer: vec![0; RECEIVE_BUFFER].into_boxed_slice(),
            log: Logger::default(),
            alarms: Alarms::new(),
        }
    }

    /// For the control EO's factory: EO sizes and the AMF from deployment `params`.
    pub fn from_params(name: &str, params: &serde_json::Value) -> Result<(EoConfig, Self), String> {
        let config = AmfConfig::from_params(name, params)?;
        Ok((Self::eo_config(name), Self::new(config)))
    }

    /// N2 traffic arrives through the SCTP socket, not the mailbox: small
    /// lanes, and buffers for the AUSF's and the SMF's answers.
    pub fn eo_config(name: &str) -> EoConfig {
        EoConfig::new(name).max_io(1).max_timers(4).system_capacity(16).buffers(&[(256, 64), (2048, 16)])
    }

    /// The N2 address actually bound, once the EO started (port 0: chosen by the kernel).
    pub fn bound_addr(&self) -> Arc<Mutex<Option<SocketAddr>>> {
        self.bound.clone()
    }

    fn send(&mut self, ctx: &mut Context<'_>, assoc: AssocId, stream: u16, pdu: Result<Vec<u8>, String>, what: &str) {
        let result = pdu.and_then(|bytes| {
            let (socket, _) = self.socket.as_ref().ok_or("N2 closed")?;
            socket.send(assoc, stream, NGAP_PPID, &bytes).map_err(|e| e.to_string())
        });
        if let Err(e) = result {
            log!(self.log, ctx, Error, "cannot send {what} on association {assoc}: {e}");
        }
    }

    fn error_indication(
        &mut self,
        ctx: &mut Context<'_>,
        assoc: AssocId,
        ids: Option<(AmfUeId, RanUeId)>,
        cause: Cause,
    ) {
        self.send(ctx, assoc, 0, n2::error_indication(ids, cause), "Error Indication");
    }

    fn on_received(&mut self, ctx: &mut Context<'_>, received: Received<'_>) {
        match received {
            Received::Up { assoc, outbound_streams } => {
                log!(self.log, ctx, Info, "SCTP association {assoc} up, {outbound_streams} streams");
                self.gnbs.insert(assoc, Gnb { node: None, streams: outbound_streams });
            }
            Received::Down { assoc } => {
                let released = self.forget_gnb(ctx, assoc);
                match self.gnbs.remove(&assoc).and_then(|g| g.node) {
                    Some((node, name)) => {
                        let label = name.unwrap_or_else(|| node.to_string());
                        log!(
                            self.log,
                            ctx,
                            Error,
                            "N2 to {label} lost (association {assoc}), {released} UE(s) released"
                        );
                        self.alarms.raise(ctx, node.key(), Severity::Major, &format!("N2 to {label} lost"));
                    }
                    None => log!(self.log, ctx, Info, "SCTP association {assoc} closed before NG Setup"),
                }
            }
            Received::Truncated { assoc } => {
                log!(self.log, ctx, Error, "message over {RECEIVE_BUFFER} bytes dropped (association {assoc})");
            }
            Received::Data { assoc, ppid, .. } if ppid != NGAP_PPID => {
                log!(self.log, ctx, Error, "payload protocol {ppid} is not NGAP (association {assoc}): dropped");
            }
            Received::Data { assoc, data, .. } => self.on_ngap(ctx, assoc, data),
            Received::Ignored => {}
        }
    }

    fn on_ngap(&mut self, ctx: &mut Context<'_>, assoc: AssocId, data: &[u8]) {
        let set_up = self.gnbs.get(&assoc).is_some_and(|g| g.node.is_some());
        match n2::decode(data) {
            Incoming::NgSetup(request) => self.on_ng_setup(ctx, assoc, request),
            Incoming::NgReset if set_up => {
                let released = self.forget_gnb(ctx, assoc);
                self.send(ctx, assoc, 0, n2::ng_reset_acknowledge(), "NG Reset Acknowledge");
                log!(self.log, ctx, Info, "NG Reset from association {assoc}: {released} UE(s) released, acknowledged");
            }
            Incoming::ErrorIndication(cause) => {
                log!(self.log, ctx, Error, "Error Indication from association {assoc}: {cause}");
            }
            Incoming::InitialUe { ran_id, mut nas, location } if set_up => {
                self.on_initial_ue(ctx, assoc, ran_id, &mut nas, location)
            }
            Incoming::UplinkNas { amf_id, ran_id, mut nas, location } if set_up => {
                match self.ues.get_mut(assoc, amf_id, ran_id) {
                    Ok(ue) => {
                        if location.is_some() {
                            ue.location = location;
                        }
                        self.on_nas(ctx, amf_id, &mut nas);
                    }
                    Err(mismatch) => self.unknown_ue(ctx, assoc, amf_id, ran_id, mismatch),
                }
            }
            Incoming::ReleaseRequest { amf_id, ran_id, cause } if set_up => {
                match self.ues.get_mut(assoc, amf_id, ran_id) {
                    Ok(_) => {
                        log!(self.log, ctx, Info, "UE {amf_id}: release requested by the gNB ({cause})");
                        self.release(ctx, amf_id, Cause::radioNetwork(CauseRadioNetwork::unspecified));
                    }
                    Err(mismatch) => self.unknown_ue(ctx, assoc, amf_id, ran_id, mismatch),
                }
            }
            Incoming::ReleaseComplete { amf_id, ran_id } => match self.ues.get_mut(assoc, amf_id, ran_id) {
                Ok(_) => {
                    let gone = self.ues.remove(amf_id);
                    let supi = gone.as_ref().and_then(|ue| ue.mm.supi().copied());
                    if let Some(ue) = &gone {
                        self.forget_sessions(ctx, ue);
                    }
                    let left = self.ues.len();
                    match supi {
                        Some(supi) => log!(self.log, ctx, Info, "UE {amf_id} ({supi}): released, {left} UE(s) left"),
                        None => log!(self.log, ctx, Info, "UE {amf_id}: released, {left} UE(s) left"),
                    }
                }
                // A late Complete for a context already gone: nothing to do (criticality ignore).
                Err(_) => log!(self.log, ctx, Debug, "UE Context Release Complete for unknown UE {amf_id}"),
            },
            Incoming::ContextSetup { amf_id, ran_id, failure } => {
                match (self.ues.get_mut(assoc, amf_id, ran_id), failure) {
                    (Ok(_), None) => log!(self.log, ctx, Info, "UE {amf_id}: Initial Context Setup done"),
                    (Ok(_), Some(cause)) => {
                        log!(self.log, ctx, Error, "UE {amf_id}: Initial Context Setup failed ({cause})");
                        self.release(ctx, amf_id, Cause::nas(CauseNas::unspecified));
                    }
                    (Err(mismatch), _) => self.unknown_ue(ctx, assoc, amf_id, ran_id, mismatch),
                }
            }
            Incoming::SessionSetup { amf_id, ran_id, setup, failed } => match self.ues.get_mut(assoc, amf_id, ran_id) {
                Ok(_) => self.on_session_setup(ctx, amf_id, setup, failed),
                Err(mismatch) => self.unknown_ue(ctx, assoc, amf_id, ran_id, mismatch),
            },
            Incoming::Malformed(e) => {
                log!(self.log, ctx, Error, "malformed NGAP from association {assoc}: {e}");
                self.error_indication(ctx, assoc, None, Cause::protocol(CauseProtocol::transfer_syntax_error));
            }
            // Before NG Setup: only NG Setup is acceptable.
            Incoming::NgReset => self.not_handled(ctx, assoc, ID_NGRESET.0, Criticality::reject),
            Incoming::InitialUe { .. } => self.not_handled(ctx, assoc, ID_INITIAL_UEMESSAGE.0, Criticality::reject),
            Incoming::UplinkNas { .. } => self.not_handled(ctx, assoc, ID_UPLINK_NASTRANSPORT.0, Criticality::reject),
            Incoming::ReleaseRequest { .. } => {
                self.not_handled(ctx, assoc, ID_UECONTEXT_RELEASE_REQUEST.0, Criticality::reject);
            }
            Incoming::Unsupported { procedure, criticality } => self.not_handled(ctx, assoc, procedure, criticality),
        }
    }

    fn on_ng_setup(&mut self, ctx: &mut Context<'_>, assoc: AssocId, request: setup::SetupRequest) {
        match setup::decide(&self.config, &request) {
            Ok(()) => {
                let response = n2::ng_setup_response(&self.config);
                self.send(ctx, assoc, 0, response, "NG Setup Response");
                let label = request.name.clone().unwrap_or_else(|| request.node.to_string());
                log!(self.log, ctx, Info, "NG Setup accepted: {label} ({}), {} TA(s)", request.node, request.tas.len());
                self.alarms.clear(ctx, request.node.key());
                let gnb = self.gnbs.entry(assoc).or_insert(Gnb { node: None, streams: 1 });
                gnb.node = Some((request.node, request.name));
            }
            Err(refusal) => {
                self.send(ctx, assoc, 0, n2::ng_setup_failure(refusal), "NG Setup Failure");
                log!(self.log, ctx, Info, "NG Setup refused for {}: {refusal:?}", request.node);
            }
        }
    }

    fn on_initial_ue(
        &mut self,
        ctx: &mut Context<'_>,
        assoc: AssocId,
        ran_id: RanUeId,
        nas: &mut [u8],
        location: Option<Location>,
    ) {
        // The gNB reuses a RAN-UE-NGAP-ID only for a new UE: the old context is stale.
        if let Some(stale) = self.ues.find(assoc, ran_id) {
            if let Some(ue) = self.ues.remove(stale) {
                self.forget_sessions(ctx, &ue);
            }
            log!(self.log, ctx, Info, "UE {stale}: replaced by a new Initial UE Message for RAN id {ran_id}");
        }
        let streams = self.gnbs.get(&assoc).map_or(1, |g| g.streams);
        let Some(ue) = self.ues.create(assoc, ran_id, streams) else {
            log!(
                self.log,
                ctx,
                Error,
                "UE table full ({} UEs): Initial UE Message for RAN id {ran_id} dropped",
                self.config.max_ues
            );
            return;
        };
        let amf_id = ue.amf_id;
        self.ues.get_mut(assoc, amf_id, ran_id).expect("just created").location = location;
        log!(self.log, ctx, Info, "UE {amf_id} (RAN id {ran_id}, association {assoc}): Initial UE Message");
        self.on_nas(ctx, amf_id, nas);
    }

    /// Runs `step` on the UE's 5GMM context, then releases the UE if 5GMM asked.
    fn with_mm(
        &mut self,
        ctx: &mut Context<'_>,
        amf_id: AmfUeId,
        step: impl FnOnce(&mut mm::Mm, &AmfConfig, Option<Tai>, &mut Port<'_, '_, '_>),
    ) {
        let Some(ue) = self.ues.get_mut_by_id(amf_id) else {
            return log!(self.log, ctx, Debug, "UE {amf_id} gone: answer dropped");
        };
        let tai = ue.location.map(|l| Tai { plmn: l.tai.0, tac: l.tai.1 });
        let mut port = Port {
            ctx,
            socket: self.socket.as_ref().map(|(s, _)| s),
            log: &mut self.log,
            ausf: &mut self.ausf,
            smf: &mut self.smf,
            config: &self.config,
            serving_network: &self.serving_network,
            sessions: &mut ue.sessions,
            ids: (amf_id, ue.ran_id, ue.assoc, ue.stream),
            release: None,
        };
        step(&mut ue.mm, &self.config, tai, &mut port);
        if let Some(why) = port.release {
            let cause = match why {
                Release::Normal => CauseNas::normal_release,
                Release::Deregister => CauseNas::deregister,
                Release::AuthenticationFailure => CauseNas::authentication_failure,
            };
            self.release(ctx, amf_id, Cause::nas(cause));
        }
    }

    /// An uplink NAS PDU for the UE's 5GMM context.
    fn on_nas(&mut self, ctx: &mut Context<'_>, amf_id: AmfUeId, pdu: &mut [u8]) {
        self.with_mm(ctx, amf_id, |mm, config, tai, port| mm.on_uplink(config, tai, pdu, port));
    }

    /// An answer of the AUSF or the SMF, for the UE its `corr_id` names.
    fn on_peer(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>) {
        let amf_id = AmfUeId::from(msg.corr_id());
        if let Some(result) = msg.body::<AuthenticateResult>() {
            self.with_mm(ctx, amf_id, |mm, _, _, port| mm.on_authenticate_result(result, port));
        } else if let Some(result) = msg.body::<ConfirmResult>() {
            self.with_mm(ctx, amf_id, |mm, config, _, port| mm.on_confirm_result(config, result, port));
        } else if let Some(result) = msg.body::<CreateSmContextResult>() {
            self.on_sm_created(ctx, amf_id, result);
        } else if let Some(result) = msg.body::<UpdateSmContextResult>() {
            let (psi, outcome) = (result.pdu_session_id, result.outcome());
            match outcome {
                librecn_sbi::Outcome::Ok => log!(self.log, ctx, Info, "UE {amf_id}: PDU session {psi} set up"),
                _ => log!(self.log, ctx, Error, "UE {amf_id}: PDU session {psi} not updated: {outcome:?}"),
            }
        }
    }

    /// Sends UE Context Release Command; the context goes on the Complete.
    fn release(&mut self, ctx: &mut Context<'_>, amf_id: AmfUeId, cause: Cause) {
        let Some(ue) = self.ues.get_mut_by_id(amf_id) else { return };
        ue.releasing = true;
        let (assoc, stream, ran_id) = (ue.assoc, ue.stream, ue.ran_id);
        let command = n2::ue_context_release_command(amf_id, ran_id, cause);
        self.send(ctx, assoc, stream, command, "UE Context Release Command");
    }

    /// TS 38.413 §10.6: unknown or inconsistent UE ids are reported.
    fn unknown_ue(
        &mut self,
        ctx: &mut Context<'_>,
        assoc: AssocId,
        amf_id: AmfUeId,
        ran_id: RanUeId,
        mismatch: Mismatch,
    ) {
        log!(self.log, ctx, Error, "UE ids AMF {amf_id} / RAN {ran_id} on association {assoc}: {mismatch:?}");
        let cause = match mismatch {
            Mismatch::UnknownAmfId => CauseRadioNetwork::unknown_local_UE_NGAP_ID,
            Mismatch::InconsistentRanId => CauseRadioNetwork::inconsistent_remote_UE_NGAP_ID,
        };
        self.error_indication(ctx, assoc, Some((amf_id, ran_id)), Cause::radioNetwork(cause));
    }

    /// A procedure not handled now: reported unless the sender asked to
    /// ignore it silently (TS 38.413 §10.3).
    fn not_handled(&mut self, ctx: &mut Context<'_>, assoc: AssocId, procedure: u8, criticality: Criticality) {
        let name = procedure_name(procedure).unwrap_or("unknown procedure");
        log!(self.log, ctx, Info, "{name} not handled (association {assoc})");
        if criticality != Criticality::ignore {
            self.error_indication(
                ctx,
                assoc,
                None,
                Cause::protocol(CauseProtocol::message_not_compatible_with_receiver_state),
            );
        }
    }
}

impl ExecutionObject for Amf {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        let opened = SctpSocket::listen(self.config.n2, STREAMS)
            .and_then(|socket| Ok((socket.local_addr()?, socket)))
            .map_err(|e| e.to_string())
            .and_then(|(addr, socket)| {
                let token = ctx.register_io(&socket, Interest::READABLE).map_err(|e| e.to_string())?;
                Ok((addr, socket, token))
            });
        match opened {
            Ok((addr, socket, token)) => {
                *self.bound.lock().unwrap() = Some(addr);
                self.socket = Some((socket, token));
                let c = &self.config;
                log!(
                    self.log,
                    ctx,
                    Vip,
                    "AMF {} ready: N2 on {addr}, PLMN {}, region {} set {} pointer {}",
                    c.name,
                    c.plmn,
                    c.amf_region,
                    c.amf_set,
                    c.amf_pointer
                );
            }
            Err(e) => {
                log!(self.log, ctx, Fatal, "cannot open N2 on {}: {e}", self.config.n2);
                self.alarms.raise(ctx, ALARM_N2_DOWN, Severity::Critical, &format!("N2 closed: {e}"));
            }
        }
    }

    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if msg.io().is_none() {
            return self.on_peer(ctx, msg);
        }
        let mut buffer = std::mem::take(&mut self.buffer);
        for _ in 0..MESSAGES_PER_TURN {
            let Some((socket, _)) = &self.socket else { break };
            match socket.recv(&mut buffer) {
                Ok(Some(received)) => self.on_received(ctx, received),
                Ok(None) => break,
                Err(e) => {
                    log!(self.log, ctx, Error, "N2 receive: {e}");
                    break;
                }
            }
        }
        self.buffer = buffer;
    }

    fn on_stop(&mut self, ctx: &mut Context<'_>) {
        log!(
            self.log,
            ctx,
            Info,
            "AMF {} stopping, {} gNB association(s), {} UE(s), {} registered",
            self.config.name,
            self.gnbs.len(),
            self.ues.len(),
            self.ues.registered()
        );
    }
}

/// Sends a request to a peer NF (AUSF, SMF) about UE `amf_id`, its
/// `corr_id`. False, logged, if the peer cannot be reached.
fn request<T: MsgBody>(
    ctx: &mut Context<'_>,
    peer: &mut Service,
    log: &mut Logger,
    amf_id: AmfUeId,
    fill: impl FnOnce(&mut T),
) -> bool {
    let Some(to) = peer.get(ctx) else {
        log!(log, ctx, Error, "no {:?} EO", peer.name());
        return false;
    };
    let sent = ctx.alloc::<T>(to).and_then(|m| {
        let mut m = m.with_corr_id(amf_id as u32).with_flags(flags::REQUEST);
        fill(&mut m);
        ctx.send(m)
    });
    if let Err(e) = &sent {
        peer.forget();
        log!(log, ctx, Error, "cannot reach {:?}: {e}", peer.name());
    }
    sent.is_ok()
}

/// 5GMM's effects for one UE: NAS over N2, requests to the AUSF and the SMF, logs.
struct Port<'a, 'c, 's> {
    ctx: &'a mut Context<'c>,
    socket: Option<&'s SctpSocket>,
    log: &'a mut Logger,
    ausf: &'a mut Service,
    smf: &'a mut Service,
    config: &'a AmfConfig,
    serving_network: &'a ServingNetwork,
    sessions: &'a mut [Option<PduSession>; 16],
    /// AMF-UE-NGAP-ID, RAN-UE-NGAP-ID, association, stream.
    ids: (AmfUeId, RanUeId, AssocId, u16),
    release: Option<Release>,
}

impl Port<'_, '_, '_> {
    /// Sends an NGAP PDU on the UE's association and stream.
    fn ngap(&mut self, pdu: Result<Vec<u8>, String>, what: &str) {
        let (amf_id, _, assoc, stream) = self.ids;
        let sent = pdu.and_then(|bytes| {
            let socket = self.socket.ok_or("N2 closed")?;
            socket.send(assoc, stream, NGAP_PPID, &bytes).map_err(|e| e.to_string())
        });
        match sent {
            Ok(()) => log!(self.log, self.ctx, Info, "UE {amf_id}: downlink {what}"),
            Err(e) => log!(self.log, self.ctx, Error, "UE {amf_id}: cannot send {what}: {e}"),
        }
    }
}

impl Effects for Port<'_, '_, '_> {
    fn send(&mut self, pdu: &[u8], what: &'static str) {
        let (amf_id, ran_id, ..) = self.ids;
        self.ngap(n2::downlink_nas_transport(amf_id, ran_id, pdu), what);
    }

    fn setup_context(&mut self, pdu: &[u8], context: &AsContext) {
        let (amf_id, ran_id, ..) = self.ids;
        let request = n2::initial_context_setup_request(amf_id, ran_id, self.config, context, pdu);
        self.ngap(request, "Registration Accept in Initial Context Setup");
    }

    fn session(&mut self, r: &SessionRequest<'_>) -> Result<(), u8> {
        let psi = r.pdu_session_id;
        if !r.initial || self.sessions[usize::from(psi)].is_some() {
            log!(self.log, self.ctx, Info, "UE {}: PDU session {psi}: only new sessions are served", self.ids.0);
            return Err(librecn_nas::mm::cause::PAYLOAD_NOT_FORWARDED);
        }
        let dnn = match r.dnn {
            Some(dnn) => DnnText::format(format_args!("{dnn}")),
            None => DnnText::new(""),
        };
        let sent = request::<CreateSmContext>(self.ctx, self.smf, self.log, self.ids.0, |m| {
            (m.supi, m.pdu_session_id) = (r.supi, psi);
            let sd = r.snssai.sd;
            m.slice = Slice { sst: r.snssai.sst, has_sd: u8::from(sd.is_some()), sd: sd.unwrap_or_default() };
            m.dnn = dnn.unwrap_or(m.dnn);
            m.n1.set(r.container);
        });
        if !sent {
            return Err(librecn_nas::mm::cause::PAYLOAD_NOT_FORWARDED);
        }
        self.sessions[usize::from(psi)] = Some(PduSession { snssai: r.snssai, sm_ctx: None });
        Ok(())
    }

    fn authenticate(&mut self, who: &SupiOrSuci, resync: Option<([u8; 16], [u8; 14])>) -> bool {
        let info = AuthInfo::new(*who, *self.serving_network);
        let info = match resync {
            Some((rand, auts)) => info.with_resync(rand, auts),
            None => info,
        };
        request::<Authenticate>(self.ctx, self.ausf, self.log, self.ids.0, |m| m.info = info)
    }

    fn confirm(&mut self, auth_ctx: u32, res_star: &[u8; 16]) -> bool {
        request::<Confirm>(self.ctx, self.ausf, self.log, self.ids.0, |m| {
            (m.auth_ctx, m.res_star) = (auth_ctx, *res_star)
        })
    }

    fn release(&mut self, why: Release) {
        self.release = Some(why);
    }

    fn tmsi(&mut self) -> u32 {
        // Unpredictable (TS 33.501 §6.12.3); the UE id if the kernel has no randomness.
        librecn_security::random().map_or(self.ids.0 as u32, u32::from_be_bytes)
    }

    fn log(&mut self, level: Level, args: fmt::Arguments<'_>) {
        let amf_id = self.ids.0;
        self.log.log(self.ctx, level, format_args!("UE {amf_id}: {args}"));
    }
}
