//! The UDM's state file: what the web API changed and the SQNs, kept across
//! a restart.

use librecn_common::manage::{Manage, Method};
use librecn_common::state::{SNAPSHOT_PERIOD, Store};
use librecn_sbi::ue_authentication::{AuthData, AuthInfo, GetAuthData};
use librecn_sbi::{Outcome, ServingNetwork, SupiOrSuci};
use librecn_scheduler::testbed::{Probe, ProbeLog, TestBed};
use librecn_scheduler::{EoConfig, EoId};
use librecn_udm::Udm;
use librecn_udm::persist::SQN_JUMP;
use zerocopy::FromZeros;

const K: &str = "465B5CE8B199B49FAA5F0A2EE238A6BC";
const OPC: &str = "E8ED289DEBA952E4283B54E88E6183CA";
const KEYS: &str = r#""k": "465B5CE8B199B49FAA5F0A2EE238A6BC", "opc": "E8ED289DEBA952E4283B54E88E6183CA""#;

/// A UDM with one configured subscriber, its state in `store`; a probe for its answers.
fn start(store: &Store) -> (TestBed, EoId, EoId, ProbeLog) {
    let mut tb = TestBed::new();
    let params: serde_json::Value =
        serde_json::from_str(&format!(r#"{{"subscribers": [{{"supi": "imsi-999700000000001", {KEYS}}}]}}"#)).unwrap();
    let (cfg, udm) = Udm::from_params("udm", &params).unwrap();
    let udm = tb.add(cfg, udm.with_store(store.clone()).unwrap()).unwrap();
    let (probe, seen) = Probe::new();
    let peer = tb.add(EoConfig::new("peer").buffers(&[(256, 8), (16 * 1024, 2)]), probe).unwrap();
    tb.run_until_idle();
    (tb, udm, peer, seen)
}

/// The SQN of the vector the UDM gives for `supi` (UERANSIM's subscriber: AK known).
fn vector_sqn(tb: &mut TestBed, udm: EoId, peer: EoId, seen: &ProbeLog, supi: &str) -> Option<u64> {
    let snn = ServingNetwork::new("5G:mnc070.mcc999.3gppnetwork.org").unwrap();
    let info = AuthInfo::new(SupiOrSuci::new(supi).unwrap(), snn);
    tb.inject_from(peer, udm, 1, &GetAuthData { info }).unwrap();
    tb.run_until_idle();
    let data: AuthData = seen.take()[0].body().unwrap();
    (data.outcome() == Outcome::Ok).then(|| {
        let m = librecn_security::milenage::Milenage::new(&hex(K), &hex(OPC));
        let ak = m.f2345(&data.rand).ak;
        (0..6).fold(0, |sqn, i| sqn << 8 | u64::from(data.autn[i] ^ ak[i]))
    })
}

fn hex(s: &str) -> [u8; 16] {
    std::array::from_fn(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap())
}

fn manage(tb: &mut TestBed, udm: EoId, peer: EoId, method: Method, path: &str, body: &str) {
    let mut request = Manage::new_zeroed();
    request.fill(method, path, body);
    tb.inject_from(peer, udm, 2, &request).unwrap();
    tb.run_until_idle();
}

#[test]
fn provisioning_and_sqns_survive_a_restart() {
    let dir = std::env::temp_dir().join(format!("librecn-udm-state-{}", std::process::id()));
    let store = Store::at(dir.join("udm.json"));
    let (mut tb, udm, peer, seen) = start(&store);
    manage(&mut tb, udm, peer, Method::Post, "subscribers", &format!(r#"{{"supi": "imsi-999700000000002", {KEYS}}}"#));
    manage(&mut tb, udm, peer, Method::Delete, "subscribers/imsi-999700000000001", "");
    seen.take();
    let first = vector_sqn(&mut tb, udm, peer, &seen, "imsi-999700000000002").unwrap();
    assert!(store.load().unwrap().is_none(), "not before the snapshot period");
    tb.advance(SNAPSHOT_PERIOD);
    assert!(store.load().unwrap().is_some(), "written once changed");

    // Restarted from the same configuration: the web API's changes are back,
    // and the next SQN is past any used before.
    let (mut tb, udm, peer, seen) = start(&store);
    assert!(vector_sqn(&mut tb, udm, peer, &seen, "imsi-999700000000001").is_none(), "still removed");
    let next = vector_sqn(&mut tb, udm, peer, &seen, "imsi-999700000000002").expect("still provisioned");
    assert!(next > first + SQN_JUMP, "SQN {next:#x} after {first:#x}");
    std::fs::remove_dir_all(dir).unwrap();
}
