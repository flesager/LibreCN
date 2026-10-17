//! UPF, the user plane function, hosted in one EO.
//!
//! - N4 (from the SMF, EO messages): sessions, each with the UE's address,
//!   the uplink tunnel the UPF allocates (its N3 address and a TEID) and the
//!   downlink one, the gNB's.
//! - N3: GTP-U on UDP port 2152 of the `n3` address. An uplink G-PDU goes
//!   to N6 if its TEID and inner source address match a session.
//! - N6: a TUN device (`n6`). A packet to a UE's address goes to its gNB in
//!   a G-PDU with the PDU Session Container (QFI 1, the default flow).
//!
//! Without `n6` the UPF keeps sessions and forwards nothing (no socket is
//! opened). Forwarding uses one preallocated buffer: no allocation per
//! packet. The EO is event-driven; a polling mode can come later, measured.
//!
//! - [`sessions`], the session table;
//! - [`tun`], the N6 device.

pub mod sessions;
pub mod tun;

use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};

use librecn_common::alarm::{Alarms, Severity};
use librecn_common::log;
use librecn_common::log::Logger;
use librecn_common::msg::flags;
use librecn_gtpu::{G_PDU_HEADER, Message as Gtp};
use librecn_sbi::Outcome;
use librecn_sbi::pdu_session::{N4Delete, N4Establish, N4EstablishResult, N4Modify, N4ModifyResult};
use librecn_scheduler::{Context, EoConfig, ExecutionObject, Interest, IoToken, Message};
use serde::Deserialize;

use sessions::{Refusal, Sessions};

/// Packets moved per readiness, so one busy side cannot hold the worker.
const PACKETS_PER_TURN: usize = 64;
/// Largest packet: an IPv4 datagram, with room for the GTP-U header.
const BUFFER: usize = G_PDU_HEADER + 65_535;
/// The default QoS flow (the SMF's).
const QFI: u8 = 1;
/// Alarm raised while the user plane cannot run.
const ALARM_USER_PLANE: u32 = 0;

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Params {
    /// N3 address, announced to the gNBs and bound for GTP-U. Not
    /// 127.0.0.1 on a host with UERANSIM: its gNB binds that address.
    n3: Ipv4Addr,
    /// N6 TUN device; none: no user plane.
    n6: Option<String>,
    max_sessions: usize,
}

impl Default for Params {
    fn default() -> Self {
        Params { n3: Ipv4Addr::new(127, 0, 0, 2), n6: None, max_sessions: 1024 }
    }
}

/// Packets and drops, for the log.
#[derive(Default, Debug)]
struct Counters {
    uplink: u64,
    downlink: u64,
    dropped: u64,
}

/// The UPF EO.
pub struct Upf {
    name: String,
    n3: Ipv4Addr,
    n6_name: Option<String>,
    sessions: Sessions,
    /// GTP-U socket and N6 device, once open.
    n3_socket: Option<(UdpSocket, IoToken)>,
    n6: Option<(File, Option<IoToken>)>,
    buffer: Box<[u8]>,
    counters: Counters,
    log: Logger,
    alarms: Alarms,
}

impl Upf {
    /// From deployment `params`: `n3` (default 127.0.0.2), `n6` (a TUN
    /// device name, default none) and `max_sessions` (1024).
    pub fn new(name: &str, params: &serde_json::Value) -> Result<Self, String> {
        let p: Params = if params.is_null() {
            Params::default()
        } else {
            serde_json::from_value(params.clone()).map_err(|e| format!("invalid UPF parameters: {e}"))?
        };
        Ok(Upf {
            name: name.to_string(),
            n3: p.n3,
            n6_name: p.n6,
            sessions: Sessions::new(p.max_sessions),
            n3_socket: None,
            n6: None,
            buffer: vec![0; BUFFER].into_boxed_slice(),
            counters: Counters::default(),
            log: Logger::default(),
            alarms: Alarms::new(),
        })
    }

    /// For the control EO's factory.
    pub fn from_params(name: &str, params: &serde_json::Value) -> Result<(EoConfig, Self), String> {
        Ok((Self::eo_config(name), Self::new(name, params)?))
    }

    /// N4 messages are small; N3 and N6 are two I/O registrations.
    pub fn eo_config(name: &str) -> EoConfig {
        EoConfig::new(name).max_io(2).max_timers(0).buffers(&[(256, 64)])
    }

