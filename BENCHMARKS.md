# LibreCN benchmarks

The figures behind the capacity and memory tables of the [README](README.md#capacity), and how to measure them again. Measured on 2026-10-17; parts 1 and 2 again on 2026-10-20, with N3 in batches (`recvmmsg`/`sendmmsg`).

## Conditions

- **Machine**: AMD Ryzen 9 5950X (16 cores, 32 threads), Linux 7.0, Ubuntu 26.04.
- **Build**: release (`panic = "abort"`).
- **Network**: kernel SCTP (N2) and UDP (N3) over loopback. N6 is a datagram socket pair standing for the TUN device.
- **Pinning**: LibreCN on cores 2 to 5, the load generators on cores 6 to 14, all distinct physical cores (no SMT sibling shared).
- **Logs**: at `error` level (`[oam] log_level`). At `info`, every NAS message is a log line, and the log EO would be what is measured.

## 1. The whole core on 1 to 4 cores (`scripts/load.sh`)

Every EO of LibreCN (AMF, AUSF, UDM, SMF, UPF, log, alarms) runs in **one pool** pinned to 1, 2, 3 or 4 cores.

**Control plane**: one registration is a full Initial Registration followed by its PDU session:
- 5G AKA through the AUSF and UDM (Milenage), Security Mode with NIA2 and NEA2;
- Registration Accept in Initial Context Setup, then Registration Complete;
- PDU Session Establishment through the SMF and UPF, up to the gNB's PDU Session Resource Setup.

That is 11 NGAP messages on N2 and 8 NF-to-NF messages per UE.

**Load generator** (`app/examples/load.rs`):
- 4 test gNBs, each a thread with its own SCTP association, register 5000 UEs each, 64 at a time: 20 000 UEs, 256 in flight;
- each simulated UE runs the real crypto;
- a UE is timed from its Initial UE Message to the PDU Session Resource Setup Request.

**User plane**: uplink and downlink at the same time, over the 20 000 sessions just set up. The rate is either as fast as the UPF takes them ("flat out"), or fixed at 500 Mbit/s each way.

**Phases**: registrations alone, then the user plane alone, then both at once (a second batch of 20 000 UEs registers while the traffic runs).

Two runs agree within 5 %. Every run registered all its UEs. "Busy" is the pool's busy time, as a share of all its cores.

### 1400-byte packets, flat out

| Cores | Registrations alone | User plane alone, each way | Both: registrations | Both: user plane, each way |
|---|---|---|---|---|
| 1 | 8 670/s, p50 29 ms, p99 32 ms, busy 100 % | 116 kpps, 1.30 Gbit/s, busy 100 % | 4 020/s, p50 62 ms, p99 71 ms | 56 kpps, 0.62 Gbit/s, busy 100 % |
| 2 | 8 980/s, p50 28 ms, p99 29 ms, busy 55 % | 115 kpps, 1.29 Gbit/s, busy 50 % | 8 740/s, p50 29 ms, p99 32 ms | 107 kpps, 1.20 Gbit/s, busy 100 % |
| 3 | 9 010/s, p50 28 ms, p99 29 ms, busy 38 % | 116 kpps, 1.30 Gbit/s, busy 33 % | 8 510/s, p50 29 ms, p99 30 ms | 110 kpps, 1.23 Gbit/s, busy 69 % |
| 4 | 8 930/s, p50 29 ms, p99 30 ms, busy 29 % | 116 kpps, 1.30 Gbit/s, busy 25 % | 8 500/s, p50 30 ms, p99 31 ms | 103 kpps, 1.16 Gbit/s, busy 52 % |

### 84-byte packets, flat out (`load ... up size=84`)

| Cores | User plane alone, each way | Both: registrations | Both: user plane, each way |
|---|---|---|---|
| 1 | 122 kpps, 82 Mbit/s | 4 210/s, p99 67 ms | 58 kpps, 39 Mbit/s |
| 2 | 120 kpps, 81 Mbit/s | 8 820/s, p99 32 ms | 111 kpps, 75 Mbit/s |
| 3 | 121 kpps, 81 Mbit/s | 8 560/s, p99 31 ms | 117 kpps, 79 Mbit/s |
| 4 | 121 kpps, 81 Mbit/s | 8 660/s, p99 31 ms | 110 kpps, 74 Mbit/s |

### 500 Mbit/s each way of 1400-byte packets (`load ... up=500`), and one UE at a time

| Cores | Registrations with that traffic | Busy | One registration with no load, p50 / p99 |
|---|---|---|---|
| 1 | 5 390/s, p50 47 ms, p99 55 ms | 100 % | 0.22 / 0.25 ms |
| 2 | 9 090/s, p50 28 ms, p99 30 ms | 74 % | 0.23 / 0.29 ms |
| 3 | 8 830/s, p50 29 ms, p99 31 ms | 52 % | 0.26 / 0.31 ms |
| 4 | 8 820/s, p50 29 ms, p99 30 ms | 40 % | 0.27 / 0.31 ms |

### Reading the tables

- **Two cores are enough; more add nothing.** The AMF and the UPF are each one EO, and an EO runs on one core at a time.
  - With two cores, each gets its own; the AUSF, UDM, SMF and OAM EOs fit in what is left (under 10 % of a core).
  - A third and fourth core stay idle (69 % and 52 % busy). Beyond, it would take several AMF or UPF instances.
- **One core is enough for either plane alone**, but they share it when both are loaded: 500 Mbit/s each way takes about 40 % of the registrations.
- **The UPF is bound by packets, not bytes**: about 115 000 packets per second each way, whatever their size.
- **Latency under load is queueing.** 256 registrations in flight at about 9 000 per second means about 28 ms each, by Little's law. One registration with its session takes 0.22 ms.
- **Where the AMF's time goes**: about 110 µs per registration.
  - About 40 % is the kernel's SCTP: 8.3 µs to send and receive one message on loopback, about 4 µs on the AMF's side, for each of its 11 NGAP messages per UE.
  - The NGAP codec takes about 1 µs per message (decoding an Uplink NAS Transport: 0.5 µs, encoding a Downlink NAS Transport: 0.8 µs).
  - NAS security takes under 1 µs per message (NIA2 and NEA2 on 64 bytes: 0.19 µs, one KDF: 0.15 µs).
  - The rest is NAS, the UE table, and the 8 NF-to-NF messages.

### The user plane as deployed (`scripts/upf-bench.sh`)

The `librecn` binary on its configuration, a gNB with 100 UEs and a data network (`load.rs`'s `gnb` and `dn` roles), each in its own network namespace (containers), linked directly by veth pairs: N3 from the gNB to LibreCN, N6 from LibreCN to the data network. The UEs register over N2, then traffic flows; the control plane is idle (registrations: above).

- **Deployments**: LibreCN on 1 core; on 2 cores in one pool; on 2 cores, the UPF's pool on one (core 3) and the rest on the other (core 2).
- **Paths**: the UPF's fast path (the kernel carries both ways: XDP uplink, TC downlink), or its fallback (the UPF alone).
- **The kernel's receive work** for LibreCN's two interfaces (veth's NAPI, one thread each) runs on LibreCN's cores, in real time as softirq runs ahead of threads: on its core; on both; split, on the core that is not the UPF's (second table: on both). veth has that thread only with XDP, so the fallback runs with the program attached but carrying nothing (tens of ns per packet). The gNB's and data network's receive work runs on their own cores; six senders on each side, short bursts (4 packets) when paced.
- **The figure** is RFC 2544's throughput: the highest rate each way with at most 1 % lost (veth's drops when its ring is full counted as lost), found by bisection (2 s per step) below what the senders give flat out (2.5 Mpps). Flooded, the fallback carries less (on one core, uplink: 100 kpps against 175): the kernel's receive work starves the UPF; the fast path does not (it drops what it cannot take before working on it).
- **Two runs**: one figure where they agree within 10 %, the range where they do not (the fallback's, where the UPF and the kernel's threads share cores). LibreCN's cores busy: the first run's.

AMD Ryzen 9 5950X, Linux 7.0, 2026-10-20:

| Deployment | Path | Traffic | Packet | Uplink | Downlink | LibreCN's cores busy |
|---|---|---|---|---|---|---|
| 1 core | fast path | uplink | 84 B | 679 kpps, 0.46 Gbit/s | - | 100 % |
| 1 core | fast path | uplink | 1400 B | 557 kpps, 6.2 Gbit/s | - | 100 % |
| 1 core | fast path | downlink | 84 B | - | 544 kpps, 0.37 Gbit/s | 100 % |
| 1 core | fast path | downlink | 1400 B | - | 502 kpps, 5.6 Gbit/s | 100 % |
| 1 core | fast path | both ways | 84 B | 284 kpps, 0.19 Gbit/s | 284 kpps, 0.19 Gbit/s | 100 % |
| 1 core | fast path | both ways | 1400 B | 261 kpps, 2.9 Gbit/s | 262 kpps, 2.9 Gbit/s | 100 % |
| 1 core | fallback | uplink | 84 B | 174 kpps, 0.12 Gbit/s | - | 84 % |
| 1 core | fallback | uplink | 1400 B | 176 kpps, 2.0 Gbit/s | - | 93 % |
| 1 core | fallback | downlink | 84 B | - | 148–189 kpps, 0.10–0.13 Gbit/s | 78 % |
| 1 core | fallback | downlink | 1400 B | - | 204 kpps, 2.3 Gbit/s | 100 % |
| 1 core | fallback | both ways | 84 B | 86–110 kpps, 0.06–0.07 Gbit/s | 86–107 kpps, 0.06–0.07 Gbit/s | 92 % |
| 1 core | fallback | both ways | 1400 B | 78–96 kpps, 0.87–1.1 Gbit/s | 78–96 kpps, 0.87–1.1 Gbit/s | 87 % |
| 2 cores, one pool | fast path | uplink | 84 B | 672 kpps, 0.45 Gbit/s | - | 2 %, 100 % |
| 2 cores, one pool | fast path | uplink | 1400 B | 571 kpps, 6.4 Gbit/s | - | 1 %, 100 % |
| 2 cores, one pool | fast path | downlink | 84 B | - | 544 kpps, 0.37 Gbit/s | 5 %, 100 % |
| 2 cores, one pool | fast path | downlink | 1400 B | - | 500 kpps, 5.6 Gbit/s | 2 %, 100 % |
| 2 cores, one pool | fast path | both ways | 84 B | 540 kpps, 0.36 Gbit/s | 540 kpps, 0.36 Gbit/s | 89 %, 100 % |
| 2 cores, one pool | fast path | both ways | 1400 B | 494 kpps, 5.5 Gbit/s | 494 kpps, 5.5 Gbit/s | 94 %, 100 % |
| 2 cores, one pool | fallback | uplink | 84 B | 149–237 kpps, 0.10–0.16 Gbit/s | - | 46 %, 39 % |
| 2 cores, one pool | fallback | uplink | 1400 B | 224 kpps, 2.5 Gbit/s | - | 70 %, 54 % |
| 2 cores, one pool | fallback | downlink | 84 B | - | 230 kpps, 0.15 Gbit/s | 99 %, 7 % |
| 2 cores, one pool | fallback | downlink | 1400 B | - | 212 kpps, 2.4 Gbit/s | 12 %, 98 % |
| 2 cores, one pool | fallback | both ways | 84 B | 107 kpps, 0.07 Gbit/s | 107 kpps, 0.07 Gbit/s | 57 %, 52 % |
| 2 cores, one pool | fallback | both ways | 1400 B | 95–113 kpps, 1.1–1.3 Gbit/s | 95–113 kpps, 1.1–1.3 Gbit/s | 52 %, 79 % |
| 2 cores: UPF, the rest | fast path | uplink | 84 B | 672 kpps, 0.45 Gbit/s | - | 100 %, 2 % |
| 2 cores: UPF, the rest | fast path | uplink | 1400 B | 561 kpps, 6.3 Gbit/s | - | 100 %, 2 % |
| 2 cores: UPF, the rest | fast path | downlink | 84 B | - | 544 kpps, 0.37 Gbit/s | 100 %, 1 % |
| 2 cores: UPF, the rest | fast path | downlink | 1400 B | - | 494 kpps, 5.5 Gbit/s | 100 %, 1 % |
| 2 cores: UPF, the rest | fast path | both ways | 84 B | 293 kpps, 0.20 Gbit/s | 293 kpps, 0.20 Gbit/s | 100 %, 2 % |
| 2 cores: UPF, the rest | fast path | both ways | 1400 B | 250 kpps, 2.8 Gbit/s | 250 kpps, 2.8 Gbit/s | 100 %, 4 % |
| 2 cores: UPF, the rest | fallback | uplink | 84 B | 300 kpps, 0.20 Gbit/s | - | 47 %, 100 % |
| 2 cores: UPF, the rest | fallback | uplink | 1400 B | 265 kpps, 3.0 Gbit/s | - | 46 %, 100 % |
| 2 cores: UPF, the rest | fallback | downlink | 84 B | - | 388 kpps, 0.26 Gbit/s | 66 %, 100 % |
| 2 cores: UPF, the rest | fallback | downlink | 1400 B | - | 367 kpps, 4.1 Gbit/s | 71 %, 100 % |
| 2 cores: UPF, the rest | fallback | both ways | 84 B | 146–174 kpps, 0.10–0.12 Gbit/s | 145–169 kpps, 0.10–0.11 Gbit/s | 57 %, 99 % |
| 2 cores: UPF, the rest | fallback | both ways | 1400 B | 155 kpps, 1.7 Gbit/s | 155 kpps, 1.7 Gbit/s | 58 %, 100 % |

The split deployment with the kernel's receive work on both cores, two runs:

| Deployment | Path | Traffic | Packet | Uplink | Downlink | LibreCN's cores busy |
|---|---|---|---|---|---|---|
| 2 cores: UPF, the rest, receive on both | fast path | uplink | 84 B | 669 kpps, 0.45 Gbit/s | - | 4 %, 100 % |
| 2 cores: UPF, the rest, receive on both | fast path | uplink | 1400 B | 571 kpps, 6.4 Gbit/s | - | 2 %, 100 % |
| 2 cores: UPF, the rest, receive on both | fast path | downlink | 84 B | - | 546 kpps, 0.37 Gbit/s | 0 %, 100 % |
| 2 cores: UPF, the rest, receive on both | fast path | downlink | 1400 B | - | 501 kpps, 5.6 Gbit/s | 100 %, 4 % |
| 2 cores: UPF, the rest, receive on both | fast path | both ways | 84 B | 529 kpps, 0.36 Gbit/s | 529 kpps, 0.36 Gbit/s | 92 %, 100 % |
| 2 cores: UPF, the rest, receive on both | fast path | both ways | 1400 B | 488 kpps, 5.5 Gbit/s | 486 kpps, 5.4 Gbit/s | 100 %, 99 % |
| 2 cores: UPF, the rest, receive on both | fallback | uplink | 84 B | 259–302 kpps, 0.17–0.20 Gbit/s | - | 48 %, 99 % |
| 2 cores: UPF, the rest, receive on both | fallback | uplink | 1400 B | 178 kpps, 2.0 Gbit/s | - | 1 %, 94 % |
| 2 cores: UPF, the rest, receive on both | fallback | downlink | 84 B | - | 191–337 kpps, 0.13–0.23 Gbit/s | 60 %, 98 % |
| 2 cores: UPF, the rest, receive on both | fallback | downlink | 1400 B | - | 176–197 kpps, 2.0–2.2 Gbit/s | 2 %, 96 % |
| 2 cores: UPF, the rest, receive on both | fallback | both ways | 84 B | 108 kpps, 0.07 Gbit/s | 108 kpps, 0.07 Gbit/s | 25 %, 81 % |
| 2 cores: UPF, the rest, receive on both | fallback | both ways | 1400 B | 95 kpps, 1.1 Gbit/s | 95 kpps, 1.1 Gbit/s | 22 %, 87 % |

**Reading the tables**:
- **The fast path carries about three times the UPF alone, per core**: on one core, 6.2 Gbit/s uplink (2.0 alone), 5.6 downlink (2.3), 2.9 each way both at once (0.9 to 1.1), at 500 to 680 kpps whatever the size. It is the kernel's work: the UPF's core stays free.
- **With the fast path, two cores take the kernel's receive work on both**: 5.5 Gbit/s each way both at once, in one pool or split (the pools no longer matter: the UPF does nothing). One direction alone stays at one core's worth: each interface has one receive queue here (a network card with several, and several gNBs, spreads it).
- **The UPF alone wants its core to itself**: split, the kernel's receive work on the other core, it carries 1.7 Gbit/s each way both at once (3.0 uplink, 4.1 downlink); with the receive work on its core too (one pool, or split on both), less (1.1 to 1.3).
- **Packets, not bytes, are the limit**: 84-byte packets go at about the same packets per second.
- **N3's socket buffer is 8 MiB**: with the default (208 KiB, 90 datagrams of 1400 bytes), bursts were lost while the UPF was away, 2 to 4 % at any rate.
- veth copies each packet for XDP (its packets have no room in front for it); a network card's driver does not, so the fast path's figures are a floor.

## 2. The UPF alone (`scripts/upf-modes.sh`)

The UPF on its own core, with a test gNB and data network, and no other NF. Its sessions are set up by N4 messages from the test. Details are in [`nf/upf/README.md`](nf/upf/README.md).

**Round trip**: an uplink packet to N6, then a downlink packet back to the gNB, back to back.

| Mode | Round trip p50 / p99 | Sparse round trip p50 / p99 (1 ms gaps) | Idle CPU |
|---|---|---|---|
| `event` (default) | 28.9 / 36.7 µs | 46.3 / 56.4 µs | 0 % |
| `poll` | 20.4 / 23.3 µs | 24.6 / 30.5 µs | 100 % of a core |
| `hybrid` | 20.5 / 23.8 µs | 36.0 / 46.1 µs | 0 % |

**One direction at a time**, `event` mode: three threads send as fast as they can, the UEs in turn.

| UEs | Packet | Uplink | Downlink |
|---|---|---|---|
| 1 | 84 B | 374 kpps, 251 Mbit/s | 228 kpps, 153 Mbit/s |
| 1 | 1400 B | 345 kpps, 3.9 Gbit/s | 211 kpps, 2.4 Gbit/s |
| 1000 | 84 B | 372 kpps, 250 Mbit/s | 227 kpps, 153 Mbit/s |
| 1000 | 1400 B | 335 kpps, 3.8 Gbit/s | 209 kpps, 2.3 Gbit/s |

With 2, 10 and 100 UEs, the figures are the same within 5 %.

**Reading the tables**:
- **Polling cuts latency, not throughput**: 8 µs less per round trip, and half the latency for sparse traffic, but it spends a core even when idle.
- **The number of UEs does not matter**: a session is found by a hash lookup.
- **Downlink is slower than uplink**: on loopback, the UPF's send to the gNB also runs the gNB's receiving side of the kernel, on the UPF's core.
- **Both directions at once share the UPF's core**, hence the 115 kpps each way of part 1.
- **N3 goes in batches** of up to 32 datagrams per system call (`recvmmsg`/`sendmmsg`): 23 % more packets than one system call each (303 to 374 kpps uplink, 187 to 228 kpps downlink; 97 to 116 kpps each way in part 1), for about 1 µs more per round trip. The TUN device (N6) still takes one system call per packet.
- More needs fewer passes through the kernel per packet: the fast path below (eBPF/XDP).

## 3. Scale with UERANSIM (`scripts/ueransim/scale.sh`)

Each UERANSIM UE does a real registration (5G AKA, NAS security) and gets its PDU session. The gNBs are UERANSIM gNBs on distinct loopback addresses. LibreCN runs on 2 cores, logging at `info`.

| gNBs × UEs | Registered | PDU sessions | Time |
|---|---|---|---|
| 1 × 1 (`registration.sh`) | 1 / 1 | 1 / 1 | — |
| 2 × 50 | 100 / 100 | 100 / 100 | 3.5 s |
| 4 × 250 | 1000 / 1000 (5 runs out of 5) | 1000 / 1000 | 3.3 to 3.7 s |
| 8 × 250 | 2000 / 2000 | 2000 / 2000 | 4.1 s |

Notes on these runs:
- **Beyond 2000 UEs, UERANSIM gives way on this machine, not LibreCN**, which logged no error.
  - At 8 × 500, fewer than half of the UEs reach the core: cell selection fails ("no SI") and the UAC checks time out on the UE side.
  - At 16 × 500, the UE processes die creating threads: 8000 UEs need more than the 37 121 tasks this session allows.
  - A UERANSIM gNB also refuses more than 1024 UEs.
- **The load generator of part 1 takes LibreCN to 20 000 UEs** (40 000 with the second batch). `max_ues`, 1024 by default, is the AMF's parameter for its UE table. The UDM's subscribers can be a range (`count`).
- **UERANSIM is patched** (`scripts/ueransim/patches/`) for two bugs of v3.3.0's gNB:
  - 0001: the gNB routed no security-protected initial message to an AMF, so no UE could come back from idle;
  - 0002: a data race on the gNB's map of simulated radio links crashed it (segfault) when hundreds of UEs attached at once;
  - 0003: the UE could not read AT_KDF, so no EAP-AKA' authentication worked; 0004: the gNB could not acknowledge an AMF Configuration Update (neither used by these runs).

## 4. Memory (`scripts/load.sh`, the `memory` lines)

The load generator of part 1 reads the process's memory at each stage:
- `VmRSS`: the resident memory, the RAM really used;
- `VmHWM`: its peak;
- glibc's `mallinfo2`: the heap bytes allocated and not freed.

The AMF's `max_ues`, the UPF's `max_sessions` and the UDM's subscribers are set to the number of UEs. Every UE is registered with its PDU session.

| UEs | Ready, before any UE: resident (heap) | After the registrations: resident (heap) | Peak |
|---|---|---|---|
| 1 000 | 12.3 MiB (13.0) | 19.0 MiB (14.1) | 19.0 MiB |
| 10 000 | 14.3 MiB (28.0) | 44.1 MiB (36.5) | 45.5 MiB |
| 20 000 | 16.4 MiB (45.1) | 56.1 to 85.3 MiB (50.7 to 60.1) | 56.6 to 94.9 MiB |
| 40 000 | 20.8 MiB (79.2) | 95.6 to 127.1 MiB (91.0 to 113.2) | 98.9 to 136.3 MiB |

The 20 000 row is eleven runs on 1 to 4 cores, some with one malloc arena (`MALLOC_ARENA_MAX=1`). The 40 000 row is five runs, two of them with user-plane traffic.

**What each EO preallocates** at creation (mailbox, buffer pools, timers, I/O, flight recorder), with 1024 UEs configured: 7.2 MiB in all.

| EO | AMF | SMF | log | UPF | UDM | PCF | AUSF | alarms |
|---|---|---|---|---|---|---|---|---|
| MiB | 2.45 | 1.38 | 1.32 | 0.86 | 0.36 | 0.36 | 0.33 | 0.10 |

The UPF's share includes a class of 512-byte buffers for N4Establish, which carries the session's PCF rule, and two 8 KiB buffers for the web API (its traffic per gNB). The rule's PCC rules are kept per session only when a session has some, so a session without them costs about 40 bytes more in the UPF and the SMF.

The AMF's share grows with `max_ues` (one NAS timer per UE): 5.13 MiB for 40 000.

**Reading the tables**:
- **2 to 4 KiB per UE**, all functions together, measured from the resident memory at its peak. The plan in the README, 20 MiB plus 5 KiB per UE, covers every run.
- **The capacity is reserved at start, but used as the UEs come.** With 40 000 UEs configured, 79 MiB of heap is reserved at start but only 21 MiB is resident. Its pages count once UEs fill them.
- **The spread between runs comes from the allocator.** The NGAP codec's temporary allocations, for up to 256 registrations in flight, interleave with each UE's long-lived context. Freed space is then kept in the allocator, more or less, depending on the timing.
- **Each core adds about 1 MiB** (15.4 MiB ready on 1 core, 18.2 MiB on 4, for 20 000 UEs): a worker's stack and its timer wheel.
- **User-plane traffic adds nothing** but its sessions: its buffers are fixed.
- **The `librecn` binary at rest**, with `app/librecn.example.toml` (1024 UEs, the default pools): 14 MiB resident.
- The load generator runs in the same process; its own share is under 1 % (a few bytes per UE).

## 5. The Docker image (musl)

The image (`deploy/`, Alpine) holds a static binary built against musl,
not glibc; every other figure here is glibc's. musl's memory allocator is
slower with several threads, and the control plane allocates (the NGAP
codec): registrations lose a quarter to a third. The user plane allocates
nothing per packet and does not change. Same machine and cores, both
builds in one container, three alternating rounds:

| | glibc | musl (the image) |
|---|---|---|
| Registrations with their PDU session, 1 core | 8 714/s, p50 29.2 ms | 6 527/s, p50 38.9 ms (−25 %) |
| The same, 2 cores | 9 120/s, p50 27.8 ms | 5 991/s, p50 42.4 ms (−34 %) |
| User plane, 1400-byte packets, 1 core | 1.13 Gbit/s each way | 1.14 Gbit/s each way |

Accepted for a small core, for an image six times smaller (25 MB against
139 MB on Debian with glibc). Built from source with glibc, LibreCN keeps
the figures above.

## 6. Found by these benchmarks

- **The AMF looked up UE contexts with a scan**: now an index, about 40 % more registrations.
- **A lost gNB flooded the SMF**: its thousands of UEs going idle at once filled the SMF's mailbox. The AMF now sends these notices 64 at most unacknowledged.
- **The scheduler starved the UPF's N4 requests under traffic**: its socket events kept its other messages waiting. On 2 cores, registrations fell to 4 700/s with p99 1.4 s, and 3 or 4 cores stalled. Sockets are now re-armed at the end of the turn ([`scheduler/DESIGN.md`](scheduler/DESIGN.md) §7 and §11).
