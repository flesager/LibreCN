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

AMD Ryzen 9 5950X, Linux 7.0, 2026-10-17, two runs agreeing within 1 µs
(medians) and 1 %:

| mode | round trip p50 / p99 (µs) | sparse round trip p50 / p99 (µs) | burst (kpps, delivered) | idle worker CPU |
|---|---|---|---|---|
| event | 27.5 / 34.2 | 45.1 / 61.3 | 314, 41 % | 0 % |
| poll | 18.6 / 21.7 | 23.6 / 31.6 | 314, 40 % | 100 % |
| hybrid | 18.8 / 22.5 | 34.6 / 44.9 | 311, 41 % | 0 % |

Reading:

- **Throughput does not depend on the mode**: about 310 000 packets per
  second on one core, bound by the two system calls per packet (receive,
  send). More needs batching (`recvmmsg`/`sendmmsg`), more UPFs or cores,
  or a kernel bypass, not polling.
- **Polling cuts latency**: 9 µs off a round trip (two wake-ups saved),
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
| 1 | 84 B | 27.6 / 32.9 | 303, 203 | 187, 126 |
| 1 | 1400 B | 28.3 / 36.2 | 274, 3065 | 176, 1969 |
| 2 | 84 B | 27.5 / 37.4 | 285, 192 | 187, 126 |
| 2 | 1400 B | 28.5 / 36.3 | 276, 3095 | 178, 1994 |
| 10 | 84 B | 28.0 / 38.5 | 291, 196 | 185, 124 |
| 10 | 1400 B | 28.6 / 40.6 | 272, 3048 | 172, 1929 |
| 100 | 84 B | 27.7 / 37.9 | 299, 201 | 186, 125 |
| 100 | 1400 B | 27.8 / 36.2 | 254, 2842 | 175, 1960 |
| 1000 | 84 B | 27.9 / 39.0 | 283, 190 | 182, 123 |
| 1000 | 1400 B | 28.8 / 41.6 | 261, 2923 | 178, 1997 |

Reading:

- **The number of UEs costs nothing**: sessions are found by hash (TEID
  uplink, UE address downlink).
- **Packets, not bytes, are the limit**: 1400-byte packets go nearly as
  many per second as 84-byte ones, hence 2 to 3 Gbit/s against 120 to 200
  Mbit/s.
- **Downlink is slower** (about 180 kpps): the UPF's UDP send on loopback
  also runs the receiving side of the stack on its core (94 % busy, every
  packet delivered). On a real network card, the send is cheaper.
- **One UPF uses one core**: it is one EO, which runs on one core at a
  time, so more cores in its pool would stay idle. More would take several
  UPF instances. The whole core's capacity, with both directions at once,
  is in [`BENCHMARKS.md`](../../BENCHMARKS.md).
