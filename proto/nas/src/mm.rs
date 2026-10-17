//! 5GMM messages (TS 24.501 §8.2): uplink ones decoded, downlink ones encoded.
//!
//! A plain 5GMM message is the extended protocol discriminator, a security
//! header type of 0, the message type, then the IEs.

use crate::ie::{
    Dnn, Guti, IdentityType, MobileIdentity, NgKsi, Nssai, SessionSet, SnssaiList, Tai, UeSecurityCapability,
};
use crate::{EPD_5GMM, Error, Reader, Result, Writer};

/// Message types (§9.7).
pub mod kind {
    pub const REGISTRATION_REQUEST: u8 = 0x41;
    pub const REGISTRATION_ACCEPT: u8 = 0x42;
    pub const REGISTRATION_COMPLETE: u8 = 0x43;
    pub const REGISTRATION_REJECT: u8 = 0x44;
    pub const DEREGISTRATION_REQUEST_UE: u8 = 0x45;
    pub const DEREGISTRATION_ACCEPT_UE: u8 = 0x46;
    pub const DEREGISTRATION_REQUEST_NETWORK: u8 = 0x47;
    pub const DEREGISTRATION_ACCEPT_NETWORK: u8 = 0x48;
    pub const SERVICE_REQUEST: u8 = 0x4c;
    pub const SERVICE_REJECT: u8 = 0x4d;
    pub const SERVICE_ACCEPT: u8 = 0x4e;
    pub const AUTHENTICATION_REQUEST: u8 = 0x56;
    pub const AUTHENTICATION_RESPONSE: u8 = 0x57;
    pub const AUTHENTICATION_REJECT: u8 = 0x58;
    pub const AUTHENTICATION_FAILURE: u8 = 0x59;
    pub const IDENTITY_REQUEST: u8 = 0x5b;
    pub const IDENTITY_RESPONSE: u8 = 0x5c;
    pub const SECURITY_MODE_COMMAND: u8 = 0x5d;
    pub const SECURITY_MODE_COMPLETE: u8 = 0x5e;
    pub const SECURITY_MODE_REJECT: u8 = 0x5f;
    pub const STATUS: u8 = 0x64;
    pub const UL_NAS_TRANSPORT: u8 = 0x67;
    pub const DL_NAS_TRANSPORT: u8 = 0x68;
}

/// The name of 5GMM message type `kind`.
pub fn name(kind: u8) -> &'static str {
    match kind {
        kind::REGISTRATION_REQUEST => "Registration Request",
        kind::REGISTRATION_ACCEPT => "Registration Accept",
        kind::REGISTRATION_COMPLETE => "Registration Complete",
        kind::REGISTRATION_REJECT => "Registration Reject",
        kind::DEREGISTRATION_REQUEST_UE => "Deregistration Request",
        kind::DEREGISTRATION_ACCEPT_UE => "Deregistration Accept",
        kind::DEREGISTRATION_REQUEST_NETWORK => "Deregistration Request (network)",
        kind::DEREGISTRATION_ACCEPT_NETWORK => "Deregistration Accept (network)",
        kind::SERVICE_REQUEST => "Service Request",
        kind::SERVICE_REJECT => "Service Reject",
        kind::SERVICE_ACCEPT => "Service Accept",
        kind::AUTHENTICATION_REQUEST => "Authentication Request",
        kind::AUTHENTICATION_RESPONSE => "Authentication Response",
        kind::AUTHENTICATION_REJECT => "Authentication Reject",
        kind::AUTHENTICATION_FAILURE => "Authentication Failure",
        kind::IDENTITY_REQUEST => "Identity Request",
        kind::IDENTITY_RESPONSE => "Identity Response",
        kind::SECURITY_MODE_COMMAND => "Security Mode Command",
        kind::SECURITY_MODE_COMPLETE => "Security Mode Complete",
        kind::SECURITY_MODE_REJECT => "Security Mode Reject",
        kind::STATUS => "5GMM Status",
        kind::UL_NAS_TRANSPORT => "UL NAS Transport",
        kind::DL_NAS_TRANSPORT => "DL NAS Transport",
        _ => "message",
    }
}

