# Integrating LibreCN with a real RAN

For the team integrating LibreCN with commercial and open RANs (gNBs, the
OCUDU and OAI CUs), and validating it end to end with real UEs (Quectel
modems); and for the Claude sessions helping them. Read this first, then
[`CLAUDE.md`](CLAUDE.md) (the design decisions and the project's rules),
[`RUNNING.md`](RUNNING.md) (configuration) and
[`deploy/README.md`](deploy/README.md) (the Docker image).

## 1. What LibreCN is

A 5G standalone core in Rust for small deployments: AMF, AUSF, UDM, SMF, PCF
and UPF in one process, each network function an execution object (EO) on a
reactor scheduler, plus OAM (logs, alarms, a web API and GUI). Between its
functions, the 3GPP service operations are messages in memory, not HTTP/2;
N4 is not PFCP (both are deliberate choices, see `CLAUDE.md`). Towards the
outside it speaks the real interfaces:

| Interface | Protocol | Where |
|---|---|---|
| N2 | NGAP (TS 38.413, Release 15 ASN.1) over kernel SCTP, port 38412 | `nf/amf` (`n2.rs`), `proto/ngap` |
| N1 | NAS 5GMM and 5GSM (TS 24.501) | `proto/nas`, `nf/amf` (`mm.rs`), `nf/smf` |
| N3 | GTP-U (TS 29.281) with the PDU Session Container, UDP 2152 | `nf/upf`, `proto/gtpu` |
| N6 | a TUN device (`upf0`); a kernel fast path (XDP, TC) carries most traffic | `nf/upf` (`fast.rs`, `bpf/upf.bpf.c`) |

## 2. What is proven, and against what

Everything so far ran against simulators and test peers, **never a
commercial gNB, an O-RAN CU or a real UE**: that is your work.

| Proven | Against |
|---|---|
| NG Setup, NG Reset, Error Indication | UERANSIM's gNB (`scripts/ueransim/ng-setup.sh`) |
| Registration (5G AKA and EAP-AKA'), NAS security (NIA2, NEA2/NEA0), PDU sessions on two DNNs, releases, deregistration, Configuration Update, UE Context Modification | UERANSIM (`registration.sh`) |
| User plane, idle and Service Request, paging, periodic registration, the fast path, QoS flows and AMBR, TCP | UERANSIM in Docker (`user-plane.sh`) |
| Idle mobility to another gNB and tracking area | UERANSIM in Docker (`mobility.sh`) |
| free5GC's NGAP and NAS codecs; 1000 UEs | free-ran-ue (`scripts/free-ran-ue/check.sh`) |
| 2000 UEs over 8 gNBs | UERANSIM (`scale.sh`) |
| Capacity (registrations, Gbit/s) | LibreCN's own load generators (`BENCHMARKS.md`) |

UERANSIM is patched for five bugs of its own (`scripts/ueransim/patches/`).

## 3. Deploying it for your lab

The Docker image (`ghcr.io/flesager/librecn`, amd64 and arm64) is the quickest
path: [`deploy/README.md`](deploy/README.md). It shares the host's network,
creates `upf0`, NAT and the fast path, and is configured from `.env`. For
anything beyond `.env` (several DNNs, several slices, the AMF's identity,
timers), mount a full configuration ([`app/librecn.example.toml`](app/librecn.example.toml),
[`RUNNING.md`](RUNNING.md) §2), or run the binary on the host (`RUNNING.md`).

What your RAN must match ([`RUNNING.md`](RUNNING.md) §5 has the full table):

| Value | LibreCN | Notes |
|---|---|---|
| AMF address | AMF `n2 = "<address>:38412"` | one address (no SCTP multi-homing) |
| PLMN | AMF `plmn`, the same on the SIMs | the AMF refuses a gNB that broadcasts no TA with its PLMN; any TAC is accepted |
| Slices | AMF and SMF `slices` (SST, optional SD) | not checked at NG Setup; a UE's requested slices must be among them (the allowed NSSAI) |
| AMF identity | AMF `amf_region`, `amf_set`, `amf_pointer` | in the GUAMI the gNB is told; check your CU's AMF configuration if it filters on it |
| N3 | UPF `n3`, the address the gNB's GTP-U reaches | told to the gNB in each PDU Session Resource Setup |
| DNNs | SMF `dnns`: name, UE pool, DNS servers, MTU | the modem's APN/DNN must be one of them, or empty (the first) |
| Ciphering | AMF `ciphering = ["NEA2", "NEA0"]` | integrity is NIA2 only; set NEA0 first to read NAS in Wireshark |

On the host: kernel SCTP (`modprobe sctp`), IP forwarding, a loose
`rp_filter` and GRO off on the card the internet's traffic comes in by (the
fast path needs both; the Docker entrypoint does GRO itself). The GUI is on
`http://127.0.0.1:8080`; its System panel shows alarms (an N2 association
lost, the fast path unavailable, GRO on).

## 4. SIMs and Quectel modems

- **Provisioning**: the GUI's Provisioning panel, one UE with its policy, or
  many from a JSON file (its format is shown on the page); or the
  configuration (`[[nf.params.subscribers]]`). Each SIM needs its IMSI, K,
  OPc, and the AMF field (default 8000). SQN: the UDM resynchronises once if
  the SIM is ahead (5G AKA; not yet with EAP-AKA').
- **SUCI**: the null scheme, or Profile A with the home network key in the
  UDM (`[[nf.params.home_network_keys]]`). **Profile B is not supported**:
  program the SIMs for the null scheme or Profile A.
- **On the modem** (check the exact commands in your module's AT manual):
  5G SA only; the DNN in `AT+CGDCONT` (IPv4 or IPv4v6: either works, the
  session is IPv4); IMS off (`AT+QCFG="ims",0` on most Quectel modules), as
  LibreCN has no IMS (no P-CSCF: the modem would retry its IMS
  registration); then `AT+C5GREG?`, `AT+CGPADDR`, and a ping and a DNS
  lookup through the session.
- **DNS**: the SMF answers a UE's request for DNS servers and the MTU (ePCO)
  with the DNN's `dns` (default 8.8.8.8, 1.1.1.1) and `mtu` (1400).

## 5. Known gaps and risks

What a real RAN or UE may hit; most are listed in [`nf/README.md`](nf/README.md)
("Not done yet"):

1. **NGAP is Release 15**: a Release 16/17 gNB or CU sends newer optional
   IEs. The decoder skips unknown extensions, and bad NGAP gets an Error
   Indication; a decoding failure in NG Setup or Initial UE Message is the
   first thing to capture (§6).
2. **IPv4 only**: an IPv6 PDU session is refused; IPv4v6 is served as IPv4.
3. **No handover** (Xn or N2), no network-requested PDU session
   modification, no SMS, no IMS or VoNR, no LTE or NB-IoT (no ng-eNB).
4. **SUCI Profile B**, NIA1/NIA3, NEA1/NEA3 are not implemented.
5. **One AMF, one SMF, one UPF**, no NRF and no HTTP/2 SBI: third-party NFs
   cannot be plugged in (a reverse proxy can expose one interface, see §7).
6. **RRC_INACTIVE** is requested from the gNB, but proven by an NGAP unit
   test only (UERANSIM never suspends a UE).
7. **QoS**: the UPF classifies the downlink and polices; the uplink rides
   the flow the gNB marks.

## 6. Debugging and reporting

- **Logs**: JSON lines on stdout (`docker compose logs -f`). At `info` (the
  default) every NAS step of every UE is a line (`UE 1: uplink Registration
  Request (plain)`, ...); `debug` (`[oam] log_level`, `LIBRECN_LOG`) adds
  the rarer cases.
- **Captures**: N2 `tcpdump -i <card> -w n2.pcap sctp port 38412`, N3 `udp port
  2152`. With NEA2 the NAS is ciphered: put `"NEA0"` first in `ciphering`
  to read it. Wireshark decodes NGAP, NAS and GTP-U.
- **State**: the GUI's Overview (gNBs and their UEs by cell and RRC state),
  UEs (one UE's sessions), System (alarms, EOs, their queues); Prometheus
  `/metrics`; the web API (`oam/README.md`).
- **A failure report** (an issue, or the owner): what was attempted, the
  RAN and UE versions, the capture and the core's log of the same minutes,
  and the LibreCN commit.

## 7. The next phase: openMCX (mission-critical push-to-talk)

[openMCX](https://github.com/openMCX) (3GPP Release 19 MCPTT, MCData,
MCVideo) runs over a 5G core with its calls' QoS asked over N5. Its tutorial
`03-mcx-over-oai-5g` (OAI core and RAN) is the reference; to run it with
LibreCN instead of OAI's core:

1. **Network-requested PDU session modification** (SMF, AMF, UPF): add a
   QoS flow to a running session and remove it: the 5GSM PDU Session
   Modification Command and Complete (TS 24.501 §6.3.2), NGAP PDU Session
   Resource Modify Request and Response, the UPF's rules updated (`N4Modify`,
   the fast path's `rules` map). Trigger it first from the web API (a flow
   posted onto a session); UERANSIM cannot test it, your RAN and modems can.
2. **N5** (`Npcf_PolicyAuthorization`, TS 29.514): `POST
   /npcf-policyauthorization/v1/app-sessions` and `DELETE`; the PCF finds the
   session by the UE's address and turns the media components into a PCC
   rule (5QI 65, GBR, ARP 3 with pre-emption), which reaches the SMF (step 1).
   The owner's choice: an off-the-shelf reverse proxy (nginx or Envoy)
   terminates HTTP/2 (and TLS, an allow-list of AFs) and forwards HTTP/1.1
   to LibreCN, so LibreCN gets no HTTP/2 code; N5 gets its own listener,
   apart from the GUI's. openMCX's MCPTT server is a trusted AF (no NEF).
3. **A DNN `mcptt`**, its default flow 5QI 69 (configuration only).
4. **Acceptance**: tutorial 3's calls with LibreCN: the app session created
   (201), QoS flow 2 with 5QI 65 set up at the gNB and the UE, speech on it in
   GTP-U, the flow released at the call's end.

## 8. Working in this repository (people and Claude)

- `CLAUDE.md` is loaded by Claude in every session: its principles apply
  (light code, no allocation on the hot path, remove before adding), and so
  do its working rules. "The owner" there is the project's owner: structural
  choices (a new dependency, a new interface, a new NF) are agreed with them
  first, as numbered questions.
- Before reporting work done: `cargo fmt --all`, `cargo clippy --workspace
  --all-targets` (zero warnings), `cargo test --workspace` (seconds).
- Work on a branch and send it to the owner (a fork and a pull request, or
  patches); the central repository has `main` only, and the owner publishes.
- Each step proven against a real peer, as the UERANSIM scripts do: a script
  in `scripts/` that sets it up, runs it and says OK or FAILED, when the lab
  allows it; otherwise the capture and the log, kept with the change's
  description.
