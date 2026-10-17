# LibreCN
Open implementation of a 5GCore in Rust based on a pattern reactor scheduler and design for embedded target in barematal and docker

## Layout

| Directory | Content |
|---|---|
| [`scheduler/`](scheduler/) | reactor scheduler hosting run-to-completion execution objects (EOs); design in [`scheduler/DESIGN.md`](scheduler/DESIGN.md) |
| [`common/`](common/) | what every EO shares: message-id families, header flags, logging, alarms |
| [`oam/`](oam/) | non-3GPP EOs: log, alarms, control, web API and GUI; see [`oam/README.md`](oam/README.md) |
| [`proto/`](proto/) | 3GPP protocols: NGAP (ASN.1 APER), NAS, security algorithms, SBI messages; see [`proto/README.md`](proto/README.md) |
| [`nf/`](nf/) | 3GPP network functions, one crate each: AMF, AUSF, UDM, SMF, UPF; see [`nf/README.md`](nf/README.md) |
| [`app/`](app/) | the `librecn` binary: configuration, runtime, OAM, NFs |
| [`docker/`](docker/) | an image with LibreCN and UERANSIM, for the user-plane check |
| [`scripts/`](scripts/) | slow checks (`stress.sh`), interop checks with UERANSIM (`ueransim/`) |

Dependencies point down only: `scheduler` ← `common` ← `oam`, `nf/*` ← `app`.

## Run

Rust 1.88 or newer. Linux only (epoll, eventfd, CPU affinity).

```
cargo run --release -p librecn -- --config app/librecn.example.toml
```

Then open http://127.0.0.1:8080. Logs are JSON lines on stdout. Stop with Ctrl-C or SIGTERM.

## Test

```
cargo test --workspace     # about 7 s: run before every commit
scripts/stress.sh          # repeated and CPU-restricted runs, loom
scripts/ueransim/ng-setup.sh       # interop with UERANSIM's gNB (UERANSIM next to this repo)
scripts/ueransim/registration.sh   # UERANSIM gNB + UE: registration, PDU session, deregistration
scripts/ueransim/user-plane.sh     # in Docker: UERANSIM's UE pings through the UPF
scripts/ueransim/scale.sh          # UERANSIM at scale: GNBS gNBs x UES UEs (default 4 x 250)
scripts/load.sh                    # the whole core on 1 to 4 cores, control and user plane (below)
scripts/upf-modes.sh               # the UPF measured: per mode, per number of UEs and packet size
```

## Benchmarks

Measured on 2026-10-17. Re-run them with the scripts named in each part.

**Machine**:
- AMD Ryzen 9 5950X (16 cores, 32 threads), Linux 7.0, Ubuntu 26.04;
- release build (`panic = "abort"`), kernel SCTP and UDP over loopback.

**Pinning**:
- LibreCN on cores 2 to 5, the load generators on cores 6 to 14;
- all distinct physical cores, no SMT sibling shared.

Logs are at `error` level (`[oam] log_level`). At `info`, every NAS message is a log line, and the log EO would be what is measured.

### The whole core on 1 to 4 cores (`scripts/load.sh`)

**Setup**: every EO of LibreCN (AMF, AUSF, UDM, SMF, UPF, log, alarms) runs in one pool pinned to 1, 2, 3 or 4 cores.

**Control plane**: one registration is a full Initial Registration followed by its PDU session:
- 5G AKA through the AUSF and UDM (Milenage), Security Mode with NIA2 and NEA2;
- Registration Accept in Initial Context Setup, then Registration Complete;
- PDU Session Establishment through the SMF and UPF, up to the gNB's PDU Session Resource Setup.

That is 11 NGAP messages on N2 and 8 NF-to-NF messages per UE.

**Load generator** (`app/examples/load.rs`):
- 4 test gNBs, each a thread with its own SCTP association, register 5000 UEs each, 64 at a time: 20 000 UEs, 256 in flight;
- each simulated UE runs the real crypto;
- a UE is timed from its Initial UE Message to the PDU Session Resource Setup Request.

