//! UPF, the user plane function, hosted in one EO.
//!
//! - N4 (from the SMF, EO messages): sessions, each with the UE's address,
//!   the uplink tunnel the UPF allocates (its N3 address and a TEID) and the
//!   downlink one, the gNB's.
//! - N3: GTP-U on UDP port 2152 of the `n3` address. An uplink G-PDU goes
//!   to N6 if its TEID and inner source address match a session, its QFI
//!   is one of the session's flows, and its rates allow it.
//! - N6: a TUN device (`n6`). A packet to a UE's address goes to its gNB in
//!   a G-PDU with the PDU Session Container: the QFI of its QoS flow, the
//!   first of the PCF's rules whose filters it matches, else the default
//!   flow, if its rates allow it (see [`qos`]). For
//!   an idle UE (no downlink tunnel) it is parked, and the SMF told
//!   (`N4Report`, at most every `REPORT_PERIOD`): the network pages the UE,
//!   whose tunnel then comes back with the parked packets.
//!
//! Without `n6` the UPF keeps sessions and forwards nothing (no socket is
//! opened). Forwarding uses one preallocated buffer: no allocation per
//! packet.
//!
//! `mode` (see `README.md` for the measurements):
//! - `event` (default): epoll wakes the UPF when N3 or N6 has packets;
//! - `poll`: no epoll; the UPF reads N3 and N6 at every turn of its worker,
//!   which never sleeps (a core for the user plane);
//! - `hybrid`: epoll wakes it, then it polls until [`EMPTY_POLLS`] polls in
//!   a row find nothing, and sleeps on epoll again.
//!
//! - [`sessions`], the session table;
//!
//! Traffic is also counted per gNB (`GET gnbs` on the web API), by the
//! gNB's key the SMF gives with each downlink tunnel: a slot per gNB in a
//! preallocated table of [`MAX_GNBS`].
//!
//! - [`qos`], the QoS flows and their rates;
//! - [`parked`], the downlink packets of idle UEs;
//! - [`tun`], the N6 device.

pub mod parked;
pub mod qos;
pub mod sessions;
pub mod tun;

use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};

use librecn_common::alarm::{Alarms, Severity};
use librecn_common::log;
use librecn_common::log::Logger;
use librecn_common::manage::{Manage, Method, send_reply};
use librecn_common::msg::flags;
use librecn_gtpu::{G_PDU_HEADER, Message as Gtp};
use librecn_sbi::Outcome;
use librecn_sbi::pdu_session::{
    N4Delete, N4Establish, N4EstablishResult, N4Modify, N4ModifyResult, N4Report, REPORT_PERIOD,
};
use librecn_sbi::policy::DEFAULT_QFI;
use librecn_scheduler::{Context, EoConfig, EoId, ExecutionObject, Interest, IoToken, Message, system};
use serde::Deserialize;

use parked::{Parked, Parking};
use sessions::{Refusal, Sessions};

/// Packets moved per readiness, so one busy side cannot hold the worker.
const PACKETS_PER_TURN: usize = 64;
/// Largest packet: an IPv4 datagram, with room for the GTP-U header.
const BUFFER: usize = G_PDU_HEADER + 65_535;
/// Alarm raised while the user plane cannot run.
const ALARM_USER_PLANE: u32 = 0;
/// gNBs whose traffic is counted apart; the others' only in the totals.
pub const MAX_GNBS: usize = 256;
/// Hybrid mode: polls in a row without a packet before the UPF sleeps.
pub const EMPTY_POLLS: u32 = 256;

/// How the UPF learns that packets wait.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Event,
    Poll,
    Hybrid,
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Params {
    /// N3 address, announced to the gNBs and bound for GTP-U. Not
    /// 127.0.0.1 on a host with UERANSIM: its gNB binds that address.
    n3: Ipv4Addr,
    /// N6 TUN device; none: no user plane.
    n6: Option<String>,
    max_sessions: usize,
    mode: Mode,
}

impl Default for Params {
    fn default() -> Self {
        Params { n3: Ipv4Addr::new(127, 0, 0, 2), n6: None, max_sessions: 1024, mode: Mode::Event }
    }
}

