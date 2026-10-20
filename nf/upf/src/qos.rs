//! A session's QoS on the user plane (the PCF's policy, through the SMF):
//!
//! - flows: a downlink packet goes on the flow of the first PCC rule (in
//!   precedence order) with a filter it matches, else on the default flow;
//!   an uplink packet must come on one of the session's flows;
//! - rates: a packet beyond its flow's MBR, or beyond the session AMBR on
//!   a non-GBR flow (TS 23.501 §5.7.2.6), is dropped. Each limit is a GCRA
//!   (a token bucket kept as one time), with [`BURST`] of traffic as burst.

use std::time::Duration;

use librecn_sbi::policy::{DEFAULT_QFI, MAX_RULES, PacketFilter, PccRule, Rates, SessionRule, direction, rule_qfi};
use librecn_scheduler::Time;

/// How much traffic, in time at the limit's rate, may go in one burst.
pub const BURST: Duration = Duration::from_millis(100);

/// What filters look at in an IPv4 packet.
struct Fields {
    protocol: u8,
    src: u32,
    /// Source and destination ports (TCP, UDP, SCTP; not in a later fragment).
    ports: Option<(u16, u16)>,
}

fn fields(packet: &[u8]) -> Option<Fields> {
    let [v, _, _, _, _, _, f0, f1, _, protocol, _, _, s0, s1, s2, s3, _, _, _, _, ..] = *packet else {
        return None;
    };
    if v >> 4 != 4 {
        return None;
    }
    let first_fragment = u16::from_be_bytes([f0, f1]) & 0x1fff == 0;
    let header = usize::from(v & 0x0f) * 4;
    let ports = match (protocol, packet.get(header..header + 4)) {
        (6 | 17 | 132, Some(&[a, b, c, d])) if first_fragment => {
            Some((u16::from_be_bytes([a, b]), u16::from_be_bytes([c, d])))
        }
        _ => None,
    };
    Some(Fields { protocol, src: u32::from_be_bytes([s0, s1, s2, s3]), ports })
}

/// Whether `port` is in `range` (0, 0: any port, even none).
fn in_range(range: [u16; 2], port: Option<u16>) -> bool {
    range == [0, 0] || port.is_some_and(|p| (range[0]..=range[1]).contains(&p))
}

/// Whether filter `f` matches a downlink packet with `fields` (its source
/// is the remote end, seen from the UE).
fn matches(f: &PacketFilter, fields: &Fields) -> bool {
    let mask = u32::MAX.checked_shl(32 - u32::from(f.remote_prefix.min(32))).unwrap_or(0);
    f.direction & direction::DOWNLINK != 0
        && (f.protocol == 0 || f.protocol == fields.protocol)
        && (fields.src ^ u32::from_be_bytes(f.remote)) & mask == 0
        && in_range(f.remote_ports, fields.ports.map(|p| p.0))
        && in_range(f.local_ports, fields.ports.map(|p| p.1))
}

/// One rate limit (GCRA): a packet conforms unless the traffic let through
/// would take more than [`BURST`] beyond now at the rate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Limit {
    /// Bit/s; 0: no limit.
    rate: u64,
    /// When the traffic let through so far is done at the rate (ns).
    done: u64,
}

impl Limit {
    fn conforms(&self, now: u64) -> bool {
        self.rate == 0 || self.done.saturating_sub(now) <= BURST.as_nanos() as u64
    }

    fn take(&mut self, now: u64, bytes: usize) {
        if self.rate != 0 {
            let ns = (bytes as u128 * 8 * 1_000_000_000 / u128::from(self.rate)).min(u128::from(u64::MAX)) as u64;
            self.done = self.done.max(now).saturating_add(ns);
        }
    }
}

/// Uplink and downlink limits of `rates`.
fn limits(rates: Rates) -> [Limit; 2] {
    [rates.uplink, rates.downlink].map(|rate| Limit { rate, done: 0 })
}

/// A session's PCC rules and their MBRs.
#[derive(Debug, PartialEq, Eq)]
struct Flows {
    rule: SessionRule,
    mbr: [[Limit; 2]; MAX_RULES],
}

/// A session's QoS: its AMBR, and its PCC rules if it has any (boxed: most
/// sessions have none, and the rules are 440 bytes).
#[derive(Debug, PartialEq, Eq)]
pub struct Qos {
    ambr: [Limit; 2],
    flows: Option<Box<Flows>>,
}

