//! Kernel SCTP for N2.
//!
//! The AMF uses the one-to-many style (RFC 6458 §3): one socket for every gNB,
//! each message tagged with its association. Data and association events come
//! from the same socket; the stream and payload protocol of each message come
//! as ancillary data. Constants and layouts are from `<linux/sctp.h>`.

use std::io;
use std::mem::{MaybeUninit, size_of, zeroed};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd};

use libc::{c_int, c_void, socklen_t};

/// SCTP payload protocol identifier of NGAP (TS 38.412 §7).
pub const NGAP_PPID: u32 = 60;

/// Association id: one per gNB on a one-to-many socket.
pub type AssocId = i32;

const SOL_SCTP: c_int = 132;
const SCTP_INITMSG: c_int = 2;
const SCTP_NODELAY: c_int = 3;
const SCTP_RECVRCVINFO: c_int = 32;
const SCTP_EVENT: c_int = 127;
/// cmsg types.
const SCTP_SNDINFO: c_int = 2;
const SCTP_RCVINFO: c_int = 3;
const MSG_NOTIFICATION: c_int = 0x8000;
/// Notification types.
const SCTP_ASSOC_CHANGE: u16 = 0x8001;
const SCTP_SHUTDOWN_EVENT: u16 = 0x8005;
/// `sac_state` values.
const SCTP_COMM_UP: u16 = 0;
const SCTP_RESTART: u16 = 2;

#[repr(C)]
struct SndInfo {
    sid: u16,
    flags: u16,
    ppid: u32,
    context: u32,
    assoc_id: AssocId,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RcvInfo {
    sid: u16,
    ssn: u16,
    flags: u16,
    ppid: u32,
    tsn: u32,
    cumtsn: u32,
    context: u32,
    assoc_id: AssocId,
}

#[repr(C)]
struct InitMsg {
    num_ostreams: u16,
    max_instreams: u16,
    max_attempts: u16,
    max_init_timeo: u16,
}

#[repr(C)]
struct Event {
    assoc_id: AssocId,
    kind: u16,
    on: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct AssocChange {
    kind: u16,
    flags: u16,
    length: u32,
    state: u16,
    error: u16,
    outbound_streams: u16,
    inbound_streams: u16,
    assoc_id: AssocId,
}

/// What one receive produced.
#[derive(Debug, PartialEq, Eq)]
pub enum Received<'a> {
    /// A complete message.
    Data { assoc: AssocId, stream: u16, ppid: u32, data: &'a [u8] },
    /// A message larger than the buffer; dropped.
    Truncated { assoc: AssocId },
    /// An association is up (new or restarted peer).
    Up { assoc: AssocId, outbound_streams: u16 },
    /// An association is gone (lost, shut down, or never established).
    Down { assoc: AssocId },
    /// Another notification, ignored.
    Ignored,
}

/// An SCTP socket, non-blocking.
#[derive(Debug)]
pub struct SctpSocket {
    fd: OwnedFd,
}

fn cvt(ret: c_int) -> io::Result<c_int> {
    if ret < 0 { Err(io::Error::last_os_error()) } else { Ok(ret) }
}

fn sockaddr(addr: &SocketAddr) -> (libc::sockaddr_storage, socklen_t) {
    // SAFETY: all-zero is a valid sockaddr_storage.
    let mut storage: libc::sockaddr_storage = unsafe { zeroed() };
    let len = match addr {
        SocketAddr::V4(a) => {
            let sin = libc::sockaddr_in {
                sin_family: libc::AF_INET as libc::sa_family_t,
                sin_port: a.port().to_be(),
                sin_addr: libc::in_addr { s_addr: u32::from_ne_bytes(a.ip().octets()) },
                sin_zero: [0; 8],
            };
            // SAFETY: sockaddr_storage is large enough and suitably aligned for any sockaddr.
            unsafe { std::ptr::write(std::ptr::from_mut(&mut storage).cast(), sin) };
            size_of::<libc::sockaddr_in>()
        }
        SocketAddr::V6(a) => {
            let sin6 = libc::sockaddr_in6 {
                sin6_family: libc::AF_INET6 as libc::sa_family_t,
                sin6_port: a.port().to_be(),
                sin6_flowinfo: a.flowinfo(),
                sin6_addr: libc::in6_addr { s6_addr: a.ip().octets() },
                sin6_scope_id: a.scope_id(),
            };
            // SAFETY: as above.
            unsafe { std::ptr::write(std::ptr::from_mut(&mut storage).cast(), sin6) };
            size_of::<libc::sockaddr_in6>()
        }
    };
    (storage, len as socklen_t)
}

impl SctpSocket {
    fn new(addr: &SocketAddr, kind: c_int) -> io::Result<Self> {
        let family = if addr.is_ipv4() { libc::AF_INET } else { libc::AF_INET6 };
        // SAFETY: plain socket(2) call; the result is checked before use.
        let fd = cvt(unsafe { libc::socket(family, kind | libc::SOCK_CLOEXEC, libc::IPPROTO_SCTP) })?;
        // SAFETY: `fd` was just created and is owned by nobody else.
        let socket = SctpSocket { fd: unsafe { OwnedFd::from_raw_fd(fd) } };
        socket.set(SCTP_NODELAY, &1 as &c_int)?;
        socket.set(SCTP_RECVRCVINFO, &1 as &c_int)?;
        Ok(socket)
    }

