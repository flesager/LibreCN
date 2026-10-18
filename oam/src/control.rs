//! The control EO: starts NFs from a deployment, stops EOs, supervises.

use std::collections::HashSet;
use std::time::Duration;

use librecn_common::alarm::{Alarms, Severity};
use librecn_common::log::Logger;
use librecn_common::{log, names};
use librecn_scheduler::{Context, EoConfig, EoId, ExecutionObject, Message};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::proto::{self, Deploy, Stop};

/// Builds an NF from its name and parameters: its EO sizes (pool overridden
/// by the deployment if given) and the EO itself.
pub type Factory = Box<dyn Fn(&str, &serde_json::Value) -> Result<(EoConfig, Box<dyn ExecutionObject>), String> + Send>;

/// NFs to start. TOML:
///
/// ```toml
/// [[nf]]
/// kind = "demo"
/// name = "demo-1"
/// pool = "cp"          # optional
/// [nf.params]          # optional, passed to the kind's factory
/// period_ms = 500
/// ```
///
/// or the same in JSON: `{"nf": [{"kind": "demo", "name": "demo-1"}]}`.
/// Other top-level tables are ignored, so a whole application configuration
/// file is a valid deployment.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Deployment {
    /// NFs, started in order.
    #[serde(default)]
    pub nf: Vec<NfSpec>,
}

/// One NF of a [`Deployment`].
#[derive(Clone, Debug, Deserialize)]
pub struct NfSpec {
    /// Factory registered with [`Control::kind`].
    pub kind: String,
    /// EO name, unique among the running EOs.
    pub name: String,
    /// Pool, instead of the factory's choice.
    pub pool: Option<String>,
    /// Kind-specific parameters.
    #[serde(default)]
    pub params: serde_json::Value,
}

impl Deployment {
    /// JSON if the text starts with `{`, TOML otherwise.
    pub fn parse(text: &str) -> Result<Self, String> {
        if text.trim_start().starts_with('{') {
            serde_json::from_str(text).map_err(|e| format!("invalid JSON deployment: {e}"))
        } else {
            toml::from_str(text).map_err(|e| format!("invalid TOML deployment: {e}"))
        }
    }
}

#[derive(Serialize)]
struct Started {
    name: String,
    id: u64,
}

#[derive(Serialize)]
struct Failed {
    name: String,
    error: String,
}

/// Supervision period.
const SUPERVISION: Duration = Duration::from_secs(1);

/// Names that are never stopped through the API.
const PROTECTED: [&str; 4] = [names::LOG, names::ALARM, names::CONTROL, "server"];

/// Control EO (name [`names::CONTROL`]). Handles [`Deploy`], [`Stop`],
/// [`proto::KINDS`] and [`proto::NFS`], each answered by a [`Reply`]; every
/// second, raises a critical alarm for each EO that faulted.
pub struct Control {
    kinds: Vec<(String, Factory)>,
    /// The NFs started: EO, kind (the GUI draws each by its kind).
    started: Vec<(EoId, String)>,
    faulted: HashSet<EoId>,
    log: Logger,
    alarms: Alarms,
}

impl Default for Control {
    fn default() -> Self {
        Control::new()
    }
}

impl Control {
    /// No NF kind known yet.
    pub fn new() -> Self {
        Control {
            kinds: Vec::new(),
            started: Vec::new(),
            faulted: HashSet::new(),
            log: Logger::default(),
            alarms: Alarms::new(),
        }
    }

    /// Registers the factory of an NF kind.
    pub fn kind(mut self, kind: impl Into<String>, factory: Factory) -> Self {
        self.kinds.push((kind.into(), factory));
        self
    }

    /// Name `control`, OAM pool, room for a 64 KiB deployment.
    pub fn config() -> EoConfig {
        EoConfig::new(names::CONTROL).pool(crate::POOL).buffers(&[(64, 64), (64 * 1024, 2)]).max_io(0)
    }

