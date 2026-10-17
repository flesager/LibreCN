//! 5GMM for one UE (TS 24.501 §5): registration with 5G AKA, NAS security
//! mode control, identification, deregistration by the UE or the network.
//!
//! Pure logic on domain values: the EO hands in the UE's uplink NAS PDUs
//! and the AUSF's answers; this module acts through [`Effects`].
//!
//! ```text
//! Registration Request (SUCI) ─▶ AUSF ─▶ Authentication Request
//! Authentication Response (RES*, checked against HXRES*) ─▶ AUSF confirms ─▶ K_AMF
//! Security Mode Command (new context) ◀▶ Security Mode Complete (whole request)
//! Registration Accept (5G-GUTI) in Initial Context Setup (K_gNB) ◀▶ Registration Complete: registered
//! UL NAS Transport (5GSM) ─▶ the SMF, through the EO
//! ```
//!
//! A registered UE keeps its context when its N2 connection goes (CM-IDLE,
//! kept by the EO). It comes back with an initial message integrity
//! protected with that context, the whole message ciphered in its NAS
//! message container (§4.4.6): a Service Request (Service Accept in Initial
//! Context Setup, sessions reactivated) or a Registration Request (accepted
//! without a new authentication). A 5G-GUTI the AMF does not know is
//! answered by an Identity Request for the SUCI.
//!
//! A message waiting for an answer is sent again when its timer expires
//! (§10.2): T3570 (Identity Request), T3560 (Authentication Request,
//! Security Mode Command), T3550 (Registration Accept), T3522 (the network's
//! Deregistration Request), 6 s, four times; the fifth expiry aborts the
//! procedure (a deregistration then completes locally).

use std::fmt;
use std::time::Duration;

use librecn_common::log::Level;
use librecn_nas::Snssai;
use librecn_nas::ie::{
    Dnn, Guti, IdentityType, MobileIdentity, NgKsi, SessionSet, SnssaiList, Tai, UeSecurityCapability,
};
use librecn_nas::mm::{
    ABBA, Downlink, PAYLOAD_N1_SM, RegistrationKind, ServiceRequest, UlNasTransport, Uplink, cause, kind, name,
};
use librecn_nas::security::{self, Header, NEA0, NIA2, SecurityContext};
use librecn_sbi::ue_authentication::{AuthenticateResult, ConfirmResult};
use librecn_sbi::{Outcome, Supi, SupiOrSuci};
use librecn_security::kdf;

use crate::config::AmfConfig;

/// Largest plain NAS message the AMF sends.
const DOWNLINK_MAX: usize = 256;
/// T3550, T3560, T3570 (§10.2).
pub const T_NAS: Duration = Duration::from_secs(6);
/// Retransmissions before a procedure is aborted (§5.4, §5.5).
const RETRANSMISSIONS: u8 = 4;

/// Why the AMF releases the UE's N2 context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Release {
    /// Registration rejected, or 5GMM done with the UE.
    Normal,
    Deregister,
    AuthenticationFailure,
}

/// The access stratum context the gNB gets in Initial Context Setup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AsContext {
    pub kgnb: [u8; 32],
    pub capability: UeSecurityCapability,
    pub allowed: SnssaiList,
}

/// A 5GSM message from the UE, for the SMF.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionRequest<'a> {
    pub supi: Supi,
    pub pdu_session_id: u8,
    /// An initial request (a new session), not one for an existing session.
    pub initial: bool,
    /// The UE's slice, else the first allowed one.
    pub snssai: Snssai,
    pub dnn: Option<Dnn<'a>>,
    pub container: &'a [u8],
}

