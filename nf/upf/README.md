# UPF (`librecn-upf`)

The user plane function, one EO: N4 sessions from the SMF (EO messages),
GTP-U on N3 (UDP 2152 of `n3`), a TUN device on N6 (`n6`). See the crate
documentation for the parameters.

## Idle UEs

A downlink packet for a session without downlink tunnel (the UE is idle) is
parked, and the SMF told (`N4Report`, at most every 10 s per session): the
AMF pages the UE, whose Service Request brings the tunnel back, and the
parked packets go first. 64 packets are kept in all (2 KiB at most each,
preallocated: 128 KiB), the oldest dropped when full.

## Fast path (XDP)

On by default (`fast_path = false` turns it off): the kernel carries the
uplink of the sessions it can before the UPF sees it. A program
([`bpf/upf.bpf.c`](bpf/upf.bpf.c), XDP) on the interface of the N3 address
takes each G-PDU to the UPF whose TEID is a session's, checks that its
inner source is the session's UE, polices the session AMBR uplink (the
same GCRA as `qos.rs`), strips the outer IPv4, UDP and GTP-U headers and
lets the UE's packet go on into the kernel's stack, which routes it out
(NAT included). The UPF's socket, its userspace and the TUN device are
skipped.

Everything else reaches the UPF's socket untouched, as without the fast
path (the fallback): echo, unknown TEIDs, a wrong source, other GTP-U
layouts (the program takes no optional field, or a 4-byte PDU Session
Container on the default flow), and sessions with PCC rules (extra QoS
flows: `fast.rs` does not give them to the kernel). The downlink is the
UPF's for now.

- The UPF gives each session to the kernel's map at establishment (again
  when its cell changes) and removes it with the session; every second it
  adds the kernel's counts to its own (`uplink_packets`, `uplink_bytes`,
  `rate_limited_packets`, the cells'), and `fast_path_packets` says how
  many uplink packets the kernel carried.
- It needs `CAP_BPF`, `CAP_PERFMON` and `CAP_NET_ADMIN`, and a loose
  reverse path filter on the N3 interface (`rp_filter` 0 or 2: the UE's
  packets come in there, while their addresses are routed to N6). Without
  them, the UPF logs why, raises a MINOR alarm and runs without it. The
  program is attached through a BPF link: it goes when the UPF's process
  goes.
- The program is C, built by [`bpf/build.sh`](bpf/build.sh) (clang, in
  Docker if the host has none) into `bpf/upf.bpf.o`, which is checked in:
  cargo needs no clang. `aya` loads it.