/// The UPF's counters (`/metrics`: `librecn_eo_counter_total`), by index.
const COUNTERS: &[&str] = &[
    "uplink_packets",
    "uplink_bytes",
    "downlink_packets",
    "downlink_bytes",
    "dropped_packets",
    "extra_flow_packets",
    "rate_limited_packets",
];
const UPLINK_PACKETS: usize = 0;
const UPLINK_BYTES: usize = 1;
const DOWNLINK_PACKETS: usize = 2;
const DOWNLINK_BYTES: usize = 3;
const DROPPED: usize = 4;
/// Downlink packets sent on one of the PCF's extra QoS flows.
const EXTRA_FLOW: usize = 5;
/// Packets dropped beyond the session AMBR or their flow's MBR.
const RATE_LIMITED: usize = 6;

/// The UPF EO.
pub struct Upf {
    name: String,
    n3: Ipv4Addr,
    n6_name: Option<String>,
    sessions: Sessions,
    mode: Mode,
    /// Hybrid mode: polls in a row that found nothing.
    empty_polls: u32,
    /// GTP-U socket and N6 device, once open; their epoll tokens, unless polled.
    n3_socket: Option<(UdpSocket, Option<IoToken>)>,
    n6: Option<(File, Option<IoToken>)>,
    buffer: Box<[u8]>,
    parked: Parked,
    /// The SMF, from its N4 messages: where downlink data reports go.
    smf: Option<EoId>,
    /// Per gNB (its key): uplink bytes and packets, downlink bytes and packets.
    traffic: Vec<(u32, [u64; 4])>,
    log: Logger,
    alarms: Alarms,
}

