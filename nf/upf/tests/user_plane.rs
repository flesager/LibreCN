//! The UPF forwarding: a test gNB on a real UDP socket (N3), a datagram
//! socket pair standing for the TUN device (N6).

use std::fs::File;
use std::net::{Ipv4Addr, UdpSocket};
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixDatagram;
use std::time::Duration;

use librecn_common::manage::{Manage, Method, Reply};
use librecn_oam::{CapturedLog, LogSink};
use librecn_sbi::pdu_session::{N4Establish, N4EstablishResult, N4Modify, N4Report, REPORT_PERIOD};
use librecn_sbi::policy::{PacketFilter, PccRule, Rates, SessionRule, direction};
use librecn_scheduler::testbed::{Probe, ProbeLog, TestBed};
use librecn_scheduler::{EoConfig, EoId};
use librecn_upf::Upf;
use zerocopy::FromZeros;

/// A minimal IPv4 header (20 bytes) from `src` to `dst`, then `data`.
fn ipv4(src: [u8; 4], dst: [u8; 4], data: &[u8]) -> Vec<u8> {
    let len = (20 + data.len()) as u16;
    let mut p = vec![0x45, 0, (len >> 8) as u8, len as u8, 0, 0, 0, 0, 64, 1, 0, 0];
    p.extend_from_slice(&src);
    p.extend_from_slice(&dst);
    p.extend_from_slice(data);
    p
}

const UE: [u8; 4] = [10, 45, 0, 2];
const SERVER: [u8; 4] = [10, 45, 0, 1];

struct Plane {
    tb: TestBed,
    log: CapturedLog,
    upf: EoId,
    smf: EoId,
    seen: ProbeLog,
    /// The data network's end of N6.
    dn: UnixDatagram,
    gnb: UdpSocket,
    n3: std::net::SocketAddr,
}

/// A UPF on N3 `n3` (a distinct loopback address per test) with one session
/// whose downlink goes to the test gNB.
fn plane(n3: Ipv4Addr) -> (Plane, u32) {
    plane_in(n3, "event")
}

fn plane_in(n3: Ipv4Addr, mode: &str) -> (Plane, u32) {
    plane_with(n3, mode, SessionRule::new_zeroed())
}

/// The same, the session with the PCF's `rule`.
fn plane_with(n3: Ipv4Addr, mode: &str, rule: SessionRule) -> (Plane, u32) {
    let mut tb = TestBed::new();
    let (sink, log) = LogSink::capture();
    tb.add(LogSink::config(), sink).unwrap();
    let (dn, upf_end) = UnixDatagram::pair().unwrap();
    upf_end.set_nonblocking(true).unwrap();
    dn.set_read_timeout(Some(Duration::from_millis(500))).unwrap();
    let upf = Upf::new("upf", &serde_json::json!({ "n3": n3.to_string(), "mode": mode })).unwrap();
    let upf = upf.with_n6(File::from(OwnedFd::from(upf_end)));
    let upf = tb.add(Upf::eo_config("upf"), upf).unwrap();
    let (probe, seen) = Probe::new();
    let smf = tb.add(EoConfig::new("smf").buffers(&[(256, 8)]), probe).unwrap();
    tb.run_until_idle();
    let gnb = UdpSocket::bind("127.0.0.1:0").unwrap();
    gnb.set_read_timeout(Some(Duration::from_millis(500))).unwrap();
    let mut p = Plane { tb, log, upf, smf, seen, dn, gnb, n3: (n3, librecn_gtpu::PORT).into() };

    p.tb.inject_from(p.smf, p.upf, 1, &N4Establish::new(1, UE, rule)).unwrap();
    p.tb.run_until_idle();
    let ul_teid = p.seen.take()[0].body::<N4EstablishResult>().unwrap().ul_teid;
    (p, ul_teid)
}

impl Plane {
    fn pump(&mut self) {
        for _ in 0..10 {
            self.tb.wait_io(Duration::from_millis(5));
            self.tb.run_until_idle();
        }
    }
}

/// An uplink G-PDU as UERANSIM sends it.
fn g_pdu(teid: u32, payload: &[u8]) -> Vec<u8> {
    g_pdu_on(teid, 1, payload)
}

