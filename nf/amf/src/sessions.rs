//! PDU sessions as the AMF carries them (TS 23.502 §4.3.2.2.1): the UE's
//! 5GSM message goes to the SMF (5GMM, `mm`); the SMF's answer goes to the
//! UE alone (a reject) or with the gNB's setup (PDU Session Resource Setup);
//! the gNB's answer goes back to the SMF. Later, the SMF's N1 and N2 for an
//! existing session (release, reactivation) go the same ways, as do the
//! network's own releases (`SmNotify`). The AMF reads neither N1 nor N2.

use librecn_common::log;
use librecn_sbi::Outcome;
use librecn_sbi::pdu_session::{
    CreateSmContextResult, ReleaseSmContext, SmNotify, SmNotifyResult, UpdateSmContext, UpdateSmContextResult,
    n2 as n2_kind, update,
};
use librecn_scheduler::{Context, Error, Message};

use crate::ue::{AmfUeId, UeContext};
use crate::{Amf, n2, request};

impl Amf {
    /// The SMF's answer to a UE's request.
    pub(crate) fn on_sm_created(&mut self, ctx: &mut Context<'_>, amf_id: AmfUeId, result: &CreateSmContextResult) {
        let psi = result.pdu_session_id;
        let session = self
            .ues
            .get_mut_by_id(amf_id)
            .and_then(|ue| ue.sessions.get_mut(usize::from(psi)))
            .and_then(|s| s.as_mut().filter(|s| s.sm_ctx.is_none()));
        let Some(session) = session else {
            log!(self.log, ctx, Info, "UE {amf_id}: answer for PDU session {psi} dropped, the session is gone");
            if result.sm_ctx != 0 {
                self.release_sm_context(ctx, result.sm_ctx);
            }
            return;
        };
        let n1 = result.n1.get();
        let transfer = result.n2.get();
        if transfer.is_empty() {
            // Refused: the UE gets the SMF's reject.
            self.ues.get_mut_by_id(amf_id).expect("found above").sessions[usize::from(psi)] = None;
            return self.with_mm(ctx, amf_id, |mm, _, _, port| mm.send_session(psi, n1, port));
        }
        session.sm_ctx = Some(result.sm_ctx);
        let slice = session.snssai;
        self.with_mm(ctx, amf_id, |mm, _, _, port| {
            let mut nas = [0u8; 512];
            if let Some(n) = mm.seal_session(psi, n1, &mut nas, port) {
                let ran_id = port.ids.1;
                let request =
                    n2::pdu_session_resource_setup_request(amf_id, ran_id, psi, Some(&nas[..n]), slice, transfer);
                port.ngap(request, "PDU Session Resource Setup Request");
            }
        });
    }

    /// The SMF's answer to an update: what it carries goes to the UE (N1)
    /// and the gNB (N2: setup again, release).
    pub(crate) fn on_sm_updated(&mut self, ctx: &mut Context<'_>, amf_id: AmfUeId, result: &UpdateSmContextResult) {
        let psi = result.pdu_session_id;
        if result.outcome() != Outcome::Ok {
            return log!(self.log, ctx, Error, "UE {amf_id}: PDU session {psi} not updated: {:?}", result.outcome());
        }
        if result.released != 0 {
            if let Some(slot) = self.ues.get_mut_by_id(amf_id).and_then(|ue| ue.sessions.get_mut(usize::from(psi))) {
                *slot = None;
            }
            return log!(self.log, ctx, Info, "UE {amf_id}: PDU session {psi} released");
        }
        match (result.n2_kind, result.n1.get()) {
            (n2_kind::NONE, []) => log!(self.log, ctx, Info, "UE {amf_id}: PDU session {psi} set up"),
            (n2_kind::NONE, n1) => self.with_mm(ctx, amf_id, |mm, _, _, port| mm.send_session(psi, n1, port)),
            (kind, n1) => {
                self.deliver(ctx, amf_id, psi, kind, n1, result.n2.get());
            }
        }
    }

