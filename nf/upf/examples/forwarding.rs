//! The UPF's forwarding on the real runtime, per mode (`event`, `poll`,
//! `hybrid`): latency, throughput and the worker's CPU when idle.
//!
//! `cargo run --release -p librecn-upf --example forwarding [-- MODE...]`
//! (`scripts/upf-modes.sh` runs every mode; results in `nf/upf/README.md`).
//!
//! A test gNB (UDP, 127.0.0.81:2152) and a data network (one end of a
//! datagram socket pair standing for the TUN device) around a UPF in a
//! one-worker pool pinned to core 2; the round trips run on core 4, the
//! burst's three senders on cores 6 to 8.

use std::fs::File;
use std::net::{Ipv4Addr, UdpSocket};
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixDatagram;
use std::time::{Duration, Instant};

use librecn_sbi::pdu_session::{N4Establish, N4EstablishResult, N4Modify};
use librecn_scheduler::runtime::{PoolConfig, Runtime, RuntimeConfig};
use librecn_scheduler::testbed::Probe;
use librecn_scheduler::{EoConfig, WorkerStats};
use librecn_upf::Upf;

const N3: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 80);
const GNB: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 81);
const UE: [u8; 4] = [10, 45, 0, 2];
const SERVER: [u8; 4] = [10, 45, 0, 1];
const DL_TEID: u32 = 7;
/// Round trips per latency run.
const ROUND_TRIPS: usize = 20_000;
/// Round trips with a gap between them (sparse traffic).
const SPARSE_TRIPS: usize = 2_000;
const GAP: Duration = Duration::from_millis(1);
/// Packets offered by the burst, from each of the sending threads.
const BURST: usize = 200_000;
const SENDERS: usize = 3;
const PAYLOAD: usize = 64;

