#!/usr/bin/env sh
# Interop check with UERANSIM: NG Setup, then UERANSIM's UE registers with
# LibreCN (AMF, AUSF, UDM): 5G AKA, NAS security mode, Registration Accept
# in Initial Context Setup and Complete; it then establishes its PDU session
# (SMF, UPF, PDU Session Resource Setup). Without root, the UE cannot create
# its TUN interface: the user plane is not checked here. The UE releases its
# session (nr-cli ps-release) and UERANSIM sets it up again; the network then
# releases it (DELETE on the SMF's web API). The network then deregisters the
# UE, re-registration required (DELETE on the AMF's web API): it registers
# again. The UE finally switches off: Deregistration Request, UE Context
# Release. UERANSIM is expected next to this repository:
# ../UERANSIM/build (override with UERANSIM=).
set -eu
cd "$(dirname "$0")/../.."
bin=${UERANSIM:-../UERANSIM/build}
[ -x "$bin/nr-gnb" ] && [ -x "$bin/nr-ue" ] && [ -x "$bin/nr-cli" ] || { echo "no UERANSIM in $bin"; exit 1; }
out=$(mktemp -d)
trap 'kill $core $gnb $ue 2>/dev/null || true; rm -rf "$out"' EXIT

wait_for() { for _ in $(seq 50); do grep -q "$2" "$1" && return 0; sleep 0.1; done; return 1; }
# wait_count FILE PATTERN N: until PATTERN appears N times.
wait_count() { for _ in $(seq 50); do [ "$(grep -c "$2" "$1")" -ge "$3" ] && return 0; sleep 0.1; done; return 1; }

cargo build -q --release -p librecn
target/release/librecn --config scripts/ueransim/librecn.toml > "$out/core.log" 2>&1 & core=$!
wait_for "$out/core.log" "AMF amf-1 ready"
"$bin/nr-gnb" -c scripts/ueransim/gnb.yaml > "$out/gnb.log" 2>&1 & gnb=$!
wait_for "$out/gnb.log" "NG Setup procedure is successful"
"$bin/nr-ue" -c scripts/ueransim/ue.yaml > "$out/ue.log" 2>&1 & ue=$!
wait_for "$out/core.log" "PDU session 1 set up" || true
cli() { "$bin/nr-cli" imsi-999700000000001 -e "$1" >> "$out/cli.log" 2>&1 || true; }
cli "ps-release 1"
wait_count "$out/core.log" "PDU session 1 set up" 2 || true
web=$(grep -o 'http://127.0.0.1:[0-9]*' "$out/core.log" | head -1)
id=$(curl -s "$web/api/nfs/smf/sessions" | grep -o '"id":[0-9]*' | head -1 | cut -d: -f2)
curl -s -X DELETE "$web/api/nfs/smf/sessions/$id" >> "$out/cli.log"
wait_count "$out/core.log" "PDU session 1 released\$" 2 || true
sleep 0.5
curl -s -X DELETE "$web/api/nfs/amf-1/ues/imsi-999700000000001" -d '{"reregister": true}' >> "$out/cli.log"
wait_count "$out/core.log" "registered: imsi-999700000000001" 2 || true
wait_count "$out/core.log" "PDU session 1 set up" 3 || true
sleep 0.5
cli "deregister switch-off"
wait_for "$out/core.log" "released, 0 UE(s) left" || true

for f in gnb ue cli core; do echo "--- $f"; cat "$out/$f.log"; done
grep -q "Initial Registration is successful" "$out/ue.log" \
  && grep -q "registered: imsi-999700000000001" "$out/core.log" \
  && grep -q "PDU Session establishment is successful PSI\[1\]" "$out/ue.log" \
  && [ "$(grep -c "UE 1: PDU session 1 set up" "$out/core.log")" -ge 2 ] \
  && grep -q "PDU session 1: PDU Session Release Request" "$out/core.log" \
  && grep -q "PDU session 1 released by the network" "$out/core.log" \
  && [ "$(grep -c "UE 1: PDU session 1 released\"" "$out/core.log")" -eq 2 ] \
  && grep -q "deregistered by the network\"" "$out/core.log" \
  && [ "$(grep -c "registered: imsi-999700000000001" "$out/core.log")" -ge 2 ] \
  && grep -q "deregistered (switch off)" "$out/core.log" \
  && grep -q "released, 0 UE(s) left" "$out/core.log" \
  && echo "OK: UERANSIM's UE registered, got, released (UE, network) and got again its PDU session, deregistered by the network then by itself" \
  || { echo "FAILED"; exit 1; }
