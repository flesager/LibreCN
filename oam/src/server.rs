//! The web server EO: HTTP/1.1 API and GUI.
//!
//! | Request | Answer |
//! |---|---|
//! | `GET /`, `/app.js`, `/style.css` | the GUI |
//! | `GET /api/eos` | EOs with their stats |
//! | `GET /api/workers` | workers |
//! | `GET /api/alarms` | active alarms (from the alarm EO) |
//! | `GET /api/kinds` | NF kinds (from control) |
//! | `GET /api/nfs` | the NFs started: name, kind, EO id, alive (control) |
//! | `POST /api/nfs` | start the NFs of a TOML or JSON deployment (control) |
//! | `POST /api/eos/{id}/stop` | stop an EO (control) |
//! | `GET`, `POST`, `DELETE /api/nfs/{name}/{path}` | the NF `name`'s own resources (`Manage`; the query string goes with the path) |
//! | `GET /metrics` | Prometheus text |
//!
//! One request per connection (`Connection: close`). The server reads what
//! any EO can read; actions go to the control EO, which decides, or to the
//! NF whose resource it is.

use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use librecn_common::log::Logger;
use librecn_common::manage::{MANAGE_TEXT, Manage, Method};
use librecn_common::{Service, log, names};
use librecn_scheduler::{Context, EoConfig, EoId, Error, ExecutionObject, Interest, IoToken, Message, Time};
use serde::Serialize;
use serde_json::json;

use crate::proto::{self, Deploy, Reply, Stop};

/// The GUI: a static page, its script and style, calling the API.
const GUI: &str = include_str!("gui/index.html");
const GUI_JS: &str = include_str!("gui/app.js");
const GUI_CSS: &str = include_str!("gui/style.css");

/// Connections served at once (one I/O registration each, plus the listener).
const MAX_CONNECTIONS: usize = 31;
/// Largest request accepted: a deployment plus headers.
const MAX_REQUEST: usize = proto::DEPLOY_TEXT + 8 * 1024;
/// A connection still open after this is closed (504 if it was waiting).
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);
/// How long a response may take to write.
const WRITE_TIMEOUT: Duration = Duration::from_millis(200);

struct Connection {
    stream: TcpStream,
    token: IoToken,
    request: Vec<u8>,
    /// `corr_id` of the request forwarded to another EO.
    waiting: Option<u32>,
    opened: Time,
}

struct Request {
    method: String,
    path: String,
    body: Vec<u8>,
}

enum Target {
    Control,
    Alarm,
    Nf(EoId),
}

/// Web server EO (name `server`).
pub struct Server {
    addr: SocketAddr,
    bound: Arc<Mutex<Option<SocketAddr>>>,
    listener: Option<(TcpListener, IoToken)>,
    connections: Vec<Option<Connection>>,
    next_corr: u32,
    control: Service,
    alarm: Service,
    log: Logger,
}

impl Server {
    /// Listens on `addr` once started (port 0: any free port, see [`bound_addr`](Self::bound_addr)).
    pub fn new(addr: SocketAddr) -> Self {
        Server {
            addr,
            bound: Arc::default(),
            listener: None,
            connections: (0..MAX_CONNECTIONS).map(|_| None).collect(),
            next_corr: 1,
            control: Service::new(names::CONTROL),
            alarm: Service::new(names::ALARM),
            log: Logger::default(),
        }
    }

    /// The address actually listened on, once the EO started.
    pub fn bound_addr(&self) -> Arc<Mutex<Option<SocketAddr>>> {
        self.bound.clone()
    }

    /// Name `server`, OAM pool, room for 16 replies of 16 KiB.
    pub fn config() -> EoConfig {
        EoConfig::new("server")
            .pool(crate::POOL)
            .max_io(MAX_CONNECTIONS as u16 + 1)
            .max_timers(2)
            .system_capacity(64)
            .buffers(&[(16 * 1024, 16)])
    }

    fn accept(&mut self, ctx: &mut Context<'_>) {
        let Some((listener, _)) = &self.listener else { return };
        loop {
            let stream = match listener.accept() {
                Ok((stream, _)) => stream,
                Err(e) if e.kind() == ErrorKind::WouldBlock => return,
                Err(e) => {
                    log!(self.log, ctx, Error, "accept: {e}");
                    return;
                }
            };
            let Some(slot) = self.connections.iter_mut().find(|c| c.is_none()) else {
                continue; // full: dropping the stream refuses the connection
            };
            if stream.set_nonblocking(true).is_err() {
                continue;
            }
            if let Ok(token) = ctx.register_io(&stream, Interest::READABLE) {
                *slot = Some(Connection { stream, token, request: Vec::new(), waiting: None, opened: ctx.now() });
            }
        }
    }

