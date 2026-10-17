//! Messages between the OAM EOs: fixed-size requests, JSON replies.
//!
//! Every request carries a `corr_id` and gets one [`Reply`] with the same
//! `corr_id`, whose text is the JSON body of the matching web API response.

use librecn_common::msg::{family, id};
use librecn_scheduler::{Context, Message, MsgBody, Result};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// Header-only: list the NF kinds `Control` can start.
pub const KINDS: u32 = id(family::OAM_CONTROL, 3);
/// Header-only: list the active alarms.
pub const ALARMS: u32 = id(family::OAM_CONTROL, 5);

/// Text bytes of a [`Deploy`].
pub const DEPLOY_TEXT: usize = 64 * 1024 - 8;
/// Text bytes of a [`Reply`].
pub const REPLY_TEXT: usize = 16 * 1024 - 8;

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

/// Answer to a request: success flag and JSON text. 16 KiB.
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Reply {
    ok: u32,
    len: u32,
    text: [u8; REPLY_TEXT],
}

impl MsgBody for Reply {
    const MSG_ID: u32 = id(family::OAM_CONTROL, 4);
}

macro_rules! text_body {
    ($ty:ident, $max:ident) => {
        impl $ty {
            fn set_text(&mut self, text: &str) -> bool {
                let fits = text.len() <= $max;
                let n = if fits { text.len() } else { 0 };
                self.text[..n].copy_from_slice(&text.as_bytes()[..n]);
                self.len = n as u32;
                fits
            }

            /// The text (empty if it was not valid UTF-8).
            pub fn text(&self) -> &str {
                std::str::from_utf8(&self.text[..(self.len as usize).min($max)]).unwrap_or_default()
            }
        }
    };
}

text_body!(Deploy, DEPLOY_TEXT);
text_body!(Reply, REPLY_TEXT);

impl Deploy {
    /// Fills a zeroed body (from `Context::alloc`); false if `text` is too long.
    pub fn fill(&mut self, text: &str) -> bool {
        self.set_text(text)
    }
}

impl Reply {
    /// Fills a zeroed body; a text too long becomes an error reply.
    pub fn fill(&mut self, ok: bool, json: &str) {
        let fits = self.set_text(json);
        self.ok = u32::from(ok && fits);
        if !fits {
            self.set_text(r#"{"error":"reply too large"}"#);
        }
    }

    /// Whether the request succeeded.
    pub fn ok(&self) -> bool {
        self.ok != 0
    }
}

/// Answers `request` (same `corr_id`). The requester needs a buffer class of
/// at least `size_of::<Reply>()` (16 KiB).
pub fn send_reply(ctx: &mut Context<'_>, request: &Message<'_>, ok: bool, json: &str) -> Result<()> {
    let mut reply = ctx.alloc::<Reply>(request.from())?.with_corr_id(request.corr_id());
    reply.fill(ok, json);
    ctx.send(reply)
}
