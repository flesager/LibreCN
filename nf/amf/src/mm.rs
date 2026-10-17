//! 5GMM for one UE (TS 24.501 §5): registration with 5G AKA, NAS security
//! mode control, identification and UE-originating deregistration.
//!
//! Pure logic on domain values: the EO hands in the UE's uplink NAS PDUs
//! and the AUSF's answers; this module acts through [`Effects`].
//!
//! ```text
//! Registration Request (SUCI) ─▶ AUSF ─▶ Authentication Request
//! Authentication Response (RES*, checked against HXRES*) ─▶ AUSF confirms ─▶ K_AMF
//! Security Mode Command (new context) ◀▶ Security Mode Complete (whole request)
//! Registration Accept (5G-GUTI) ◀▶ Registration Complete: registered
//! ```
//!
//! A 5G-GUTI the AMF does not know (this release keeps no context once the
//! UE leaves) is answered by an Identity Request for the SUCI. Retransmission
//! timers (T3550, T3560, T3570) are not run yet: a UE that stops answering
//! keeps its context until the gNB releases it.

use std::fmt;

use librecn_common::log::Level;
use librecn_nas::ie::{Guti, IdentityType, MobileIdentity, NgKsi, SnssaiList, Tai, UeSecurityCapability};
use librecn_nas::mm::{ABBA, Downlink, RegistrationKind, Uplink, cause, kind, name};
use librecn_nas::security::{self, Header, NEA0, NIA2, SecurityContext};
use librecn_sbi::ue_authentication::{AuthenticateResult, ConfirmResult};
use librecn_sbi::{Outcome, Supi, SupiOrSuci};
use librecn_security::kdf;

use crate::config::AmfConfig;

/// Largest plain NAS message the AMF sends.
const DOWNLINK_MAX: usize = 256;

/// Why the AMF releases the UE's N2 context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Release {
    /// Registration rejected, or 5GMM done with the UE.
    Normal,
    Deregister,
    AuthenticationFailure,
}

/// What 5GMM needs from the AMF.
pub trait Effects {
    /// Sends a NAS PDU (already protected) to the UE; `what` names it for logs.
    fn send(&mut self, pdu: &[u8], what: &'static str);
    /// Asks the AUSF to authenticate (with RAND and AUTS after a synch
    /// failure). False if the AUSF cannot be reached.
    fn authenticate(&mut self, who: &SupiOrSuci, resync: Option<([u8; 16], [u8; 14])>) -> bool;
    /// Confirms 5G AKA with the UE's RES*. False if the AUSF cannot be reached.
    fn confirm(&mut self, auth_ctx: u32, res_star: &[u8; 16]) -> bool;
    /// Releases the UE's N2 context, after the PDUs already sent.
    fn release(&mut self, why: Release);
    /// A fresh, unpredictable 5G-TMSI.
    fn tmsi(&mut self) -> u32;
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
}

/// The challenge sent to the UE.
#[derive(Clone, Copy, Debug)]
struct Challenge {
    auth_ctx: u32,
    rand: [u8; 16],
    hxres_star: [u8; 16],
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
    supi: Option<Supi>,
    guti: Option<Guti>,
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

    /// An uplink NAS PDU; `tai` is where the UE is. `pdu` is deciphered in place.
    pub fn on_uplink(&mut self, config: &AmfConfig, tai: Option<Tai>, pdu: &mut [u8], fx: &mut impl Effects) {
        let Some(header) = security::header(pdu) else {
            return fx.log(Level::Info, format_args!("uplink {} dropped", librecn_nas::describe(pdu)));
        };
        let protected = header != Header::Plain as u8;
        let plain: &[u8] = match (&mut self.security, protected) {
            (Some(ctx), true) => match ctx.unprotect(pdu) {
                Ok(plain) => plain,
                Err(e) => return fx.log(Level::Info, format_args!("uplink discarded: {e}")),
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
        let security = if protected { "protected" } else { "plain" };
        fx.log(Level::Info, format_args!("uplink {} ({security})", name(message.kind())));
        self.on_message(config, tai, message, fx);
    }

    fn on_message(&mut self, config: &AmfConfig, tai: Option<Tai>, message: Uplink<'_>, fx: &mut impl Effects) {
        match (self.state, message) {
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
                self.accept(config, tai, fx);
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
            (_, Uplink::Status { cause }) => {
                fx.log(Level::Info, format_args!("5GMM Status from the UE: cause #{cause}"))
            }
            // The AMF keeps no context for an idle UE: it must register again (§5.6.1.5).
            (_, Uplink::Other(kind::SERVICE_REQUEST)) => {
                self.send(Downlink::ServiceReject { cause: cause::UE_IDENTITY_CANNOT_BE_DERIVED }, fx);
                fx.release(Release::Normal);
            }
            (_, Uplink::Other(_)) => self.send(Downlink::Status { cause: cause::MESSAGE_TYPE_NOT_IMPLEMENTED }, fx),
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
                self.send(Downlink::IdentityRequest(IdentityType::Suci), fx);
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
        self.send(Downlink::AuthenticationRequest { ngksi: self.ngksi, rand, autn }, fx);
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
        self.send(command, fx);
    }

    fn accept(&mut self, config: &AmfConfig, tai: Option<Tai>, fx: &mut impl Effects) {
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
        self.state = State::Accepting;
        self.send(Downlink::RegistrationAccept { guti, tai, allowed }, fx);
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

    /// Encodes `message`, protected once a security context exists.
    fn send(&mut self, message: Downlink, fx: &mut impl Effects) {
        let mut plain = [0u8; DOWNLINK_MAX];
        let what = name(message.kind());
        let Ok(n) = message.encode(&mut plain) else {
            return fx.log(Level::Error, format_args!("{what} does not fit in {DOWNLINK_MAX} bytes"));
        };
        let Some(ctx) = &mut self.security else { return fx.send(&plain[..n], what) };
        let header = match message {
            Downlink::SecurityModeCommand { .. } => Header::IntegrityNewContext,
            _ => Header::IntegrityCiphered,
        };
        let mut protected = [0u8; DOWNLINK_MAX + 7];
        match ctx.protect(&plain[..n], header, &mut protected) {
            Ok(len) => fx.send(&protected[..len], what),
            Err(e) => fx.log(Level::Error, format_args!("{what} not protected: {e}")),
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
        authenticate: Vec<(String, bool)>,
        confirm: Vec<(u32, [u8; 16])>,
        released: Option<Release>,
        logs: Vec<String>,
    }

    impl Effects for Recorder {
        fn send(&mut self, pdu: &[u8], _what: &'static str) {
            self.sent.push(pdu.to_vec());
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
        let (res_star, _) = ue.answer(&request);

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
        assert_eq!(accept[accept.len() - 4..], [0x15, 0x02, 0x01, 0x01], "allowed NSSAI: SST 1");

        mm.on_uplink(&config, tai(), &mut ue.protect(Header::IntegrityCiphered, &hex("7e0043")), &mut fx);
        assert_eq!(mm.state(), State::Registered);
        assert_eq!(mm.supi().unwrap().as_str(), "imsi-999700000000001");
        assert!(
            fx.logs.iter().any(|l| l.starts_with("registered: imsi-999700000000001, 5G-GUTI 999/70 1:1:0 00c0ffee"))
        );
        assert_eq!(fx.released, None, "the UE stays connected");

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
        assert_eq!(ue.open(&fx.sent.pop().unwrap(), 2), hex("7e0046"), "Deregistration Accept");
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
