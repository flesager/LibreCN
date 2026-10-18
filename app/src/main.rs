//! `librecn [--config FILE]`: starts the runtime and the OAM EOs, then the
//! NFs listed in the configuration (more can be started through the web API),
//! and runs until SIGTERM or SIGINT.

mod config;
mod demo;

use std::process::ExitCode;
use std::time::Duration;

use librecn_common::log::{self, Level};
use librecn_oam::proto::Deploy;
use librecn_oam::{AlarmManager, Control, Deployment, LogSink, Server};
use librecn_scheduler::runtime::Runtime;
use librecn_scheduler::{EoConfig, ExecutionObject};
use signal_hook::consts::{SIGINT, SIGTERM};
use zerocopy::FromZeros;

use config::AppConfig;

const NAME: &str = "librecn";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            log::print(NAME, Level::Fatal, format_args!("{e}"));
            ExitCode::FAILURE
        }
    }
}

/// A control EO factory from an NF's `from_params`.
fn factory<E, F>(from_params: F) -> librecn_oam::Factory
where
    E: ExecutionObject,
    F: Fn(&str, &serde_json::Value) -> Result<(EoConfig, E), String> + Send + 'static,
{
    Box::new(move |name, params| {
        let (cfg, eo) = from_params(name, params)?;
        Ok((cfg, Box::new(eo) as Box<dyn ExecutionObject>))
    })
}

fn run() -> Result<(), String> {
    let text = match std::env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [] => None,
        [flag, path] if flag == "--config" || flag == "-c" => {
            Some(std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?)
        }
        _ => return Err("usage: librecn [--config FILE]".into()),
    };
    let app = text.as_deref().map(AppConfig::parse).transpose()?.unwrap_or_default();
    let level =
        Level::parse(&app.oam.log_level).ok_or_else(|| format!("log_level {:?}: unknown", app.oam.log_level))?;
    log::set_default_level(level);
    if let Some(dir) = &app.oam.state_dir {
        librecn_common::state::set_dir(dir.clone());
    }
    let nfs = text.as_deref().map(Deployment::parse).transpose()?.map_or(0, |d| d.nf.len());
    let mut signals = signal_hook::iterator::Signals::new([SIGINT, SIGTERM]).map_err(|e| format!("signals: {e}"))?;

    let mut runtime = Runtime::new(app.runtime()?).map_err(|e| format!("cannot start the runtime: {e}"))?;
    let add_failed = |e| format!("cannot start the OAM EOs: {e}");
    runtime.add(LogSink::config(), LogSink::stdout()).map_err(add_failed)?;
    runtime.add(AlarmManager::config(), AlarmManager::new()).map_err(add_failed)?;
    let control = Control::new()
        .kind("demo", demo::factory())
        .kind("amf", factory(librecn_amf::Amf::from_params))
        .kind("ausf", factory(librecn_ausf::Ausf::from_params))
        .kind("udm", factory(librecn_udm::Udm::from_params))
        .kind("smf", factory(librecn_smf::Smf::from_params))
        .kind("upf", factory(librecn_upf::Upf::from_params));
    let control = runtime.add(Control::config(), control).map_err(add_failed)?;
    runtime.add(Server::config(), Server::new(app.oam.listen)).map_err(add_failed)?;
    log::print(NAME, Level::Vip, format_args!("LibreCN started, {nfs} NF(s) to deploy"));

    if let Some(text) = text.filter(|_| nfs > 0) {
        // Through the control EO, like a deployment posted to the web API.
        let mut deploy = Deploy::new_zeroed();
        if !deploy.fill(&text) {
            return Err("configuration too large to deploy".into());
        }
        runtime.inject(control, &deploy).map_err(|e| format!("cannot deploy: {e}"))?;
    }

    if let Some(signal) = signals.forever().next() {
        log::print(NAME, Level::Info, format_args!("signal {signal}, stopping"));
    }
    let clean = runtime.shutdown(Duration::from_secs(5));
    log::print(NAME, Level::Vip, format_args!("LibreCN stopped{}", if clean { "" } else { " (timeout)" }));
    if clean { Ok(()) } else { Err("some EOs did not stop in time".into()) }
}