/// The same, on QoS flow `qfi`.
fn g_pdu_on(teid: u32, qfi: u8, payload: &[u8]) -> Vec<u8> {
    let len = (8 + payload.len()) as u16;
    let mut d = vec![0x34, 0xff, (len >> 8) as u8, len as u8];
    d.extend_from_slice(&teid.to_be_bytes());
    d.extend_from_slice(&[0, 0, 0, 0x85, 1, 0x10, qfi, 0]);
    d.extend_from_slice(payload);
    d
}

#[test]
fn uplink_packets_reach_n6_only_from_their_ue() {
    let (mut p, teid) = plane(Ipv4Addr::new(127, 0, 0, 61));
    let packet = ipv4(UE, SERVER, b"ping");
    p.gnb.send_to(&g_pdu(teid, &packet), p.n3).unwrap();
    p.pump();
    let mut buf = [0u8; 128];
    let n = p.dn.recv(&mut buf).unwrap();
    assert_eq!(buf[..n], packet[..], "the inner packet, as sent");

    // Another source address, an unknown TEID: dropped.
    p.gnb.send_to(&g_pdu(teid, &ipv4([10, 45, 0, 9], SERVER, b"spoof")), p.n3).unwrap();
    p.gnb.send_to(&g_pdu(teid + 1, &packet), p.n3).unwrap();
    p.pump();
    p.dn.set_nonblocking(true).unwrap();
    assert!(p.dn.recv(&mut buf).is_err(), "nothing more on N6");

    // Echo is answered.
    p.gnb.send_to(&[0x32, 1, 0, 4, 0, 0, 0, 0, 0x12, 0x34, 0, 0], p.n3).unwrap();
    p.pump();
    let n = p.gnb.recv(&mut buf).unwrap();
    assert_eq!(buf[..n], [0x32, 2, 0, 6, 0, 0, 0, 0, 0x12, 0x34, 0, 0, 14, 0]);
    let counters = p.tb.stats(p.upf).unwrap().counters;
    let n = |name: &str| counters.iter().find(|c| c.0 == name).unwrap().1;
    assert_eq!((n("uplink_packets"), n("uplink_bytes"), n("dropped_packets")), (1, packet.len() as u64, 2));
    p.tb.stop(p.upf).unwrap();
    p.tb.run_until_idle();
    assert!(p.log.lines().iter().any(|l| l.msg.contains("1 packet(s) up, 0 down, 2 dropped")), "{:#?}", p.log.lines());
}

#[test]
fn downlink_packets_go_to_the_gnb_in_a_g_pdu() {
    let (mut p, _) = plane(Ipv4Addr::new(127, 0, 0, 62));
    // The gNB's GTP-U endpoint: port 2152 of an address of its own.
    let gnb = UdpSocket::bind("127.0.0.63:2152").unwrap();
    gnb.set_read_timeout(Some(Duration::from_millis(500))).unwrap();
    // The UE's cell (its NR Cell Identity, from the AMF through the SMF): its traffic counted apart.
    p.tb.inject_from(p.smf, p.upf, 2, &N4Modify::new(1, [127, 0, 0, 63], 0x77, 0x10)).unwrap();
    p.pump();

    let packet = ipv4(SERVER, UE, b"pong");
    p.dn.send(&packet).unwrap();
    p.dn.send(&ipv4(SERVER, [10, 45, 0, 99], b"nobody")).unwrap();
    p.pump();
    let mut buf = [0u8; 128];
    let n = gnb.recv(&mut buf).unwrap();
    let librecn_gtpu::Message::GPdu { teid, qfi, payload } = librecn_gtpu::decode(&buf[..n]).unwrap() else { panic!() };
    assert_eq!((teid, qfi, payload), (0x77, Some(1), &packet[..]));
    gnb.set_nonblocking(true).unwrap();
    assert!(gnb.recv(&mut buf).is_err(), "no session for 10.45.0.99: dropped");

    let (probe, replies) = Probe::new();
    let web = p.tb.add(EoConfig::new("web").buffers(&[(16 * 1024, 2)]), probe).unwrap();
    let mut request = Manage::new_zeroed();
    request.fill(Method::Get, "cells", "");
    p.tb.inject_from(web, p.upf, 9, &request).unwrap();
    p.tb.run_until_idle();
    let reply: Reply = replies.take()[0].body().unwrap();
    let expected = format!(
        r#"[{{"cell":"000000010","downlink_bytes":{},"downlink_packets":1,"uplink_bytes":0,"uplink_packets":0}}]"#,
        packet.len()
    );
    assert_eq!(reply.text(), expected);
}

