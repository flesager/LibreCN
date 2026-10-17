# LibreCN
Open implementation of a 5GCore in Rust based on a pattern reactor scheduler and design for embedded target in barematal and docker

## Layout

| Directory | Content |
|---|---|
| [`scheduler/`](scheduler/) | reactor scheduler hosting run-to-completion execution objects (EOs); design in [`scheduler/DESIGN.md`](scheduler/DESIGN.md) |
| [`common/`](common/) | what every EO shares: message-id families, header flags, logging, alarms |
| [`oam/`](oam/) | non-3GPP EOs: log, alarms, control, web API and GUI; see [`oam/README.md`](oam/README.md) |
| [`proto/`](proto/) | 3GPP protocol codecs: NGAP (ASN.1 APER), see [`proto/ngap/README.md`](proto/ngap/README.md) |
| [`nf/`](nf/) | 3GPP network functions, one crate each: the AMF (N2 so far, NAS stubbed) |
| [`app/`](app/) | the `librecn` binary: configuration, runtime, OAM, NFs |
| [`scripts/`](scripts/) | slow checks (`stress.sh`) |

Dependencies point down only: `scheduler` ← `common` ← `oam`, `nf/*` ← `app`.

## Run

Rust 1.88 or newer. Linux only (epoll, eventfd, CPU affinity).

```
cargo run --release -p librecn -- --config app/librecn.example.toml
```

Then open http://127.0.0.1:8080. Logs are JSON lines on stdout. Stop with Ctrl-C or SIGTERM.

## Test

```
cargo test --workspace     # about 2 s: run before every commit
scripts/stress.sh          # repeated and CPU-restricted runs, loom
scripts/ueransim/ng-setup.sh       # interop with UERANSIM's gNB (UERANSIM next to this repo)
scripts/ueransim/registration.sh   # UERANSIM gNB + UE: UE signalling over N2
```
