//! The UPF's forwarding on the real runtime: latency, throughput and the
//! worker's CPU when idle.
//!
//! - `cargo run --release -p librecn-upf --example forwarding [-- MODE...]`:
//!   per mode (`event`, `poll`, `hybrid`), one UE, 84-byte packets;
//! - `... --example forwarding -- ues [N...]`: in `event` mode, per number of
//!   UEs (default 1 2 10 100 1000) and packet size (84 and 1400 bytes),
//!   uplink and downlink.
//!
//! `scripts/upf-modes.sh` runs both; results in `nf/upf/README.md`.
//!
//! A test gNB (UDP, 127.0.0.81:2152) and a data network (one end of a
//! datagram socket pair standing for the TUN device) around a UPF in a pool
//! pinned to core 2, one PDU session per
//! UE; the round trips run on core 6, the bursts' three senders on cores 8
//! to 10.

use std::collections::HashMap;
use std::fs::File;
use std::net::{Ipv4Addr, UdpSocket};
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixDatagram;
use std::time::{Duration, Instant};

use librecn_common::log::{self, Level};
use librecn_sbi::pdu_session::{N4Establish, N4EstablishResult, N4Modify};
use librecn_scheduler::runtime::{PoolConfig, Runtime, RuntimeConfig};
use librecn_scheduler::testbed::Probe;
use librecn_scheduler::{EoConfig, MsgBody, WorkerStats};
use librecn_upf::Upf;

const N3: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 80);
const GNB: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 81);
/// The first UE's address; the next UEs count up from it.
const FIRST_UE: u32 = u32::from_be_bytes([10, 45, 0, 2]);
const SERVER: [u8; 4] = [10, 45, 0, 1];
/// The first UE's downlink TEID; the next UEs count up from it.
const DL_TEID: u32 = 7;
/// Round trips per latency run.
const ROUND_TRIPS: usize = 20_000;
/// Round trips with a gap between them (sparse traffic).
const SPARSE_TRIPS: usize = 2_000;
const GAP: Duration = Duration::from_millis(1);
/// Packets offered by a burst, from each of the sending threads.
const BURST: usize = 200_000;
const SENDERS: usize = 3;
/// A burst is over when nothing arrives for this long.
const QUIET: Duration = Duration::from_millis(200);
/// IPv4 packet size of the mode runs.
const SMALL: usize = 84;

/// An IPv4 packet of `size` bytes (header only checked by the UPF) from `src` to `dst`.
fn ipv4(src: [u8; 4], dst: [u8; 4], size: usize) -> Vec<u8> {
    let len = size as u16;
    let mut p = vec![0x45, 0, (len >> 8) as u8, len as u8, 0, 0, 0, 0, 64, 17, 0, 0];
    p.extend_from_slice(&src);
    p.extend_from_slice(&dst);
    p.resize(size, 0);
    p
}

/// An uplink G-PDU with the PDU Session Container, as UERANSIM sends it.
fn g_pdu(teid: u32, payload: &[u8]) -> Vec<u8> {
    let len = (8 + payload.len()) as u16;
    let mut d = vec![0x34, 0xff, (len >> 8) as u8, len as u8];
    d.extend_from_slice(&teid.to_be_bytes());
    d.extend_from_slice(&[0, 0, 0, 0x85, 1, 0x10, 1, 0]);
    d.extend_from_slice(payload);
    d
}

fn pin(core: usize) {
    // SAFETY: an all-zero cpu_set_t is a valid empty set.
    let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    // SAFETY: core < CPU_SETSIZE; the set outlives the call.
    unsafe {
        libc::CPU_SET(core, &mut set);
        libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &set);
    }
}

fn percentile(sorted: &[Duration], p: f64) -> f64 {
    sorted[((sorted.len() - 1) as f64 * p) as usize].as_secs_f64() * 1e6
}

fn busy(stats: &[WorkerStats]) -> (Duration, Duration) {
    stats.iter().filter(|w| w.pool == "up").fold((Duration::ZERO, Duration::ZERO), |(b, i), w| (b + w.busy, i + w.idle))
}

/// Retried while the UPF's mailbox is full.
fn inject<T: MsgBody>(runtime: &Runtime, from: librecn_scheduler::EoId, to: librecn_scheduler::EoId, value: &T) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while runtime.inject_from(from, to, 1, value).is_err() {
        assert!(Instant::now() < deadline, "the UPF takes no N4 message");
        std::thread::sleep(Duration::from_micros(100));
    }
}