/// What 5GMM needs from the AMF.
pub trait Effects {
    /// Sends a NAS PDU (already protected) to the UE; `what` names it for logs.
    fn send(&mut self, pdu: &[u8], what: &'static str);
    /// Sends a NAS PDU in an Initial Context Setup, with the AS context.
    fn setup_context(&mut self, pdu: &[u8], context: &AsContext, what: &'static str);
    /// The PDU sessions the network holds for the UE.
    fn sessions(&mut self) -> SessionSet;
    /// The sessions the UE was paged for while idle (asked once).
    fn paged(&mut self) -> SessionSet;
    /// The UE is back: reactivate the user plane of `reactivate`, release
    /// the sessions the UE no longer holds (if it said which it holds).
    fn resume_sessions(&mut self, reactivate: SessionSet, held_by_ue: Option<SessionSet>);
    /// Hands a 5GSM message to the SMF; `Err` with a 5GMM cause if it cannot
    /// be forwarded (the message then goes back to the UE).
    fn session(&mut self, request: &SessionRequest<'_>) -> Result<(), u8>;
    /// Asks the AUSF to authenticate (with RAND and AUTS after a synch
    /// failure). False if the AUSF cannot be reached.
    fn authenticate(&mut self, who: &SupiOrSuci, resync: Option<([u8; 16], [u8; 14])>) -> bool;
    /// Confirms 5G AKA with the UE's RES*. False if the AUSF cannot be reached.
    fn confirm(&mut self, auth_ctx: u32, res_star: &[u8; 16]) -> bool;
    /// Releases the UE's N2 context, after the PDUs already sent.
    fn release(&mut self, why: Release);
    /// A fresh, unpredictable 5G-TMSI.
    fn tmsi(&mut self) -> u32;
    /// (Re)starts the UE's NAS timer: [`Mm::on_timer`] after `after`.
    fn arm(&mut self, after: Duration);
    /// Stops it.
    fn disarm(&mut self);
    fn log(&mut self, level: Level, args: fmt::Arguments<'_>);
}

/// Where the UE's registration stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Deregistered,
    /// Identity Request sent.
    Identifying,
    /// Waiting for the AUSF's challenge.
    Authenticating,
    /// Authentication Request sent.
    Challenged,
    /// Waiting for the AUSF's confirmation.
    Confirming,
    /// Security Mode Command sent.
    SecurityMode,
    /// Registration Accept sent.
    Accepting,
    Registered,
    /// The network's Deregistration Request sent.
    Deregistering,
}

/// The challenge sent to the UE.
#[derive(Clone, Copy, Debug)]
struct Challenge {
    auth_ctx: u32,
    rand: [u8; 16],
    hxres_star: [u8; 16],
}

/// The timer guarding a message (§10.2), for logs.
fn timer_name(message: &Downlink<'_>) -> &'static str {
    match message {
        Downlink::IdentityRequest(_) => "T3570",
        Downlink::RegistrationAccept { .. } => "T3550",
        Downlink::DeregistrationRequest { .. } => "T3522",
        _ => "T3560",
    }
}

/// One UE's 5GMM context.
#[derive(Clone, Debug, Default)]
pub struct Mm {
    state: State,
    /// From the Registration Request.
    capability: Option<UeSecurityCapability>,
    requested: SnssaiList,
    ue_ngksi: u8,
    suci: Option<SupiOrSuci>,
    /// The ngKSI given to the new security context.
    ngksi: u8,
    resynchronised: bool,
    challenge: Option<Challenge>,
    security: Option<SecurityContext>,
    kamf: Option<[u8; 32]>,
    supi: Option<Supi>,
    guti: Option<Guti>,
    allowed: SnssaiList,
    /// The message waiting for its answer, and how often it was sent again.
    retransmit: Option<(Downlink<'static>, u8)>,
    /// A registration from idle without follow-on request: once complete,
    /// the UE goes back to idle (§5.5.1.3.4).
    release_when_complete: bool,
}

/// The type of an initial message (plain or integrity protected), if it is one.
fn initial_kind(pdu: &[u8]) -> Option<u8> {
    let inner = match security::header(pdu)? {
        0 => pdu,
        1 => security::skip_header(pdu).ok()?,
        _ => return None,
    };
    inner.get(2).copied().filter(|k| [kind::REGISTRATION_REQUEST, kind::SERVICE_REQUEST].contains(k))
}

/// The 5G-TMSI an initial message names, if it is one of this AMF's (the
/// UE may have an idle context here): a 5G-GUTI of this AMF in a
/// Registration Request, the 5G-S-TMSI of a Service Request.
pub fn initial_tmsi(pdu: &[u8], config: &AmfConfig) -> Option<u32> {
    initial_kind(pdu)?;
    let inner = if security::header(pdu)? == 0 { pdu } else { security::skip_header(pdu).ok()? };
    let ours = |set: u16, pointer: u8| set == config.amf_set && pointer == config.amf_pointer;
    match Uplink::decode(inner).ok()? {
        Uplink::RegistrationRequest(r) => match r.identity {
            MobileIdentity::Guti(g) if g.plmn == config.plmn && g.amf_region == config.amf_region => {
                ours(g.amf_set, g.amf_pointer).then_some(g.tmsi)
            }
            _ => None,
        },
        Uplink::ServiceRequest(r) => match r.identity {
            MobileIdentity::STmsi { amf_set, amf_pointer, tmsi } => ours(amf_set, amf_pointer).then_some(tmsi),
            _ => None,
        },
        _ => None,
    }
}

/// Plain messages accepted once a security context exists (§4.4.4.3).
const PLAIN_ALLOWED: [u8; 7] = [
    kind::REGISTRATION_REQUEST,
    kind::IDENTITY_RESPONSE,
    kind::AUTHENTICATION_RESPONSE,
    kind::AUTHENTICATION_FAILURE,
    kind::SECURITY_MODE_REJECT,
    kind::DEREGISTRATION_REQUEST_UE,
    kind::SERVICE_REQUEST,
];

impl Mm {
    pub fn state(&self) -> State {
        self.state
    }

    pub fn supi(&self) -> Option<&Supi> {
        self.supi.as_ref()
    }

    /// The 5G-TMSI of the UE's 5G-GUTI, the key of its idle context.
    pub fn tmsi(&self) -> Option<u32> {
        self.guti.map(|g| g.tmsi)
    }

    /// An uplink NAS PDU; `tai` is where the UE is. `pdu` is deciphered in place.
    pub fn on_uplink(&mut self, config: &AmfConfig, tai: Option<Tai>, pdu: &mut [u8], fx: &mut impl Effects) {
        let Some(header) = security::header(pdu) else {
            return fx.log(Level::Info, format_args!("uplink {} dropped", librecn_nas::describe(pdu)));
        };
        let protected = header != Header::Plain as u8;
        let mut verified = false;
        let plain: &[u8] = match (&mut self.security, protected) {
            (Some(ctx), true) => match ctx.unprotect(pdu) {
                Ok(plain) => {
                    verified = true;
                    plain
                }
                // An initial message the kept context does not check (§4.4.4.3).
                Err(e) => match initial_kind(pdu) {
                    Some(kind::REGISTRATION_REQUEST) => {
                        fx.log(Level::Info, format_args!("Registration Request: {e}, the UE starts over"));
                        self.security = None;
                        security::skip_header(pdu).unwrap_or_default()
                    }
                    Some(kind::SERVICE_REQUEST) => {
                        fx.log(Level::Info, format_args!("Service Request: {e}"));
                        self.send(Downlink::ServiceReject { cause: cause::UE_IDENTITY_CANNOT_BE_DERIVED }, fx);
                        return fx.release(Release::Normal);
                    }
                    _ => return fx.log(Level::Info, format_args!("uplink discarded: {e}")),
                },
            },
            // An initial message protected with a context this AMF does not
            // have: only its cleartext IEs are used (§4.4.6).
            (None, true) => match security::skip_header(pdu) {
                Ok(inner) => inner,
                Err(e) => return fx.log(Level::Info, format_args!("uplink discarded: {e}")),
            },
            (Some(_), false) if !pdu.get(2).is_some_and(|k| PLAIN_ALLOWED.contains(k)) => {
                let what = librecn_nas::describe(pdu);
                return fx.log(Level::Info, format_args!("uplink {what} discarded: not integrity protected"));
            }
            (_, false) => pdu,
        };
        let message = match Uplink::decode(plain) {
            Ok(message) => message,
            Err(e) => {
                fx.log(Level::Info, format_args!("uplink {}: {e}", librecn_nas::describe(plain)));
                let cause = match e {
                    librecn_nas::Error::UnknownMessage(_) => cause::MESSAGE_TYPE_NOT_IMPLEMENTED,
                    _ => cause::INVALID_MANDATORY_INFORMATION,
                };
                return self.send(Downlink::Status { cause }, fx);
            }
        };
        let security = if verified {
            "protected"
        } else if protected {
            "protected, unchecked"
        } else {
            "plain"
        };
        fx.log(Level::Info, format_args!("uplink {} ({security})", name(message.kind())));
        // From an idle UE: the whole message is in the ciphered container.
        let container = match message {
            Uplink::RegistrationRequest(r) if verified && header == Header::Integrity as u8 => r.container,
            Uplink::ServiceRequest(r) if verified && header == Header::Integrity as u8 => r.container,
            _ => None,
        };
        if let (Some(container), Some(ctx)) = (container, &self.security) {
            let mut whole = [0u8; DOWNLINK_MAX];
            if let Some(whole) = whole.get_mut(..container.len()) {
                whole.copy_from_slice(container);
                ctx.decipher_container(whole);
                match Uplink::decode(whole) {
                    Ok(inner) if inner.kind() == message.kind() => {
                        return self.on_message(config, tai, inner, true, fx);
                    }
                    _ => fx.log(Level::Info, format_args!("NAS message container unreadable: cleartext IEs used")),
                }
            }
        }
        self.on_message(config, tai, message, verified, fx);
    }

    /// `verified`: integrity checked with the current security context.
    fn on_message(
        &mut self,
        config: &AmfConfig,
        tai: Option<Tai>,
        message: Uplink<'_>,
        verified: bool,
        fx: &mut impl Effects,
    ) {
        // An answer, or a new procedure: the guarded message is done with.
        let answers = matches!(
            (self.state, &message),
            (State::Identifying, Uplink::IdentityResponse { .. })
                | (State::Challenged, Uplink::AuthenticationResponse { .. } | Uplink::AuthenticationFailure { .. })
                | (State::SecurityMode, Uplink::SecurityModeComplete { .. } | Uplink::SecurityModeReject { .. })
                | (State::Accepting, Uplink::RegistrationComplete)
                | (State::Deregistering, Uplink::DeregistrationAccept)
                | (_, Uplink::RegistrationRequest(_) | Uplink::DeregistrationRequest { .. })
        );
        if answers && self.retransmit.take().is_some() {
            fx.disarm();
        }
        match (self.state, message) {
            // Back from idle: the kept context vouches for the UE (§5.5.1.3).
            (State::Registered, Uplink::RegistrationRequest(request)) if verified => {
                fx.log(Level::Info, format_args!("registration from idle ({:?}): no new authentication", request.kind));
                let update = matches!(request.kind, RegistrationKind::Mobility | RegistrationKind::Periodic);
                self.release_when_complete = update && !request.follow_on;
                if request.capability.is_some() {
                    self.capability = request.capability;
                }
                self.requested = SnssaiList::default();
                request.requested().for_each(|s| self.requested.push(s));
                self.accept(config, tai, Some((request.uplink_data, request.sessions)), fx);
            }
            (State::Registered, Uplink::ServiceRequest(request)) if verified => self.on_service_request(&request, fx),
            (_, Uplink::ServiceRequest(_)) => {
                // No context vouches for this UE: it must register again (§5.6.1.5).
                self.send(Downlink::ServiceReject { cause: cause::UE_IDENTITY_CANNOT_BE_DERIVED }, fx);
                fx.release(Release::Normal);
            }
            (_, Uplink::RegistrationRequest(request)) => {
                // A new registration restarts 5GMM for this UE (§5.5.1.2.8).
                *self = Mm::default();
                self.on_registration_request(&request, fx);
            }
            (State::Identifying, Uplink::IdentityResponse { identity: MobileIdentity::Suci(suci) }) => {
                self.suci = SupiOrSuci::format(format_args!("{suci}"));
                self.authenticate(None, fx);
            }
            (State::Challenged, Uplink::AuthenticationResponse { res_star }) => self.on_response(res_star, fx),
            (State::Challenged, Uplink::AuthenticationFailure { cause, auts }) => self.on_failure(cause, auts, fx),
            (State::SecurityMode, Uplink::SecurityModeComplete { imeisv, container }) => {
                if let Some(imeisv) = imeisv {
                    fx.log(Level::Info, format_args!("{imeisv}"));
                }
                // The whole Registration Request, of which only the cleartext IEs came first.
                if let Some(Ok(Uplink::RegistrationRequest(request))) = container.map(Uplink::decode) {
                    self.requested = SnssaiList::default();
                    request.requested().for_each(|s| self.requested.push(s));
                }
                self.accept(config, tai, None, fx);
            }
            (State::SecurityMode, Uplink::SecurityModeReject { cause }) => {
                fx.log(Level::Info, format_args!("Security Mode Command rejected (5GMM cause #{cause})"));
                self.state = State::Deregistered;
                fx.release(Release::Normal);
            }
            (State::Accepting, Uplink::RegistrationComplete) => {
                self.state = State::Registered;
                if let (Some(supi), Some(guti)) = (&self.supi, &self.guti) {
                    fx.log(Level::Info, format_args!("registered: {supi}, {guti}"));
                }
                if std::mem::take(&mut self.release_when_complete) {
                    fx.log(Level::Info, format_args!("registration updated, no follow-on request: back to idle"));
                    fx.release(Release::Normal);
                }
            }
            (_, Uplink::DeregistrationRequest { switch_off, .. }) => {
                if !switch_off {
                    self.send(Downlink::DeregistrationAccept, fx);
                }
                let off = if switch_off { " (switch off)" } else { "" };
                fx.log(Level::Info, format_args!("deregistered{off}"));
                self.state = State::Deregistered;
                fx.release(Release::Deregister);
            }
            (State::Deregistering, Uplink::DeregistrationAccept) => {
                fx.log(Level::Info, format_args!("deregistered by the network"));
                self.state = State::Deregistered;
                fx.release(Release::Deregister);
            }
            (_, Uplink::Status { cause }) => {
                fx.log(Level::Info, format_args!("5GMM Status from the UE: cause #{cause}"))
            }
            (State::Registered, Uplink::UlNasTransport(t)) => self.on_transport(&t, fx),
            (state, message) => {
                fx.log(Level::Info, format_args!("{} unexpected in state {state:?}", name(message.kind())));
                self.send(Downlink::Status { cause: cause::NOT_COMPATIBLE_WITH_PROTOCOL_STATE }, fx);
            }
        }
    }

    fn on_registration_request(&mut self, request: &librecn_nas::mm::RegistrationRequest<'_>, fx: &mut impl Effects) {
        self.ue_ngksi = request.ngksi.value;
        self.capability = request.capability;
        request.requested().for_each(|s| self.requested.push(s));
        if let RegistrationKind::Other(kind) = request.kind {
            fx.log(Level::Info, format_args!("registration type {kind} not served"));
            return self.reject(cause::INVALID_MANDATORY_INFORMATION, fx);
        }
        match self.capability {
            None => return self.reject(cause::INVALID_MANDATORY_INFORMATION, fx),
            Some(c) if !c.integrity(NIA2) => return self.reject(cause::UE_SECURITY_CAPABILITIES_MISMATCH, fx),
            Some(_) => {}
        }
        match request.identity {
            MobileIdentity::Suci(suci) => {
                self.suci = SupiOrSuci::format(format_args!("{suci}"));
                self.authenticate(None, fx);
            }
            MobileIdentity::Guti(guti) => {
                fx.log(Level::Info, format_args!("{guti} unknown: identification"));
                self.state = State::Identifying;
                self.expect(Downlink::IdentityRequest(IdentityType::Suci), fx);
            }
            other => {
                fx.log(Level::Info, format_args!("{other} not acceptable"));
                self.reject(cause::ILLEGAL_UE, fx);
            }
        }
    }

    fn authenticate(&mut self, resync: Option<([u8; 16], [u8; 14])>, fx: &mut impl Effects) {
        let Some(suci) = self.suci else { return self.reject(cause::ILLEGAL_UE, fx) };
        self.state = State::Authenticating;
        if !fx.authenticate(&suci, resync) {
            self.reject(cause::PROTOCOL_ERROR, fx);
        }
    }

    /// The AUSF's challenge, or its refusal.
    pub fn on_authenticate_result(&mut self, result: &AuthenticateResult, fx: &mut impl Effects) {
        if self.state != State::Authenticating {
            return fx.log(Level::Debug, format_args!("late AUSF challenge dropped"));
        }
        let refusal = match result.outcome() {
            Outcome::Ok => None,
            Outcome::ServingNetworkNotAuthorized => Some(cause::SERVING_NETWORK_NOT_AUTHORIZED),
            Outcome::Unavailable => Some(cause::PROTOCOL_ERROR),
            _ => Some(cause::ILLEGAL_UE),
        };
        if let Some(cause) = refusal {
            fx.log(Level::Info, format_args!("authentication refused by the AUSF: {:?}", result.outcome()));
            return self.reject(cause, fx);
        }
        // A KSI the UE does not hold already (§5.4.1.3.2).
        self.ngksi = if self.ue_ngksi >= NgKsi::NONE { 0 } else { (self.ue_ngksi + 1) % NgKsi::NONE };
        let (rand, autn) = (result.rand, result.autn);
        self.challenge = Some(Challenge { auth_ctx: result.auth_ctx, rand, hxres_star: result.hxres_star });
        self.state = State::Challenged;
        self.expect(Downlink::AuthenticationRequest { ngksi: self.ngksi, rand, autn }, fx);
    }

    fn on_response(&mut self, res_star: Option<&[u8]>, fx: &mut impl Effects) {
        let Some(challenge) = self.challenge else { return };
        let Some(res_star) = res_star.and_then(|r| <&[u8; 16]>::try_from(r).ok()) else {
            fx.log(Level::Info, format_args!("Authentication Response without RES*"));
            return self.reject_authentication(fx);
        };
        // The SEAF's check before asking the home network (TS 33.501 §6.1.3.2.0).
        if kdf::hres_star(&challenge.rand, res_star) != challenge.hxres_star {
            fx.log(Level::Info, format_args!("RES* does not match HXRES*"));
            return self.reject_authentication(fx);
        }
        self.state = State::Confirming;
        if !fx.confirm(challenge.auth_ctx, res_star) {
            self.reject(cause::PROTOCOL_ERROR, fx);
        }
    }

    fn on_failure(&mut self, failure: u8, auts: Option<&[u8; 14]>, fx: &mut impl Effects) {
        fx.log(Level::Info, format_args!("authentication failed on the UE: 5GMM cause #{failure}"));
        match (failure, auts, self.challenge) {
            (cause::SYNCH_FAILURE, Some(auts), Some(c)) if !self.resynchronised => {
                self.resynchronised = true;
                self.authenticate(Some((c.rand, *auts)), fx);
            }
            _ => self.reject_authentication(fx),
        }
    }

    /// The AUSF's confirmation: K_SEAF, then the NAS security context.
    pub fn on_confirm_result(&mut self, config: &AmfConfig, result: &ConfirmResult, fx: &mut impl Effects) {
        if self.state != State::Confirming {
            return fx.log(Level::Debug, format_args!("late AUSF confirmation dropped"));
        }
        if result.outcome() != Outcome::Ok {
            fx.log(Level::Info, format_args!("authentication not confirmed: {:?}", result.outcome()));
            return self.reject_authentication(fx);
        }
        let (Some(imsi), Some(capability)) = (result.supi.imsi(), self.capability) else {
            return self.reject(cause::ILLEGAL_UE, fx);
        };
        let kamf = kdf::kamf(&result.kseaf, imsi, &ABBA);
        let ciphering = config.ciphering.iter().copied().find(|&c| capability.ciphering(c)).unwrap_or(NEA0);
        self.security = Some(SecurityContext::new(&kamf, self.ngksi, ciphering, NIA2));
        self.kamf = Some(kamf);
        self.supi = Some(result.supi);
        self.state = State::SecurityMode;
        fx.log(Level::Info, format_args!("{} authenticated, NEA{ciphering} NIA{NIA2}", result.supi));
        let command = Downlink::SecurityModeCommand {
            ciphering,
            integrity: NIA2,
            ngksi: self.ngksi,
            replayed: capability,
            // The initial message was not checked: ask for all of it (§4.4.6).
            retransmit_initial: true,
        };
        self.expect(command, fx);
    }

    /// Registration Accept; `from_idle`: the sessions with uplink data and
    /// those the UE holds, for a registration from idle (§5.5.1.3.4): the
    /// network's sessions are told back, those the UE lost released.
    fn accept(
        &mut self,
        config: &AmfConfig,
        tai: Option<Tai>,
        from_idle: Option<(SessionSet, Option<SessionSet>)>,
        fx: &mut impl Effects,
    ) {
        let Some(tai) = tai else {
            fx.log(Level::Error, format_args!("no NR location for the UE: registration refused"));
            return self.reject(cause::PROTOCOL_ERROR, fx);
        };
        // Requested slices the AMF serves; all of them if none is (no subscription data yet).
        let mut allowed = SnssaiList::default();
        self.requested.iter().filter(|s| config.slices.contains(s)).for_each(|s| allowed.push(s));
        if allowed.is_empty() {
            config.slices.iter().for_each(|&s| allowed.push(s));
        }
        let guti = Guti {
            plmn: config.plmn,
            amf_region: config.amf_region,
            amf_set: config.amf_set,
            amf_pointer: config.amf_pointer,
            tmsi: fx.tmsi(),
        };
        self.guti = Some(guti);
        self.allowed = allowed;
        self.state = State::Accepting;
        // As Open5GS and free5GC: the Accept goes in Initial Context Setup.
        let Some(context) = self.as_context() else { return self.reject(cause::PROTOCOL_ERROR, fx) };
        let held = from_idle.and_then(|(_, held)| held);
        let both = held.map(|h| SessionSet(fx.sessions().0 & h.0));
        let accept = Downlink::RegistrationAccept { guti, tai, allowed, sessions: both, t3512: config.t3512 };
        let mut pdu = [0u8; DOWNLINK_MAX + 7];
        if let Some(n) = self.seal(accept, &mut pdu, fx) {
            fx.setup_context(&pdu[..n], &context, "Registration Accept in Initial Context Setup");
        }
        if let Some((uplink_data, held)) = from_idle {
            let reactivate = SessionSet(uplink_data.0 & both.unwrap_or(fx.sessions()).0);
            fx.resume_sessions(reactivate, held);
        }
        // Sent again, if need be, in DL NAS Transport.
        self.retransmit = Some((accept, 0));
        fx.arm(T_NAS);
    }

    /// Sends `message`, which expects an answer: its timer runs.
    fn expect(&mut self, message: Downlink<'static>, fx: &mut impl Effects) {
        self.send(message, fx);
        self.retransmit = Some((message, 0));
        fx.arm(T_NAS);
    }

    /// The UE's NAS timer expired: the message again, or the procedure aborted.
    pub fn on_timer(&mut self, fx: &mut impl Effects) {
        let Some((message, sent)) = self.retransmit else { return };
        let (timer, what) = (timer_name(&message), name(message.kind()));
        if sent < RETRANSMISSIONS {
            fx.log(Level::Info, format_args!("{timer} expired: {what} again ({}/{RETRANSMISSIONS})", sent + 1));
            self.retransmit = Some((message, sent + 1));
            self.send(message, fx);
            return fx.arm(T_NAS);
        }
        self.retransmit = None;
        if let Downlink::RegistrationAccept { .. } = message {
            // The UE most likely has its 5G-GUTI (§5.5.1.2.8).
            fx.log(Level::Info, format_args!("{timer} expired 5 times: registered without Registration Complete"));
            self.state = State::Registered;
        } else if let Downlink::DeregistrationRequest { .. } = message {
            // The deregistration completes locally (§5.5.2.3.4).
            fx.log(Level::Info, format_args!("{timer} expired 5 times: deregistered by the network, locally"));
            self.state = State::Deregistered;
            fx.release(Release::Deregister);
        } else {
            fx.log(Level::Info, format_args!("{timer} expired 5 times: {what} unanswered, the UE is released"));
            self.state = State::Deregistered;
            fx.release(Release::Normal);
        }
    }

    /// The network deregisters the registered UE (§5.5.2.3); `reregister`:
    /// it must register again. False if the UE is in another procedure.
    pub fn deregister(&mut self, reregister: bool, fx: &mut impl Effects) -> bool {
        if self.state != State::Registered {
            return false;
        }
        let again = if reregister { ", re-registration required" } else { "" };
        fx.log(Level::Info, format_args!("deregistration by the network{again}"));
        self.state = State::Deregistering;
        self.expect(Downlink::DeregistrationRequest { reregister }, fx);
        true
    }

    /// The gNB's context: K_gNB from the uplink NAS COUNT of the message
    /// that triggers it (Security Mode Complete, an initial message).
    fn as_context(&self) -> Option<AsContext> {
        let (Some(kamf), Some(security), Some(capability)) = (self.kamf, &self.security, self.capability) else {
            return None;
        };
        Some(AsContext { kgnb: kdf::kgnb(&kamf, security.last_uplink_count()), capability, allowed: self.allowed })
    }

    /// A Service Request from idle, integrity checked (§5.6.1.4).
    fn on_service_request(&mut self, request: &ServiceRequest<'_>, fx: &mut impl Effects) {
        let sessions = fx.sessions();
        // Sessions with uplink data, and those it was paged for (downlink data).
        let reactivate = SessionSet(request.uplink_data.0 | fx.paged().0);
        let held = request.sessions;
        fx.log(Level::Info, format_args!("back from idle: Service Request (type {})", request.service_type));
        let Some(context) = self.as_context() else {
            self.send(Downlink::ServiceReject { cause: cause::PROTOCOL_ERROR }, fx);
            return fx.release(Release::Normal);
        };
        let held_by_both = SessionSet(sessions.0 & held.map_or(u16::MAX, |h| h.0));
        let mut pdu = [0u8; DOWNLINK_MAX + 7];
        if let Some(n) = self.seal(Downlink::ServiceAccept { sessions: held_by_both }, &mut pdu, fx) {
            fx.setup_context(&pdu[..n], &context, "Service Accept in Initial Context Setup");
        }
        fx.resume_sessions(SessionSet(reactivate.0 & held_by_both.0), held);
    }

    /// A UL NAS Transport of a registered UE: its 5GSM message to the SMF.
    fn on_transport(&mut self, t: &UlNasTransport<'_>, fx: &mut impl Effects) {
        if t.container_type != PAYLOAD_N1_SM {
            fx.log(Level::Info, format_args!("payload container type {} not served", t.container_type));
            return self.send(Downlink::Status { cause: cause::MESSAGE_TYPE_NOT_IMPLEMENTED }, fx);
        }
        let back = |this: &mut Self, psi: u8, cause: u8, fx: &mut _| {
            this.send(Downlink::DlNasTransport { pdu_session_id: psi, container: t.container, cause: Some(cause) }, fx)
        };
        let Some(psi) = t.pdu_session_id.filter(|p| (1..=15).contains(p)) else {
            fx.log(Level::Info, format_args!("5GSM message without a valid PDU session ID"));
            return back(self, t.pdu_session_id.unwrap_or(0), cause::PAYLOAD_NOT_FORWARDED, fx);
        };
        let snssai = match t.snssai {
            Some(s) if !self.allowed.iter().any(|a| a == s) => {
                fx.log(Level::Info, format_args!("PDU session {psi}: slice {s:?} not allowed"));
                return back(self, psi, cause::PAYLOAD_NOT_FORWARDED, fx);
            }
            Some(s) => s,
            None => {
                let first = self.allowed.iter().next();
                match first {
                    Some(s) => s,
                    None => return back(self, psi, cause::PAYLOAD_NOT_FORWARDED, fx),
                }
            }
        };
        let Some(supi) = self.supi else { return back(self, psi, cause::PAYLOAD_NOT_FORWARDED, fx) };
        let initial = t.request_type == Some(librecn_nas::mm::request::INITIAL);
        let request = SessionRequest { supi, pdu_session_id: psi, initial, snssai, dnn: t.dnn, container: t.container };
        if let Err(cause) = fx.session(&request) {
            back(self, psi, cause, fx);
        }
    }

    /// Sends a 5GSM message from the SMF to the UE (DL NAS Transport).
    pub fn send_session(&mut self, pdu_session_id: u8, container: &[u8], fx: &mut impl Effects) {
        self.send(Downlink::DlNasTransport { pdu_session_id, container, cause: None }, fx);
    }

    /// The protected DL NAS Transport of a 5GSM message, for the gNB to
    /// pass on (PDU Session Resource Setup); its length in `out`.
    pub fn seal_session(
        &mut self,
        pdu_session_id: u8,
        container: &[u8],
        out: &mut [u8],
        fx: &mut impl Effects,
    ) -> Option<usize> {
        self.seal(Downlink::DlNasTransport { pdu_session_id, container, cause: None }, out, fx)
    }

    fn reject(&mut self, cause: u8, fx: &mut impl Effects) {
        self.state = State::Deregistered;
        self.send(Downlink::RegistrationReject { cause }, fx);
        fx.release(Release::Normal);
    }

    fn reject_authentication(&mut self, fx: &mut impl Effects) {
        self.state = State::Deregistered;
        self.send(Downlink::AuthenticationReject, fx);
        fx.release(Release::AuthenticationFailure);
    }

    /// Sends `message`, protected once a security context exists.
    fn send(&mut self, message: Downlink<'_>, fx: &mut impl Effects) {
        let mut pdu = [0u8; DOWNLINK_MAX + 7];
        if let Some(n) = self.seal(message, &mut pdu, fx) {
            fx.send(&pdu[..n], name(message.kind()));
        }
    }

    /// Encodes `message` into `out`, protected once a security context
    /// exists; its length, or `None` (logged) if it does not fit.
    fn seal(&mut self, message: Downlink<'_>, out: &mut [u8], fx: &mut impl Effects) -> Option<usize> {
        let mut plain = [0u8; DOWNLINK_MAX];
        let what = name(message.kind());
        let Ok(n) = message.encode(&mut plain) else {
            fx.log(Level::Error, format_args!("{what} does not fit in {DOWNLINK_MAX} bytes"));
            return None;
        };
        let Some(ctx) = &mut self.security else {
            let out = out.get_mut(..n)?;
            out.copy_from_slice(&plain[..n]);
            return Some(n);
        };
        let header = match message {
            Downlink::SecurityModeCommand { .. } => Header::IntegrityNewContext,
            _ => Header::IntegrityCiphered,
        };
        match ctx.protect(&plain[..n], header, out) {
            Ok(len) => Some(len),
            Err(e) => {
                fx.log(Level::Error, format_args!("{what} not protected: {e}"));
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use librecn_nas::Plmn;
    use librecn_nas::security::NEA2;
    use librecn_security::milenage::Milenage;
    use librecn_security::{Direction, nea2, nia2};
    use zerocopy::FromZeros;

    const K: [u8; 16] =
        [0x46, 0x5b, 0x5c, 0xe8, 0xb1, 0x99, 0xb4, 0x9f, 0xaa, 0x5f, 0x0a, 0x2e, 0xe2, 0x38, 0xa6, 0xbc];
    const OPC: [u8; 16] =
        [0xe8, 0xed, 0x28, 0x9d, 0xeb, 0xa9, 0x52, 0xe4, 0x28, 0x3b, 0x54, 0xe8, 0x8e, 0x61, 0x83, 0xca];
    const SNN: &str = "5G:mnc070.mcc999.3gppnetwork.org";

    /// Records the effects.
    #[derive(Default)]
    struct Recorder {
        sent: Vec<Vec<u8>>,
        contexts: Vec<AsContext>,
        authenticate: Vec<(String, bool)>,
        confirm: Vec<(u32, [u8; 16])>,
        /// PDU session ID, initial, slice, DNN, container.
        sessions: Vec<(u8, bool, Snssai, String, Vec<u8>)>,
        /// What `session` answers.
        refuse_sessions: Option<u8>,
        /// What `sessions` answers, and what `resume_sessions` got.
        network_sessions: SessionSet,
        /// What `paged` answers.
        paged: SessionSet,
        resumed: Vec<(SessionSet, Option<SessionSet>)>,
        /// Whether the NAS timer runs.
        timer: bool,
        released: Option<Release>,
        logs: Vec<String>,
    }

    impl Effects for Recorder {
        fn send(&mut self, pdu: &[u8], _what: &'static str) {
            self.sent.push(pdu.to_vec());
        }
        fn setup_context(&mut self, pdu: &[u8], context: &AsContext, _what: &'static str) {
            self.sent.push(pdu.to_vec());
            self.contexts.push(*context);
        }
        fn sessions(&mut self) -> SessionSet {
            self.network_sessions
        }
        fn paged(&mut self) -> SessionSet {
            std::mem::take(&mut self.paged)
        }
        fn resume_sessions(&mut self, reactivate: SessionSet, held_by_ue: Option<SessionSet>) {
            self.resumed.push((reactivate, held_by_ue));
        }
        fn session(&mut self, r: &SessionRequest<'_>) -> Result<(), u8> {
            let dnn = r.dnn.map(|d| d.to_string()).unwrap_or_default();
            self.sessions.push((r.pdu_session_id, r.initial, r.snssai, dnn, r.container.to_vec()));
            self.refuse_sessions.map_or(Ok(()), Err)
        }
        fn authenticate(&mut self, who: &SupiOrSuci, resync: Option<([u8; 16], [u8; 14])>) -> bool {
            self.authenticate.push((who.to_string(), resync.is_some()));
            true
        }
        fn confirm(&mut self, auth_ctx: u32, res_star: &[u8; 16]) -> bool {
            self.confirm.push((auth_ctx, *res_star));
            true
        }
        fn release(&mut self, why: Release) {
            self.released = Some(why);
        }
        fn tmsi(&mut self) -> u32 {
            0xc0ffee
        }
        fn arm(&mut self, after: Duration) {
            assert_eq!(after, T_NAS);
            self.timer = true;
        }
        fn disarm(&mut self) {
            self.timer = false;
        }
        fn log(&mut self, _level: Level, args: fmt::Arguments<'_>) {
            self.logs.push(args.to_string());
        }
    }

    fn config() -> AmfConfig {
        AmfConfig::from_params("amf", &serde_json::Value::Null).unwrap()
    }

    fn tai() -> Option<Tai> {
        Some(Tai { plmn: Plmn::new("999", "70").unwrap(), tac: [0, 0, 1] })
    }

    fn hex(s: &str) -> Vec<u8> {
        let s: String = s.split_whitespace().collect();
        (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
    }

    /// UERANSIM's initial request: SUCI of imsi-999700000000001, all algorithms.
    fn registration_request() -> Vec<u8> {
        hex("7e004179 000d 0199f907 0000 0000 0000000010 2e04f0f0f0f0")
    }

    /// The UE side of the tests.
    struct Ue {
        k_int: [u8; 16],
        k_enc: [u8; 16],
        ul: u32,
    }

    impl Ue {
        /// RES* and K_AMF for a challenge (Authentication Request bytes).
        fn answer(&mut self, request: &[u8]) -> ([u8; 16], [u8; 32]) {
            let rand: [u8; 16] = request[8..24].try_into().unwrap();
            let m = Milenage::new(&K, &OPC);
            let v = m.f2345(&rand);
            let sqn_xor_ak: [u8; 6] = request[26..32].try_into().unwrap();
            let kausf = kdf::kausf(&v.ck, &v.ik, SNN, &sqn_xor_ak);
            let kamf = kdf::kamf(&kdf::kseaf(&kausf, SNN), "999700000000001", &ABBA);
            (self.k_enc, self.k_int) = kdf::nas_keys(&kamf, NEA2, NIA2);
            (kdf::res_star(&v.ck, &v.ik, SNN, &rand, &v.res), kamf)
        }

        /// An initial message from idle: `cleartext` IEs, the whole message
        /// ciphered in its NAS message container, integrity protected (§4.4.6).
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

        /// Protects and ciphers an uplink message.
        fn protect(&mut self, header: Header, plain: &[u8]) -> Vec<u8> {
            let mut pdu = [&[0x7e, header as u8, 0, 0, 0, 0, self.ul as u8][..], plain].concat();
            nea2(&self.k_enc, self.ul, 1, Direction::Uplink, &mut pdu[7..]);
            let mac = nia2(&self.k_int, self.ul, 1, Direction::Uplink, &pdu[6..]);
            pdu[2..6].copy_from_slice(&mac);
            self.ul += 1;
            pdu
        }

        /// Checks and deciphers a downlink PDU.
        fn open(&self, pdu: &[u8], count: u32) -> Vec<u8> {
            assert_eq!(nia2(&self.k_int, count, 1, Direction::Downlink, &pdu[6..]), pdu[2..6], "MAC");
            let mut plain = pdu[7..].to_vec();
            if pdu[1] == 2 {
                nea2(&self.k_enc, count, 1, Direction::Downlink, &mut plain);
            }
            plain
        }
    }

    /// The AUSF's side, computed here from the subscriber's keys.
    fn challenge(rand: [u8; 16]) -> (AuthenticateResult, ConfirmResult) {
        let m = Milenage::new(&K, &OPC);
        let sqn = [0, 0, 0, 0, 0, 0x20];
        let v = m.f2345(&rand);
        let sqn_xor_ak: [u8; 6] = std::array::from_fn(|i| sqn[i] ^ v.ak[i]);
        let mut result = AuthenticateResult::new_zeroed();
        result.auth_ctx = 9;
        result.rand = rand;
        result.autn[..6].copy_from_slice(&sqn_xor_ak);
        result.autn[6..8].copy_from_slice(&[0x80, 0]);
        result.autn[8..].copy_from_slice(&m.f1(&rand, &sqn, &[0x80, 0]).0);
        result.hxres_star = kdf::hres_star(&rand, &kdf::res_star(&v.ck, &v.ik, SNN, &rand, &v.res));
        let mut confirm = ConfirmResult::new_zeroed();
        confirm.supi = Supi::new("imsi-999700000000001").unwrap();
        confirm.kseaf = kdf::kseaf(&kdf::kausf(&v.ck, &v.ik, SNN, &sqn_xor_ak), SNN);
        (result, confirm)
    }

    #[test]
    fn a_ue_registers_with_5g_aka_and_nas_security() {
        let (config, mut mm, mut fx) = (config(), Mm::default(), Recorder::default());
        mm.on_uplink(&config, tai(), &mut registration_request(), &mut fx);
        assert_eq!(fx.authenticate, [("suci-0-999-70-0000-0-0-0000000001".to_string(), false)]);
        assert_eq!(mm.state(), State::Authenticating);

        let (result, confirm) = challenge([0x23; 16]);
        mm.on_authenticate_result(&result, &mut fx);
        let request = fx.sent.pop().unwrap();
        assert_eq!(request[..5], [0x7e, 0, 0x56, 0x00, 0x02], "plain Authentication Request, ngKSI 0, ABBA");
        let mut ue = Ue { k_int: [0; 16], k_enc: [0; 16], ul: 0 };
        let (res_star, kamf) = ue.answer(&request);

        let mut response = [&hex("7e0057 2d10")[..], &res_star].concat();
        mm.on_uplink(&config, tai(), &mut response, &mut fx);
        assert_eq!(fx.confirm, [(9, res_star)]);
        mm.on_confirm_result(&config, &confirm, &mut fx);
        assert_eq!(mm.state(), State::SecurityMode);
        let smc = fx.sent.pop().unwrap();
        assert_eq!(smc[1], 3, "integrity protected with the new context");
        assert_eq!(
            ue.open(&smc, 0),
            hex("7e005d 22 00 04f0f0f0f0 e1 360102"),
            "NEA2, NIA2, replayed capability, RINMR"
        );

        // The whole request comes back in Security Mode Complete, with the slices.
        let whole = hex("7e004179 000d 0199f907 0000 0000 0000000010 2e04f0f0f0f0 2f020101");
        let complete = [&hex("7e005e 71")[..], &(whole.len() as u16).to_be_bytes(), &whole].concat();
        mm.on_uplink(&config, tai(), &mut ue.protect(Header::IntegrityCipheredNewContext, &complete), &mut fx);
        let accept = ue.open(&fx.sent.pop().unwrap(), 1);
        assert_eq!(accept[..5], [0x7e, 0, 0x42, 0x01, 0x01], "Registration Accept over 3GPP access");
        assert!(accept.windows(4).any(|w| w == [0x00, 0xc0, 0xff, 0xee]), "the new 5G-TMSI");
        assert_eq!(accept[accept.len() - 7..], hex("1502 0101 5e01 06"), "allowed NSSAI: SST 1; T3512: 1 h");
        let context = fx.contexts.pop().expect("the Accept went in Initial Context Setup");
        assert_eq!(context.kgnb, kdf::kgnb(&kamf, 0), "from the count of Security Mode Complete");
        assert_eq!(context.capability.as_bytes(), [0xf0; 4]);

        mm.on_uplink(&config, tai(), &mut ue.protect(Header::IntegrityCiphered, &hex("7e0043")), &mut fx);
        assert_eq!(mm.state(), State::Registered);
        assert_eq!(mm.supi().unwrap().as_str(), "imsi-999700000000001");
        assert!(
            fx.logs.iter().any(|l| l.starts_with("registered: imsi-999700000000001, 5G-GUTI 999/70 1:1:0 00c0ffee"))
        );
        assert_eq!(fx.released, None, "the UE stays connected");

        // A PDU session request goes to the SMF, the default slice filled in.
        let n1 = hex("2e0101c1ffff91");
        let transport = |psi: &str, n1: &[u8]| {
            [&hex("7e0067 01")[..], &(n1.len() as u16).to_be_bytes(), n1, &hex(psi), &hex("81 2509 08696e7465726e6574")]
                .concat()
        };
        mm.on_uplink(&config, tai(), &mut ue.protect(Header::IntegrityCiphered, &transport("1201", &n1)), &mut fx);
        let sst1 = Snssai { sst: 1, sd: None };
        assert_eq!(fx.sessions.pop(), Some((1, true, sst1, "internet".into(), n1.clone())));
        // Refused by the AMF: the 5GSM message goes back with #90.
        fx.refuse_sessions = Some(cause::PAYLOAD_NOT_FORWARDED);
        mm.on_uplink(&config, tai(), &mut ue.protect(Header::IntegrityCiphered, &transport("1202", &n1)), &mut fx);
        let back = ue.open(&fx.sent.pop().unwrap(), 2);
        assert_eq!(back, [&hex("7e0068 01 0007")[..], &n1, &hex("1202 585a")].concat());
        // Without a PDU session ID: back too, never to the SMF.
        mm.on_uplink(&config, tai(), &mut ue.protect(Header::IntegrityCiphered, &transport("", &n1)), &mut fx);
        assert_eq!(ue.open(&fx.sent.pop().unwrap(), 3)[..3], hex("7e0068")[..]);
        assert!(fx.sessions.len() == 1, "only the refused one was asked");

        // Plain messages are now refused, protected ones with a bad MAC discarded.
        mm.on_uplink(&config, tai(), &mut hex("7e0043"), &mut fx);
        assert!(fx.logs.last().unwrap().contains("not integrity protected"));
        let mut tampered = ue.protect(Header::IntegrityCiphered, &hex("7e0045 01 000b f299f907010040 00c0ffee"));
        tampered[3] ^= 1;
        mm.on_uplink(&config, tai(), &mut tampered, &mut fx);
        assert!(fx.logs.last().unwrap().contains("integrity check failed"));

        ue.ul -= 1;
        let dereg = ue.protect(Header::IntegrityCiphered, &hex("7e0045 01 000b f299f907010040 00c0ffee"));
        mm.on_uplink(&config, tai(), &mut dereg.clone(), &mut fx);
        assert_eq!(ue.open(&fx.sent.pop().unwrap(), 4), hex("7e0046"), "Deregistration Accept");
        assert_eq!((mm.state(), fx.released), (State::Deregistered, Some(Release::Deregister)));
    }

    #[test]
    fn a_wrong_res_star_gets_an_authentication_reject() {
        let (config, mut mm, mut fx) = (config(), Mm::default(), Recorder::default());
        mm.on_uplink(&config, tai(), &mut registration_request(), &mut fx);
        mm.on_authenticate_result(&challenge([1; 16]).0, &mut fx);
        mm.on_uplink(&config, tai(), &mut [&hex("7e0057 2d10")[..], &[0; 16]].concat(), &mut fx);
        assert_eq!(fx.sent.last().unwrap(), &hex("7e0058"));
        assert!(fx.confirm.is_empty(), "the home network is not asked");
        assert_eq!(fx.released, Some(Release::AuthenticationFailure));
    }

    #[test]
    fn a_synch_failure_resynchronises_once() {
        let (config, mut mm, mut fx) = (config(), Mm::default(), Recorder::default());
        mm.on_uplink(&config, tai(), &mut registration_request(), &mut fx);
        mm.on_authenticate_result(&challenge([1; 16]).0, &mut fx);
        let failure = [&hex("7e0059 15 300e")[..], &[7; 14]].concat();
        mm.on_uplink(&config, tai(), &mut failure.clone(), &mut fx);
        assert!(fx.authenticate.last().unwrap().1, "resynchronisation info sent");
        mm.on_authenticate_result(&challenge([2; 16]).0, &mut fx);
        mm.on_uplink(&config, tai(), &mut failure.clone(), &mut fx);
        assert_eq!(fx.sent.last().unwrap(), &hex("7e0058"), "a second one is a failure");
    }

    #[test]
    fn registrations_are_refused_or_identified_up_front() {
        let config = config();
        let run = |pdu: &str| {
            let (mut mm, mut fx) = (Mm::default(), Recorder::default());
            mm.on_uplink(&config, tai(), &mut hex(pdu), &mut fx);
            (fx.sent.pop(), fx.released, fx.authenticate.len())
        };
        // A GUTI the AMF does not know: identification, no release.
        let guti = run("7e004101 000b f299f907010040 01020304 2e02f0f0");
        assert_eq!(guti, (Some(hex("7e005b01")), None, 0));
        // No NIA2: #23. No capability: #96. An IMEI: #3.
        assert_eq!(run("7e004101 000d 0199f907000000000000000010 2e02f0d0").0, Some(hex("7e004417")));
        assert_eq!(run("7e004101 000d 0199f907000000000000000010").0, Some(hex("7e004460")));
        assert_eq!(run("7e004101 0008 4b09101010101010 2e02f0f0").0, Some(hex("7e004403")));
        // Not 5GMM-decodable: a 5GMM Status.
        assert_eq!(run("7e0099").0, Some(hex("7e006461")));
    }

    #[test]
    fn an_identity_response_leads_to_authentication() {
        let (config, mut mm, mut fx) = (config(), Mm::default(), Recorder::default());
        mm.on_uplink(&config, tai(), &mut hex("7e004101 000b f299f907010040 01020304 2e02f0f0"), &mut fx);
        mm.on_uplink(&config, tai(), &mut hex("7e005c 000d 0199f907000000000000000010"), &mut fx);
        assert_eq!(fx.authenticate, [("suci-0-999-70-0000-0-0-0000000001".to_string(), false)]);
        let mut refused = AuthenticateResult::new_zeroed();
        refused.set_outcome(Outcome::UserNotFound);
        mm.on_authenticate_result(&refused, &mut fx);
        assert_eq!(fx.sent.last().unwrap(), &hex("7e004403"), "Registration Reject, illegal UE");
    }

    /// A UE registered, as `a_ue_registers_with_5g_aka_and_nas_security`
    /// leaves it; the AMF's next downlink count is 2.
    fn registered() -> (AmfConfig, Mm, Recorder, Ue) {
        let (config, mut mm, mut fx) = (config(), Mm::default(), Recorder::default());
        mm.on_uplink(&config, tai(), &mut registration_request(), &mut fx);
        let (result, confirm) = challenge([0x23; 16]);
        mm.on_authenticate_result(&result, &mut fx);
        let mut ue = Ue { k_int: [0; 16], k_enc: [0; 16], ul: 0 };
        let (res_star, _) = ue.answer(&fx.sent.pop().unwrap());
        mm.on_uplink(&config, tai(), &mut [&hex("7e0057 2d10")[..], &res_star].concat(), &mut fx);
        mm.on_confirm_result(&config, &confirm, &mut fx);
        let whole = hex("7e004179 000d 0199f907 0000 0000 0000000010 2e04f0f0f0f0 2f020101");
        let complete = [&hex("7e005e 71")[..], &(whole.len() as u16).to_be_bytes(), &whole].concat();
        mm.on_uplink(&config, tai(), &mut ue.protect(Header::IntegrityCipheredNewContext, &complete), &mut fx);
        mm.on_uplink(&config, tai(), &mut ue.protect(Header::IntegrityCiphered, &hex("7e0043")), &mut fx);
        assert_eq!(mm.state(), State::Registered);
        (config, mm, Recorder::default(), ue)
    }

    /// The Service Request of an idle UE: data for PDU session 1, holding 1.
    const SERVICE_REQUEST: &str = "7e004c 10 0007 f4004000c0ffee 4002 0200 5002 0200";
    const SERVICE_CLEARTEXT: &str = "7e004c 10 0007 f4004000c0ffee";

    #[test]
    fn an_idle_ue_comes_back_with_a_service_request() {
        let (config, mut mm, mut fx, mut ue) = registered();
        assert_eq!(initial_tmsi(&ue.initial(&hex(SERVICE_CLEARTEXT), &hex(SERVICE_REQUEST)), &config), Some(0xc0ffee));
        ue.ul -= 1;
        fx.network_sessions = SessionSet(0x06); // sessions 1 and 2; the UE holds only 1
        let mut request = ue.initial(&hex(SERVICE_CLEARTEXT), &hex(SERVICE_REQUEST));
        mm.on_uplink(&config, tai(), &mut request, &mut fx);
        assert_eq!(ue.open(&fx.sent.pop().unwrap(), 2), hex("7e004e 50020200"), "Service Accept: session 1");
        let context = fx.contexts.pop().expect("in Initial Context Setup");
        let kamf = mm.kamf.unwrap();
        assert_eq!(context.kgnb, kdf::kgnb(&kamf, 2), "from the Service Request's count");
        assert_eq!(fx.resumed, [(SessionSet(0x02), Some(SessionSet(0x02)))], "reactivate 1, 2 is gone at the UE");
        assert_eq!((mm.state(), fx.released), (State::Registered, None));
    }

    #[test]
    fn a_paged_ue_gets_its_paged_sessions_set_up_again() {
        let (config, mut mm, mut fx, mut ue) = registered();
        fx.network_sessions = SessionSet(0x06);
        fx.paged = SessionSet(0x04);
        // Mobile terminated services, no uplink data, holding sessions 1 and 2.
        let cleartext = hex("7e004c 20 0007 f4004000c0ffee");
        let whole = hex("7e004c 20 0007 f4004000c0ffee 5002 0600");
        mm.on_uplink(&config, tai(), &mut ue.initial(&cleartext, &whole), &mut fx);
        assert_eq!(ue.open(&fx.sent.pop().unwrap(), 2), hex("7e004e 50020600"), "Service Accept: 1 and 2");
        assert_eq!(fx.resumed, [(SessionSet(0x04), Some(SessionSet(0x06)))], "the paged session only");
    }

    #[test]
    fn a_service_request_that_fails_the_integrity_check_is_rejected() {
        let (config, mut mm, mut fx, mut ue) = registered();
        let mut request = ue.initial(&hex(SERVICE_CLEARTEXT), &hex(SERVICE_REQUEST));
        request[3] ^= 1;
        mm.on_uplink(&config, tai(), &mut request, &mut fx);
        assert_eq!(ue.open(&fx.sent.pop().unwrap(), 2), hex("7e004d09"), "Service Reject #9");
        assert_eq!(fx.released, Some(Release::Normal));
        assert!(fx.resumed.is_empty() && fx.contexts.is_empty());
    }

    #[test]
    fn a_registration_from_idle_needs_no_new_authentication() {
        let (config, mut mm, mut fx, mut ue) = registered();
        // Mobility registration with the 5G-GUTI, slices in the container only.
        let cleartext = hex("7e004102 000b f299f907 010040 00c0ffee 2e04f0f0f0f0");
        let whole = hex("7e004102 000b f299f907 010040 00c0ffee 2e04f0f0f0f0 2f020101");
        let mut request = ue.initial(&cleartext, &whole);
        assert_eq!(initial_tmsi(&request, &config), Some(0xc0ffee));
        mm.on_uplink(&config, tai(), &mut request, &mut fx);
        assert!(fx.authenticate.is_empty(), "no AUSF round trip");
        let accept = ue.open(&fx.sent.pop().unwrap(), 2);
        assert_eq!(accept[..3], hex("7e0042")[..], "Registration Accept");
        assert_eq!(fx.contexts.len(), 1);
        mm.on_uplink(&config, tai(), &mut ue.protect(Header::IntegrityCiphered, &hex("7e0043")), &mut fx);
        assert_eq!(mm.state(), State::Registered);
        assert_eq!(fx.released, Some(Release::Normal), "no follow-on request: back to idle");

        // Periodic, with a follow-on request (data to send): the UE stays connected.
        let mut fx = Recorder::default();
        let periodic = hex("7e00410b 000b f299f907 010040 00c0ffee 2e04f0f0f0f0");
        mm.on_uplink(&config, tai(), &mut ue.initial(&periodic, &periodic), &mut fx);
        mm.on_uplink(&config, tai(), &mut ue.protect(Header::IntegrityCiphered, &hex("7e0043")), &mut fx);
        assert_eq!((mm.state(), fx.released), (State::Registered, None));

        // A Registration Request the kept context does not check: a full registration.
        let mut forged = ue.initial(&cleartext, &whole);
        forged[4] ^= 1;
        mm.on_uplink(&config, tai(), &mut forged, &mut fx);
        assert_eq!(fx.sent.pop().unwrap(), hex("7e005b01"), "Identity Request for the SUCI, in clear");
        assert!(initial_tmsi(&hex("7e004101 000b f299f907 010041 00c0ffee"), &config).is_none(), "another AMF");
    }

    #[test]
    fn a_registration_from_idle_tells_the_sessions_back_and_releases_the_lost_ones() {
        let (config, mut mm, mut fx, mut ue) = registered();
        fx.network_sessions = SessionSet(0x06);
        // Mobility registration holding session 1 only (PDU session status).
        let cleartext = hex("7e00410a 000b f299f907 010040 00c0ffee 2e04f0f0f0f0");
        let whole = hex("7e00410a 000b f299f907 010040 00c0ffee 2e04f0f0f0f0 5002 0200");
        mm.on_uplink(&config, tai(), &mut ue.initial(&cleartext, &whole), &mut fx);
        let accept = ue.open(&fx.sent.pop().unwrap(), 2);
        assert!(accept.windows(4).any(|w| w == hex("50020200")), "the network's sessions the UE holds: 1");
        assert_eq!(fx.resumed, [(SessionSet(0), Some(SessionSet(0x02)))], "nothing to reactivate, session 2 released");
    }

    /// The NAS timer fires, as the EO delivers it: no longer running.
    fn expire(mm: &mut Mm, fx: &mut Recorder) {
        assert!(fx.timer, "the timer runs");
        fx.timer = false;
        mm.on_timer(fx);
    }

    #[test]
    fn an_unanswered_authentication_request_is_sent_four_times_more_then_given_up() {
        let (config, mut mm, mut fx) = (config(), Mm::default(), Recorder::default());
        mm.on_uplink(&config, tai(), &mut registration_request(), &mut fx);
        mm.on_authenticate_result(&challenge([1; 16]).0, &mut fx);
        let first = fx.sent.pop().unwrap();
        assert!(fx.timer, "T3560 runs");
        for _ in 0..4 {
            expire(&mut mm, &mut fx);
            assert_eq!(fx.sent.pop().unwrap(), first, "the same challenge: the UE answers the same RES*");
            assert!(fx.timer);
        }
        expire(&mut mm, &mut fx);
        assert!(fx.sent.is_empty() && !fx.timer);
        assert_eq!((mm.state(), fx.released), (State::Deregistered, Some(Release::Normal)));
        assert!(fx.logs.iter().any(|l| l.contains("T3560 expired 5 times")));
    }

    #[test]
    fn an_answer_stops_the_timer_and_a_lost_registration_complete_still_registers() {
        let (config, mut mm, mut fx) = (config(), Mm::default(), Recorder::default());
        mm.on_uplink(&config, tai(), &mut hex("7e004101 000b f299f907010040 01020304 2e02f0f0"), &mut fx);
        assert!(fx.timer, "T3570 for the Identity Request");
        expire(&mut mm, &mut fx);
        assert_eq!(fx.sent.pop().unwrap(), hex("7e005b01"), "sent again");
        mm.on_uplink(&config, tai(), &mut hex("7e005c 000d 0199f907000000000000000010"), &mut fx);
        assert!(!fx.timer, "answered");

        let (config, mut mm, _, mut ue) = registered();
        let mut fx = Recorder::default();
        let cleartext = hex("7e004102 000b f299f907 010040 00c0ffee 2e04f0f0f0f0");
        mm.on_uplink(&config, tai(), &mut ue.initial(&cleartext, &cleartext), &mut fx);
        assert_eq!(mm.state(), State::Accepting);
        fx.sent.clear();
        for n in 0..4 {
            expire(&mut mm, &mut fx);
            assert_eq!(ue.open(&fx.sent.pop().unwrap(), 3 + n)[..3], hex("7e0042")[..], "T3550: the Accept again");
        }
        expire(&mut mm, &mut fx);
        assert_eq!((mm.state(), fx.released, fx.timer), (State::Registered, None, false));
    }

    #[test]
    fn the_network_deregisters_a_ue_which_accepts_or_never_answers() {
        let (config, mut mm, mut fx, mut ue) = registered();
        assert!(mm.deregister(true, &mut fx));
        assert_eq!(ue.open(&fx.sent.pop().unwrap(), 2), hex("7e004705"), "re-registration required, 3GPP");
        assert!(!mm.deregister(true, &mut fx), "already deregistering");
        assert_eq!((mm.state(), fx.released, fx.timer), (State::Deregistering, None, true), "T3522 runs");
        mm.on_uplink(&config, tai(), &mut ue.protect(Header::IntegrityCiphered, &hex("7e0048")), &mut fx);
        assert_eq!((mm.state(), fx.released, fx.timer), (State::Deregistered, Some(Release::Deregister), false));

        let (_, mut mm, mut fx, _) = registered();
        mm.deregister(false, &mut fx);
        for _ in 0..4 {
            expire(&mut mm, &mut fx);
        }
        assert_eq!(fx.sent.len(), 5, "sent, then four times again");
        expire(&mut mm, &mut fx);
        assert_eq!((mm.state(), fx.released), (State::Deregistered, Some(Release::Deregister)), "locally");
    }

    /// The selected algorithms octet of the Security Mode Command, for a UE
    /// capability and the AMF's preference.
    fn selected(capability: &str, ciphering: &str) -> u8 {
        let params = serde_json::json!({ "ciphering": ciphering.split(',').collect::<Vec<_>>() });
        let config = AmfConfig::from_params("amf", &params).unwrap();
        let (mut mm, mut fx) = (Mm::default(), Recorder::default());
        let request = format!("7e004179 000d 0199f907 0000 0000 0000000010 2e02{capability}");
        mm.on_uplink(&config, tai(), &mut hex(&request), &mut fx);
        let (result, confirm) = challenge([3; 16]);
        mm.on_authenticate_result(&result, &mut fx);
        mm.state = State::Confirming;
        mm.on_confirm_result(&config, &confirm, &mut fx);
        fx.sent.last().unwrap()[10]
    }

    #[test]
    fn ciphering_follows_the_preference_within_the_ue_capability() {
        assert_eq!(selected("f0f0", "NEA2,NEA0"), 0x22);
        assert_eq!(selected("f0f0", "NEA0,NEA2"), 0x02, "NEA0 first: no ciphering");
        assert_eq!(selected("80f0", "NEA2"), 0x02, "the UE lacks NEA2: NEA0, always supported");
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(500))]

        /// Whatever the UE sends, at whatever point of the registration.
        #[test]
        fn random_uplinks_never_panic(
            stage in 0..4usize,
            k in 0..PLAIN_ALLOWED.len(),
            header in 0..5u8,
            tail in proptest::collection::vec(proptest::prelude::any::<u8>(), 0..40),
        ) {
            let (config, mut mm, mut fx) = (config(), Mm::default(), Recorder::default());
            let (result, confirm) = challenge([3; 16]);
            if stage > 0 {
                mm.on_uplink(&config, tai(), &mut registration_request(), &mut fx);
            }
            if stage > 1 {
                mm.on_authenticate_result(&result, &mut fx);
            }
            if stage > 2 {
                mm.state = State::Confirming;
                mm.on_confirm_result(&config, &confirm, &mut fx);
            }
            let mut pdu = [&[0x7e, header][..], &tail].concat();
            mm.on_uplink(&config, tai(), &mut pdu, &mut fx);
            let mut plain = [&[0x7e, 0, PLAIN_ALLOWED[k]][..], &tail].concat();
            mm.on_uplink(&config, None, &mut plain, &mut fx);
        }
    }
}
