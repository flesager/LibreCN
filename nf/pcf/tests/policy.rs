//! The PCF EO: AM and SM policies per subscriber, changed through the web
//! API (the AMF told), kept across a restart.

use librecn_common::manage::{Manage, Method, Reply};
use librecn_common::state::{SNAPSHOT_PERIOD, Store};
use librecn_pcf::Pcf;
use librecn_sbi::pdu_session::Slice;
use librecn_sbi::policy::{AmPolicyCreate, AmPolicyResult, AmPolicyUpdate, SmPolicyCreate, SmPolicyResult};
use librecn_sbi::{DnnText, Supi};
use librecn_scheduler::testbed::{Probe, ProbeLog, TestBed};
use librecn_scheduler::{EoConfig, EoId};
use zerocopy::FromZeros;

const ME: &str = "imsi-999700000000001";

struct Core {
    tb: TestBed,
    pcf: EoId,
    peer: EoId,
    seen: ProbeLog,
}

fn core(store: Option<&Store>) -> Core {
    let mut tb = TestBed::new();
    let params = serde_json::json!({
        "default": { "ue_ambr_mbps": { "uplink": 100, "downlink": 500 } },
        "policies": [{ "supi": ME, "ue_ambr_mbps": { "uplink": 50, "downlink": 200 },
                       "sessions": [{ "dnn": "internet", "ambr_mbps": { "uplink": 20, "downlink": 100 } }] }]
    });
    let (cfg, pcf) = Pcf::from_params("pcf", &params).unwrap();
    let pcf = match store {
        Some(store) => pcf.with_store(store.clone()).unwrap(),
        None => pcf,
    };
    let pcf = tb.add(cfg, pcf).unwrap();
    let (probe, seen) = Probe::new();
    let peer = tb.add(EoConfig::new("amf").buffers(&[(256, 8), (1024, 8), (16 * 1024, 2)]), probe).unwrap();
    tb.run_until_idle();
    Core { tb, pcf, peer, seen }
}

impl Core {
    fn ue_ambr(&mut self, supi: &str) -> (u64, u64) {
        self.tb.inject_from(self.peer, self.pcf, 3, &AmPolicyCreate { supi: Supi::new(supi).unwrap() }).unwrap();
        self.tb.run_until_idle();
        let r: AmPolicyResult = self.seen.take()[0].body().unwrap();
        (r.ue_ambr.uplink / 1_000_000, r.ue_ambr.downlink / 1_000_000)
    }

    fn session_ambr(&mut self, supi: &str, dnn: &str) -> (u64, u64) {
        let mut request = SmPolicyCreate::new_zeroed();
        (request.supi, request.dnn, request.pdu_session_id) = (Supi::new(supi).unwrap(), DnnText::new(dnn).unwrap(), 1);
        request.slice = Slice { sst: 1, has_sd: 0, sd: [0; 3] };
        self.tb.inject_from(self.peer, self.pcf, 4, &request).unwrap();
        self.tb.run_until_idle();
        let r: SmPolicyResult = self.seen.take()[0].body().unwrap();
        (r.rule.ambr.uplink / 1_000_000, r.rule.ambr.downlink / 1_000_000)
    }

    fn manage(&mut self, method: Method, path: &str, body: &str) -> u16 {
        let mut request = Manage::new_zeroed();
        request.fill(method, path, body);
        self.tb.inject_from(self.peer, self.pcf, 5, &request).unwrap();
        self.tb.run_until_idle();
        let got = self.seen.take();
        got.iter().find_map(|m| m.body::<Reply>()).unwrap().status()
    }
}

#[test]
fn a_subscriber_gets_its_policy_and_hears_of_its_change() {
    let mut c = core(None);
    assert_eq!(c.ue_ambr(ME), (50, 200));
    assert_eq!(c.ue_ambr("imsi-999700000000009"), (100, 500), "the default");
    assert_eq!(c.session_ambr(ME, "internet"), (20, 100));
    assert_eq!(c.session_ambr(ME, "ims"), (1000, 1000), "no session policy for ims: the built-in one");

    // The web API changes the policy: the AMF that asked hears of it.
    let body = format!(r#"{{"supi": "{ME}", "ue_ambr_mbps": {{"uplink": 5, "downlink": 10}}}}"#);
    let mut request = Manage::new_zeroed();
    request.fill(Method::Post, "policies", &body);
    c.tb.inject_from(c.peer, c.pcf, 5, &request).unwrap();
    c.tb.run_until_idle();
    let got = c.seen.take();
    let update: AmPolicyUpdate = got.iter().find_map(|m| m.body()).expect("the AMF is told");
    assert_eq!((update.supi.as_str(), update.ue_ambr.downlink), (ME, 10_000_000));
    assert_eq!(got.iter().find_map(|m| m.body::<Reply>()).unwrap().status(), 200);
    assert_eq!(c.session_ambr(ME, "internet"), (1000, 1000), "the new policy has no session policy: the default's");
    assert_eq!(c.manage(Method::Delete, &format!("policies/{ME}"), ""), 200);
    assert_eq!(c.ue_ambr(ME), (100, 500), "back to the default");
    assert_eq!(c.manage(Method::Delete, &format!("policies/{ME}"), ""), 404);
    assert_eq!(c.manage(Method::Post, "policies", r#"{"supi": "imsi-1", "sessions": [{"five_qi": 1}]}"#), 400);
}

#[test]
fn the_web_api_changes_survive_a_restart() {
    let dir = std::env::temp_dir().join(format!("librecn-pcf-state-{}", std::process::id()));
    let store = Store::at(dir.join("pcf.json"));
    let mut c = core(Some(&store));
    let body = r#"{"supi": "imsi-999700000000002", "ue_ambr_mbps": {"uplink": 7, "downlink": 8}}"#;
    assert_eq!(c.manage(Method::Post, "policies", body), 200);
    assert_eq!(c.manage(Method::Delete, &format!("policies/{ME}"), ""), 200);
    c.tb.advance(SNAPSHOT_PERIOD);

    let mut c = core(Some(&store));
    assert_eq!(c.ue_ambr("imsi-999700000000002"), (7, 8), "set by the web API");
    assert_eq!(c.ue_ambr(ME), (100, 500), "the configuration's, removed by the web API");
    std::fs::remove_dir_all(dir).unwrap();
}
