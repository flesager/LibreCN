//! PDU sessions as the AMF carries them (TS 23.502 §4.3.2.2.1): the UE's
//! 5GSM message goes to the SMF (5GMM, `mm`); the SMF's answer goes to the
//! UE alone (a reject) or with the gNB's setup (PDU Session Resource Setup);
//! the gNB's answer goes back to the SMF. The AMF reads neither N1 nor N2.

use librecn_common::log;
use librecn_sbi::pdu_session::{CreateSmContextResult, ReleaseSmContext, UpdateSmContext, n2 as n2_kind};
use librecn_scheduler::Context;

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
                let request = n2::pdu_session_resource_setup_request(amf_id, ran_id, psi, &nas[..n], slice, transfer);
                port.ngap(request, "PDU Session Resource Setup Request");
            }
        });
    }

    /// The gNB's PDU Session Resource Setup Response: each transfer to the SMF.
    pub(crate) fn on_session_setup(
        &mut self,
        ctx: &mut Context<'_>,
        amf_id: AmfUeId,
        setup: Vec<(u8, Vec<u8>)>,
        failed: Vec<(u8, Vec<u8>)>,
    ) {
        let answers = setup.into_iter().map(|a| (a, n2_kind::SETUP_RESPONSE));
        for ((psi, transfer), kind) in answers.chain(failed.into_iter().map(|a| (a, n2_kind::SETUP_FAILED))) {
            let Some(ue) = self.ues.get_mut_by_id(amf_id) else { return };
            let Some(slot) = ue.sessions.get_mut(usize::from(psi)) else { continue };
            let Some(sm_ctx) = slot.and_then(|s| s.sm_ctx) else {
                log!(self.log, ctx, Info, "UE {amf_id}: setup answer for unknown PDU session {psi}");
                continue;
            };
            if kind == n2_kind::SETUP_FAILED {
                *slot = None;
                log!(self.log, ctx, Info, "UE {amf_id}: the gNB could not set PDU session {psi} up");
            }
            request::<UpdateSmContext>(ctx, &mut self.smf, &mut self.log, amf_id, |m| {
                (m.sm_ctx, m.n2_kind) = (sm_ctx, kind);
                m.n2.set(&transfer);
            });
        }
    }

    /// A UE context is gone: its sessions too, at the SMF.
    pub(crate) fn forget_sessions(&mut self, ctx: &mut Context<'_>, ue: &UeContext) {
        for sm_ctx in ue.sessions.iter().flatten().filter_map(|s| s.sm_ctx) {
            self.release_sm_context(ctx, sm_ctx);
        }
    }

    /// Removes the UEs of a gNB, and their sessions; how many.
    pub(crate) fn forget_gnb(&mut self, ctx: &mut Context<'_>, assoc: crate::sctp::AssocId) -> usize {
        let gone = self.ues.remove_gnb(assoc);
        for ue in &gone {
            self.forget_sessions(ctx, ue);
        }
        gone.len()
    }

    fn release_sm_context(&mut self, ctx: &mut Context<'_>, sm_ctx: u32) {
        if let Some(smf) = self.smf.get(ctx) {
            let _ = ctx.send_value(smf, &ReleaseSmContext { sm_ctx });
        }
    }
}
