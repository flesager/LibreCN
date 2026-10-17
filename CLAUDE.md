# LibreCN: working notes for Claude

LibreCN is an open 5G core in Rust, built on a reactor scheduler of run-to-completion execution objects (EOs). It must run on Linux, baremetal or in Docker, with the kernel network stack (baremetal means "no container", not `no_std`). Each network function (NF) is hosted in an EO.

The detailed design is in `scheduler/DESIGN.md` and `oam/README.md`. This file records the owner's decisions and how they want to work. When the code and this file disagree, ask.

## Principles (apply to every change)

- **SOLID, mostly**: pragmatic, not dogmatic. One responsibility per module; EOs depend only on `Context`, never on the platform running them.
- **Light is Right** (Colin Chapman): minimal code, dependencies, memory and CPU. No allocation on the hot path.
- **"Perfection is achieved not when there is nothing more to add, but when there is nothing left to take away"** (Saint-Exupéry). Prefer removing or merging to adding. Every type, option, counter and dependency must earn its place. When adding a feature, look for what it lets you delete, and record removals and deliberate omissions in `scheduler/DESIGN.md` §11.

## How to work with the owner

- **Discuss before implementing** anything structural: propose, list numbered questions, wait for answers (the owner replies by number). Once the owner says "go ahead", proceed without further check-ins.
- **Trunk-based on the central repo, local branch for the work**:
  - The central repo has only `main`, no feature branches.
  - Locally, `main` stays clean, a mirror of the central `main` that the owner compares against. Never commit on local `main`.
  - Work is committed on a local working branch, and only when the owner says so.
  - The owner publishes: pulls local `main`, rebases the working branch onto it, and pushes to the central `main`. Never push.
- **Strong and fast tests**: `cargo test --workspace` must stay at a few seconds and catch regressions. Slow checks (repeated release runs, `taskset` runs on 1 and 2 CPUs, loom) go in `scripts/stress.sh`, never in the default test run.
- Run `cargo fmt --all`, `cargo clippy --workspace --all-targets` (zero warnings) and the tests before reporting work as done.
- The owner asked for test coverage of load balancing, overload and the execution-time API from the start. Keep them.

## Layout

```
scheduler/  reactor, EOs, test bed (librecn-scheduler)
common/     shared conventions: msg-id families, flags, Logger, Alarms (librecn-common)
oam/        non-3GPP EOs: log, alarm, control, web server + GUI (librecn-oam)
nf/         3GPP NFs, one crate each (none yet; the `demo` kind in app/ stands in)
app/        the `librecn` binary: config, runtime, OAM, NF factories
scripts/    stress.sh
```

Later: deployment and Docker directories. Dependencies point down only: `scheduler` ← `common` ← `oam`, `nf/*` ← `app`.

## Scheduler decisions

- **Reactor pattern**: pools of worker threads (1–64, optionally pinned to cores). An EO runs on any worker of its pool, never on two at once.
- **Everything is a message**: data from other EOs, timers, I/O readiness, polls, lifecycle. The header carries the rest (`msg_id`, `from`, `to`, `corr_id`, `flags`, `reserved`).
- **Mailboxes**: bounded MPSC queues, with a system lane drained before the data lane.
- **Memory**: preallocated and prefaulted when an EO is created, never on a worker. Each mailbox has its own buffer pool, so memory needs are tracked per EO.
  - Buffer size classes fall back to larger ones when empty. `high_water` and `fallbacks` show which class is undersized.
  - The owner wants to know the maximum number of messages that were in each queue (high-water marks) and to collect it in real time.
- **Zero-copy bodies**: fixed-layout structs (`zerocopy`) written into the receiver's buffer. Multi-process messaging will be done another way, later.
- **Message ids**: `family << 16 | message`. Families are grouped by procedure (registration, PDU session, ...), not by NF. The scheduler reserves only `0x0000_xxxx`; families are declared in `common::msg`.
- **Instrumentation**: per-message processing time, queue depths and high-water marks, buffer use, creation and allocation cost, per-worker busy/idle time.
- **Failures**: a panic must always tell the root cause (EO, message, flight-recorder history, stats, backtrace). Default policy: report then abort; `Isolate` is optional.
- **Timers**: 1 ms tick by default.
- **Poll and hybrid modes** (for the UPF): a per-EO `ctx.set_polling(bool)`, not pool modes.
- **EOs can be added at runtime**: `Runtime::add`, `ctx.spawn`.
- **Dependencies**: `rustix`, `zerocopy`, `thiserror`; tests only: `proptest`, `loom`, `criterion`. No Tokio, no async in the scheduler.
- **Deliberately not done**: zero-copy buffer hand-off between EOs (see DESIGN.md §11), multiple instances of one NF (maybe later for the UPF).

