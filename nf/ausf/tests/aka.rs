//! 5G AKA through the AUSF and the UDM EOs, an AMF probe playing the SEAF
//! and computing what the UE would.

use std::time::Duration;

use librecn_ausf::{Ausf, LIFETIME};
use librecn_oam::{CapturedLog, LogSink};
use librecn_sbi::ue_authentication::{AuthInfo, Authenticate, AuthenticateResult, Confirm, ConfirmResult, method};
use librecn_sbi::{Outcome, ServingNetwork, SupiOrSuci};
use librecn_scheduler::testbed::{Probe, ProbeLog, TestBed};
use librecn_scheduler::{EoConfig, EoId};
use librecn_security::milenage::Milenage;
use librecn_security::{eap_aka, kdf};
use librecn_udm::Udm;
use zerocopy::FromZeros;

const K: &str = "465B5CE8B199B49FAA5F0A2EE238A6BC";
const OPC: &str = "E8ED289DEBA952E4283B54E88E6183CA";
const SNN: &str = "5G:mnc070.mcc999.3gppnetwork.org";
const SUCI: &str = "suci-0-999-70-0000-0-0-0000000001";

fn hex<const N: usize>(s: &str) -> [u8; N] {
    std::array::from_fn(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap())
}

struct Core {
    tb: TestBed,
    log: CapturedLog,
    amf: EoId,
    ausf: EoId,
    seen: ProbeLog,
}

fn core() -> Core {
    core_with("5g-aka")
}

/// The same, the subscriber authenticated with `auth`.
fn core_with(auth: &str) -> Core {
    let mut tb = TestBed::new();
    let (sink, log) = LogSink::capture();
    tb.add(LogSink::config(), sink).unwrap();
    let subscribers =
        serde_json::json!({ "subscribers": [{ "supi": "imsi-999700000000001", "k": K, "opc": OPC, "auth": auth }] });
    let (cfg, udm) = Udm::from_params("udm", &subscribers).unwrap();
    tb.add(cfg, udm).unwrap();
    let (cfg, ausf) = Ausf::from_params("ausf", &serde_json::Value::Null).unwrap();
    let ausf = tb.add(cfg, ausf).unwrap();
    let (probe, seen) = Probe::new();
    let amf = tb.add(EoConfig::new("amf").buffers(&[(256, 16)]), probe).unwrap();
    tb.run_until_idle();
    Core { tb, log, amf, ausf, seen }
}

/// What the UE computes from a challenge: RES*, and K_SEAF to compare.
fn ue(challenge: &AuthenticateResult, sqn_ms: &[u8; 6]) -> Result<([u8; 16], [u8; 32]), [u8; 14]> {
    let m = Milenage::new(&hex(K), &hex(OPC));
    let v = m.f2345(&challenge.rand);
    let sqn_xor_ak: [u8; 6] = challenge.autn[..6].try_into().unwrap();
    let sqn: [u8; 6] = std::array::from_fn(|i| sqn_xor_ak[i] ^ v.ak[i]);
    let amf: [u8; 2] = challenge.autn[6..8].try_into().unwrap();
    assert_eq!(m.f1(&challenge.rand, &sqn, &amf).0, challenge.autn[8..], "network authenticated");
    if sqn <= *sqn_ms {
        // Synch failure: AUTS = SQN_MS ⊕ AK* || MAC-S.
        let ak = m.f5_star(&challenge.rand);
        let mut auts = [0u8; 14];
        for i in 0..6 {
            auts[i] = sqn_ms[i] ^ ak[i];
        }
        auts[6..].copy_from_slice(&m.f1(&challenge.rand, sqn_ms, &[0, 0]).1);
        return Err(auts);
    }
    let res_star = kdf::res_star(&v.ck, &v.ik, SNN, &challenge.rand, &v.res);
    assert_eq!(kdf::hres_star(&challenge.rand, &res_star), challenge.hxres_star);
    Ok((res_star, kdf::kseaf(&kdf::kausf(&v.ck, &v.ik, SNN, &sqn_xor_ak), SNN)))
}

impl Core {
    fn authenticate(&mut self, info: AuthInfo) -> AuthenticateResult {
        self.tb.inject_from(self.amf, self.ausf, 7, &Authenticate { info }).unwrap();
        self.tb.run_until_idle();
        let got = self.seen.take();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].header.corr_id, 7, "the AMF's correlation id comes back");
        got[0].body().unwrap()
    }

    fn confirm(&mut self, auth_ctx: u32, res_star: [u8; 16]) -> ConfirmResult {
        let mut confirm = Confirm::new_zeroed();
        (confirm.auth_ctx, confirm.res_star) = (auth_ctx, res_star);
        self.tb.inject_from(self.amf, self.ausf, 8, &confirm).unwrap();
        self.tb.run_until_idle();
        self.seen.take()[0].body().unwrap()
    }

    fn confirm_eap(&mut self, auth_ctx: u32, eap: &[u8]) -> ConfirmResult {
        let mut confirm = Confirm::new_zeroed();
        confirm.auth_ctx = auth_ctx;
        confirm.eap.set(eap);
        self.tb.inject_from(self.amf, self.ausf, 8, &confirm).unwrap();
        self.tb.run_until_idle();
        self.seen.take()[0].body().unwrap()
    }
}

fn info(who: &str) -> AuthInfo {
    AuthInfo::new(SupiOrSuci::new(who).unwrap(), ServingNetwork::new(SNN).unwrap())
}

