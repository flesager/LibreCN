//! Registration end to end: a test gNB on SCTP, the AMF, AUSF and UDM EOs on
//! the test bed, and a UE computing its side of 5G AKA and NAS security.

mod common;

use common::{Setup, amf_with, initiating, release_complete, vector};
use librecn_amf::sctp::{NGAP_PPID, SctpSocket};
use librecn_ausf::Ausf;
use librecn_ngap::*;
use librecn_security::milenage::Milenage;
use librecn_security::{Direction, kdf, nea2, nia2};
use librecn_udm::Udm;
use rasn::types::{FixedBitString, FixedOctetString};

const K: &str = "465B5CE8B199B49FAA5F0A2EE238A6BC";
const OPC: &str = "E8ED289DEBA952E4283B54E88E6183CA";
/// free5GC's test gNB (vector ci0000) serves PLMN 208/93.
const SNN: &str = "5G:mnc093.mcc208.3gppnetwork.org";
const PLMN: [u8; 3] = [0x02, 0xf8, 0x39];

fn hex(s: &str) -> Vec<u8> {
    let s: String = s.split_whitespace().collect();
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

/// Core with the UDM knowing `imsi-208930000000001`, and a gNB set up.
fn core() -> (Setup, SctpSocket) {
    let mut s = amf_with("208", "93", |tb| {
        let subscriber = serde_json::json!({ "supi": "imsi-208930000000001", "k": K, "opc": OPC });
        let (cfg, udm) = Udm::from_params("udm", &serde_json::json!({ "subscribers": [subscriber] })).unwrap();
        tb.add(cfg, udm).unwrap();
        let (cfg, ausf) = Ausf::from_params("ausf", &serde_json::Value::Null).unwrap();
        tb.add(cfg, ausf).unwrap();
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
    /// The next Downlink NAS Transport's NAS PDU, checked and deciphered if protected.
    fn downlink(&mut self, s: &mut Setup, gnb: &SctpSocket) -> Vec<u8> {
        let (stream, pdu) = s.receive(gnb);
        let message: DownlinkNASTransport = initiating(&pdu, ID_DOWNLINK_NASTRANSPORT);
        let ie = |id: ProtocolIEID| &message.protocol_ies.0.iter().find(|ie| ie.id == id.0).unwrap().value;
        (self.amf_id, self.stream) = (open::<AMFUENGAPID>(ie(ID_AMF_UE_NGAP_ID)).unwrap().0, stream);
        let nas = open::<NASPDU>(ie(ID_NAS_PDU)).unwrap().0.to_vec();
        if nas[1] == 0 {
            return nas;
        }
        assert_eq!(nia2(&self.k_int, self.dl, 1, Direction::Downlink, &nas[6..]), nas[2..6], "downlink MAC");
        let mut plain = nas[7..].to_vec();
        if nas[1] == 2 {
            nea2(&self.k_enc, self.dl, 1, Direction::Downlink, &mut plain);
        }
        self.dl += 1;
        plain
    }

    fn uplink(&mut self, gnb: &SctpSocket, nas: &[u8]) {
        gnb.send(0, self.stream, NGAP_PPID, &uplink_nas(self.amf_id, 1, nas)).unwrap();
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
}

#[test]
fn a_ue_registers_then_deregisters() {
    let (mut s, gnb) = core();
    let mut ue = Ue::default();
    gnb.send(0, 1, NGAP_PPID, &initial_ue_message(1, &hex(REGISTRATION_REQUEST))).unwrap();

    let request = ue.downlink(&mut s, &gnb);
    assert_eq!(request[..3], [0x7e, 0, 0x56], "Authentication Request");
    let res_star = ue.authenticate(&request);
    ue.uplink(&gnb, &[&hex("7e0057 2d10")[..], &res_star].concat());

    let command = ue.downlink(&mut s, &gnb);
    assert_eq!(command, hex("7e005d 22 00 04f0f0f0f0 e1 360102"), "Security Mode Command: NEA2, NIA2");
    let whole = hex(&format!("{REGISTRATION_REQUEST} 2f020101"));
    let complete = [&hex("7e005e 71")[..], &(whole.len() as u16).to_be_bytes(), &whole].concat();
    ue.uplink_protected(&gnb, 4, &complete);

    let accept = ue.downlink(&mut s, &gnb);
    assert_eq!(accept[..5], hex("7e0042 0101"), "Registration Accept");
    assert_eq!(accept[5..12], hex("77000b f2 02f839"), "a 5G-GUTI of this PLMN");
    assert_eq!(accept[accept.len() - 13..], hex("5407 00 02f839 000001 1502 0101"), "TAI list, allowed NSSAI");
    ue.uplink_protected(&gnb, 2, &hex("7e0043"));
    s.pump();
    let registered = format!("UE {}: registered: imsi-208930000000001, 5G-GUTI 208/93 1:1:0", ue.amf_id);
    assert!(s.logged("amf-1", &registered), "{:#?}", s.log.lines());
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
fn an_unknown_subscriber_is_rejected() {
    let (mut s, gnb) = core();
    let mut ue = Ue::default();
    let stranger = REGISTRATION_REQUEST.replace("0000000010", "0000000020");
    gnb.send(0, 1, NGAP_PPID, &initial_ue_message(1, &hex(&stranger))).unwrap();
    assert_eq!(ue.downlink(&mut s, &gnb), hex("7e004403"), "Registration Reject, illegal UE");
    let (_, pdu) = s.receive(&gnb);
    let _: UEContextReleaseCommand = initiating(&pdu, ID_UECONTEXT_RELEASE);
    assert!(s.logged("udm", "no authentication vector for suci-0-208-93-0000-0-0-0000000002: UserNotFound"));
}