    fn set<T>(&self, option: c_int, value: &T) -> io::Result<()> {
        // SAFETY: `value` points to a live `T` of the given size.
        cvt(unsafe {
            libc::setsockopt(
                self.fd.as_raw_fd(),
                SOL_SCTP,
                option,
                std::ptr::from_ref(value).cast(),
                size_of::<T>() as socklen_t,
            )
        })
        .map(drop)
    }

    /// One-to-many server socket bound to `addr`, offering `streams` streams
    /// each way, reporting association events.
    pub fn listen(addr: SocketAddr, streams: u16) -> io::Result<Self> {
        let socket = Self::new(&addr, libc::SOCK_SEQPACKET | libc::SOCK_NONBLOCK)?;
        let init = InitMsg { num_ostreams: streams, max_instreams: streams, max_attempts: 0, max_init_timeo: 0 };
        socket.set(SCTP_INITMSG, &init)?;
        for kind in [SCTP_ASSOC_CHANGE, SCTP_SHUTDOWN_EVENT] {
            socket.set(SCTP_EVENT, &Event { assoc_id: 0, kind, on: 1 })?;
        }
        let (storage, len) = sockaddr(&addr);
        // SAFETY: `storage` holds a valid address of `len` bytes.
        cvt(unsafe { libc::bind(socket.fd.as_raw_fd(), std::ptr::from_ref(&storage).cast(), len) })?;
        // SAFETY: plain listen(2) on our socket.
        cvt(unsafe { libc::listen(socket.fd.as_raw_fd(), 64) })?;
        Ok(socket)
    }

    /// One-to-one client socket connected to `addr`, blocking: for tests and
    /// tools playing a gNB.
    pub fn connect(addr: SocketAddr) -> io::Result<Self> {
        let socket = Self::new(&addr, libc::SOCK_STREAM)?;
        let (storage, len) = sockaddr(&addr);
        // SAFETY: `storage` holds a valid address of `len` bytes.
        cvt(unsafe { libc::connect(socket.fd.as_raw_fd(), std::ptr::from_ref(&storage).cast(), len) })?;
        Ok(socket)
    }

