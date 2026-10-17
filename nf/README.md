# Network functions

One crate per 3GPP network function, each hosted in an execution object of
`librecn-scheduler`, logging and raising alarms through `librecn-common`, and
registered as a kind in the `librecn` application so the control EO can
deploy it. NFs talk to each other through EO messages (`proto/sbi`), found by
their EO names (parameters `ausf` and `smf` of the AMF, `udm` of the AUSF,
`upf` of the SMF).

| Crate | NF | Status |
|---|---|---|
| [`amf`](amf/) | AMF | N2 (SCTP one-to-many, NG Setup, NG Reset, Error Indication, UE contexts, NAS transport, UE Context Release); registration: 5G AKA, NAS security, Registration Accept in Initial Context Setup, retransmission timers (T3550, T3560, T3570), identification, UE-originating deregistration; idle UEs kept (Service Request, registration from idle, implicit deregistration); PDU sessions carried between UE, SMF and gNB (PDU Session Resource Setup, reactivation) |
| [`ausf`](ausf/) | AUSF | 5G AKA (Nausf_UEAuthentication): challenge with HXRES*, confirmation, K_SEAF; contexts expire after 30 s |
| [`smf`](smf/) | SMF | IPv4 PDU session establishment: checks (DNN, slice, type, SSC mode), address pool, default QoS flow, N1 accept/reject, N2 transfers, N4 to the UPF; release by the UE or the network (`/api/nfs/smf/sessions`); user plane deactivated and reactivated with the UE's idleness; a UE's modification request rejected |
| [`upf`](upf/) | UPF | N4 sessions; GTP-U on N3 and a TUN device on N6, both ways, IPv4, source address checked, no allocation per packet; packet and byte counters on `/metrics` |
| [`udm`](udm/) | UDM | subscribers from the deployment or the web API (`/api/nfs/udm/subscribers`, GUI view); SUCI de-concealment (null scheme); authentication vectors (Milenage), SQN resynchronisation |

Each NF keeps protocol decisions as pure functions on domain values (unit
tested without I/O), codecs in one module, and the EO as thin glue.

Interop with UERANSIM: `scripts/ueransim/ng-setup.sh` (gNB) and
`scripts/ueransim/registration.sh` (gNB + UE: registration, PDU session, then
switch-off deregistration), and `scripts/ueransim/user-plane.sh` (in Docker:
a ping through the UPF); UERANSIM is built next to this repository, see
`CLAUDE.md`.

## Not done yet

Deliberately left out of this release, to be added when a step needs them:

- **Paging**: downlink data for an idle UE is dropped by the UPF; the network
  does not page it (no Downlink Data Notification). The UE comes back by
  itself, with uplink data or signalling.
- **T3512 in Registration Accept**: the UE keeps its default (54 min).
- **User plane extras**: IPv6, Error Indication for unknown TEIDs, AMBR
  enforcement, the polling mode.
- **Network-requested PDU session modification** (QoS, AMBR): UERANSIM
  supports neither the Modification Command nor PDU Session Resource Modify,
  so nothing could prove it; a UE's modification request is rejected (#31).
- **QoS** beyond the default flow (5QI 9, match-all rule); the UE-AMBR in
  Initial Context Setup; N4 as PFCP (EO messages instead, by choice).
- **Subscription data** (Nudm_SDM, Nudm_UECM): the allowed NSSAI is the
  requested slices the AMF serves, else all of them; the SMF's DNN, pool
  and QoS apply to every UE.
- **SUCI protection schemes** A and B (ECIES), EAP-AKA', NIA1/NIA3,
  NEA1/NEA3, the AUSF's result confirmation to the UDM.
