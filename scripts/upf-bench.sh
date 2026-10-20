#!/usr/bin/env sh
# The user plane of the whole core, as deployed (results in BENCHMARKS.md):
# a gNB with its UEs, LibreCN and a data network in three containers, linked
# by veth pairs (N3: gNB to core, N6: core to the data network), no bridge.
# LibreCN is the `librecn` binary on its configuration; the gNB and the data
# network are app/examples/load.rs's `gnb` and `dn` roles: the UEs register
# over N2, then traffic flows as fast as three senders on each side go.
#
# For each deployment (LibreCN on 1 core; on 2 cores in one pool; on 2 cores,
# the UPF's pool on one and the rest on the other) and each path (the UPF's
# fast path, or its fallback: the UPF alone), uplink, downlink and both ways,
# with 84- and 1400-byte packets: the highest rate carried without loss each
# way (RFC 2544's throughput: at most 1 % lost; flooded, the fallback carries
# less, the kernel's receive work starving the UPF) and how busy LibreCN's
# cores were then. The control plane is idle (registrations are
# measured by scripts/load.sh).
#
# The kernel's receive work for LibreCN's interfaces runs on LibreCN's cores
# (on the other one than the UPF's in the split deployment: on the UPF's it
# would starve the fallback while the other core idles): veth's NAPI, in a
# thread pinned there and in real time, as softirq runs ahead of threads;
# veth has that thread only with XDP, so the fallback runs with the fast
# path's program attached but carrying nothing (its cost: tens of ns per
# packet). The gNB's and data network's receive work runs on their own
# cores. Six senders on each side send 4 packets at a time when paced
# (short bursts, as from many UEs: veth's ring holds 256).
#
# Containers are privileged and see the host's processes (to link them and
# pin the kernel's threads): for this benchmark only. Cores (n and n + 16
# are SMT siblings here): LibreCN 2 and 3 (18 and 19 left idle); the gNB:
# its receive work 6, registration and draining 7, six senders on 8 to 10
# and 24 to 26; the data network: 11, 12, senders on 13 to 15 and 29 to 31.
#
#   scripts/upf-bench.sh            # UES=100 SECONDS=2 (per measure), about 15 minutes
#   ONLY="one true" LOG=info scripts/upf-bench.sh   # one deployment and path, the core's log kept
set -eu
cd "$(dirname "$0")/.."
cargo build -q --release -p librecn -p librecn --example load
ues=${UES:-100}
seconds=${SECONDS:-2}
dir=$(mktemp -d)
cp target/release/librecn target/release/examples/load nf/upf/bpf/upf.bpf.o "$dir"
printf 'FROM ubuntu:26.04\nRUN apt-get update && apt-get install -y --no-install-recommends ethtool iproute2 procps util-linux && rm -rf /var/lib/apt/lists/*\n' \
  | docker build -q -t librecn-bench - >/dev/null
names="librecn-bench-core librecn-bench-gnb librecn-bench-dn"
cleanup() {
  for c in $names; do docker rm -f "$c" >/dev/null 2>&1 || true; done
  rm -rf "$dir"
}
trap cleanup EXIT
for c in $names; do docker rm -f "$c" >/dev/null 2>&1 || true; done
for c in $names; do
  # The repository at its own path: load.rs reads its NG Setup vector there.
  docker run -d --name "$c" --privileged --pid=host --network none -v "$dir:/bench" -v "$PWD:$PWD:ro" \
    librecn-bench sleep infinity >/dev/null
done
pid() { docker inspect -f '{{.State.Pid}}' "$1"; }
core_pid=$(pid librecn-bench-core) gnb_pid=$(pid librecn-bench-gnb) dn_pid=$(pid librecn-bench-dn)
docker exec librecn-bench-core sh -c "
  ip link set lo up
  ip link add n3 type veth peer name n3g netns $gnb_pid
  ip link add n6 type veth peer name n6d netns $dn_pid
  ip addr add 10.212.0.10/24 dev n3 && ip link set n3 up
  ip addr add 10.213.0.10/24 dev n6 && ip link set n6 up
  ip tuntap add dev upf0 mode tun && ip addr add 10.45.0.1/16 dev upf0 && ip link set upf0 up
  sysctl -qw net.ipv4.ip_forward=1"
docker exec librecn-bench-gnb sh -c 'ip link set lo up; ip addr add 10.212.0.20/24 dev n3g && ip link set n3g up'
# The data network sends complete checksums, as the internet does (a
# container's are left to its interface: the fast path passes them to the UPF).
docker exec librecn-bench-dn sh -c 'ip link set lo up; ip addr add 10.213.0.30/24 dev n6d && ip link set n6d up
  ethtool -K n6d tx off >/dev/null
  ip route add 10.45.0.0/16 via 10.213.0.10'

