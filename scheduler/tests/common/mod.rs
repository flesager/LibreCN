//! Message bodies shared by the integration tests.
#![allow(dead_code)]

use librecn_scheduler::MsgBody;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

#[derive(Clone, Copy, Debug, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Ping {
    pub seq: u64,
}

impl MsgBody for Ping {
    const MSG_ID: u32 = 0x0001_0001;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Pong {
    pub seq: u64,
}

impl MsgBody for Pong {
    const MSG_ID: u32 = 0x0001_0002;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Big {
    pub data: [u8; 1024],
}

impl MsgBody for Big {
    const MSG_ID: u32 = 0x0001_0003;
}

/// Header-only message id.
pub const KICK: u32 = 0x0001_0010;

/// Busy-waits `d` (simulated processing).
pub fn spin(d: std::time::Duration) {
    let start = std::time::Instant::now();
    while start.elapsed() < d {
        std::hint::spin_loop();
    }
}

/// Polls `done` every millisecond until it holds or `timeout` passes.
pub fn wait_until(timeout: std::time::Duration, mut done: impl FnMut() -> bool) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    while std::time::Instant::now() < deadline {
        if done() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    done()
}

/// UDP echo EO: binds on start, publishes its address, echoes **one**
/// datagram per readiness (the platform reports the socket again while data
/// remains), deregisters on `KICK`.
pub struct UdpEcho {
    pub addr: std::sync::Arc<std::sync::Mutex<Option<std::net::SocketAddr>>>,
    pub sock: Option<std::net::UdpSocket>,
    pub token: Option<librecn_scheduler::IoToken>,
}

impl UdpEcho {
    pub fn new() -> (Self, std::sync::Arc<std::sync::Mutex<Option<std::net::SocketAddr>>>) {
        let addr = std::sync::Arc::default();
        (UdpEcho { addr: std::sync::Arc::clone(&addr), sock: None, token: None }, addr)
    }
}

impl librecn_scheduler::ExecutionObject for UdpEcho {
    fn on_start(&mut self, ctx: &mut librecn_scheduler::Context<'_>) {
        let sock = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        sock.set_nonblocking(true).unwrap();
        self.token = Some(ctx.register_io(&sock, librecn_scheduler::Interest::READABLE).unwrap());
        *self.addr.lock().unwrap() = Some(sock.local_addr().unwrap());
        self.sock = Some(sock);
    }

    fn on_message(&mut self, msg: &librecn_scheduler::Message<'_>, ctx: &mut librecn_scheduler::Context<'_>) {
        if let Some(ready) = msg.io() {
            assert_eq!(Some(ready.token), self.token);
            assert!(ready.readable());
            let sock = self.sock.as_ref().unwrap();
            let mut buf = [0u8; 1500];
            match sock.recv_from(&mut buf) {
                Ok((n, peer)) => {
                    sock.send_to(&buf[..n], peer).unwrap();
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => panic!("recv: {e}"),
            }
        } else if msg.id() == KICK {
            assert!(ctx.deregister_io(self.token.take().unwrap()));
        }
    }
}

// ---------------------------------------------------------- metric loop

use std::time::Duration;

use librecn_scheduler::{Context, EoId, EoStats, ExecutionObject, Message, Time};

/// Callback receiving each collection: platform time and one entry per live target.
pub type MetricSink = Box<dyn FnMut(Time, &[(EoId, EoStats)]) + Send>;

/// Periodically snapshots the metrics of a set of EOs and hands them to a sink.
///
/// Collection allocates (names, buffer class lists): it runs on the metric
/// path, never on the message path of the EOs it observes.
pub struct MetricCollector {
    period: Duration,
    targets: Vec<EoId>,
    reset_peaks: bool,
    sink: MetricSink,
}

impl MetricCollector {
    /// Collects every `period`; with `reset_peaks`, each snapshot covers the
    /// peaks since the previous one.
    pub fn new(period: Duration, targets: Vec<EoId>, reset_peaks: bool, sink: MetricSink) -> Self {
        MetricCollector { period, targets, reset_peaks, sink }
    }

    fn arm(&self, ctx: &mut Context<'_>) {
        // Out of timers means a misconfigured collector; nothing better to do than stop collecting.
        let _ = ctx.arm_timer(self.period, 0);
    }
}

impl ExecutionObject for MetricCollector {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        self.arm(ctx);
    }

    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if msg.timer().is_none() {
            return;
        }
        let snapshots: Vec<_> = self
            .targets
            .iter()
            .filter_map(|&id| {
                let stats = if self.reset_peaks { ctx.stats_and_reset_peaks(id) } else { ctx.stats(id) };
                stats.map(|s| (id, s))
            })
            .collect();
        (self.sink)(ctx.now(), &snapshots);
        self.arm(ctx);
    }
}

impl std::fmt::Debug for MetricCollector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricCollector")
            .field("period", &self.period)
            .field("targets", &self.targets)
            .finish_non_exhaustive()
    }
}
