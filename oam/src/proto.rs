//! Messages between the OAM EOs: fixed-size requests, JSON replies.
//!
//! Every request carries a `corr_id` and gets one [`Reply`] with the same
//! `corr_id`, whose text is the JSON body of the matching web API response.

pub use librecn_common::manage::{REPLY_TEXT, Reply, send_reply};
use librecn_common::msg::{family, id};
use librecn_scheduler::MsgBody;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// Header-only: list the NF kinds `Control` can start.
pub const KINDS: u32 = id(family::OAM_CONTROL, 3);
/// Header-only: list the NFs `Control` started (name, kind, EO id).
pub const NFS: u32 = id(family::OAM_CONTROL, 7);
/// Header-only: list the active alarms.
pub const ALARMS: u32 = id(family::OAM_CONTROL, 5);

/// Text bytes of a [`Deploy`].
pub const DEPLOY_TEXT: usize = 64 * 1024 - 8;

/// Start the NFs of a deployment (TOML or JSON, see [`Deployment`](crate::Deployment)). 64 KiB.
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Deploy {
    len: u32,
    _pad: u32,
    text: [u8; DEPLOY_TEXT],
}

impl MsgBody for Deploy {
    const MSG_ID: u32 = id(family::OAM_CONTROL, 1);
}

/// Stop an EO.
#[derive(Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Stop {
    /// `EoId::raw()` of the EO.
    pub eo: u64,
}

impl MsgBody for Stop {
    const MSG_ID: u32 = id(family::OAM_CONTROL, 2);
}

impl Deploy {
    /// Fills a zeroed body (from `Context::alloc`); false if `text` is too long.
    pub fn fill(&mut self, text: &str) -> bool {
        let fits = text.len() <= DEPLOY_TEXT;
        let n = if fits { text.len() } else { 0 };
        self.text[..n].copy_from_slice(&text.as_bytes()[..n]);
        self.len = n as u32;
        fits
    }

    /// The text (empty if it was not valid UTF-8).
    pub fn text(&self) -> &str {
        std::str::from_utf8(&self.text[..(self.len as usize).min(DEPLOY_TEXT)]).unwrap_or_default()
    }
}
