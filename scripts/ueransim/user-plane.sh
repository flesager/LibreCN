#!/usr/bin/env sh
# User-plane interop check with UERANSIM, in Docker (UERANSIM's UE needs
# root for uesimtun0): the core in one container, UERANSIM's gNB and UE in
# another. The UE registers, gets its PDU session, then pings the UPF's N6
# address (10.45.0.1) through uesimtun0: GTP-U on N3, upf0 on N6. The gNB
# then releases the UE (idle): the next ping brings it back with a Service
# Request, its session reactivated (UERANSIM may drop the packet that woke
# it: one answer at least is expected). Released again, the UE is then
# pinged from the data network (the core's container, through upf0): the
# UPF parks the packets and reports them, the AMF pages the UE, whose
# Service Request brings its session back with the parked packets.
# UERANSIM is expected next to this repository: ../UERANSIM/build
# (override with UERANSIM=), with patches/0001 applied. Needs docker and the
# host's sctp module.
set -eu
cd "$(dirname "$0")/../.."
bin=${UERANSIM:-../UERANSIM/build}
[ -x "$bin/nr-gnb" ] && [ -x "$bin/nr-ue" ] && [ -x "$bin/nr-cli" ] || { echo "no UERANSIM in $bin"; exit 1; }
patch=scripts/ueransim/patches/0001-gnb-select-an-AMF-for-protected-initial-messages.patch
git -C "$bin/../src" apply --check -R "$PWD/$patch" 2>/dev/null \
  || { echo "UERANSIM's gNB lacks $patch: git -C $bin/../src apply, then rebuild"; exit 1; }
image=librecn-interop
net=librecn-interop
ctx=$(mktemp -d)
remove() {
  docker rm -f librecn-core librecn-ran >/dev/null 2>&1 || true
  docker network rm "$net" >/dev/null 2>&1 || true
}
trap 'remove; rm -rf "$ctx"' EXIT
remove

cargo build -q --release -p librecn
cp docker/* target/release/librecn "$bin/nr-gnb" "$bin/nr-ue" "$bin/nr-cli" "$ctx"
docker build -q -t "$image" "$ctx" >/dev/null
docker network create --subnet 10.210.0.0/24 "$net" >/dev/null
# run NAME IP COMMAND...: the command is PID 1 (LibreCN must get SIGTERM).
run() {
  name=$1 ip=$2; shift 2
  docker run -d --name "$name" --network "$net" --ip "$ip" --cap-add NET_ADMIN --device /dev/net/tun "$image" "$@" >/dev/null
}
run librecn-core 10.210.0.10 /etc/librecn/core.sh
wait_for() { for _ in $(seq 100); do docker logs "$1" 2>&1 | grep -q "$2" && return 0; sleep 0.1; done; return 1; }
# wait_count CONTAINER PATTERN N: until PATTERN appears N times in its logs.
wait_count() {
  for _ in $(seq 100); do [ "$(docker logs "$1" 2>&1 | grep -c "$2")" -ge "$3" ] && return 0; sleep 0.1; done; return 1
}
# release: the gNB releases the UE (it goes idle), whatever its id at the gNB.
release() {
  id=$(docker exec librecn-ran nr-cli UERANSIM-gnb-999-70-1 -e ue-list | grep -o 'ue-id: [0-9]*' | head -1 | cut -d' ' -f2)
  docker exec librecn-ran nr-cli UERANSIM-gnb-999-70-1 -e "ue-release ${id:-1}" >/dev/null || true
}
wait_for librecn-core "AMF amf-1 ready"
run librecn-ran 10.210.0.20 sh -c 'nr-gnb -c /etc/librecn/gnb.yaml > /tmp/gnb.log 2>&1 & sleep 1; exec nr-ue -c /etc/librecn/ue.yaml'
wait_for librecn-ran "TUN interface\[uesimtun0" || true

ping=$(docker exec librecn-ran ping -I uesimtun0 -c 3 -W 2 10.45.0.1 2>&1 || true)
release
wait_for librecn-core "deactivated (the UE is idle)" || true
idle_ping=$(docker exec librecn-ran ping -I uesimtun0 -c 3 -W 2 10.45.0.1 2>&1 || true)
release
wait_count librecn-core "deactivated (the UE is idle)" 2 || true
paged_ping=$(docker exec librecn-core ping -c 3 -W 2 10.45.0.2 2>&1 || true)
metrics=$(docker exec librecn-core curl -s http://127.0.0.1:8080/metrics | grep 'eo="upf"' | grep counter_total || true)
docker stop -t 5 librecn-core >/dev/null

echo "--- gnb"; docker exec librecn-ran cat /tmp/gnb.log 2>/dev/null || docker cp librecn-ran:/tmp/gnb.log - 2>/dev/null || true
echo "--- ue"; docker logs librecn-ran 2>&1
echo "--- core"; docker logs librecn-core 2>&1
echo "--- ping"; echo "$ping"
echo "--- ping after idle"; echo "$idle_ping"
echo "--- ping of the idle UE (paging)"; echo "$paged_ping"
echo "--- UPF counters"; echo "$metrics"
core=$(docker logs librecn-core 2>&1)
echo "$ping" | grep -q " 0% packet loss" \
  && echo "$idle_ping" | grep -Eq " [1-9][0-9]* received" \
  && echo "$core" | grep -q "resumed after" \
  && echo "$core" | grep -q "PDU session 1 reactivated" \
  && echo "$paged_ping" | grep -Eq " [1-9][0-9]* received" \
  && echo "$core" | grep -q "paged for PDU session 1" \
  && echo "$metrics" | grep -Eq 'counter="downlink_packets"} [1-9]' \
  && echo "OK: UERANSIM's UE pinged through LibreCN's user plane, before and after idle; pinged while idle: paged" \
  || { echo "FAILED"; exit 1; }
