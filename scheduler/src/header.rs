//! Fixed-layout message header and system message ids.

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

use crate::EoId;

/// Header carried by every message. Fixed `repr(C)` layout, 40 bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(C)]
pub struct MsgHeader {
    /// Message type. `0..=SYSTEM_MAX` is reserved for the scheduler.
    pub msg_id: u32,
    /// Body length in bytes; 0 for header-only messages.
    pub body_len: u32,
    /// Sender, [`EoId::SYSTEM`] for timers, I/O and lifecycle.
    pub from: EoId,
    /// Receiver.
    pub to: EoId,
    /// Request/response correlation; for `TIMER_EXPIRED`, the tag given when arming.
    pub corr_id: u32,
    /// Free for the application (e.g. request / response role).
    pub flags: u32,
    /// System messages: timer id, I/O token, ... Free for user messages.
    pub reserved: u64,
}

const _: () = assert!(std::mem::size_of::<MsgHeader>() == 40);

impl MsgHeader {
    /// Header with every optional field set to zero.
    pub const fn new(msg_id: u32, from: EoId, to: EoId) -> Self {
        MsgHeader { msg_id, body_len: 0, from, to, corr_id: 0, flags: 0, reserved: 0 }
    }

    pub(crate) const fn system(msg_id: u32, to: EoId, reserved: u64) -> Self {
        let mut hdr = MsgHeader::new(msg_id, EoId::SYSTEM, to);
        hdr.reserved = reserved;
        hdr
    }
}

/// Message ids reserved for the scheduler.
pub mod system {
    /// Highest reserved id.
    pub const SYSTEM_MAX: u32 = 0xFFFF;
    /// First message of every EO, dispatched to `on_start`.
    pub const EO_START: u32 = 0x0001;
    /// Stop request, dispatched to `on_stop`; the EO is removed afterwards.
    pub const EO_STOP: u32 = 0x0002;
    /// A timer expired: `reserved` = timer id, `corr_id` = tag.
    pub const TIMER_EXPIRED: u32 = 0x0003;
    /// A registered file descriptor is ready: `reserved` = I/O token.
    pub const IO_READY: u32 = 0x0004;
    /// Once per turn to an EO that polls (`Context::set_polling`). Not kept
    /// in the flight recorder.
    pub const EO_POLL: u32 = 0x0005;

    /// Whether `msg_id` is in the reserved range.
    pub const fn is_system(msg_id: u32) -> bool {
        msg_id <= SYSTEM_MAX
    }
}

impl std::fmt::Display for MsgHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "id={:#06x} from={} to={} corr_id={} flags={:#x} body_len={} reserved={:#x}",
            self.msg_id, self.from, self.to, self.corr_id, self.flags, self.body_len, self.reserved
        )
    }
}
