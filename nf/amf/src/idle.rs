//! UEs in CM-IDLE (TS 23.502 §4.2.6): a registered UE whose N2 connection
//! goes keeps its 5GMM context and its PDU sessions, their user plane
//! deactivated, until it comes back (Service Request, Registration Request)
//! or the mobile reachable timer expires: the UE is then implicitly
//! deregistered (TS 24.501 §5.3.7).

use std::time::Duration;

use librecn_common::log;
use librecn_sbi::pdu_session::{UpdateSmContext, update};
use librecn_scheduler::{Context, Time};

use crate::mm::{self, Mm, State};
use crate::ue::{AmfUeId, PduSession, UeContext};
use crate::{Amf, request};

/// The mobile reachable timer: T3512's default (54 min, the AMF sends no
/// other value) plus 4 minutes (§5.3.7).
pub(crate) const MOBILE_REACHABLE: Duration = Duration::from_secs(58 * 60);
/// Expired idle contexts are swept at this period.
pub(crate) const SWEEP: Duration = Duration::from_secs(60);

/// What the AMF keeps of an idle UE.
pub(crate) struct Idle {
    mm: Mm,
    sessions: [Option<PduSession>; 16],
    since: Time,
}

impl Idle {
    /// The session with SMF context `sm_ctx`, if this UE has it, is gone.
    pub(crate) fn forget_session(&mut self, sm_ctx: u32) {
        for slot in &mut self.sessions {
            if slot.is_some_and(|s| s.sm_ctx == Some(sm_ctx)) {
                *slot = None;
            }
        }
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
            request::<UpdateSmContext>(ctx, &mut self.smf, &mut self.log, ue.amf_id, |m| {
                (m.sm_ctx, m.kind) = (sm_ctx, update::DEACTIVATE);
            });
        }
        let idle = Idle { mm: ue.mm, sessions: ue.sessions, since: ctx.now() };
        self.idle.insert(tmsi, idle);
        log!(self.log, ctx, Info, "UE {}: idle (5G-TMSI {tmsi:08x}), {} idle UE(s)", ue.amf_id, self.idle.len());
        true
    }

    /// A new N2 context: the UE's idle context, if its initial message names one.
    pub(crate) fn resume(&mut self, ctx: &mut Context<'_>, amf_id: AmfUeId, nas: &[u8]) {
        let Some(tmsi) = mm::initial_tmsi(nas, &self.config) else { return };
        let Some(idle) = self.idle.remove(&tmsi) else { return };
        let Some(ue) = self.ues.get_mut_by_id(amf_id) else { return };
        let secs = (ctx.now() - idle.since).as_secs();
        (ue.mm, ue.sessions) = (idle.mm, idle.sessions);
        log!(self.log, ctx, Info, "UE {amf_id}: context of 5G-TMSI {tmsi:08x} resumed after {secs} s idle");
    }

    /// Implicit deregistration of the UEs idle for too long.
    pub(crate) fn sweep_idle(&mut self, ctx: &mut Context<'_>) {
        let now = ctx.now();
        let expired: Vec<u32> =
            self.idle.iter().filter(|(_, i)| now - i.since >= MOBILE_REACHABLE).map(|(tmsi, _)| *tmsi).collect();
        for tmsi in expired {
            let Some(idle) = self.idle.remove(&tmsi) else { continue };
            for sm_ctx in idle.sessions.iter().flatten().filter_map(|s| s.sm_ctx) {
                self.release_sm_context(ctx, sm_ctx);
            }
            let supi = idle.mm.supi().map_or_else(String::new, |s| format!(" ({s})"));
            log!(self.log, ctx, Info, "5G-TMSI {tmsi:08x}{supi}: implicitly deregistered, unreachable");
        }
    }
}
