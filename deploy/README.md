# LibreCN in Docker

LibreCN, a 5G core, for real gNBs, in one container. Linux on x86-64
(amd64) or ARM (arm64), with Docker and Docker Compose.

## Start

```
mkdir librecn && cd librecn
curl -fsSLO https://raw.githubusercontent.com/flesager/LibreCN/main/deploy/compose.yaml
curl -fsSL https://raw.githubusercontent.com/flesager/LibreCN/main/deploy/env.example -o .env
# set the PLMN of your gNB and SIMs in .env (LIBRECN_MCC, LIBRECN_MNC)
sudo sysctl -w net.ipv4.ip_forward=1     # once: the UEs reach the internet through the host
docker compose up -d
```

Then:

1. Open the GUI on the host: http://127.0.0.1:8080 (or through an SSH
   tunnel: `ssh -L 8080:127.0.0.1:8080 host`).
2. Add your SIMs under **Provisioning** → Subscribers: IMSI, K and OPc.
   They are kept across restarts and upgrades.
3. Point the gNB's N2 at the host's address, port 38412 (SCTP). The core
   tells the gNB its N3 address (GTP-U 2152) itself.

The **Overview** shows the gNBs and their UEs as they come.

| Command | |
|---|---|
| `docker compose logs -f` | the core's logs (JSON lines) |
| `docker compose down` | stop it (subscribers and policies are kept) |
| `docker compose pull && docker compose up -d` | upgrade to the latest image |
| `docker compose down -v` | stop it and forget the subscribers and policies |

## Settings (`.env`)

Every line is optional; [`env.example`](env.example) explains each one.

| Setting | Default | |
|---|---|---|
| `LIBRECN_ADDRESS` | the host's main address | where gNBs reach N2 and N3 |
| `LIBRECN_MCC`, `LIBRECN_MNC` | 001, 01 | the PLMN, as on the gNB and the SIMs |
| `LIBRECN_SST`, `LIBRECN_SD` | 1, none | the slice served |
| `LIBRECN_DNN`, `LIBRECN_UE_POOL` | internet, 10.45.0.0/16 | the data network and the UEs' addresses |
| `LIBRECN_NAT` | yes | the UEs reach the outside through the host |
| `LIBRECN_GUI` | 127.0.0.1:8080 | the GUI and web API; `0.0.0.0:8080` opens them to the network (**no authentication yet**: trusted networks only) |
| `LIBRECN_LOG` | info | log level: vip, fatal, error, info, debug |
| `LIBRECN_VERSION` | latest | the image's version |

For anything else (several DNNs, policies, timers, the CPU cores used):
write a full configuration ([`app/librecn.example.toml`](../app/librecn.example.toml),
[`RUNNING.md`](../RUNNING.md)) and mount it, as `compose.yaml` shows; the
`.env` network settings are then ignored. Use `n6 = "upf0"` and keep the UE
pool in `LIBRECN_UE_POOL` for the NAT.

## What the container does to the host

It shares the host's network: the gNBs reach it on the host's address,
and the UEs' traffic leaves through the host. While it runs, the host has
a TUN device `upf0` (the UE pool's first address, 10.45.0.1) and, with
`LIBRECN_NAT=yes`, NAT and forwarding rules for the UE pool; they are
removed when it stops. It needs `NET_ADMIN` and `/dev/net/tun`, and
`BPF` and `PERFMON` for the UPF's fast path, nothing more (not
privileged).

**The fast path**: the kernel carries the UEs' uplink (XDP, on the
interface of `LIBRECN_ADDRESS`), about twice the packets of the UPF alone
on half the cores. It needs a loose reverse path filter on that interface
(`rp_filter` 0 or 2, the default on Ubuntu and Debian; 1 on Red Hat
systems): `sudo sysctl -w net.ipv4.conf.all.rp_filter=2
net.ipv4.conf.<interface>.rp_filter=2`. Without it, or without the
capabilities, the core runs without the fast path, says why in its logs
and raises a MINOR alarm (the GUI's System panel).

The host must:
- forward IP packets, for the NAT: `sudo sysctl -w net.ipv4.ip_forward=1`,
  kept with `echo net.ipv4.ip_forward=1 | sudo tee /etc/sysctl.d/99-librecn.conf`;
- allow SCTP (N2): the kernel loads it when the core opens its socket. If
  the logs say SCTP is missing: `sudo modprobe sctp`.

## The image

`ghcr.io/flesager/librecn`, for amd64 and arm64, built from this repository
by [`Dockerfile`](Dockerfile): a static binary (musl) compiled on the
build machine (arm64 cross-compiled, not emulated), on Alpine with `ip`
and `iptables`; about 25 MB. With musl the core registers UEs about a
third slower than built with glibc (BENCHMARKS.md §5: still about 6 000
per second); its user plane is the same. To build it yourself:

```
docker buildx build -f deploy/Dockerfile --platform linux/amd64 -t librecn:local --load .
```

A version tag (`v0.1.0`) pushed to GitHub builds and publishes both
architectures ([`.github/workflows/image.yml`](../.github/workflows/image.yml)),
tagged with the version and `latest`.
