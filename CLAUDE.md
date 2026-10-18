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
proto/      3GPP protocols: ngap, nas, gtpu, security (algorithms), sbi (NF-to-NF messages)
nf/         3GPP NFs, one crate each: amf, ausf, udm, smf, upf
app/        the `librecn` binary: config, runtime, OAM, NF factories
docker/     one image with LibreCN and UERANSIM, for the user-plane interop check
scripts/    stress.sh, ueransim/ (interop checks)
```

Later: deployment. Dependencies point down only: `scheduler` ← `common` ← `oam`, `proto/*` ← `nf/*` ← `app`.

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

- **Non-3GPP EOs run in the `oam` pool**: `log`, `alarm`, `control`, `server`. Metrics have no EO: counters are read in place and exported by the server (Prometheus `/metrics`). An NF's own counters are named in its `EoConfig` (`counters`) and incremented with `ctx.count(i, n)`; they come out as `librecn_eo_counter_total{eo,pool,counter}` (the UPF's packets and bytes).
- **Control EO**: starts NFs from deployments, stops EOs (OAM EOs are protected), supervises (critical alarm when an EO faults).
- **Server EO**: web API (REST + JSON with **serde**) and a GUI in **plain HTML/JS embedded in the binary**, with no framework and no build step.
- **Boot**: the system starts with OAM only. NFs come from the TOML config file (`[[nf]]`) or are loaded through the server (`POST /api/nfs`, TOML or JSON), both through the control EO.
- **Logs are JSON lines**. Levels: `VIP` / `FATAL` / `ERROR` / `INFO` / `DEBUG`; `VIP` is never filtered and is meant for rare lines such as "NF started successfully".
  - EOs log through a convenient call (`log!(self.log, ctx, Info, ...)`): a message to the log EO if it runs, otherwise a plain print.
  - Tests check logs with a capture helper that sits next to the log EO, not inside it (`LogSink::capture()`).
- **Persistence** (`common::state`, `[oam] state_dir`): an NF keeps its state in `<state_dir>/<name>.json`, read at creation, written as a snapshot (temporary file then rename, mode 0600) when it changed, at most every 5 s, and at stop. The UDM keeps the web API's changes (subscribers added, the configuration's removed) and every SQN, moved 1024 SEQ ahead on restore; the PCF its policies set or removed; the configuration stays the source of what it lists. A database comes later (the owner's choice: a state file first).
- **Alarms are required**: raise, update and clear with X.733 severities; every change is logged.
- **NF resources on the web API**: `/api/nfs/{name}/{path}` is forwarded to the NF as a generic `Manage` request (`common::manage`), so the server never knows an NF. The UDM serves `subscribers` (list without keys, add, remove), with a Subscribers view in the GUI; the PCF `policies`, with a Policies view (edited as JSON); the AMF `gnbs` and `ues`, the SMF `sessions` (the first 64, with a count), shown in the GUI. The GUI has four panels (the owner's design): an Overview drawn as in TS 23.501 §4.2.3 (the service-based NFs on a bus, the reference points below, the NFs LibreCN lacks in grey, gNBs and UEs counted), UEs (one at `#ues/{supi}`), Provisioning, System; split into `index.html`, `app.js`, `style.css`, each view finding its NF by kind (`GET /api/nfs`). Under the Overview's drawing, a card per gNB (the owner's sketch, side by side as they fit): its throughput and each cell's (the UPF counts per cell, by the UE's cell that the AMF gives the SMF with the gNB's tunnel, `UpdateSmContext.cell`, `N4Modify.cell`; a gNB's is the sum of its cells: the owner's choice; a session keeps the cell of its last setup, as UERANSIM has no handover) and its UEs by cell in the three RRC states of TS 38.331 (the owner's choice): connected, inactive, idle (an idle UE keeps its last cell). RRC_INACTIVE: the AMF sends Core Network Assistance Information for RRC INACTIVE and asks for RRC Inactive Transition Reports in Initial Context Setup; UERANSIM's gNB never suspends a UE, so only the NGAP test proves it. `scripts/ueransim/demo.sh` keeps a Docker RAN busy to watch them (the owner's request). Subscribers added this way live in memory only, like their SQNs.

- **PCF** (`nf/pcf`, mandatory: the owner's choice): per-subscriber policies with a `default` entry, in the configuration and on the web API (`/api/nfs/pcf/policies`). AM policy: the UE-AMBR, asked by the AMF at registration; a change reaches the AMF (`AmPolicyUpdate`), which sends UE Context Modification. SM policy, by DNN: the session AMBR, the default flow's 5QI and ARP, and up to 4 PCC rules (extra QoS flows: 5QI, ARP, MBR, GBR, up to 4 packet filters on the remote IPv4 prefix, protocol, remote and local ports, direction). Session policies apply to new sessions only. The SMF and the AMF have no QoS parameters of their own. PCC rule `i` is QoS flow `i + 2` (the default flow is QFI 1, first in the N2 list: UERANSIM's gNB marks all uplink with the first flow); the SMF sends the rules to the UE (QoS rules, flow descriptions), the gNB (QoS flows, GBR information) and the UPF (in `N4Establish`), which classifies the downlink by the filters (first rule in precedence order), checks uplink QFIs, and enforces the session AMBR (non-GBR flows) and each flow's MBR both ways (GCRA, 100 ms of burst; `rate_limited_packets`). Policed, not shaped (the owner's choice: no queue per session), so one TCP flow over a sub-millisecond round trip gets about half its rate; `user-plane.sh` measures the AMBR with UDP.

## 3GPP protocols (N2 / NGAP first)

- **Codecs live in `proto/`**, one crate per protocol: `ngap`, `nas`, `gtpu`, `security`, `sbi`. No PFCP: N4 is EO messages.
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
- **NAS** (TS 24.501) is not ASN.1: a hand-written codec in `proto/nas`, without allocation from the start (views into the PDU when decoding, the caller's buffer when encoding). Only the messages the AMF needs.
- **Security algorithms** in `proto/security`, on RustCrypto (`aes`, `cmac`, `ctr`, `hmac`, `sha2`, accepted by the owner) and `x25519-dalek` 2 (accepted for SUCI Profile A): Milenage, the TS 33.501 key derivations, NIA2/NEA2, ECIES Profile A (checked against TS 33.501 Annex C.4.3's example). Cross-checked against UERANSIM's own implementation (a small C++ program linked to UERANSIM's libraries).
- **SBI**: no HTTP/2 yet. NFs exchange the 3GPP service operations as fixed-layout EO messages (`proto/sbi`), paired by `corr_id`; an HTTP/2 binding is a later decision.
- **N4 is EO messages too, not PFCP** (the owner's choice: the core targets small deployments, not 20 Gbit/s; the owner knows it is not a pure 3GPP core). The messages carry the session's outcome (uplink tunnel, downlink tunnel), not PFCP's rules.

## AMF (first NF, `nf/amf`)

- Built step by step, each step proven against UERANSIM:
  1. N2 setup: SCTP, NG Setup, NG Reset, Error Indication (done).
  2. Initial UE Message, Downlink/Uplink NAS Transport, UE contexts, UE Context Release (done).
  3. Registration (done): 5G AKA with separate **AUSF and UDM EOs** (the owner's choice), NAS security mode (NIA2, NEA2 or NEA0), Registration Accept/Complete, identification, UE-originating deregistration.
  4. PDU sessions, in two steps:
     - 4a (done): control plane. **Initial Context Setup at registration**, carrying the Registration Accept, as Open5GS and free5GC do; UL NAS Transport to the **SMF**, PDU Session Resource Setup, the **UPF** keeping the tunnels. UERANSIM logs "PDU Session establishment is successful".
     - 4b (done): user plane, GTP-U on N3 and a TUN device on N6 (`upf0`, 10.45.0.1/16), event-driven (poll mode later, measured), no allocation per packet. Proof: `scripts/ueransim/user-plane.sh`, a ping through `uesimtun0`. UERANSIM's UE needs root for its TUN (it checks `IsRoot`, not a capability), so this check runs in **Docker** (the owner's choice): the core in one container, UERANSIM's gNB and UE in another.
- **5GMM** (`mm.rs`): pure, one context per UE, acting through an `Effects` trait the EO implements. The list of what is left out is in `nf/README.md`.
- **NAS timers**: T3570 (Identity Request), T3560 (Authentication Request, Security Mode Command), T3550 (Registration Accept, sent again in DL NAS Transport), 6 s, four retransmissions; the fifth expiry aborts (the UE released), except T3550 (the UE taken as registered). One scheduler timer per UE, tagged with its AMF-UE-NGAP-ID (tag 0: the idle sweep).
- **Idle UEs** (`idle.rs`): a registered UE whose N2 connection goes (gNB release, link loss, NG Reset) keeps its 5GMM context and sessions, indexed by 5G-TMSI, the sessions' user plane deactivated at the SMF/UPF. It comes back with a Service Request (Service Accept in Initial Context Setup, sessions of the uplink data status reactivated) or a Registration Request, both checked with the kept keys and read from their ciphered container; a failed check means a Service Reject #9 or a full registration. **T3512** (`t3512_s`, default 3240 s, sent as a GPRS timer 3: 54 min cannot be said, so 1 h) goes in Registration Accept; idle contexts expire after T3512 + 4 min (the mobile reachable timer). A registration from idle without follow-on request is released once complete (UERANSIM always sets follow-on, so this is unit tested only); `user-plane.sh` proves the periodic registration with T3512 30 s (`docker/core.toml`), IPv6 off in the containers (the kernel's router solicitations on `uesimtun0` would wake the idle UE). **Paging** (TS 23.502 §4.2.3.3): the UPF parks downlink packets of idle UEs (64 in all, preallocated) and reports them to the SMF (`N4Report`, at most every 10 s per session); the SMF sends the AMF an `SmNotify` with the N2 setup; the AMF pages the UE in its last TAI (every gNB serving it, NGAP Paging) and sets the paged sessions up on its Service Request (mobile terminated). An index from SMF context to its holder (connected UE or idle 5G-TMSI) routes the SMF's notices, whatever UE id the SMF last saw (it hears none from a deactivation). Proved by `user-plane.sh` (a ping from the core's side to the idle UE).
- **Benchmarks**: the README has a short capacity table for customers (cores for LibreCN → what it carries); every figure and its conditions are in `BENCHMARKS.md`. They are measured with the whole core (every EO in one pool) on 1 to 4 cores (`scripts/load.sh`); the UPF alone (`scripts/upf-modes.sh`) only for modes and packet sizes, never with more cores in its pool (one EO, one core). About 9 000 registrations per second with their PDU session, or about 1 Gbit/s each way of 1400-byte packets, on one core; both at once need two cores (the AMF on one, the UPF on the other), and a third or fourth adds nothing. The AMF spends about 110 µs per registration, 40 % of it kernel SCTP; the other NFs use under 10 %. More needs several AMF or UPF instances (multi-UPF: a later feature, the owner's idea) or a cheaper AMF. Memory (RAM sizing, in the README too): 2 to 4 KiB per UE measured (resident, at its peak), planned as 20 MiB + 5 KiB per UE; the capacity (`max_ues`, `max_sessions`, subscribers) is reserved at start and becomes resident as UEs come. Benchmarks run with `[oam] log_level = "error"`.
- **Overload**: every NF's buffer classes hold as many bodies as its mailbox has slots (1024). The AMF's one-way notices to the SMF (deactivations, releases) are acknowledged and sent 64 at most unacknowledged, so a gNB's loss (thousands of UEs going idle) cannot fill the SMF's mailbox and refuse other traffic.
- **Idle mobility** (`mobility.sh`, Docker, two gNBs in tracking areas 1 and 2): the idle UE moves, sends a mobility registration (the sessions it holds told back in the Accept, those it lost released), and is reached through the new gNB. Once a UE's Initial Context Setup succeeded, later NAS (a connected Service Request's Accept) goes in DL NAS Transport. UERANSIM's connected UE sends uplink data without asking for its session's user plane, so the check pings from the network's side.
- **Configuration Update Command**: with `network_name` (full, short optional) the AMF sends the names after each registration (UCS-2, no acknowledgement); `POST /api/nfs/{amf}/ues/{supi}/guti` gives a connected UE a new 5G-GUTI, acknowledged (T3555), current once the UE answers. `registration.sh` checks the UE's stored 5G-GUTI (`nr-cli status`).
- **AMF Configuration Update and UE Context Modification** (the AMF's `manage.rs`): `POST capacity` announces a new relative capacity to every gNB; `POST ues/{supi}/ambr` sets a connected UE's UE-AMBR at its gNB, and the UE-AMBR (that one, or `ue_ambr_mbps`) goes with the UE's PDU Session Resource Setup Requests. `registration.sh` checks both.
- **EAP-AKA'** (`auth = "eap-aka-prime"` per UDM subscriber, 5G AKA by default): the UDM gives XRES, CK', IK'; the AUSF builds EAP-Request/AKA'-Challenge (`proto/security/eap_aka.rs`: CK'/IK', PRF', MK with the SUPI's digits as identity, AT_MAC), checks the UE's EAP-Response and gives K_SEAF (from EMSK) with EAP-Success; the AMF carries the EAP packets (Authentication Request/Response, EAP-Success in the Security Mode Command, EAP-Failure in Authentication Reject). `registration.sh` uses EAP-AKA', the Docker and scale checks 5G AKA.
- **Overload control**: at 90 % of `max_ues` UE contexts, every gNB gets Overload Start (only emergency and mobile terminated: paging still works) and a MAJOR alarm is raised; under 80 %, Overload Stop. UERANSIM's gNB only logs both (`overload.sh`).
- **Deregistration by the network** (`DELETE /api/nfs/{amf}/ues/{supi}`, `{"reregister": true}` optional; the AMF's `manage.rs`, which also lists UEs): a connected UE gets Deregistration Request (T3522, local after the fifth expiry), an idle one is deregistered locally. Proved by `registration.sh` (UERANSIM registers again).
- **AMF-UE-NGAP-IDs** are allocated within 32 bits, so that they are also the `corr_id` of the UE's requests to the AUSF and the SMF.
- **PDU sessions in the AMF** (`sessions.rs`): the AMF carries N1 and N2 without reading them; the SMF encodes and decodes both. The SMF's answer to CreateSMContext (and to UpdateSMContext) also carries what N1N2MessageTransfer would; the network's own releases come as `SmNotify` (N1N2MessageTransfer). A 5GSM message for an existing session goes to the SMF as UpdateSMContext (N1). Releases: by the UE (UERANSIM `ps-release`) and by the network (`DELETE /api/nfs/smf/sessions/{id}`); a UE's modification request is rejected, network-requested modification is not done (UERANSIM supports none). A UE's sessions are released at the SMF when its context is forgotten (not when it goes idle).
- **SCTP**: one-to-many style (one socket for every gNB, association id per message), kernel SCTP through `libc`.
- **Structure**: procedures are pure functions on domain values (`setup.rs`); ASN.1 only in `n2.rs`; the EO is thin glue (`lib.rs`).
- **Test values** (the owner let me choose; they match UERANSIM's samples): PLMN 999/70, TAC 1, SST 1, AMF Region 1 / Set 1 / Pointer 0, capacity 255, N2 on 127.0.0.1:38412; subscriber imsi-999700000000001 with UERANSIM's K and OPc, AMF field 8000. The UDM's SQN is SEQ || IND (5 bits, IND 0), as UERANSIM checks it.
- **Session values** (agreed with the owner, in the SMF's params until subscription data exists): DNNs as a list (`dnns`, each with its IPv4 pool, the first by default; the interop setups serve `internet` 10.45.0.0/16 and `ims` 10.46.0.0/16, and the UE holds a session on each), SST 1, the first address of each pool is the UPF's N6 side, default 5QI 9, session AMBR 1000 Mbit/s each way. The UPF's N3 is 127.0.0.2: UERANSIM's gNB binds GTP-U on 127.0.0.1.
- **UERANSIM** v3.3.0 lives in `/mnt/part_1TB_disck/prg/UERANSIM`, next to this repository, built by its `build.sh` without system packages: cmake and libsctp come from its `tools/`.
  - **UERANSIM is patched** (`scripts/ueransim/patches/`, applied to `../UERANSIM/src` and rebuilt):
    - 0001: v3.3.0's gNB picks an AMF only from the slice of a plain Registration Request, so no protected initial message (a UE back from idle) could reach any AMF. `user-plane.sh` refuses to run without it.
    - 0002: a data race on the gNB's RLS UE map (two threads, no lock) crashed it with hundreds of UEs attaching at once.
    - 0004: the gNB could not encode its AMF Configuration Update Acknowledge (an empty optional list it always added): no acknowledgement ever left it. `registration.sh` refuses to run without 0003 and 0004.
    - 0003: the UE refused EAP attributes of one 4-octet unit, so it could not read AT_KDF (RFC 9048) and crashed on any EAP-AKA' challenge.
    - 0005: the gNB has one cell only; `extraCells` (`nci`, `linkIp`) adds cells, each with its own radio address and transmitter identity, the UE's cell in its User Location Information. `demo.sh` (its first gNB has two cells) refuses to run without it; two `nr-ue` in one container need their own `tunName` and routing tables listed in advance (they pick the same numbers otherwise).
  - **Scale validated with UERANSIM** (`scale.sh`): 2000 UEs over 8 gNBs, registered with their PDU session; beyond, UERANSIM itself fails on this machine (cell selection, UAC timing, thread limit). The core's own figures come from `app/examples/load.rs` (`scripts/load.sh`), in `README.md`.
  - On the host, UERANSIM's UE does not come back from idle (its RRC thread misses the 100 ms UAC check); in Docker it does, so the idle cycle is checked by `user-plane.sh`.
  - Interop checks: `scripts/ueransim/ng-setup.sh` (gNB), `scripts/ueransim/registration.sh` (gNB + UE: registration, PDU session, then switch-off deregistration through `nr-cli`), `scripts/ueransim/user-plane.sh` (in Docker: a ping through the UPF, then the UE idle, back with a Service Request, a ping again).
- **UPF modes** (`mode`): `event` (default), `poll`, `hybrid`, measured in `nf/upf/README.md` (`scripts/upf-modes.sh`): throughput is the same (about 310 kpps per core, system-call bound), polling cuts a round trip by 9 µs and sparse latency by half, for a core spent when idle; hybrid keeps poll's latency under traffic and event's idle cost. The number of UEs (1 to 1000) does not change throughput: about 300 kpps uplink and 180 kpps downlink one direction at a time, about 100 kpps each way with both at once (the figure customers see).
- **UPF**: user plane only when `n6` (a TUN device) is configured; then N3 is bound on `n3`:2152. It attaches to the device and never configures the network itself (address, link: the deployment's job, `docker/core.sh`).

## Commands

```
cargo test --workspace                                    # before every commit
scripts/stress.sh [RUNS]                                  # slow checks
cargo bench -p librecn-scheduler
cargo run --release -p librecn -- --config app/librecn.example.toml   # GUI on http://127.0.0.1:8080
scripts/ueransim/ng-setup.sh                              # interop with UERANSIM's gNB
scripts/ueransim/registration.sh                          # UERANSIM gNB + UE: registration, PDU session, deregistration
scripts/ueransim/user-plane.sh                            # in Docker: UERANSIM's UE pings through the UPF
scripts/upf-modes.sh                                      # the UPF alone: modes, UEs, packet sizes
scripts/load.sh                                           # capacity on 1 to 4 cores (control and user plane)
scripts/ueransim/scale.sh                                 # UERANSIM at scale (GNBS=4 UES=250 by default)
scripts/ueransim/overload.sh                              # UERANSIM: Overload Start and Stop from a full AMF
scripts/ueransim/mobility.sh                              # in Docker: an idle UE moves to another gNB and tracking area
scripts/ueransim/demo.sh [stop]                           # in Docker: a long run to watch in the GUI (GNBS=2 UES=10 PORT=8080), Ctrl-C ends it
```