/// 5GMM causes (§9.11.3.2) the AMF sends.
pub mod cause {
    pub const ILLEGAL_UE: u8 = 3;
    pub const UE_IDENTITY_CANNOT_BE_DERIVED: u8 = 9;
    pub const SYNCH_FAILURE: u8 = 21;
    pub const MAC_FAILURE: u8 = 20;
    pub const UE_SECURITY_CAPABILITIES_MISMATCH: u8 = 23;
    pub const SERVING_NETWORK_NOT_AUTHORIZED: u8 = 73;
    pub const SEMANTICALLY_INCORRECT: u8 = 95;
    pub const INVALID_MANDATORY_INFORMATION: u8 = 96;
    pub const MESSAGE_TYPE_NOT_IMPLEMENTED: u8 = 97;
    pub const NOT_COMPATIBLE_WITH_PROTOCOL_STATE: u8 = 98;
    pub const PAYLOAD_NOT_FORWARDED: u8 = 90;
    pub const PROTOCOL_ERROR: u8 = 111;
}

/// Payload container type N1 SM information (§9.11.3.40): a 5GSM message.
pub const PAYLOAD_N1_SM: u8 = 1;

/// Request types of UL NAS Transport (§9.11.3.47).
pub mod request {
    pub const INITIAL: u8 = 1;
    pub const EXISTING: u8 = 2;
}

/// The IEs of a UL NAS Transport (§8.2.10) the AMF uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UlNasTransport<'a> {
    pub container_type: u8,
    pub container: &'a [u8],
    pub pdu_session_id: Option<u8>,
    pub request_type: Option<u8>,
    pub snssai: Option<crate::Snssai>,
    pub dnn: Option<Dnn<'a>>,
}

/// 5GS registration type (§9.11.3.7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistrationKind {
    Initial,
    Mobility,
    Periodic,
    Emergency,
    Other(u8),
}

/// The IEs of a Registration Request (§8.2.6) the AMF uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegistrationRequest<'a> {
    pub kind: RegistrationKind,
    /// Follow-on request pending.
    pub follow_on: bool,
    pub ngksi: NgKsi,
    pub identity: MobileIdentity<'a>,
    pub capability: Option<UeSecurityCapability>,
    pub requested_nssai: Option<&'a [u8]>,
    /// The whole request, when this one carries only the cleartext IEs (§4.4.6).
    pub container: Option<&'a [u8]>,
}

impl RegistrationRequest<'_> {
    /// The requested S-NSSAIs (invalid ones skipped).
    pub fn requested(&self) -> impl Iterator<Item = crate::Snssai> + '_ {
        Nssai(self.requested_nssai.unwrap_or_default()).map_while(Result::ok)
    }
}

/// The IEs of a Service Request (§8.2.16) the AMF uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServiceRequest<'a> {
    pub ngksi: NgKsi,
    /// Service type (§9.11.3.50): 0 signalling, 1 data, 2 mobile terminated...
    pub service_type: u8,
    pub identity: MobileIdentity<'a>,
    /// Sessions with uplink data to send: their user plane to reactivate.
    pub uplink_data: SessionSet,
    /// Sessions the UE holds.
    pub sessions: Option<SessionSet>,
    /// The whole request, when this one carries only the cleartext IEs (§4.4.6).
    pub container: Option<&'a [u8]>,
}