#[test]
fn downlink_packets_go_on_their_qos_flow_and_uplink_ones_on_the_sessions() {
    // The PCF's rule: pings from the server on an extra flow (QFI 2).
    let mut rule = SessionRule::new_zeroed();
    let mut pings = PccRule::new_zeroed();
    pings.set_filters(&[PacketFilter::new(direction::DOWNLINK, 1, (SERVER, 32), [0, 0], [0, 0])]);
    rule.set_rules(&[pings]);
    let (mut p, ul_teid) = plane_with(Ipv4Addr::new(127, 0, 0, 68), "event", rule);
    let gnb = UdpSocket::bind("127.0.0.69:2152").unwrap();
    gnb.set_read_timeout(Some(Duration::from_millis(500))).unwrap();
    p.tb.inject_from(p.smf, p.upf, 2, &N4Modify::new(1, [127, 0, 0, 69], 0x79, 0)).unwrap();
    p.pump();
    assert!(p.log.lines().iter().any(|l| l.msg.contains("uplink TEID 0x1, 1 extra flow(s)")), "{:#?}", p.log.lines());

    p.dn.send(&ipv4(SERVER, UE, b"pong")).unwrap();
    p.dn.send(&ipv4([10, 45, 0, 7], UE, b"other")).unwrap();
    p.pump();
    let mut qfis = vec![];
    let mut buf = [0u8; 128];
    for _ in 0..2 {
        let n = gnb.recv(&mut buf).unwrap();
        let librecn_gtpu::Message::GPdu { qfi, .. } = librecn_gtpu::decode(&buf[..n]).unwrap() else { panic!() };
        qfis.push(qfi);
    }
    assert_eq!(qfis, [Some(2), Some(1)], "the rule's flow, then the default one");

    // Uplink: the session's flows only.
    let packet = ipv4(UE, SERVER, b"ping");
    for qfi in [2, 5] {
        p.gnb.send_to(&g_pdu_on(ul_teid, qfi, &packet), p.n3).unwrap();
    }
    p.pump();
    assert!(p.dn.recv(&mut buf).is_ok(), "QFI 2 forwarded");
    p.dn.set_nonblocking(true).unwrap();
    assert!(p.dn.recv(&mut buf).is_err(), "QFI 5 dropped");
    let counters = p.tb.stats(p.upf).unwrap().counters;
    let n = |name: &str| counters.iter().find(|c| c.0 == name).unwrap().1;
    assert_eq!((n("extra_flow_packets"), n("downlink_packets"), n("dropped_packets")), (1, 2, 1));
}

#[test]
fn packets_beyond_the_session_ambr_are_dropped() {
    // 8 kbit/s down: a 100 ms burst lets 5 packets of 24 bytes (24 ms each) through.
    let mut rule = SessionRule::new_zeroed();
    rule.ambr = Rates { uplink: 0, downlink: 8_000 };
    let (mut p, _) = plane_with(Ipv4Addr::new(127, 0, 0, 70), "event", rule);
    let gnb = UdpSocket::bind("127.0.0.71:2152").unwrap();
    gnb.set_read_timeout(Some(Duration::from_millis(200))).unwrap();
    p.tb.inject_from(p.smf, p.upf, 2, &N4Modify::new(1, [127, 0, 0, 71], 0x7a, 0)).unwrap();
    p.pump();
    let packet = ipv4(SERVER, UE, b"pong");
    let received = || {
        let mut buf = [0u8; 128];
        std::iter::from_fn(|| gnb.recv(&mut buf).ok()).count()
    };
    for _ in 0..8 {
        p.dn.send(&packet).unwrap();
    }
    p.pump();
    assert_eq!(received(), 5);
    // A second later the limit has room again.
    p.tb.advance(Duration::from_secs(1));
    p.dn.send(&packet).unwrap();
    p.pump();
    assert_eq!(received(), 1);
    let counters = p.tb.stats(p.upf).unwrap().counters;
    let n = |name: &str| counters.iter().find(|c| c.0 == name).unwrap().1;
    assert_eq!((n("downlink_packets"), n("rate_limited_packets"), n("dropped_packets")), (6, 3, 0));
}

