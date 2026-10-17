# Running LibreCN

How to get LibreCN running on a machine, and connect it to a gNB and a data network.

## 1. What runs

LibreCN is **one process**, `librecn`, with **one configuration file**:

```
cargo build --release -p librecn
target/release/librecn --config librecn.toml
```

At start, in this order:
1. the runtime starts its worker threads, in the pools of `[runtime]`;
2. the OAM functions start: logs, alarms, the control function, and the web server on `[oam] listen`;
3. the control function starts each `[[nf]]` of the file, in order;
4. LibreCN runs until SIGTERM or SIGINT (Ctrl-C), then stops every function cleanly.

More network functions can be started later through the GUI or `POST /api/nfs`, with the same `[[nf]]` text.

**Privileges**: none, if the TUN device for N6 is created beforehand for the user that runs LibreCN (§4). Ports 38412 and 2152 are above 1024.

**Platform**: Linux, kernel SCTP (`modprobe sctp` if the module is not loaded), Rust 1.88 or newer to build.

## 2. The configuration file

Four kinds of sections, all optional. A full example with every parameter and its default is in [`app/librecn.example.toml`](app/librecn.example.toml).

```toml
[runtime]                 # the scheduler
fault_policy = "abort"    # or "isolate": a failing function is stopped, the others run on

[[runtime.pools]]         # worker threads: one pool per group of cores
name = "cp"
cores = [2, 3]            # pinned to these cores; or `workers = 2`, unpinned

[oam]                     # operation and maintenance
listen = "0.0.0.0:8080"   # web GUI, REST API, Prometheus /metrics
log_level = "info"        # vip, fatal, error, info, debug

[[nf]]                    # one network function
kind = "amf"              # amf, ausf, udm, smf, upf
name = "amf-1"            # its name, used by the others to find it
pool = "cp"               # optional: the first pool by default
[nf.params]               # its parameters (below)
n2 = "192.168.1.10:38412"
```

**The functions find each other by name.** The defaults match the names used here:

```
gNB ──N2──► amf-1 ──► ausf ──► udm      (authentication, subscribers)
              │
              └─────► smf ──► upf ◄──N3── gNB
                                │
                                N6 (TUN) ──► data network
```

| Function | Points to | Parameters, default |
|---|---|---|
| AMF | the AUSF and the SMF | `ausf = "ausf"`, `smf = "smf"` |
| AUSF | the UDM | `udm = "udm"` |
| SMF | the UPF | `upf = "upf"` |

