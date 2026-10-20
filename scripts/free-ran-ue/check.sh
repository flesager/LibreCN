#!/usr/bin/env sh
# Interop check with free-ran-ue (a gNB and UE simulator in Go, built on
# free5GC's NGAP and NAS libraries), in Docker: the core in one container,
# free-ran-ue's gNB and UEs in another (its UE needs root).
# 1. One UE: registration (5G AKA, NEA2/NIA2), its PDU session, a ping
#    through its ueTun0 and LibreCN's user plane (free-ran-ue uses PDU
#    session ID 4).
# 2. N UEs (default 1000), control plane only: all registered with their
#    session within 60 s, and no error in the core's log.
# free-ran-ue waits for a Configuration Update Command after registering and
# reads GSM 7-bit network names only: the core sends its names. Its slice
# has an SD (010203), which the core serves. The core runs without the
# rights for its fast path (BPF): the UPF says so and carries everything
# itself (the fallback), with a MINOR alarm and no error.
#
# free-ran-ue is expected next to this repository, built (no Go needed):
#   git clone https://github.com/free-ran-ue/free-ran-ue ../free-ran-ue
#   docker run --rm -v "$PWD/../free-ran-ue":/src -w /src golang:1.26 \
#     sh -c 'CGO_ENABLED=0 go build -o build/free-ran-ue main.go'
# (FREE_RAN_UE= another binary). The image is the UERANSIM checks' one:
# see scripts/ueransim/docker.sh for what is expected of the host.
#
#   scripts/free-ran-ue/check.sh          # 1 UE, then 1000
#   N=10000 scripts/free-ran-ue/check.sh
set -eu
cd "$(dirname "$0")/../.."
frue=$(readlink -f "${FREE_RAN_UE:-../free-ran-ue/build/free-ran-ue}" 2>/dev/null || true)
[ -x "$frue" ] || { echo "no free-ran-ue binary (see the header to build it)"; exit 1; }
n=${N:-1000}
. scripts/ueransim/docker.sh

# The configurations: the core serves N + 1 subscribers, the slice with its SD.
cfg="$ctx/frue"
mkdir -p "$cfg"
cat > "$cfg/core.toml" <<TOML
[[runtime.pools]]
name = "cp"
workers = 1
[[runtime.pools]]
name = "up"
workers = 1
[oam]
listen = "0.0.0.0:8080"
[[nf]]
kind = "udm"
name = "udm"
[[nf.params.subscribers]]
supi = "imsi-999700000000001"
count = $((n + 1))
k = "465B5CE8B199B49FAA5F0A2EE238A6BC"
opc = "E8ED289DEBA952E4283B54E88E6183CA"
[[nf]]
kind = "ausf"
name = "ausf"
[nf.params]
max_contexts = $((n + 64))
[[nf]]
kind = "upf"
name = "upf"
pool = "up"
[nf.params]
n3 = "10.210.0.10"
n6 = "upf0"
max_sessions = $((n + 64))
[[nf]]
kind = "pcf"
name = "pcf"
[[nf]]
kind = "smf"
name = "smf"
[nf.params]
slices = [{ sst = 1 }, { sst = 1, sd = "010203" }]
[[nf]]
kind = "amf"
name = "amf-1"
[nf.params]
n2 = "10.210.0.10:38412"
max_ues = $((n + n / 4 + 64))
slices = [{ sst = 1 }, { sst = 1, sd = "010203" }]
network_name = { full = "LibreCN", short = "LibreCN" }
TOML
cat > "$cfg/gnb.yaml" <<'YAML'
gnb:
  amfN2Ip: "10.210.0.10"
  ranN2Ip: "10.210.0.20"
  upfN3Ip: "10.210.0.10"
  ranN3Ip: "10.210.0.20"
  ranControlPlaneIp: "127.0.0.1"
  ranDataPlaneIp: "127.0.0.1"
  amfN2Port: 38412
  ranN2Port: 38413
  upfN3Port: 2152
  ranN3Port: 2152
  ranControlPlanePort: 31413
  ranDataPlanePort: 31414
  gnbId: "000314"
  gnbName: "free-ran-ue-gnb"
  plmnId: { mcc: "999", mnc: "70" }
  tai:
    tac: "000001"
    broadcastPlmnId: { mcc: "999", mnc: "70" }
  snssai: { sst: "1", sd: "010203" }
  api: { ip: "127.0.0.1", port: 40104 }
logger:
  level: "info"
