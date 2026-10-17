# Network functions

One crate per 3GPP network function, each hosted in an execution object of
`librecn-scheduler`, logging and raising alarms through `librecn-common`, and
registered as a kind in the `librecn` application so the control EO can
deploy it.

| Crate | NF | Status |
|---|---|---|
| [`amf`](amf/) | AMF | N2: SCTP (one-to-many), NG Setup, NG Reset, Error Indication, UE contexts, NAS transport, UE Context Release; NAS is a stub |

Each NF keeps protocol decisions as pure functions on domain values (unit
tested without I/O), ASN.1 in one module, and the EO as thin glue.

Interop with UERANSIM: `scripts/ueransim/ng-setup.sh` (gNB) and
`scripts/ueransim/registration.sh` (gNB + UE); UERANSIM is built next to this
repository, see `CLAUDE.md`.
