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
//! opened). Forwarding uses two preallocated batches of datagrams (see
//! [`batch`]): no allocation per packet, one system call on N3 for up to
//! 32 packets. A packet larger than a batch's slot (9216 bytes with its
//! GTP-U header) is dropped: N6's MTU must stay under 9200.
//!
//! `mode` (see `README.md` for the measurements):
//! - `event` (default): epoll wakes the UPF when N3 or N6 has packets;
//! - `poll`: no epoll; the UPF reads N3 and N6 at every turn of its worker,
//!   which never sleeps (a core for the user plane);
//! - `hybrid`: epoll wakes it, then it polls until [`EMPTY_POLLS`] polls in
//!   a row find nothing, and sleeps on epoll again.
//!
//! - [`sessions`], the session table;
//! - [`batch`], N3's datagrams in batches (`recvmmsg`, `sendmmsg`);
//! - [`fast`], the fast path: the kernel carries the uplink (XDP);
//!
//! Traffic is also counted per cell (`GET cells` on the web API: a gNB's
//! is the sum of its cells), by the UE's cell the SMF gives with each
//! downlink tunnel: a slot per cell in a preallocated table of
//! [`MAX_CELLS`].
//!
//! - [`qos`], the QoS flows and their rates;
//! - [`parked`], the downlink packets of idle UEs;
//! - [`tun`], the N6 device.

pub mod batch;
pub mod fast;
pub mod parked;
pub mod qos;
pub mod sessions;
pub mod tun;

use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::time::Duration;

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

use batch::Batch;
use parked::{Parked, Parking};
use sessions::{Refusal, Sessions};

/// Packets moved per readiness, so one busy side cannot hold the worker.
const PACKETS_PER_TURN: usize = 64;
/// Alarm raised while the user plane cannot run.
const ALARM_USER_PLANE: u32 = 0;
/// Alarm raised while the fast path cannot run (the user plane is slower).
const ALARM_FAST_PATH: u32 = 1;
/// How often the fast path's counts are folded into the UPF's.
const FOLD_PERIOD: Duration = Duration::from_secs(1);
/// Cells whose traffic is counted apart; the others' only in the totals.
pub const MAX_CELLS: usize = 1024;
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
    /// The kernel carries what it can (see [`fast`]).
    fast_path: bool,
}

