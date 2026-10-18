//! PDU session establishment through the PCF, SMF and UPF EOs, an AMF probe
//! carrying the N1 and N2 containers.

use librecn_common::manage::{Manage, Method, Reply};
use librecn_ngap::*;
use librecn_oam::{CapturedLog, LogSink};
use librecn_pcf::Pcf;
use librecn_sbi::pdu_session::{
    CreateSmContext, CreateSmContextResult, N4Report, ReleaseSmContext, Slice, SmNotify, SmNotifyResult,
    UpdateSmContext, UpdateSmContextResult, n2, update,
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
    upf: EoId,
    seen: ProbeLog,
}

fn core() -> Core {
    core_with(&serde_json::Value::Null)
}

/// The same, the SMF with `params`.
fn core_with(params: &serde_json::Value) -> Core {
    let mut tb = TestBed::new();
    let (sink, log) = LogSink::capture();
    tb.add(LogSink::config(), sink).unwrap();
    let (cfg, upf) = Upf::from_params("upf", &serde_json::Value::Null).unwrap();
    let upf = tb.add(cfg, upf).unwrap();
    add_pcf(&mut tb);
    let (cfg, smf) = Smf::from_params("smf", params).unwrap();
    let smf = tb.add(cfg, smf).unwrap();
    let (probe, seen) = Probe::new();
    let amf = tb.add(EoConfig::new("amf").buffers(&[(256, 8), (2048, 8), (16 * 1024, 2)]), probe).unwrap();
    tb.run_until_idle();
    Core { tb, log, amf, smf, upf, seen }
}

/// The PCF: the subscriber's sessions to `internet` get 20 Mbit/s up, 100 down.
fn add_pcf(tb: &mut TestBed) {
    let params = serde_json::json!({ "policies": [{ "supi": "imsi-999700000000001",
        "sessions": [{ "dnn": "internet", "ambr_mbps": { "uplink": 20, "downlink": 100 } }] }] });
    let (cfg, pcf) = Pcf::from_params("pcf", &params).unwrap();
    tb.add(cfg, pcf).unwrap();
}

impl Core {
    fn create(&mut self, dnn: &str, n1: &[u8]) -> CreateSmContextResult {
        self.create_session(1, dnn, n1)
    }

