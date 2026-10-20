//! The fast path (`fast_path`, on by default): the kernel carries the
//! uplink of the sessions it can, before the UPF sees it (`bpf/upf.bpf.c`,
//! XDP on the N3 interface): decapsulated there, the UE's packet goes on
//! into the kernel's stack. What it cannot carry reaches the UPF's socket,
//! as without it (the fallback). It needs `CAP_BPF`, `CAP_PERFMON` and
//! `CAP_NET_ADMIN`, and a loose reverse path filter on the N3 interface
//! (the UE's packets come in there, while their addresses are routed to
//! N6); without them, or on a kernel without XDP, the UPF says why and runs
//! without it.
//!
//! The UPF copies each session without extra QoS flows into the kernel's
//! `uplink` map (its TEID, UE address, cell slot, uplink AMBR), removes it
//! with the session, and folds the kernel's counts into its own.

use std::net::Ipv4Addr;

use aya::maps::{HashMap, MapData, PerCpuArray};
use aya::programs::{Xdp, XdpMode};
use aya::{EbpfLoader, Pod};

use crate::MAX_CELLS;
use crate::sessions::Session;

/// The program, built by `bpf/build.sh`.
static OBJECT: &[u8] = aya::include_bytes_aligned!("../bpf/upf.bpf.o");

/// `struct uplink`: a session's uplink, by its TEID.
#[repr(C)]
#[derive(Clone, Copy)]
struct Uplink {
    /// Network order.
    ue_ip: u32,
    /// Its slot in `counts`; [`MAX_CELLS`]: none.
    cell: u32,
    /// Uplink AMBR, bit/s; 0: no limit.
    rate: u64,
    /// The kernel's GCRA state (ns).
    done: u64,
}

// SAFETY: integers only, without padding (4 + 4 + 8 + 8 bytes).
unsafe impl Pod for Uplink {}

/// `struct counts`: what the kernel carried and dropped, for one cell slot.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub packets: u64,
    pub bytes: u64,
    pub rate_limited: u64,
}

// SAFETY: integers only, without padding.
unsafe impl Pod for Counts {}

impl Counts {
    fn minus(self, before: Counts) -> Counts {
        Counts {
            packets: self.packets.wrapping_sub(before.packets),
            bytes: self.bytes.wrapping_sub(before.bytes),
            rate_limited: self.rate_limited.wrapping_sub(before.rate_limited),
        }
    }
}

/// The program, attached while this lives (its link goes with the
/// process), and its maps.
pub struct FastPath {
    _ebpf: aya::Ebpf,
    interface: String,
    uplink: HashMap<MapData, u32, Uplink>,
    counts: PerCpuArray<MapData, Counts>,
    /// What [`take`](Self::take) has given so far, per slot.
    given: Vec<Counts>,
}

impl FastPath {
    /// Loads the program and attaches it to the interface with address
    /// `n3`, room for `max_sessions`.
    pub fn start(n3: Ipv4Addr, max_sessions: usize) -> Result<Self, String> {
        let interface = interface_of(n3).ok_or_else(|| format!("no interface has {n3}"))?;
        if strict_rp_filter(&interface) {
            return Err(format!(
                "strict reverse path filter on {interface} would drop the UEs' packets: \
                 net.ipv4.conf.all.rp_filter and net.ipv4.conf.{interface}.rp_filter must be 0 or 2"
            ));
        }
        let addr = u32::from(n3).to_be();
        let max = u32::try_from(max_sessions).map_err(|_| "too many sessions".to_string())?;
        let mut ebpf = EbpfLoader::new()
            .override_global("n3_addr", &addr, true)
            .map_max_entries("uplink", max.max(1))
            .load(OBJECT)
            .map_err(|e| format!("loading ({RIGHTS}): {e}"))?;
        let program: &mut Xdp =
            ebpf.program_mut("upf_uplink").ok_or("no program upf_uplink")?.try_into().map_err(|e| format!("{e}"))?;
        program.load().map_err(|e| format!("verifier ({RIGHTS}): {}", first_sentence(&e.to_string())))?;
        program.attach(&interface, XdpMode::Default).map_err(|e| format!("attaching to {interface}: {e}"))?;
        let mut map = |name: &str| ebpf.take_map(name).ok_or(format!("no map {name}"));
        let uplink = HashMap::try_from(map("uplink")?).map_err(|e| e.to_string())?;
        let counts = PerCpuArray::try_from(map("counts")?).map_err(|e| e.to_string())?;
        Ok(FastPath { _ebpf: ebpf, interface, uplink, counts, given: vec![Counts::default(); MAX_CELLS + 1] })
    }

    /// The interface it is attached to.
    pub fn interface(&self) -> &str {
        &self.interface
    }

