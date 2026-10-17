//! Conventions shared by the LibreCN network functions, on top of the
//! scheduler's opaque message header.

/// Values for `MsgHeader::flags`: the role of a message in a procedure.
/// With `corr_id`, enough to pair requests and responses.
pub mod flags {
    /// Expects a response carrying the same `corr_id`.
    pub const REQUEST: u32 = 1 << 0;
    /// Response to the request with the same `corr_id`.
    pub const RESPONSE: u32 = 1 << 1;
    /// One-way notification.
    pub const NOTIFICATION: u32 = 1 << 2;
}