/// A UPF with one session per UE, its gNB and its data network, and each
/// UE's uplink G-PDU and downlink packet.
struct Bench {
    runtime: Runtime,
    dn: UnixDatagram,
    gnb: UdpSocket,
    uplinks: Vec<Vec<u8>>,
    downlinks: Vec<Vec<u8>>,
}

impl Bench {
    fn new(mode: &str, ues: usize, size: usize) -> Bench {
        let mut config = RuntimeConfig::new(vec![PoolConfig::pinned("up", &[2]), PoolConfig::new("cp", 1)]);
        config.max_eos = 8;
        let runtime = Runtime::new(config).expect("runtime");
        let (dn, upf_end) = UnixDatagram::pair().unwrap();
        upf_end.set_nonblocking(true).unwrap();
        let params = serde_json::json!({ "n3": N3.to_string(), "mode": mode, "max_sessions": ues });
        let upf = Upf::new("upf", &params).unwrap().with_n6(File::from(OwnedFd::from(upf_end)));
        let upf = runtime.add(Upf::eo_config("upf").pool("up"), upf).unwrap();
        let (probe, seen) = Probe::new();
        let smf = runtime.add(EoConfig::new("smf").pool("cp").buffers(&[(256, 1024)]), probe).unwrap();
        let gnb = UdpSocket::bind((GNB, librecn_gtpu::PORT)).expect("127.0.0.81:2152 free");

        let ue = |i: usize| (FIRST_UE + i as u32).to_be_bytes();
        let rule = <librecn_sbi::policy::SessionRule as zerocopy::FromZeros>::new_zeroed();
        for i in 0..ues {
            inject(&runtime, smf, upf, &N4Establish::new(i as u64 + 1, ue(i), rule));
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        let ul_teids: HashMap<u64, u32> = loop {
            let answers: HashMap<u64, u32> = seen
                .messages()
                .iter()
                .filter_map(|m| m.body::<N4EstablishResult>().map(|r| (r.seid, r.ul_teid)))
                .collect();
            if answers.len() == ues {
                break answers;
            }
            assert!(Instant::now() < deadline, "{} N4 answers out of {ues}", answers.len());
            std::thread::sleep(Duration::from_millis(1));
        };
        for i in 0..ues {
            let modify = N4Modify::new(i as u64 + 1, GNB.octets(), DL_TEID + i as u32, 0);
            inject(&runtime, smf, upf, &modify);
        }
        std::thread::sleep(Duration::from_millis(100));
        let uplinks = (0..ues).map(|i| g_pdu(ul_teids[&(i as u64 + 1)], &ipv4(ue(i), SERVER, size))).collect();
        let downlinks = (0..ues).map(|i| ipv4(SERVER, ue(i), size)).collect();
        pin(6);
        Bench { runtime, dn, gnb, uplinks, downlinks }
    }

    /// Uplink to N6 then downlink back to the gNB, `count` times, each time
    /// for the next UE, after `gap`; sorted.
    fn round_trips(&self, count: usize, gap: Duration) -> Vec<Duration> {
        self.dn.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        self.gnb.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
        let mut buf = [0u8; 2048];
        let mut trips: Vec<Duration> = (0..count)
            .map(|k| {
                if !gap.is_zero() {
                    std::thread::sleep(gap);
                }
                let ue = k % self.uplinks.len();
                let start = Instant::now();
                self.gnb.send_to(&self.uplinks[ue], (N3, librecn_gtpu::PORT)).unwrap();
                self.dn.recv(&mut buf).expect("uplink lost");
                self.dn.send(&self.downlinks[ue]).unwrap();
                self.gnb.recv(&mut buf).expect("downlink lost");
                start.elapsed()
            })
            .collect();
        trips.sort();
        trips
    }

    /// Uplink G-PDUs from several gNB threads as fast as they send, the UEs
    /// in turn: (packets per second reaching N6, share delivered).
    fn uplink_burst(&self) -> (f64, f64) {
        self.dn.set_read_timeout(Some(QUIET)).unwrap();
        let start = Instant::now();
        let senders: Vec<_> = (0..SENDERS)
            .map(|s| {
                let sender = UdpSocket::bind((GNB, 0)).unwrap();
                let packets = self.uplinks.clone();
                std::thread::spawn(move || {
                    pin(8 + s);
                    for k in 0..BURST {
                        let _ = sender.send_to(&packets[(k * SENDERS + s) % packets.len()], (N3, librecn_gtpu::PORT));
                    }
                })
            })
            .collect();
        let rate = count_until_quiet(start, |buf| self.dn.recv(buf).is_ok());
        senders.into_iter().for_each(|s| s.join().unwrap());
        rate
    }

    /// Downlink packets from several data-network threads, the UEs in turn:
    /// (packets per second reaching the gNB, share delivered).
    fn downlink_burst(&self) -> (f64, f64) {
        self.gnb.set_read_timeout(Some(QUIET)).unwrap();
        let start = Instant::now();
        let senders: Vec<_> = (0..SENDERS)
            .map(|s| {
                let sender = self.dn.try_clone().unwrap();
                let packets = self.downlinks.clone();
                std::thread::spawn(move || {
                    pin(8 + s);
                    for k in 0..BURST {
                        let _ = sender.send(&packets[(k * SENDERS + s) % packets.len()]);
                    }
                })
            })
            .collect();
        let rate = count_until_quiet(start, |buf| self.gnb.recv(buf).is_ok());
        senders.into_iter().for_each(|s| s.join().unwrap());
        rate
    }

    /// The UPF's worker CPU over an idle second, in percent.
    fn idle_cpu(&self) -> f64 {
        let before = busy(&self.runtime.worker_stats());
        std::thread::sleep(Duration::from_secs(1));
        let after = busy(&self.runtime.worker_stats());
        let (busy_time, idle_time) = (after.0 - before.0, after.1 - before.1);
        busy_time.as_secs_f64() / (busy_time + idle_time).as_secs_f64().max(1e-9) * 100.0
    }

    fn end(mut self) {
        self.runtime.shutdown(Duration::from_secs(2));
    }
}

/// Packets received until none for [`QUIET`]: (thousands per second, share of
/// the burst).
fn count_until_quiet(start: Instant, mut recv: impl FnMut(&mut [u8]) -> bool) -> (f64, f64) {
    let mut buf = [0u8; 2048];
    let (mut received, mut last) = (0usize, start);
    while recv(&mut buf) {
        received += 1;
        last = Instant::now();
    }
    (received as f64 / (last - start).as_secs_f64() / 1e3, received as f64 / (BURST * SENDERS) as f64 * 100.0)
}

fn modes(modes: &[String]) {
    println!(
        "| mode | round trip p50 / p99 (µs) | sparse round trip p50 / p99 (µs) | burst (kpps, delivered) | idle worker CPU |"
    );
    println!("|---|---|---|---|---|");
    for mode in modes {
        let bench = Bench::new(mode, 1, SMALL);
        bench.round_trips(1000, Duration::ZERO); // warm up
        let trips = bench.round_trips(ROUND_TRIPS, Duration::ZERO);
        let sparse = bench.round_trips(SPARSE_TRIPS, GAP);
        let (kpps, delivered) = bench.uplink_burst();
        let idle = bench.idle_cpu();
        bench.end();
        println!(
            "| {mode} | {:.1} / {:.1} | {:.1} / {:.1} | {kpps:.0}, {delivered:.0} % | {idle:.0} % |",
            percentile(&trips, 0.5),
            percentile(&trips, 0.99),
            percentile(&sparse, 0.5),
            percentile(&sparse, 0.99),
        );
    }
}

fn ues(counts: &[usize]) {
    println!("| UEs | packet | round trip p50 / p99 (µs) | uplink kpps, Mbit/s | downlink kpps, Mbit/s |");
    println!("|---|---|---|---|---|");
    for &ues in counts {
        for size in [SMALL, 1400] {
            let bench = Bench::new("event", ues, size);
            bench.round_trips(1000, Duration::ZERO); // warm up
            let trips = bench.round_trips(ROUND_TRIPS, Duration::ZERO);
            let (up, _) = bench.uplink_burst();
            let (down, _) = bench.downlink_burst();
            bench.end();
            let mbps = |kpps: f64| kpps * size as f64 * 8.0 / 1e3;
            println!(
                "| {ues} | {size} B | {:.1} / {:.1} | {up:.0}, {:.0} | {down:.0}, {:.0} |",
                percentile(&trips, 0.5),
                percentile(&trips, 0.99),
                mbps(up),
                mbps(down),
            );
        }
    }
}

fn main() {
    log::set_default_level(Level::Error);
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("ues") => {
            let counts: Vec<usize> = args[1..].iter().map(|n| n.parse().expect("a number of UEs")).collect();
            ues(if counts.is_empty() { &[1, 2, 10, 100, 1000] } else { &counts });
        }
        None => modes(&["event".into(), "poll".into(), "hybrid".into()]),
        Some(_) => modes(&args),
    }
}