#[test]
fn a_ue_is_authenticated_and_the_amf_gets_kseaf_and_the_supi() {
    let mut c = core();
    let challenge = c.authenticate(info(SUCI));
    assert_eq!(challenge.outcome(), Outcome::Ok);
    let (res_star, kseaf) = ue(&challenge, &[0; 6]).unwrap();

    let result = c.confirm(challenge.auth_ctx, res_star);
    assert_eq!((result.outcome(), result.supi.as_str()), (Outcome::Ok, "imsi-999700000000001"));
    assert_eq!(result.kseaf, kseaf);
    assert!(c.log.lines().iter().any(|l| l.eo == "ausf" && l.msg == "imsi-999700000000001 authenticated"));
    assert_eq!(c.confirm(challenge.auth_ctx, res_star).outcome(), Outcome::ContextNotFound, "used once");
}

#[test]
fn a_wrong_res_star_is_rejected_without_keys() {
    let mut c = core();
    let challenge = c.authenticate(info(SUCI));
    let result = c.confirm(challenge.auth_ctx, [0; 16]);
    assert_eq!(result.outcome(), Outcome::AuthenticationRejected);
    assert_eq!((result.kseaf, result.supi.as_str()), ([0; 32], ""));
}

#[test]
fn unknown_subscribers_and_networks_are_refused() {
    let mut c = core();
    assert_eq!(c.authenticate(info("suci-0-999-70-0000-0-0-0000000002")).outcome(), Outcome::UserNotFound);
    assert_eq!(c.authenticate(info("suci-0-999-70-0000-1-1-00ff")).outcome(), Outcome::UnsupportedProtectionScheme);
    let wrong_network = AuthInfo::new(SupiOrSuci::new(SUCI).unwrap(), ServingNetwork::new("mnc070").unwrap());
    assert_eq!(c.authenticate(wrong_network).outcome(), Outcome::ServingNetworkNotAuthorized);
}

#[test]
fn a_ue_ahead_of_the_udm_resynchronises_it() {
    let mut c = core();
    let challenge = c.authenticate(info(SUCI));
    let ue_sqn = hex("000000000400");
    let auts = ue(&challenge, &ue_sqn).unwrap_err();
    let retry = c.authenticate(info(SUCI).with_resync(challenge.rand, auts));
    assert_eq!(retry.outcome(), Outcome::Ok);
    let (res_star, _) = ue(&retry, &ue_sqn).expect("fresh after resynchronisation");
    assert_eq!(c.confirm(retry.auth_ctx, res_star).outcome(), Outcome::Ok);

    let mut forged = auts;
    forged[13] ^= 1;
    let refused = c.authenticate(info(SUCI).with_resync(challenge.rand, forged));
    assert_eq!(refused.outcome(), Outcome::AuthenticationRejected);
}

#[test]
fn unconfirmed_challenges_expire() {
    let mut c = core();
    let challenge = c.authenticate(info(SUCI));
    let (res_star, _) = ue(&challenge, &[0; 6]).unwrap();
    c.tb.advance(LIFETIME + Duration::from_secs(6));
    assert!(c.log.lines().iter().any(|l| l.msg == "1 authentication context(s) expired"), "{:#?}", c.log.lines());
    assert_eq!(c.confirm(challenge.auth_ctx, res_star).outcome(), Outcome::ContextNotFound);
}

/// The UE's side of EAP-AKA': RES and K_aut from the challenge's RAND and
/// AUTN (in its EAP-Request), and K_SEAF to compare.
fn eap_ue(request: &[u8]) -> ([u8; 8], [u8; 32], [u8; 32], u8) {
    // AT_RAND and AT_AUTN are the first two attributes, after the 8-octet header.
    let (rand, autn): ([u8; 16], [u8; 16]) = (request[12..28].try_into().unwrap(), request[32..48].try_into().unwrap());
    let m = Milenage::new(&hex(K), &hex(OPC));
    let v = m.f2345(&rand);
    let sqn_xor_ak: [u8; 6] = autn[..6].try_into().unwrap();
    let (ck_p, ik_p) = eap_aka::ck_ik_prime(&v.ck, &v.ik, SNN, &sqn_xor_ak);
    let keys = eap_aka::keys(&ck_p, &ik_p, "999700000000001");
    (v.res, keys.k_aut, kdf::kseaf(&keys.kausf, SNN), request[1])
}

#[test]
fn an_eap_aka_prime_subscriber_answers_an_eap_challenge() {
    let mut c = core_with("eap-aka-prime");
    let challenge = c.authenticate(info(SUCI));
    assert_eq!((challenge.outcome(), challenge.method), (Outcome::Ok, method::EAP_AKA_PRIME));
    let request = challenge.eap.get().to_vec();
    assert_eq!(request[0], eap_aka::code::REQUEST);
    assert!(request.windows(SNN.len()).any(|w| w == SNN.as_bytes()), "AT_KDF_INPUT: the serving network name");
    let (res, k_aut, kseaf, id) = eap_ue(&request);

    // A wrong RES, then the right one (a new challenge each time).
    let wrong = c.confirm_eap(challenge.auth_ctx, &eap_aka::answer(id, &[0; 8], &k_aut));
    assert_eq!((wrong.outcome(), wrong.eap), (Outcome::AuthenticationRejected, eap_aka::failure(id)));
    let challenge = c.authenticate(info(SUCI));
    let (res2, k_aut, kseaf2, id) = eap_ue(challenge.eap.get());
    assert_ne!(res, res2, "a fresh RAND");
    let result = c.confirm_eap(challenge.auth_ctx, &eap_aka::answer(id, &res2, &k_aut));
    assert_eq!((result.outcome(), result.supi.as_str()), (Outcome::Ok, "imsi-999700000000001"));
    assert_eq!((result.kseaf, result.eap), (kseaf2, eap_aka::success(id)), "K_SEAF from EMSK; EAP-Success");
    assert_ne!(kseaf, kseaf2);
}