**User plane**:
- 1400-byte packets, uplink and downlink at the same time, over the 20 000 sessions just set up;
- the UPF has N3 on UDP and, for N6, a datagram socket pair standing for the TUN device;
- either as fast as the UPF takes them ("flat out"), or 500 Mbit/s each way.

**Each alone, then both at once** (a second batch of 20 000 UEs registers while the traffic runs):

| Cores | Registrations alone | User plane alone, flat out | Both: registrations + user plane flat out |
|---|---|---|---|
| 1 | 8 930/s, p99 30 ms (busy 100 %) | 1.08 Gbit/s each way, 97 kpps (busy 100 %) | 3 400/s, p99 81 ms + 0.62 Gbit/s each way (busy 100 %) |
| 2 | 9 330/s, p99 29 ms (busy 55 %) | 1.07 Gbit/s each way (busy 50 %) | **8 890/s, p99 31 ms + 1.00 Gbit/s each way** (busy 100 %) |
| 3 | 9 270/s, p99 35 ms (busy 38 %) | 1.07 Gbit/s each way (busy 33 %) | 8 890/s, p99 31 ms + 1.00 Gbit/s each way (busy 69 %) |
| 4 | 9 010/s, p99 31 ms (busy 29 %) | 1.09 Gbit/s each way (busy 25 %) | 8 740/s, p99 32 ms + 0.94 Gbit/s each way (busy 52 %) |

**At a fixed traffic of 500 Mbit/s each way**, and one UE at a time (no load):

| Cores | Registrations with 500 Mbit/s each way | Pool busy | One registration alone, p50 / p99 |
|---|---|---|---|
| 1 | 4 950/s, p50 51 ms, p99 59 ms | 100 % | 0.22 / 0.29 ms |
| 2 | 9 150/s, p50 27 ms, p99 30 ms | 77 % | 0.22 / 0.29 ms |
| 3 | 9 180/s, p50 28 ms, p99 30 ms | 54 % | 0.25 / 0.30 ms |
| 4 | 9 070/s, p50 28 ms, p99 31 ms | 41 % | 0.25 / 0.31 ms |

"Busy" is the pool's busy time, as a share of all its cores. Two runs agree within 5 %. Every run registered all its UEs.

**Reading the tables**:
- **Two cores are enough; more add nothing.** The AMF and the UPF are each one EO, and an EO runs on one core at a time.
  - With two cores, each gets its own; the AUSF, UDM, SMF and OAM EOs fit in what is left (under 10 % of a core).
  - A third and fourth core stay idle (69 % and 52 % busy): beyond, it would take several AMF or UPF instances.
- **One core is enough for either plane alone**, about 9 000 registrations/s or 1 Gbit/s each way, but they share it when both are loaded: 500 Mbit/s each way halves the registrations.
- **Where the AMF's time goes**: about 110 µs per registration.
  - About 40 % is the kernel's SCTP: 8.3 µs to send and receive one message on loopback, about 4 µs on the AMF's side, for each of its 11 NGAP messages per UE.
  - The NGAP codec takes about 1 µs per message (decoding an Uplink NAS Transport: 0.5 µs, encoding a Downlink NAS Transport: 0.8 µs).
  - NAS security takes under 1 µs per message (NIA2 and NEA2 on 64 bytes: 0.19 µs, one KDF: 0.15 µs).
  - The rest is NAS, the UE table, and the 8 NF-to-NF messages.
- **Latency under load is queueing.** 256 registrations in flight at about 9 000 per second means about 28 ms each, by Little's law. Unloaded, one registration with its session takes 0.22 ms.
- **This benchmark found a scheduler bug, now fixed.** Under traffic, the UPF's socket events kept its other messages (the SMF's N4 requests) waiting: on 2 cores, registrations fell to 4 700/s with p99 1.4 s, and 3 or 4 cores stalled. See [`scheduler/DESIGN.md`](scheduler/DESIGN.md) §7 and §11.

