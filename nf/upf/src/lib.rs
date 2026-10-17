//! UPF, the user plane function, hosted in one EO.
//!
//! This release keeps the sessions the SMF sets over N4: for each, the UE's
//! address, the uplink tunnel endpoint it allocates on N3 (its address and
//! a TEID) and the downlink one, the gNB's. Forwarding (GTP-U on N3, the
//! TUN device on N6) comes next.

use std::collections::HashMap;
use std::net::Ipv4Addr;

use librecn_common::log;
use librecn_common::log::Logger;
use librecn_common::msg::flags;
use librecn_sbi::Outcome;
use librecn_sbi::pdu_session::{N4Delete, N4Establish, N4EstablishResult, N4Modify, N4ModifyResult};
use librecn_scheduler::{Context, EoConfig, ExecutionObject, Message};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Params {
    /// N3 address announced to the gNBs. Not 127.0.0.1: a gNB on the same
    /// host (UERANSIM) binds the GTP-U port there.
    n3: Ipv4Addr,
    max_sessions: usize,
}

impl Default for Params {
    fn default() -> Self {
        Params { n3: Ipv4Addr::new(127, 0, 0, 2), max_sessions: 1024 }
    }
}

/// One PDU session's forwarding state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Session {
    ue_ip: Ipv4Addr,
    ul_teid: u32,
    /// gNB address and TEID, once known.
    dl: Option<(Ipv4Addr, u32)>,
}

/// The UPF EO.
pub struct Upf {
    name: String,
    n3: Ipv4Addr,
    max_sessions: usize,
    sessions: HashMap<u64, Session>,
    next_teid: u32,
    log: Logger,
}

impl Upf {
    /// From deployment `params`: `n3` (default 127.0.0.2) and `max_sessions` (1024).
    pub fn new(name: &str, params: &serde_json::Value) -> Result<Self, String> {
        let p: Params = if params.is_null() {
            Params::default()
        } else {
            serde_json::from_value(params.clone()).map_err(|e| format!("invalid UPF parameters: {e}"))?
        };
        Ok(Upf {
            name: name.to_string(),
            n3: p.n3,
            max_sessions: p.max_sessions,
            sessions: HashMap::with_capacity(p.max_sessions),
            next_teid: 1,
            log: Logger::default(),
        })
    }

    /// For the control EO's factory.
    pub fn from_params(name: &str, params: &serde_json::Value) -> Result<(EoConfig, Self), String> {
        Ok((Self::eo_config(name), Self::new(name, params)?))
    }

    pub fn eo_config(name: &str) -> EoConfig {
        EoConfig::new(name).max_timers(0).buffers(&[(256, 64)])
    }

    /// A TEID no session uses (0 is reserved).
    fn teid(&mut self) -> u32 {
        loop {
            let teid = self.next_teid;
            self.next_teid = self.next_teid.wrapping_add(1).max(1);
            if !self.sessions.values().any(|s| s.ul_teid == teid) {
                return teid;
            }
        }
    }

    fn establish(&mut self, request: &N4Establish) -> Result<Session, Outcome> {
        if let Some(existing) = self.sessions.get(&request.seid) {
            return Ok(*existing); // a retransmission
        }
        if self.sessions.len() >= self.max_sessions {
            return Err(Outcome::Unavailable);
        }
        let session = Session { ue_ip: Ipv4Addr::from(request.ue_ip), ul_teid: self.teid(), dl: None };
        self.sessions.insert(request.seid, session);
        Ok(session)
    }

    fn on_establish(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &N4Establish) {
        let result = self.establish(request);
        match result {
            Ok(s) => {
                log!(self.log, ctx, Info, "session {:#x}: UE {}, uplink TEID {:#x}", request.seid, s.ue_ip, s.ul_teid)
            }
            Err(e) => log!(self.log, ctx, Error, "session {:#x} refused: {e:?}", request.seid),
        }
        let (n3, seid) = (self.n3, request.seid);
        let sent = ctx.alloc::<N4EstablishResult>(msg.from()).and_then(|r| {
            let mut r = r.with_corr_id(msg.corr_id()).with_flags(flags::RESPONSE);
            r.seid = seid;
            match result {
                Ok(s) => (r.n3_addr, r.ul_teid) = (n3.octets(), s.ul_teid),
                Err(outcome) => r.set_outcome(outcome),
            }
            ctx.send(r)
        });
        if let Err(e) = sent {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", msg.from());
        }
    }

