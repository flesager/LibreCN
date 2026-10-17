//! N6: the kernel TUN device the UPF reads and writes IP packets on.

use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;

/// Attaches to TUN device `name` (IP packets, no packet information),
/// non-blocking. Creating it needs `CAP_NET_ADMIN`; attaching to a
/// persistent device owned by the user does not (`ip tuntap add dev NAME
/// mode tun user USER`). Its address and link state are set outside.
pub fn open(name: &str) -> io::Result<File> {
    if name.is_empty() || name.len() >= libc::IFNAMSIZ {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "interface name of 1 to 15 bytes"));
    }
    let file = OpenOptions::new().read(true).write(true).custom_flags(libc::O_NONBLOCK).open("/dev/net/tun")?;
    // SAFETY: an all-zero ifreq is valid.
    let mut ifr: libc::ifreq = unsafe { std::mem::zeroed() };
    for (dst, src) in ifr.ifr_name.iter_mut().zip(name.bytes()) {
        *dst = src as libc::c_char;
    }
    ifr.ifr_ifru.ifru_flags = (libc::IFF_TUN | libc::IFF_NO_PI) as libc::c_short;
    // SAFETY: TUNSETIFF reads and writes one ifreq, which outlives the call.
    if unsafe { libc::ioctl(file.as_raw_fd(), libc::TUNSETIFF, &mut ifr) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(file)
}
