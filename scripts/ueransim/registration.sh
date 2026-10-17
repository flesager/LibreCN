#!/usr/bin/env sh
# Interop check with UERANSIM: NG Setup, then UERANSIM's UE registers with
# LibreCN (AMF, AUSF, UDM): 5G AKA, NAS security mode, Registration Accept
# in Initial Context Setup and Complete; it then establishes its PDU session
# (SMF, UPF, PDU Session Resource Setup). Without root, the UE cannot create
# its TUN interface: the user plane is not checked here. The UE then
# switches off (nr-cli): Deregistration Request, UE Context Release, the
# session released. UERANSIM is expected next to this repository:
# ../UERANSIM/build (override with UERANSIM=).
set -eu
cd "$(dirname "$0")/../.."
bin=${UERANSIM:-../UERANSIM/build}
[ -x "$bin/nr-gnb" ] && [ -x "$bin/nr-ue" ] && [ -x "$bin/nr-cli" ] || { echo "no UERANSIM in $bin"; exit 1; }
out=$(mktemp -d)
trap 'kill $core $gnb $ue 2>/dev/null || true; rm -rf "$out"' EXIT

wait_for() { for _ in $(seq 50); do grep -q "$2" "$1" && return 0; sleep 0.1; done; return 1; }

cargo build -q --release -p librecn
target/release/librecn --config scripts/ueransim/librecn.toml > "$out/core.log" 2>&1 & core=$!
wait_for "$out/core.log" "AMF amf-1 ready"
"$bin/nr-gnb" -c scripts/ueransim/gnb.yaml > "$out/gnb.log" 2>&1 & gnb=$!
wait_for "$out/gnb.log" "NG Setup procedure is successful"
"$bin/nr-ue" -c scripts/ueransim/ue.yaml > "$out/ue.log" 2>&1 & ue=$!
wait_for "$out/core.log" "PDU session 1 set up" || true
sleep 0.5
"$bin/nr-cli" imsi-999700000000001 -e "deregister switch-off" > "$out/cli.log" 2>&1 || true
wait_for "$out/core.log" "released, 0 UE(s) left" || true

for f in gnb ue cli core; do echo "--- $f"; cat "$out/$f.log"; done
grep -q "Initial Registration is successful" "$out/ue.log" \
  && grep -q "registered: imsi-999700000000001" "$out/core.log" \
  && grep -q "PDU Session establishment is successful PSI\[1\]" "$out/ue.log" \
  && grep -q "PDU session 1 set up" "$out/core.log" \
  && grep -q "PDU session 1 released, 0 left" "$out/core.log" \
  && grep -q "deregistered (switch off)" "$out/core.log" \
  && grep -q "released, 0 UE(s) left" "$out/core.log" \
  && echo "OK: UERANSIM's UE registered with LibreCN, got its PDU session, then deregistered" || { echo "FAILED"; exit 1; }
