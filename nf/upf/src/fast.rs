//! The fast path (`fast_path`, on by default): the kernel carries the
//! sessions it can before the UPF sees them (`bpf/upf.bpf.c`). Uplink: XDP
//! on the N3 interface decapsulates, and the UE's packet goes on into the
//! kernel's stack. Downlink: TC on the N6 device's egress encapsulates, and
//! the G-PDU leaves by the N3 interface. What it cannot carry reaches the
//! UPF's socket or the N6 device, as without it (the fallback). It needs `CAP_BPF`, `CAP_PERFMON` and
//! `CAP_NET_ADMIN`, and a loose reverse path filter on the N3 interface
//! (the UE's packets come in there, while their addresses are routed to
//! N6); without them, or on a kernel without XDP, the UPF says why and runs
//! without it.
//!
//! The UPF copies each session into the kernel's `uplink` map (its TEID, UE
//! address, cell slot, uplink AMBR), while the gNB's tunnel is known into
//! `downlink` (its UE address, the gNB's address and TEID, cell slot,
//! downlink AMBR), and its PCC rules into `rules` (their MBRs, GBR or not,
//! their downlink filters: the kernel classifies as [`crate::qos`] does);
//! removes them with the session (the downlink when the UE goes idle: its
//! packets reach the UPF, which parks them); and folds the kernel's counts
//! into its own.

use std::net::Ipv4Addr;

use aya::maps::{HashMap, MapData, PerCpuArray};
use aya::programs::tc::{self, SchedClassifier, TcAttachType};
use aya::programs::{Xdp, XdpMode};
use aya::{EbpfLoader, Pod};
use librecn_sbi::policy::{MAX_FILTERS, MAX_RULES, PccRule, direction};

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

/// `struct downlink`: a session's downlink, by its UE's address.
#[repr(C)]
#[derive(Clone, Copy)]
struct Downlink {
    /// The gNB's N3 address and TEID, network order.
    gnb: u32,
    teid: u32,
    /// Its slot in `counts`; [`MAX_CELLS`]: none.
    cell: u32,
    pad: u32,
    /// Downlink AMBR, bit/s; 0: no limit.
    rate: u64,
    /// The kernel's GCRA state (ns).
    done: u64,
}

// SAFETY: integers only, without padding (4 * 4 + 8 + 8 bytes).
unsafe impl Pod for Downlink {}

/// `struct filter`: a downlink packet filter.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Filter {
    /// Network order, masked.
    remote: u32,
    mask: u32,
    remote_ports: [u16; 2],
    local_ports: [u16; 2],
    protocol: u8,
    pad: [u8; 3],
}

/// `struct rule`: a PCC rule's QoS flow.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Rule {
    /// Uplink, downlink, bit/s.
    mbr: [u64; 2],
    done: [u64; 2],
    gbr: u32,
    filters: u32,
    f: [Filter; MAX_FILTERS],
}

/// `struct rules`: a session's PCC rules, in precedence order.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Rules {
    count: u32,
    pad: u32,
    r: [Rule; MAX_RULES],
}

// SAFETY: integers only, without padding (20, 120 and 488 bytes, aligned).
unsafe impl Pod for Filter {}
// SAFETY: as above.
unsafe impl Pod for Rule {}
// SAFETY: as above.
unsafe impl Pod for Rules {}

impl Rules {
    /// The kernel's copy of `rules`: their downlink filters only (the
    /// kernel classifies the downlink alone).
    fn of(rules: &[PccRule]) -> Rules {
        let mut out = Rules { count: rules.len() as u32, ..Rules::default() };
        for (r, rule) in out.r.iter_mut().zip(rules) {
            r.mbr = [rule.mbr.uplink, rule.mbr.downlink];
            r.gbr = u32::from(!rule.gbr.is_zero());
            let downlink = rule.filters().iter().filter(|f| f.direction & direction::DOWNLINK != 0);
            for (f, filter) in r.f.iter_mut().zip(downlink) {
                let prefix = u32::from(filter.remote_prefix.min(32));
                let mask = u32::MAX.checked_shl(32 - prefix).unwrap_or(0);
                *f = Filter {
                    remote: (u32::from_be_bytes(filter.remote) & mask).to_be(),
                    mask: mask.to_be(),
                    remote_ports: filter.remote_ports,
                    local_ports: filter.local_ports,
                    protocol: filter.protocol,
                    pad: [0; 3],
                };
                r.filters += 1;
            }
        }
        out
    }
}

/// `struct counts`: what the kernel carried and dropped, for one cell slot
/// (the UEs' packets, their bytes without the tunnel's headers).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub up_packets: u64,
    pub up_bytes: u64,
    pub down_packets: u64,
    pub down_bytes: u64,
    pub rate_limited: u64,
    /// Downlink packets on a PCC rule's flow.
    pub down_extra: u64,
}