impl Default for Params {
    fn default() -> Self {
        Params { n3: Ipv4Addr::new(127, 0, 0, 2), n6: None, max_sessions: 1024, mode: Mode::Event, fast_path: true }
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
    "fast_path_packets",
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
/// Uplink packets the kernel carried (the fast path), also in UPLINK_PACKETS.
const FAST_PATH: usize = 7;

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
    /// Whether the fast path is wanted; the fast path, once running.
    fast_path: bool,
    fast: Option<fast::FastPath>,
    max_sessions: usize,
    /// N3's datagrams in batches: received (uplink), to send (downlink).
    rx: Batch<()>,
    tx: Batch<Sent>,
    parked: Parked,
    /// The SMF, from its N4 messages: where downlink data reports go.
    smf: Option<EoId>,
    /// Per cell (its NR Cell Identity): uplink bytes and packets, downlink
    /// bytes and packets.
    traffic: Vec<(u64, [u64; 4])>,
    log: Logger,
    alarms: Alarms,
}

impl Upf {
    /// From deployment `params`: `n3` (default 127.0.0.2), `n6` (a TUN
    /// device name, default none), `max_sessions` (1024), `mode`
    /// (`event`, `poll` or `hybrid`; default `event`) and `fast_path`
    /// (default true).
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
            max_sessions: p.max_sessions,
            mode: p.mode,
            fast_path: p.fast_path,
            fast: None,
            empty_polls: 0,
            n3_socket: None,
            n6: None,
            rx: Batch::new(),
            tx: Batch::new(),
            parked: Parked::default(),
            smf: None,
            traffic: Vec::with_capacity(MAX_CELLS),
            log: Logger::default(),
            alarms: Alarms::new(),
        })
    }

    /// For the control EO's factory.
    pub fn from_params(name: &str, params: &serde_json::Value) -> Result<(EoConfig, Self), String> {
        Ok((Self::eo_config(name), Self::new(name, params)?))
    }

    /// N4 messages are small, but N4Establish (the session's rule); web
    /// API requests 8 KiB; N3 and N6 are two I/O registrations; one timer
    /// folds the fast path's counts; the packet counters.
    pub fn eo_config(name: &str) -> EoConfig {
        let buffers = [(256, 1024), (512, 1024), (8 * 1024, 2)];
        EoConfig::new(name).max_io(2).max_timers(1).buffers(&buffers).counters(COUNTERS)
    }

    /// N6 on an already open, non-blocking packet device instead of the
    /// `n6` TUN (tests: one end of a datagram socket pair), without the
    /// fast path (it would hand the uplink to the kernel, not the device).
    pub fn with_n6(mut self, device: File) -> Self {
        self.n6 = Some((device, None));
        self.fast_path = false;
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

    /// The fast path, if wanted: on, or the reason it is not (an alarm:
    /// the user plane runs, slower, in the UPF alone).
    fn start_fast_path(&mut self, ctx: &mut Context<'_>) {
        if !self.fast_path {
            return;
        }
        match fast::FastPath::start(self.n3, self.max_sessions) {
            Ok(f) => {
                log!(self.log, ctx, Info, "fast path on {}: the kernel carries the uplink", f.interface());
                self.fast = Some(f);
                if let Err(e) = ctx.arm_timer(FOLD_PERIOD, 0) {
                    log!(self.log, ctx, Error, "fast path counts not folded: {e}");
                }
            }
            Err(e) => {
                log!(self.log, ctx, Info, "fast path unavailable ({e}): the user plane runs in the UPF alone");
                let text = format!("fast path unavailable: {e}");
                self.alarms.raise(ctx, ALARM_FAST_PATH, Severity::Minor, &text);
            }
        }
    }

    /// Session `seid` given to the fast path, if it runs (again after a
    /// change).
    fn carry(&mut self, seid: u64) {
        if let (Some(f), Some(s)) = (&mut self.fast, self.sessions.get(seid)) {
            f.add(s);
        }
    }

    /// The fast path's counts since the last fold, added to the UPF's.
    fn fold(&mut self, ctx: &mut Context<'_>) {
        let (Some(f), traffic) = (&mut self.fast, &mut self.traffic) else { return };
        f.take(traffic.len(), |slot, c| {
            ctx.count(UPLINK_PACKETS, c.packets);
            ctx.count(FAST_PATH, c.packets);
            ctx.count(UPLINK_BYTES, c.bytes);
            ctx.count(RATE_LIMITED, c.rate_limited);
            if let Some(t) = slot.and_then(|i| traffic.get_mut(i)) {
                (t.1[0], t.1[1]) = (t.1[0] + c.bytes, t.1[1] + c.packets);
            }
        });
    }

    /// GTP-U datagrams from the gNBs, read in batches; how many were read.
    fn on_n3(&mut self, ctx: &mut Context<'_>) -> usize {
        let (Some((socket, _)), Some((device, _))) = (&self.n3_socket, &mut self.n6) else { return 0 };
        let now = ctx.now();
        let mut read = 0;
        while read < PACKETS_PER_TURN {
            let got = match self.rx.recv(socket) {
                Ok(0) => break,
                Ok(got) => got,
                Err(e) => {
                    log!(self.log, ctx, Error, "N3 receive: {e}");
                    break;
                }
            };
            read += got;
            for i in 0..got {
                let (datagram, from) = self.rx.datagram(i);
                match librecn_gtpu::decode(datagram) {
                    Ok(Gtp::GPdu { teid, qfi, payload }) => {
                        // The inner source must be the session's UE (no spoofing).
                        let session = self.sessions.by_teid(teid);
                        let valid =
                            |s: &&mut sessions::Session| ipv4_src(payload) == Some(s.ue_ip) && s.qos.uplink_known(qfi);
                        let Some(s) = session.filter(valid) else {
                            ctx.count(DROPPED, 1);
                            continue;
                        };
                        if !s.qos.allow(qfi.unwrap_or(DEFAULT_QFI), false, now, payload.len()) {
                            ctx.count(RATE_LIMITED, 1);
                        } else if device.write(payload).is_ok() {
                            ctx.count(UPLINK_PACKETS, 1);
                            ctx.count(UPLINK_BYTES, payload.len() as u64);
                            if let Some(t) = s.cell.and_then(|i| self.traffic.get_mut(usize::from(i))) {
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
        }
        read
    }

    /// IP packets from the data network, to the UEs: each read into the next
    /// slot of a batch, behind room for its GTP-U header, and sent with the
    /// others; how many were read.
    fn on_n6(&mut self, ctx: &mut Context<'_>) -> usize {
        let (Some((socket, _)), Some((device, _))) = (&self.n3_socket, &mut self.n6) else { return 0 };
        let now = ctx.now();
        let mut read = 0;
        while read < PACKETS_PER_TURN {
            let n = match device.read(&mut self.tx.slot()[G_PDU_HEADER..]) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) => {
                    log!(self.log, ctx, Error, "N6 read: {e}");
                    break;
                }
            };
            read += 1;
            if G_PDU_HEADER + n == batch::SLOT {
                // As large as the slot: maybe cut. The device's MTU is too large.
                ctx.count(DROPPED, 1);
                continue;
            }
            let packet = &self.tx.slot()[G_PDU_HEADER..G_PDU_HEADER + n];
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
            let cell = session.cell;
            let header: &mut [u8; G_PDU_HEADER] = (&mut self.tx.slot()[..G_PDU_HEADER]).try_into().expect("16 bytes");
            librecn_gtpu::write_g_pdu_header(header, teid, qfi, n);
            self.tx.push(G_PDU_HEADER + n, gnb, Sent { cell, extra: qfi != DEFAULT_QFI });
            if self.tx.is_full() {
                send_batch(&mut self.tx, socket, ctx, &mut self.traffic);
            }
        }
        send_batch(&mut self.tx, socket, ctx, &mut self.traffic);
        read
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
        if result.is_ok() {
            self.carry(request.seid);
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
        let slot = self.cell_slot(request.cell);
        if let Some(s) = self.sessions.get_mut(request.seid).filter(|_| slot.is_some()) {
            s.cell = slot;
        }
        let outcome = if self.sessions.modify(request.seid, dl) {
            let (seid, teid) = (request.seid, request.dl_teid);
            self.carry(seid);
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

    /// The slot of cell `cell` in the traffic table, a new one if there is
    /// room; none for 0 (unknown).
    fn cell_slot(&mut self, cell: u64) -> Option<u16> {
        if cell == 0 {
            return None;
        }
        let slot = match self.traffic.iter().position(|(c, _)| *c == cell) {
            Some(slot) => slot,
            None if self.traffic.len() < MAX_CELLS => {
                self.traffic.push((cell, [0; 4]));
                self.traffic.len() - 1
            }
            None => return None,
        };
        u16::try_from(slot).ok()
    }

    /// `GET cells`: the traffic per cell (its NR Cell Identity, 9 hex digits).
    fn on_manage(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &Manage) {
        let (status, json) = match (request.method(), request.path()) {
            (Some(Method::Get), "cells") => {
                let cells: Vec<_> = self
                    .traffic
                    .iter()
                    .map(|(cell, t)| {
                        serde_json::json!({ "cell": format!("{cell:09x}"), "uplink_bytes": t[0], "uplink_packets": t[1],
                                            "downlink_bytes": t[2], "downlink_packets": t[3] })
                    })
                    .collect();
                (200, serde_json::json!(cells))
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
        let (Some((socket, _)), Some(session)) = (&self.n3_socket, self.sessions.get(seid)) else { return };
        let (qos, cell, tx) = (&session.qos, session.cell, &mut self.tx);
        let mut sent = 0;
        self.parked.release(seid, |packet| {
            let n = packet.len();
            let slot = tx.slot();
            slot[G_PDU_HEADER..G_PDU_HEADER + n].copy_from_slice(packet);
            let qfi = qos.downlink(packet);
            let header: &mut [u8; G_PDU_HEADER] = (&mut slot[..G_PDU_HEADER]).try_into().expect("16 bytes");
            librecn_gtpu::write_g_pdu_header(header, teid, qfi, n);
            tx.push(G_PDU_HEADER + n, gnb, Sent { cell, extra: qfi != DEFAULT_QFI });
            sent += 1;
            if tx.is_full() {
                send_batch(tx, socket, ctx, &mut self.traffic);
            }
        });
        send_batch(tx, socket, ctx, &mut self.traffic);
        if sent > 0 {
            log!(self.log, ctx, Info, "session {seid:#x}: {sent} parked packet(s) sent");
        }
    }
}

/// What the UPF keeps with each downlink datagram of a batch, to count it
/// once sent.
#[derive(Clone, Copy, Default)]
struct Sent {
    cell: Option<u16>,
    extra: bool,
}

/// Sends a downlink batch, counting each datagram sent (its payload, behind
/// the GTP-U header) or dropped.
fn send_batch(tx: &mut Batch<Sent>, socket: &UdpSocket, ctx: &mut Context<'_>, traffic: &mut [(u64, [u64; 4])]) {
    tx.send(socket, |sent, len, ok| {
        if !ok {
            return ctx.count(DROPPED, 1);
        }
        let n = (len - G_PDU_HEADER) as u64;
        ctx.count(DOWNLINK_PACKETS, 1);
        ctx.count(DOWNLINK_BYTES, n);
        ctx.count(EXTRA_FLOW, u64::from(sent.extra));
        if let Some(t) = sent.cell.and_then(|i| traffic.get_mut(usize::from(i))) {
            (t.1[2], t.1[3]) = (t.1[2] + n, t.1[3] + 1);
        }
    });
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
                self.start_fast_path(ctx);
            }
            Ok(()) => log!(self.log, ctx, Vip, "UPF {} ready, N3 {}, no N6: sessions only", self.name, self.n3),
            Err(e) => {
                log!(self.log, ctx, Fatal, "UPF {}: no user plane: {e}", self.name);
                self.alarms.raise(ctx, ALARM_USER_PLANE, Severity::Critical, &format!("no user plane: {e}"));
            }
        }
    }

    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if msg.timer().is_some() {
            self.fold(ctx);
            let _ = ctx.arm_timer(FOLD_PERIOD, 0);
        } else if let Some(ready) = msg.io() {
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
            && let Some(session) = self.sessions.delete(request.seid)
        {
            if let Some(f) = &mut self.fast {
                f.remove(session.ul_teid);
            }
            self.parked.release(request.seid, |_| ctx.count(DROPPED, 1));
            log!(self.log, ctx, Info, "session {:#x} deleted, {} left", request.seid, self.sessions.len());
        }
    }

    fn on_stop(&mut self, ctx: &mut Context<'_>) {
        self.fold(ctx);
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
