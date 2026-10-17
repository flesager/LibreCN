#!/usr/bin/env sh
# Scale check with UERANSIM: GNBS gNBs (default 4), each with UES UEs
# (default 250, one nr-ue process each), all registering with 5G AKA and
# getting their PDU session. The UDM knows GNBS x UES subscribers (one
# range). Prints how many registered and got a session, and how long it took.
# UERANSIM is expected next to this repository: ../UERANSIM/build.
set -eu
cd "$(dirname "$0")/../.."
gnbs=${GNBS:-4}
ues=${UES:-250}
total=$((gnbs * ues))
bin=${UERANSIM:-../UERANSIM/build}
out=$(mktemp -d)
pids=""
trap 'kill $pids 2>/dev/null || true; [ -n "${KEEP:-}" ] && echo "logs in $out" || rm -rf "$out"' EXIT

cat > "$out/core.toml" <<TOML
[[runtime.pools]]
name = "cp"
workers = 2
[oam]
listen = "127.0.0.1:0"
[[nf]]
kind = "udm"
name = "udm"
[[nf.params.subscribers]]
supi = "imsi-999700000000001"
count = $total
k = "465B5CE8B199B49FAA5F0A2EE238A6BC"
opc = "E8ED289DEBA952E4283B54E88E6183CA"
[[nf]]
kind = "ausf"
name = "ausf"
[[nf]]
kind = "smf"
name = "smf"
[[nf]]
kind = "upf"
name = "upf"
[nf.params]
max_sessions = $total
[[nf]]
kind = "amf"
name = "amf-1"
[nf.params]
max_ues = $((total + 16))
TOML

cargo build -q --release -p librecn
target/release/librecn --config "$out/core.toml" > "$out/core.log" 2>&1 & pids="$!"
for _ in $(seq 50); do grep -q "AMF amf-1 ready" "$out/core.log" && break; sleep 0.1; done

started=$(date +%s.%N)
i=0
while [ "$i" -lt "$gnbs" ]; do
  ip="127.0.0.$((11 + i))"
  sed -e "s/^nci: .*/nci: '0x$(printf %09x $((16 * (i + 1))))'/" \
      -e "s/^linkIp: .*/linkIp: $ip/" -e "s/^ngapIp: .*/ngapIp: $ip/" -e "s/^gtpIp: .*/gtpIp: $ip/" \
      scripts/ueransim/gnb.yaml > "$out/gnb$i.yaml"
  "$bin/nr-gnb" -c "$out/gnb$i.yaml" > "$out/gnb$i.log" 2>&1 & pids="$pids $!"
  first=$(printf "%015d" $((999700000000001 + i * ues)))
  sed -e "s/^supi: .*/supi: 'imsi-$first'/" -e "s/^  - 127.0.0.1/  - $ip/" scripts/ueransim/ue.yaml > "$out/ue$i.yaml"
  i=$((i + 1))
done
sleep 1
i=0
while [ "$i" -lt "$gnbs" ]; do
  "$bin/nr-ue" -c "$out/ue$i.yaml" -n "$ues" > "$out/ue$i.log" 2>&1 & pids="$pids $!"
  i=$((i + 1))
done

for _ in $(seq 600); do
  [ "$(grep -c "PDU session 1 set up" "$out/core.log")" -ge "$total" ] && break
  sleep 0.1
done
elapsed=$(echo "$(date +%s.%N) - $started - 1" | bc)
gnb_up=$(grep -c "NG Setup accepted" "$out/core.log" || true)
registered=$(grep -c "registered: imsi-" "$out/core.log" || true)
sessions=$(grep -c "PDU session 1 set up" "$out/core.log" || true)
ue_ok=$(cat "$out"/ue*.log | grep -c "PDU Session establishment is successful" || true)
errors=$(grep -c '"level":"ERROR"' "$out/core.log" || true)
echo "gNBs set up: $gnb_up/$gnbs; UEs registered: $registered/$total; PDU sessions: $sessions (UERANSIM: $ue_ok); \
core errors: $errors; ${elapsed}s from the first UE"
[ "$registered" -eq "$total" ] && [ "$ue_ok" -eq "$total" ] && echo "OK" || { echo "FAILED"; exit 1; }