# xdp CONTAINER DEVICE: the program attached, carrying nothing (no N3
# address): veth gets its NAPI.
xdp() { docker exec "$1" ip link set dev "$2" xdpdrv obj /bench/upf.bpf.o sec xdp; }
# napi CONTAINER DEVICE CORES: the device's receive work in a thread, on
# CORES, in real time (SCHED_FIFO), ahead of other threads as softirq is:
# as an ordinary thread it could wait a time slice (milliseconds) behind
# the UPF or another device's thread, and veth's ring holds 256 packets.
napi() {
  docker exec "$1" sh -c "echo 1 > /sys/class/net/$2/threaded
    for p in \$(pgrep '^napi/$2-'); do taskset -pc $3 \$p >/dev/null; chrt -f -p 50 \$p; done"
}
xdp librecn-bench-gnb n3g && napi librecn-bench-gnb n3g 6
xdp librecn-bench-dn n6d && napi librecn-bench-dn n6d 11
xdp librecn-bench-core n6

# config POOLS UPF_POOL FAST: LibreCN's configuration.
config() {
  cat <<TOML
$1
[oam]
listen = "127.0.0.1:8080"
log_level = "${LOG:-error}"
[[nf]]
kind = "udm"
name = "udm"
pool = "cp"
[[nf.params.subscribers]]
supi = "imsi-208930000000001"
count = $ues
k = "465B5CE8B199B49FAA5F0A2EE238A6BC"
opc = "E8ED289DEBA952E4283B54E88E6183CA"
[[nf]]
kind = "ausf"
name = "ausf"
pool = "cp"
[[nf]]
kind = "upf"
name = "upf"
pool = "$2"
[nf.params]
n3 = "10.212.0.10"
n6 = "upf0"
max_sessions = $ues
fast_path = $3
[[nf]]
kind = "pcf"
name = "pcf"
pool = "cp"
[[nf.params.default.sessions]]
ambr_mbps = { uplink = 100000, downlink = 100000 }   # no limit here
[[nf]]
kind = "smf"
name = "smf"
pool = "cp"
[[nf]]
kind = "amf"
name = "amf-1"
pool = "cp"
[nf.params]
n2 = "10.212.0.10:38412"
plmn = { mcc = "208", mnc = "93" }
max_ues = $((ues * 2))
TOML
}
one='[[runtime.pools]]
name = "cp"
cores = [2]
[[runtime.pools]]
name = "oam"
cores = [2]'
two='[[runtime.pools]]
name = "cp"
cores = [2, 3]
[[runtime.pools]]
name = "oam"
cores = [2]'
split='[[runtime.pools]]
name = "cp"
cores = [2]
[[runtime.pools]]
name = "up"
cores = [3]
[[runtime.pools]]
name = "oam"
cores = [2]'

# snapshot: packets into the data network (uplink carried) and into the
# gNB (downlink carried), out of the gNB (uplink offered) and of the data
# network (downlink offered: sent, or dropped by veth when its peer's ring
# is full, before anything counts them); busy and total ticks of cores 2
# and 3. Read together, from the host.
snapshot() {
  awk '/^ *n6d:/ { sub(/.*:/, ""); up = $2; down_tx = $10 + $12 } /^ *n3g:/ { sub(/.*:/, ""); down = $2; up_tx = $10 + $12 }
       /^cpu[23] / { b = 0; t = 0; for (i = 2; i <= NF; i++) { t += $i; if (i != 5 && i != 6) b += $i } c[$1] = b " " t }
       END { print up, down, up_tx, down_tx, c["cpu2"], c["cpu3"] }' "/proc/$dn_pid/net/dev" "/proc/$gnb_pid/net/dev" /proc/stat
}
# measure TRAFFIC SIZE RATE: uplink offered and carried, downlink offered
# and carried (kpps), cores 2 and 3 busy (%), over $seconds at RATE packets
# per second each way (0: as fast as the senders go).
measure() {
  echo "$1 $2 $3" > "$dir/ctl"
  sleep 0.5
  before=$(snapshot)
  sleep "$seconds"
  echo "$before $(snapshot)" | awk -v s="$seconds" '{
    printf "%.1f %.1f %.1f %.1f %.0f %.0f\n", ($11 - $3) / s / 1e3, ($9 - $1) / s / 1e3, ($12 - $4) / s / 1e3,
      ($10 - $2) / s / 1e3, ($13 - $5) * 100 / ($14 - $6), ($15 - $7) * 100 / ($16 - $8) }'
}
# lossless MEASURE TRAFFIC RATE: whether each way that runs lost at most 1 %
# and its senders gave the rate (within 3 %).
lossless() {
  echo "$1" | awk -v t="$2" -v r="$3" '{
    ok = 1
    if (t != "down") ok = ok && $2 >= 0.99 * $1 && (r == 0 || $1 >= 0.97 * r / 1e3)
    if (t != "up") ok = ok && $4 >= 0.99 * $3 && (r == 0 || $3 >= 0.97 * r / 1e3)
    exit !ok }'
}
# capacity TRAFFIC SIZE: the highest rate each way carried without loss
# (RFC 2544's throughput): as fast as the senders go if that is lossless
# ("at least": the senders are the limit), else a bisection below it.
capacity() {
  m=$(measure "$1" "$2" 0)
  [ -z "${DEBUG:-}" ] || echo "  $1 $2 flat out: $m" >&2
  if lossless "$m" "$1" 0; then
    echo "$m senders"
    return
  fi
  lo=0
  hi=$(echo "$m" | awk -v t="$1" '{ up = $1; down = $3; r = t == "up" ? up : t == "down" ? down : (up < down ? up : down); print int(r * 1e3) }')
  best="0 0 0 0 0 0"
  for _ in 1 2 3 4 5 6 7; do
    r=$(( (lo + hi) / 2 ))
    m=$(measure "$1" "$2" "$r")
    [ -z "${DEBUG:-}" ] || echo "  $1 $2 at $r: $m, UDP receive buffer errors $(awk '/^Udp: [0-9]/ { print $6 }' "/proc/$core_pid/net/snmp")" >&2
    if lossless "$m" "$1" "$r"; then lo=$r best=$m; else hi=$r; fi
  done
  echo "$best"
}
# row DEPLOYMENT PATH TRAFFIC SIZE CORES CAPACITY: a line of the table.
row() {
  echo "$6" | awk -v d="$1" -v p="$2" -v t="$3" -v size="$4" -v cores="$5" '{
    at = $7 == "senders" ? "≥ " : ""
    way = t == "up" ? "uplink" : t == "down" ? "downlink" : "both ways"
    up = t == "down" ? "-" : sprintf("%s%d kpps, %.2f Gbit/s", at, $2, $2 * size * 8 / 1e6)
    down = t == "up" ? "-" : sprintf("%s%d kpps, %.2f Gbit/s", at, $4, $4 * size * 8 / 1e6)
    busy = cores == 1 ? sprintf("%d %%", $5) : sprintf("%d %%, %d %%", $5, $6)
    printf "| %s | %s | %s | %d B | %s | %s | %s |\n", d, p, way, size, up, down, busy }'
}

