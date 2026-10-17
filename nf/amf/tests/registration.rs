//! Registration and PDU session end to end: a test gNB on SCTP, the AMF,
//! AUSF, UDM, SMF and UPF EOs on the test bed, and a UE computing its side
//! of 5G AKA and NAS security.

mod common;

use common::{Setup, amf_with, initiating, release_complete, vector};
use librecn_amf::sctp::{NGAP_PPID, SctpSocket};
use librecn_ausf::Ausf;
use librecn_ngap::*;
use librecn_security::milenage::Milenage;
use librecn_security::{Direction, kdf, nea2, nia2};
use librecn_smf::Smf;
use librecn_udm::Udm;
use librecn_upf::Upf;
use rasn::types::{BitString, FixedBitString, FixedOctetString, Integer};

const K: &str = "465B5CE8B199B49FAA5F0A2EE238A6BC";
const OPC: &str = "E8ED289DEBA952E4283B54E88E6183CA";
/// free5GC's test gNB (vector ci0000) serves PLMN 208/93.
const SNN: &str = "5G:mnc093.mcc208.3gppnetwork.org";
const PLMN: [u8; 3] = [0x02, 0xf8, 0x39];

fn hex(s: &str) -> Vec<u8> {
    let s: String = s.split_whitespace().collect();
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

/// Core with the UDM knowing `imsi-208930000000001`, SMF and UPF, and a gNB set up.
fn core() -> (Setup, SctpSocket) {
    let mut s = amf_with("208", "93", |tb| {
        let subscriber = serde_json::json!({ "supi": "imsi-208930000000001", "k": K, "opc": OPC });
        let (cfg, udm) = Udm::from_params("udm", &serde_json::json!({ "subscribers": [subscriber] })).unwrap();
        tb.add(cfg, udm).unwrap();
        let (cfg, ausf) = Ausf::from_params("ausf", &serde_json::Value::Null).unwrap();
        tb.add(cfg, ausf).unwrap();
        let (cfg, smf) = Smf::from_params("smf", &serde_json::Value::Null).unwrap();
        tb.add(cfg, smf).unwrap();
        let (cfg, upf) = Upf::from_params("upf", &serde_json::Value::Null).unwrap();
        tb.add(cfg, upf).unwrap();
    });
    let gnb = s.connect();
    assert!(matches!(s.exchange(&gnb, &vector("ci0000")), NGAPPDU::successfulOutcome(_)));
    (s, gnb)
}

/// SUCI of imsi-208930000000001 (null scheme), all algorithms.
const REGISTRATION_REQUEST: &str = "7e004179 000d 0102f839 0000 0000 0000000010 2e04f0f0f0f0";

fn location() -> Any {
    let plmn = PLMNIdentity(FixedOctetString::new(PLMN));
    let tai = TAI::new(plmn.clone(), TAC(FixedOctetString::new([0, 0, 1])), None);
    let cgi = NRCGI::new(plmn, NRCellIdentity(FixedBitString::ZERO), None);
    wrap(&UserLocationInformation::userLocationInformationNR(UserLocationInformationNR::new(cgi, tai, None, None)))
        .unwrap()
}

fn initial_ue_message(ran_id: u32, nas: &[u8]) -> Vec<u8> {
    use AnonymousInitialUEMessageProtocolIEsCriticality::{ignore, reject};
    let ies = vec![
        AnonymousInitialUEMessageProtocolIEs::new(ID_RAN_UE_NGAP_ID.0, reject, wrap(&RANUENGAPID(ran_id)).unwrap()),
        AnonymousInitialUEMessageProtocolIEs::new(ID_NAS_PDU.0, reject, wrap(&NASPDU(nas.to_vec().into())).unwrap()),
        AnonymousInitialUEMessageProtocolIEs::new(ID_USER_LOCATION_INFORMATION.0, reject, location()),
        AnonymousInitialUEMessageProtocolIEs::new(
            ID_RRCESTABLISHMENT_CAUSE.0,
            ignore,
            wrap(&RRCEstablishmentCause::mo_Signalling).unwrap(),
        ),
    ];
    let message = wrap(&InitialUEMessage::new(InitialUEMessageProtocolIEs(ies))).unwrap();
    encode(&NGAPPDU::initiatingMessage(InitiatingMessage::new(ID_INITIAL_UEMESSAGE, Criticality::ignore, message)))
        .unwrap()
}

fn uplink_nas(amf_id: u64, ran_id: u32, nas: &[u8]) -> Vec<u8> {
    let ie = |id: ProtocolIEID, value: Any| AnonymousUplinkNASTransportProtocolIEs::new(id, Criticality::reject, value);
    let ies = vec![
        ie(ID_AMF_UE_NGAP_ID, wrap(&AMFUENGAPID(amf_id)).unwrap()),
        ie(ID_RAN_UE_NGAP_ID, wrap(&RANUENGAPID(ran_id)).unwrap()),
        ie(ID_NAS_PDU, wrap(&NASPDU(nas.to_vec().into())).unwrap()),
        ie(ID_USER_LOCATION_INFORMATION, location()),
    ];
    let message = wrap(&UplinkNASTransport::new(UplinkNASTransportProtocolIEs(ies))).unwrap();
    encode(&NGAPPDU::initiatingMessage(InitiatingMessage::new(ID_UPLINK_NASTRANSPORT, Criticality::ignore, message)))
        .unwrap()
}

fn successful<T: rasn::Encode>(code: ProcedureCode, message: &T) -> Vec<u8> {
    encode(&NGAPPDU::successfulOutcome(SuccessfulOutcome::new(code, Criticality::reject, wrap(message).unwrap())))
        .unwrap()
}

fn context_setup_response(amf_id: u64, ran_id: u32) -> Vec<u8> {
    use AnonymousInitialContextSetupResponseProtocolIEsCriticality::ignore;
    let ies = vec![
        AnonymousInitialContextSetupResponseProtocolIEs::new(
            ID_AMF_UE_NGAP_ID.0,
            ignore,
            wrap(&AMFUENGAPID(amf_id)).unwrap(),
        ),
        AnonymousInitialContextSetupResponseProtocolIEs::new(
            ID_RAN_UE_NGAP_ID.0,
            ignore,
            wrap(&RANUENGAPID(ran_id)).unwrap(),
        ),
    ];
    successful(ID_INITIAL_CONTEXT_SETUP, &InitialContextSetupResponse::new(InitialContextSetupResponseProtocolIEs(ies)))
}

/// PDU Session Resource Setup Response: the gNB's tunnel 127.0.0.1, TEID `teid`.
fn session_setup_response(amf_id: u64, ran_id: u32, psi: u8, teid: u32) -> Vec<u8> {
    use AnonymousPDUSessionResourceSetupResponseProtocolIEsCriticality::ignore;
    let tunnel = UPTransportLayerInformation::gTPTunnel(GTPTunnel::new(
        TransportLayerAddress(BitString::from_slice(&[127, 0, 0, 1])),
        GTPTEID(FixedOctetString::new(teid.to_be_bytes())),
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
    let transfer = wrap(&transfer).unwrap().as_bytes().to_vec();
    let list = PDUSessionResourceSetupListSURes(vec![PDUSessionResourceSetupItemSURes::new(
        PDUSessionID(psi),
        transfer.into(),
        None,
    )]);
    let ies = vec![
        AnonymousPDUSessionResourceSetupResponseProtocolIEs::new(
            ID_AMF_UE_NGAP_ID.0,
            ignore,
            wrap(&AMFUENGAPID(amf_id)).unwrap(),
        ),
        AnonymousPDUSessionResourceSetupResponseProtocolIEs::new(
            ID_RAN_UE_NGAP_ID.0,
            ignore,
            wrap(&RANUENGAPID(ran_id)).unwrap(),
        ),
        AnonymousPDUSessionResourceSetupResponseProtocolIEs::new(
            ID_PDUSESSION_RESOURCE_SETUP_LIST_SURES.0,
            ignore,
            wrap(&list).unwrap(),
        ),
    ];
    successful(
        ID_PDUSESSION_RESOURCE_SETUP,
        &PDUSessionResourceSetupResponse::new(PDUSessionResourceSetupResponseProtocolIEs(ies)),
    )
}

/// What the gNB got for the UE, besides its NAS PDU.
#[derive(Debug)]
enum Carrier {
    DownlinkNas,
    ContextSetup,
    /// PDU Session Resource Setup: PDU session ID and the SMF's transfer.
    SessionSetup(u8, Vec<u8>),
}

/// The UE: its keys once authenticated, and its NAS counts.
#[derive(Default)]
struct Ue {
    k_enc: [u8; 16],
    k_int: [u8; 16],
    ul: u32,
    dl: u32,
    amf_id: u64,
    stream: u16,
}

impl Ue {
    /// The next NAS PDU for the UE, checked and deciphered if protected,
    /// with what carried it.
    fn downlink(&mut self, s: &mut Setup, gnb: &SctpSocket) -> (Carrier, Vec<u8>) {
        let (stream, pdu) = s.receive(gnb);
        self.stream = stream;
        let NGAPPDU::initiatingMessage(m) = &pdu else { panic!("not an initiating message: {pdu:?}") };
        let ids = |ies: Vec<(u16, Any)>| {
            let ie = |id: ProtocolIEID| ies.iter().find(|(i, _)| *i == id.0).map(|(_, v)| v.clone());
            (open::<AMFUENGAPID>(&ie(ID_AMF_UE_NGAP_ID).unwrap()).unwrap().0, ie(ID_NAS_PDU))
        };
        let (carrier, amf_id, nas) = match m.procedure_code {
            ID_DOWNLINK_NASTRANSPORT => {
                let message: DownlinkNASTransport = open(&m.value).unwrap();
                let (amf_id, nas) = ids(message.protocol_ies.0.into_iter().map(|ie| (ie.id, ie.value)).collect());
                (Carrier::DownlinkNas, amf_id, open::<NASPDU>(&nas.unwrap()).unwrap().0.to_vec())
            }
            ID_INITIAL_CONTEXT_SETUP => {
                let message: InitialContextSetupRequest = open(&m.value).unwrap();
                let (amf_id, nas) = ids(message.protocol_ies.0.into_iter().map(|ie| (ie.id, ie.value)).collect());
                (Carrier::ContextSetup, amf_id, open::<NASPDU>(&nas.unwrap()).unwrap().0.to_vec())
            }
            ID_PDUSESSION_RESOURCE_SETUP => {
                let message: PDUSessionResourceSetupRequest = open(&m.value).unwrap();
                let ies: Vec<_> = message.protocol_ies.0.into_iter().map(|ie| (ie.id, ie.value)).collect();
                let list = ies.iter().find(|(i, _)| *i == ID_PDUSESSION_RESOURCE_SETUP_LIST_SUREQ.0).unwrap().1.clone();
                let item = open::<PDUSessionResourceSetupListSUReq>(&list).unwrap().0.remove(0);
                let (amf_id, _) = ids(ies);
                let carrier = Carrier::SessionSetup(
                    item.p_dusession_id.0,
                    item.p_dusession_resource_setup_request_transfer.to_vec(),
                );
                (carrier, amf_id, item.p_dusession_nas_pdu.unwrap().0.to_vec())
            }
            ref other => panic!("unexpected procedure {other:?}"),
        };
        self.amf_id = amf_id;
        if nas[1] == 0 {
            return (carrier, nas);
        }
        assert_eq!(nia2(&self.k_int, self.dl, 1, Direction::Downlink, &nas[6..]), nas[2..6], "downlink MAC");
        let mut plain = nas[7..].to_vec();
        if nas[1] == 2 {
            nea2(&self.k_enc, self.dl, 1, Direction::Downlink, &mut plain);
        }
        self.dl += 1;
        (carrier, plain)
    }

    fn uplink(&mut self, gnb: &SctpSocket, nas: &[u8]) {
        gnb.send(0, self.stream, NGAP_PPID, &uplink_nas(self.amf_id, 1, nas)).unwrap();
    }

    /// An initial message from idle (§4.4.6): `cleartext` IEs, the whole
    /// message ciphered in its container, integrity protected.
    fn initial(&mut self, cleartext: &[u8], whole: &[u8]) -> Vec<u8> {
        let mut container = whole.to_vec();
        nea2(&self.k_enc, self.ul, 1, Direction::Uplink, &mut container);
        let plain = [cleartext, &[0x71], &(container.len() as u16).to_be_bytes(), &container].concat();
        let mut pdu = [&[0x7e, 1, 0, 0, 0, 0, self.ul as u8][..], &plain].concat();
        let mac = nia2(&self.k_int, self.ul, 1, Direction::Uplink, &pdu[6..]);
        pdu[2..6].copy_from_slice(&mac);
        self.ul += 1;
        pdu
    }

    /// Protects (header type 2 or 4: ciphered) and sends.
    fn uplink_protected(&mut self, gnb: &SctpSocket, header: u8, plain: &[u8]) {
        let mut pdu = [&[0x7e, header, 0, 0, 0, 0, self.ul as u8][..], plain].concat();
        nea2(&self.k_enc, self.ul, 1, Direction::Uplink, &mut pdu[7..]);
        let mac = nia2(&self.k_int, self.ul, 1, Direction::Uplink, &pdu[6..]);
        pdu[2..6].copy_from_slice(&mac);
        self.ul += 1;
        self.uplink(gnb, &pdu);
    }

    /// RES* for an Authentication Request, and the NAS keys (NEA2, NIA2) for later.
    fn authenticate(&mut self, request: &[u8]) -> [u8; 16] {
        let key = |h: &str| -> [u8; 16] { hex(h).try_into().unwrap() };
        let m = Milenage::new(&key(K), &key(OPC));
        let rand: [u8; 16] = request[8..24].try_into().unwrap();
        let v = m.f2345(&rand);
        let sqn_xor_ak: [u8; 6] = request[26..32].try_into().unwrap();
        let sqn: [u8; 6] = std::array::from_fn(|i| sqn_xor_ak[i] ^ v.ak[i]);
        assert_eq!(m.f1(&rand, &sqn, &[0x80, 0]).0, request[34..42], "the network is authentic");
        let kseaf = kdf::kseaf(&kdf::kausf(&v.ck, &v.ik, SNN, &sqn_xor_ak), SNN);
        let kamf = kdf::kamf(&kseaf, "208930000000001", &[0, 0]);
        (self.k_enc, self.k_int) = kdf::nas_keys(&kamf, 2, 2);
        kdf::res_star(&v.ck, &v.ik, SNN, &rand, &v.res)
    }

    /// The whole registration; returns the Registration Accept.
    fn register(&mut self, s: &mut Setup, gnb: &SctpSocket) -> Vec<u8> {
        gnb.send(0, 1, NGAP_PPID, &initial_ue_message(1, &hex(REGISTRATION_REQUEST))).unwrap();
        let (_, request) = self.downlink(s, gnb);
        assert_eq!(request[..3], [0x7e, 0, 0x56], "Authentication Request");
        let res_star = self.authenticate(&request);
        self.uplink(gnb, &[&hex("7e0057 2d10")[..], &res_star].concat());

        let (_, command) = self.downlink(s, gnb);
        assert_eq!(command, hex("7e005d 22 00 04f0f0f0f0 e1 360102"), "Security Mode Command: NEA2, NIA2");
        let whole = hex(&format!("{REGISTRATION_REQUEST} 2f020101"));
        let complete = [&hex("7e005e 71")[..], &(whole.len() as u16).to_be_bytes(), &whole].concat();
        self.uplink_protected(gnb, 4, &complete);

        let (carrier, accept) = self.downlink(s, gnb);
        assert!(matches!(carrier, Carrier::ContextSetup), "the Accept comes in Initial Context Setup: {carrier:?}");
        gnb.send(0, self.stream, NGAP_PPID, &context_setup_response(self.amf_id, 1)).unwrap();
        self.uplink_protected(gnb, 2, &hex("7e0043"));
        s.pump();
        accept
    }
}

#[test]
fn a_ue_registers_then_deregisters() {
    let (mut s, gnb) = core();
    let mut ue = Ue::default();
    let accept = ue.register(&mut s, &gnb);
    assert_eq!(accept[..5], hex("7e0042 0101"), "Registration Accept");
    assert_eq!(accept[5..12], hex("77000b f2 02f839"), "a 5G-GUTI of this PLMN");
    assert_eq!(accept[accept.len() - 13..], hex("5407 00 02f839 000001 1502 0101"), "TAI list, allowed NSSAI");
    let registered = format!("UE {}: registered: imsi-208930000000001, 5G-GUTI 208/93 1:1:0", ue.amf_id);
    assert!(s.logged("amf-1", &registered), "{:#?}", s.log.lines());
    assert!(s.logged("amf-1", "Initial Context Setup done"));
    assert!(s.logged("ausf", "imsi-208930000000001 authenticated"));
    assert!(s.logged("udm", "authentication vector for imsi-208930000000001 (SQN 0x20)"));

    // Switch off: no Deregistration Accept, the UE is released.
    let tmsi = &accept[15..19];
    let dereg = [&hex("7e0045 09 000b f2 02f839 01 0040")[..], tmsi].concat();
    ue.uplink_protected(&gnb, 2, &dereg);
    let (_, pdu) = s.receive(&gnb);
    let _: UEContextReleaseCommand = initiating(&pdu, ID_UECONTEXT_RELEASE);
    gnb.send(0, ue.stream, NGAP_PPID, &release_complete(ue.amf_id, 1)).unwrap();
    s.pump();
    let released = format!("UE {} (imsi-208930000000001): released, 0 UE(s) left", ue.amf_id);
    assert!(s.logged("amf-1", &released), "{:#?}", s.log.lines());
}

#[test]
fn a_registered_ue_gets_a_pdu_session_kept_when_its_gnb_goes() {
    let (mut s, gnb) = core();
    let mut ue = Ue::default();
    ue.register(&mut s, &gnb);

    // UL NAS Transport: PDU Session Establishment Request, PSI 1, initial, DNN internet.
    let n1 = hex("2e 01 01 c1 ffff 91 a1");
    let transport =
        [&hex("7e0067 01")[..], &(n1.len() as u16).to_be_bytes(), &n1, &hex("1201 81 2509 08696e7465726e6574")]
            .concat();
    ue.uplink_protected(&gnb, 2, &transport);

    let (carrier, dl) = ue.downlink(&mut s, &gnb);
    let Carrier::SessionSetup(1, transfer) = carrier else { panic!("no PDU Session Resource Setup: {carrier:?}") };
    assert_eq!(dl[..5], hex("7e0068 01 00"), "DL NAS Transport of N1 SM");
    let accept = &dl[6..];
    assert_eq!(accept[..4], hex("2e0101c2"), "PDU Session Establishment Accept");
    assert!(accept.windows(5).any(|w| w == [0x01, 10, 45, 0, 2]), "address 10.45.0.2");
    let t: PDUSessionResourceSetupRequestTransfer = open(&Any::new(transfer)).unwrap();
    let tunnel = &t.protocol_ies.0.iter().find(|ie| ie.id == ID_UL_NGU_UP_TNLINFORMATION.0).unwrap().value;
    let UPTransportLayerInformation::gTPTunnel(tunnel) = open(tunnel).unwrap() else { panic!() };
    assert_eq!(tunnel.transport_layer_address.0.as_raw_slice(), [127, 0, 0, 2], "the UPF's N3");

    gnb.send(0, ue.stream, NGAP_PPID, &session_setup_response(ue.amf_id, 1, 1, 0x77)).unwrap();
    s.pump();
    assert!(s.logged("upf", "downlink to 127.0.0.1 TEID 0x77"), "{:#?}", s.log.lines());
    assert!(s.logged("amf-1", &format!("UE {}: PDU session 1 set up", ue.amf_id)));

    // Its gNB goes: the UE is idle, its session kept, the downlink off.
    drop(gnb);
    for _ in 0..20 {
        if s.logged("upf", "downlink deactivated") {
            break;
        }
        s.pump();
    }
    assert!(s.logged("amf-1", "idle (5G-TMSI"), "{:#?}", s.log.lines());
    assert!(s.logged("smf", "imsi-208930000000001: PDU session 1 deactivated (the UE is idle)"));
    assert!(!s.logged("smf", "released"), "the session stays");
}

fn release_request(amf_id: u64, ran_id: u32) -> Vec<u8> {
    let ie =
        |id: ProtocolIEID, value: Any| AnonymousUEContextReleaseRequestProtocolIEs::new(id, Criticality::reject, value);
    let ies = vec![
        ie(ID_AMF_UE_NGAP_ID, wrap(&AMFUENGAPID(amf_id)).unwrap()),
        ie(ID_RAN_UE_NGAP_ID, wrap(&RANUENGAPID(ran_id)).unwrap()),
        ie(ID_CAUSE, wrap(&Cause::radioNetwork(CauseRadioNetwork::user_inactivity)).unwrap()),
    ];
    let message = wrap(&UEContextReleaseRequest::new(UEContextReleaseRequestProtocolIEs(ies))).unwrap();
    encode(&NGAPPDU::initiatingMessage(InitiatingMessage::new(
        ID_UECONTEXT_RELEASE_REQUEST,
        Criticality::ignore,
        message,
    )))
    .unwrap()
}

#[test]
fn an_idle_ue_comes_back_and_its_session_is_reactivated() {
    let (mut s, gnb) = core();
    let mut ue = Ue::default();
    let accept = ue.register(&mut s, &gnb);
    let tmsi = accept[15..19].to_vec();
    let n1 = hex("2e 01 01 c1 ffff 91 a1");
    let transport = [&hex("7e0067 01")[..], &(n1.len() as u16).to_be_bytes(), &n1, &hex("1201 81")].concat();
    ue.uplink_protected(&gnb, 2, &transport);
    ue.downlink(&mut s, &gnb);
    gnb.send(0, ue.stream, NGAP_PPID, &session_setup_response(ue.amf_id, 1, 1, 0x77)).unwrap();
    s.pump();

    // The gNB releases the UE: it goes idle, its session's downlink off.
    let first = ue.amf_id;
    gnb.send(0, ue.stream, NGAP_PPID, &release_request(first, 1)).unwrap();
    let (_, pdu) = s.receive(&gnb);
    let _: UEContextReleaseCommand = initiating(&pdu, ID_UECONTEXT_RELEASE);
    gnb.send(0, ue.stream, NGAP_PPID, &release_complete(first, 1)).unwrap();
    s.pump();
    assert!(s.logged("amf-1", &format!("UE {first}: idle")), "{:#?}", s.log.lines());
    assert!(s.logged("upf", "downlink deactivated"));

    // Back with a Service Request (data for session 1) on a new RAN id.
    let cleartext = [&hex("7e004c 10 0007 f4 0040")[..], &tmsi].concat();
    let whole = [&cleartext[..], &hex("4002 0200 5002 0200")].concat();
    let request = ue.initial(&cleartext, &whole);
    gnb.send(0, 2, NGAP_PPID, &initial_ue_message(2, &request)).unwrap();
    let (carrier, service_accept) = ue.downlink(&mut s, &gnb);
    assert!(matches!(carrier, Carrier::ContextSetup), "{carrier:?}");
    assert_eq!(service_accept, hex("7e004e 50020200"), "Service Accept, session 1 kept");
    assert_ne!(ue.amf_id, first, "a new N2 context");
    gnb.send(0, ue.stream, NGAP_PPID, &context_setup_response(ue.amf_id, 2)).unwrap();
    let (stream, pdu) = s.receive(&gnb);
    let setup: PDUSessionResourceSetupRequest = initiating(&pdu, ID_PDUSESSION_RESOURCE_SETUP);
    let list =
        &setup.protocol_ies.0.iter().find(|ie| ie.id == ID_PDUSESSION_RESOURCE_SETUP_LIST_SUREQ.0).unwrap().value;
    let item = open::<PDUSessionResourceSetupListSUReq>(list).unwrap().0.remove(0);
    assert!(item.p_dusession_nas_pdu.is_none(), "a reactivation: no NAS");
    gnb.send(0, stream, NGAP_PPID, &session_setup_response(ue.amf_id, 2, 1, 0x88)).unwrap();
    s.pump();
    assert!(s.logged("amf-1", "resumed after"), "{:#?}", s.log.lines());
    assert!(s.logged("upf", "downlink to 127.0.0.1 TEID 0x88"), "{:#?}", s.log.lines());
}

#[test]
fn an_unanswered_challenge_is_sent_again_after_t3560() {
    let (mut s, gnb) = core();
    let mut ue = Ue::default();
    gnb.send(0, 1, NGAP_PPID, &initial_ue_message(1, &hex(REGISTRATION_REQUEST))).unwrap();
    let (_, first) = ue.downlink(&mut s, &gnb);
    s.tb.advance(std::time::Duration::from_secs(6));
    let (_, again) = ue.downlink(&mut s, &gnb);
    assert_eq!(again, first, "the same Authentication Request");
    assert!(s.logged("amf-1", "T3560 expired: Authentication Request again (1/4)"));
    let res_star = ue.authenticate(&again);
    ue.uplink(&gnb, &[&hex("7e0057 2d10")[..], &res_star].concat());
    let (_, command) = ue.downlink(&mut s, &gnb);
    assert_eq!(command[..3], hex("7e005d")[..], "the registration goes on");
}

#[test]
fn an_unknown_subscriber_is_rejected() {
    let (mut s, gnb) = core();
    let mut ue = Ue::default();
    let stranger = REGISTRATION_REQUEST.replace("0000000010", "0000000020");
    gnb.send(0, 1, NGAP_PPID, &initial_ue_message(1, &hex(&stranger))).unwrap();
    assert_eq!(ue.downlink(&mut s, &gnb).1, hex("7e004403"), "Registration Reject, illegal UE");
    let (_, pdu) = s.receive(&gnb);
    let _: UEContextReleaseCommand = initiating(&pdu, ID_UECONTEXT_RELEASE);
    assert!(s.logged("udm", "no authentication vector for suci-0-208-93-0000-0-0-0000000002: UserNotFound"));
}
