//! The `librecn` binary, end to end: configuration, OAM, an NF, the web API,
//! a clean stop on SIGTERM.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn http(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(addr).unwrap();
    stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    write!(stream, "{method} {path} HTTP/1.1\r\nHost: x\r\nContent-Length: {}\r\n\r\n{body}", body.len()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let status = response[9..12].parse().unwrap();
    (status, response.split_once("\r\n\r\n").map(|(_, b)| b.to_string()).unwrap_or_default())
}

/// Waits for a stdout line containing `needle`; returns it.
fn wait_line(lines: &mpsc::Receiver<String>, seen: &mut Vec<String>, needle: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(10);
    if let Some(l) = seen.iter().find(|l| l.contains(needle)) {
        return l.clone();
    }
    while let Ok(line) = lines.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
        seen.push(line.clone());
        if line.contains(needle) {
            return line;
        }
    }
    panic!("no line with {needle:?} in {seen:#?}");
}

#[test]
fn boots_deploys_serves_and_stops_cleanly() {
    let config = std::env::temp_dir().join(format!("librecn-e2e-{}.toml", std::process::id()));
    std::fs::write(
        &config,
        r#"
        [[runtime.pools]]
        name = "cp"
        workers = 1

        [oam]
        listen = "127.0.0.1:0"

        [[nf]]
        kind = "demo"
        name = "demo-1"
        [nf.params]
        period_ms = 20
        alarm = true
        "#,
    )
    .unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_librecn"))
        .args(["--config", config.to_str().unwrap()])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let (tx, lines) = mpsc::channel();
    let stdout = child.stdout.take().unwrap();
    std::thread::spawn(move || BufReader::new(stdout).lines().map_while(Result::ok).for_each(|l| drop(tx.send(l))));
    let mut seen = Vec::new();

    let listening = wait_line(&lines, &mut seen, "web server on http://");
    let addr: SocketAddr = listening.split("http://").nth(1).unwrap().trim_end_matches("\"}").parse().unwrap();
    let started = wait_line(&lines, &mut seen, "demo NF started");
    assert!(started.contains(r#""level":"VIP""#) && started.contains(r#""eo":"demo-1""#), "{started}");

    let (status, eos) = http(addr, "GET", "/api/eos", "");
    assert_eq!(status, 200);
    let names: Vec<String> = serde_json::from_str::<serde_json::Value>(&eos)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(names, ["log", "alarm", "control", "server", "demo-1"]);
    let (_, alarms) = http(addr, "GET", "/api/alarms", "");
    assert!(alarms.contains("demo alarm"), "{alarms}");
    let (status, body) = http(addr, "POST", "/api/nfs", "[[nf]]\nkind = \"demo\"\nname = \"demo-2\"\n");
    assert_eq!(status, 200, "{body}");
    wait_line(&lines, &mut seen, r#""eo":"demo-2","msg":"demo NF started"#);

    let killed = Command::new("kill").args(["-TERM", &child.id().to_string()]).status().unwrap();
    assert!(killed.success());
    let status = child.wait().unwrap();
    wait_line(&lines, &mut seen, "demo NF stopped after");
    let _ = std::fs::remove_file(&config);
    let mut stderr = String::new();
    child.stderr.take().unwrap().read_to_string(&mut stderr).unwrap();
    assert!(status.success(), "exit {status:?}, stderr:\n{stderr}");
    assert!(stderr.contains("LibreCN stopped\""), "{stderr}");
}
