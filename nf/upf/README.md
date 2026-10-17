# UPF (`librecn-upf`)

The user plane function, one EO: N4 sessions from the SMF (EO messages),
GTP-U on N3 (UDP 2152 of `n3`), a TUN device on N6 (`n6`). See the crate
documentation for the parameters.

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