impl Upf {
    /// From deployment `params`: `n3` (default 127.0.0.2), `n6` (a TUN
    /// device name, default none), `max_sessions` (1024) and `mode`
    /// (`event`, `poll` or `hybrid`; default `event`).
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
            mode: p.mode,
            empty_polls: 0,
            n3_socket: None,
            n6: None,
            buffer: vec![0; BUFFER].into_boxed_slice(),
            parked: Parked::default(),
            smf: None,
            traffic: Vec::with_capacity(MAX_GNBS),
            log: Logger::default(),
            alarms: Alarms::new(),
        })
    }

    /// For the control EO's factory.
    pub fn from_params(name: &str, params: &serde_json::Value) -> Result<(EoConfig, Self), String> {
        Ok((Self::eo_config(name), Self::new(name, params)?))
    }

    /// N4 messages are small, but N4Establish (the session's rule); web
    /// API requests 8 KiB; N3 and N6 are two I/O registrations; the packet
    /// counters.
    pub fn eo_config(name: &str) -> EoConfig {
        let buffers = [(256, 1024), (512, 1024), (8 * 1024, 2)];
        EoConfig::new(name).max_io(2).max_timers(0).buffers(&buffers).counters(COUNTERS)
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
        if self.mode == Mode::Poll {
            self.n3_socket = Some((socket, None));
            ctx.set_polling(true);
            return Ok(());
        }
        let token = ctx.register_io(&socket, Interest::READABLE).map_err(|e| format!("N3: {e}"))?;
        self.n3_socket = Some((socket, Some(token)));
        if let Some((device, token)) = &mut self.n6 {
            *token = Some(ctx.register_io(&*device, Interest::READABLE).map_err(|e| format!("N6: {e}"))?);
        }
        Ok(())
    }

    /// GTP-U datagrams from the gNBs; how many were read.
    fn on_n3(&mut self, ctx: &mut Context<'_>) -> usize {
        let (Some((socket, _)), Some((device, _))) = (&self.n3_socket, &mut self.n6) else { return 0 };
        let now = ctx.now();
        for read in 0..PACKETS_PER_TURN {
            let (n, from) = match socket.recv_from(&mut self.buffer) {
                Ok(received) => received,
                Err(e) if e.kind() == ErrorKind::WouldBlock => return read,
                Err(e) => {
                    log!(self.log, ctx, Error, "N3 receive: {e}");
                    return read;
                }
            };
            match librecn_gtpu::decode(&self.buffer[..n]) {
                Ok(Gtp::GPdu { teid, qfi, payload }) => {
                    // The inner source must be the session's UE (no spoofing).
                    let session = self.sessions.by_teid(teid);
                    let Some(s) = session.filter(|s| ipv4_src(payload) == Some(s.ue_ip) && s.qos.uplink_known(qfi))
                    else {
                        ctx.count(DROPPED, 1);
                        continue;
                    };
                    if !s.qos.allow(qfi.unwrap_or(DEFAULT_QFI), false, now, payload.len()) {
                        ctx.count(RATE_LIMITED, 1);
                    } else if device.write(payload).is_ok() {
                        ctx.count(UPLINK_PACKETS, 1);
                        ctx.count(UPLINK_BYTES, payload.len() as u64);
                        if let Some(t) = s.gnb.and_then(|i| self.traffic.get_mut(usize::from(i))) {
                            (t.1[0], t.1[1]) = (t.1[0] + payload.len() as u64, t.1[1] + 1);
                        }
                    } else {
                        ctx.count(DROPPED, 1);
                    }
                }
                Ok(Gtp::EchoRequest { seq }) => {
                    let mut response = [0u8; 14];
                    let len = librecn_gtpu::echo_response(seq, &mut response);
                    let _ = socket.send_to(&response[..len], from);
                }
                Ok(Gtp::Other { .. }) | Err(_) => ctx.count(DROPPED, 1),
            }
        }
        PACKETS_PER_TURN
    }

    /// IP packets from the data network, to the UEs; how many were read.
    fn on_n6(&mut self, ctx: &mut Context<'_>) -> usize {
        let (Some((socket, _)), Some((device, _))) = (&self.n3_socket, &mut self.n6) else { return 0 };
        let now = ctx.now();
        for read in 0..PACKETS_PER_TURN {
            // The packet lands after the room left for its GTP-U header.
            let n = match device.read(&mut self.buffer[G_PDU_HEADER..]) {
                Ok(0) => return read,
                Ok(n) => n,
                Err(e) if e.kind() == ErrorKind::WouldBlock => return read,
                Err(e) => {
                    log!(self.log, ctx, Error, "N6 read: {e}");
                    return read;
                }
            };
            let packet = &self.buffer[G_PDU_HEADER..G_PDU_HEADER + n];
            let Some((seid, session)) = ipv4_dst(packet).and_then(|dst| self.sessions.by_ue(dst)) else {
                ctx.count(DROPPED, 1);
                continue;
            };
            let Some((gnb, teid)) = session.dl else {
                // The UE is idle: the packet waits for its tunnel, the SMF is told.
                if self.parked.park(seid, packet) != Parking::Kept {
                    ctx.count(DROPPED, 1);
                }
                if session.reported.is_none_or(|at| now - at >= REPORT_PERIOD) {
                    session.reported = Some(now);
                    let sent = self
                        .smf
                        .ok_or(librecn_scheduler::Error::UnknownEo)
                        .and_then(|smf| ctx.send_value(smf, &N4Report { seid }));
                    match sent {
                        Ok(()) => log!(self.log, ctx, Info, "session {seid:#x}: downlink data, the UE is idle"),
                        Err(e) => log!(self.log, ctx, Error, "session {seid:#x}: downlink data not reported: {e}"),
                    }
                }
                continue;
            };
            let qfi = session.qos.downlink(packet);
            if !session.qos.allow(qfi, true, now, n) {
                ctx.count(RATE_LIMITED, 1);
                continue;
            }
            let header: &mut [u8; G_PDU_HEADER] = (&mut self.buffer[..G_PDU_HEADER]).try_into().expect("16 bytes");
            librecn_gtpu::write_g_pdu_header(header, teid, qfi, n);
            match socket.send_to(&self.buffer[..G_PDU_HEADER + n], SocketAddr::V4(gnb)) {
                Ok(_) => {
                    ctx.count(DOWNLINK_PACKETS, 1);
                    ctx.count(DOWNLINK_BYTES, n as u64);
                    ctx.count(EXTRA_FLOW, u64::from(qfi != DEFAULT_QFI));
                    if let Some(t) = session.gnb.and_then(|i| self.traffic.get_mut(usize::from(i))) {
                        (t.1[2], t.1[3]) = (t.1[2] + n as u64, t.1[3] + 1);
                    }
                }
                Err(_) => ctx.count(DROPPED, 1),
            }
        }
        PACKETS_PER_TURN
    }

    fn on_establish(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &N4Establish) {
        let result = self.sessions.establish(request.seid, Ipv4Addr::from(request.ue_ip), &request.rule);
        let result = result.map(|s| (s.ue_ip, s.ul_teid));
        match result {
            Ok((ue_ip, teid)) => {
                let flows = request.rule.rules().len();
                log!(
                    self.log,
                    ctx,
                    Info,
                    "session {:#x}: UE {ue_ip}, uplink TEID {teid:#x}, {flows} extra flow(s)",
                    request.seid
                )
            }
            Err(e) => log!(self.log, ctx, Error, "session {:#x} refused: {e:?}", request.seid),
        }
        let (n3, seid) = (self.n3, request.seid);
        let sent = ctx.alloc::<N4EstablishResult>(msg.from()).and_then(|r| {
            let mut r = r.with_corr_id(msg.corr_id()).with_flags(flags::RESPONSE);
            r.seid = seid;
            match result {
                Ok((_, teid)) => (r.n3_addr, r.ul_teid) = (n3.octets(), teid),
                Err(Refusal::Full | Refusal::AddressInUse) => r.set_outcome(Outcome::Unavailable),
            }
            ctx.send(r)
        });
        if let Err(e) = sent {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", msg.from());
        }
    }

    fn on_modify(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &N4Modify) {
        self.smf = Some(msg.from());
        let gnb = SocketAddrV4::new(Ipv4Addr::from(request.gnb_addr), librecn_gtpu::PORT);
        let dl = (request.dl_teid != 0).then_some((gnb, request.dl_teid));
        let slot = self.gnb_slot(request.gnb);
        if let Some(s) = self.sessions.get_mut(request.seid).filter(|_| slot.is_some()) {
            s.gnb = slot;
        }
        let outcome = if self.sessions.modify(request.seid, dl) {
            let (seid, teid) = (request.seid, request.dl_teid);
            match dl {
                Some(_) => {
                    log!(self.log, ctx, Info, "session {seid:#x}: downlink to {} TEID {teid:#x}", gnb.ip());
                    self.send_parked(ctx, seid, gnb, teid);
                }
                None => log!(self.log, ctx, Info, "session {seid:#x}: downlink deactivated"),
            }
            Outcome::Ok
        } else {
            Outcome::ContextNotFound
        };
        // A notification (the UE went idle) wants no answer.
        if msg.header().flags & flags::REQUEST == 0 {
            return;
        }
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

    /// The slot of gNB `key` in the traffic table, a new one if there is
    /// room; none for key 0 (unknown).
    fn gnb_slot(&mut self, key: u32) -> Option<u16> {
        if key == 0 {
            return None;
        }
        let slot = match self.traffic.iter().position(|(k, _)| *k == key) {
            Some(slot) => slot,
            None if self.traffic.len() < MAX_GNBS => {
                self.traffic.push((key, [0; 4]));
                self.traffic.len() - 1
            }
            None => return None,
        };
        u16::try_from(slot).ok()
    }

    /// `GET gnbs`: the traffic per gNB.
    fn on_manage(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &Manage) {
        let (status, json) = match (request.method(), request.path()) {
            (Some(Method::Get), "gnbs") => {
                let gnbs: Vec<_> = self
                    .traffic
                    .iter()
                    .map(|(key, t)| {
                        serde_json::json!({ "gnb": key, "uplink_bytes": t[0], "uplink_packets": t[1],
                                            "downlink_bytes": t[2], "downlink_packets": t[3] })
                    })
                    .collect();
                (200, serde_json::json!(gnbs))
            }
            (_, path) => (404, serde_json::json!({ "error": format!("no resource {path:?}") })),
        };
        if let Err(e) = send_reply(ctx, msg, status, &json.to_string()) {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", msg.from());
        }
    }

    /// The packets parked for session `seid`, now that its tunnel is back
    /// (64 at most in all: outside the session's rates).
    fn send_parked(&mut self, ctx: &mut Context<'_>, seid: u64, gnb: SocketAddrV4, teid: u32) {
        let (buffer, socket) = (&mut self.buffer, &self.n3_socket);
        let Some((qos, slot)) = self.sessions.get(seid).map(|s| (&s.qos, s.gnb)) else { return };
        let mut traffic = slot.and_then(|i| self.traffic.get_mut(usize::from(i)));
        let mut sent = 0;
        self.parked.release(seid, |packet| {
            let n = packet.len();
            buffer[G_PDU_HEADER..G_PDU_HEADER + n].copy_from_slice(packet);
            let qfi = qos.downlink(packet);
            let header: &mut [u8; G_PDU_HEADER] = (&mut buffer[..G_PDU_HEADER]).try_into().expect("16 bytes");
            librecn_gtpu::write_g_pdu_header(header, teid, qfi, n);
            let delivered = socket.as_ref().is_some_and(|(s, _)| s.send_to(&buffer[..G_PDU_HEADER + n], gnb).is_ok());
            if delivered {
                ctx.count(DOWNLINK_PACKETS, 1);
                ctx.count(DOWNLINK_BYTES, n as u64);
                ctx.count(EXTRA_FLOW, u64::from(qfi != DEFAULT_QFI));
                if let Some(t) = traffic.as_mut() {
                    (t.1[2], t.1[3]) = (t.1[2] + n as u64, t.1[3] + 1);
                }
                sent += 1;
            } else {
                ctx.count(DROPPED, 1);
            }
        });
        if sent > 0 {
            log!(self.log, ctx, Info, "session {seid:#x}: {sent} parked packet(s) sent");
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
                let (name, n3, mode) = (&self.name, self.n3, self.mode);
                log!(self.log, ctx, Vip, "UPF {name} ready, N3 {n3}:{}, N6 {n6}, {mode:?} mode", librecn_gtpu::PORT);
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
            if self.n3_socket.as_ref().is_some_and(|(_, t)| *t == Some(ready.token)) {
                self.on_n3(ctx);
            } else {
                self.on_n6(ctx);
            }
            // Hybrid: traffic is here, poll until it stops.
            if self.mode == Mode::Hybrid {
                self.empty_polls = 0;
                ctx.set_polling(true);
            }
        } else if msg.id() == system::EO_POLL {
            let moved = self.on_n3(ctx) + self.on_n6(ctx);
            if self.mode == Mode::Hybrid {
                self.empty_polls = if moved == 0 { self.empty_polls + 1 } else { 0 };
                if self.empty_polls >= EMPTY_POLLS {
                    ctx.set_polling(false);
                }
            }
        } else if let Some(request) = msg.body::<N4Establish>() {
            self.smf = Some(msg.from());
            self.on_establish(ctx, msg, request);
        } else if let Some(request) = msg.body::<N4Modify>() {
            self.on_modify(ctx, msg, request);
        } else if let Some(request) = msg.body::<Manage>() {
            self.on_manage(ctx, msg, request);
        } else if let Some(request) = msg.body::<N4Delete>()
            && self.sessions.delete(request.seid).is_some()
        {
            self.parked.release(request.seid, |_| ctx.count(DROPPED, 1));
            log!(self.log, ctx, Info, "session {:#x} deleted, {} left", request.seid, self.sessions.len());
        }
    }

    fn on_stop(&mut self, ctx: &mut Context<'_>) {
        let counters = ctx.stats(ctx.self_id()).map(|s| s.counters).unwrap_or_default();
        let get = |i: usize| counters.get(i).map_or(0, |c| c.1);
        log!(
            self.log,
            ctx,
            Info,
            "UPF {} stopping, {} session(s), {} packet(s) up, {} down, {} dropped",
            self.name,
            self.sessions.len(),
            get(UPLINK_PACKETS),
            get(DOWNLINK_PACKETS),
            get(DROPPED)
        );
    }
}