// SAFETY: integers only, without padding.
unsafe impl Pod for Counts {}

impl Counts {
    fn plus(self, other: Counts, f: fn(u64, u64) -> u64) -> Counts {
        Counts {
            up_packets: f(self.up_packets, other.up_packets),
            up_bytes: f(self.up_bytes, other.up_bytes),
            down_packets: f(self.down_packets, other.down_packets),
            down_bytes: f(self.down_bytes, other.down_bytes),
            rate_limited: f(self.rate_limited, other.rate_limited),
            down_extra: f(self.down_extra, other.down_extra),
        }
    }
}

/// The programs, attached while this lives (their links go with the
/// process), and their maps.
pub struct FastPath {
    _ebpf: aya::Ebpf,
    interface: String,
    uplink: HashMap<MapData, u32, Uplink>,
    downlink: HashMap<MapData, u32, Downlink>,
    rules: HashMap<MapData, u32, Rules>,
    counts: PerCpuArray<MapData, Counts>,
    /// What [`take`](Self::take) has given so far, per slot.
    given: Vec<Counts>,
}

impl FastPath {
    /// Loads the programs and attaches them to the interface with address
    /// `n3` (uplink) and to N6 device `n6` (downlink), room for
    /// `max_sessions`.
    pub fn start(n3: Ipv4Addr, n6: &str, max_sessions: usize) -> Result<Self, String> {
        let interface = interface_of(n3).ok_or_else(|| format!("no interface has {n3}"))?;
        if strict_rp_filter(&interface) {
            return Err(format!(
                "strict reverse path filter on {interface} would drop the UEs' packets: \
                 net.ipv4.conf.all.rp_filter and net.ipv4.conf.{interface}.rp_filter must be 0 or 2"
            ));
        }
        let addr = u32::from(n3).to_be();
        let index = ifindex(&interface)?;
        let mtu: u32 = std::fs::read_to_string(format!("/sys/class/net/{interface}/mtu"))
            .ok()
            .and_then(|m| m.trim().parse().ok())
            .unwrap_or(1500);
        let max = u32::try_from(max_sessions).map_err(|_| "too many sessions".to_string())?.max(1);
        let mut ebpf = EbpfLoader::new()
            .override_global("n3_addr", &addr, true)
            .override_global("n3_ifindex", &index, true)
            .override_global("n3_mtu", &mtu, true)
            .map_max_entries("uplink", max)
            .map_max_entries("downlink", max)
            .map_max_entries("rules", max)
            .load(OBJECT)
            .map_err(|e| format!("loading ({RIGHTS}): {e}"))?;
        let program: &mut Xdp =
            ebpf.program_mut("upf_uplink").ok_or("no program upf_uplink")?.try_into().map_err(|e| format!("{e}"))?;
        program.load().map_err(|e| format!("verifier ({RIGHTS}): {}", first_sentence(&e.to_string())))?;
        program.attach(&interface, XdpMode::Default).map_err(|e| format!("attaching to {interface}: {e}"))?;
        let program: &mut SchedClassifier = ebpf
            .program_mut("upf_downlink")
            .ok_or("no program upf_downlink")?
            .try_into()
            .map_err(|e| format!("{e}"))?;
        program.load().map_err(|e| format!("verifier ({RIGHTS}): {}", first_sentence(&e.to_string())))?;
        // Before Linux 6.6, through a qdisc (netlink): a filter left by a
        // UPF that crashed goes first. Since 6.6, a link (TCX).
        let _ = tc::qdisc_detach_program(n6, TcAttachType::Egress, "upf_downlink");
        let _ = tc::qdisc_add_clsact(n6);
        program.attach(n6, TcAttachType::Egress).map_err(|e| format!("attaching to {n6}: {e}"))?;
        let mut map = |name: &str| ebpf.take_map(name).ok_or(format!("no map {name}"));
        let uplink = HashMap::try_from(map("uplink")?).map_err(|e| e.to_string())?;
        let downlink = HashMap::try_from(map("downlink")?).map_err(|e| e.to_string())?;
        let rules = HashMap::try_from(map("rules")?).map_err(|e| e.to_string())?;
        let counts = PerCpuArray::try_from(map("counts")?).map_err(|e| e.to_string())?;
        let given = vec![Counts::default(); MAX_CELLS + 1];
        Ok(FastPath { _ebpf: ebpf, interface, uplink, downlink, rules, counts, given })
    }

    /// The interface it is attached to.
    pub fn interface(&self) -> &str {
        &self.interface
    }