impl Qos {
    pub fn new(rule: &SessionRule) -> Self {
        let flows = (!rule.rules().is_empty()).then(|| {
            let mut mbr = [[Limit::default(); 2]; MAX_RULES];
            for (m, r) in mbr.iter_mut().zip(rule.rules()) {
                *m = limits(r.mbr);
            }
            Box::new(Flows { rule: *rule, mbr })
        });
        Qos { ambr: limits(rule.ambr), flows }
    }

    /// The session AMBR uplink, bit/s (0: none).
    pub fn uplink_ambr(&self) -> u64 {
        self.ambr[0].rate
    }

    /// The session's PCC rules.
    pub fn rules(&self) -> &[PccRule] {
        self.flows.as_ref().map_or(&[], |f| f.rule.rules())
    }

    /// The QoS flow of downlink `packet` (IPv4): its QFI.
    pub fn downlink(&self, packet: &[u8]) -> u8 {
        if self.rules().is_empty() {
            return DEFAULT_QFI;
        }
        let Some(fields) = fields(packet) else { return DEFAULT_QFI };
        match self.rules().iter().position(|r| r.filters().iter().any(|f| matches(f, &fields))) {
            Some(i) => rule_qfi(i),
            None => DEFAULT_QFI,
        }
    }

    /// Whether an uplink packet's `qfi` is one of the session's flows
    /// (none: no PDU Session Container, the default flow).
    pub fn uplink_known(&self, qfi: Option<u8>) -> bool {
        qfi.is_none_or(|q| q == DEFAULT_QFI || (rule_qfi(0)..rule_qfi(self.rules().len())).contains(&q))
    }