/// The uplink 5GMM messages the AMF handles, decoded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Uplink<'a> {
    RegistrationRequest(RegistrationRequest<'a>),
    RegistrationComplete,
    AuthenticationResponse {
        res_star: Option<&'a [u8]>,
    },
    AuthenticationFailure {
        cause: u8,
        auts: Option<&'a [u8; 14]>,
    },
    SecurityModeComplete {
        imeisv: Option<MobileIdentity<'a>>,
        container: Option<&'a [u8]>,
    },
    SecurityModeReject {
        cause: u8,
    },
    IdentityResponse {
        identity: MobileIdentity<'a>,
    },
    DeregistrationRequest {
        switch_off: bool,
        ngksi: NgKsi,
        identity: MobileIdentity<'a>,
    },
    /// The UE accepts the network's deregistration.
    DeregistrationAccept,
    UlNasTransport(UlNasTransport<'a>),
    ServiceRequest(ServiceRequest<'a>),
    Status {
        cause: u8,
    },
}

impl<'a> Uplink<'a> {
    /// Decodes a plain 5GMM message.
    pub fn decode(pdu: &'a [u8]) -> Result<Self> {
        let mut r = Reader::new(pdu);
        if r.u8()? != EPD_5GMM || r.u8()? & 0x0f != 0 {
            return Err(Error::Invalid("plain 5GMM header"));
        }
        let kind = r.u8()?;
        let message = match kind {
            kind::REGISTRATION_REQUEST => Uplink::RegistrationRequest(registration_request(r)?),
            kind::REGISTRATION_COMPLETE => Uplink::RegistrationComplete,
            kind::AUTHENTICATION_RESPONSE => {
                let mut res_star = None;
                while let Some((iei, v)) = r.optional(&[])? {
                    if iei == 0x2d {
                        res_star = Some(v);
                    }
                }
                Uplink::AuthenticationResponse { res_star }
            }
            kind::AUTHENTICATION_FAILURE => {
                let cause = r.u8()?;
                let mut auts = None;
                while let Some((iei, v)) = r.optional(&[])? {
                    if iei == 0x30 {
                        auts = Some(v.try_into().map_err(|_| Error::Invalid("authentication failure parameter"))?);
                    }
                }
                Uplink::AuthenticationFailure { cause, auts }
            }
            kind::SECURITY_MODE_COMPLETE => {
                let (mut imeisv, mut container) = (None, None);
                while let Some((iei, v)) = r.optional(&[])? {
                    match iei {
                        0x77 => imeisv = Some(MobileIdentity::decode(v)?),
                        0x71 => container = Some(v),
                        _ => {}
                    }
                }
                Uplink::SecurityModeComplete { imeisv, container }
            }
            kind::SECURITY_MODE_REJECT => Uplink::SecurityModeReject { cause: r.u8()? },
            kind::IDENTITY_RESPONSE => Uplink::IdentityResponse { identity: MobileIdentity::decode(r.lve()?)? },
            kind::DEREGISTRATION_REQUEST_UE => {
                let octet = r.u8()?;
                let identity = MobileIdentity::decode(r.lve()?)?;
                Uplink::DeregistrationRequest {
                    switch_off: octet & 0x08 != 0,
                    ngksi: NgKsi::from_nibble(octet >> 4),
                    identity,
                }
            }
            kind::DEREGISTRATION_ACCEPT_NETWORK => Uplink::DeregistrationAccept,
            kind::STATUS => Uplink::Status { cause: r.u8()? },
            kind::UL_NAS_TRANSPORT => Uplink::UlNasTransport(ul_nas_transport(r)?),
            kind::SERVICE_REQUEST => Uplink::ServiceRequest(service_request(r)?),
            _ => return Err(Error::UnknownMessage(kind)),
        };
        Ok(message)
    }

    /// The message type.
    pub fn kind(&self) -> u8 {
        match self {
            Uplink::RegistrationRequest(_) => kind::REGISTRATION_REQUEST,
            Uplink::RegistrationComplete => kind::REGISTRATION_COMPLETE,
            Uplink::AuthenticationResponse { .. } => kind::AUTHENTICATION_RESPONSE,
            Uplink::AuthenticationFailure { .. } => kind::AUTHENTICATION_FAILURE,
            Uplink::SecurityModeComplete { .. } => kind::SECURITY_MODE_COMPLETE,
            Uplink::SecurityModeReject { .. } => kind::SECURITY_MODE_REJECT,
            Uplink::IdentityResponse { .. } => kind::IDENTITY_RESPONSE,
            Uplink::DeregistrationRequest { .. } => kind::DEREGISTRATION_REQUEST_UE,
            Uplink::DeregistrationAccept => kind::DEREGISTRATION_ACCEPT_NETWORK,
            Uplink::UlNasTransport(_) => kind::UL_NAS_TRANSPORT,
            Uplink::ServiceRequest(_) => kind::SERVICE_REQUEST,
            Uplink::Status { .. } => kind::STATUS,
        }
    }
}

fn registration_request(mut r: Reader<'_>) -> Result<RegistrationRequest<'_>> {
    let octet = r.u8()?;
    let kind = match octet & 0x7 {
        1 => RegistrationKind::Initial,
        2 => RegistrationKind::Mobility,
        3 => RegistrationKind::Periodic,
        4 => RegistrationKind::Emergency,
        other => RegistrationKind::Other(other),
    };
    let mut request = RegistrationRequest {
        kind,
        follow_on: octet & 0x08 != 0,
        ngksi: NgKsi::from_nibble(octet >> 4),
        identity: MobileIdentity::decode(r.lve()?)?,
        capability: None,
        requested_nssai: None,
        container: None,
    };
    // Last visited registered TAI (0x52) is the only TV IE of the message.
    while let Some((iei, v)) = r.optional(&[(0x52, 6)])? {
        match iei {
            0x2e => request.capability = Some(UeSecurityCapability::decode(v)?),
            0x2f => request.requested_nssai = Some(v),
            0x71 => request.container = Some(v),
            _ => {}
        }
    }
    Ok(request)
}

fn service_request(mut r: Reader<'_>) -> Result<ServiceRequest<'_>> {
    let octet = r.u8()?;
    let mut request = ServiceRequest {
        ngksi: NgKsi::from_nibble(octet & 0x0f),
        service_type: octet >> 4,
        identity: MobileIdentity::decode(r.lve()?)?,
        uplink_data: SessionSet::default(),
        sessions: None,
        container: None,
    };
    while let Some((iei, v)) = r.optional(&[])? {
        match iei {
            0x40 => request.uplink_data = SessionSet::decode(v)?,
            0x50 => request.sessions = Some(SessionSet::decode(v)?),
            0x71 => request.container = Some(v),
            _ => {}
        }
    }
    Ok(request)
}

fn ul_nas_transport(mut r: Reader<'_>) -> Result<UlNasTransport<'_>> {
    let container_type = r.u8()? & 0x0f;
    let mut t = UlNasTransport {
        container_type,
        container: r.lve()?,
        pdu_session_id: None,
        request_type: None,
        snssai: None,
        dnn: None,
    };
    // PDU session ID (0x12) and old PDU session ID (0x59) are TV.
    while let Some((iei, v)) = r.optional(&[(0x12, 1), (0x59, 1)])? {
        match iei {
            0x12 => t.pdu_session_id = Some(v[0]),
            0x80 => t.request_type = Some(v[0] & 0x07),
            0x22 => t.snssai = Some(crate::Snssai::decode(v)?),
            0x25 => t.dnn = Some(Dnn(v)),
            _ => {}
        }
    }
    Ok(t)
}

/// The downlink 5GMM messages the AMF sends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Downlink<'a> {
    AuthenticationRequest {
        ngksi: u8,
        rand: [u8; 16],
        autn: [u8; 16],
    },
    AuthenticationReject,
    IdentityRequest(IdentityType),
    SecurityModeCommand {
        ciphering: u8,
        integrity: u8,
        ngksi: u8,
        replayed: UeSecurityCapability,
        /// Ask for the whole initial message again (RINMR, §9.11.3.12).
        retransmit_initial: bool,
    },
    RegistrationAccept {
        guti: Guti,
        tai: Tai,
        allowed: SnssaiList,
    },
    RegistrationReject {
        cause: u8,
    },
    DeregistrationAccept,
    /// The network deregisters the UE over 3GPP access (§8.2.14);
    /// `reregister`: it must register again.
    DeregistrationRequest {
        reregister: bool,
    },
    ServiceReject {
        cause: u8,
    },
    /// With the sessions the network holds (the UE releases the others).
    ServiceAccept {
        sessions: SessionSet,
    },
    Status {
        cause: u8,
    },
    /// A 5GSM message for PDU session `pdu_session_id`; with a 5GMM cause
    /// when it is the UE's own, sent back (§5.4.5.2.5).
    DlNasTransport {
        pdu_session_id: u8,
        container: &'a [u8],
        cause: Option<u8>,
    },
}

/// ABBA (§9.11.3.10): 0x0000 for this release of TS 33.501.
pub const ABBA: [u8; 2] = [0, 0];

impl Downlink<'_> {
    pub fn kind(&self) -> u8 {
        match self {
            Downlink::AuthenticationRequest { .. } => kind::AUTHENTICATION_REQUEST,
            Downlink::AuthenticationReject => kind::AUTHENTICATION_REJECT,
            Downlink::IdentityRequest(_) => kind::IDENTITY_REQUEST,
            Downlink::SecurityModeCommand { .. } => kind::SECURITY_MODE_COMMAND,
            Downlink::RegistrationAccept { .. } => kind::REGISTRATION_ACCEPT,
            Downlink::RegistrationReject { .. } => kind::REGISTRATION_REJECT,
            Downlink::DeregistrationAccept => kind::DEREGISTRATION_ACCEPT_UE,
            Downlink::DeregistrationRequest { .. } => kind::DEREGISTRATION_REQUEST_NETWORK,
            Downlink::ServiceReject { .. } => kind::SERVICE_REJECT,
            Downlink::ServiceAccept { .. } => kind::SERVICE_ACCEPT,
            Downlink::Status { .. } => kind::STATUS,
            Downlink::DlNasTransport { .. } => kind::DL_NAS_TRANSPORT,
        }
    }

    /// Encodes the plain message into `out`; returns its length.
    pub fn encode(&self, out: &mut [u8]) -> Result<usize> {
        let mut w = Writer::new(out);
        w.bytes(&[EPD_5GMM, 0, self.kind()]);
        match *self {
            Downlink::AuthenticationRequest { ngksi, rand, autn } => {
                w.u8(ngksi & 0x7);
                w.lv(|w| w.bytes(&ABBA));
                w.u8(0x21);
                w.bytes(&rand);
                w.tlv(0x20, |w| w.bytes(&autn));
            }
            Downlink::IdentityRequest(identity) => w.u8(identity as u8),
            Downlink::SecurityModeCommand { ciphering, integrity, ngksi, replayed, retransmit_initial } => {
                w.u8(ciphering << 4 | integrity & 0xf);
                w.u8(NgKsi { mapped: false, value: ngksi }.nibble());
                w.lv(|w| w.bytes(replayed.as_bytes()));
                w.u8(0xe1); // IMEISV requested
                w.tlv(0x36, |w| w.u8(u8::from(retransmit_initial) << 1));
            }
            Downlink::RegistrationAccept { guti, tai, allowed } => {
                w.lv(|w| w.u8(0x01)); // registered over 3GPP access, no SMS
                w.tlve(0x77, |w| guti.encode(w));
                w.tlv(0x54, |w| tai.encode_list(w));
                if !allowed.is_empty() {
                    w.tlv(0x15, |w| allowed.encode(w));
                }
            }
            Downlink::RegistrationReject { cause } | Downlink::ServiceReject { cause } | Downlink::Status { cause } => {
                w.u8(cause)
            }
            Downlink::ServiceAccept { sessions } => w.tlv(0x50, |w| w.bytes(&sessions.octets())),
            // De-registration type (§9.11.3.20): not switch off, re-registration
            // required or not, 3GPP access; the spare half octet.
            Downlink::DeregistrationRequest { reregister } => w.u8(u8::from(reregister) << 2 | 0x01),
            Downlink::DlNasTransport { pdu_session_id, container, cause } => {
                w.u8(PAYLOAD_N1_SM);
                w.lve(|w| w.bytes(container));
                w.bytes(&[0x12, pdu_session_id]);
                if let Some(cause) = cause {
                    w.bytes(&[0x58, cause]);
                }
            }
            Downlink::AuthenticationReject | Downlink::DeregistrationAccept => {}
        }
        w.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Plmn, Snssai, hex};

    /// UERANSIM's initial Registration Request: cleartext IEs only (SUCI,
    /// UE security capability), then the whole request in the container of
    /// Security Mode Complete.
    #[test]
    fn registration_request_is_decoded() {
        let pdu = hex("7e004179 000d 0199f907 0000 0000 0000000010 2e02f0f0 2f020101 52 99f907000001 7100037e0043");
        let Uplink::RegistrationRequest(r) = Uplink::decode(&pdu).unwrap() else { panic!() };
        assert_eq!(
            (r.kind, r.follow_on, r.ngksi),
            (RegistrationKind::Initial, true, NgKsi { mapped: false, value: 7 })
        );
        assert_eq!(r.identity.to_string(), "suci-0-999-70-0000-0-0-0000000001");
        assert!(r.capability.unwrap().integrity(2));
        assert_eq!(r.requested().collect::<Vec<_>>(), [Snssai { sst: 1, sd: None }]);
        assert_eq!(r.container, Some(&hex("7e0043")[..]));
    }

    #[test]
    fn uplink_messages_are_decoded_and_bad_ones_refused() {
        let res = hex("7e0057 2d10 00112233445566778899aabbccddeeff");
        assert_eq!(Uplink::decode(&res).unwrap(), Uplink::AuthenticationResponse { res_star: Some(&res[5..]) });
        let failure = hex("7e0059 15 300e 0102030405060708090a0b0c0d0e");
        let Uplink::AuthenticationFailure { cause: 21, auts: Some(auts) } = Uplink::decode(&failure).unwrap() else {
            panic!()
        };
        assert_eq!(auts[13], 0x0e);
        let dereg = hex("7e0045 01 000b f299f907010040 01020304");
        let Uplink::DeregistrationRequest { switch_off: false, ngksi, identity: MobileIdentity::Guti(g) } =
            Uplink::decode(&dereg).unwrap()
        else {
            panic!()
        };
        assert_eq!((ngksi.value, g.tmsi, g.amf_set, g.amf_pointer), (0, 0x01020304, 1, 0));
        // UERANSIM's Service Request from idle, cleartext IEs: ngKSI 0, data, 5G-S-TMSI.
        let sr = hex("7e004c 10 0007 f4004001020304 4002 0200 5002 0200 7100 027e00");
        let Uplink::ServiceRequest(r) = Uplink::decode(&sr).unwrap() else { panic!() };
        assert_eq!((r.ngksi.value, r.service_type), (0, 1));
        assert!(matches!(r.identity, MobileIdentity::STmsi { tmsi: 0x01020304, .. }));
        assert_eq!((r.uplink_data.0, r.sessions, r.container), (0x02, Some(SessionSet(0x02)), Some(&hex("7e00")[..])));
        // UERANSIM's PDU session request: N1 SM container, PSI 1, initial, SST 1, DNN "internet".
        let ul = hex("7e0067 01 0003 2e0101 12 01 81 22 01 01 25 09 08696e7465726e6574");
        let Uplink::UlNasTransport(t) = Uplink::decode(&ul).unwrap() else { panic!() };
        assert_eq!((t.container_type, t.container), (PAYLOAD_N1_SM, &hex("2e0101")[..]));
        assert_eq!((t.pdu_session_id, t.request_type), (Some(1), Some(request::INITIAL)));
        assert_eq!(t.snssai, Some(Snssai { sst: 1, sd: None }));
        assert!(t.dnn.unwrap().is("internet"));
        assert_eq!(Uplink::decode(&hex("7e0048")), Ok(Uplink::DeregistrationAccept));
        assert_eq!(Uplink::decode(&hex("7e0099")), Err(Error::UnknownMessage(0x99)));
        assert_eq!(Uplink::decode(&hex("7e0041 01 0005 01")), Err(Error::Truncated));
        assert_eq!(Uplink::decode(&hex("2e0041")), Err(Error::Invalid("plain 5GMM header")));
    }

    /// The registration downlink messages, byte for byte (checked against TS 24.501 §8.2).
    #[test]
    fn downlink_messages_are_encoded() {
        let mut buf = [0u8; 128];
        let encode = |m: Downlink, buf: &mut [u8]| {
            let n = m.encode(buf).unwrap();
            hex_of(&buf[..n])
        };
        let h = |s: &str| s.replace(' ', "");
        let auth = Downlink::AuthenticationRequest { ngksi: 0, rand: [0xaa; 16], autn: [0xbb; 16] };
        assert_eq!(
            encode(auth, &mut buf),
            h(&format!("7e0056 00 020000 21{} 2010{}", "aa".repeat(16), "bb".repeat(16)))
        );
        let replayed = UeSecurityCapability::decode(&[0xf0, 0xf0]).unwrap();
        let smc =
            Downlink::SecurityModeCommand { ciphering: 2, integrity: 2, ngksi: 0, replayed, retransmit_initial: true };
        assert_eq!(encode(smc, &mut buf), h("7e005d 22 00 02f0f0 e1 360102"));
        let plmn = Plmn::new("999", "70").unwrap();
        let guti = Guti { plmn, amf_region: 1, amf_set: 1, amf_pointer: 0, tmsi: 0x01020304 };
        let mut allowed = SnssaiList::default();
        allowed.push(Snssai { sst: 1, sd: None });
        allowed.push(Snssai { sst: 1, sd: None });
        let accept = Downlink::RegistrationAccept { guti, tai: Tai { plmn, tac: [0, 0, 1] }, allowed };
        let expected = "7e0042 0101 77000b f299f907 01 0040 01020304 5407 00 99f907 000001 1502 0101";
        assert_eq!(encode(accept, &mut buf), h(expected), "one slice: duplicates dropped");
        assert_eq!(encode(Downlink::RegistrationReject { cause: 111 }, &mut buf), "7e00446f");
        assert_eq!(encode(Downlink::IdentityRequest(IdentityType::Suci), &mut buf), "7e005b01");
        assert_eq!(encode(Downlink::DeregistrationAccept, &mut buf), "7e0046");
        assert_eq!(encode(Downlink::DeregistrationRequest { reregister: true }, &mut buf), "7e004705");
        assert_eq!(encode(Downlink::DeregistrationRequest { reregister: false }, &mut buf), "7e004701");
        assert_eq!(
            encode(Downlink::ServiceAccept { sessions: SessionSet(0x02) }, &mut buf),
            "7e004e 50020200".replace(' ', "")
        );
        let n1 = hex("2e0101c3 1b");
        let dl = Downlink::DlNasTransport { pdu_session_id: 1, container: &n1, cause: Some(90) };
        assert_eq!(encode(dl, &mut buf), h("7e0068 01 0005 2e0101c31b 1201 585a"));
        assert_eq!(Downlink::AuthenticationReject.encode(&mut [0u8; 2]), Err(Error::BufferTooSmall));
    }

    fn hex_of(b: &[u8]) -> String {
        b.iter().map(|b| format!("{b:02x}")).collect()
    }
}
