# Running LibreCN

How to get LibreCN running on a machine, and connect it to a gNB and a data network.

**The quickest way is Docker**: one image, a `compose.yaml` and a `.env`, see [`deploy/README.md`](deploy/README.md). This page explains what is underneath, and running the binary yourself.

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
kind = "amf"              # amf, ausf, udm, pcf, smf, upf
name = "amf-1"            # its name, used by the others to find it
pool = "cp"               # optional: the first pool by default
[nf.params]               # its parameters (below)
n2 = "192.168.1.10:38412"
```

**The functions find each other by name.** The defaults match the names used here:

```
gNB ──N2──► amf-1 ──► ausf ──► udm      (authentication, subscribers)
              │  └──────► pcf ◄──┐      (policies: UE-AMBR, sessions' QoS)
              │                  │
              └─────► smf ───────┘
                       └────► upf ◄──N3── gNB
                                │
                                N6 (TUN) ──► data network
```

| Function | Points to | Parameters, default |
|---|---|---|
| AMF | the AUSF, the SMF and the PCF | `ausf = "ausf"`, `smf = "smf"`, `pcf = "pcf"` |
| AUSF | the UDM | `udm = "udm"` |
| SMF | the UPF and the PCF | `upf = "upf"`, `pcf = "pcf"` |

The PCF is required: without it, registrations go on with no UE-AMBR, but every PDU session is refused.

**Pools and cores**: the [README](README.md#capacity) tells how many cores: 2 for the full capacity. The OAM functions always run in a pool named `oam`, added with one worker if the file has none. They are light, so they can share a core. With the UPF's fast path (on when the kernel allows it: the UPF's ready line says `fast path on ...`), put every function in one pool pinned to the cores you give LibreCN, and the network card's interrupts on the same cores (the kernel carries the traffic there):

```toml
[[runtime.pools]]
name = "cp"
cores = [2, 3]

[[runtime.pools]]
name = "oam"
cores = [2]
```

Without the fast path, give the UPF a core of its own (`pool = "up"` in its `[[nf]]`), and the card's interrupts the other one:

```toml
[[runtime.pools]]
name = "cp"
cores = [2]

[[runtime.pools]]
name = "up"
cores = [3]

[[runtime.pools]]
name = "oam"
cores = [2]
```

The card's interrupts: `grep <card> /proc/interrupts` lists them, `echo 2,3 | sudo tee /proc/irq/<n>/smp_affinity_list` places one (`irqbalance`, if it runs, spreads them over every core).

**GRO** on the card the internet's traffic comes in by aggregates TCP, which the fast path cannot tunnel: TCP downloads would go through the UPF (the UPF raises a WARNING alarm when GRO is on for its N3 interface). Turn it off: `sudo ethtool -K <card> gro off` (the Docker deployment does it while the core runs).

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

**QoS and bit rates** come from the PCF, per subscriber, with a `default` entry for the others (see `app/librecn.example.toml`): the UE-AMBR, and for each DNN the session AMBR, the default flow's 5QI and ARP, and extra QoS flows with their packet filters. What an entry leaves out comes from `default`, then from the built-in values: 5QI 9, ARP 8, 1000 Mbit/s each way. The GUI's Provisioning panel, or `POST /api/nfs/pcf/policies`, changes them while LibreCN runs. The UPF enforces the session AMBR and the flows' MBRs.

Subscribers can also be added while LibreCN runs: the GUI's Provisioning panel (one UE with its policy, or many from a JSON file), or `POST /api/nfs/udm/subscribers`. With `[oam] state_dir` set (for example `/var/lib/librecn`), they survive restarts:
- the UDM keeps what the web API changed (subscribers added and removed) and every subscriber's SQN in `<state_dir>/udm.json`, readable by its owner only (it holds keys);
- it is written at most every 5 s when something changed, and at stop;
- at start, the configuration is read first, then the state file on top of it; each SQN moves 1024 steps ahead, so none is used twice;
- the PCF does the same for policies (`POST /api/nfs/pcf/policies`) in `<state_dir>/pcf.json`.

Without `state_dir`, they live in memory only and are gone at the next start. A database will come later.

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
kind = "pcf"
name = "pcf"
[nf.params.default]
ue_ambr_mbps = { uplink = 100, downlink = 500 }
[[nf.params.default.sessions]]
ambr_mbps = { uplink = 50, downlink = 200 }

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
- **GUI**: http://192.168.1.10:8080. Its Overview draws your core as in TS 23.501, with the gNBs and UEs counted; the UEs panel lists gNBs, UEs (details for each) and PDU sessions with the user plane's throughput; Provisioning holds subscribers and policies (edited there as JSON); System the functions, workers, alarms and starting NFs. To see it busy without a real RAN: `scripts/ueransim/demo.sh` (Docker, UERANSIM).
- **Alarms**: a lost gNB raises one (`N2 to ... lost`), and so does a failing function.
- **Metrics**: http://192.168.1.10:8080/metrics, for Prometheus. It includes per-function queues and processing times, and the UPF's packets and bytes.

## 8. Docker

LibreCN's image, `ghcr.io/flesager/librecn` (amd64 and arm64), runs with Docker Compose: see [`deploy/README.md`](deploy/README.md). It shares the host's network (N2 and N3 must be reachable by the gNB on the host's address, §3), writes its configuration from a few `.env` settings (or takes yours), and prepares N6 itself (§4: `upf0`, NAT; GRO off for the fast path), undone at stop.

The image in [`docker/`](docker/) is another one: LibreCN with UERANSIM, for the interop checks only.