    fn read(&mut self, ctx: &mut Context<'_>, i: usize) {
        let Some(conn) = self.connections[i].as_mut() else { return };
        let mut chunk = [0u8; 4096];
        loop {
            match conn.stream.read(&mut chunk) {
                Ok(0) => return self.close(ctx, i),
                Ok(n) => conn.request.extend_from_slice(&chunk[..n]),
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(_) => return self.close(ctx, i),
            }
            if conn.request.len() > MAX_REQUEST {
                return self.respond_json(ctx, i, 413, &json!({ "error": "request too large" }));
            }
        }
        if conn.waiting.is_some() {
            return;
        }
        match parse(&conn.request) {
            Ok(Some(request)) => self.handle(ctx, i, request),
            Ok(None) => {}
            Err(()) => self.respond_json(ctx, i, 400, &json!({ "error": "malformed request" })),
        }
    }

    fn handle(&mut self, ctx: &mut Context<'_>, i: usize, request: Request) {
        let path = request.path.split('?').next().unwrap_or_default();
        match (request.method.as_str(), path) {
            ("GET", "/" | "/index.html") => self.respond(ctx, i, 200, "text/html; charset=utf-8", GUI.as_bytes()),
            ("GET", "/app.js") => self.respond(ctx, i, 200, "text/javascript; charset=utf-8", GUI_JS.as_bytes()),
            ("GET", "/style.css") => self.respond(ctx, i, 200, "text/css; charset=utf-8", GUI_CSS.as_bytes()),
            ("GET", "/api/nfs") => {
                self.forward(ctx, i, Target::Control, |ctx, to, corr| ctx.signal(to, proto::NFS, corr));
            }
            ("GET", "/api/eos") => {
                let eos = eo_views(ctx);
                self.respond_json(ctx, i, 200, &eos);
            }
            ("GET", "/api/workers") => {
                let workers: Vec<_> = ctx.worker_stats().iter().map(WorkerView::from).collect();
                self.respond_json(ctx, i, 200, &workers);
            }
            ("GET", "/metrics") => {
                let eos: Vec<_> = ctx.eos().into_iter().filter_map(|e| ctx.stats(e.id).map(|s| (e, s))).collect();
                let text = crate::metrics::render(&eos, &ctx.worker_stats());
                self.respond(ctx, i, 200, "text/plain; version=0.0.4", text.as_bytes());
            }
            ("GET", "/api/alarms") => {
                self.forward(ctx, i, Target::Alarm, |ctx, to, corr| ctx.signal(to, proto::ALARMS, corr))
            }
            ("GET", "/api/kinds") => {
                self.forward(ctx, i, Target::Control, |ctx, to, corr| ctx.signal(to, proto::KINDS, corr));
            }
            ("POST", "/api/nfs") => {
                let Ok(text) = String::from_utf8(request.body) else {
                    return self.respond_json(ctx, i, 400, &json!({ "error": "deployment is not UTF-8" }));
                };
                if text.len() > proto::DEPLOY_TEXT {
                    return self.respond_json(ctx, i, 413, &json!({ "error": "deployment too large" }));
                }
                self.forward(ctx, i, Target::Control, move |ctx, to, corr| {
                    let mut deploy = ctx.alloc::<Deploy>(to)?.with_corr_id(corr);
                    deploy.fill(&text);
                    ctx.send(deploy)
                });
            }
            ("POST", p) if p.starts_with("/api/eos/") && p.ends_with("/stop") => {
                let id = &p["/api/eos/".len()..p.len() - "/stop".len()];
                let Ok(eo) = id.parse::<u64>() else {
                    return self.respond_json(ctx, i, 400, &json!({ "error": "bad EO id" }));
                };
                self.forward(ctx, i, Target::Control, move |ctx, to, corr| {
                    let mut stop = ctx.alloc::<Stop>(to)?.with_corr_id(corr);
                    stop.eo = eo;
                    ctx.send(stop)
                });
            }
            (method, p) if p.starts_with("/api/nfs/") => {
                // The NF's path keeps the query string.
                let full = &request.path["/api/nfs/".len()..];
                let name = &p["/api/nfs/".len()..];
                let name = name.split_once('/').map_or(name, |(n, _)| n);
                let path = full.strip_prefix(name).and_then(|r| r.strip_prefix('/')).unwrap_or_default();
                let (Some(method), Some(nf)) = (Method::parse(method), ctx.find(name)) else {
                    return self.respond_json(ctx, i, 404, &json!({ "error": format!("no NF {name:?}") }));
                };
                let Ok(body) = String::from_utf8(request.body) else {
                    return self.respond_json(ctx, i, 400, &json!({ "error": "body is not UTF-8" }));
                };
                if path.len() + body.len() > MANAGE_TEXT {
                    return self.respond_json(ctx, i, 413, &json!({ "error": "request too large" }));
                }
                let path = path.to_string();
                self.forward(ctx, i, Target::Nf(nf), move |ctx, to, corr| {
                    let mut manage = ctx.alloc::<Manage>(to)?.with_corr_id(corr);
                    manage.fill(method, &path, &body);
                    ctx.send(manage)
                });
            }
            _ => self.respond_json(ctx, i, 404, &json!({ "error": "not found" })),
        }
    }