    /// N6 on an already open, non-blocking packet device instead of the
    /// `n6` TUN (tests: one end of a datagram socket pair).
    pub fn with_n6(mut self, device: File) -> Self {
        self.n6 = Some((device, None));
        self
    }

    /// Opens N3 and N6, if a device is configured.
    fn open_user_plane(&mut self, ctx: &mut Context<'_>) -> Result<(), String> {
        if self.n6.is_none() {
            let Some(name) = &self.n6_name else { return Ok(()) };
            let device = tun::open(name).map_err(|e| format!("N6 device {name}: {e}"))?;
            self.n6 = Some((device, None));
        }
        let socket = UdpSocket::bind(SocketAddrV4::new(self.n3, librecn_gtpu::PORT))
            .and_then(|s| s.set_nonblocking(true).map(|()| s))
            .map_err(|e| format!("N3 on {}:{}: {e}", self.n3, librecn_gtpu::PORT))?;
        let token = ctx.register_io(&socket, Interest::READABLE).map_err(|e| format!("N3: {e}"))?;
        self.n3_socket = Some((socket, token));
        if let Some((device, token)) = &mut self.n6 {
            *token = Some(ctx.register_io(&*device, Interest::READABLE).map_err(|e| format!("N6: {e}"))?);
        }
        Ok(())
    }

    /// GTP-U datagrams from the gNBs.
    fn on_n3(&mut self, ctx: &mut Context<'_>) {
        let (Some((socket, _)), Some((device, _))) = (&self.n3_socket, &mut self.n6) else { return };
        for _ in 0..PACKETS_PER_TURN {
            let (n, from) = match socket.recv_from(&mut self.buffer) {
                Ok(received) => received,
                Err(e) if e.kind() == ErrorKind::WouldBlock => return,
                Err(e) => return log!(self.log, ctx, Error, "N3 receive: {e}"),
            };
            match librecn_gtpu::decode(&self.buffer[..n]) {
                Ok(Gtp::GPdu { teid, payload, .. }) => {
                    // The inner source must be the session's UE (no spoofing).
                    let session = self.sessions.by_teid(teid);
                    if session.is_some_and(|s| ipv4_src(payload) == Some(s.ue_ip)) && device.write(payload).is_ok() {
                        self.counters.uplink += 1;
                    } else {
                        self.counters.dropped += 1;
                    }
                }
                Ok(Gtp::EchoRequest { seq }) => {
                    let mut response = [0u8; 14];
                    let len = librecn_gtpu::echo_response(seq, &mut response);
                    let _ = socket.send_to(&response[..len], from);
                }
                Ok(Gtp::Other { .. }) | Err(_) => self.counters.dropped += 1,
            }
        }
    }

    /// IP packets from the data network, to the UEs.
    fn on_n6(&mut self, ctx: &mut Context<'_>) {
        let (Some((socket, _)), Some((device, _))) = (&self.n3_socket, &mut self.n6) else { return };
        for _ in 0..PACKETS_PER_TURN {
            // The packet lands after the room left for its GTP-U header.
            let n = match device.read(&mut self.buffer[G_PDU_HEADER..]) {
                Ok(0) => return,
                Ok(n) => n,
                Err(e) if e.kind() == ErrorKind::WouldBlock => return,
                Err(e) => return log!(self.log, ctx, Error, "N6 read: {e}"),
            };
            let packet = &self.buffer[G_PDU_HEADER..G_PDU_HEADER + n];
            let Some((gnb, teid)) = ipv4_dst(packet).and_then(|dst| self.sessions.by_ue(dst)).and_then(|s| s.dl) else {
                self.counters.dropped += 1;
                continue;
            };
            let header: &mut [u8; G_PDU_HEADER] = (&mut self.buffer[..G_PDU_HEADER]).try_into().expect("16 bytes");
            librecn_gtpu::write_g_pdu_header(header, teid, QFI, n);
            match socket.send_to(&self.buffer[..G_PDU_HEADER + n], SocketAddr::V4(gnb)) {
                Ok(_) => self.counters.downlink += 1,
                Err(_) => self.counters.dropped += 1,
            }
        }
    }

