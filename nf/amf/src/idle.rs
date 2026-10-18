//! UEs in CM-IDLE (TS 23.502 §4.2.6): a registered UE whose N2 connection
//! goes keeps its 5GMM context and its PDU sessions, their user plane
//! deactivated, until it comes back (Service Request, Registration Request)
//! or the mobile reachable timer (T3512 + 4 min) expires: the UE is then
//! implicitly deregistered (TS 24.501 §5.3.7). Downlink data for an idle UE pages it
//! (TS 23.502 §4.2.3.3) in its last tracking area; its Service Request then
//! sets the paged sessions up again.

use std::time::Duration;

use librecn_common::log;
use librecn_nas::ie::SessionSet;
use librecn_scheduler::{Context, Time};

use crate::mm::{self, Mm, State};
use crate::sessions::{Holder, Notice};
use crate::ue::{AmfUeId, Location, PduSession, UeContext};
use crate::{Amf, n2};

/// Expired idle contexts are swept at this period.
pub(crate) const SWEEP: Duration = Duration::from_secs(60);

/// What the AMF keeps of an idle UE.
pub(crate) struct Idle {
    mm: Mm,
    sessions: [Option<PduSession>; 16],
    since: Time,
    /// Its last location: paged in its tracking area, shown under its cell.
    pub(crate) location: Option<Location>,
    /// Sessions it was paged for.
    paged: SessionSet,
    ambr: Option<(u64, u64)>,
}

impl Idle {
    fn sm_contexts(&self) -> impl Iterator<Item = u32> + '_ {
        self.sessions.iter().flatten().filter_map(|s| s.sm_ctx)
    }

    pub(crate) fn supi(&self) -> Option<&librecn_sbi::Supi> {
        self.mm.supi()
    }

    /// For the web API.
    pub(crate) fn describe(&self, tmsi: u32, now: Time) -> serde_json::Value {
        let sessions: Vec<usize> =
            self.sessions.iter().enumerate().filter(|(_, s)| s.is_some()).map(|(p, _)| p).collect();
        serde_json::json!({
            "supi": self.mm.supi().map(|s| s.as_str()),
            "cm": "idle",
            "rrc": "idle",
            "cell": self.location.map(|l| format!("{:09x}", l.cell.1)),
            "tmsi": format!("{tmsi:08x}"),
            "idle_s": (now - self.since).as_secs(),
            "sessions": sessions,
        })
    }
}

impl Amf {
    /// A UE context leaves N2: a registered UE goes idle, its sessions'
    /// user plane deactivated; any other is forgotten, its sessions released.
    /// True if it went idle.
    pub(crate) fn retire(&mut self, ctx: &mut Context<'_>, ue: UeContext) -> bool {
        if let Some(timer) = ue.timer {
            ctx.cancel_timer(timer);
        }
        let tmsi = ue.mm.tmsi().filter(|_| ue.mm.state() == State::Registered);
        let Some(tmsi) = tmsi.filter(|_| self.idle.len() < self.config.max_ues) else {
            self.forget_sessions(ctx, &ue);
            return false;
        };
        for sm_ctx in ue.sessions.iter().flatten().filter_map(|s| s.sm_ctx) {
            self.notify_smf(ctx, Notice::Deactivate { sm_ctx });
            self.holders.insert(sm_ctx, Holder::Idle(tmsi));
        }
        let idle = Idle {
            mm: ue.mm,
            sessions: ue.sessions,
            since: ctx.now(),
            location: ue.location,
            paged: SessionSet::default(),
            ambr: ue.ambr,
        };
        self.idle.insert(tmsi, idle);
        log!(self.log, ctx, Info, "UE {}: idle (5G-TMSI {tmsi:08x}), {} idle UE(s)", ue.amf_id, self.idle.len());
        true
    }

    /// A new N2 context: the UE's idle context, if its initial message names one.
    pub(crate) fn resume(&mut self, ctx: &mut Context<'_>, amf_id: AmfUeId, nas: &[u8]) {
        let Some(tmsi) = mm::initial_tmsi(nas, &self.config) else { return };
        let Some(idle) = self.idle.remove(&tmsi) else { return };
        for sm_ctx in idle.sm_contexts() {
            self.holders.insert(sm_ctx, Holder::Connected(amf_id));
        }
        let Some(ue) = self.ues.get_mut_by_id(amf_id) else { return };
        let secs = (ctx.now() - idle.since).as_secs();
        (ue.mm, ue.sessions, ue.paged, ue.ambr) = (idle.mm, idle.sessions, idle.paged, idle.ambr);
        log!(self.log, ctx, Info, "UE {amf_id}: context of 5G-TMSI {tmsi:08x} resumed after {secs} s idle");
    }