    /// Whether a packet of `bytes` on flow `qfi` (one of the session's),
    /// `downlink` or up, may go at `now`; if so, its share is taken.
    pub fn allow(&mut self, qfi: u8, downlink: bool, now: Time, bytes: usize) -> bool {
        let (way, now) = (usize::from(downlink), now.as_nanos());
        let i = usize::from(qfi).wrapping_sub(usize::from(rule_qfi(0)));
        let flow = self.flows.as_deref_mut().filter(|f| i < f.rule.rules().len());
        // A GBR flow is held to its MBR only, outside the session AMBR.
        let (flow, gbr) = match flow {
            Some(f) => (Some(&mut f.mbr[i][way]), !f.rule.rules()[i].gbr.is_zero()),
            None => (None, false),
        };
        let ambr = (!gbr).then_some(&mut self.ambr[way]);
        let conforms = |l: &Option<&mut Limit>| l.as_ref().is_none_or(|l| l.conforms(now));
        if !conforms(&flow) || !conforms(&ambr) {
            return false;
        }
        for limit in flow.into_iter().chain(ambr) {
            limit.take(now, bytes);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zerocopy::FromZeros;

    /// An IPv4 packet from `src` to `dst`, its protocol and ports.
    fn packet(protocol: u8, src: [u8; 4], dst: [u8; 4], ports: (u16, u16)) -> Vec<u8> {
        let mut p = vec![0x45, 0, 0, 28, 0, 0, 0, 0, 64, protocol, 0, 0];
        p.extend(src);
        p.extend(dst);
        p.extend(ports.0.to_be_bytes());
        p.extend(ports.1.to_be_bytes());
        p.extend([0, 8, 0, 0]);
        p
    }

    fn rule(filters: &[&[PacketFilter]]) -> SessionRule {
        let mut rule = SessionRule::new_zeroed();
        let rules: Vec<_> = filters
            .iter()
            .map(|f| {
                let mut r = PccRule::new_zeroed();
                r.set_filters(f);
                r
            })
            .collect();
        rule.set_rules(&rules);
        rule
    }

    const UE: [u8; 4] = [10, 45, 0, 2];

    #[test]
    fn downlink_packets_take_the_first_matching_rule() {
        let sip = PacketFilter::new(direction::BOTH, 17, ([10, 10, 0, 0], 16), [5060, 5061], [0, 0]);
        let pings = PacketFilter::new(direction::DOWNLINK, 1, ([10, 45, 0, 1], 32), [0, 0], [0, 0]);
        let web = PacketFilter::new(direction::DOWNLINK, 6, ([0; 4], 0), [0, 0], [8000, 8080]);
        let up_only = PacketFilter::new(direction::UPLINK, 0, ([0; 4], 0), [0, 0], [0, 0]);
        let r = rule(&[&[up_only, sip], &[pings], &[web]]);

        assert_eq!(Qos::new(&r).downlink(&packet(17, [10, 10, 3, 4], UE, (5061, 40000))), 2, "SIP");
        assert_eq!(Qos::new(&r).downlink(&packet(17, [10, 11, 3, 4], UE, (5061, 40000))), 1, "another network");
        assert_eq!(Qos::new(&r).downlink(&packet(17, [10, 10, 3, 4], UE, (5062, 40000))), 1, "another port");
        assert_eq!(Qos::new(&r).downlink(&packet(1, [10, 45, 0, 1], UE, (0, 0))), 3, "ICMP, no ports");
        assert_eq!(Qos::new(&r).downlink(&packet(6, [1, 2, 3, 4], UE, (443, 8080))), 4, "local port");
        assert_eq!(Qos::new(&r).downlink(&packet(6, [1, 2, 3, 4], UE, (8080, 443))), 1, "a remote port");
        assert_eq!(Qos::new(&r).downlink(&[0x60, 0, 0, 0]), 1, "not IPv4");

        let mut fragment = packet(17, [10, 10, 3, 4], UE, (5061, 40000));
        fragment[7] = 1;
        assert_eq!(Qos::new(&r).downlink(&fragment), 1, "a later fragment has no ports");
        assert_eq!(Qos::new(&rule(&[])).downlink(&packet(1, [10, 45, 0, 1], UE, (0, 0))), 1, "no rules");
    }

    /// Bytes let through in `seconds`, downlink: 1250-byte packets (10 kbit)
    /// offered every millisecond (10 Mbit/s), on the flows `qfis` in turn.
    fn through(qos: &mut Qos, qfis: &[u8], seconds: u64) -> u64 {
        let mut bytes = 0;
        for ms in 0..seconds * 1000 {
            let qfi = qfis[ms as usize % qfis.len()];
            if qos.allow(qfi, true, Time::from_nanos(ms * 1_000_000 + 1), 1250) {
                bytes += 1250;
            }
        }
        bytes
    }

    #[test]
    fn rates_are_held_to_the_ambr_and_the_mbrs() {
        let mbps = |up: u64, down: u64| Rates { uplink: up * 1_000_000, downlink: down * 1_000_000 };
        let any = PacketFilter::new(direction::BOTH, 0, ([0; 4], 0), [0, 0], [0, 0]);
        let mut r = rule(&[&[any], &[any]]);
        r.ambr = mbps(1, 2);
        let mut flows = r.rules().to_vec();
        flows[0].mbr = mbps(1, 1); // non-GBR: its MBR, and the AMBR
        (flows[1].mbr, flows[1].gbr) = (mbps(4, 4), mbps(1, 1)); // GBR: its MBR only
        r.set_rules(&flows);

        // 2 Mbit/s for 10 s is 2.5 MB, plus the first burst (100 ms: 25 kB).
        let ambr = through(&mut Qos::new(&r), &[1], 10);
        assert!((2_500_000..=2_530_000).contains(&ambr), "AMBR 2 Mbit/s: {ambr} bytes");
        let flow = through(&mut Qos::new(&r), &[2], 10);
        assert!((1_250_000..=1_270_000).contains(&flow), "MBR 1 Mbit/s: {flow} bytes");
        let gbr = through(&mut Qos::new(&r), &[3], 10);
        assert!((5_000_000..=5_060_000).contains(&gbr), "GBR, MBR 4 Mbit/s: {gbr} bytes");

        // A non-GBR flow shares the AMBR with the default flow; a GBR one does not.
        let both = through(&mut Qos::new(&r), &[1, 2], 10);
        assert!((2_500_000..=2_530_000).contains(&both), "AMBR shared: {both} bytes");
        let apart = through(&mut Qos::new(&r), &[1, 3], 10);
        assert!((7_500_000..=7_580_000).contains(&apart), "2.5 MB and 5 MB, and both bursts: {apart} bytes");

        // Uplink is another limit; no rate, no limit.
        let mut l = Qos::new(&r);
        through(&mut l, &[1], 1);
        assert!(l.allow(1, false, Time::from_nanos(1_000_000_000), 1250), "uplink untouched");
        assert_eq!(through(&mut Qos::new(&rule(&[])), &[1], 1), 1_250_000, "no AMBR: all of it");
    }

    #[test]
    fn uplink_qfis_must_be_the_sessions() {
        let any = PacketFilter::new(direction::BOTH, 0, ([0; 4], 0), [0, 0], [0, 0]);
        let q = Qos::new(&rule(&[&[any], &[any]]));
        assert!(q.uplink_known(None) && q.uplink_known(Some(1)) && q.uplink_known(Some(3)));
        assert!(!q.uplink_known(Some(4)) && !q.uplink_known(Some(0)));
        assert!(!Qos::new(&rule(&[])).uplink_known(Some(2)));
    }
}