Measured by `scripts/upf-fast.sh` (the `fast_path` example): the UPF in
one container, a gNB's three senders in another, over veth (where XDP runs
natively); the receive path of the N3 interface (veth's NAPI, in a thread)
pinned to its own core, the UPF to another, the UEs' packets forwarded out
of a dummy device standing for the data network. "Fallback": the same
sessions with a PCC rule, so the program passes them to the UPF. AMD Ryzen
9 5950X, Linux 7.0, 2026-10-20, 100 UEs (1000: the same), two runs within
2 %:

| uplink | packet | carried | UPF core | receive core |
|---|---|---|---|---|
| fast path | 84 B | 700 kpps, 0.47 Gbit/s | 1 % | 100 % |
| fallback | 84 B | 361 kpps, 0.24 Gbit/s | 100 % | 100 % |
| fast path | 1400 B | 637 kpps, 7.1 Gbit/s | 2 % | 100 % |
| fallback | 1400 B | 357 kpps, 4.0 Gbit/s | 100 % | 100 % |

- **One core instead of two, and 1.8 times the packets**: the fast path is
  bound by the receive core alone (about 1.5 µs per packet: veth's receive,
  the program, the kernel's forwarding); the UPF's core stays free.
- veth copies each packet for XDP (its packets have no room in front for
  it); a network card's driver runs XDP on the packet as received, so
  these figures are a floor.

## Batches

N3 moves datagrams in batches: up to 32 received with one `recvmmsg`, up
to 32 sent with one `sendmmsg` (`batch.rs`). A downlink packet is read from
the TUN device straight into its slot, behind room for its GTP-U header, and
goes with the others when the device has nothing more (or the batch is
full). The device still takes one system call per packet. Each batch holds
32 slots of 9216 bytes, allocated once (576 KiB for both): N6's MTU must
stay under 9200, a larger packet is dropped. Parked packets go through the
same batch when their tunnel comes back.

Measured below: 23 % more packets per second one direction at a time, 15 to
20 % with both at once (BENCHMARKS.md), for about 1 µs more per round trip
(the kernel's batch path, and a downlink packet sent only once the device
is empty).

## Modes

`mode` chooses how the UPF learns that packets wait:

| Mode | How | Cost |
|---|---|---|
| `event` (default) | epoll wakes the UPF when N3 or N6 is readable | a wake-up per burst; no CPU when idle |
| `poll` | no epoll: N3 and N6 are read at every turn of the worker, which never sleeps | a whole core, always |
| `hybrid` | epoll wakes it; it then polls until 256 polls in a row find nothing | a core while traffic flows; none when idle |

Put a polling UPF in a pool of its own (`[[runtime.pools]]`, pinned with
`cores`): its worker never sleeps, so it would starve nothing but would hold
a core away from other EOs of the pool.

## Measurements

`scripts/upf-modes.sh` (the `forwarding` example, release build): a UPF in a
one-worker pool pinned to a core; a test gNB on UDP and a datagram socket
pair standing for the TUN device; 84-byte IPv4 packets.

- *round trip*: an uplink G-PDU to N6, then a downlink packet back to the
  gNB, 20 000 times back to back;
- *sparse*: the same with 1 ms between round trips, 2 000 times (the UPF
  idle in between);
- *burst*: three threads sending uplink G-PDUs as fast as they can
  (600 000 in all), the packets per second reaching N6;
- *idle worker CPU*: the UPF's worker over one idle second.

AMD Ryzen 9 5950X, Linux 7.0, 2026-10-20, two runs agreeing within 1 µs
(medians) and 1 %:

| mode | round trip p50 / p99 (µs) | sparse round trip p50 / p99 (µs) | burst (kpps, delivered) | idle worker CPU |
|---|---|---|---|---|
| event | 28.9 / 36.7 | 46.3 / 56.4 | 372, 46 % | 0 % |
| poll | 20.4 / 23.3 | 24.6 / 30.5 | 360, 44 % | 100 % |
| hybrid | 20.5 / 23.8 | 36.0 / 46.1 | 372, 46 % | 0 % |

Reading:

- **Throughput does not depend on the mode**: about 370 000 packets per
  second on one core, bound by system calls: one per packet on N6 (the
  TUN device), one per batch on N3. More needs fewer system calls on N6
  (the TUN device's offloads), more UPFs or cores, or a kernel bypass
  (eBPF), not polling.
- **Polling cuts latency**: 8 µs off a round trip (two wake-ups saved),
  half the latency of sparse traffic, for a core spent when idle.
- **Hybrid** keeps poll's latency while traffic flows and event's idle
  cost; after a 1 ms gap it has gone back to sleep, so sparse traffic pays
  part of the wake-up again.

So `event` stays the default; `hybrid` suits latency-sensitive traffic on
a shared machine, `poll` a dedicated core.

### By number of UEs

`forwarding ues` (also run by `scripts/upf-modes.sh`), `event` mode: one
PDU session per UE, the packets sent to the UEs in turn. *Uplink*: three
gNB threads send G-PDUs as fast as they can; *downlink*: three threads
write IP packets on N6 as fast as they can; the packets per second that
come out of the UPF, and their IP bytes in Mbit/s. Two runs agree within
5 %; the first:

| UEs | packet | round trip p50 / p99 (µs) | uplink kpps, Mbit/s | downlink kpps, Mbit/s |
|---|---|---|---|---|
| 1 | 84 B | 28.3 / 32.7 | 374, 251 | 228, 153 |
| 1 | 1400 B | 29.9 / 34.7 | 345, 3866 | 211, 2358 |
| 2 | 84 B | 28.6 / 32.8 | 371, 249 | 226, 152 |
| 2 | 1400 B | 29.7 / 33.8 | 339, 3801 | 211, 2362 |
| 10 | 84 B | 28.4 / 33.4 | 374, 251 | 227, 153 |
| 10 | 1400 B | 29.7 / 33.9 | 340, 3807 | 211, 2367 |
| 100 | 84 B | 28.8 / 33.0 | 374, 251 | 227, 152 |
| 100 | 1400 B | 29.8 / 34.1 | 344, 3855 | 211, 2358 |
| 1000 | 84 B | 28.7 / 32.9 | 372, 250 | 227, 153 |
| 1000 | 1400 B | 29.7 / 33.7 | 335, 3756 | 209, 2346 |

Reading:

- **The number of UEs costs nothing**: sessions are found by hash (TEID
  uplink, UE address downlink).
- **Packets, not bytes, are the limit**: 1400-byte packets go nearly as
  many per second as 84-byte ones, hence 2.3 to 3.9 Gbit/s against 150 to
  250 Mbit/s.
- **Downlink is slower** (about 225 kpps): the UPF's UDP send on loopback
  also runs the receiving side of the stack on its core (94 % busy, every
  packet delivered). On a real network card, the send is cheaper.
- **One UPF uses one core**: it is one EO, which runs on one core at a
  time, so more cores in its pool would stay idle. More would take several
  UPF instances. The whole core's capacity, with both directions at once,
  is in [`BENCHMARKS.md`](../../BENCHMARKS.md).
