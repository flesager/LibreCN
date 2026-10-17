//! Error type shared by the whole scheduler.

/// Everything that can go wrong when talking to the scheduler.
///
/// Every variant is cheap to build and copy: errors are returned on the hot
/// path (overload) and must not allocate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The destination lane is full.
    #[error("destination mailbox is full")]
    QueueFull,
    /// The destination has no free buffer in the size class the body needs.
    #[error("destination buffer pool is exhausted")]
    PoolExhausted,
    /// The body is larger than the destination's largest buffer class.
    #[error("message body of {0} bytes exceeds the destination's largest buffer class")]
    BodyTooLarge(usize),
    /// The id does not name a live execution object (never existed, stopped, or stale generation).
    #[error("unknown or stopped execution object")]
    UnknownEo,
    /// The execution object panicked and was isolated.
    #[error("execution object is faulted")]
    EoFaulted,
    /// No free slot left in the registry.
    #[error("execution object registry is full")]
    RegistryFull,
    /// No free timer left (per-EO table or platform wheel).
    #[error("no free timer")]
    TimersExhausted,
    /// User messages cannot use the system range.
    #[error("message id {0:#x} is in the reserved system range")]
    ReservedMsgId(u32),
    /// No free I/O registration left in the EO's table.
    #[error("no free I/O registration")]
    IoExhausted,
    /// A system call failed, with its errno.
    #[error("system call failed: errno {0}")]
    Os(i32),
    /// A configuration value is out of range.
    #[error("invalid configuration: {0}")]
    InvalidConfig(&'static str),
}

/// Scheduler result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl From<rustix::io::Errno> for Error {
    fn from(e: rustix::io::Errno) -> Self {
        Error::Os(e.raw_os_error())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Os(e.raw_os_error().unwrap_or(0))
    }
}