    fn on_modify(&mut self, ctx: &mut Context<'_>, msg: &Message<'_>, request: &N4Modify) {
        let outcome = match self.sessions.get_mut(&request.seid) {
            Some(s) => {
                s.dl = Some((Ipv4Addr::from(request.gnb_addr), request.dl_teid));
                log!(
                    self.log,
                    ctx,
                    Info,
                    "session {:#x}: downlink to {} TEID {:#x}",
                    request.seid,
                    Ipv4Addr::from(request.gnb_addr),
                    request.dl_teid
                );
                Outcome::Ok
            }
            None => Outcome::ContextNotFound,
        };
        let seid = request.seid;
        let sent = ctx.alloc::<N4ModifyResult>(msg.from()).and_then(|r| {
            let mut r = r.with_corr_id(msg.corr_id()).with_flags(flags::RESPONSE);
            r.seid = seid;
            r.set_outcome(outcome);
            ctx.send(r)
        });
        if let Err(e) = sent {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", msg.from());
        }
    }
}

impl ExecutionObject for Upf {
    fn on_start(&mut self, ctx: &mut Context<'_>) {
        log!(self.log, ctx, Vip, "UPF {} ready, N3 {}", self.name, self.n3);
    }

    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>) {
        if let Some(request) = msg.body::<N4Establish>() {
            self.on_establish(ctx, msg, request);
        } else if let Some(request) = msg.body::<N4Modify>() {
            self.on_modify(ctx, msg, request);
        } else if let Some(request) = msg.body::<N4Delete>()
            && self.sessions.remove(&request.seid).is_some()
        {
            log!(self.log, ctx, Info, "session {:#x} deleted, {} left", request.seid, self.sessions.len());
        }
    }

    fn on_stop(&mut self, ctx: &mut Context<'_>) {
        log!(self.log, ctx, Info, "UPF {} stopping, {} session(s)", self.name, self.sessions.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use librecn_scheduler::testbed::{Probe, TestBed};

    #[test]
    fn sessions_get_tunnels_and_are_bounded() {
        let mut tb = TestBed::new();
        let (cfg, upf) = Upf::from_params("upf", &serde_json::json!({ "max_sessions": 2 })).unwrap();
        let upf = tb.add(cfg, upf).unwrap();
        let (probe, seen) = Probe::new();
        let smf = tb.add(EoConfig::new("smf").buffers(&[(256, 8)]), probe).unwrap();
        let establish = |tb: &mut TestBed, seid: u64| {
            tb.inject_from(smf, upf, 7, &N4Establish::new(seid, [10, 45, 0, 2])).unwrap();
            tb.run_until_idle();
            seen.take()[0].body::<N4EstablishResult>().unwrap()
        };
        let a = establish(&mut tb, 1);
        assert_eq!((a.outcome(), a.n3_addr, a.seid), (Outcome::Ok, [127, 0, 0, 2], 1));
        let b = establish(&mut tb, 2);
        assert_ne!(a.ul_teid, b.ul_teid);
        assert_eq!(establish(&mut tb, 1).ul_teid, a.ul_teid, "a retransmission gets the same tunnel");
        assert_eq!(establish(&mut tb, 3).outcome(), Outcome::Unavailable, "full");

        tb.inject_from(smf, upf, 8, &N4Modify { seid: 9, gnb_addr: [127, 0, 0, 1], dl_teid: 5 }).unwrap();
        tb.run_until_idle();
        assert_eq!(seen.take()[0].body::<N4ModifyResult>().unwrap().outcome(), Outcome::ContextNotFound);
        tb.inject_from(smf, upf, 0, &N4Delete { seid: 2 }).unwrap();
        tb.run_until_idle();
        assert_eq!(establish(&mut tb, 3).outcome(), Outcome::Ok, "room again");
        assert!(Upf::new("upf", &serde_json::json!({ "n3": "nowhere" })).is_err());
    }
}