/// An IPv4 packet (header only checked by the UPF) from `src` to `dst`.
fn ipv4(src: [u8; 4], dst: [u8; 4]) -> Vec<u8> {
    let len = (20 + PAYLOAD) as u16;
    let mut p = vec![0x45, 0, (len >> 8) as u8, len as u8, 0, 0, 0, 0, 64, 17, 0, 0];
    p.extend_from_slice(&src);
    p.extend_from_slice(&dst);
    p.resize(usize::from(len), 0);
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

struct Results {
    rtt: (f64, f64),
    sparse: (f64, f64),
    kpps: f64,
    delivered: f64,
    idle_cpu: f64,
}

fn busy(stats: &[WorkerStats]) -> (Duration, Duration) {
    stats.iter().filter(|w| w.pool == "up").fold((Duration::ZERO, Duration::ZERO), |(b, i), w| (b + w.busy, i + w.idle))
}

fn run(mode: &str) -> Results {
    let mut config = RuntimeConfig::new(vec![PoolConfig::pinned("up", &[2]), PoolConfig::new("cp", 1)]);
    config.max_eos = 8;
    let mut runtime = Runtime::new(config).expect("runtime");
    let (dn, upf_end) = UnixDatagram::pair().unwrap();
    upf_end.set_nonblocking(true).unwrap();
    dn.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
    let upf = Upf::new("upf", &serde_json::json!({ "n3": N3.to_string(), "mode": mode })).unwrap();
    let upf = upf.with_n6(File::from(OwnedFd::from(upf_end)));
    let upf = runtime.add(Upf::eo_config("upf").pool("up"), upf).unwrap();
    let (probe, seen) = Probe::new();
    let smf = runtime.add(EoConfig::new("smf").pool("cp").buffers(&[(256, 8)]), probe).unwrap();
    let gnb = UdpSocket::bind((GNB, librecn_gtpu::PORT)).expect("127.0.0.81:2152 free");
    gnb.set_read_timeout(Some(Duration::from_secs(1))).unwrap();
    runtime.inject_from(smf, upf, 1, &N4Establish::new(1, UE)).unwrap();
    let deadline = Instant::now() + Duration::from_secs(2);
    let ul_teid = loop {
        if let Some(r) = seen.messages().iter().find_map(|m| m.body::<N4EstablishResult>()) {
            break r.ul_teid;
        }
        assert!(Instant::now() < deadline, "no N4 answer");
        std::thread::sleep(Duration::from_millis(1));
    };
    runtime.inject_from(smf, upf, 2, &N4Modify { seid: 1, gnb_addr: GNB.octets(), dl_teid: DL_TEID }).unwrap();
    std::thread::sleep(Duration::from_millis(50));
    pin(4);

    let n3 = (N3, librecn_gtpu::PORT);
    let uplink = g_pdu(ul_teid, &ipv4(UE, SERVER));
    let downlink = ipv4(SERVER, UE);
    let mut buf = [0u8; 2048];
    let mut round_trip = || {
        let start = Instant::now();
        gnb.send_to(&uplink, n3).unwrap();
        dn.recv(&mut buf).expect("uplink lost");
        dn.send(&downlink).unwrap();
        gnb.recv(&mut buf).expect("downlink lost");
        start.elapsed()
    };
    for _ in 0..1000 {
        round_trip(); // warm up
    }
    let mut trips: Vec<Duration> = (0..ROUND_TRIPS).map(|_| round_trip()).collect();
    trips.sort();
    let mut sparse: Vec<Duration> = (0..SPARSE_TRIPS)
        .map(|_| {
            std::thread::sleep(GAP);
            round_trip()
        })
        .collect();
    sparse.sort();

    // Burst: uplink from several threads as fast as they send; what reaches N6.
    let start = Instant::now();
    let blasters: Vec<_> = (0..SENDERS)
        .map(|i| {
            let sender = UdpSocket::bind((GNB, 0)).unwrap();
            let offered = uplink.clone();
            std::thread::spawn(move || {
                pin(6 + i);
                for _ in 0..BURST {
                    let _ = sender.send_to(&offered, n3);
                }
            })
        })
        .collect();
    dn.set_read_timeout(Some(Duration::from_millis(200))).unwrap();
    let mut received = 0usize;
    let mut last = start;
    while dn.recv(&mut buf).is_ok() {
        received += 1;
        last = Instant::now();
    }
    for b in blasters {
        b.join().unwrap();
    }
    let kpps = received as f64 / (last - start).as_secs_f64() / 1e3;

    // The worker's CPU over an idle second.
    let before = busy(&runtime.worker_stats());
    std::thread::sleep(Duration::from_secs(1));
    let after = busy(&runtime.worker_stats());
    let (busy_time, idle_time) = (after.0 - before.0, after.1 - before.1);
    let idle_cpu = busy_time.as_secs_f64() / (busy_time + idle_time).as_secs_f64().max(1e-9) * 100.0;
    runtime.shutdown(Duration::from_secs(2));
    Results {
        rtt: (percentile(&trips, 0.5), percentile(&trips, 0.99)),
        sparse: (percentile(&sparse, 0.5), percentile(&sparse, 0.99)),
        kpps,
        delivered: received as f64 / (BURST * SENDERS) as f64 * 100.0,
        idle_cpu,
    }
}

fn main() {
    let modes: Vec<String> = std::env::args().skip(1).collect();
    let modes = if modes.is_empty() { vec!["event".into(), "poll".into(), "hybrid".into()] } else { modes };
    println!(
        "| mode | round trip p50 / p99 (µs) | sparse round trip p50 / p99 (µs) | burst (kpps, delivered) | idle worker CPU |"
    );
    println!("|---|---|---|---|---|");
    for mode in modes {
        let r = run(&mode);
        println!(
            "| {mode} | {:.1} / {:.1} | {:.1} / {:.1} | {:.0}, {:.0} % | {:.0} % |",
            r.rtt.0, r.rtt.1, r.sparse.0, r.sparse.1, r.kpps, r.delivered, r.idle_cpu
        );
    }
}
