//! The UPF's uplink with and without its fast path, between two machines
//! (two containers: `scripts/upf-fast.sh` runs it, results in
//! `nf/upf/README.md`).
//!
//! - `fast_path upf N3 UES MODE SECONDS`: a UPF on N3 address `N3`, its
//!   N6 the TUN device `upf0`, its fast path on, with `UES` sessions
//!   (uplink TEIDs 1 to `UES`, UEs from 10.45.0.2): the kernel carries them
//!   (`MODE` `fast`), or they have a PCC rule, which the fast path leaves to
//!   the UPF (`fallback`: as any session it cannot carry). In a pool
//!   pinned to core 2. A second after packets come, it prints what it carried for
//!   `SECONDS`: the packets out of the device `n6` (where the UEs' packets,
//!   to 10.99.0.2, are routed), and the busy time of cores 2 (the UPF) and
//!   3 (where the script puts the receive path of the N3 interface, the
//!   fast path's).
//! - `fast_path gnb N3 UES SIZE THREADS`: G-PDUs of IPv4 packets of
//!   `SIZE` bytes, the UEs in turn, as fast as `THREADS` threads send them
//!   (`sendmmsg`), until killed.

use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::os::fd::AsRawFd;
use std::time::{Duration, Instant};

use librecn_common::log::{self, Level};
use librecn_sbi::pdu_session::{N4Establish, N4EstablishResult, N4Modify};
use librecn_scheduler::runtime::{PoolConfig, Runtime, RuntimeConfig};
use librecn_scheduler::testbed::Probe;
use librecn_scheduler::{EoConfig, MsgBody};
use librecn_upf::Upf;

/// The first UE's address; the next UEs count up from it.
const FIRST_UE: u32 = u32::from_be_bytes([10, 45, 0, 2]);
/// Where the UEs' packets go: routed out of the device `n6`.
const SERVER: [u8; 4] = [10, 99, 0, 2];
const BATCH: usize = 32;

