# LibreCN benchmarks

The figures behind the capacity and memory tables of the [README](README.md#capacity), and how to measure them again. Measured on 2026-10-17.

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
| 1 | 8 930/s, p50 28 ms, p99 30 ms, busy 100 % | 97 kpps, 1.08 Gbit/s, busy 100 % | 3 400/s, p50 75 ms, p99 81 ms | 55 kpps, 0.62 Gbit/s, busy 100 % |
| 2 | 9 330/s, p50 27 ms, p99 29 ms, busy 55 % | 96 kpps, 1.07 Gbit/s, busy 50 % | 8 890/s, p50 29 ms, p99 31 ms | 90 kpps, 1.00 Gbit/s, busy 100 % |
| 3 | 9 270/s, p50 27 ms, p99 35 ms, busy 38 % | 96 kpps, 1.07 Gbit/s, busy 33 % | 8 890/s, p50 29 ms, p99 31 ms | 89 kpps, 1.00 Gbit/s, busy 69 % |
| 4 | 9 010/s, p50 28 ms, p99 31 ms, busy 29 % | 97 kpps, 1.09 Gbit/s, busy 25 % | 8 740/s, p50 29 ms, p99 32 ms | 84 kpps, 0.94 Gbit/s, busy 52 % |

### 84-byte packets, flat out (`load ... up size=84`)

| Cores | User plane alone, each way | Both: registrations | Both: user plane, each way |
|---|---|---|---|
| 1 | 103 kpps, 69 Mbit/s | 3 690/s, p99 78 ms | 56 kpps, 38 Mbit/s |
| 2 | 104 kpps, 70 Mbit/s | 9 220/s, p99 30 ms | 98 kpps, 66 Mbit/s |
| 3 | 104 kpps, 70 Mbit/s | 9 100/s, p99 29 ms | 98 kpps, 66 Mbit/s |
| 4 | 104 kpps, 70 Mbit/s | 8 960/s, p99 32 ms | 91 kpps, 61 Mbit/s |

### 500 Mbit/s each way of 1400-byte packets (`load ... up=500`), and one UE at a time

| Cores | Registrations with that traffic | Busy | One registration with no load, p50 / p99 |
|---|---|---|---|
| 1 | 4 950/s, p50 51 ms, p99 59 ms | 100 % | 0.22 / 0.29 ms |
| 2 | 9 150/s, p50 27 ms, p99 30 ms | 77 % | 0.22 / 0.29 ms |
| 3 | 9 180/s, p50 28 ms, p99 30 ms | 54 % | 0.25 / 0.30 ms |
| 4 | 9 070/s, p50 28 ms, p99 31 ms | 41 % | 0.25 / 0.31 ms |

### Reading the tables

- **Two cores are enough; more add nothing.** The AMF and the UPF are each one EO, and an EO runs on one core at a time.
  - With two cores, each gets its own; the AUSF, UDM, SMF and OAM EOs fit in what is left (under 10 % of a core).
  - A third and fourth core stay idle (69 % and 52 % busy). Beyond, it would take several AMF or UPF instances.
- **One core is enough for either plane alone**, but they share it when both are loaded: 500 Mbit/s each way halves the registrations.
- **The UPF is bound by packets, not bytes**: about 100 000 packets per second each way, whatever their size.
- **Latency under load is queueing.** 256 registrations in flight at about 9 000 per second means about 28 ms each, by Little's law. One registration with its session takes 0.22 ms.
- **Where the AMF's time goes**: about 110 µs per registration.
  - About 40 % is the kernel's SCTP: 8.3 µs to send and receive one message on loopback, about 4 µs on the AMF's side, for each of its 11 NGAP messages per UE.
  - The NGAP codec takes about 1 µs per message (decoding an Uplink NAS Transport: 0.5 µs, encoding a Downlink NAS Transport: 0.8 µs).
  - NAS security takes under 1 µs per message (NIA2 and NEA2 on 64 bytes: 0.19 µs, one KDF: 0.15 µs).
  - The rest is NAS, the UE table, and the 8 NF-to-NF messages.

## 2. The UPF alone (`scripts/upf-modes.sh`)

The UPF on its own core, with a test gNB and data network, and no other NF. Its sessions are set up by N4 messages from the test. Details are in [`nf/upf/README.md`](nf/upf/README.md).

**Round trip**: an uplink packet to N6, then a downlink packet back to the gNB, back to back.

| Mode | Round trip p50 / p99 | Sparse round trip p50 / p99 (1 ms gaps) | Idle CPU |
|---|---|---|---|
| `event` (default) | 27.5 / 34.2 µs | 45.1 / 61.3 µs | 0 % |
| `poll` | 18.6 / 21.7 µs | 23.6 / 31.6 µs | 100 % of a core |
| `hybrid` | 18.8 / 22.5 µs | 34.6 / 44.9 µs | 0 % |

**One direction at a time**, `event` mode: three threads send as fast as they can, the UEs in turn.

| UEs | Packet | Uplink | Downlink |
|---|---|---|---|
| 1 | 84 B | 303 kpps, 203 Mbit/s | 187 kpps, 126 Mbit/s |
| 1 | 1400 B | 274 kpps, 3.1 Gbit/s | 176 kpps, 2.0 Gbit/s |
| 1000 | 84 B | 283 kpps, 190 Mbit/s | 182 kpps, 123 Mbit/s |
| 1000 | 1400 B | 261 kpps, 2.9 Gbit/s | 178 kpps, 2.0 Gbit/s |

With 2, 10 and 100 UEs, the figures are the same within 5 %.

**Reading the tables**:
- **Polling cuts latency, not throughput**: 9 µs less per round trip, and half the latency for sparse traffic, but it spends a core even when idle.
- **The number of UEs does not matter**: a session is found by a hash lookup.
- **Downlink is slower than uplink**: on loopback, the UPF's send to the gNB also runs the gNB's receiving side of the kernel, on the UPF's core.
- **Both directions at once share the UPF's core**, hence the 100 kpps each way of part 1.
- More would need batched system calls (`recvmmsg`/`sendmmsg`), several UPFs, or a kernel bypass.

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

**What each EO preallocates** at creation (mailbox, buffer pools, timers, I/O, flight recorder), with 1024 UEs configured: 7.1 MiB in all.

| EO | AMF | SMF | log | UPF | UDM | PCF | AUSF | alarms |
|---|---|---|---|---|---|---|---|---|
| MiB | 2.43 | 1.36 | 1.32 | 0.85 | 0.36 | 0.36 | 0.33 | 0.10 |

The UPF's share includes a class of 512-byte buffers for N4Establish, which carries the session's PCF rule. The rule's PCC rules are kept per session only when a session has some, so a session without them costs about 40 bytes more in the UPF and the SMF.

The AMF's share grows with `max_ues` (one NAS timer per UE): 5.11 MiB for 40 000.

**Reading the tables**:
- **2 to 4 KiB per UE**, all functions together, measured from the resident memory at its peak. The plan in the README, 20 MiB plus 5 KiB per UE, covers every run.
- **The capacity is reserved at start, but used as the UEs come.** With 40 000 UEs configured, 79 MiB of heap is reserved at start but only 21 MiB is resident. Its pages count once UEs fill them.
- **The spread between runs comes from the allocator.** The NGAP codec's temporary allocations, for up to 256 registrations in flight, interleave with each UE's long-lived context. Freed space is then kept in the allocator, more or less, depending on the timing.
- **Each core adds about 1 MiB** (15.4 MiB ready on 1 core, 18.2 MiB on 4, for 20 000 UEs): a worker's stack and its timer wheel.
- **User-plane traffic adds nothing** but its sessions: its buffers are fixed.
- **The `librecn` binary at rest**, with `app/librecn.example.toml` (1024 UEs, the default pools): 14 MiB resident.
- The load generator runs in the same process; its own share is under 1 % (a few bytes per UE).

## 5. Found by these benchmarks

- **The AMF looked up UE contexts with a scan**: now an index, about 40 % more registrations.
- **A lost gNB flooded the SMF**: its thousands of UEs going idle at once filled the SMF's mailbox. The AMF now sends these notices 64 at most unacknowledged.
- **The scheduler starved the UPF's N4 requests under traffic**: its socket events kept its other messages waiting. On 2 cores, registrations fell to 4 700/s with p99 1.4 s, and 3 or 4 cores stalled. Sockets are now re-armed at the end of the turn ([`scheduler/DESIGN.md`](scheduler/DESIGN.md) §7 and §11).