    fn create_session(&mut self, psi: u8, dnn: &str, n1: &[u8]) -> CreateSmContextResult {
        let mut request = CreateSmContext::new_zeroed();
        request.supi = Supi::new("imsi-999700000000001").unwrap();
        request.pdu_session_id = psi;
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
    let ambr = [6, 6, 0, 100, 6, 0, 20];
    assert!(result.n1.get().windows(7).any(|w| w == ambr), "the PCF's session AMBR, in Mbit/s");
    assert!(c.logged("pcf", "SM policy for PDU session 1 to internet: 5QI 9, ARP 8, AMBR 20 / 100 Mbit/s"));
    let transfer: PDUSessionResourceSetupRequestTransfer = open(&Any::new(result.n2.get().to_vec())).unwrap();
    assert_eq!(transfer.protocol_ies.0.len(), 4);
    assert!(c.logged("upf", "UE 10.45.0.2, uplink TEID 0x1"));

    let mut request = UpdateSmContext::new_zeroed();
    (request.sm_ctx, request.kind) = (result.sm_ctx, update::SETUP_RESPONSE);
    request.container.set(&response_transfer());
    c.tb.inject_from(c.amf, c.smf, 43, &request).unwrap();
    c.tb.run_until_idle();
    let answer: UpdateSmContextResult = c.seen.take()[0].body().unwrap();
    assert_eq!((answer.outcome(), answer.pdu_session_id), (Outcome::Ok, 1));
    assert!(c.logged("upf", "downlink to 127.0.0.1 TEID 0x9"), "{:#?}", c.log.lines());

    // The UE goes idle: no downlink; it comes back: N2 again for the same tunnel.
    request.kind = update::DEACTIVATE;
    c.tb.inject_from(c.amf, c.smf, 44, &request).unwrap();
    c.tb.run_until_idle();
    let ack: UpdateSmContextResult = c.seen.take()[0].body().unwrap();
    assert_eq!((ack.outcome(), ack.n2.get().len()), (Outcome::Ok, 0), "a deactivation is acknowledged, nothing more");
    assert!(c.logged("upf", "downlink deactivated"));
    request.kind = update::ACTIVATE;
    c.tb.inject_from(c.amf, c.smf, 45, &request).unwrap();
    c.tb.run_until_idle();
    let again: UpdateSmContextResult = c.seen.take()[0].body().unwrap();
    assert_eq!((again.outcome(), again.pdu_session_id), (Outcome::Ok, 1));
    assert_eq!(again.n2.get(), result.n2.get(), "the same setup as at establishment");

    c.tb.inject_from(c.amf, c.smf, 0, &ReleaseSmContext { sm_ctx: result.sm_ctx }).unwrap();
    c.tb.run_until_idle();
    assert!(c.logged("smf", "PDU session 1 released, 0 left"));
    assert!(c.logged("upf", "deleted, 0 left"));
    let ack: UpdateSmContextResult = c.seen.take()[0].body().unwrap();
    assert_eq!(ack.released, 1, "a release is acknowledged");
    let again = c.create("internet", &hex("2e 01 02 c1 ffff 91"));
    assert!(again.n1.get().windows(5).any(|w| w == [0x01, 10, 45, 0, 3]), "the next address");
}

impl Core {
    /// A 5GSM message of the UE for session `sm_ctx`.
    fn n1(&mut self, sm_ctx: u32, n1: &[u8]) -> UpdateSmContextResult {
        let mut request = UpdateSmContext::new_zeroed();
        (request.sm_ctx, request.kind) = (sm_ctx, update::N1);
        request.container.set(n1);
        self.tb.inject_from(self.amf, self.smf, 46, &request).unwrap();
        self.tb.run_until_idle();
        self.seen.take()[0].body().unwrap()
    }
}

#[test]
fn the_ue_releases_its_session_and_cannot_modify_it() {
    let mut c = core();
    let sm_ctx = c.create("internet", &hex("2e 01 01 c1 ffff 91 a1")).sm_ctx;
    let modify = c.n1(sm_ctx, &hex("2e0103c9"));
    assert_eq!((modify.n1.get(), modify.released), (&hex("2e0103ca1f")[..], 0), "Modification Reject #31");

    let release = c.n1(sm_ctx, &hex("2e0102d1 5924"));
    assert_eq!(release.n1.get(), hex("2e0102d324"), "Release Command, the UE's PTI, #36");
    assert_eq!(release.n2_kind, n2::RELEASE);
    let transfer: PDUSessionResourceReleaseCommandTransfer = open(&Any::new(release.n2.get().to_vec())).unwrap();
    assert_eq!(transfer.cause, Cause::nas(CauseNas::normal_release));
    assert!(c.logged("upf", "deleted, 0 left"), "the user plane goes at once");
    assert_eq!(release.released, 0, "until the UE completes");

    let complete = c.n1(sm_ctx, &hex("2e0102d4"));
    assert_eq!(complete.released, 1);
    assert!(c.logged("smf", "PDU session 1 released, 0 left"));
    assert_eq!(c.n1(sm_ctx, &hex("2e0102d4")).outcome(), Outcome::ContextNotFound);
}

#[test]
fn downlink_data_for_an_idle_ue_goes_to_the_amf_with_the_n2_setup() {
    let mut c = core();
    let result = c.create("internet", &hex("2e 01 01 c1 ffff 91 a1"));
    let mut request = UpdateSmContext::new_zeroed();
    (request.sm_ctx, request.kind) = (result.sm_ctx, update::SETUP_RESPONSE);
    request.container.set(&response_transfer());
    c.tb.inject_from(c.amf, c.smf, 43, &request).unwrap();
    c.tb.run_until_idle();
    c.seen.take();
    let report = N4Report { seid: u64::from(result.sm_ctx) };
    // The downlink is up: a stale report is ignored.
    c.tb.inject_from(c.upf, c.smf, 0, &report).unwrap();
    c.tb.run_until_idle();
    assert!(c.seen.take().is_empty());

    request.kind = update::DEACTIVATE;
    c.tb.inject_from(c.amf, c.smf, 0, &request).unwrap();
    c.tb.run_until_idle();
    c.seen.take();
    c.tb.inject_from(c.upf, c.smf, 0, &report).unwrap();
    c.tb.run_until_idle();
    let got = c.seen.take();
    let notify: SmNotify = got[0].body().expect("the AMF is told");
    assert_eq!((notify.sm_ctx, notify.pdu_session_id, notify.n2_kind), (result.sm_ctx, 1, n2::SETUP));
    assert_eq!((notify.n1.get(), notify.n2.get()), (&[][..], result.n2.get()), "no N1, the setup of the session");
    assert!(c.logged("smf", "PDU session 1: downlink data, the AMF told"));

    // The AMF pages the UE: the session stays.
    let mut paged = SmNotifyResult::new_zeroed();
    paged.sm_ctx = result.sm_ctx;
    c.tb.inject_from(c.amf, c.smf, 0, &paged).unwrap();
    c.tb.run_until_idle();
    assert!(!c.logged("smf", "released"));
}

#[test]
fn the_network_releases_a_session_through_the_amf() {
    let mut c = core();
    let sm_ctx = c.create("internet", &hex("2e 01 01 c1 ffff 91 a1")).sm_ctx;
    let mut manage = Manage::new_zeroed();
    manage.fill(Method::Get, "sessions", "");
    c.tb.inject_from(c.amf, c.smf, 50, &manage).unwrap();
    c.tb.run_until_idle();
    let listed = c.seen.take();
    let listed: serde_json::Value = serde_json::from_str(listed[0].body::<Reply>().unwrap().text()).unwrap();
    assert_eq!((&listed["count"], &listed["sessions"][0]["state"]), (&1.into(), &"idle".into()), "{listed}");

    manage.fill(Method::Delete, &format!("sessions/{sm_ctx}"), "");
    c.tb.inject_from(c.amf, c.smf, 51, &manage).unwrap();
    c.tb.run_until_idle();
    let got = c.seen.take();
    let notify = got.iter().find_map(|m| m.body::<SmNotify>()).expect("the AMF is told");
    assert_eq!(got.iter().find(|m| m.body::<SmNotify>().is_some()).unwrap().header.corr_id, 42, "the AMF's UE id");
    assert_eq!((notify.n1.get(), notify.n2_kind), (&hex("2e0100d324")[..], n2::RELEASE), "PTI 0: the network's");

    // The UE is idle: the AMF cannot deliver, the SMF ends the session alone.
    let mut result = SmNotifyResult::new_zeroed();
    (result.sm_ctx, _) = (sm_ctx, result.set_outcome(Outcome::ContextNotFound));
    c.tb.inject_from(c.amf, c.smf, 0, &result).unwrap();
    c.tb.run_until_idle();
    assert!(c.logged("smf", "PDU session 1 released, 0 left"));
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
fn without_a_pcf_or_a_upf_the_request_is_refused_for_resources() {
    let mut tb = TestBed::new();
    let (cfg, smf) = Smf::from_params("smf", &serde_json::Value::Null).unwrap();
    let smf = tb.add(cfg, smf).unwrap();
    let (probe, seen) = Probe::new();
    let amf = tb.add(EoConfig::new("amf").buffers(&[(2048, 4)]), probe).unwrap();
    let mut request = CreateSmContext::new_zeroed();
    (request.pdu_session_id, request.slice.sst) = (1, 1);
    request.n1.set(&hex("2e 01 01 c1 ffff 91"));
    for pcf in [false, true] {
        if pcf {
            add_pcf(&mut tb);
        }
        tb.inject_from(amf, smf, 1, &request).unwrap();
        tb.run_until_idle();
        let result: CreateSmContextResult = seen.take()[0].body().unwrap();
        assert_eq!(result.n1.get(), hex("2e0101c31a"), "Reject, #26 insufficient resources (PCF {pcf})");
    }
}

#[test]
fn a_ue_gets_sessions_on_two_dnns_each_from_its_pool() {
    let params = serde_json::json!({ "dnns": [{ "dnn": "internet", "pool": "10.45.0.0/16" },
                                              { "dnn": "ims", "pool": "10.46.0.0/16" }] });
    let mut c = core_with(&params);
    assert!(c.logged("smf", "DNN internet (10.45.0.0/16), ims (10.46.0.0/16)"), "{:#?}", c.log.lines());
    let internet = c.create_session(1, "internet", &hex("2e 01 01 c1 ffff 91 a1"));
    let ims = c.create_session(2, "ims", &hex("2e 02 02 c1 ffff 91 a1"));
    let default = c.create_session(3, "", &hex("2e 03 03 c1 ffff 91 a1"));
    let address =
        |r: &CreateSmContextResult, a: [u8; 4]| r.n1.get().windows(5).any(|w| w == [0x01, a[0], a[1], a[2], a[3]]);
    assert!(address(&internet, [10, 45, 0, 2]) && address(&ims, [10, 46, 0, 2]), "each DNN its pool");
    assert!(address(&default, [10, 45, 0, 3]), "no DNN asked: the first one");
    assert_ne!(internet.sm_ctx, ims.sm_ctx);

    let mut manage = Manage::new_zeroed();
    manage.fill(Method::Get, "sessions", "");
    c.tb.inject_from(c.amf, c.smf, 50, &manage).unwrap();
    c.tb.run_until_idle();
    let reply: Reply = c.seen.take()[0].body().unwrap();
    let list: serde_json::Value = serde_json::from_str(reply.text()).unwrap();
    let dnns: Vec<_> =
        list["sessions"].as_array().unwrap().iter().map(|s| s["dnn"].as_str().unwrap().to_string()).collect();
    assert_eq!(dnns, ["internet", "ims", "internet"]);
}