**Pools and cores**: put every function in one pool pinned to the cores you give LibreCN. The [README](README.md#capacity) tells how many: 2 for the full capacity. The OAM functions always run in a pool named `oam`, added with one worker if the file has none. They are light, so they can share one of these cores:

```toml
[[runtime.pools]]
name = "cp"
cores = [2, 3]

[[runtime.pools]]
name = "oam"
cores = [2]
```

## 3. The interfaces

| Interface | Between | Protocol, port | Set by | What the other side needs |
|---|---|---|---|---|
| **N2** | gNB ↔ AMF | SCTP, 38412 | AMF: `n2 = "ADDR:38412"` | the gNB's AMF address: ADDR, port 38412 |
| **N3** | gNB ↔ UPF | GTP-U over UDP, 2152 | UPF: `n3 = "ADDR"` | nothing: the gNB is told ADDR at each PDU session |
| **N6** | UPF ↔ data network | IP packets on a TUN device | UPF: `n6 = "upf0"` | a route to the UE addresses (§4) |
| **OAM** | operator ↔ LibreCN | HTTP, 8080 | `[oam] listen` | a browser, Prometheus |

- **N2**: the AMF listens on `n2`. `0.0.0.0:38412` listens on every address of the machine.
- **N3**: `n3` must be a real address that the gNB reaches directly: the UPF binds it, and the SMF gives it to the gNB for uplink packets. Not `0.0.0.0`, and not an address hidden behind NAT. The gNB's own N3 address (UERANSIM: `gtpIp`) is learned from its answers.
- **N6**: without `n6`, the UPF keeps the sessions but forwards no packet: enough to test the control plane alone.

## 4. Preparing N6

LibreCN attaches to the TUN device, but never configures the network. The UE addresses come from the pools of the SMF's `dnns` (default: `internet`, `10.45.0.0/16`); the first address of each pool (10.45.0.1) is the UPF's side, to put on the device (one `ip addr add` per pool).

**Once per boot of the machine** (as root):

```
ip tuntap add dev upf0 mode tun user librecn   # owned by the user that runs LibreCN
ip addr add 10.45.0.1/16 dev upf0
ip link set upf0 up
```

**Then, so that the UEs reach the data network**, one of the following:
- **NAT on this machine**: the UEs go out with the machine's address.
  ```
  sysctl -w net.ipv4.ip_forward=1
  iptables -t nat -A POSTROUTING -s 10.45.0.0/16 ! -o upf0 -j MASQUERADE
  ```
- **Routing without NAT**: the data network's router sends 10.45.0.0/16 to this machine, which forwards it (`ip_forward=1`).

The project's own checks only ping 10.45.0.1, the UPF's side of N6 (`scripts/ueransim/user-plane.sh`). Traffic past it, through NAT or a router, is the deployment's part and is not tested yet.

## 5. What must match the gNB and the UEs

| Value | In LibreCN | In the gNB / UE (UERANSIM's names) |
|---|---|---|
| PLMN | AMF: `plmn = { mcc = "999", mnc = "70" }` | gNB and UE: `mcc`, `mnc`. The AMF refuses a gNB that broadcasts no TA with its PLMN |
| Slices | AMF and SMF: `slices = [{ sst = 1 }]` | gNB: `slices`; UE: `sessions[].slice` |
| AMF address | AMF: `n2` | gNB: `amfConfigs` |
| Subscriber | UDM: `[[nf.params.subscribers]]` with `supi`, `k`, `opc` (`amf`, `sqn`, `count`, `auth` = `5g-aka` or `eap-aka-prime` optional) | UE: `supi`, `key`, `op` with `opType: OPC`, `amf` |
| SUCI concealment | UDM: `[[nf.params.home_network_keys]]` with `id` and `private_key` (Profile A); its public key is logged at start | UE: `protectionScheme: 1`, `homeNetworkPublicKeyId`, `homeNetworkPublicKey`. With `protectionScheme: 0` (null scheme) no key is needed |
| Data networks | SMF: `dnns = [{ dnn = "internet", pool = "10.45.0.0/16" }]`, several allowed, the first by default | UE: `sessions[].apn` |

Subscribers can also be added while LibreCN runs: the GUI's Subscribers view, or `POST /api/nfs/udm/subscribers`. Those live in memory only and are gone at the next start.

## 6. A complete example

A machine at 192.168.1.10 on the RAN network, with 2 cores (2 and 3) for LibreCN:

```toml
[[runtime.pools]]
name = "cp"
cores = [2, 3]

[[runtime.pools]]
name = "oam"
cores = [2]

[oam]
listen = "0.0.0.0:8080"
log_level = "info"

[[nf]]
kind = "udm"
name = "udm"
[[nf.params.subscribers]]
supi = "imsi-001010000000001"
k = "465B5CE8B199B49FAA5F0A2EE238A6BC"
opc = "E8ED289DEBA952E4283B54E88E6183CA"

[[nf]]
kind = "ausf"
name = "ausf"

[[nf]]
kind = "upf"
name = "upf"
[nf.params]
n3 = "192.168.1.10"
n6 = "upf0"

[[nf]]
kind = "smf"
name = "smf"
[nf.params]
dnns = [{ dnn = "internet", pool = "10.45.0.0/16" }]

[[nf]]
kind = "amf"
name = "amf-1"
[nf.params]
n2 = "192.168.1.10:38412"
plmn = { mcc = "001", mnc = "01" }
```

Then:

```
sudo modprobe sctp
sudo ip tuntap add dev upf0 mode tun user $USER
sudo ip addr add 10.45.0.1/16 dev upf0
sudo ip link set upf0 up
target/release/librecn --config librecn.toml
```

The gNB is configured with AMF 192.168.1.10:38412, PLMN 001/01 and SST 1.

## 7. Checking that it runs

- **Logs**: JSON lines on stdout. The `VIP` lines tell each start: `LibreCN started`, `UDM udm ready, 1 subscriber(s)`, `UPF upf ready, N3 192.168.1.10:2152, N6 upf0`, `AMF amf-1 ready: N2 on ...`. A gNB's NG Setup and each UE's registration are logged at `info`.
- **GUI**: http://192.168.1.10:8080. It shows the running functions, workers, alarms and subscribers.
- **Alarms**: a lost gNB raises one (`N2 to ... lost`), and so does a failing function.
- **Metrics**: http://192.168.1.10:8080/metrics, for Prometheus. It includes per-function queues and processing times, and the UPF's packets and bytes.

## 8. Docker

The image in [`docker/`](docker/) holds LibreCN together with UERANSIM, for the user-plane test only. There is no LibreCN-only image yet. To run LibreCN in a container today, use the host's network (`--network host`), because the N3 address must be reachable by the gNB (§3). Also give it `--cap-add NET_ADMIN --device /dev/net/tun`, with the TUN device prepared as in §4.
