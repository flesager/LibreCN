//! NGAP ↔ domain values. The only module that sees ASN.1 types.

use librecn_ngap::*;
use rasn::types::{FixedBitString, FixedOctetString, PrintableString};

use crate::config::{AmfConfig, Plmn, Snssai};
use crate::setup::{RanNodeId, Refusal, SetupRequest, SupportedTa};

/// An NGAP PDU from a gNB, as far as this AMF understands it.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Incoming {
    NgSetup(SetupRequest),
    NgReset,
    /// The gNB reports an error; its cause, for the log.
    ErrorIndication(String),
    /// A procedure this AMF does not handle, with the criticality the
    /// sender gave it (TS 38.413 §10.3: decides whether to answer).
    Unsupported {
        procedure: u8,
        criticality: Criticality,
    },
    /// Not a valid NGAP PDU, or a mandatory IE missing.
    Malformed(String),
}

/// Decodes a received PDU; never panics.
pub(crate) fn decode(bytes: &[u8]) -> Incoming {
    let pdu = match librecn_ngap::decode(bytes) {
        Ok(pdu) => pdu,
        Err(e) => return Incoming::Malformed(e.to_string()),
    };
    let NGAPPDU::initiatingMessage(message) = pdu else {
        let procedure = procedure_code(&pdu).unwrap_or(0);
        return Incoming::Unsupported { procedure, criticality: Criticality::ignore };
    };
    let result = match message.procedure_code {
        ID_NGSETUP => open::<NGSetupRequest>(&message.value).map_err(|e| e.to_string()).and_then(|m| setup_request(&m)),
        ID_NGRESET => open::<NGReset>(&message.value).map(|_| Incoming::NgReset).map_err(|e| e.to_string()),
        ID_ERROR_INDICATION => open::<ErrorIndication>(&message.value).map_err(|e| e.to_string()).map(|m| {
            let cause =
                m.protocol_ies.0.iter().find(|ie| ie.id == ID_CAUSE.0).and_then(|ie| open::<Cause>(&ie.value).ok());
            Incoming::ErrorIndication(cause.map_or_else(|| "no cause".into(), |c| format!("{c:?}")))
        }),
        code => Ok(Incoming::Unsupported { procedure: code.0, criticality: message.criticality }),
    };
    result.unwrap_or_else(Incoming::Malformed)
}

fn setup_request(message: &NGSetupRequest) -> Result<Incoming, String> {
    let ie = |id: ProtocolIEID| message.protocol_ies.0.iter().find(|ie| ie.id == id.0).map(|ie| &ie.value);
    let mandatory = |id: ProtocolIEID, name: &str| ie(id).ok_or_else(|| format!("NG Setup Request without {name}"));
    let decode_err = |e: DecodeError| e.to_string();

    let node =
        match open::<GlobalRANNodeID>(mandatory(ID_GLOBAL_RANNODE_ID, "Global RAN Node ID")?).map_err(decode_err)? {
            GlobalRANNodeID::globalGNB_ID(gnb) => match &gnb.g_nb_id {
                GNBID::gNB_ID(bits) => RanNodeId::Gnb {
                    plmn: Plmn(*gnb.p_lmnidentity.0),
                    id: bits.iter().fold(0u32, |acc, bit| acc << 1 | u32::from(*bit)),
                    bits: bits.len() as u8,
                },
                _ => RanNodeId::Other,
            },
            _ => RanNodeId::Other,
        };
    let name = ie(ID_RANNODE_NAME)
        .map(|v| open::<RANNodeName>(v).map(|n| String::from_utf8_lossy(n.0.as_bytes()).into_owned()))
        .transpose()
        .map_err(decode_err)?;
    let list: SupportedTAList = open(mandatory(ID_SUPPORTED_TALIST, "Supported TA List")?).map_err(decode_err)?;
    let tas = list
        .0
        .iter()
        .map(|ta| SupportedTa {
            tac: *ta.t_ac.0,
            plmns: ta
                .broadcast_plmnlist
                .0
                .iter()
                .map(|b| {
                    let slices = b
                        .t_aislice_support_list
                        .0
                        .iter()
                        .map(|s| Snssai { sst: s.s_nssai.s_st.0[0], sd: s.s_nssai.s_d.as_ref().map(|sd| *sd.0) })
                        .collect();
                    (Plmn(*b.p_lmnidentity.0), slices)
                })
                .collect(),
        })
        .collect();
    Ok(Incoming::NgSetup(SetupRequest { node, name, tas }))
}

