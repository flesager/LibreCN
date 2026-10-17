//! The OAM EOs on the test bed: logging, alarms, control, web server.

use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use librecn_common::alarm::{Alarms, Severity};
use librecn_common::log;
use librecn_common::log::{Level, Logger};
use librecn_oam::proto::{self, Deploy, Reply, Stop};
use librecn_oam::{AlarmManager, CapturedLog, Control, Factory, LogSink, Server};
use librecn_scheduler::testbed::{TestBed, TestBedConfig};
use librecn_scheduler::{Context, EoConfig, EoId, ExecutionObject, FaultPolicy, Message};

const KICK: u32 = 0x0100_0001;

/// Logs one line per level on start.
struct Chatty(Logger);

impl ExecutionObject for Chatty {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        log!(self.0, ctx, Vip, "started");
        log!(self.0, ctx, Fatal, "fatal {}", 1);
        log!(self.0, ctx, Error, "error {}", 2);
        log!(self.0, ctx, Info, "info \"quoted\"");
        log!(self.0, ctx, Debug, "debug");
    }
    fn on_message(&mut self, _: &Message<'_>, _: &mut Context<'_>) {}
}

fn with_log(tb: &mut TestBed) -> CapturedLog {
    let (sink, captured) = LogSink::capture();
    tb.add(LogSink::config(), sink).unwrap();
    captured
}

fn levels(captured: &CapturedLog, eo: &str) -> Vec<String> {
    captured.lines().into_iter().filter(|l| l.eo == eo).map(|l| l.level).collect()
}

#[test]
fn logs_go_to_the_sink_filtered_by_level_vip_always() {
    let mut tb = TestBed::new();
    let captured = with_log(&mut tb);
    tb.add(EoConfig::new("normal"), Chatty(Logger::default())).unwrap();
    tb.add(EoConfig::new("quiet"), Chatty(Logger::new(Level::Vip))).unwrap();
    tb.add(EoConfig::new("verbose"), Chatty(Logger::new(Level::Debug))).unwrap();
    tb.run_until_idle();

    assert_eq!(levels(&captured, "normal"), ["VIP", "FATAL", "ERROR", "INFO"]);
    assert_eq!(levels(&captured, "quiet"), ["VIP"]);
    assert_eq!(levels(&captured, "verbose"), ["VIP", "FATAL", "ERROR", "INFO", "DEBUG"]);
    let info = captured.lines().into_iter().find(|l| l.eo == "normal" && l.level == "INFO").unwrap();
    assert_eq!(info.msg, "info \"quoted\"");
    assert!(info.ts.ends_with('Z') && info.ts.len() == 24, "{}", info.ts);
}

#[test]
fn without_a_log_eo_lines_are_printed_and_nothing_breaks() {
    let mut tb = TestBed::new();
    let id = tb.add(EoConfig::new("alone"), Chatty(Logger::default())).unwrap();
    tb.run_until_idle();
    assert!(tb.is_alive(id));
}

/// Sends the queued requests one per `KICK`, records the replies.
type Ask = Box<dyn FnOnce(&mut Context<'_>, u32) + Send>;

struct Asker {
    pending: VecDeque<Ask>,
    replies: Arc<Mutex<Vec<(bool, serde_json::Value)>>>,
}

impl ExecutionObject for Asker {
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if msg.id() == KICK {
            if let Some(ask) = self.pending.pop_front() {
                ask(ctx, 7);
            }
        } else if let Some(reply) = msg.body::<Reply>() {
            let json = serde_json::from_str(reply.text()).expect("replies are JSON");
            self.replies.lock().unwrap().push((reply.ok(), json));
        }
    }
}

fn ask_signal(service: &'static str, msg_id: u32) -> Ask {
    Box::new(move |ctx, corr| {
        let to = ctx.find(service).unwrap();
        ctx.signal(to, msg_id, corr).unwrap();
    })
}

fn ask_deploy(text: &'static str) -> Ask {
    Box::new(move |ctx, corr| {
        let to = ctx.find("control").unwrap();
        let mut deploy = ctx.alloc::<Deploy>(to).unwrap().with_corr_id(corr);
        assert!(deploy.fill(text));
        ctx.send(deploy).unwrap();
    })
}

fn ask_stop(name: &'static str) -> Ask {
    Box::new(move |ctx, corr| {
        let eo = ctx.find(name).unwrap().raw();
        let to = ctx.find("control").unwrap();
        let mut stop = ctx.alloc::<Stop>(to).unwrap().with_corr_id(corr);
        stop.eo = eo;
        ctx.send(stop).unwrap();
    })
}

