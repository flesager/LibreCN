# Docker

One image with LibreCN and UERANSIM's binaries, for the interop checks in
Docker (`scripts/ueransim/user-plane.sh`, `mobility.sh`, `demo.sh`, sharing
`docker.sh`): UERANSIM's UE needs root for its TUN device, which a
container gives without touching the host.

| File | Role |
|---|---|
| `Dockerfile` | Ubuntu 26.04 (the build host's), `iproute2`, `ping`, `curl` (the metrics check), `iperf3` (the session AMBR measured), `libsctp1` (UERANSIM's gNB), the binaries |
| `core.sh`, `core.toml` | the core's container (10.210.0.10), `core.sh CONFIG` for another configuration (`demo.sh` mounts its own): `upf0` (10.45.0.1/16) for N6, an iperf3 server on 10.45.0.1, then every NF (the PCF's policy: an extra flow, the session AMBR) |
| `gnb.yaml`, `ue.yaml` | the RAN's container (10.210.0.20): UERANSIM's gNB and UE |

The binaries are built on the host and copied in, so the image needs the
host's glibc: hence the same Ubuntu release. Both containers need
`--cap-add NET_ADMIN --device /dev/net/tun`; SCTP comes from the host's
kernel (`sctp` module).