    /// Sends a request to the control or alarm EO, or to an NF; the response
    /// goes out when its [`Reply`] arrives.
    fn forward(
        &mut self,
        ctx: &mut Context<'_>,
        i: usize,
        target: Target,
        send: impl FnOnce(&mut Context<'_>, EoId, u32) -> librecn_scheduler::Result<()>,
    ) {
        let (to, service) = match target {
            Target::Nf(nf) => (nf, None),
            Target::Control | Target::Alarm => {
                let service = if matches!(target, Target::Control) { &mut self.control } else { &mut self.alarm };
                let Some(to) = service.get(ctx) else {
                    return self.respond_json(ctx, i, 503, &json!({ "error": "service not running" }));
                };
                (to, Some(service))
            }
        };
        let corr = self.next_corr;
        self.next_corr = self.next_corr.wrapping_add(1).max(1);
        match send(ctx, to, corr) {
            Ok(()) => {
                if let Some(conn) = self.connections[i].as_mut() {
                    conn.waiting = Some(corr);
                }
            }
            Err(e) => {
                if let Some(service) = service.filter(|_| matches!(e, Error::UnknownEo | Error::EoFaulted)) {
                    service.forget();
                }
                self.respond_json(ctx, i, 503, &json!({ "error": e.to_string() }));
            }
        }
    }

    fn on_reply(&mut self, ctx: &mut Context<'_>, corr: u32, reply: &Reply) {
        let found = self.connections.iter().position(|c| c.as_ref().is_some_and(|c| c.waiting == Some(corr)));
        if let Some(i) = found {
            self.respond(ctx, i, reply.status(), "application/json", reply.text().as_bytes());
        }
    }

    fn sweep(&mut self, ctx: &mut Context<'_>) {
        let now = ctx.now();
        for i in 0..self.connections.len() {
            let Some(conn) = &self.connections[i] else { continue };
            if now - conn.opened >= CONNECTION_TIMEOUT {
                if conn.waiting.is_some() {
                    self.respond_json(ctx, i, 504, &json!({ "error": "no answer" }));
                } else {
                    self.close(ctx, i);
                }
            }
        }
    }

    fn respond_json(&mut self, ctx: &mut Context<'_>, i: usize, status: u16, body: &impl Serialize) {
        let body = serde_json::to_vec(body).unwrap_or_default();
        self.respond(ctx, i, status, "application/json", &body);
    }

    /// Writes the response (blocking, bounded by [`WRITE_TIMEOUT`]) and closes.
    fn respond(&mut self, ctx: &mut Context<'_>, i: usize, status: u16, content_type: &str, body: &[u8]) {
        let Some(mut conn) = self.connections[i].take() else { return };
        ctx.deregister_io(conn.token);
        let head = format!(
            "HTTP/1.1 {status} {}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\
             Cache-Control: no-store\r\nConnection: close\r\n\r\n",
            reason(status),
            body.len()
        );
        let _ = conn.stream.set_nonblocking(false);
        let _ = conn.stream.set_write_timeout(Some(WRITE_TIMEOUT));
        let _ = conn.stream.write_all(head.as_bytes()).and_then(|()| conn.stream.write_all(body));
    }

    fn close(&mut self, ctx: &mut Context<'_>, i: usize) {
        if let Some(conn) = self.connections[i].take() {
            ctx.deregister_io(conn.token);
        }
    }
}

impl ExecutionObject for Server {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        let listener = match TcpListener::bind(self.addr).and_then(|l| l.set_nonblocking(true).map(|()| l)) {
            Ok(l) => l,
            Err(e) => {
                log!(self.log, ctx, Fatal, "cannot listen on {}: {e}", self.addr);
                return;
            }
        };
        let token = match ctx.register_io(&listener, Interest::READABLE) {
            Ok(token) => token,
            Err(e) => {
                log!(self.log, ctx, Fatal, "cannot watch {}: {e}", self.addr);
                return;
            }
        };
        let addr = listener.local_addr().unwrap_or(self.addr);
        *self.bound.lock().unwrap() = Some(addr);
        self.listener = Some((listener, token));
        log!(self.log, ctx, Info, "web server on http://{addr}");
        let _ = ctx.arm_timer(Duration::from_secs(1), 0);
    }

    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if let Some(ready) = msg.io() {
            if self.listener.as_ref().is_some_and(|(_, t)| *t == ready.token) {
                self.accept(ctx);
            } else if let Some(i) =
                self.connections.iter().position(|c| c.as_ref().is_some_and(|c| c.token == ready.token))
            {
                self.read(ctx, i);
            }
        } else if let Some(reply) = msg.body::<Reply>() {
            self.on_reply(ctx, msg.corr_id(), reply);
        } else if msg.timer().is_some() {
            self.sweep(ctx);
            let _ = ctx.arm_timer(Duration::from_secs(1), 0);
        }
    }
}

