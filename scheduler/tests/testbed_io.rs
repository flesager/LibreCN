//! Socket readiness on the test bed.

mod common;

use std::net::UdpSocket;
use std::time::Duration;

use common::{KICK, UdpEcho};
use librecn_scheduler::EoConfig;
use librecn_scheduler::testbed::TestBed;

fn client() -> UdpSocket {
    let c = UdpSocket::bind("127.0.0.1:0").unwrap();
    c.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    c
}

#[test]
fn readiness_is_reported_again_until_drained() {
    let mut tb = TestBed::new();
    let (echo, addr) = UdpEcho::new();
    let id = tb.add(EoConfig::new("echo"), echo).unwrap();
    tb.run_until_idle();
    let server = addr.lock().unwrap().unwrap();

    let c = client();
    for i in 0..3u8 {
        c.send_to(&[i], server).unwrap();
    }
    // One datagram per readiness: three polls, three echoes, then silence.
    for round in 0..3 {
        assert_eq!(tb.wait_io(Duration::from_secs(1)), 1, "round {round}");
        tb.run_until_idle();
    }
    let mut buf = [0u8; 8];
    let echoed: Vec<u8> = (0..3).map(|_| (c.recv(&mut buf).unwrap(), buf[0]).1).collect();
    assert_eq!(echoed, [0, 1, 2]);
    assert_eq!(tb.poll_io(), 0);
    assert_eq!(tb.stats(id).unwrap().io_events, 3);
}

#[test]
fn deregistered_and_stopped_eos_get_no_readiness() {
    let mut tb = TestBed::new();
    let (echo, addr) = UdpEcho::new();
    let id = tb.add(EoConfig::new("echo"), echo).unwrap();
    tb.run_until_idle();
    let server = addr.lock().unwrap().unwrap();
    let c = client();

    // Readiness queued, then the registration goes away before delivery.
    c.send_to(b"a", server).unwrap();
    assert_eq!(tb.wait_io(Duration::from_secs(1)), 1);
    tb.signal(id, KICK).unwrap();
    tb.run_until_idle();
    c.send_to(b"b", server).unwrap();
    std::thread::sleep(Duration::from_millis(20));
    assert_eq!(tb.poll_io(), 0);
    let s = tb.stats(id).unwrap();
    // KICK is on the data lane, IO_READY on the system lane: the readiness
    // was delivered first, so the first datagram was echoed.
    assert_eq!((s.io_events, s.io_stale), (1, 0));

    // A second EO, stopped while registered: its socket is released.
    let (echo, addr) = UdpEcho::new();
    let id = tb.add(EoConfig::new("echo2"), echo).unwrap();
    tb.run_until_idle();
    let server = addr.lock().unwrap().unwrap();
    tb.stop(id).unwrap();
    tb.run_until_idle();
    c.send_to(b"c", server).unwrap();
    std::thread::sleep(Duration::from_millis(20));
    assert_eq!(tb.poll_io(), 0);
}
