# Network functions

One crate per 3GPP network function, each hosted in an execution object of
`librecn-scheduler`, logging and raising alarms through `librecn-common`, and
registered as a kind in the `librecn` application so the control EO can
deploy it. NFs talk to each other through EO messages (`proto/sbi`), found by
their EO names (parameters `ausf` of the AMF, `udm` of the AUSF).

| Crate | NF | Status |
|---|---|---|
| [`amf`](amf/) | AMF | N2 (SCTP one-to-many, NG Setup, NG Reset, Error Indication, UE contexts, NAS transport, UE Context Release); registration: 5G AKA, NAS security, Registration Accept/Complete, identification, UE-originating deregistration |
| [`ausf`](ausf/) | AUSF | 5G AKA (Nausf_UEAuthentication): challenge with HXRES*, confirmation, K_SEAF; contexts expire after 30 s |
| [`udm`](udm/) | UDM | subscribers from the deployment; SUCI de-concealment (null scheme); authentication vectors (Milenage), SQN resynchronisation |

Each NF keeps protocol decisions as pure functions on domain values (unit
tested without I/O), codecs in one module, and the EO as thin glue.

Interop with UERANSIM: `scripts/ueransim/ng-setup.sh` (gNB) and
`scripts/ueransim/registration.sh` (gNB + UE: registration, then switch-off
deregistration); UERANSIM is built next to this repository, see `CLAUDE.md`.

## Not done yet

Deliberately left out of this release, to be added when a step needs them:

- **AMF contexts of idle UEs**: a UE's 5GMM context lives as long as its N2
  context. A UE coming back with a 5G-GUTI is identified again (Identity
  Request), and a Service Request is answered by Service Reject #9.
- **NAS retransmission timers** (T3550, T3560, T3570): a UE that stops
  answering keeps its context until the gNB releases it.
- **Initial Context Setup** (K_gNB to the gNB, AS security) and PDU
  sessions: step 4, with an SMF.
- **Subscription data** (Nudm_SDM, Nudm_UECM): the allowed NSSAI is the
  requested slices the AMF serves, else all of them.
- **SUCI protection schemes** A and B (ECIES), EAP-AKA', NIA1/NIA3,
  NEA1/NEA3, the AUSF's result confirmation to the UDM.