    /// Session `s` carried by the kernel, if there is room: its PCC rules,
    /// its uplink, and its downlink while its gNB's tunnel is known; again
    /// after a change (tunnel, cell).
    pub fn add(&mut self, s: &Session) -> bool {
        let (ue_ip, cell) = (u32::from(s.ue_ip).to_be(), s.cell.map_or(MAX_CELLS as u32, u32::from));
        let mut added = match s.qos.rules() {
            [] => {
                let _ = self.rules.remove(&ue_ip);
                true
            }
            rules => self.rules.insert(ue_ip, Rules::of(rules), 0).is_ok(),
        };
        let up = Uplink { ue_ip, cell, rate: s.qos.uplink_ambr(), done: 0 };
        added &= self.uplink.insert(s.ul_teid.to_be(), up, 0).is_ok();
        match s.dl {
            Some((gnb, teid)) => {
                let (gnb, teid) = (u32::from(*gnb.ip()).to_be(), teid.to_be());
                let down = Downlink { gnb, teid, cell, pad: 0, rate: s.qos.downlink_ambr(), done: 0 };
                added &= self.downlink.insert(ue_ip, down, 0).is_ok();
            }
            None => {
                let _ = self.downlink.remove(&ue_ip);
            }
        }
        added
    }

    /// Session `s`, now gone.
    pub fn remove(&mut self, s: &Session) {
        let ue_ip = u32::from(s.ue_ip).to_be();
        let _ = self.uplink.remove(&s.ul_teid.to_be());
        let _ = self.downlink.remove(&ue_ip);
        let _ = self.rules.remove(&ue_ip);
    }

    /// What the kernel carried since the last call, for the cell slots
    /// below `cells` (`Some(slot)`) and without a cell (`None`).
    pub fn take(&mut self, cells: usize, mut f: impl FnMut(Option<usize>, Counts)) {
        for slot in (0..cells.min(MAX_CELLS)).chain([MAX_CELLS]) {
            let Ok(per_cpu) = self.counts.get(&(slot as u32), 0) else { continue };
            let total = per_cpu.iter().fold(Counts::default(), |a, c| a.plus(*c, u64::wrapping_add));
            let new = total.plus(self.given[slot], u64::wrapping_sub);
            self.given[slot] = total;
            if new != Counts::default() {
                f((slot < MAX_CELLS).then_some(slot), new);
            }
        }
    }
}

fn ifindex(interface: &str) -> Result<u32, String> {
    let name = std::ffi::CString::new(interface).map_err(|e| e.to_string())?;
    // SAFETY: a NUL-terminated string, which outlives the call.
    match unsafe { libc::if_nametoindex(name.as_ptr()) } {
        0 => Err(format!("no interface {interface}")),
        index => Ok(index),
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
    fn the_kernel_gets_the_rules_downlink_filters() {
        use librecn_sbi::policy::{PacketFilter, Rates};
        use zerocopy::FromZeros;
        let sip = PacketFilter::new(direction::BOTH, 17, ([10, 10, 3, 4], 16), [5060, 5061], [0, 0]);
        let up_only = PacketFilter::new(direction::UPLINK, 6, ([0; 4], 0), [0, 0], [80, 80]);
        let mut voice = PccRule::new_zeroed();
        voice.set_filters(&[up_only, sip]);
        (voice.mbr, voice.gbr) = (Rates { uplink: 1, downlink: 2 }, Rates { uplink: 1, downlink: 1 });
        let any = PccRule::new_zeroed();
        let k = Rules::of(&[voice, any]);
        assert_eq!((k.count, k.r[0].mbr, k.r[0].gbr, k.r[0].filters, k.r[1].gbr), (2, [1, 2], 1, 1, 0));
        let f = k.r[0].f[0];
        assert_eq!((f.protocol, f.remote_ports, f.local_ports), (17, [5060, 5061], [0, 0]));
        assert_eq!((u32::from_be(f.remote), u32::from_be(f.mask)), (0x0a0a_0000, 0xffff_0000), "masked");
        let all = PacketFilter::new(direction::DOWNLINK, 0, ([1, 2, 3, 4], 0), [0, 0], [0, 0]);
        let mut r = PccRule::new_zeroed();
        r.set_filters(&[all]);
        assert_eq!((Rules::of(&[r]).r[0].f[0].remote, Rules::of(&[r]).r[0].f[0].mask), (0, 0), "any address");
    }

    #[test]
    fn counts_are_given_once() {
        let a =
            Counts { up_packets: 5, up_bytes: 500, down_packets: 1, down_bytes: 90, rate_limited: 1, down_extra: 0 };
        let b =
            Counts { up_packets: 7, up_bytes: 900, down_packets: 4, down_bytes: 290, rate_limited: 1, down_extra: 2 };
        let new =
            Counts { up_packets: 2, up_bytes: 400, down_packets: 3, down_bytes: 200, rate_limited: 0, down_extra: 2 };
        assert_eq!(b.plus(a, u64::wrapping_sub), new);
        assert_eq!(a.plus(new, u64::wrapping_add), b);
    }
}
