//! AMF, the access and mobility management function, hosted in one EO.
//!
//! This release covers the N2 interface setup: SCTP (one-to-many) on the N2
//! address, NG Setup, NG Reset, Error Indication.
//!
//! - [`sctp`], the transport;
//! - `n2`, NGAP ↔ domain values (the only module seeing ASN.1);
//! - `setup`, the NG Setup decision, on domain values;
//! - [`config`], identity and addresses, from the deployment `params`.

pub mod config;
mod n2;
pub mod sctp;
mod setup;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use librecn_common::alarm::{Alarms, Severity};
use librecn_common::log;
use librecn_common::log::Logger;
use librecn_ngap::{CauseProtocol, Criticality, procedure_name};
use librecn_scheduler::{Context, EoConfig, ExecutionObject, Interest, IoToken, Message};

use config::AmfConfig;
use n2::Incoming;
use sctp::{AssocId, NGAP_PPID, Received, SctpSocket};
use setup::RanNodeId;

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
}

/// The AMF EO.
pub struct Amf {
    config: AmfConfig,
    socket: Option<(SctpSocket, IoToken)>,
    bound: Arc<Mutex<Option<SocketAddr>>>,
    gnbs: HashMap<AssocId, Gnb>,
    buffer: Box<[u8]>,
    log: Logger,
    alarms: Alarms,
}

impl Amf {
    /// An AMF with this configuration; N2 opens when the EO starts.
    pub fn new(config: AmfConfig) -> Self {
        Amf {
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

    /// N2 traffic arrives through the SCTP socket, not the mailbox: small lanes and buffers.
    pub fn eo_config(name: &str) -> EoConfig {
        EoConfig::new(name).max_io(1).max_timers(4).system_capacity(16).buffers(&[(256, 64)])
    }

    /// The N2 address actually bound, once the EO started (port 0: chosen by the kernel).
    pub fn bound_addr(&self) -> Arc<Mutex<Option<SocketAddr>>> {
        self.bound.clone()
    }

    fn send(&mut self, ctx: &mut Context<'_>, assoc: AssocId, pdu: Result<Vec<u8>, String>, what: &str) {
        let result = pdu.and_then(|bytes| {
            let (socket, _) = self.socket.as_ref().ok_or("N2 closed")?;
            socket.send(assoc, 0, NGAP_PPID, &bytes).map_err(|e| e.to_string())
        });
        if let Err(e) = result {
            log!(self.log, ctx, Error, "cannot send {what} on association {assoc}: {e}");
        }
    }

    fn on_received(&mut self, ctx: &mut Context<'_>, received: Received<'_>) {
        match received {
            Received::Up { assoc, outbound_streams } => {
                log!(self.log, ctx, Info, "SCTP association {assoc} up, {outbound_streams} streams");
                self.gnbs.insert(assoc, Gnb { node: None });
            }
            Received::Down { assoc } => match self.gnbs.remove(&assoc).and_then(|g| g.node) {
                Some((node, name)) => {
                    let label = name.unwrap_or_else(|| node.to_string());
                    log!(self.log, ctx, Error, "N2 to {label} lost (association {assoc})");
                    self.alarms.raise(ctx, node.key(), Severity::Major, &format!("N2 to {label} lost"));
                }
                None => log!(self.log, ctx, Info, "SCTP association {assoc} closed before NG Setup"),
            },
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
            Incoming::NgSetup(request) => match setup::decide(&self.config, &request) {
                Ok(()) => {
                    let response = n2::ng_setup_response(&self.config);
                    self.send(ctx, assoc, response, "NG Setup Response");
                    let label = request.name.clone().unwrap_or_else(|| request.node.to_string());
                    log!(
                        self.log,
                        ctx,
                        Info,
                        "NG Setup accepted: {label} ({}), {} TA(s)",
                        request.node,
                        request.tas.len()
                    );
                    self.alarms.clear(ctx, request.node.key());
                    self.gnbs.entry(assoc).or_insert(Gnb { node: None }).node = Some((request.node, request.name));
                }
                Err(refusal) => {
                    self.send(ctx, assoc, n2::ng_setup_failure(refusal), "NG Setup Failure");
                    log!(self.log, ctx, Info, "NG Setup refused for {}: {refusal:?}", request.node);
                }
            },
            Incoming::NgReset if set_up => {
                self.send(ctx, assoc, n2::ng_reset_acknowledge(), "NG Reset Acknowledge");
                log!(self.log, ctx, Info, "NG Reset from association {assoc}: acknowledged");
            }
            Incoming::ErrorIndication(cause) => {
                log!(self.log, ctx, Error, "Error Indication from association {assoc}: {cause}");
            }
            Incoming::Malformed(e) => {
                log!(self.log, ctx, Error, "malformed NGAP from association {assoc}: {e}");
                let error = n2::error_indication(CauseProtocol::transfer_syntax_error);
                self.send(ctx, assoc, error, "Error Indication");
            }
            // NG Reset before NG Setup: nothing to reset.
            Incoming::NgReset => self.not_handled(ctx, assoc, librecn_ngap::ID_NGRESET.0, Criticality::reject),
            Incoming::Unsupported { procedure, criticality } => self.not_handled(ctx, assoc, procedure, criticality),
        }
    }

    /// A procedure not handled now: reported unless the sender asked to
    /// ignore it silently (TS 38.413 §10.3).
    fn not_handled(&mut self, ctx: &mut Context<'_>, assoc: AssocId, procedure: u8, criticality: Criticality) {
        let name = procedure_name(procedure).unwrap_or("unknown procedure");
        log!(self.log, ctx, Info, "{name} not handled (association {assoc})");
        if criticality != Criticality::ignore {
            let error = n2::error_indication(CauseProtocol::message_not_compatible_with_receiver_state);
            self.send(ctx, assoc, error, "Error Indication");
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
            return;
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
        log!(self.log, ctx, Info, "AMF {} stopping, {} gNB association(s)", self.config.name, self.gnbs.len());
    }
}