    /// The SMF's N1N2 message transfer: a release by the network, or
    /// downlink data (N2 setup) for a session without its user plane. To the
    /// UE and its gNB if it is connected; an idle UE is paged for downlink
    /// data; else the SMF ends the session alone.
    pub(crate) fn on_sm_notify(
        &mut self,
        ctx: &mut Context<'_>,
        msg: &Message<'_>,
        amf_id: AmfUeId,
        notify: &SmNotify,
    ) {
        let (psi, sm_ctx) = (notify.pdu_session_id, notify.sm_ctx);
        let connected = self
            .ues
            .get_mut_by_id(amf_id)
            .and_then(|ue| ue.sessions.get(usize::from(psi)).copied().flatten())
            .is_some_and(|s| s.sm_ctx == Some(sm_ctx));
        let delivered = connected && self.deliver(ctx, amf_id, psi, notify.n2_kind, notify.n1.get(), notify.n2.get());
        // Downlink data for an idle UE: it is paged, the session set up once it is back.
        let reached = delivered || (notify.n2_kind == n2_kind::SETUP && self.page(ctx, sm_ctx));
        if !reached {
            self.forget_idle_session(sm_ctx);
            log!(self.log, ctx, Info, "PDU session context {sm_ctx}: the UE not reached, the session ends");
        }
        let outcome = if reached { Outcome::Ok } else { Outcome::ContextNotFound };
        let sent = ctx.alloc::<SmNotifyResult>(msg.from()).and_then(|r| {
            let mut r = r.with_corr_id(msg.corr_id()).with_flags(librecn_common::msg::flags::RESPONSE);
            r.sm_ctx = sm_ctx;
            r.set_outcome(outcome);
            ctx.send(r)
        });
        if let Err(e) = sent {
            log!(self.log, ctx, Error, "cannot answer {}: {e}", msg.from());
        }
    }

    /// N1 and N2 of the SMF to a connected UE and its gNB; false if the UE is gone.
    fn deliver(
        &mut self,
        ctx: &mut Context<'_>,
        amf_id: AmfUeId,
        psi: u8,
        kind: u8,
        n1: &[u8],
        transfer: &[u8],
    ) -> bool {
        let Some(ue) = self.ues.get_mut_by_id(amf_id) else { return false };
        let Some(Some(session)) = ue.sessions.get(usize::from(psi)).copied() else { return false };
        let slice = session.snssai;
        self.with_mm(ctx, amf_id, |mm, _, _, port| {
            let mut sealed = [0u8; 512];
            let nas = match n1 {
                [] => None,
                n1 => mm.seal_session(psi, n1, &mut sealed, port).map(|n| &sealed[..n]),
            };
            let ran_id = port.ids.1;
            if kind == n2_kind::SETUP {
                let request = n2::pdu_session_resource_setup_request(amf_id, ran_id, psi, nas, slice, transfer);
                port.ngap(request, "PDU Session Resource Setup Request");
            } else {
                let command = n2::pdu_session_resource_release_command(amf_id, ran_id, psi, nas, transfer);
                port.ngap(command, "PDU Session Resource Release Command");
            }
        });
        if kind == n2_kind::SETUP {
            log!(self.log, ctx, Info, "UE {amf_id}: PDU session {psi} reactivated");
        }
        true
    }

    /// The gNB's PDU Session Resource Setup Response: each transfer to the SMF.
    pub(crate) fn on_session_setup(
        &mut self,
        ctx: &mut Context<'_>,
        amf_id: AmfUeId,
        setup: Vec<(u8, Vec<u8>)>,
        failed: Vec<(u8, Vec<u8>)>,
    ) {
        let answers = setup.into_iter().map(|a| (a, update::SETUP_RESPONSE));
        for ((psi, transfer), kind) in answers.chain(failed.into_iter().map(|a| (a, update::SETUP_FAILED))) {
            let Some(ue) = self.ues.get_mut_by_id(amf_id) else { return };
            let Some(slot) = ue.sessions.get_mut(usize::from(psi)) else { continue };
            let Some(sm_ctx) = slot.and_then(|s| s.sm_ctx) else {
                log!(self.log, ctx, Info, "UE {amf_id}: setup answer for unknown PDU session {psi}");
                continue;
            };
            if kind == update::SETUP_FAILED {
                *slot = None;
                log!(self.log, ctx, Info, "UE {amf_id}: the gNB could not set PDU session {psi} up");
            }
            request::<UpdateSmContext>(ctx, &mut self.smf, &mut self.log, amf_id, |m| {
                (m.sm_ctx, m.kind) = (sm_ctx, kind);
                m.container.set(&transfer);
            });
        }
    }