    /// The bound address (the port actually chosen for port 0).
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        // SAFETY: all-zero is a valid sockaddr_storage.
        let mut storage: libc::sockaddr_storage = unsafe { zeroed() };
        let mut len = size_of::<libc::sockaddr_storage>() as socklen_t;
        // SAFETY: `storage` and `len` describe a writable buffer.
        cvt(unsafe { libc::getsockname(self.fd.as_raw_fd(), std::ptr::from_mut(&mut storage).cast(), &mut len) })?;
        // SAFETY: the kernel wrote an address of the family it reports.
        Ok(unsafe {
            match c_int::from(storage.ss_family) {
                libc::AF_INET => {
                    let sin: libc::sockaddr_in = std::ptr::read(std::ptr::from_ref(&storage).cast());
                    SocketAddr::V4(SocketAddrV4::new(
                        sin.sin_addr.s_addr.to_ne_bytes().into(),
                        u16::from_be(sin.sin_port),
                    ))
                }
                _ => {
                    let sin6: libc::sockaddr_in6 = std::ptr::read(std::ptr::from_ref(&storage).cast());
                    SocketAddr::V6(SocketAddrV6::new(
                        sin6.sin6_addr.s6_addr.into(),
                        u16::from_be(sin6.sin6_port),
                        sin6.sin6_flowinfo,
                        sin6.sin6_scope_id,
                    ))
                }
            }
        })
    }

    /// Receives one message or event into `buf`; `Ok(None)` if nothing is
    /// waiting (non-blocking socket).
    pub fn recv<'b>(&self, buf: &'b mut [u8]) -> io::Result<Option<Received<'b>>> {
        // Control buffer, aligned for cmsghdr.
        let mut control = [MaybeUninit::<u64>::uninit(); 16];
        let mut iov = libc::iovec { iov_base: buf.as_mut_ptr().cast::<c_void>(), iov_len: buf.len() };
        // SAFETY: all-zero is a valid msghdr.
        let mut msg: libc::msghdr = unsafe { zeroed() };
        msg.msg_iov = &mut iov;
        msg.msg_iovlen = 1;
        msg.msg_control = control.as_mut_ptr().cast();
        msg.msg_controllen = size_of_val(&control);
        // SAFETY: `msg` describes live buffers of the given sizes.
        let n = match cvt(unsafe { libc::recvmsg(self.fd.as_raw_fd(), &mut msg, 0) as c_int }) {
            Ok(n) => n as usize,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(None),
            Err(e) => return Err(e),
        };
        if msg.msg_flags & MSG_NOTIFICATION != 0 {
            return Ok(Some(notification(&buf[..n])));
        }
        let info = rcvinfo(&msg);
        let assoc = info.map_or(0, |i| i.assoc_id);
        if msg.msg_flags & libc::MSG_EOR == 0 {
            return Ok(Some(Received::Truncated { assoc }));
        }
        Ok(Some(Received::Data {
            assoc,
            stream: info.map_or(0, |i| i.sid),
            ppid: info.map_or(0, |i| u32::from_be(i.ppid)),
            data: &buf[..n],
        }))
    }

    /// Sends `data` on `stream` of association `assoc` (ignored on a
    /// one-to-one socket).
    pub fn send(&self, assoc: AssocId, stream: u16, ppid: u32, data: &[u8]) -> io::Result<()> {
        let mut control = [MaybeUninit::<u64>::uninit(); 8];
        let mut iov = libc::iovec { iov_base: data.as_ptr().cast_mut().cast::<c_void>(), iov_len: data.len() };
        // SAFETY: all-zero is a valid msghdr.
        let mut msg: libc::msghdr = unsafe { zeroed() };
        msg.msg_iov = &mut iov;
        msg.msg_iovlen = 1;
        msg.msg_control = control.as_mut_ptr().cast();
        // SAFETY: CMSG_SPACE is a pure size computation.
        msg.msg_controllen = unsafe { libc::CMSG_SPACE(size_of::<SndInfo>() as u32) } as usize;
        // SAFETY: `msg_control` points to a buffer of at least `msg_controllen`
        // bytes, so the first header and its data fit.
        unsafe {
            let cmsg = libc::CMSG_FIRSTHDR(&msg);
            (*cmsg).cmsg_level = libc::IPPROTO_SCTP;
            (*cmsg).cmsg_type = SCTP_SNDINFO;
            (*cmsg).cmsg_len = libc::CMSG_LEN(size_of::<SndInfo>() as u32) as usize;
            let info = SndInfo { sid: stream, flags: 0, ppid: ppid.to_be(), context: 0, assoc_id: assoc };
            std::ptr::write_unaligned(libc::CMSG_DATA(cmsg).cast::<SndInfo>(), info);
        }
        // SAFETY: `msg` describes live buffers.
        let sent = cvt(unsafe { libc::sendmsg(self.fd.as_raw_fd(), &msg, libc::MSG_NOSIGNAL) as c_int })?;
        if sent as usize != data.len() {
            return Err(io::Error::new(io::ErrorKind::WriteZero, "partial SCTP send"));
        }
        Ok(())
    }
}

