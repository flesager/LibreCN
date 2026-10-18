//! Managing an NF from the web API: a [`Manage`] request (method, path and
//! body, the HTTP request minus `/api/nfs/{name}/`) answered by a [`Reply`]
//! carrying a status and JSON. The path may carry a query (`?key=value&...`). The web server forwards without knowing the
//! NF; each NF serves its own resources (the UDM: `subscribers`).

use librecn_scheduler::{Context, Message, MsgBody, Result};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::msg::{family, id};

/// Path and body bytes of a [`Manage`].
pub const MANAGE_TEXT: usize = 8 * 1024 - 8;
/// Text bytes of a [`Reply`].
pub const REPLY_TEXT: usize = 16 * 1024 - 8;

/// HTTP methods of [`Manage`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Method {
    Get = 1,
    Post = 2,
    Delete = 3,
}

impl Method {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "GET" => Some(Method::Get),
            "POST" => Some(Method::Post),
            "DELETE" => Some(Method::Delete),
            _ => None,
        }
    }
}

/// A management request: `method path`, with a JSON body. 8 KiB.
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Manage {
    method: u8,
    _pad: u8,
    path_len: u16,
    body_len: u32,
    text: [u8; MANAGE_TEXT],
}

impl MsgBody for Manage {
    const MSG_ID: u32 = id(family::OAM_CONTROL, 6);
}

impl Manage {
    /// Fills a zeroed body (from `Context::alloc`); false if it does not fit.
    pub fn fill(&mut self, method: Method, path: &str, body: &str) -> bool {
        let (p, b) = (path.len(), body.len());
        if p + b > MANAGE_TEXT || p > usize::from(u16::MAX) {
            return false;
        }
        self.text[..p].copy_from_slice(path.as_bytes());
        self.text[p..p + b].copy_from_slice(body.as_bytes());
        (self.method, self.path_len, self.body_len) = (method as u8, p as u16, b as u32);
        true
    }

    pub fn method(&self) -> Option<Method> {
        [Method::Get, Method::Post, Method::Delete].into_iter().find(|m| *m as u8 == self.method)
    }

    /// The path without its query, e.g. `subscribers/imsi-001010000000001`.
    pub fn path(&self) -> &str {
        let full = self.str(0, usize::from(self.path_len));
        full.split_once('?').map_or(full, |(path, _)| path)
    }

    /// The value of `key` in the query (`sessions?supi=imsi-1`), as sent.
    pub fn query(&self, key: &str) -> Option<&str> {
        let (_, query) = self.str(0, usize::from(self.path_len)).split_once('?')?;
        query.split('&').find_map(|pair| pair.split_once('=').filter(|(k, _)| *k == key).map(|(_, v)| v))
    }

    pub fn body(&self) -> &str {
        let p = usize::from(self.path_len);
        self.str(p, p + self.body_len as usize)
    }

    fn str(&self, from: usize, to: usize) -> &str {
        self.text.get(from..to).and_then(|b| std::str::from_utf8(b).ok()).unwrap_or_default()
    }
}

/// Answer to a request: an HTTP status and JSON text. 16 KiB.
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct Reply {
    status: u32,
    len: u32,
    text: [u8; REPLY_TEXT],
}

impl MsgBody for Reply {
    const MSG_ID: u32 = id(family::OAM_CONTROL, 4);
}

impl Reply {
    /// Fills a zeroed body; a text too long becomes a 500 error.
    pub fn fill(&mut self, status: u16, json: &str) {
        let (status, json) =
            if json.len() <= REPLY_TEXT { (status, json) } else { (500, r#"{"error":"reply too large"}"#) };
        self.text[..json.len()].copy_from_slice(json.as_bytes());
        (self.status, self.len) = (u32::from(status), json.len() as u32);
    }

    /// The HTTP status.
    pub fn status(&self) -> u16 {
        u16::try_from(self.status).unwrap_or(500)
    }

    /// Whether the request succeeded (2xx).
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// The text (empty if it was not valid UTF-8).
    pub fn text(&self) -> &str {
        std::str::from_utf8(&self.text[..(self.len as usize).min(REPLY_TEXT)]).unwrap_or_default()
    }
}

/// Answers `request` (same `corr_id`) with `status` and `json`. The
/// requester needs a buffer class of at least `size_of::<Reply>()` (16 KiB).
pub fn send_reply(ctx: &mut Context<'_>, request: &Message<'_>, status: u16, json: &str) -> Result<()> {
    let mut reply = ctx.alloc::<Reply>(request.from())?.with_corr_id(request.corr_id());
    reply.fill(status, json);
    ctx.send(reply)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zerocopy::FromZeros;

    #[test]
    fn requests_and_replies_carry_their_text() {
        let mut m = Manage::new_zeroed();
        assert!(m.fill(Method::Delete, "subscribers/imsi-1", "{}"));
        assert_eq!((m.method(), m.path(), m.body()), (Some(Method::Delete), "subscribers/imsi-1", "{}"));
        assert!(!m.fill(Method::Post, "x", &"y".repeat(MANAGE_TEXT)));
        assert_eq!(Method::parse("PUT"), None);
        assert!(m.fill(Method::Get, "sessions?dnn=ims&supi=imsi-1", ""));
        assert_eq!(
            (m.path(), m.query("supi"), m.query("dnn"), m.query("x")),
            ("sessions", Some("imsi-1"), Some("ims"), None)
        );

        let mut r = Reply::new_zeroed();
        r.fill(404, r#"{"error":"no such subscriber"}"#);
        assert_eq!((r.status(), r.ok(), r.text()), (404, false, r#"{"error":"no such subscriber"}"#));
        r.fill(201, &"x".repeat(REPLY_TEXT + 1));
        assert_eq!((r.status(), r.text()), (500, r#"{"error":"reply too large"}"#));
    }
}