    /// Session `s` carried by the kernel, if it can be (no extra QoS
    /// flows) and there is room; again after a change (its cell).
    pub fn add(&mut self, s: &Session) -> bool {
        if !s.qos.rules().is_empty() {
            return false;
        }
        let cell = s.cell.map_or(MAX_CELLS as u32, u32::from);
        let value = Uplink { ue_ip: u32::from(s.ue_ip).to_be(), cell, rate: s.qos.uplink_ambr(), done: 0 };
        self.uplink.insert(s.ul_teid.to_be(), value, 0).is_ok()
    }

    /// The session with uplink TEID `teid`, now gone.
    pub fn remove(&mut self, teid: u32) {
        let _ = self.uplink.remove(&teid.to_be());
    }

    /// What the kernel carried since the last call, for the cell slots
    /// below `cells` (`Some(slot)`) and without a cell (`None`).
    pub fn take(&mut self, cells: usize, mut f: impl FnMut(Option<usize>, Counts)) {
        for slot in (0..cells.min(MAX_CELLS)).chain([MAX_CELLS]) {
            let Ok(per_cpu) = self.counts.get(&(slot as u32), 0) else { continue };
            let total = per_cpu.iter().fold(Counts::default(), |a, c| Counts {
                packets: a.packets + c.packets,
                bytes: a.bytes + c.bytes,
                rate_limited: a.rate_limited + c.rate_limited,
            });
            let new = total.minus(self.given[slot]);
            self.given[slot] = total;
            if new != Counts::default() {
                f((slot < MAX_CELLS).then_some(slot), new);
            }
        }
    }
}

/// What loading needs, said when it fails.
const RIGHTS: &str = "needs CAP_BPF, CAP_PERFMON and CAP_NET_ADMIN";

/// Up to the verifier's log, which can be long.
fn first_sentence(error: &str) -> &str {
    error.split(". Verifier output").next().unwrap_or(error)
}

/// Whether the kernel checks sources strictly on `interface` (rp_filter
/// 1: the larger of `all` and the interface's).
fn strict_rp_filter(interface: &str) -> bool {
    let read = |conf: &str| {
        std::fs::read_to_string(format!("/proc/sys/net/ipv4/conf/{conf}/rp_filter"))
            .ok()
            .and_then(|v| v.trim().parse::<u8>().ok())
            .unwrap_or(0)
    };
    read("all").max(read(interface)) == 1
}

/// The interface with address `addr`, else the first whose network has
/// it (127.0.0.2: `lo`).
fn interface_of(addr: Ipv4Addr) -> Option<String> {
    let mut list = std::ptr::null_mut();
    // SAFETY: getifaddrs fills `list`, freed below.
    if unsafe { libc::getifaddrs(&mut list) } != 0 {
        return None;
    }
    let (mut exact, mut network) = (None, None);
    let mut next = list;
    while !next.is_null() {
        // SAFETY: a node of the list getifaddrs gave, not freed yet.
        let ifa = unsafe { &*next };
        next = ifa.ifa_next;
        let v4 = |p: *mut libc::sockaddr| {
            // SAFETY: a sockaddr of family AF_INET is a sockaddr_in.
            (!p.is_null() && unsafe { (*p).sa_family } == libc::AF_INET as libc::sa_family_t)
                .then(|| Ipv4Addr::from(u32::from_be(unsafe { (*p.cast::<libc::sockaddr_in>()).sin_addr.s_addr })))
        };
        let (Some(ip), Some(mask)) = (v4(ifa.ifa_addr), v4(ifa.ifa_netmask)) else { continue };
        // SAFETY: ifa_name is a NUL-terminated string of the list.
        let name = unsafe { std::ffi::CStr::from_ptr(ifa.ifa_name) }.to_string_lossy().into_owned();
        let mask = u32::from(mask);
        if ip == addr {
            exact = Some(name);
        } else if network.is_none() && u32::from(ip) & mask == u32::from(addr) & mask {
            network = Some(name);
        }
    }
    // SAFETY: the list getifaddrs gave, no longer used.
    unsafe { libc::freeifaddrs(list) };
    exact.or(network)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_interface_is_found_by_address_or_network() {
        assert_eq!(interface_of(Ipv4Addr::LOCALHOST).as_deref(), Some("lo"));
        assert_eq!(interface_of(Ipv4Addr::new(127, 0, 0, 2)).as_deref(), Some("lo"), "lo's network");
        assert_eq!(interface_of(Ipv4Addr::new(192, 0, 2, 77)), None, "TEST-NET-1: nobody's");
    }

    #[test]
    fn the_verifier_log_is_cut() {
        let e = "the BPF_PROG_LOAD syscall returned Permission denied (os error 13). Verifier output: 0: R1=ctx()";
        assert_eq!(first_sentence(e), "the BPF_PROG_LOAD syscall returned Permission denied (os error 13)");
        assert_eq!(first_sentence("no map"), "no map");
    }

    #[test]
    fn counts_are_given_once() {
        let a = Counts { packets: 5, bytes: 500, rate_limited: 1 };
        let b = Counts { packets: 7, bytes: 900, rate_limited: 1 };
        assert_eq!(b.minus(a), Counts { packets: 2, bytes: 400, rate_limited: 0 });
    }
}
