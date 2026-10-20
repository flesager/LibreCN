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

To run it on a real machine, with a gNB on N2 and N3 and a data network on N6: [`RUNNING.md`](RUNNING.md). Integrating it with a commercial or open RAN and real UEs: [`INTEGRATION.md`](INTEGRATION.md).

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
| 1 | 6.2 Gbit/s (2.0) | 5.6 Gbit/s (2.3) | 2.9 Gbit/s (0.9 to 1.1) |
| **2** | 6.4 Gbit/s (3.0) | 5.6 Gbit/s (4.1) | **5.5 Gbit/s (1.7)** |

- **The fast path** (XDP and TC, on when the kernel allows it; the Docker deployment gives it the rights) carries the traffic in the kernel, both ways: about three times the UPF alone per core, the UPF's own core left free.
- **Two cores**: with the fast path, the network card's interrupts on both (the pools do not matter: the kernel does the work); without it, the UPF in a pool of its own and the interrupts on the other core. One direction alone takes one core's worth with one receive queue; a card with several, and several gNBs, spreads it.
- The limit is in packets, not bytes: with 84-byte packets, about the same packets per second (500 000 to 680 000 in the fast path), so a seventeenth of the Gbit/s.
- The fast path carries the PCF's QoS flows too (classification costs 3 to 4 % at worst). The host's own traffic goes through the UPF, and so would TCP downloads if the card aggregates them (GRO): the Docker deployment turns GRO off while the core runs (`nf/upf/README.md`).

**Registrations** (`scripts/load.sh`: every function in one pool, test gNBs on loopback).

| Cores for LibreCN | Registrations per second, no traffic | Both at once (registrations and user plane, no fast path) |
|---|---|---|
| 1 | 9 000 | shared: 4 000 registrations/s and 0.6 Gbit/s each way |
| **2** | 9 000 | **8 700 registrations/s and 1.2 Gbit/s each way** |
| 3 | 9 000 | as with 2 cores |
| 4 | 9 000 | as with 2 cores |

**Choosing the number of cores**:
- **2 cores give the full capacity.** How to lay them out depends on the UPF's fast path (on by default, and in the Docker deployment):
  - **With the fast path**: every function in one pool on both cores, and the network card's interrupts on both cores (where its receive work, and so the fast path, runs; `irqbalance` spreads them). The kernel carries the traffic (5.5 Gbit/s each way), the AMF takes a core when registrations come.
  - **Without it** (no rights for it): the UPF in a pool of its own on one core, the other functions and the card's interrupts on the other: 1.7 Gbit/s each way, against 1.1 to 1.3 with the UPF sharing its core.
- **1 core** suits a lab or a light load: everything shares it. With the fast path, 2.9 Gbit/s each way, or 9 000 registrations per second; without it, 0.9 to 1.1 Gbit/s, and with 500 Mbit/s each way of user traffic, 5 400 registrations per second remain.
- **3 or 4 cores add no capacity to the functions**: each is one instance, and an instance uses one core at a time (a larger pool for the UPF would stay idle). With the fast path, more cores can take more of the kernel's receive work, if the card has several receive queues and the traffic comes from several gNBs (not measured: the benchmark's links have one queue each).
- Registrations and traffic at once were measured without the fast path (`scripts/load.sh`); with it, the UPF's core is free for the AMF.

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
