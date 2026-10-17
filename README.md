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

To run it on a real machine, with a gNB on N2 and N3 and a data network on N6: [`RUNNING.md`](RUNNING.md).

## Test

```
cargo test --workspace     # about 7 s: run before every commit
scripts/stress.sh          # repeated and CPU-restricted runs, loom
scripts/ueransim/ng-setup.sh       # interop with UERANSIM's gNB (UERANSIM next to this repo)
scripts/ueransim/registration.sh   # UERANSIM gNB + UE: registration, PDU session, deregistration
scripts/ueransim/user-plane.sh     # in Docker: UERANSIM's UE pings through the UPF
scripts/ueransim/scale.sh          # UERANSIM at scale: GNBS gNBs x UES UEs (default 4 x 250)
scripts/load.sh                    # capacity on 1 to 4 cores, control and user plane (BENCHMARKS.md)
scripts/upf-modes.sh               # the UPF alone: modes, number of UEs, packet size (BENCHMARKS.md)
```

## Capacity

What LibreCN carries for the number of cores given to it. Every network function (AMF, AUSF, UDM, SMF, UPF) and the OAM run in one pool of cores.

| Cores for LibreCN | Registrations per second, no traffic | User plane, no registrations | Both at once |
|---|---|---|---|
| 1 | 9 000 | 1.1 Gbit/s each way | shared: 3 400 registrations/s and 0.6 Gbit/s each way |
| **2** | 9 000 | 1.1 Gbit/s each way | **8 900 registrations/s and 1.0 Gbit/s each way** |
| 3 | 9 000 | 1.1 Gbit/s each way | as with 2 cores |
| 4 | 9 000 | 1.1 Gbit/s each way | as with 2 cores |

**Choosing the number of cores**:
- **2 cores give the full capacity**, both planes loaded at once: the AMF uses one core, the UPF the other, and the other functions fit in what is left.
- **1 core** suits a lab or a light load: the two planes share it. With 500 Mbit/s each way of user traffic, 5 000 registrations per second remain.
- **3 or 4 cores add no capacity**, only headroom. Each function is one instance, and an instance uses one core at a time.

**What the figures mean**:
- **A registration** is a UE's whole Initial Registration with its PDU session: 5G AKA, NAS security (NIA2, NEA2), Initial Context Setup, PDU Session Establishment.
- **The user plane** is 1400-byte packets, uplink and downlink at the same time.
  - The UPF's limit is in packets: about 100 000 per second each way, whatever their size.
  - With 84-byte packets, that is 70 Mbit/s each way.
- **Latency**:
  - With no load, a registration with its PDU session takes 0.22 ms.
  - At full load, a registration takes about 28 ms: that is queueing, with 256 registrations in flight.
  - A packet's round trip through the UPF, uplink then downlink, takes 28 µs, or 19 µs in polling mode.
- **Scale**: tested with 40 000 UEs and their sessions. The number of UEs does not change the user-plane throughput. With UERANSIM, 2000 UEs over 8 gNBs.

Measured on an AMD Ryzen 9 5950X (2026-10-17), over loopback, with logs at `error`. A real network card and TUN device will give somewhat different figures.

Conditions, every figure (p50 and p99 latencies, busy time, small packets, UPF modes, UERANSIM runs) and how to measure them again: [`BENCHMARKS.md`](BENCHMARKS.md).