### User plane: the UPF (`scripts/upf-modes.sh`, details in [`nf/upf/README.md`](nf/upf/README.md))

**Setup**:
- one UPF, on one core; one PDU session per UE;
- a test gNB on UDP (GTP-U), and a datagram socket pair standing for the TUN device on N6;
- the UPF has no SMF or AMF around it here: its sessions are set up by N4 messages from the test.

**What is measured**:
- *round trip*: one uplink packet to N6, then one downlink packet back to the gNB, the UEs in turn; this is the latency through the UPF, both ways;
- *uplink* and *downlink*: three threads send packets as fast as they can, the UEs in turn; the figure is what comes out of the UPF, in packets per second (kpps, thousands) and in Mbit/s of IP packets;
- packets of 84 bytes (small, like TCP acknowledgements or voice) and of 1400 bytes (large, like downloads).

**By number of UEs**, in `event` mode (the default):

| UEs | Packet | Round trip p50 / p99 | Uplink | Downlink |
|---|---|---|---|---|
| 1 | 84 B | 27.6 / 32.9 µs | 303 kpps, 203 Mbit/s | 187 kpps, 126 Mbit/s |
| 1 | 1400 B | 28.3 / 36.2 µs | 274 kpps, 3.1 Gbit/s | 176 kpps, 2.0 Gbit/s |
| 2 | 84 B | 27.5 / 37.4 µs | 285 kpps, 192 Mbit/s | 187 kpps, 126 Mbit/s |
| 2 | 1400 B | 28.5 / 36.3 µs | 276 kpps, 3.1 Gbit/s | 178 kpps, 2.0 Gbit/s |
| 10 | 84 B | 28.0 / 38.5 µs | 291 kpps, 196 Mbit/s | 185 kpps, 124 Mbit/s |
| 10 | 1400 B | 28.6 / 40.6 µs | 272 kpps, 3.0 Gbit/s | 172 kpps, 1.9 Gbit/s |
| 100 | 84 B | 27.7 / 37.9 µs | 299 kpps, 201 Mbit/s | 186 kpps, 125 Mbit/s |
| 100 | 1400 B | 27.8 / 36.2 µs | 254 kpps, 2.8 Gbit/s | 175 kpps, 2.0 Gbit/s |
| 1000 | 84 B | 27.9 / 39.0 µs | 283 kpps, 190 Mbit/s | 182 kpps, 123 Mbit/s |
| 1000 | 1400 B | 28.8 / 41.6 µs | 261 kpps, 2.9 Gbit/s | 178 kpps, 2.0 Gbit/s |

The figures are for all the UEs together: with 1000 UEs, each one gets a thousandth of them. A second run agrees within 5 %.

**Reading the table**:
- **The number of UEs does not matter**: a session is found by a hash lookup, so 1 or 1000 UEs cost the same.
- **The UPF is bound by packets, not bytes**: two system calls per packet. So small packets give about 200 Mbit/s, large ones 2 to 3 Gbit/s.
- **Downlink is slower than uplink**: on loopback, the UPF's send to the gNB also runs the gNB's receiving side of the kernel, on the UPF's core (busy 94 %, every packet delivered).
- More would need batched system calls (`recvmmsg`/`sendmmsg`), several UPFs, or a kernel bypass.

**By number of cores** in the UPF's pool (`UPF_CORES`), `event` mode, 1 and 1000 UEs:

| Pool cores | UEs | Round trip p50 / p99 (1400 B) | Uplink, 84 B / 1400 B | Downlink, 84 B / 1400 B |
|---|---|---|---|---|
| 1 | 1 | 27.8 / 32.7 µs | 296 kpps, 199 Mbit/s / 276 kpps, 3.1 Gbit/s | 188 kpps, 126 Mbit/s / 179 kpps, 2.0 Gbit/s |
| 1 | 1000 | 28.3 / 38.2 µs | 294 kpps, 197 Mbit/s / 267 kpps, 3.0 Gbit/s | 187 kpps, 125 Mbit/s / 175 kpps, 2.0 Gbit/s |
| 2 | 1 | 30.7 / 37.1 µs | 296 kpps, 199 Mbit/s / 273 kpps, 3.1 Gbit/s | 186 kpps, 125 Mbit/s / 177 kpps, 2.0 Gbit/s |
| 2 | 1000 | 30.9 / 36.2 µs | 295 kpps, 198 Mbit/s / 273 kpps, 3.1 Gbit/s | 185 kpps, 124 Mbit/s / 178 kpps, 2.0 Gbit/s |
| 3 | 1 | 31.0 / 35.9 µs | 296 kpps, 199 Mbit/s / 275 kpps, 3.1 Gbit/s | 187 kpps, 125 Mbit/s / 178 kpps, 2.0 Gbit/s |
| 3 | 1000 | 31.1 / 41.2 µs | 296 kpps, 199 Mbit/s / 266 kpps, 3.0 Gbit/s | 186 kpps, 125 Mbit/s / 178 kpps, 2.0 Gbit/s |
| 4 | 1 | 30.9 / 35.0 µs | 296 kpps, 199 Mbit/s / 271 kpps, 3.0 Gbit/s | 186 kpps, 125 Mbit/s / 178 kpps, 2.0 Gbit/s |
| 4 | 1000 | 31.3 / 39.7 µs | 293 kpps, 197 Mbit/s / 265 kpps, 3.0 Gbit/s | 186 kpps, 125 Mbit/s / 177 kpps, 2.0 Gbit/s |

The same with 2, 10 and 100 UEs, within 3 %. **The cores do not add throughput**: the UPF is one EO, which runs on one core at a time, so the other cores of its pool stay idle. With more than one core, a packet sometimes wakes another worker than the last one, which costs about 3 µs of round trip. Using more cores would take several UPF instances, each with its share of the UEs; that is not done yet.

**By mode and pool size**, one UE, 84-byte packets:

| UPF mode, pool cores | Round trip p50 / p99 | Sparse round trip p50 / p99 (1 ms gaps) | Uplink | Idle CPU |
|---|---|---|---|---|
| event, 1 core | 27.5 / 34.2 µs | 45.1 / 61.3 µs | 314 kpps | 0 % |
| event, 2 cores | 30.0 / 34.1 µs | 47.1 / 55.9 µs | 301 kpps | 0 % |
| event, 3 cores | 30.3 / 35.1 µs | 47.4 / 57.8 µs | 303 kpps | 0 % |
| event, 4 cores | 30.3 / 36.0 µs | 47.7 / 63.0 µs | 302 kpps | 0 % |
| poll, 1 core | 18.6 / 21.7 µs | 23.6 / 31.6 µs | 314 kpps | 100 % |
| hybrid, 1 core | 18.8 / 22.5 µs | 34.6 / 44.9 µs | 311 kpps | 0 % |

**Reading the table**:
- **More cores do not help one UPF**: it is one EO, which runs on one core at a time.
- **Polling cuts latency, not throughput**: 9 µs less per round trip, half the latency for sparse traffic, but a core spent even when idle.

### Scale with UERANSIM (`scripts/ueransim/scale.sh`)

Each UERANSIM UE does a real registration (5G AKA, NAS security) and gets its PDU session. The gNBs are UERANSIM gNBs on distinct loopback addresses. LibreCN runs in a 2-worker pool, logging at `info`.

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
- **The load generator above takes LibreCN to 20 000 registered UEs.** `max_ues`, 1024 by default, is the AMF's parameter for its UE table (the load runs set it to 20 000). The UDM's subscribers can be a range (`count`).
- **UERANSIM is patched** (`scripts/ueransim/patches/`) for two bugs of v3.3.0's gNB:
  - 0001: the gNB routed no security-protected initial message to an AMF, so no UE could come back from idle;
  - 0002: a data race on the gNB's map of simulated radio links crashed it (segfault) when hundreds of UEs attached at once.