YAML
# ue.yaml MSIN TUNNEL_SETUP_IGNORED
ue_yaml() {
  cat <<YAML
ue:
  ranControlPlaneIp: "127.0.0.1"
  ranDataPlaneIp: "127.0.0.1"
  ranControlPlanePort: 31413
  ranDataPlanePort: 31414
  plmnId: { mcc: "999", mnc: "70" }
  msin: "$1"
  authenticationSubscription:
    encPermanentKey: "465B5CE8B199B49FAA5F0A2EE238A6BC"
    encOpcKey: "E8ED289DEBA952E4283B54E88E6183CA"
    authenticationManagementField: "8000"
    sequenceNumber: "000000000000"
  integrityAlgorithm: { nia0: false, nia1: false, nia2: true, nia3: false }
  cipheringAlgorithm: { nea0: false, nea1: false, nea2: true, nea3: false }
  pduSession:
    dnn: "internet"
    snssai: { sst: "1", sd: "010203" }
  accessType: "3GPP_ACCESS"
  nrdc: { enable: false }
  ueTunnelDevice: "ueTun"
  ignoreSetupTunnel: $2
logger:
  level: "error"
YAML
}
ue_yaml 0000000001 false > "$cfg/ue-one.yaml"
ue_yaml 0000000002 true > "$cfg/ue-many.yaml"

opts="-v $cfg:/frue:ro"
fast_caps=""
run librecn-core 10.210.0.10 /etc/librecn/core.sh /frue/core.toml
opts="-v $frue:/usr/local/bin/free-ran-ue:ro -v $cfg:/frue:ro"
run librecn-frue 10.210.0.20 sleep infinity
opts=""
wait_for librecn-core "AMF amf-1 ready"
docker exec -d librecn-frue sh -c 'free-ran-ue gnb -c /frue/gnb.yaml > /tmp/gnb.log 2>&1'
wait_for librecn-core "NG Setup accepted: free-ran-ue-gnb" || true

# sessions: the SMF's count.
sessions() {
  c=$(docker exec librecn-core curl -s http://127.0.0.1:8080/api/nfs/smf/sessions | grep -o '"count":[0-9]*' | cut -d: -f2)
  echo "${c:-0}"
}
# until_sessions N SECONDS: until the SMF holds N sessions; the seconds taken.
until_sessions() {
  t=0
  while [ "$t" -lt "$2" ] && [ "$(sessions)" -lt "$1" ]; do
    timeout 1 tail -f /dev/null || true
    t=$((t + 1))
  done
  echo "$t"
}

# 1. One UE, with its tunnel: a ping through the user plane.
docker exec -d librecn-frue sh -c 'free-ran-ue ue -c /frue/ue-one.yaml > /tmp/ue-one.log 2>&1'
until_sessions 1 30 > /dev/null
sleep 1
ping=$(docker exec librecn-frue ping -I ueTun0 -c 3 -W 2 10.45.0.1 2>&1 || true)
docker exec librecn-frue pkill -INT -f "ue-one.yaml" || true
for _ in $(seq 50); do [ "$(sessions)" -eq 0 ] && break; timeout 0.2 tail -f /dev/null || true; done

# 2. N UEs, control plane only.
docker exec -d librecn-frue sh -c "free-ran-ue ue -c /frue/ue-many.yaml -n $n -p 100 > /tmp/ue-many.log 2>&1"
took=$(until_sessions "$n" 60)
held=$(sessions)
docker stop -t 5 librecn-core >/dev/null

core=$(docker logs librecn-core 2>&1)
errors=$(echo "$core" | grep -cE '"level":"(ERROR|FATAL)"' || true)
echo "--- gnb"; docker exec librecn-frue tail -20 /tmp/gnb.log
echo "--- ue (one)"; docker exec librecn-frue tail -20 /tmp/ue-one.log
echo "--- ue ($n)"; docker exec librecn-frue tail -20 /tmp/ue-many.log
echo "--- core errors"; echo "$core" | grep -E '"level":"(ERROR|FATAL)"' | head -20
echo "--- ping"; echo "$ping"
echo "--- $n UEs: $held sessions after ${took} s, $errors error(s) in the core"
echo "$core" | grep -q "registered: imsi-999700000000001" \
  && echo "$core" | grep -q "imsi-999700000000001: PDU session 4 to internet" \
  && echo "$ping" | grep -q " 0% packet loss" \
  && [ "$held" -ge "$n" ] && [ "$errors" -eq 0 ] \
  && echo "$core" | grep -q "fast path unavailable" \
  && echo "OK: free-ran-ue's UE registered, got its PDU session and pinged through LibreCN; $n UEs registered with their session in ${took} s" \
  || { echo "FAILED"; exit 1; }