/// `value` in the first `N` bits, most significant first.
fn bits<const N: usize>(value: u32) -> FixedBitString<N> {
    let mut b = FixedBitString::<N>::ZERO;
    for i in 0..N {
        b.set(i, (value >> (N - 1 - i)) & 1 == 1);
    }
    b
}

fn plmn(p: Plmn) -> PLMNIdentity {
    PLMNIdentity(FixedOctetString::new(p.0))
}

fn encoding_error(e: EncodeError) -> String {
    format!("encoding: {e}")
}

fn pdu(pdu: NGAPPDU) -> Result<Vec<u8>, String> {
    encode(&pdu).map_err(encoding_error)
}

/// NG SETUP RESPONSE announcing `config`.
pub(crate) fn ng_setup_response(config: &AmfConfig) -> Result<Vec<u8>, String> {
    use AnonymousNGSetupResponseProtocolIEsCriticality as C;
    let name = AMFName(PrintableString::from_bytes(config.name.as_bytes()).map_err(|e| format!("AMF name: {e}"))?);
    let guami = GUAMI::new(
        plmn(config.plmn),
        AMFRegionID(bits(u32::from(config.amf_region))),
        AMFSetID(bits(u32::from(config.amf_set))),
        AMFPointer(bits(u32::from(config.amf_pointer))),
        None,
    );
    let slices = config
        .slices
        .iter()
        .map(|s| {
            let sd = s.sd.map(|sd| SD(FixedOctetString::new(sd)));
            SliceSupportItem::new(SNSSAI::new(SST(FixedOctetString::new([s.sst])), sd, None), None)
        })
        .collect();
    let ie = |id: ProtocolIEID, criticality, value: Result<Any, EncodeError>| {
        value.map(|value| AnonymousNGSetupResponseProtocolIEs::new(id.0, criticality, value)).map_err(encoding_error)
    };
    let ies = vec![
        ie(ID_AMFNAME, C::reject, wrap(&name))?,
        ie(ID_SERVED_GUAMILIST, C::reject, wrap(&ServedGUAMIList(vec![ServedGUAMIItem::new(guami, None, None)])))?,
        ie(ID_RELATIVE_AMFCAPACITY, C::ignore, wrap(&RelativeAMFCapacity(config.relative_capacity)))?,
        ie(
            ID_PLMNSUPPORT_LIST,
            C::reject,
            wrap(&PLMNSupportList(vec![PLMNSupportItem::new(plmn(config.plmn), SliceSupportList(slices), None)])),
        )?,
    ];
    let message = wrap(&NGSetupResponse::new(NGSetupResponseProtocolIEs(ies))).map_err(encoding_error)?;
    pdu(NGAPPDU::successfulOutcome(SuccessfulOutcome::new(ID_NGSETUP, Criticality::reject, message)))
}

/// NG SETUP FAILURE.
pub(crate) fn ng_setup_failure(refusal: Refusal) -> Result<Vec<u8>, String> {
    let cause = match refusal {
        Refusal::UnknownPlmn => Cause::misc(CauseMisc::unknown_PLMN),
    };
    let ies = vec![AnonymousNGSetupFailureProtocolIEs::new(
        ID_CAUSE.0,
        AnonymousNGSetupFailureProtocolIEsCriticality::ignore,
        wrap(&cause).map_err(encoding_error)?,
    )];
    let message = wrap(&NGSetupFailure::new(NGSetupFailureProtocolIEs(ies))).map_err(encoding_error)?;
    pdu(NGAPPDU::unsuccessfulOutcome(UnsuccessfulOutcome::new(ID_NGSETUP, Criticality::reject, message)))
}

/// NG RESET ACKNOWLEDGE (whole interface: no UE list).
pub(crate) fn ng_reset_acknowledge() -> Result<Vec<u8>, String> {
    let message = wrap(&NGResetAcknowledge::new(NGResetAcknowledgeProtocolIEs(vec![]))).map_err(encoding_error)?;
    pdu(NGAPPDU::successfulOutcome(SuccessfulOutcome::new(ID_NGRESET, Criticality::reject, message)))
}