fn parse(buf: &[u8]) -> Result<Option<Request>, ()> {
    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut req = httparse::Request::new(&mut headers);
    let header_len = match req.parse(buf) {
        Ok(httparse::Status::Complete(n)) => n,
        Ok(httparse::Status::Partial) => return Ok(None),
        Err(_) => return Err(()),
    };
    let content_length = req
        .headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case("content-length"))
        .map(|h| std::str::from_utf8(h.value).ok().and_then(|v| v.trim().parse::<usize>().ok()).ok_or(()))
        .transpose()?
        .unwrap_or(0);
    if buf.len() < header_len + content_length {
        return Ok(None);
    }
    Ok(Some(Request {
        method: req.method.unwrap_or_default().to_string(),
        path: req.path.unwrap_or_default().to_string(),
        body: buf[header_len..header_len + content_length].to_vec(),
    }))
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        400 => "Bad Request",
        404 => "Not Found",
        409 => "Conflict",
        413 => "Payload Too Large",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "",
    }
}

#[derive(Serialize)]
struct EoView {
    /// `EoId::raw()` as a string: JavaScript numbers lose 64-bit integers.
    id: String,
    name: String,
    pool: String,
    alive: bool,
    stats: Option<StatsView>,
}

#[derive(Serialize)]
struct StatsView {
    messages: u64,
    busy_ms: f64,
    mean_us: f64,
    max_us: f64,
    data_depth: usize,
    data_high_water: usize,
    data_capacity: usize,
    data_rejected: u64,
    system_depth: usize,
    system_high_water: usize,
    timers_fired: u64,
    io_events: u64,
    buffers: Vec<BufferView>,
    /// The EO's own counters, by name.
    counters: std::collections::BTreeMap<&'static str, u64>,
}

#[derive(Serialize)]
struct BufferView {
    size: usize,
    count: u32,
    in_use: u32,
    high_water: u32,
    fallbacks: u64,
    failures: u64,
}

fn eo_views(ctx: &mut Context<'_>) -> Vec<EoView> {
    let eos = ctx.eos();
    eos.into_iter()
        .map(|e| {
            let stats = ctx.stats(e.id).map(|s| StatsView {
                messages: s.processing.count,
                busy_ms: s.processing.total.as_secs_f64() * 1e3,
                mean_us: s.processing.mean().unwrap_or_default().as_secs_f64() * 1e6,
                max_us: s.processing.max.as_secs_f64() * 1e6,
                data_depth: s.data_lane.depth,
                data_high_water: s.data_lane.high_water,
                data_capacity: s.data_lane.capacity,
                data_rejected: s.data_lane.rejected,
                system_depth: s.system_lane.depth,
                system_high_water: s.system_lane.high_water,
                timers_fired: s.timers_fired,
                io_events: s.io_events,
                buffers: s
                    .buffers
                    .iter()
                    .map(|b| BufferView {
                        size: b.size,
                        count: b.count,
                        in_use: b.in_use,
                        high_water: b.high_water,
                        fallbacks: b.fallbacks,
                        failures: b.alloc_failures,
                    })
                    .collect(),
                counters: s.counters.iter().copied().collect(),
            });
            EoView { id: e.id.raw().to_string(), name: e.name, pool: e.pool, alive: e.alive, stats }
        })
        .collect()
}

#[derive(Serialize)]
struct WorkerView {
    pool: String,
    index: usize,
    core: Option<usize>,
    turns: u64,
    messages: u64,
    busy_ms: f64,
    idle_ms: f64,
    wakeups: u64,
}

impl From<&librecn_scheduler::WorkerStats> for WorkerView {
    fn from(w: &librecn_scheduler::WorkerStats) -> Self {
        WorkerView {
            pool: w.pool.clone(),
            index: w.index,
            core: w.core,
            turns: w.turns,
            messages: w.messages,
            busy_ms: w.busy.as_secs_f64() * 1e3,
            idle_ms: w.idle.as_secs_f64() * 1e3,
            wakeups: w.wakeups,
        }
    }
}