    /// A UE context is gone: its sessions too, at the SMF.
    pub(crate) fn forget_sessions(&mut self, ctx: &mut Context<'_>, ue: &UeContext) {
        for sm_ctx in ue.sessions.iter().flatten().filter_map(|s| s.sm_ctx) {
            self.release_sm_context(ctx, sm_ctx);
        }
    }

    /// Removes the UEs of a gNB (the registered ones go idle); how many.
    pub(crate) fn forget_gnb(&mut self, ctx: &mut Context<'_>, assoc: crate::sctp::AssocId) -> usize {
        let gone = self.ues.remove_gnb(assoc);
        let n = gone.len();
        for ue in gone {
            self.retire(ctx, ue);
        }
        n
    }

    pub(crate) fn release_sm_context(&mut self, ctx: &mut Context<'_>, sm_ctx: u32) {
        self.notify_smf(ctx, Notice::Release { sm_ctx });
    }

    /// A notice to the SMF: sent while fewer than [`NOTICE_WINDOW`] wait
    /// for the SMF's acknowledgement, else queued.
    pub(crate) fn notify_smf(&mut self, ctx: &mut Context<'_>, notice: Notice) {
        self.notices.push_back(notice);
        self.flush_notices(ctx);
    }

    /// The SMF acknowledged a notice: the next ones can go.
    pub(crate) fn on_notice_ack(&mut self, ctx: &mut Context<'_>) {
        self.notices_in_flight = self.notices_in_flight.saturating_sub(1);
        self.flush_notices(ctx);
    }

    /// Sends the queued notices in order, [`NOTICE_WINDOW`] at most without
    /// an acknowledgement: a gNB's loss can idle thousands of UEs at once,
    /// and a full SMF mailbox would refuse every other message. One refused
    /// all the same (mailbox or pool full) waits for the retry timer.
    /// Without an SMF they are dropped. Their `corr_id` is 0, no UE's.
    pub(crate) fn flush_notices(&mut self, ctx: &mut Context<'_>) {
        while self.notices_in_flight < NOTICE_WINDOW {
            let Some(&notice) = self.notices.front() else { return };
            let Some(smf) = self.smf.get(ctx) else {
                log!(self.log, ctx, Error, "no SMF: {} session notice(s) dropped", self.notices.len());
                self.notices_in_flight = 0;
                return self.notices.clear();
            };
            let sent = match notice {
                Notice::Release { sm_ctx } => ctx.send_value(smf, &ReleaseSmContext { sm_ctx }),
                Notice::Deactivate { sm_ctx } => ctx.alloc::<UpdateSmContext>(smf).and_then(|m| {
                    let mut m = m.with_flags(librecn_common::msg::flags::REQUEST);
                    (m.sm_ctx, m.kind) = (sm_ctx, update::DEACTIVATE);
                    ctx.send(m)
                }),
            };
            match sent {
                Ok(()) => {
                    self.notices.pop_front();
                    self.notices_in_flight += 1;
                }
                Err(Error::QueueFull | Error::PoolExhausted) => {
                    if self.notice_timer.is_none() {
                        self.notice_timer = ctx.arm_timer(NOTICE_RETRY, 0).ok();
                    }
                    return;
                }
                Err(e) => {
                    self.smf.forget();
                    log!(self.log, ctx, Error, "session notice to the SMF lost: {e}");
                    self.notices.pop_front();
                }
            }
        }
    }
}

/// A one-way notice to the SMF about a session.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Notice {
    /// The UE is idle: no downlink (UpdateSMContext, upCnxState DEACTIVATED).
    Deactivate { sm_ctx: u32 },
    /// The UE is forgotten: the session too (ReleaseSMContext).
    Release { sm_ctx: u32 },
}

/// A notice refused by a full SMF is tried again after this.
const NOTICE_RETRY: std::time::Duration = std::time::Duration::from_millis(1);

/// Notices sent and not yet acknowledged by the SMF, at most.
const NOTICE_WINDOW: usize = 64;
