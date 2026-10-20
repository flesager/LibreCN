//! Datagrams on N3 in batches: up to [`BATCH`] received with one
//! `recvmmsg`, up to [`BATCH`] sent with one `sendmmsg`, in buffers
//! allocated once. One system call for many packets instead of one each.

use std::io;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::os::fd::AsRawFd;

/// Datagrams per system call.
pub const BATCH: usize = 32;
/// Room per datagram: a jumbo frame (a TUN device's MTU up to 9200 with
/// its GTP-U header).
pub const SLOT: usize = 9216;

/// `BATCH` datagram slots and their system-call headers; `M`: what the
/// caller keeps with each datagram to send.
pub struct Batch<M> {
    bufs: Box<[u8]>,
    /// Built once: each points into `bufs` and its own `names` entry, both
    /// on the heap, so they stay valid when the batch moves.
    headers: Box<Headers>,
    meta: [M; BATCH],
    count: usize,
}

struct Headers {
    iovs: [libc::iovec; BATCH],
    names: [libc::sockaddr_in; BATCH],
    msgs: [libc::mmsghdr; BATCH],
}

// SAFETY: the raw pointers point into the batch's own allocations only.
unsafe impl<M: Send> Send for Batch<M> {}

const NAME_LEN: libc::socklen_t = size_of::<libc::sockaddr_in>() as _;

fn to_sockaddr(a: SocketAddrV4) -> libc::sockaddr_in {
    libc::sockaddr_in {
        sin_family: libc::AF_INET as libc::sa_family_t,
        sin_port: a.port().to_be(),
        sin_addr: libc::in_addr { s_addr: u32::from(*a.ip()).to_be() },
        sin_zero: [0; 8],
    }
}

fn from_sockaddr(s: &libc::sockaddr_in) -> SocketAddrV4 {
    SocketAddrV4::new(Ipv4Addr::from(u32::from_be(s.sin_addr.s_addr)), u16::from_be(s.sin_port))
}

impl<M: Copy + Default> Batch<M> {
    pub fn new() -> Self {
        let mut bufs = vec![0; BATCH * SLOT].into_boxed_slice();
        // SAFETY: all-zero is a valid iovec, sockaddr_in and mmsghdr.
        let mut headers: Box<Headers> = Box::new(unsafe { std::mem::zeroed() });
        let h = &mut *headers;
        for i in 0..BATCH {
            h.iovs[i].iov_base = bufs[i * SLOT..].as_mut_ptr().cast();
            h.msgs[i].msg_hdr.msg_iov = &mut h.iovs[i];
            h.msgs[i].msg_hdr.msg_iovlen = 1;
            h.msgs[i].msg_hdr.msg_name = (&mut h.names[i] as *mut libc::sockaddr_in).cast();
        }
        Batch { bufs, headers, meta: [M::default(); BATCH], count: 0 }
    }

    /// Receives up to [`BATCH`] datagrams; how many (0: none waiting). A
    /// datagram larger than [`SLOT`] comes back empty.
    pub fn recv(&mut self, socket: &UdpSocket) -> io::Result<usize> {
        let h = &mut *self.headers;
        for (iov, msg) in h.iovs.iter_mut().zip(&mut h.msgs) {
            iov.iov_len = SLOT;
            msg.msg_hdr.msg_namelen = NAME_LEN;
        }
        // SAFETY: every header points to a live slot of SLOT bytes and its address.
        let n = unsafe {
            libc::recvmmsg(
                socket.as_raw_fd(),
                h.msgs.as_mut_ptr(),
                BATCH as _,
                libc::MSG_DONTWAIT as _,
                std::ptr::null_mut(),
            )
        };
        if n < 0 {
            let e = io::Error::last_os_error();
            return if e.kind() == io::ErrorKind::WouldBlock { Ok(0) } else { Err(e) };
        }
        self.count = n as usize;
        Ok(self.count)
    }

    /// Received datagram `i`, and where it came from.
    pub fn datagram(&self, i: usize) -> (&[u8], SocketAddrV4) {
        let msg = &self.headers.msgs[i];
        let len = if msg.msg_hdr.msg_flags & libc::MSG_TRUNC != 0 { 0 } else { msg.msg_len as usize };
        (&self.bufs[i * SLOT..i * SLOT + len], from_sockaddr(&self.headers.names[i]))
    }

    /// The next slot to fill before [`push`](Self::push).
    pub fn slot(&mut self) -> &mut [u8] {
        let i = self.count;
        &mut self.bufs[i * SLOT..(i + 1) * SLOT]
    }

    /// The slot filled: its first `len` bytes go to `to`.
    pub fn push(&mut self, len: usize, to: SocketAddrV4, meta: M) {
        let (i, h) = (self.count, &mut *self.headers);
        h.iovs[i].iov_len = len;
        h.names[i] = to_sockaddr(to);
        h.msgs[i].msg_hdr.msg_namelen = NAME_LEN;
        self.meta[i] = meta;
        self.count += 1;
    }

    pub fn is_full(&self) -> bool {
        self.count == BATCH
    }

    /// Sends what was pushed; `done` learns each datagram's fate (with its
    /// length). One system call, more if the kernel takes part of the batch
    /// or refuses one datagram (that one dropped).
    pub fn send(&mut self, socket: &UdpSocket, mut done: impl FnMut(M, usize, bool)) {
        let count = std::mem::take(&mut self.count);
        let h = &mut *self.headers;
        let mut start = 0;
        while start < count {
            // SAFETY: headers start..count point to live slots and addresses.
            let sent =
                unsafe { libc::sendmmsg(socket.as_raw_fd(), h.msgs[start..].as_mut_ptr(), (count - start) as _, 0) };
            let (ok, end) = if sent <= 0 { (false, start + 1) } else { (true, start + sent as usize) };
            for i in start..end {
                done(self.meta[i], h.iovs[i].iov_len, ok);
            }
            start = end;
        }
    }
}

impl<M: Copy + Default> Default for Batch<M> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datagrams_go_and_come_in_batches() {
        let a = UdpSocket::bind("127.0.0.1:0").unwrap();
        let b = UdpSocket::bind("127.0.0.1:0").unwrap();
        b.set_nonblocking(true).unwrap();
        let std::net::SocketAddr::V4(to) = b.local_addr().unwrap() else { panic!() };
        let mut tx = Batch::<u8>::new();
        for i in 0..3u8 {
            let slot = tx.slot();
            slot[..2].copy_from_slice(&[i, i]);
            tx.push(2 + usize::from(i), to, i);
        }
        let mut fates = vec![];
        tx.send(&a, |meta, len, ok| fates.push((meta, len, ok)));
        assert_eq!(fates, [(0, 2, true), (1, 3, true), (2, 4, true)]);

        let mut rx = Batch::<()>::new();
        std::thread::sleep(std::time::Duration::from_millis(20));
        assert_eq!(rx.recv(&b).unwrap(), 3, "one call");
        let std::net::SocketAddr::V4(from) = a.local_addr().unwrap() else { panic!() };
        assert_eq!(rx.datagram(1), (&[1, 1, 0][..], from));
        assert_eq!(rx.recv(&b).unwrap(), 0, "nothing more");

        // Larger than a slot: empty.
        a.send_to(&vec![7u8; SLOT + 10], b.local_addr().unwrap()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        assert_eq!(rx.recv(&b).unwrap(), 1);
        assert!(rx.datagram(0).0.is_empty());
    }
}