## OAM decisions

- **Non-3GPP EOs run in the `oam` pool**: `log`, `alarm`, `control`, `server`. Metrics have no EO: counters are read in place and exported by the server (Prometheus `/metrics`).
- **Control EO**: starts NFs from deployments, stops EOs (OAM EOs are protected), supervises (critical alarm when an EO faults).
- **Server EO**: web API (REST + JSON with **serde**) and a GUI in **plain HTML/JS embedded in the binary**, with no framework and no build step.
- **Boot**: the system starts with OAM only. NFs come from the TOML config file (`[[nf]]`) or are loaded through the server (`POST /api/nfs`, TOML or JSON), both through the control EO.
- **Logs are JSON lines**. Levels: `VIP` / `FATAL` / `ERROR` / `INFO` / `DEBUG`; `VIP` is never filtered and is meant for rare lines such as "NF started successfully".
  - EOs log through a convenient call (`log!(self.log, ctx, Info, ...)`): a message to the log EO if it runs, otherwise a plain print.
  - Tests check logs with a capture helper that sits next to the log EO, not inside it (`LogSink::capture()`).
- **Alarms are required**: raise, update and clear with X.733 severities; every change is logged.

## 3GPP protocols (N2 / NGAP first)

- **Codecs live in `proto/`**, one crate per protocol: `proto/ngap` first; later NAS, PFCP, GTP-U.
- **NGAP** (TS 38.413, ASN.1 APER):
  - Rust code is generated with `rasn-compiler` for the `rasn` runtime. The `.asn` sources and the generated code are both checked in.
  - `oxirush-ngap` serves only as a cross-check.
  - **Release**: start with Rel-15 (v15.13.0, `38413-fd0`) to validate the process, then move to Rel-19 (the code roughly doubles).
  - **ASN.1 source**: the 3GPP FTP archive, `https://www.3gpp.org/ftp/Specs/archive/38_series/38.413/` (needs a browser user agent). Version letters: f=15, g=16, h=17, i=18, j=19. The zip holds only the spec as `.doc`/`.docx`; the ASN.1 is between `-- ASN1START` / `-- ASN1STOP`, with smart quotes and non-breaking spaces to normalise.
- **`rasn` 0.29 has APER bugs** (four, found by the deep round trip, cross-checked with asn1tools; see `proto/ngap/README.md`).
  - They are fixed in a fork: branch `librecn-aper-fixes` from tag `rasn-v0.29.0`, on the owner's personal GitHub, used through `[patch.crates-io]`. The owner has done the same before for the C asn1 compiler.
  - The fork's commit is also kept as `proto/ngap/rasn/0001-*.patch`, so it can always be recreated.
  - Reporting them upstream is deferred until the project is more mature and legitimate.
- **hampi rejected**: it refuses a whole message on an unknown IE, and decodes an enumeration's extension value as value 0.
- **Isolation**: ASN.1 types stay in the codec crate and the N2 code. The NF logic sees only small domain types, so the codec can be replaced.
- **Zero allocation is the owner's preference**: the first working version may allocate (`rasn`); the owner will explain the target zero-allocation design later.
- **Robustness**: decoding external input must never panic (one bad packet would abort the core). Fuzz the decoders and answer bad NGAP with Error Indication.
- **N2 placement**: inside the AMF EO (modules of the AMF crate), not a separate EO.
- **SCTP**: kernel SCTP, through `libc` (accepted). On Docker it needs the host's `sctp` module.
- Interop and test peer: UERANSIM, plus Wireshark-checked captures.
- NAS (TS 24.501) is not ASN.1: a separate hand-written codec, discussed later.

## Commands

```
cargo test --workspace                                    # before every commit
scripts/stress.sh [RUNS]                                  # slow checks
cargo bench -p librecn-scheduler
cargo run --release -p librecn -- --config app/librecn.example.toml   # GUI on http://127.0.0.1:8080
```