/// Runs the asker's requests in order; returns the replies.
fn ask(tb: &mut TestBed, asks: Vec<Ask>) -> Vec<(bool, serde_json::Value)> {
    let replies = Arc::default();
    let n = asks.len();
    // Replies are 16 KiB bodies, allocated in the asker's pool.
    let cfg = EoConfig::new("asker").buffers(&[(16 * 1024, 4)]);
    let asker = tb.add(cfg, Asker { pending: asks.into(), replies: Arc::clone(&replies) }).unwrap();
    for _ in 0..n {
        tb.signal(asker, KICK).unwrap();
        tb.run_until_idle();
    }
    tb.stop(asker).unwrap();
    tb.run_until_idle();
    replies.lock().unwrap().clone()
}

/// Raises alarm 1 (major) on `KICK` the first time, updates it to minor the
/// second time, clears it the third.
struct Alarming {
    alarms: Alarms,
    kicks: u32,
}

impl ExecutionObject for Alarming {
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if msg.id() == KICK {
            self.kicks += 1;
            let ok = match self.kicks {
                1 => self.alarms.raise(ctx, 1, Severity::Major, "link down"),
                2 => self.alarms.raise(ctx, 1, Severity::Minor, "link flapping"),
                _ => self.alarms.clear(ctx, 1),
            };
            assert!(ok);
        }
    }
}

#[test]
fn alarms_are_raised_updated_cleared_listed_and_logged() {
    let mut tb = TestBed::new();
    let captured = with_log(&mut tb);
    tb.add(AlarmManager::config(), AlarmManager::new()).unwrap();
    let nf = tb.add(EoConfig::new("nf"), Alarming { alarms: Alarms::new(), kicks: 0 }).unwrap();
    tb.run_until_idle();

    tb.signal(nf, KICK).unwrap();
    tb.signal(nf, KICK).unwrap();
    tb.run_until_idle();
    let replies = ask(&mut tb, vec![ask_signal("alarm", proto::ALARMS)]);
    let active = replies[0].1.as_array().unwrap();
    assert_eq!(active.len(), 1, "updated, not duplicated: {active:?}");
    assert_eq!((active[0]["severity"].as_str(), active[0]["source"].as_str()), (Some("MINOR"), Some("nf")));
    assert_eq!(active[0]["text"], "link flapping");

    tb.signal(nf, KICK).unwrap();
    tb.run_until_idle();
    let replies = ask(&mut tb, vec![ask_signal("alarm", proto::ALARMS)]);
    assert!(replies[0].1.as_array().unwrap().is_empty());

    let logged: Vec<_> = captured.lines().into_iter().filter(|l| l.eo == "alarm").map(|l| (l.level, l.msg)).collect();
    assert_eq!(logged.len(), 3, "{logged:?}");
    assert_eq!(logged[0].0, "ERROR", "major alarms log as errors");
    assert!(logged[0].1.contains("raised MAJOR nf/1: link down"), "{}", logged[0].1);
    assert!(logged[1].1.contains("updated MINOR"));
    assert!(logged[2].1.contains("cleared"));
}

/// An NF kind for the tests: counts its start (counter "starts"); panics on `KICK`.
struct Dummy;

impl ExecutionObject for Dummy {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        ctx.count(0, 1);
    }

    fn on_message(&mut self, msg: &Message<'_>, _: &mut Context<'_>) {
        assert_ne!(msg.id(), KICK, "dummy kicked");
    }
}

fn dummy_factory() -> Factory {
    Box::new(|name, params| {
        if params.get("fail").and_then(|v| v.as_bool()) == Some(true) {
            return Err("asked to fail".into());
        }
        Ok((EoConfig::new(name).counters(&["starts"]), Box::new(Dummy)))
    })
}

fn oam(tb: &mut TestBed) -> CapturedLog {
    let captured = with_log(tb);
    tb.add(AlarmManager::config(), AlarmManager::new()).unwrap();
    tb.add(Control::config(), Control::new().kind("dummy", dummy_factory())).unwrap();
    tb.run_until_idle();
    captured
}

