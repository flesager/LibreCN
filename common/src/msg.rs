//! Message ids and header flags.
//!
//! `msg_id = family << 16 | message`. Families are procedures (UE
//! registration, PDU session, ...), not NFs: a message keeps one id whoever
//! sends it. Every family is declared here, once; the messages of a family
//! are declared next to their bodies.

/// Message-id families. `0x0000` is the scheduler's.
pub mod family {
    /// 5G AKA: AMF, AUSF and UDM (`librecn-sbi`).
    pub const UE_AUTHENTICATION: u16 = 0x0001;

    /// Log records.
    pub const OAM_LOG: u16 = 0xFF01;
    /// Alarm raise and clear.
    pub const OAM_ALARM: u16 = 0xFF02;
    /// Control requests between OAM EOs.
    pub const OAM_CONTROL: u16 = 0xFF03;

    /// Every family, for the uniqueness test.
    pub const ALL: [u16; 4] = [UE_AUTHENTICATION, OAM_LOG, OAM_ALARM, OAM_CONTROL];
}

/// Message id of `message` in `family`.
pub const fn id(family: u16, message: u16) -> u32 {
    ((family as u32) << 16) | message as u32
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn families_are_unique_and_outside_the_system_range() {
        let mut all = family::ALL.to_vec();
        all.sort();
        all.dedup();
        assert_eq!(all.len(), family::ALL.len());
        assert!(all.iter().all(|&f| !librecn_scheduler::system::is_system(id(f, 0))));
    }
}