/// The `SCTP_RCVINFO` ancillary data of a received message.
fn rcvinfo(msg: &libc::msghdr) -> Option<RcvInfo> {
    // SAFETY: the kernel filled `msg_control` with well-formed cmsg headers;
    // CMSG_FIRSTHDR/NXTHDR stay within `msg_controllen`.
    unsafe {
        let mut cmsg = libc::CMSG_FIRSTHDR(msg);
        while !cmsg.is_null() {
            if (*cmsg).cmsg_level == libc::IPPROTO_SCTP && (*cmsg).cmsg_type == SCTP_RCVINFO {
                return Some(std::ptr::read_unaligned(libc::CMSG_DATA(cmsg).cast::<RcvInfo>()));
            }
            cmsg = libc::CMSG_NXTHDR(msg, cmsg);
        }
    }
    None
}

fn notification(bytes: &[u8]) -> Received<'_> {
    let kind = bytes.get(..2).map(|b| u16::from_ne_bytes([b[0], b[1]]));
    match kind {
        Some(SCTP_ASSOC_CHANGE) if bytes.len() >= size_of::<AssocChange>() => {
            // SAFETY: the kernel wrote a complete sctp_assoc_change.
            let change: AssocChange = unsafe { std::ptr::read_unaligned(bytes.as_ptr().cast()) };
            match change.state {
                SCTP_COMM_UP | SCTP_RESTART => {
                    Received::Up { assoc: change.assoc_id, outbound_streams: change.outbound_streams }
                }
                _ => Received::Down { assoc: change.assoc_id },
            }
        }
        // sctp_shutdown_event: type, flags, length (u32), assoc_id.
        Some(SCTP_SHUTDOWN_EVENT) if bytes.len() >= 12 => {
            Received::Down { assoc: AssocId::from_ne_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) }
        }
        _ => Received::Ignored,
    }
}

impl AsFd for SctpSocket {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Waits up to a second for the non-blocking server socket, then receives.
    fn recv<'b>(server: &SctpSocket, buf: &'b mut [u8]) -> Received<'b> {
        let mut pollfd = libc::pollfd { fd: server.fd.as_raw_fd(), events: libc::POLLIN, revents: 0 };
        // SAFETY: one valid pollfd.
        assert_eq!(unsafe { libc::poll(&mut pollfd, 1, 1000) }, 1, "nothing received");
        server.recv(buf).unwrap().expect("readable")
    }

    #[test]
    fn one_to_many_server_sees_associations_streams_and_ppid() {
        let server = SctpSocket::listen("127.0.0.1:0".parse().unwrap(), 4).unwrap();
        let addr = server.local_addr().unwrap();
        let client = SctpSocket::connect(addr).unwrap();
        let mut buf = [0u8; 1024];

        let Received::Up { assoc, outbound_streams } = recv(&server, &mut buf) else { panic!("no association") };
        assert!(outbound_streams >= 1);
        client.send(0, 1, NGAP_PPID, b"hello").unwrap();
        assert_eq!(recv(&server, &mut buf), Received::Data { assoc, stream: 1, ppid: NGAP_PPID, data: b"hello" });

        server.send(assoc, 0, NGAP_PPID, b"back").unwrap();
        let mut reply = [0u8; 64];
        // The client socket is blocking.
        let Some(Received::Data { data, ppid, .. }) = client.recv(&mut reply).unwrap() else { panic!() };
        assert_eq!((data, ppid), (&b"back"[..], NGAP_PPID));

        drop(client);
        assert_eq!(recv(&server, &mut buf), Received::Down { assoc });
    }
}