#[test]
fn control_deploys_from_toml_or_json_and_reports_each_nf() {
    let mut tb = TestBed::new();
    let captured = oam(&mut tb);
    let replies = ask(
        &mut tb,
        vec![
            ask_deploy("[[nf]]\nkind = \"dummy\"\nname = \"d1\"\n\n[[nf]]\nkind = \"nope\"\nname = \"d2\"\n"),
            ask_deploy(
                r#"{"nf": [{"kind": "dummy", "name": "d3"}, {"kind": "dummy", "name": "d1"},
                                 {"kind": "dummy", "name": "d4", "params": {"fail": true}}]}"#,
            ),
            ask_deploy("not = [valid"),
            ask_signal("control", proto::KINDS),
        ],
    );
    assert!(!replies[0].0, "one NF failed");
    assert_eq!(replies[0].1["started"][0]["name"], "d1");
    assert_eq!(replies[0].1["failed"][0]["error"], "unknown kind");
    let failed: Vec<_> = replies[1].1["failed"].as_array().unwrap().iter().map(|f| f["error"].clone()).collect();
    assert_eq!(failed, ["an EO with this name runs already", "asked to fail"]);
    assert_eq!(replies[1].1["started"][0]["name"], "d3");
    assert!(replies[2].1["error"].as_str().unwrap().starts_with("invalid TOML"));
    assert_eq!(replies[3].1, serde_json::json!(["dummy"]));

    let names: Vec<_> = ["d1", "d2", "d3", "d4"].iter().map(|n| tb_has(&mut tb, n)).collect();
    assert_eq!(names, [true, false, true, false]);
    assert!(captured.lines().iter().any(|l| l.eo == "control" && l.level == "ERROR" && l.msg.contains("d2")));
}

