//! The application configuration file (TOML). Every section is optional.
//!
//! ```toml
//! [runtime]
//! max_eos = 256
//! fault_policy = "abort"        # or "isolate"
//!
//! [[runtime.pools]]             # default: "cp" with one worker per core but one
//! name = "cp"
//! cores = [2, 3]                # pinned workers; or `workers = 2` unpinned
//!
//! [oam]
//! listen = "127.0.0.1:8080"
//!
//! [[nf]]                        # NFs to start, see librecn_oam::Deployment
//! kind = "demo"
//! name = "demo-1"
//! ```
//!
//! The `oam` pool (one unpinned worker) is added unless the file defines it.

use std::net::SocketAddr;
use std::time::Duration;

use librecn_scheduler::FaultPolicy;
use librecn_scheduler::runtime::{PoolConfig, RuntimeConfig};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub runtime: RuntimeSection,
    pub oam: OamSection,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RuntimeSection {
    pub max_eos: u32,
    pub tick_ms: u64,
    pub fault_policy: String,
    pub pools: Vec<PoolSection>,
}

impl Default for RuntimeSection {
    fn default() -> Self {
        RuntimeSection { max_eos: 256, tick_ms: 1, fault_policy: "abort".into(), pools: Vec::new() }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PoolSection {
    pub name: String,
    pub workers: Option<usize>,
    #[serde(default)]
    pub cores: Vec<usize>,
    pub budget: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OamSection {
    pub listen: SocketAddr,
    /// Threshold of every EO's logs: vip, fatal, error, info or debug.
    pub log_level: String,
}

impl Default for OamSection {
    fn default() -> Self {
        OamSection { listen: SocketAddr::from(([127, 0, 0, 1], 8080)), log_level: "info".into() }
    }
}

impl AppConfig {
    pub fn parse(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|e| format!("invalid configuration: {e}"))
    }

    pub fn runtime(&self) -> Result<RuntimeConfig, String> {
        let r = &self.runtime;
        let mut pools: Vec<PoolConfig> = r
            .pools
            .iter()
            .map(|p| {
                let pool = if p.cores.is_empty() {
                    PoolConfig::new(&p.name, p.workers.unwrap_or(1))
                } else {
                    PoolConfig::pinned(&p.name, &p.cores)
                };
                pool.budget(p.budget.unwrap_or(16))
            })
            .collect();
        if pools.is_empty() {
            let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
            pools.push(PoolConfig::new("cp", cores.saturating_sub(1).max(1)));
        }
        if !pools.iter().any(|p| p.name == librecn_oam::POOL) {
            pools.push(PoolConfig::new(librecn_oam::POOL, 1));
        }
        let mut cfg = RuntimeConfig::new(pools);
        cfg.max_eos = r.max_eos;
        cfg.tick = Duration::from_millis(r.tick_ms);
        cfg.fault_policy = match r.fault_policy.as_str() {
            "abort" => FaultPolicy::Abort,
            "isolate" => FaultPolicy::Isolate,
            other => return Err(format!("fault_policy must be \"abort\" or \"isolate\", not {other:?}")),
        };
        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_file_gives_cp_then_oam() {
        let cfg = AppConfig::parse("").unwrap().runtime().unwrap();
        let names: Vec<_> = cfg.pools.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["cp", "oam"]);
        assert_eq!(cfg.fault_policy, FaultPolicy::Abort);
    }

    #[test]
    fn pools_and_nfs_in_one_file() {
        let text = r#"
            [runtime]
            fault_policy = "isolate"
            [[runtime.pools]]
            name = "up"
            cores = [0]
            [[runtime.pools]]
            name = "oam"
            workers = 2
            [oam]
            listen = "0.0.0.0:9000"
            [[nf]]
            kind = "demo"
            name = "d"
        "#;
        let app = AppConfig::parse(text).unwrap();
        let cfg = app.runtime().unwrap();
        assert_eq!(cfg.pools.iter().map(|p| (p.name.as_str(), p.workers)).collect::<Vec<_>>(), [("up", 1), ("oam", 2)]);
        assert_eq!(app.oam.listen.port(), 9000);
        assert_eq!(librecn_oam::Deployment::parse(text).unwrap().nf.len(), 1);
    }

    #[test]
    fn example_file_is_valid() {
        let text = include_str!("../librecn.example.toml");
        AppConfig::parse(text).unwrap().runtime().unwrap();
        let nfs = librecn_oam::Deployment::parse(text).unwrap().nf;
        let kinds: Vec<_> = nfs.iter().map(|n| n.kind.as_str()).collect();
        assert_eq!(kinds, ["udm", "ausf", "upf", "smf", "amf", "demo"]);
        // The NFs' parameters are valid too.
        librecn_udm::Udm::from_params("udm", &nfs[0].params).unwrap();
        librecn_ausf::Ausf::from_params("ausf", &nfs[1].params).unwrap();
        librecn_upf::Upf::from_params("upf", &nfs[2].params).unwrap();
        librecn_smf::Smf::from_params("smf", &nfs[3].params).unwrap();
        librecn_amf::Amf::from_params("amf", &nfs[4].params).unwrap();
    }

    #[test]
    fn typos_are_refused() {
        assert!(AppConfig::parse("[runtime]\nmax_eo = 3").is_err());
        assert!(AppConfig::parse("[runtime]\nfault_policy = \"panic\"").unwrap().runtime().is_err());
    }
}