/// An IPv4 and UDP packet of `size` bytes from `src` to `dst`, its header
/// checksum right (the kernel checks it when it forwards).
fn ipv4(src: [u8; 4], dst: [u8; 4], size: usize) -> Vec<u8> {
    let len = size as u16;
    let mut p = vec![0x45, 0, (len >> 8) as u8, len as u8, 0, 0, 0, 0, 64, 17, 0, 0];
    p.extend_from_slice(&src);
    p.extend_from_slice(&dst);
    let mut sum: u32 = p.chunks(2).map(|w| u32::from(u16::from_be_bytes([w[0], w[1]]))).sum();
    while sum > 0xffff {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    p[10..12].copy_from_slice(&(!(sum as u16)).to_be_bytes());
    p.extend_from_slice(&[0x13, 0x89, 0x13, 0x89]);
    p.extend_from_slice(&(len - 20).to_be_bytes());
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

fn ue(i: usize) -> [u8; 4] {
    (FIRST_UE + i as u32).to_be_bytes()
}

/// Retried while the UPF's mailbox is full.
fn inject<T: MsgBody>(runtime: &Runtime, from: librecn_scheduler::EoId, to: librecn_scheduler::EoId, value: &T) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while runtime.inject_from(from, to, 1, value).is_err() {
        assert!(Instant::now() < deadline, "the UPF takes no N4 message");
        std::thread::sleep(Duration::from_micros(100));
    }
}

/// Busy and total time of `core` so far (`/proc/stat`, in ticks).
fn cpu(core: usize) -> (u64, u64) {
    let stat = std::fs::read_to_string("/proc/stat").unwrap();
    let line = stat.lines().find(|l| l.starts_with(&format!("cpu{core} "))).expect("core in /proc/stat");
    let t: Vec<u64> = line.split_whitespace().skip(1).map(|v| v.parse().unwrap()).collect();
    let idle = t[3] + t[4];
    (t.iter().sum::<u64>() - idle, t.iter().sum())
}

fn tx_packets() -> u64 {
    std::fs::read_to_string("/sys/class/net/n6/statistics/tx_packets").unwrap().trim().parse().unwrap()
}

fn upf(n3: Ipv4Addr, ues: usize, fast: bool, seconds: u64) {
    use zerocopy::FromZeros;
    let mut config = RuntimeConfig::new(vec![PoolConfig::pinned("up", &[2]), PoolConfig::new("cp", 1)]);
    config.max_eos = 8;
    let mut runtime = Runtime::new(config).expect("runtime");
    let params = serde_json::json!({ "n3": n3.to_string(), "n6": "upf0", "max_sessions": ues });
    let upf = runtime.add(Upf::eo_config("upf").pool("up"), Upf::new("upf", &params).unwrap()).unwrap();
    let (probe, seen) = Probe::new();
    let smf = runtime.add(EoConfig::new("smf").pool("cp").buffers(&[(256, 1024)]), probe).unwrap();
    let mut rule = librecn_sbi::policy::SessionRule::new_zeroed();
    if !fast {
        rule.set_rules(&[librecn_sbi::policy::PccRule::new_zeroed()]);
    }
    for i in 0..ues {
        inject(&runtime, smf, upf, &N4Establish::new(i as u64 + 1, ue(i), rule));
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    while seen.messages().iter().filter(|m| m.body::<N4EstablishResult>().is_some()).count() < ues {
        assert!(Instant::now() < deadline, "the UPF answers no N4Establish");
        std::thread::sleep(Duration::from_millis(1));
    }
    for m in seen.messages() {
        let r = m.body::<N4EstablishResult>().unwrap();
        assert_eq!(u64::from(r.ul_teid), r.seid, "TEIDs 1 to UES, as the gNB side assumes");
    }
    for i in 0..ues {
        inject(&runtime, smf, upf, &N4Modify::new(i as u64 + 1, [10, 211, 0, 20], 7 + i as u32, 0));
    }
    println!("ready");
    // Once the gNB sends, a second to settle.
    while tx_packets() == 0 {
        std::thread::sleep(Duration::from_millis(10));
    }
    std::thread::sleep(Duration::from_secs(1));
    let (packets, upf_cpu, rx_cpu, start) = (tx_packets(), cpu(2), cpu(3), Instant::now());
    std::thread::sleep(Duration::from_secs(seconds));
    let (packets, elapsed) = (tx_packets() - packets, start.elapsed().as_secs_f64());
    let busy = |(b0, t0): (u64, u64), (b1, t1): (u64, u64)| (b1 - b0) as f64 / (t1 - t0).max(1) as f64 * 100.0;
    println!(
        "carried {:.0} kpps, UPF core busy {:.0} %, receive core busy {:.0} %",
        packets as f64 / elapsed / 1e3,
        busy(upf_cpu, cpu(2)),
        busy(rx_cpu, cpu(3))
    );
    runtime.shutdown(Duration::from_secs(2));
}

fn gnb(n3: Ipv4Addr, ues: usize, size: usize, threads: usize) {
    let senders: Vec<_> = (0..threads).map(|t| std::thread::spawn(move || send(n3, ues, size, t))).collect();
    senders.into_iter().for_each(|s| s.join().unwrap());
}

fn send(n3: Ipv4Addr, ues: usize, size: usize, first: usize) -> ! {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let datagrams: Vec<Vec<u8>> = (0..ues).map(|i| g_pdu(i as u32 + 1, &ipv4(ue(i), SERVER, size))).collect();
    let to = libc::sockaddr_in {
        sin_family: libc::AF_INET as libc::sa_family_t,
        sin_port: librecn_gtpu::PORT.to_be(),
        sin_addr: libc::in_addr { s_addr: u32::from(*SocketAddrV4::new(n3, 0).ip()).to_be() },
        sin_zero: [0; 8],
    };
    let mut iovs = [libc::iovec { iov_base: std::ptr::null_mut(), iov_len: 0 }; BATCH];
    // SAFETY: all-zero is a valid mmsghdr.
    let mut msgs: [libc::mmsghdr; BATCH] = unsafe { std::mem::zeroed() };
    let mut next = first;
    loop {
        for (iov, msg) in iovs.iter_mut().zip(msgs.iter_mut()) {
            let d = &datagrams[next % ues];
            next += 1;
            *iov = libc::iovec { iov_base: d.as_ptr() as *mut _, iov_len: d.len() };
            msg.msg_hdr.msg_iov = iov;
            msg.msg_hdr.msg_iovlen = 1;
            msg.msg_hdr.msg_name = (&to as *const libc::sockaddr_in).cast_mut().cast();
            msg.msg_hdr.msg_namelen = size_of::<libc::sockaddr_in>() as _;
        }
        // SAFETY: the headers point to live datagrams and to `to`.
        unsafe { libc::sendmmsg(socket.as_raw_fd(), msgs.as_mut_ptr(), BATCH as _, 0) };
    }
}

fn main() {
    log::set_default_level(Level::Error);
    let args: Vec<String> = std::env::args().skip(1).collect();
    let n3: Ipv4Addr = args.get(1).and_then(|a| a.parse().ok()).expect("N3 address");
    let ues: usize = args.get(2).and_then(|a| a.parse().ok()).expect("UES");
    match args[0].as_str() {
        "upf" => upf(n3, ues, args[3] == "fast", args[4].parse().expect("SECONDS")),
        "gnb" => gnb(n3, ues, args[3].parse().expect("SIZE"), args[4].parse().expect("THREADS")),
        _ => panic!("usage: fast_path upf N3 UES fast|fallback SECONDS | gnb N3 UES SIZE THREADS"),
    }
}