#[test]
fn downlink_for_an_idle_ue_is_parked_reported_then_sent_with_its_tunnel() {
    let (mut p, _) = plane(Ipv4Addr::new(127, 0, 0, 66));
    // No downlink tunnel (the UE is idle): the packets wait, the SMF is told once.
    let (first, second) = (ipv4(SERVER, UE, b"one"), ipv4(SERVER, UE, b"two"));
    p.dn.send(&first).unwrap();
    p.dn.send(&second).unwrap();
    p.pump();
    let reports: Vec<_> = p.seen.take().iter().filter_map(|m| m.body::<N4Report>().map(|r| r.seid)).collect();
    assert_eq!(reports, [1], "one report for the two packets");
    // Still idle after the report period: reported again.
    p.tb.advance(REPORT_PERIOD);
    p.dn.send(&first).unwrap();
    p.pump();
    assert_eq!(p.seen.take().iter().filter(|m| m.body::<N4Report>().is_some()).count(), 1);

    // The tunnel is back: the parked packets go, in order.
    let gnb = UdpSocket::bind("127.0.0.67:2152").unwrap();
    gnb.set_read_timeout(Some(Duration::from_millis(500))).unwrap();
    p.tb.inject_from(p.smf, p.upf, 2, &N4Modify::new(1, [127, 0, 0, 67], 0x78, 0)).unwrap();
    p.pump();
    let mut buf = [0u8; 128];
    for expected in [&first, &second, &first] {
        let n = gnb.recv(&mut buf).unwrap();
        let librecn_gtpu::Message::GPdu { teid, payload, .. } = librecn_gtpu::decode(&buf[..n]).unwrap() else {
            panic!()
        };
        assert_eq!((teid, payload), (0x78, &expected[..]));
    }
    assert!(p.log.lines().iter().any(|l| l.msg.contains("3 parked packet(s) sent")), "{:#?}", p.log.lines());
}

#[test]
fn poll_mode_forwards_without_epoll() {
    let (mut p, teid) = plane_in(Ipv4Addr::new(127, 0, 0, 64), "poll");
    let packet = ipv4(UE, SERVER, b"poll");
    p.gnb.send_to(&g_pdu(teid, &packet), p.n3).unwrap();
    std::thread::sleep(Duration::from_millis(5));
    assert_eq!(p.tb.wait_io(Duration::ZERO), 0, "nothing registered with epoll");
    assert_eq!(p.tb.poll(), 1, "the UPF polls");
    p.tb.run_until_idle();
    let mut buf = [0u8; 128];
    let n = p.dn.recv(&mut buf).unwrap();
    assert_eq!(buf[..n], packet[..]);
}

#[test]
fn hybrid_mode_polls_while_traffic_flows_then_sleeps() {
    let (mut p, teid) = plane_in(Ipv4Addr::new(127, 0, 0, 65), "hybrid");
    assert_eq!(p.tb.poll(), 0, "asleep on epoll");
    p.gnb.send_to(&g_pdu(teid, &ipv4(UE, SERVER, b"wake")), p.n3).unwrap();
    p.pump();
    let mut buf = [0u8; 128];
    assert!(p.dn.recv(&mut buf).is_ok(), "forwarded on readiness");
    for _ in 0..librecn_upf::EMPTY_POLLS {
        assert_eq!(p.tb.poll(), 1, "polling after traffic");
        p.tb.run_until_idle();
    }
    assert_eq!(p.tb.poll(), 0, "back to epoll after empty polls");
}
