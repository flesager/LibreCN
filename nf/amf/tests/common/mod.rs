//! The AMF on the test bed, and a test gNB on a real SCTP association.
#![allow(dead_code)] // each test binary uses part of it

use std::os::fd::{AsFd, AsRawFd};
use std::time::Duration;

use librecn_amf::Amf;
use librecn_amf::config::AmfConfig;
use librecn_amf::sctp::{NGAP_PPID, Received, SctpSocket};
use librecn_ngap::*;
use librecn_oam::{AlarmManager, CapturedLog, LogSink};
use librecn_scheduler::EoId;
use librecn_scheduler::testbed::TestBed;

pub fn vector(name: &str) -> Vec<u8> {
    std::fs::read(format!("{}/../../proto/ngap/tests/vectors/free5gc/{name}.bin", env!("CARGO_MANIFEST_DIR"))).unwrap()
}

pub struct Setup {
    pub tb: TestBed,
    pub log: CapturedLog,
    pub addr: std::net::SocketAddr,
    pub amf: EoId,
}

/// Test bed with log, alarms and an AMF on a free port, serving `plmn`.
pub fn amf(mcc: &str, mnc: &str) -> Setup {
    amf_with(mcc, mnc, |_| {})
}

/// Same, `more` adding EOs (AUSF, UDM) before the AMF starts.
pub fn amf_with(mcc: &str, mnc: &str, more: impl FnOnce(&mut TestBed)) -> Setup {
    let mut tb = TestBed::new();
    let (sink, log) = LogSink::capture();
    tb.add(LogSink::config(), sink).unwrap();
    tb.add(AlarmManager::config(), AlarmManager::new()).unwrap();
    more(&mut tb);
    let params = serde_json::json!({ "n2": "127.0.0.1:0", "plmn": { "mcc": mcc, "mnc": mnc } });
    let amf = Amf::new(AmfConfig::from_params("amf-1", &params).unwrap());
    let bound = amf.bound_addr();
    let amf = tb.add(amf.eo_config(), amf).unwrap();
    tb.run_until_idle();
    let addr = bound.lock().unwrap().expect("N2 open");
    Setup { tb, log, addr, amf }
}

impl Setup {
    /// Runs the test bed until `gnb` has something to read, or 2 s passed.
    pub fn pump_until_readable(&mut self, gnb: &SctpSocket) -> bool {
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
    pub fn pump(&mut self) {
        for _ in 0..20 {
            self.tb.wait_io(Duration::from_millis(5));
            self.tb.run_until_idle();
        }
    }

    pub fn connect(&mut self) -> SctpSocket {
        let gnb = SctpSocket::connect(self.addr).unwrap();
        self.pump();
        gnb
    }

    /// Sends `pdu` from the gNB and returns the AMF's answer, which must be on stream 0.
    pub fn exchange(&mut self, gnb: &SctpSocket, pdu: &[u8]) -> NGAPPDU {
        gnb.send(0, 0, NGAP_PPID, pdu).unwrap();
        let (stream, answer) = self.receive(gnb);
        assert_eq!(stream, 0, "non-UE signalling on stream 0");
        answer
    }

    /// The AMF's next PDU to the gNB, with its stream.
    pub fn receive(&mut self, gnb: &SctpSocket) -> (u16, NGAPPDU) {
        assert!(self.pump_until_readable(gnb), "nothing from the AMF");
        let mut buf = [0u8; 4096];
        let Some(Received::Data { data, ppid, stream, .. }) = gnb.recv(&mut buf).unwrap() else { panic!() };
        assert_eq!(ppid, NGAP_PPID);
        (stream, decode(data).unwrap())
    }

    pub fn logged(&self, eo: &str, needle: &str) -> bool {
        self.log.lines().iter().any(|l| l.eo == eo && l.msg.contains(needle))
    }
}

pub fn initiating<T: rasn::Decode>(pdu: &NGAPPDU, code: ProcedureCode) -> T {
    let NGAPPDU::initiatingMessage(m) = pdu else { panic!("not an initiating message: {pdu:?}") };
    assert_eq!(m.procedure_code, code);
    open(&m.value).unwrap()
}

pub fn release_complete(amf_id: u64, ran_id: u32) -> Vec<u8> {
    let ie = |id: ProtocolIEID, value: Any| {
        AnonymousUEContextReleaseCompleteProtocolIEs::new(id, Criticality::ignore, value)
    };
    let ies = vec![
        ie(ID_AMF_UE_NGAP_ID, wrap(&AMFUENGAPID(amf_id)).unwrap()),
        ie(ID_RAN_UE_NGAP_ID, wrap(&RANUENGAPID(ran_id)).unwrap()),
    ];
    let message = wrap(&UEContextReleaseComplete::new(UEContextReleaseCompleteProtocolIEs(ies))).unwrap();
    encode(&NGAPPDU::successfulOutcome(SuccessfulOutcome::new(ID_UECONTEXT_RELEASE, Criticality::reject, message)))
        .unwrap()
}
