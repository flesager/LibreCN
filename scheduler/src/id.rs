//! Execution object identifiers.

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// Identifies an execution object: registry index in the low 32 bits,
/// generation in the high 32 bits.
///
/// The generation changes every time a registry slot is reused, so an id kept
/// after its EO stopped can never reach the EO that took the slot next.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, FromBytes, IntoBytes, KnownLayout, Immutable)]
#[repr(transparent)]
pub struct EoId(u64);

impl EoId {
    /// Sender of timer, I/O and lifecycle messages, and of messages injected from outside.
    pub const SYSTEM: EoId = EoId(u32::MAX as u64);

    pub(crate) const fn new(index: u32, generation: u32) -> Self {
        EoId(((generation as u64) << 32) | index as u64)
    }

    /// Registry index.
    pub const fn index(self) -> u32 {
        self.0 as u32
    }

    /// Generation of the registry slot.
    pub const fn generation(self) -> u32 {
        (self.0 >> 32) as u32
    }

    /// Raw 64-bit value, as carried in message headers.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl std::fmt::Debug for EoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == EoId::SYSTEM {
            f.write_str("EoId(SYSTEM)")
        } else {
            write!(f, "EoId({}#{})", self.index(), self.generation())
        }
    }
}

impl std::fmt::Display for EoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_index_and_generation() {
        let id = EoId::new(7, 3);
        assert_eq!(id.index(), 7);
        assert_eq!(id.generation(), 3);
        assert_ne!(id, EoId::new(7, 4));
    }
}
