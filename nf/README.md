# Network functions

One crate per 3GPP network function, each hosted in an execution object of
`librecn-scheduler`, logging and raising alarms through `librecn-common`, and
registered as a kind in the `librecn` application so the control EO can
deploy it. NFs talk to each other through EO messages (`proto/sbi`), found by
their EO names (parameters `ausf`, `smf` and `pcf` of the AMF, `udm` of the
AUSF, `upf` and `pcf` of the SMF).

| Crate | NF | Status |
|---|---|---|
| [`amf`](amf/) | AMF | N2 (SCTP one-to-many, NG Setup, NG Reset, Error Indication, UE contexts, NAS transport, UE Context Release); registration: 5G AKA, NAS security, Registration Accept in Initial Context Setup, retransmission timers (T3550, T3560, T3570), identification, deregistration by the UE or the network (`/api/nfs/amf-1/ues`, T3522); Configuration Update Command (the network's names after each registration, a new 5G-GUTI through `/api/nfs/amf-1/ues/{supi}/guti`, T3555); AMF Configuration Update (the relative capacity, `/api/nfs/amf-1/capacity`); the UE-AMBR from the PCF (AM policy), given with the sessions and changed by UE Context Modification; RRC_INACTIVE allowed (Core Network Assistance Information in Initial Context Setup) and reported (RRC Inactive Transition Report: the UE's RRC state and cell, on the web API with the UEs per gNB and cell); overload control (Overload Start at 90 % of `max_ues`, Stop under 80 %, an alarm); idle UEs kept (Service Request, registration from idle with the sessions told back, mobility to another gNB and tracking area, periodic registration with the AMF's T3512, implicit deregistration), paged for downlink data; PDU sessions carried between UE, SMF and gNB (PDU Session Resource Setup, reactivation) |
| [`ausf`](ausf/) | AUSF | 5G AKA and EAP-AKA' (Nausf_UEAuthentication): challenge (HXRES*, or EAP-Request/AKA'-Challenge), confirmation (RES*, or the UE's EAP-Response: AT_MAC, RES), K_SEAF, EAP-Success or EAP-Failure; contexts expire after 30 s |
| [`smf`](smf/) | SMF | IPv4 PDU session establishment: checks (DNN, slice, type, SSC mode), several DNNs each with its address pool, several sessions per UE, the session AMBR and QoS flows from the PCF (SM policy: the default flow, and one per PCC rule with its QoS rule, packet filters and flow description to the UE, its QoS parameters and GBR information to the gNB, its filters to the UPF), N1 accept/reject, N2 transfers, N4 to the UPF; release by the UE or the network (`/api/nfs/smf/sessions`); user plane deactivated and reactivated with the UE's idleness, downlink data for an idle UE passed to the AMF; a UE's modification request rejected |
| [`upf`](upf/) | UPF | N4 sessions; GTP-U on N3 and a TUN device on N6, both ways, IPv4, source address checked, downlink packets put on the PCF's QoS flows by their filters (QFI), uplink QFIs checked, the session AMBR and the flows' MBRs enforced both ways (GCRA, 100 ms bursts; GBR flows outside the AMBR), no allocation per packet; downlink packets of idle UEs parked (64 in all) and reported to the SMF; packet and byte counters on `/metrics` (`extra_flow_packets`: downlink packets on a PCC rule's flow; `rate_limited_packets`), and per cell (`/api/nfs/upf/cells`, the UE's cell given by the AMF through the SMF with the gNB's tunnel, 1024 cells at most); event, poll or hybrid mode, measured in [`upf/README.md`](upf/README.md) |
| [`pcf`](pcf/) | PCF | per-subscriber policies with a `default` entry, from the deployment or the web API (`/api/nfs/pcf/policies`), kept in the state file: AM policy (the UE-AMBR, its changes sent to the UE's AMF), SM policy by DNN (session AMBR, default 5QI and ARP, PCC rules with packet filters) |
| [`udm`](udm/) | UDM | subscribers from the deployment or the web API (`/api/nfs/udm/subscribers`, GUI view); SUCI de-concealment (null scheme, ECIES Profile A with the home network keys of the deployment); authentication vectors (Milenage) for 5G AKA or EAP-AKA' (`auth` per subscriber), SQN resynchronisation |

Each NF keeps protocol decisions as pure functions on domain values (unit
tested without I/O), codecs in one module, and the EO as thin glue.

Interop with UERANSIM: `scripts/ueransim/ng-setup.sh` (gNB) and
`scripts/ueransim/registration.sh` (gNB + UE: registration, PDU session, then
switch-off deregistration), and `scripts/ueransim/user-plane.sh` (in Docker:
a ping through the UPF, the UE back from idle, then paged); UERANSIM is built next to this repository, see
`CLAUDE.md`.

## Not done yet

Deliberately left out of this release, to be added when a step needs them:

- **Paging retransmission** (T3513) and paging priorities: an idle UE is
  paged once per downlink data report, the UPF reporting again at most every
  10 s while the UE stays away.
- **User plane extras**: IPv6, Error Indication for unknown TEIDs, AMBR
  enforcement, batched system calls (`recvmmsg`/`sendmmsg`: the measured
  limit, see `upf/README.md`).
- **Network-requested PDU session modification** (QoS, AMBR): UERANSIM
  supports neither the Modification Command nor PDU Session Resource Modify,
  so nothing could prove it; a UE's modification request is rejected (#31).
- **QoS**: the UPF classifies the downlink only; uplink packets are taken
  on the flow the gNB marks (UERANSIM's UE keeps its QoS rules unused and
  its gNB marks every packet with the session's first flow, so nothing could
  prove more). No reflective QoS, no dynamic 5QI characteristics. An Accept
  fits in 512 bytes: four GBR flows of four filters with every field do
  not, and such a session is refused (#26). The UE-AMBR is not enforced by
  the UPF (the gNB gets it); N4 as PFCP (EO messages instead, by choice).
- **Rate shaping**: the UPF polices (drops beyond the rate, no queue), as a
  QER's MBR does. Measured through UERANSIM in Docker (`user-plane.sh`), UDP
  gets the session AMBR exactly, but one TCP flow over that sub-millisecond
  round trip gets about half of it (10 of 20 Mbit/s): a window of one or two
  packets cannot raise three duplicate ACKs, so each loss waits for the
  retransmission timeout. Over real round trips (tens of ms) the 100 ms burst
  covers the window.
- **Subscription data** (Nudm_SDM, Nudm_UECM): the allowed NSSAI is the
  requested slices the AMF serves, else all of them; the SMF's DNNs and
  pools apply to every UE (the QoS comes from the PCF).
- **SUCI protection scheme** B (ECIES, P-256: UERANSIM does not support it); EAP-AKA' resynchronisation (the UE's AUTS in an EAP-Response gets EAP-Failure); NIA1/NIA3,
  NEA1/NEA3, the AUSF's result confirmation to the UDM.
