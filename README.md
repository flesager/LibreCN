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
| [`deploy/`](deploy/) | LibreCN's image (`ghcr.io/flesager/librecn`, amd64 and arm64) and its `compose.yaml` and `.env`, for real gNBs |
| [`docker/`](docker/) | an image with LibreCN and UERANSIM, for the interop checks |
| [`scripts/`](scripts/) | slow checks (`stress.sh`), interop checks with UERANSIM (`ueransim/`) |

Dependencies point down only: `scheduler` ← `common` ← `oam`, `nf/*` ← `app`.

## Run

With Docker, for real gNBs: download [`deploy/compose.yaml`](deploy/compose.yaml) and [`deploy/env.example`](deploy/env.example) (as `.env`), then `docker compose up -d`; see [`deploy/README.md`](deploy/README.md).

From source: Rust 1.88 or newer. Linux only (epoll, eventfd, CPU affinity).

```
cargo run --release -p librecn -- --config app/librecn.example.toml
```

Then open http://127.0.0.1:8080. Logs are JSON lines on stdout. Stop with Ctrl-C or SIGTERM.

To run it on a real machine, with a gNB on N2 and N3 and a data network on N6: [`RUNNING.md`](RUNNING.md).

## Test

```
cargo test --workspace     # about 7 s: run before every commit
scripts/stress.sh          # repeated and CPU-restricted runs, loom
scripts/ueransim/ng-setup.sh       # interop with UERANSIM's gNB (UERANSIM next to this repo)
scripts/ueransim/registration.sh   # UERANSIM gNB + UE: registration, PDU session, deregistration
scripts/ueransim/user-plane.sh     # in Docker: UERANSIM's UE pings through the UPF
scripts/ueransim/scale.sh          # UERANSIM at scale: GNBS gNBs x UES UEs (default 4 x 250)
scripts/ueransim/overload.sh       # UERANSIM: Overload Start and Stop from a full AMF
scripts/ueransim/mobility.sh       # in Docker: an idle UE moves to another gNB and tracking area
scripts/ueransim/demo.sh           # in Docker: gNBs and UEs kept busy, to watch in the GUI (Ctrl-C to stop)
scripts/free-ran-ue/check.sh       # in Docker: free-ran-ue (free5GC's codecs) pings through LibreCN, then 1000 UEs
scripts/load.sh                    # capacity on 1 to 4 cores, control and user plane (BENCHMARKS.md)
scripts/upf-modes.sh               # the UPF alone: modes, number of UEs, packet size (BENCHMARKS.md)
scripts/upf-bench.sh               # in Docker: the user plane as deployed, 1 or 2 cores, with and without the fast path
```

## Capacity

What LibreCN carries for the number of cores given to it.

**User plane** (`scripts/upf-bench.sh`): a gNB, LibreCN and a data network on three network namespaces, linked directly (veth); the most each way without loss, 1400-byte packets, with the UPF's fast path (in brackets: without it, the UPF alone).

| Cores for LibreCN | Uplink alone | Downlink alone | Both ways at once, each way |
|---|---|---|---|
| 1 | 6.7 Gbit/s (2.1) | 2.5 Gbit/s (2.4) | 1.8 Gbit/s (1.1) |
| 2, one pool | 6.6 Gbit/s (3.0) | 4.2 Gbit/s (4.2) | 2.8 Gbit/s (1.8) |
| **2: the UPF on one, the rest on the other** | 6.5 Gbit/s (3.0) | 4.2 Gbit/s (4.3) | **3.2 Gbit/s (1.8)** |

- **The fast path** (XDP, on when the kernel allows it; the Docker deployment gives it the rights) carries the uplink in the kernel: three times the UPF alone on one core. The downlink is still the UPF's (its fast path is next), so it is the limit both ways.
- The limit is in packets, not bytes: with 84-byte packets, the same packets per second (700 000 uplink in the fast path, 250 000 downlink per core), so a fifteenth of the Gbit/s.
- The kernel's receive work for LibreCN's interfaces runs on LibreCN's cores (with two cores split, on the one that is not the UPF's): on a real host, the network card's interrupts go there.

**Registrations** (`scripts/load.sh`: every function in one pool, test gNBs on loopback).

| Cores for LibreCN | Registrations per second, no traffic | Both at once (registrations and user plane, no fast path) |
|---|---|---|
| 1 | 9 000 | shared: 4 000 registrations/s and 0.6 Gbit/s each way |
| **2** | 9 000 | **8 700 registrations/s and 1.2 Gbit/s each way** |
| 3 | 9 000 | as with 2 cores |
| 4 | 9 000 | as with 2 cores |

**Choosing the number of cores**:
- **2 cores give the full capacity**, both planes loaded at once: the AMF uses one core, the UPF the other, and the other functions fit in what is left. For the user plane, the UPF in a pool of its own is a little better (above).
- **1 core** suits a lab or a light load: the two planes share it. With 500 Mbit/s each way of user traffic, 5 400 registrations per second remain.
- **3 or 4 cores add no capacity**, only headroom. Each function is one instance, and an instance uses one core at a time: a larger pool for the UPF would stay idle.

**Memory (RAM)**: plan **20 MiB, plus 5 KiB per UE** (each UE registered with its PDU session).

| UEs | Memory measured, at its peak | Memory to plan |
|---|---|---|
| 1 000 | 19 MiB | 25 MiB |
| 10 000 | 46 MiB | 70 MiB |
| 20 000 | 57 to 95 MiB | 120 MiB |
| 40 000 | 99 to 136 MiB | 220 MiB |

- The tables for the UEs are reserved at start for the configured capacity (the AMF's `max_ues`, the UPF's `max_sessions`, the UDM's subscribers). Their memory is used as the UEs register: plan for the capacity you configure.
- With nothing configured beyond the example file (1024 UEs), LibreCN uses 14 MiB at rest.
- Each core adds about 1 MiB. User-plane traffic adds nothing: its buffers are fixed.
- Beyond 40 000 UEs, the rule is extrapolated: for example, 100 000 UEs would need about 510 MiB.

**What the figures mean**:
- **A registration** is a UE's whole Initial Registration with its PDU session: 5G AKA, NAS security (NIA2, NEA2), Initial Context Setup, PDU Session Establishment.
- **Latency**:
  - With no load, a registration with its PDU session takes 0.22 ms.
  - At full load, a registration takes about 28 ms: that is queueing, with 256 registrations in flight.
  - A packet's round trip through the UPF, uplink then downlink, takes 29 µs, or 20 µs in polling mode.
- **Scale**: tested with 40 000 UEs and their sessions. The number of UEs does not change the user-plane throughput. With UERANSIM, 2000 UEs over 8 gNBs.

Measured on an AMD Ryzen 9 5950X (registrations 2026-10-17, user plane 2026-10-20), with logs at `error`. A real network card will give somewhat different figures: veth copies each packet for XDP, a card's driver does not.

Conditions, every figure (p50 and p99 latencies, busy time, small packets, UPF modes, UERANSIM runs) and how to measure them again: [`BENCHMARKS.md`](BENCHMARKS.md).