/// ERROR INDICATION with a protocol cause.
pub(crate) fn error_indication(cause: CauseProtocol) -> Result<Vec<u8>, String> {
    let ies = vec![AnonymousErrorIndicationProtocolIEs::new(
        ID_CAUSE.0,
        AnonymousErrorIndicationProtocolIEsCriticality::ignore,
        wrap(&Cause::protocol(cause)).map_err(encoding_error)?,
    )];
    let message = wrap(&ErrorIndication::new(ErrorIndicationProtocolIEs(ies))).map_err(encoding_error)?;
    pdu(NGAPPDU::initiatingMessage(InitiatingMessage::new(ID_ERROR_INDICATION, Criticality::ignore, message)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vector(name: &str) -> Vec<u8> {
        std::fs::read(format!("{}/../../proto/ngap/tests/vectors/free5gc/{name}.bin", env!("CARGO_MANIFEST_DIR")))
            .unwrap()
    }

    #[test]
    fn free5gc_ng_setup_request_becomes_domain_values() {
        let Incoming::NgSetup(request) = decode(&vector("ci0000")) else { panic!("not an NG Setup") };
        let plmn = Plmn::new("208", "93").unwrap();
        assert_eq!(request.node, RanNodeId::Gnb { plmn, id: 0x000102, bits: 24 });
        assert_eq!(request.name.as_deref(), Some("free5gc"));
        assert_eq!(request.tas.len(), 1);
        assert_eq!(request.tas[0].tac, [0, 0, 1]);
        assert_eq!(request.tas[0].plmns, [(plmn, vec![Snssai { sst: 1, sd: Some([1, 2, 3]) }])]);
    }

    #[test]
    fn response_carries_the_configured_identity() {
        let params = serde_json::json!({ "slices": [{ "sst": 1 }, { "sst": 2, "sd": "abcdef" }] });
        let config = AmfConfig::from_params("amf-1", &params).unwrap();
        let bytes = ng_setup_response(&config).unwrap();
        let NGAPPDU::successfulOutcome(outcome) = librecn_ngap::decode(&bytes).unwrap() else { panic!() };
        assert_eq!(outcome.procedure_code, ID_NGSETUP);
        assert_eq!(check(&librecn_ngap::decode(&bytes).unwrap()).unwrap(), 5, "message + 4 IEs round-trip");
        let response: NGSetupResponse = open(&outcome.value).unwrap();
        let ie = |id: ProtocolIEID| &response.protocol_ies.0.iter().find(|ie| ie.id == id.0).unwrap().value;
        assert_eq!(open::<AMFName>(ie(ID_AMFNAME)).unwrap().0.as_bytes(), b"amf-1");
        let guami = &open::<ServedGUAMIList>(ie(ID_SERVED_GUAMILIST)).unwrap().0[0].g_uami;
        assert_eq!(*guami.p_lmnidentity.0, [0x99, 0xf9, 0x07]);
        assert_eq!(guami.a_mfregion_id.0[..8].iter().fold(0u8, |a, b| a << 1 | u8::from(*b)), 1);
        assert_eq!(guami.a_mfset_id.0[..10].iter().fold(0u16, |a, b| a << 1 | u16::from(*b)), 1);
        assert_eq!(open::<RelativeAMFCapacity>(ie(ID_RELATIVE_AMFCAPACITY)).unwrap().0, 255);
        let support = &open::<PLMNSupportList>(ie(ID_PLMNSUPPORT_LIST)).unwrap().0[0];
        let slices: Vec<_> = support
            .slice_support_list
            .0
            .iter()
            .map(|s| (s.s_nssai.s_st.0[0], s.s_nssai.s_d.as_ref().map(|d| *d.0)))
            .collect();
        assert_eq!(slices, [(1, None), (2, Some([0xab, 0xcd, 0xef]))]);
    }

    #[test]
    fn failure_reset_ack_and_error_indication_are_valid_ngap() {
        let failure = librecn_ngap::decode(&ng_setup_failure(Refusal::UnknownPlmn).unwrap()).unwrap();
        let NGAPPDU::unsuccessfulOutcome(f) = &failure else { panic!() };
        let f: NGSetupFailure = open(&f.value).unwrap();
        assert_eq!(open::<Cause>(&f.protocol_ies.0[0].value).unwrap(), Cause::misc(CauseMisc::unknown_PLMN));
        assert_eq!(check(&failure).unwrap(), 2);

        let ack = librecn_ngap::decode(&ng_reset_acknowledge().unwrap()).unwrap();
        assert_eq!(procedure_code(&ack), Some(ID_NGRESET.0));
        let error = error_indication(CauseProtocol::transfer_syntax_error).unwrap();
        assert!(matches!(decode(&error), Incoming::ErrorIndication(cause) if cause.contains("transfer_syntax_error")));
    }

    #[test]
    fn garbage_and_unhandled_procedures_are_classified() {
        assert!(matches!(decode(&[0xff, 0x00]), Incoming::Malformed(_)));
        // ci0832: Initial UE Message, not handled at this step.
        assert!(matches!(decode(&vector("ci0832")), Incoming::Unsupported { procedure: 15, .. }));
        assert_eq!(decode(&vector("ci0709")), Incoming::NgReset);
    }
}