/// Whether an EO with this name runs, asked through an EO.
fn tb_has(tb: &mut TestBed, name: &'static str) -> bool {
    let found = Arc::new(Mutex::new(false));
    struct Finder(&'static str, Arc<Mutex<bool>>);
    impl ExecutionObject for Finder {
        fn on_start(&mut self, ctx: &mut Context<'_>) {
            *self.1.lock().unwrap() = ctx.find(self.0).is_some();
            ctx.stop(ctx.self_id()).unwrap();
        }
        fn on_message(&mut self, _: &Message<'_>, _: &mut Context<'_>) {}
    }
    tb.add(EoConfig::new("finder"), Finder(name, found.clone())).unwrap();
    tb.run_until_idle();
    *found.lock().unwrap()
}

#[test]
fn control_stops_nfs_but_not_oam_eos() {
    let mut tb = TestBed::new();
    oam(&mut tb);
    let replies =
        ask(&mut tb, vec![ask_deploy("[[nf]]\nkind = \"dummy\"\nname = \"d1\"\n"), ask_stop("d1"), ask_stop("log")]);
    assert_eq!(replies[1], (true, serde_json::json!({ "stopping": "d1" })));
    assert_eq!(replies[2], (false, serde_json::json!({ "error": "OAM EOs cannot be stopped" })));
    assert!(!tb_has(&mut tb, "d1"));
    assert!(tb_has(&mut tb, "log"));
}

#[test]
fn control_raises_a_critical_alarm_when_an_eo_faults() {
    let mut tb = TestBed::with_config(TestBedConfig { fault_policy: FaultPolicy::Isolate, ..Default::default() });
    let captured = oam(&mut tb);
    ask(&mut tb, vec![ask_deploy("[[nf]]\nkind = \"dummy\"\nname = \"bomb\"\n")]);
    let bomb = find(&mut tb, "bomb");
    tb.signal(bomb, KICK).unwrap();
    tb.run_until_idle();
    tb.advance(Duration::from_secs(1));

    let replies = ask(&mut tb, vec![ask_signal("alarm", proto::ALARMS)]);
    let active = replies[0].1.as_array().unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!((active[0]["severity"].as_str(), active[0]["source"].as_str()), (Some("CRITICAL"), Some("control")));
    assert_eq!(active[0]["text"], "EO bomb faulted");
    assert!(captured.lines().iter().any(|l| l.level == "FATAL" && l.msg == "EO bomb faulted"));
    tb.advance(Duration::from_secs(1));
    let replies = ask(&mut tb, vec![ask_signal("alarm", proto::ALARMS)]);
    assert_eq!(replies[0].1.as_array().unwrap().len(), 1, "raised once");
}

fn find(tb: &mut TestBed, name: &'static str) -> EoId {
    let found = Arc::new(Mutex::new(None));
    struct Finder(&'static str, Arc<Mutex<Option<EoId>>>);
    impl ExecutionObject for Finder {
        fn on_start(&mut self, ctx: &mut Context<'_>) {
            *self.1.lock().unwrap() = ctx.find(self.0);
            ctx.stop(ctx.self_id()).unwrap();
        }
        fn on_message(&mut self, _: &Message<'_>, _: &mut Context<'_>) {}
    }
    tb.add(EoConfig::new("finder"), Finder(name, found.clone())).unwrap();
    tb.run_until_idle();
    found.lock().unwrap().expect("EO runs")
}

// ------------------------------------------------------------ web server

/// One HTTP request from a client thread, while the test bed serves it.
fn http(tb: &mut TestBed, addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, String) {
    let request = format!("{method} {path} HTTP/1.1\r\nHost: x\r\nContent-Length: {}\r\n\r\n{body}", body.len());
    let client = std::thread::spawn(move || {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    });
    while !client.is_finished() {
        tb.wait_io(Duration::from_millis(5));
        tb.run_until_idle();
    }
    let response = client.join().unwrap();
    let status = response[9..12].parse().unwrap();
    let body = response.split_once("\r\n\r\n").map(|(_, b)| b.to_string()).unwrap_or_default();
    (status, body)
}

fn web(tb: &mut TestBed) -> SocketAddr {
    oam(tb);
    let server = Server::new("127.0.0.1:0".parse().unwrap());
    let bound = server.bound_addr();
    tb.add(Server::config(), server).unwrap();
    tb.run_until_idle();
    bound.lock().unwrap().expect("listening")
}

#[test]
fn web_api_lists_deploys_stops_and_exports_metrics() {
    let mut tb = TestBed::new();
    let addr = web(&mut tb);

    let (status, html) = http(&mut tb, addr, "GET", "/", "");
    assert_eq!(status, 200);
    assert!(html.contains("<title>LibreCN</title>"));

    let (status, body) = http(&mut tb, addr, "POST", "/api/nfs", "[[nf]]\nkind = \"dummy\"\nname = \"d1\"\n");
    assert_eq!(status, 200, "{body}");
    let id = serde_json::from_str::<serde_json::Value>(&body).unwrap()["started"][0]["id"].as_u64().unwrap();

    let (status, body) = http(&mut tb, addr, "GET", "/api/eos", "");
    assert_eq!(status, 200);
    let eos: serde_json::Value = serde_json::from_str(&body).unwrap();
    let names: Vec<_> = eos.as_array().unwrap().iter().map(|e| e["name"].as_str().unwrap().to_string()).collect();
    assert_eq!(names, ["log", "alarm", "control", "server", "d1"]);
    assert!(eos[0]["stats"]["messages"].as_u64().unwrap() > 0, "the log EO handled records");
    assert_eq!(eos[4]["stats"]["counters"]["starts"], 1, "the NF's own counter");

    let (_, kinds) = http(&mut tb, addr, "GET", "/api/kinds", "");
    assert_eq!(kinds, r#"["dummy"]"#);
    let (status, alarms) = http(&mut tb, addr, "GET", "/api/alarms", "");
    assert_eq!((status, alarms.as_str()), (200, "[]"));
    let (status, workers) = http(&mut tb, addr, "GET", "/api/workers", "");
    assert_eq!((status, workers.as_str()), (200, "[]"), "no workers on the test bed");

    let (status, metrics) = http(&mut tb, addr, "GET", "/metrics", "");
    assert_eq!(status, 200);
    assert!(metrics.contains("librecn_eo_processing_seconds_bucket{eo=\"d1\",pool=\"\",le=\"+Inf\"}"), "{metrics}");
    assert!(metrics.contains("# TYPE librecn_eo_lane_depth gauge"));
    assert!(metrics.contains("librecn_eo_counter_total{eo=\"d1\",pool=\"\",counter=\"starts\"} 1"), "{metrics}");

    let (status, body) = http(&mut tb, addr, "POST", &format!("/api/eos/{id}/stop"), "");
    assert_eq!((status, body.as_str()), (200, r#"{"stopping":"d1"}"#));
    let (status, _) = http(&mut tb, addr, "POST", "/api/eos/12/stop", "");
    assert_eq!(status, 400, "unknown EO");
    let (status, _) = http(&mut tb, addr, "GET", "/nowhere", "");
    assert_eq!(status, 404);
    let (status, _) = http(&mut tb, addr, "POST", "/api/nfs", "[[nf]");
    assert_eq!(status, 400, "invalid deployment");
}

#[test]
fn web_api_without_control_answers_503() {
    let mut tb = TestBed::new();
    let server = Server::new("127.0.0.1:0".parse().unwrap());
    let bound = server.bound_addr();
    tb.add(Server::config(), server).unwrap();
    tb.run_until_idle();
    let addr = bound.lock().unwrap().unwrap();
    let (status, body) = http(&mut tb, addr, "GET", "/api/kinds", "");
    assert_eq!((status, body.as_str()), (503, r#"{"error":"service not running"}"#));
}