    /// Implicit deregistration of the UEs idle for too long.
    pub(crate) fn sweep_idle(&mut self, ctx: &mut Context<'_>) {
        let (now, reachable) = (ctx.now(), self.config.mobile_reachable());
        let expired: Vec<u32> =
            self.idle.iter().filter(|(_, i)| now - i.since >= reachable).map(|(tmsi, _)| *tmsi).collect();
        for tmsi in expired {
            self.forget_idle(ctx, tmsi, "implicitly deregistered, unreachable");
        }
    }

    /// Deregisters idle UE `tmsi`: its sessions released; `why` logged.
    pub(crate) fn forget_idle(&mut self, ctx: &mut Context<'_>, tmsi: u32, why: &str) {
        let Some(idle) = self.idle.remove(&tmsi) else { return };
        for sm_ctx in idle.sm_contexts() {
            self.release_sm_context(ctx, sm_ctx);
        }
        let supi = idle.mm.supi().map_or_else(String::new, |s| format!(" ({s})"));
        log!(self.log, ctx, Info, "5G-TMSI {tmsi:08x}{supi}: {why}");
    }
}

impl Amf {
    /// The idle UE holding SMF context `sm_ctx` loses that session; false
    /// if no idle UE has it.
    pub(crate) fn forget_idle_session(&mut self, sm_ctx: u32) -> bool {
        let Some(&Holder::Idle(tmsi)) = self.holders.get(&sm_ctx) else { return false };
        self.holders.remove(&sm_ctx);
        if let Some(idle) = self.idle.get_mut(&tmsi) {
            for slot in &mut idle.sessions {
                if slot.is_some_and(|s| s.sm_ctx == Some(sm_ctx)) {
                    *slot = None;
                }
            }
        }
        true
    }

    /// Downlink data for SMF context `sm_ctx` of an idle UE: the UE is
    /// paged by every gNB serving its last tracking area. False if no idle
    /// UE has that session.
    pub(crate) fn page(&mut self, ctx: &mut Context<'_>, sm_ctx: u32) -> bool {
        let Some(&Holder::Idle(tmsi)) = self.holders.get(&sm_ctx) else { return false };
        let Some(idle) = self.idle.get_mut(&tmsi) else { return false };
        let Some(psi) = idle.sessions.iter().position(|s| s.is_some_and(|s| s.sm_ctx == Some(sm_ctx))) else {
            return false;
        };
        idle.paged.insert(psi as u8);
        let Some(tai) = idle.location.map(|l| l.tai) else {
            log!(self.log, ctx, Error, "5G-TMSI {tmsi:08x}: no tracking area known, not paged");
            return true;
        };
        let pdu = n2::paging(&self.config, tmsi, tai);
        let mut gnbs: Vec<_> = self.gnbs.iter().filter(|(_, g)| g.serves(tai)).map(|(assoc, _)| *assoc).collect();
        if gnbs.is_empty() {
            // No gNB declared that tracking area: every gNB pages.
            gnbs = self.gnbs.iter().filter(|(_, g)| g.node.is_some()).map(|(assoc, _)| *assoc).collect();
        }
        for &assoc in &gnbs {
            self.send(ctx, assoc, 0, pdu.clone(), "Paging");
        }
        log!(self.log, ctx, Info, "5G-TMSI {tmsi:08x}: paged for PDU session {psi} by {} gNB(s)", gnbs.len());
        true
    }
}

impl Amf {
    /// The UE-AMBR of idle UE `supi`, for when it is back; false if no idle UE has it.
    pub(crate) fn set_idle_ambr(&mut self, supi: &str, ambr: (u64, u64)) -> bool {
        let idle = self.idle.values_mut().find(|i| i.supi().is_some_and(|s| s.as_str() == supi));
        idle.map(|i| i.ambr = Some(ambr)).is_some()
    }
}
