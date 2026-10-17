//! NGAP ↔ domain values. The only module that sees ASN.1 types.

use librecn_ngap::*;
use rasn::types::{FixedBitString, FixedOctetString, PrintableString};

use crate::config::{AmfConfig, Plmn, Snssai};
use crate::mm::AsContext;
use crate::setup::{RanNodeId, Refusal, SetupRequest, SupportedTa};
use crate::ue::{AmfUeId, Location, RanUeId};

/// An NGAP PDU from a gNB, as far as this AMF understands it.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Incoming {
    NgSetup(SetupRequest),
    NgReset,
    /// The gNB reports an error; its cause, for the log.
    ErrorIndication(String),
    /// A UE's first NAS message.
    InitialUe {
        ran_id: RanUeId,
        nas: Vec<u8>,
        location: Option<Location>,
    },
    UplinkNas {
        amf_id: AmfUeId,
        ran_id: RanUeId,
        nas: Vec<u8>,
        location: Option<Location>,
    },
    /// The gNB asks to release a UE; its cause, for the log.
    ReleaseRequest {
        amf_id: AmfUeId,
        ran_id: RanUeId,
        cause: String,
    },
    /// UE Context Release Complete.
    ReleaseComplete {
        amf_id: AmfUeId,
        ran_id: RanUeId,
    },
    /// Initial Context Setup Response, or Failure with its cause.
    ContextSetup {
        amf_id: AmfUeId,
        ran_id: RanUeId,
        failure: Option<String>,
    },
    /// PDU Session Resource Setup Response: transfers of the sessions set
    /// up, and of those that failed, by PDU session ID.
    SessionSetup {
        amf_id: AmfUeId,
        ran_id: RanUeId,
        setup: Vec<(u8, Vec<u8>)>,
        failed: Vec<(u8, Vec<u8>)>,
    },
    /// PDU Session Resource Release Response: the PDU session IDs released.
    SessionRelease {
        amf_id: AmfUeId,
        ran_id: RanUeId,
        released: Vec<u8>,
    },
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
    let message = match pdu {
        NGAPPDU::initiatingMessage(message) => message,
        NGAPPDU::successfulOutcome(outcome) if outcome.procedure_code == ID_UECONTEXT_RELEASE => {
            return open::<UEContextReleaseComplete>(&outcome.value)
                .map_err(|e| e.to_string())
                .and_then(|m| {
                    let ies = &m.protocol_ies.0;
                    Ok(Incoming::ReleaseComplete { amf_id: amf_id(ies)?, ran_id: ran_id(ies)? })
                })
                .unwrap_or_else(Incoming::Malformed);
        }
        NGAPPDU::successfulOutcome(outcome) if outcome.procedure_code == ID_INITIAL_CONTEXT_SETUP => {
            return open::<InitialContextSetupResponse>(&outcome.value)
                .map_err(|e| e.to_string())
                .and_then(|m| {
                    let ies = &m.protocol_ies.0;
                    Ok(Incoming::ContextSetup { amf_id: amf_id(ies)?, ran_id: ran_id(ies)?, failure: None })
                })
                .unwrap_or_else(Incoming::Malformed);
        }
        NGAPPDU::unsuccessfulOutcome(outcome) if outcome.procedure_code == ID_INITIAL_CONTEXT_SETUP => {
            return open::<InitialContextSetupFailure>(&outcome.value)
                .map_err(|e| e.to_string())
                .and_then(|m| {
                    let ies = &m.protocol_ies.0;
                    let cause = find(ies, ID_CAUSE).and_then(|v| open::<Cause>(v).ok());
                    let failure = Some(cause.map_or_else(|| "no cause".into(), |c| format!("{c:?}")));
                    Ok(Incoming::ContextSetup { amf_id: amf_id(ies)?, ran_id: ran_id(ies)?, failure })
                })
                .unwrap_or_else(Incoming::Malformed);
        }
        NGAPPDU::successfulOutcome(outcome) if outcome.procedure_code == ID_PDUSESSION_RESOURCE_SETUP => {
            return open::<PDUSessionResourceSetupResponse>(&outcome.value)
                .map_err(|e| e.to_string())
                .and_then(|m| {
                    let ies = &m.protocol_ies.0;
                    let setup = match find(ies, ID_PDUSESSION_RESOURCE_SETUP_LIST_SURES) {
                        Some(v) => open::<PDUSessionResourceSetupListSURes>(v)
                            .map_err(|e| format!("setup list: {e}"))?
                            .0
                            .into_iter()
                            .map(|i| (i.p_dusession_id.0, i.p_dusession_resource_setup_response_transfer.to_vec()))
                            .collect(),
                        None => vec![],
                    };
                    let failed = match find(ies, ID_PDUSESSION_RESOURCE_FAILED_TO_SETUP_LIST_SURES) {
                        Some(v) => open::<PDUSessionResourceFailedToSetupListSURes>(v)
                            .map_err(|e| format!("failed list: {e}"))?
                            .0
                            .into_iter()
                            .map(|i| (i.p_dusession_id.0, i.p_dusession_resource_setup_unsuccessful_transfer.to_vec()))
                            .collect(),
                        None => vec![],
                    };
                    Ok(Incoming::SessionSetup { amf_id: amf_id(ies)?, ran_id: ran_id(ies)?, setup, failed })
                })
                .unwrap_or_else(Incoming::Malformed);
        }
        NGAPPDU::successfulOutcome(outcome) if outcome.procedure_code == ID_PDUSESSION_RESOURCE_RELEASE => {
            return open::<PDUSessionResourceReleaseResponse>(&outcome.value)
                .map_err(|e| e.to_string())
                .and_then(|m| {
                    let ies = &m.protocol_ies.0;
                    let released = match find(ies, ID_PDUSESSION_RESOURCE_RELEASED_LIST_REL_RES) {
                        Some(v) => open::<PDUSessionResourceReleasedListRelRes>(v)
                            .map_err(|e| format!("released list: {e}"))?
                            .0
                            .into_iter()
                            .map(|i| i.p_dusession_id.0)
                            .collect(),
                        None => vec![],
                    };
                    Ok(Incoming::SessionRelease { amf_id: amf_id(ies)?, ran_id: ran_id(ies)?, released })
                })
                .unwrap_or_else(Incoming::Malformed);
        }
        other => {
            let procedure = procedure_code(&other).unwrap_or(0);
            return Incoming::Unsupported { procedure, criticality: Criticality::ignore };
        }
    };
    let result = match message.procedure_code {
        ID_INITIAL_UEMESSAGE => open::<InitialUEMessage>(&message.value).map_err(|e| e.to_string()).and_then(|m| {
            let ies = &m.protocol_ies.0;
            Ok(Incoming::InitialUe { ran_id: ran_id(ies)?, nas: nas_pdu(ies)?, location: location(ies) })
        }),
        ID_UPLINK_NASTRANSPORT => open::<UplinkNASTransport>(&message.value).map_err(|e| e.to_string()).and_then(|m| {
            let ies = &m.protocol_ies.0;
            Ok(Incoming::UplinkNas {
                amf_id: amf_id(ies)?,
                ran_id: ran_id(ies)?,
                nas: nas_pdu(ies)?,
                location: location(ies),
            })
        }),
        ID_UECONTEXT_RELEASE_REQUEST => {
            open::<UEContextReleaseRequest>(&message.value).map_err(|e| e.to_string()).and_then(|m| {
                let ies = &m.protocol_ies.0;
                let cause = find(ies, ID_CAUSE).and_then(|v| open::<Cause>(v).ok());
                Ok(Incoming::ReleaseRequest {
                    amf_id: amf_id(ies)?,
                    ran_id: ran_id(ies)?,
                    cause: cause.map_or_else(|| "no cause".into(), |c| format!("{c:?}")),
                })
            })
        }
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

/// A protocol IE container entry, whichever shape the generator gave it.
trait Ie {
    fn id(&self) -> u16;
    fn value(&self) -> &Any;
}

macro_rules! ie {
    ($($entry:ty: $id:expr;)*) => {$(
        impl Ie for $entry {
            fn id(&self) -> u16 {
                $id(self)
            }
            fn value(&self) -> &Any {
                &self.value
            }
        }
    )*};
}

ie! {
    AnonymousInitialUEMessageProtocolIEs: |e: &Self| e.id;
    AnonymousUplinkNASTransportProtocolIEs: |e: &Self| e.id.0;
    AnonymousUEContextReleaseRequestProtocolIEs: |e: &Self| e.id.0;
    AnonymousUEContextReleaseCompleteProtocolIEs: |e: &Self| e.id.0;
    AnonymousInitialContextSetupResponseProtocolIEs: |e: &Self| e.id;
    AnonymousInitialContextSetupFailureProtocolIEs: |e: &Self| e.id;
    AnonymousPDUSessionResourceSetupResponseProtocolIEs: |e: &Self| e.id;
    AnonymousPDUSessionResourceReleaseResponseProtocolIEs: |e: &Self| e.id;
}

fn find<E: Ie>(ies: &[E], id: ProtocolIEID) -> Option<&Any> {
    ies.iter().find(|ie| ie.id() == id.0).map(Ie::value)
}

fn mandatory<T: rasn::Decode, E: Ie>(ies: &[E], id: ProtocolIEID, name: &str) -> Result<T, String> {
    open(find(ies, id).ok_or_else(|| format!("{name} missing"))?).map_err(|e| format!("{name}: {e}"))
}

fn amf_id<E: Ie>(ies: &[E]) -> Result<AmfUeId, String> {
    mandatory::<AMFUENGAPID, _>(ies, ID_AMF_UE_NGAP_ID, "AMF-UE-NGAP-ID").map(|id| id.0)
}

fn ran_id<E: Ie>(ies: &[E]) -> Result<RanUeId, String> {
    mandatory::<RANUENGAPID, _>(ies, ID_RAN_UE_NGAP_ID, "RAN-UE-NGAP-ID").map(|id| id.0)
}

fn nas_pdu<E: Ie>(ies: &[E]) -> Result<Vec<u8>, String> {
    mandatory::<NASPDU, _>(ies, ID_NAS_PDU, "NAS-PDU").map(|nas| nas.0.to_vec())
}

/// NR location only: E-UTRA (ng-eNB) is not served yet.
fn location<E: Ie>(ies: &[E]) -> Option<Location> {
    let UserLocationInformation::userLocationInformationNR(nr) = open(find(ies, ID_USER_LOCATION_INFORMATION)?).ok()?
    else {
        return None;
    };
    let cell = nr.n_r_cgi.n_rcell_identity.0[..36].iter().fold(0u64, |acc, bit| acc << 1 | u64::from(*bit));
    Some(Location {
        tai: (Plmn(*nr.t_ai.p_lmnidentity.0), *nr.t_ai.t_ac.0),
        cell: (Plmn(*nr.n_r_cgi.p_lmnidentity.0), cell),
    })
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

/// PAGING of an idle UE by its 5G-S-TMSI, in tracking area `tai`.
pub(crate) fn paging(config: &AmfConfig, tmsi: u32, tai: (Plmn, [u8; 3])) -> Result<Vec<u8>, String> {
    use AnonymousPagingProtocolIEsCriticality::ignore;
    let identity = UEPagingIdentity::fiveG_S_TMSI(FiveGSTMSI::new(
        AMFSetID(bits(u32::from(config.amf_set))),
        AMFPointer(bits(u32::from(config.amf_pointer))),
        FiveGTMSI(FixedOctetString::new(tmsi.to_be_bytes())),
        None,
    ));
    let tai = TAI::new(plmn(tai.0), TAC(FixedOctetString::new(tai.1)), None);
    let tais = TAIListForPaging(vec![TAIListForPagingItem::new(tai, None)]);
    let ie = |id: ProtocolIEID, value: Result<Any, EncodeError>| {
        value.map(|v| AnonymousPagingProtocolIEs::new(id.0, ignore, v)).map_err(encoding_error)
    };
    let ies = vec![ie(ID_UEPAGING_IDENTITY, wrap(&identity))?, ie(ID_TAILIST_FOR_PAGING, wrap(&tais))?];
    let message = wrap(&Paging::new(PagingProtocolIEs(ies))).map_err(encoding_error)?;
    pdu(NGAPPDU::initiatingMessage(InitiatingMessage::new(ID_PAGING, Criticality::ignore, message)))
}

/// DOWNLINK NAS TRANSPORT of `nas` to a UE.
pub(crate) fn downlink_nas_transport(amf_id: AmfUeId, ran_id: RanUeId, nas: &[u8]) -> Result<Vec<u8>, String> {
    use AnonymousDownlinkNASTransportProtocolIEsCriticality as C;
    let ie = |id: ProtocolIEID, value: Result<Any, EncodeError>| {
        value.map(|v| AnonymousDownlinkNASTransportProtocolIEs::new(id.0, C::reject, v)).map_err(encoding_error)
    };
    let ies = vec![
        ie(ID_AMF_UE_NGAP_ID, wrap(&AMFUENGAPID(amf_id)))?,
        ie(ID_RAN_UE_NGAP_ID, wrap(&RANUENGAPID(ran_id)))?,
        ie(ID_NAS_PDU, wrap(&NASPDU(nas.into())))?,
    ];
    let message = wrap(&DownlinkNASTransport::new(DownlinkNASTransportProtocolIEs(ies))).map_err(encoding_error)?;
    pdu(NGAPPDU::initiatingMessage(InitiatingMessage::new(ID_DOWNLINK_NASTRANSPORT, Criticality::ignore, message)))
}

fn snssai(s: Snssai) -> SNSSAI {
    SNSSAI::new(SST(FixedOctetString::new([s.sst])), s.sd.map(|sd| SD(FixedOctetString::new(sd))), None)
}

/// The NR (or E-UTRA) algorithms of a NAS capability octet as NGAP's
/// 16-bit string: NAS bits 128-EA1/IA1, EA2/IA2, EA3/IA3 (0x40, 0x20, 0x10)
/// become the string's first three bits; EA0/IA0 has no bit.
fn algorithms(nas_octet: u8) -> rasn::types::BitString {
    rasn::types::BitString::from_slice(&[(nas_octet & 0x70) << 1, 0])
}

/// INITIAL CONTEXT SETUP REQUEST carrying `nas`, no PDU session.
pub(crate) fn initial_context_setup_request(
    amf_id: AmfUeId,
    ran_id: RanUeId,
    config: &AmfConfig,
    context: &AsContext,
    nas: &[u8],
) -> Result<Vec<u8>, String> {
    use AnonymousInitialContextSetupRequestProtocolIEsCriticality::{ignore, reject};
    let guami = GUAMI::new(
        plmn(config.plmn),
        AMFRegionID(bits(u32::from(config.amf_region))),
        AMFSetID(bits(u32::from(config.amf_set))),
        AMFPointer(bits(u32::from(config.amf_pointer))),
        None,
    );
    let allowed = AllowedNSSAI(context.allowed.iter().map(|s| AllowedNSSAIItem::new(snssai(s), None)).collect());
    let caps = context.capability.as_bytes();
    let octet = |i: usize| caps.get(i).copied().unwrap_or(0);
    let security = UESecurityCapabilities::new(
        NRencryptionAlgorithms(algorithms(octet(0))),
        NRintegrityProtectionAlgorithms(algorithms(octet(1))),
        EUTRAencryptionAlgorithms(algorithms(octet(2))),
        EUTRAintegrityProtectionAlgorithms(algorithms(octet(3))),
        None,
    );
    let mut key = FixedBitString::<256>::ZERO;
    for (i, byte) in context.kgnb.iter().enumerate() {
        for bit in 0..8 {
            key.set(i * 8 + bit, byte >> (7 - bit) & 1 == 1);
        }
    }
    let key = SecurityKey(key);
    let ie = |id: ProtocolIEID, criticality, value: Result<Any, EncodeError>| {
        value.map(|v| AnonymousInitialContextSetupRequestProtocolIEs::new(id.0, criticality, v)).map_err(encoding_error)
    };
    let ies = vec![
        ie(ID_AMF_UE_NGAP_ID, reject, wrap(&AMFUENGAPID(amf_id)))?,
        ie(ID_RAN_UE_NGAP_ID, reject, wrap(&RANUENGAPID(ran_id)))?,
        ie(ID_GUAMI, reject, wrap(&guami))?,
        ie(ID_ALLOWED_NSSAI, reject, wrap(&allowed))?,
        ie(ID_UESECURITY_CAPABILITIES, reject, wrap(&security))?,
        ie(ID_SECURITY_KEY, reject, wrap(&key))?,
        ie(ID_NAS_PDU, ignore, wrap(&NASPDU(nas.into())))?,
    ];
    let message =
        wrap(&InitialContextSetupRequest::new(InitialContextSetupRequestProtocolIEs(ies))).map_err(encoding_error)?;
    pdu(NGAPPDU::initiatingMessage(InitiatingMessage::new(ID_INITIAL_CONTEXT_SETUP, Criticality::reject, message)))
}

/// PDU SESSION RESOURCE SETUP REQUEST for one session: the UE's NAS (the
/// SMF's accept in DL NAS Transport; none for a reactivation) and the
/// SMF's transfer for the gNB.
pub(crate) fn pdu_session_resource_setup_request(
    amf_id: AmfUeId,
    ran_id: RanUeId,
    pdu_session_id: u8,
    nas: Option<&[u8]>,
    slice: Snssai,
    transfer: &[u8],
) -> Result<Vec<u8>, String> {
    use AnonymousPDUSessionResourceSetupRequestProtocolIEsCriticality::reject;
    let item = PDUSessionResourceSetupItemSUReq::new(
        PDUSessionID(pdu_session_id),
        nas.map(|nas| NASPDU(nas.into())),
        snssai(slice),
        transfer.to_vec().into(),
        None,
    );
    let ie = |id: ProtocolIEID, value: Result<Any, EncodeError>| {
        value.map(|v| AnonymousPDUSessionResourceSetupRequestProtocolIEs::new(id.0, reject, v)).map_err(encoding_error)
    };
    let ies = vec![
        ie(ID_AMF_UE_NGAP_ID, wrap(&AMFUENGAPID(amf_id)))?,
        ie(ID_RAN_UE_NGAP_ID, wrap(&RANUENGAPID(ran_id)))?,
        ie(ID_PDUSESSION_RESOURCE_SETUP_LIST_SUREQ, wrap(&PDUSessionResourceSetupListSUReq(vec![item])))?,
    ];
    let message = wrap(&PDUSessionResourceSetupRequest::new(PDUSessionResourceSetupRequestProtocolIEs(ies)))
        .map_err(encoding_error)?;
    pdu(NGAPPDU::initiatingMessage(InitiatingMessage::new(ID_PDUSESSION_RESOURCE_SETUP, Criticality::reject, message)))
}

/// PDU SESSION RESOURCE RELEASE COMMAND for one session: the UE's NAS (the
/// SMF's Release Command in DL NAS Transport) and the SMF's transfer.
pub(crate) fn pdu_session_resource_release_command(
    amf_id: AmfUeId,
    ran_id: RanUeId,
    pdu_session_id: u8,
    nas: Option<&[u8]>,
    transfer: &[u8],
) -> Result<Vec<u8>, String> {
    use AnonymousPDUSessionResourceReleaseCommandProtocolIEsCriticality::{ignore, reject};
    let item = PDUSessionResourceToReleaseItemRelCmd::new(PDUSessionID(pdu_session_id), transfer.to_vec().into(), None);
    let ie = |id: ProtocolIEID, criticality, value: Result<Any, EncodeError>| {
        value
            .map(|v| AnonymousPDUSessionResourceReleaseCommandProtocolIEs::new(id.0, criticality, v))
            .map_err(encoding_error)
    };
    let mut ies = vec![
        ie(ID_AMF_UE_NGAP_ID, reject, wrap(&AMFUENGAPID(amf_id)))?,
        ie(ID_RAN_UE_NGAP_ID, reject, wrap(&RANUENGAPID(ran_id)))?,
    ];
    if let Some(nas) = nas {
        ies.push(ie(ID_NAS_PDU, ignore, wrap(&NASPDU(nas.into())))?);
    }
    ies.push(ie(
        ID_PDUSESSION_RESOURCE_TO_RELEASE_LIST_REL_CMD,
        reject,
        wrap(&PDUSessionResourceToReleaseListRelCmd(vec![item])),
    )?);
    let message = wrap(&PDUSessionResourceReleaseCommand::new(PDUSessionResourceReleaseCommandProtocolIEs(ies)))
        .map_err(encoding_error)?;
    pdu(NGAPPDU::initiatingMessage(InitiatingMessage::new(
        ID_PDUSESSION_RESOURCE_RELEASE,
        Criticality::reject,
        message,
    )))
}

/// UE CONTEXT RELEASE COMMAND for a UE.
pub(crate) fn ue_context_release_command(amf_id: AmfUeId, ran_id: RanUeId, cause: Cause) -> Result<Vec<u8>, String> {
    let ids = UENGAPIDs::uE_NGAP_ID_pair(UENGAPIDPair::new(AMFUENGAPID(amf_id), RANUENGAPID(ran_id), None));
    let ies = vec![
        AnonymousUEContextReleaseCommandProtocolIEs::new(
            ID_UE_NGAP_IDS,
            Criticality::reject,
            wrap(&ids).map_err(encoding_error)?,
        ),
        AnonymousUEContextReleaseCommandProtocolIEs::new(
            ID_CAUSE,
            Criticality::ignore,
            wrap(&cause).map_err(encoding_error)?,
        ),
    ];
    let message =
        wrap(&UEContextReleaseCommand::new(UEContextReleaseCommandProtocolIEs(ies))).map_err(encoding_error)?;
    pdu(NGAPPDU::initiatingMessage(InitiatingMessage::new(ID_UECONTEXT_RELEASE, Criticality::reject, message)))
}

/// ERROR INDICATION, about a UE if `ids` are given.
pub(crate) fn error_indication(ids: Option<(AmfUeId, RanUeId)>, cause: Cause) -> Result<Vec<u8>, String> {
    use AnonymousErrorIndicationProtocolIEsCriticality as C;
    let ie = |id: ProtocolIEID, value: Result<Any, EncodeError>| {
        value.map(|v| AnonymousErrorIndicationProtocolIEs::new(id.0, C::ignore, v)).map_err(encoding_error)
    };
    let mut ies = Vec::new();
    if let Some((amf_id, ran_id)) = ids {
        ies.push(ie(ID_AMF_UE_NGAP_ID, wrap(&AMFUENGAPID(amf_id)))?);
        ies.push(ie(ID_RAN_UE_NGAP_ID, wrap(&RANUENGAPID(ran_id)))?);
    }
    ies.push(ie(ID_CAUSE, wrap(&cause))?);
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
        let error = error_indication(None, Cause::protocol(CauseProtocol::transfer_syntax_error)).unwrap();
        assert!(matches!(decode(&error), Incoming::ErrorIndication(cause) if cause.contains("transfer_syntax_error")));
    }

    #[test]
    fn ue_associated_messages_become_domain_values() {
        let Incoming::InitialUe { ran_id, nas, location } = decode(&vector("ci0002")) else { panic!() };
        assert_eq!(nas[..3], [0x7e, 0x00, 0x41], "plain Registration Request");
        let location = location.expect("NR location");
        assert_eq!(location.tai.0, Plmn::new("208", "93").unwrap());
        let Incoming::UplinkNas { amf_id, ran_id: ul_ran, nas, .. } = decode(&vector("ci0004")) else { panic!() };
        assert!(!nas.is_empty());
        assert!(matches!(decode(&vector("ci0096")), Incoming::ReleaseRequest { .. }));
        assert!(matches!(decode(&vector("ci0050")), Incoming::ReleaseComplete { .. }));
        println!("ran {ran_id}, uplink amf {amf_id} ran {ul_ran}");
    }

    #[test]
    fn downlink_and_release_command_are_valid_ngap() {
        let nas = [0x7e, 0x00, 0x44, 0x6f];
        let dl = librecn_ngap::decode(&downlink_nas_transport(7, 9, &nas).unwrap()).unwrap();
        assert_eq!(check(&dl).unwrap(), 4, "message + 3 IEs");
        let NGAPPDU::initiatingMessage(m) = &dl else { panic!() };
        let transport: DownlinkNASTransport = open(&m.value).unwrap();
        let carried = transport.protocol_ies.0.iter().find(|ie| ie.id == ID_NAS_PDU.0).unwrap();
        assert_eq!(&*open::<NASPDU>(&carried.value).unwrap().0, nas);

        let command =
            librecn_ngap::decode(&ue_context_release_command(7, 9, Cause::nas(CauseNas::normal_release)).unwrap())
                .unwrap();
        assert_eq!(check(&command).unwrap(), 3);
        let error = error_indication(Some((7, 9)), Cause::radioNetwork(CauseRadioNetwork::unknown_local_UE_NGAP_ID));
        assert_eq!(check(&librecn_ngap::decode(&error.unwrap()).unwrap()).unwrap(), 4);
    }

    #[test]
    fn garbage_and_unhandled_procedures_are_classified() {
        assert!(matches!(decode(&[0xff, 0x00]), Incoming::Malformed(_)));
        // ci0003: Downlink NAS Transport, which only the AMF sends.
        assert!(matches!(decode(&vector("ci0003")), Incoming::Unsupported { procedure: 4, .. }));
        assert_eq!(decode(&vector("ci0709")), Incoming::NgReset);
    }
}