    fn deploy(&mut self, ctx: &mut Context<'_>, text: &str) -> (bool, String) {
        let deployment = match Deployment::parse(text) {
            Ok(d) => d,
            Err(e) => {
                log!(self.log, ctx, Error, "{e}");
                return (false, json!({ "error": e }).to_string());
            }
        };
        let (mut started, mut failed) = (Vec::new(), Vec::new());
        for spec in deployment.nf {
            match self.start(ctx, &spec) {
                Ok(id) => {
                    log!(self.log, ctx, Info, "started {} ({}) as {id}", spec.name, spec.kind);
                    self.started.push((id, spec.kind.clone()));
                    started.push(Started { name: spec.name, id: id.raw() });
                }
                Err(error) => {
                    log!(self.log, ctx, Error, "cannot start {} ({}): {error}", spec.name, spec.kind);
                    failed.push(Failed { name: spec.name, error });
                }
            }
        }
        (failed.is_empty(), json!({ "started": started, "failed": failed }).to_string())
    }

    fn start(&self, ctx: &mut Context<'_>, spec: &NfSpec) -> Result<EoId, String> {
        if ctx.find(&spec.name).is_some() {
            return Err("an EO with this name runs already".into());
        }
        let (_, factory) = self.kinds.iter().find(|(k, _)| *k == spec.kind).ok_or("unknown kind")?;
        let (mut cfg, eo) = factory(&spec.name, &spec.params)?;
        cfg.name.clone_from(&spec.name);
        if spec.pool.is_some() {
            cfg.pool.clone_from(&spec.pool);
        }
        ctx.spawn(cfg, eo).map_err(|e| e.to_string())
    }

    fn stop(&mut self, ctx: &mut Context<'_>, id: EoId) -> (bool, String) {
        let Some(info) = ctx.eos().into_iter().find(|e| e.id == id) else {
            return (false, json!({ "error": "unknown EO" }).to_string());
        };
        if PROTECTED.contains(&info.name.as_str()) {
            return (false, json!({ "error": "OAM EOs cannot be stopped" }).to_string());
        }
        match ctx.stop(id) {
            Ok(()) => {
                log!(self.log, ctx, Info, "stopping {}", info.name);
                (true, json!({ "stopping": info.name }).to_string())
            }
            Err(e) => (false, json!({ "error": e.to_string() }).to_string()),
        }
    }

    fn supervise(&mut self, ctx: &mut Context<'_>) {
        for eo in ctx.eos().into_iter().filter(|e| !e.alive) {
            if self.faulted.insert(eo.id) {
                log!(self.log, ctx, Fatal, "EO {} faulted", eo.name);
                let text = format!("EO {} faulted", eo.name);
                self.alarms.raise(ctx, eo.id.index(), Severity::Critical, &text);
            }
        }
    }

    fn reply(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, (ok, json): (bool, String)) {
        // Deployments from the application's configuration come from SYSTEM: logged, not answered.
        if msg.from() == EoId::SYSTEM {
            return;
        }
        if let Err(e) = crate::proto::send_reply(ctx, msg, if ok { 200 } else { 400 }, &json) {
            log!(self.log, ctx, Error, "cannot reply to {}: {e}", msg.from());
        }
    }
}

impl ExecutionObject for Control {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        let _ = ctx.arm_timer(SUPERVISION, 0);
    }

    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if msg.timer().is_some() {
            self.supervise(ctx);
            let _ = ctx.arm_timer(SUPERVISION, 0);
        } else if let Some(deploy) = msg.body::<Deploy>() {
            let answer = self.deploy(ctx, deploy.text());
            self.reply(ctx, msg, answer);
        } else if let Some(stop) = msg.body::<Stop>() {
            let answer = self.stop(ctx, EoId::from_raw(stop.eo));
            self.reply(ctx, msg, answer);
        } else if msg.id() == proto::NFS {
            // Those still known to the scheduler (running or faulted).
            let eos = ctx.eos();
            self.started.retain(|(id, _)| eos.iter().any(|e| e.id == *id));
            let nfs: Vec<_> = self
                .started
                .iter()
                .filter_map(|(id, kind)| eos.iter().find(|e| e.id == *id).map(|e| (e, kind)))
                .map(|(e, kind)| json!({ "name": e.name, "kind": kind, "id": e.id.raw(), "alive": e.alive }))
                .collect();
            let answer = (true, json!(nfs).to_string());
            self.reply(ctx, msg, answer);
        } else if msg.id() == proto::KINDS {
            let kinds: Vec<&str> = self.kinds.iter().map(|(k, _)| k.as_str()).collect();
            let answer = (true, json!(kinds).to_string());
            self.reply(ctx, msg, answer);
        }
    }
}
