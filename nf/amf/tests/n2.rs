//! The AMF EO on the test bed, with a test gNB on a real SCTP association.

mod common;

use common::{amf, initiating, release_complete, vector};
use librecn_amf::sctp::NGAP_PPID;
use librecn_ngap::*;

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
    open(&indication.protocol_ies.0.iter().find(|ie| ie.id == ID_CAUSE.0).expect("a cause").value).unwrap()
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

#[test]
fn initial_ue_message_gets_a_downlink_nas_then_a_release() {
    let mut s = amf("208", "93");
    let gnb = s.connect();
    assert!(matches!(s.exchange(&gnb, &vector("ci0000")), NGAPPDU::successfulOutcome(_)));

    // ci0002: Initial UE Message carrying a plain Registration Request.
    gnb.send(0, 1, NGAP_PPID, &vector("ci0002")).unwrap();
    let (stream, pdu) = s.receive(&gnb);
    assert_ne!(stream, 0, "UE signalling off stream 0");
    let downlink: DownlinkNASTransport = initiating(&pdu, ID_DOWNLINK_NASTRANSPORT);
    let ie = |id: ProtocolIEID| &downlink.protocol_ies.0.iter().find(|ie| ie.id == id.0).unwrap().value;
    let amf_id = open::<AMFUENGAPID>(ie(ID_AMF_UE_NGAP_ID)).unwrap().0;
    let ran_id = open::<RANUENGAPID>(ie(ID_RAN_UE_NGAP_ID)).unwrap().0;
    assert_eq!(&*open::<NASPDU>(ie(ID_NAS_PDU)).unwrap().0, [0x7e, 0x00, 0x44, 0x6f], "Registration Reject #111");

    let (stream2, pdu) = s.receive(&gnb);
    assert_eq!(stream2, stream, "the UE keeps its stream");
    let command: UEContextReleaseCommand = initiating(&pdu, ID_UECONTEXT_RELEASE);
    let ids = command.protocol_ies.0.iter().find(|ie| ie.id == ID_UE_NGAP_IDS).unwrap();
    let UENGAPIDs::uE_NGAP_ID_pair(pair) = open(&ids.value).unwrap() else { panic!() };
    assert_eq!((pair.a_mf_ue_ngap_id.0, pair.r_an_ue_ngap_id.0), (amf_id, ran_id));

    gnb.send(0, stream, NGAP_PPID, &release_complete(amf_id, ran_id)).unwrap();
    s.pump();
    assert!(s.logged("amf-1", "Registration Request (plain)"));
    assert!(s.logged("amf-1", &format!("UE {amf_id}: released, 0 UE(s) left")), "{:#?}", s.log.lines());
}

#[test]
fn unknown_ue_ids_get_an_error_indication() {
    let mut s = amf("208", "93");
    let gnb = s.connect();
    assert!(matches!(s.exchange(&gnb, &vector("ci0000")), NGAPPDU::successfulOutcome(_)));
    // ci0004: Uplink NAS Transport for an AMF-UE-NGAP-ID this AMF never gave.
    let answer = s.exchange(&gnb, &vector("ci0004"));
    assert_eq!(error_cause(&answer), Cause::radioNetwork(CauseRadioNetwork::unknown_local_UE_NGAP_ID));
}

#[test]
fn ue_signalling_before_ng_setup_is_refused() {
    let mut s = amf("208", "93");
    let gnb = s.connect();
    let answer = s.exchange(&gnb, &vector("ci0002"));
    assert_eq!(error_cause(&answer), Cause::protocol(CauseProtocol::message_not_compatible_with_receiver_state));
}

#[test]
fn ues_are_released_with_their_gnb() {
    let mut s = amf("208", "93");
    let gnb = s.connect();
    assert!(matches!(s.exchange(&gnb, &vector("ci0000")), NGAPPDU::successfulOutcome(_)));
    gnb.send(0, 1, NGAP_PPID, &vector("ci0002")).unwrap();
    s.receive(&gnb);
    s.receive(&gnb);
    drop(gnb); // no Release Complete: the context dies with the association
    s.pump();
    assert!(s.logged("amf-1", "1 UE(s) released"), "{:#?}", s.log.lines());
}