    fn on_establish(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &N4Establish) {
        let result = self.sessions.establish(request.seid, Ipv4Addr::from(request.ue_ip));
        match result {
            Ok(s) => {
                log!(self.log, ctx, Info, "session {:#x}: UE {}, uplink TEID {:#x}", request.seid, s.ue_ip, s.ul_teid)
            }
            Err(e) => log!(self.log, ctx, Error, "session {:#x} refused: {e:?}", request.seid),
        }
        let (n3, seid) = (self.n3, request.seid);
        let sent = ctx.alloc::<N4EstablishResult>(msg.from()).and_then(|r| {
            let mut r = r.with_corr_id(msg.corr_id()).with_flags(flags::RESPONSE);
            r.seid = seid;
            match result {
                Ok(s) => (r.n3_addr, r.ul_teid) = (n3.octets(), s.ul_teid),
                Err(Refusal::Full | Refusal::AddressInUse) => r.set_outcome(Outcome::Unavailable),
            }
            ctx.send(r)
        });
        if let Err(e) = sent {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", msg.from());
        }
    }

    fn on_modify(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &N4Modify) {
        let gnb = SocketAddrV4::new(Ipv4Addr::from(request.gnb_addr), librecn_gtpu::PORT);
        let outcome = if self.sessions.modify(request.seid, gnb, request.dl_teid) {
            log!(
                self.log,
                ctx,
                Info,
                "session {:#x}: downlink to {} TEID {:#x}",
                request.seid,
                gnb.ip(),
                request.dl_teid
            );
            Outcome::Ok
        } else {
            Outcome::ContextNotFound
        };
        let seid = request.seid;
        let sent = ctx.alloc::<N4ModifyResult>(msg.from()).and_then(|r| {
            let mut r = r.with_corr_id(msg.corr_id()).with_flags(flags::RESPONSE);
            r.seid = seid;
            r.set_outcome(outcome);
            ctx.send(r)
        });
        if let Err(e) = sent {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", msg.from());
        }
    }
}

/// Source address of an IPv4 packet.
fn ipv4_src(packet: &[u8]) -> Option<Ipv4Addr> {
    match packet {
        [v, _, _, _, _, _, _, _, _, _, _, _, a, b, c, d, ..] if v >> 4 == 4 => Some(Ipv4Addr::new(*a, *b, *c, *d)),
        _ => None,
    }
}

/// Destination address of an IPv4 packet.
fn ipv4_dst(packet: &[u8]) -> Option<Ipv4Addr> {
    match packet {
        [v, _, _, _, _, _, _, _, _, _, _, _, _, _, _, _, a, b, c, d, ..] if v >> 4 == 4 => {
            Some(Ipv4Addr::new(*a, *b, *c, *d))
        }
        _ => None,
    }
}

impl ExecutionObject for Upf {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        match self.open_user_plane(ctx) {
            Ok(()) if self.n6.is_some() => {
                let n6 = self.n6_name.as_deref().unwrap_or("given device");
                log!(self.log, ctx, Vip, "UPF {} ready, N3 {}:{}, N6 {n6}", self.name, self.n3, librecn_gtpu::PORT);
            }
            Ok(()) => log!(self.log, ctx, Vip, "UPF {} ready, N3 {}, no N6: sessions only", self.name, self.n3),
            Err(e) => {
                log!(self.log, ctx, Fatal, "UPF {}: no user plane: {e}", self.name);
                self.alarms.raise(ctx, ALARM_USER_PLANE, Severity::Critical, &format!("no user plane: {e}"));
            }
        }
    }

    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if let Some(ready) = msg.io() {
            if self.n3_socket.as_ref().is_some_and(|(_, t)| *t == ready.token) {
                self.on_n3(ctx);
            } else {
                self.on_n6(ctx);
            }
        } else if let Some(request) = msg.body::<N4Establish>() {
            self.on_establish(ctx, msg, request);
        } else if let Some(request) = msg.body::<N4Modify>() {
            self.on_modify(ctx, msg, request);
        } else if let Some(request) = msg.body::<N4Delete>()
            && self.sessions.delete(request.seid).is_some()
        {
            log!(self.log, ctx, Info, "session {:#x} deleted, {} left", request.seid, self.sessions.len());
        }
    }

    fn on_stop(&mut self, ctx: &mut Context<'_>) {
        let c = &self.counters;
        log!(
            self.log,
            ctx,
            Info,
            "UPF {} stopping, {} session(s), {} packet(s) up, {} down, {} dropped",
            self.name,
            self.sessions.len(),
            c.uplink,
            c.downlink,
            c.dropped
        );
    }
}
