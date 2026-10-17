//! The AMF EO on the test bed, with a test gNB on a real SCTP association.

use std::os::fd::{AsFd, AsRawFd};
use std::time::Duration;

use librecn_amf::Amf;
use librecn_amf::config::AmfConfig;
use librecn_amf::sctp::{NGAP_PPID, Received, SctpSocket};
use librecn_ngap::*;
use librecn_oam::{AlarmManager, CapturedLog, LogSink};
use librecn_scheduler::testbed::TestBed;

fn vector(name: &str) -> Vec<u8> {
    std::fs::read(format!("{}/../../proto/ngap/tests/vectors/free5gc/{name}.bin", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

struct Setup {
    tb: TestBed,
    log: CapturedLog,
    addr: std::net::SocketAddr,
}

/// Test bed with log, alarms and an AMF on a free port, serving `plmn`.
fn amf(mcc: &str, mnc: &str) -> Setup {
    let mut tb = TestBed::new();
    let (sink, log) = LogSink::capture();
    tb.add(LogSink::config(), sink).unwrap();
    tb.add(AlarmManager::config(), AlarmManager::new()).unwrap();
    let params = serde_json::json!({ "n2": "127.0.0.1:0", "plmn": { "mcc": mcc, "mnc": mnc } });
    let amf = Amf::new(AmfConfig::from_params("amf-1", &params).unwrap());
    let bound = amf.bound_addr();
    tb.add(Amf::eo_config("amf-1"), amf).unwrap();
    tb.run_until_idle();
    let addr = bound.lock().unwrap().expect("N2 open");
    Setup { tb, log, addr }
}

impl Setup {
    /// Runs the test bed until `gnb` has something to read, or 2 s passed.
    fn pump_until_readable(&mut self, gnb: &SctpSocket) -> bool {
        for _ in 0..400 {
            self.tb.wait_io(Duration::from_millis(5));
            self.tb.run_until_idle();
            let mut pollfd = libc::pollfd { fd: gnb.as_fd().as_raw_fd(), events: libc::POLLIN, revents: 0 };
            // SAFETY: one valid pollfd.
            if unsafe { libc::poll(&mut pollfd, 1, 0) } == 1 {
                return true;
            }
        }
        false
    }

    /// Lets the AMF handle what the gNB sent, without expecting an answer.
    fn pump(&mut self) {
        for _ in 0..20 {
            self.tb.wait_io(Duration::from_millis(5));
            self.tb.run_until_idle();
        }
    }

    fn connect(&mut self) -> SctpSocket {
        let gnb = SctpSocket::connect(self.addr).unwrap();
        self.pump();
        gnb
    }

    /// Sends `pdu` from the gNB and returns the AMF's answer.
    fn exchange(&mut self, gnb: &SctpSocket, pdu: &[u8]) -> NGAPPDU {
        gnb.send(0, 0, NGAP_PPID, pdu).unwrap();
        assert!(self.pump_until_readable(gnb), "no answer from the AMF");
        let mut buf = [0u8; 4096];
        let Some(Received::Data { data, ppid, stream, .. }) = gnb.recv(&mut buf).unwrap() else { panic!() };
        assert_eq!((ppid, stream), (NGAP_PPID, 0), "NGAP on stream 0");
        decode(data).unwrap()
    }

    fn logged(&self, eo: &str, needle: &str) -> bool {
        self.log.lines().iter().any(|l| l.eo == eo && l.msg.contains(needle))
    }
}

#[test]
fn ng_setup_is_accepted_for_the_served_plmn() {
    let mut s = amf("208", "93");
    assert!(s.log.lines().iter().any(|l| l.level == "VIP" && l.msg.contains("AMF amf-1 ready")));
    let gnb = s.connect();
    let answer = s.exchange(&gnb, &vector("ci0000"));

    let NGAPPDU::successfulOutcome(outcome) = &answer else { panic!("not accepted: {answer:?}") };
    assert_eq!(outcome.procedure_code, ID_NGSETUP);
    let response: NGSetupResponse = open(&outcome.value).unwrap();
    let name = response.protocol_ies.0.iter().find(|ie| ie.id == ID_AMFNAME.0).unwrap();
    assert_eq!(open::<AMFName>(&name.value).unwrap().0.as_bytes(), b"amf-1");
    assert!(s.logged("amf-1", "NG Setup accepted: free5gc"));
}

#[test]
fn ng_setup_is_refused_for_another_plmn() {
    let mut s = amf("999", "70");
    let gnb = s.connect();
    let NGAPPDU::unsuccessfulOutcome(outcome) = s.exchange(&gnb, &vector("ci0000")) else { panic!("not refused") };
    let failure: NGSetupFailure = open(&outcome.value).unwrap();
    assert_eq!(open::<Cause>(&failure.protocol_ies.0[0].value).unwrap(), Cause::misc(CauseMisc::unknown_PLMN));
}

fn error_cause(pdu: &NGAPPDU) -> Cause {
    let NGAPPDU::initiatingMessage(m) = pdu else { panic!("not an Error Indication: {pdu:?}") };
    assert_eq!(m.procedure_code, ID_ERROR_INDICATION);
    let indication: ErrorIndication = open(&m.value).unwrap();
    open(&indication.protocol_ies.0[0].value).unwrap()
}

#[test]
fn malformed_ngap_gets_an_error_indication() {
    let mut s = amf("208", "93");
    let gnb = s.connect();
    let answer = s.exchange(&gnb, &[0xff, 0xff, 0xff]);
    assert_eq!(error_cause(&answer), Cause::protocol(CauseProtocol::transfer_syntax_error));
    assert!(s.logged("amf-1", "malformed NGAP"));
}

#[test]
fn ng_reset_is_refused_before_setup_and_acknowledged_after() {
    let mut s = amf("208", "93");
    let gnb = s.connect();
    let reset = vector("ci0709");
    let answer = s.exchange(&gnb, &reset);
    assert_eq!(error_cause(&answer), Cause::protocol(CauseProtocol::message_not_compatible_with_receiver_state));

    assert!(matches!(s.exchange(&gnb, &vector("ci0000")), NGAPPDU::successfulOutcome(_)));
    let NGAPPDU::successfulOutcome(ack) = s.exchange(&gnb, &reset) else { panic!("no acknowledge") };
    assert_eq!(ack.procedure_code, ID_NGRESET);
}

#[test]
fn losing_a_set_up_gnb_raises_an_alarm_cleared_at_its_next_setup() {
    let mut s = amf("208", "93");
    let gnb = s.connect();
    assert!(matches!(s.exchange(&gnb, &vector("ci0000")), NGAPPDU::successfulOutcome(_)));
    drop(gnb);
    s.pump();
    assert!(s.logged("amf-1", "N2 to free5gc lost"));
    assert!(s.logged("alarm", "alarm raised MAJOR amf-1/258: N2 to free5gc lost"), "{:#?}", s.log.lines());

    let gnb = s.connect();
    assert!(matches!(s.exchange(&gnb, &vector("ci0000")), NGAPPDU::successfulOutcome(_)));
    s.pump();
    assert!(s.logged("alarm", "alarm cleared"), "{:#?}", s.log.lines());
}
