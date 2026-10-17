//! Subscribers provisioned through the web API, then used for authentication.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use librecn_oam::{CapturedLog, LogSink, Server};
use librecn_sbi::ue_authentication::{AuthData, AuthInfo, GetAuthData};
use librecn_sbi::{Outcome, ServingNetwork, SupiOrSuci};
use librecn_scheduler::testbed::{Probe, TestBed};
use librecn_scheduler::{EoConfig, EoId};
use librecn_udm::Udm;

/// One HTTP request from a client thread, while the test bed serves it.
fn http(tb: &mut TestBed, addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, String) {
    let request = format!("{method} {path} HTTP/1.1\r\nHost: x\r\nContent-Length: {}\r\n\r\n{body}", body.len());
    let client = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    });
    while !client.is_finished() {
        tb.wait_io(Duration::from_millis(5));
        tb.run_until_idle();
    }
    let response = client.join().unwrap();
    let status = response[9..12].parse().unwrap();
    let body = response.split_once("\r\n\r\n").map(|(_, b)| b.to_string()).unwrap_or_default();
    (status, body)
}

fn setup() -> (TestBed, SocketAddr, CapturedLog, EoId) {
    let mut tb = TestBed::new();
    let (sink, log) = LogSink::capture();
    tb.add(LogSink::config(), sink).unwrap();
    let server = Server::new("127.0.0.1:0".parse().unwrap());
    let bound = server.bound_addr();
    tb.add(Server::config(), server).unwrap();
    let (cfg, udm) = Udm::from_params("udm", &serde_json::Value::Null).unwrap();
    let udm = tb.add(cfg, udm).unwrap();
    tb.run_until_idle();
    let addr = bound.lock().unwrap().expect("listening");
    (tb, addr, log, udm)
}

/// Whether the UDM gives a vector for `suci`.
fn authenticates(tb: &mut TestBed, udm: EoId, suci: &str) -> bool {
    let (probe, seen) = Probe::new();
    let ausf = tb.add(EoConfig::new(format!("ausf-{suci}")).buffers(&[(256, 4)]), probe).unwrap();
    let info =
        AuthInfo::new(SupiOrSuci::new(suci).unwrap(), ServingNetwork::new("5G:mnc070.mcc999.3gppnetwork.org").unwrap());
    tb.inject_from(ausf, udm, 1, &GetAuthData { info }).unwrap();
    tb.run_until_idle();
    seen.take()[0].body::<AuthData>().unwrap().outcome() == Outcome::Ok
}

#[test]
fn subscribers_are_provisioned_listed_and_removed_on_the_web_api() {
    let (mut tb, addr, log, udm) = setup();
    let suci = "suci-0-999-70-0000-0-0-0000000001";
    assert!(!authenticates(&mut tb, udm, suci));

    let subscriber = r#"{"supi": "imsi-999700000000001", "k": "465B5CE8B199B49FAA5F0A2EE238A6BC",
                         "opc": "E8ED289DEBA952E4283B54E88E6183CA"}"#;
    let (status, body) = http(&mut tb, addr, "POST", "/api/nfs/udm/subscribers", subscriber);
    assert_eq!((status, body.as_str()), (201, r#"{"added":["imsi-999700000000001"]}"#));
    assert!(authenticates(&mut tb, udm, suci), "usable at once");
    assert!(log.lines().iter().any(|l| l.msg == "subscriber(s) added: imsi-999700000000001, 1 subscriber(s)"));

    let (status, body) = http(&mut tb, addr, "GET", "/api/nfs/udm/subscribers", "");
    assert_eq!(status, 200);
    assert_eq!(body, r#"[{"amf":"8000","sqn":"000000000020","supi":"imsi-999700000000001"}]"#, "SQN used once");
    assert_eq!(http(&mut tb, addr, "POST", "/api/nfs/udm/subscribers", subscriber).0, 409);

    let (status, _) = http(&mut tb, addr, "DELETE", "/api/nfs/udm/subscribers/imsi-999700000000001", "");
    assert_eq!(status, 200);
    assert!(!authenticates(&mut tb, udm, suci));
    assert_eq!(http(&mut tb, addr, "GET", "/api/nfs/udm/keys", "").0, 404);
    let (status, body) = http(&mut tb, addr, "GET", "/api/nfs/nobody/subscribers", "");
    assert_eq!((status, body.as_str()), (404, r#"{"error":"no NF \"nobody\""}"#));
}
