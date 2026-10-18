//! A session's QoS flows on the user plane (the PCF's PCC rules, through
//! the SMF): a downlink packet goes on the flow of the first rule (in
//! precedence order) with a filter it matches, else on the default flow;
//! an uplink packet must come on one of the session's flows.

use librecn_sbi::policy::{DEFAULT_QFI, PacketFilter, SessionRule, direction, rule_qfi};

/// What filters look at in an IPv4 packet.
struct Fields {
    protocol: u8,
    src: u32,
    dst: u32,
    /// Source and destination ports (TCP, UDP, SCTP; not in a later fragment).
    ports: Option<(u16, u16)>,
}

fn fields(packet: &[u8]) -> Option<Fields> {
    let [v, _, _, _, _, _, f0, f1, _, protocol, _, _, s0, s1, s2, s3, d0, d1, d2, d3, ..] = *packet else {
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
    Some(Fields {
        protocol,
        src: u32::from_be_bytes([s0, s1, s2, s3]),
        dst: u32::from_be_bytes([d0, d1, d2, d3]),
        ports,
    })
}

/// Whether `port` is in `range` (0, 0: any port, even none).
fn in_range(range: [u16; 2], port: Option<u16>) -> bool {
    range == [0, 0] || port.is_some_and(|p| (range[0]..=range[1]).contains(&p))
}

/// Whether filter `f` matches a packet with `fields`, going down to the UE
/// (`downlink`) or up from it.
fn matches(f: &PacketFilter, fields: &Fields, downlink: bool) -> bool {
    let wanted = if downlink { direction::DOWNLINK } else { direction::UPLINK };
    // Seen from the UE: the remote end is the source of a downlink packet.
    let (remote, ports) =
        if downlink { (fields.src, fields.ports) } else { (fields.dst, fields.ports.map(|(src, dst)| (dst, src))) };
    let mask = u32::MAX.checked_shl(32 - u32::from(f.remote_prefix.min(32))).unwrap_or(0);
    f.direction & wanted != 0
        && (f.protocol == 0 || f.protocol == fields.protocol)
        && (remote ^ u32::from_be_bytes(f.remote)) & mask == 0
        && in_range(f.remote_ports, ports.map(|p| p.0))
        && in_range(f.local_ports, ports.map(|p| p.1))
}

/// The flow of downlink `packet` (IPv4): its QFI, and the index of its PCC rule.
pub fn downlink(rule: &SessionRule, packet: &[u8]) -> (u8, Option<usize>) {
    if rule.rules().is_empty() {
        return (DEFAULT_QFI, None);
    }
    let Some(fields) = fields(packet) else { return (DEFAULT_QFI, None) };
    match rule.rules().iter().position(|r| r.filters().iter().any(|f| matches(f, &fields, true))) {
        Some(i) => (rule_qfi(i), Some(i)),
        None => (DEFAULT_QFI, None),
    }
}

/// Whether an uplink packet's `qfi` is one of the session's flows (none:
/// no PDU Session Container, the default flow).
pub fn uplink_known(rule: &SessionRule, qfi: Option<u8>) -> bool {
    qfi.is_none_or(|q| q == DEFAULT_QFI || (rule_qfi(0)..rule_qfi(rule.rules().len())).contains(&q))
}

#[cfg(test)]
mod tests {
    use super::*;
    use librecn_sbi::policy::PccRule;
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

        assert_eq!(downlink(&r, &packet(17, [10, 10, 3, 4], UE, (5061, 40000))), (2, Some(0)), "SIP");
        assert_eq!(downlink(&r, &packet(17, [10, 11, 3, 4], UE, (5061, 40000))), (1, None), "another network");
        assert_eq!(downlink(&r, &packet(17, [10, 10, 3, 4], UE, (5062, 40000))), (1, None), "another port");
        assert_eq!(downlink(&r, &packet(1, [10, 45, 0, 1], UE, (0, 0))), (3, Some(1)), "ICMP, no ports");
        assert_eq!(downlink(&r, &packet(6, [1, 2, 3, 4], UE, (443, 8080))), (4, Some(2)), "local port");
        assert_eq!(downlink(&r, &packet(6, [1, 2, 3, 4], UE, (8080, 443))), (1, None), "a remote port");
        assert_eq!(downlink(&r, &[0x60, 0, 0, 0]), (1, None), "not IPv4");

        let mut fragment = packet(17, [10, 10, 3, 4], UE, (5061, 40000));
        fragment[7] = 1;
        assert_eq!(downlink(&r, &fragment), (1, None), "a later fragment has no ports");
        assert_eq!(downlink(&rule(&[]), &packet(1, [10, 45, 0, 1], UE, (0, 0))), (1, None), "no rules");
    }

    #[test]
    fn uplink_qfis_must_be_the_sessions() {
        let any = PacketFilter::new(direction::BOTH, 0, ([0; 4], 0), [0, 0], [0, 0]);
        let r = rule(&[&[any], &[any]]);
        assert!(uplink_known(&r, None) && uplink_known(&r, Some(1)) && uplink_known(&r, Some(3)));
        assert!(!uplink_known(&r, Some(4)) && !uplink_known(&r, Some(0)));
        assert!(!uplink_known(&rule(&[]), Some(2)));
    }
}