echo "| deployment | path | traffic | packet | uplink | downlink | LibreCN's cores busy |"
echo "|---|---|---|---|---|---|---|"
deployments="one two split" paths="true false"
[ -z "${ONLY:-}" ] || deployments=${ONLY%% *} paths=${ONLY#* }
for deployment in $deployments; do
  case $deployment in
    one) pools=$one upf_pool=cp receive=2 cores=1 label="1 core" ;;
    two) pools=$two upf_pool=cp receive=2,3 cores=2 label="2 cores, one pool" ;;
    split) pools=$split upf_pool=up receive=${SPLIT_RECEIVE:-2} cores=2 label="2 cores: UPF, the rest" ;;
  esac
  for fast in $paths; do
    config "$pools" "$upf_pool" "$fast" > "$dir/core.toml"
    echo idle > "$dir/ctl"
    rm -f "$dir/ctl.ues"
    docker exec -d librecn-bench-core sh -c '/bench/librecn --config /bench/core.toml > /bench/core.log 2>&1'
    for _ in $(seq 100); do grep -q "AMF amf-1 ready" "$dir/core.log" 2>/dev/null && break; sleep 0.1; done
    if [ "$fast" = true ]; then
      for _ in $(seq 50); do grep -q "fast path on n3" "$dir/core.log" && break; sleep 0.1; done
      grep -q "fast path on n3" "$dir/core.log" || { echo "no fast path:"; cat "$dir/core.log"; exit 1; }
      path="fast path"
    else
      xdp librecn-bench-core n3
      path="fallback"
    fi
    napi librecn-bench-core n3 "$receive"
    napi librecn-bench-core n6 "$receive"
    docker exec -d librecn-bench-dn sh -c '/bench/load dn 10.213.0.30 /bench/ctl 12 13,14,15,29,30,31 > /bench/dn.log 2>&1'
    docker exec -d librecn-bench-gnb sh -c "/bench/load gnb 10.212.0.10:38412 10.212.0.20 10.213.0.30 $ues /bench/ctl 7 8,9,10,24,25,26 > /bench/gnb.log 2>&1"
    for _ in $(seq 100); do grep -q registered "$dir/gnb.log" 2>/dev/null && break; sleep 0.1; done
    grep -q "registered $ues UEs" "$dir/gnb.log" || { echo "registration failed:"; cat "$dir/gnb.log" "$dir/core.log"; exit 1; }
    for traffic in ${TRAFFIC:-up down both}; do
      for size in ${SIZES:-84 1400}; do
        row "$label" "$path" "$traffic" "$size" "$cores" "$(capacity "$traffic" "$size")"
      done
    done
    echo quit > "$dir/ctl"
    sleep 0.3
    docker exec librecn-bench-core pkill -TERM -f /bench/librecn || true
    for _ in $(seq 50); do docker exec librecn-bench-core pgrep -f /bench/librecn >/dev/null || break; sleep 0.1; done
    [ "$fast" = true ] || docker exec librecn-bench-core ip link set dev n3 xdpdrv off
    [ -z "${ONLY:-}" ] || cp "$dir/core.log" "${TMPDIR:-/tmp}/upf-bench-core.log"
  done
done
