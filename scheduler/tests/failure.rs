//! Failure reports: a panicking handler always leaves its root cause behind.

mod common;

use std::process::Command;

use common::{Ping, Pong};
use librecn_scheduler::testbed::{Probe, TestBed, TestBedConfig};
use librecn_scheduler::{Context, EoConfig, Error, ExecutionObject, FaultPolicy, Message, MsgBody, system};

/// Panics on `Ping { seq: 3 }`.
struct Bomb;

impl ExecutionObject for Bomb {
    fn on_message(&mut self, msg: &Message<'_>, _: &mut Context<'_>) {
        if let Some(p) = msg.body::<Ping>() {
            assert_ne!(p.seq, 3, "boom on 3");
        }
    }
}

fn bed(fault_policy: FaultPolicy) -> TestBed {
    TestBed::with_config(TestBedConfig { fault_policy, ..Default::default() })
}

#[test]
fn isolated_eo_report_names_the_message_and_its_history() {
    let mut tb = bed(FaultPolicy::Isolate);
    let bomb = tb.add(EoConfig::new("bomb"), Bomb).unwrap();
    let (probe, received) = Probe::new();
    let bystander = tb.add(EoConfig::new("bystander"), probe).unwrap();
    for seq in 1..=3 {
        tb.inject_from(bystander, bomb, 10 + seq as u32, &Ping { seq }).unwrap();
    }
    tb.run_until_idle();

    let r = tb.failure(bomb).expect("report kept for the faulted EO");
    assert_eq!((r.eo, r.eo_name.as_str()), (bomb, "bomb"));
    assert_eq!((r.message.msg_id, r.message.corr_id, r.message.from), (Ping::MSG_ID, 13, bystander));
    let history: Vec<_> = r.history.iter().map(|h| (h.msg_id, h.corr_id)).collect();
    assert_eq!(history, [(system::EO_START, 0), (Ping::MSG_ID, 11), (Ping::MSG_ID, 12)]);
    assert!(r.panic_message.contains("boom on 3"), "{}", r.panic_message);
    assert!(r.location.as_deref().unwrap().contains("failure.rs"));
    assert!(!r.backtrace.is_empty());
    assert_eq!(r.stats.processing.count, 3, "snapshot taken at the panic: messages completed before it");
    let text = r.to_string();
    assert!(text.contains("eo:       bomb") && text.contains("corr_id=13"), "{text}");

    // The faulted EO refuses messages; the others keep working.
    assert!(!tb.is_alive(bomb));
    assert_eq!(tb.inject(bomb, &Ping { seq: 4 }), Err(Error::EoFaulted));
    assert!(tb.stats(bomb).is_some(), "metrics stay readable");
    tb.inject(bystander, &Pong { seq: 1 }).unwrap();
    tb.run_until_idle();
    assert_eq!(received.len(), 1);
}

#[test]
#[should_panic(expected = "boom on 3")]
fn propagate_policy_lets_the_panic_through() {
    let mut tb = bed(FaultPolicy::Propagate);
    let bomb = tb.add(EoConfig::new("bomb"), Bomb).unwrap();
    tb.inject(bomb, &Ping { seq: 3 }).unwrap();
    tb.run_until_idle();
}

const CHILD_ENV: &str = "LIBRECN_ABORT_CHILD";

/// Runs the abort scenario in a child process (abort kills the whole process).
#[test]
fn abort_policy_writes_report_then_aborts() {
    if std::env::var_os(CHILD_ENV).is_some() {
        let mut tb = bed(FaultPolicy::Abort);
        let bomb = tb.add(EoConfig::new("bomb"), Bomb).unwrap();
        tb.inject(bomb, &Ping { seq: 3 }).unwrap();
        tb.run_until_idle();
        unreachable!("the panic must abort the process");
    }
    let out = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "abort_policy_writes_report_then_aborts", "--nocapture", "--test-threads=1"])
        .env(CHILD_ENV, "1")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(out.status.signal(), Some(6), "SIGABRT expected, got {:?}", out.status);
    }
    assert!(stderr.contains("=== execution object failure ==="), "{stderr}");
    assert!(stderr.contains("eo:       bomb") && stderr.contains("boom on 3"), "{stderr}");
}
