//! PDU session establishment through the SMF and UPF EOs, an AMF probe
//! carrying the N1 and N2 containers.

use librecn_ngap::*;
use librecn_oam::{CapturedLog, LogSink};
use librecn_sbi::pdu_session::{
    CreateSmContext, CreateSmContextResult, ReleaseSmContext, Slice, UpdateSmContext, UpdateSmContextResult, n2,
};
use librecn_sbi::{DnnText, Outcome, Supi};
use librecn_scheduler::testbed::{Probe, ProbeLog, TestBed};
use librecn_scheduler::{EoConfig, EoId};
use librecn_smf::Smf;
use librecn_upf::Upf;
use rasn::types::{BitString, FixedOctetString, Integer};
use zerocopy::FromZeros;

fn hex(s: &str) -> Vec<u8> {
    let s: String = s.split_whitespace().collect();
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

struct Core {
    tb: TestBed,
    log: CapturedLog,
    amf: EoId,
    smf: EoId,
    seen: ProbeLog,
}

fn core() -> Core {
    let mut tb = TestBed::new();
    let (sink, log) = LogSink::capture();
    tb.add(LogSink::config(), sink).unwrap();
    let (cfg, upf) = Upf::from_params("upf", &serde_json::Value::Null).unwrap();
    tb.add(cfg, upf).unwrap();
    let (cfg, smf) = Smf::from_params("smf", &serde_json::Value::Null).unwrap();
    let smf = tb.add(cfg, smf).unwrap();
    let (probe, seen) = Probe::new();
    let amf = tb.add(EoConfig::new("amf").buffers(&[(256, 8), (2048, 8)]), probe).unwrap();
    tb.run_until_idle();
    Core { tb, log, amf, smf, seen }
}

impl Core {
    fn create(&mut self, dnn: &str, n1: &[u8]) -> CreateSmContextResult {
        let mut request = CreateSmContext::new_zeroed();
        request.supi = Supi::new("imsi-999700000000001").unwrap();
        request.pdu_session_id = 1;
        request.slice = Slice { sst: 1, has_sd: 0, sd: [0; 3] };
        request.dnn = DnnText::new(dnn).unwrap();
        request.n1.set(n1);
        self.tb.inject_from(self.amf, self.smf, 42, &request).unwrap();
        self.tb.run_until_idle();
        let got = self.seen.take();
        assert_eq!(got[0].header.corr_id, 42);
        got[0].body().unwrap()
    }

    fn logged(&self, eo: &str, needle: &str) -> bool {
        self.log.lines().iter().any(|l| l.eo == eo && l.msg.contains(needle))
    }
}

/// The gNB's answer: its tunnel endpoint for the default flow.
fn response_transfer() -> Vec<u8> {
    let tunnel = UPTransportLayerInformation::gTPTunnel(GTPTunnel::new(
        TransportLayerAddress(BitString::from_slice(&[127, 0, 0, 1])),
        GTPTEID(FixedOctetString::new([0, 0, 0, 9])),
        None,
    ));
    let flows =
        AssociatedQosFlowList(vec![AssociatedQosFlowItem::new(QosFlowIdentifier(Integer::from(1)), None, None)]);
    let transfer = PDUSessionResourceSetupResponseTransfer::new(
        QosFlowPerTNLInformation::new(tunnel, flows, None),
        None,
        None,
        None,
        None,
    );
    wrap(&transfer).unwrap().as_bytes().to_vec()
}

#[test]
fn a_session_is_established_set_up_on_the_gnb_and_released() {
    let mut c = core();
    let result = c.create("internet", &hex("2e 01 01 c1 ffff 91 a1"));
    assert_eq!(result.pdu_session_id, 1);
    assert_eq!(result.n1.get()[..4], hex("2e0101c2"), "PDU Session Establishment Accept");
    assert!(result.n1.get().windows(5).any(|w| w == [0x01, 10, 45, 0, 2]), "the first UE address");
    let transfer: PDUSessionResourceSetupRequestTransfer = open(&Any::new(result.n2.get().to_vec())).unwrap();
    assert_eq!(transfer.protocol_ies.0.len(), 4);
    assert!(c.logged("upf", "UE 10.45.0.2, uplink TEID 0x1"));

    let mut update = UpdateSmContext::new_zeroed();
    (update.sm_ctx, update.n2_kind) = (result.sm_ctx, n2::SETUP_RESPONSE);
    update.n2.set(&response_transfer());
    c.tb.inject_from(c.amf, c.smf, 43, &update).unwrap();
    c.tb.run_until_idle();
    let answer: UpdateSmContextResult = c.seen.take()[0].body().unwrap();
    assert_eq!((answer.outcome(), answer.pdu_session_id), (Outcome::Ok, 1));
    assert!(c.logged("upf", "downlink to 127.0.0.1 TEID 0x9"), "{:#?}", c.log.lines());

    c.tb.inject_from(c.amf, c.smf, 0, &ReleaseSmContext { sm_ctx: result.sm_ctx }).unwrap();
    c.tb.run_until_idle();
    assert!(c.logged("smf", "PDU session 1 released, 0 left"));
    assert!(c.logged("upf", "deleted, 0 left"));
    let again = c.create("internet", &hex("2e 01 02 c1 ffff 91"));
    assert!(again.n1.get().windows(5).any(|w| w == [0x01, 10, 45, 0, 3]), "the next address");
}

#[test]
fn a_refused_request_gets_a_reject_without_n2() {
    let mut c = core();
    let result = c.create("ims", &hex("2e 01 05 c1 ffff 91"));
    assert_eq!(result.n1.get(), hex("2e0105c31b"), "Reject, #27 missing or unknown DNN");
    assert!(result.n2.get().is_empty());
    assert!(!c.logged("upf", "session 0x"), "the UPF is not asked");
}

#[test]
fn without_a_upf_the_request_is_refused_for_resources() {
    let mut tb = TestBed::new();
    let (cfg, smf) = Smf::from_params("smf", &serde_json::Value::Null).unwrap();
    let smf = tb.add(cfg, smf).unwrap();
    let (probe, seen) = Probe::new();
    let amf = tb.add(EoConfig::new("amf").buffers(&[(2048, 4)]), probe).unwrap();
    let mut request = CreateSmContext::new_zeroed();
    (request.pdu_session_id, request.slice.sst) = (1, 1);
    request.n1.set(&hex("2e 01 01 c1 ffff 91"));
    tb.inject_from(amf, smf, 1, &request).unwrap();
    tb.run_until_idle();
    let result: CreateSmContextResult = seen.take()[0].body().unwrap();
    assert_eq!(result.n1.get(), hex("2e0101c31a"), "Reject, #26 insufficient resources");
}
