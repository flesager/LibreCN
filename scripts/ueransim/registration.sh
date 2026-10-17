#!/usr/bin/env sh
# Interop check with UERANSIM: NG Setup, then UERANSIM's UE registers and
# LibreCN's AMF answers through N2 (Initial UE Message, Downlink NAS
# Transport carrying the NAS answer, UE Context Release). UERANSIM is
# expected next to this repository: ../UERANSIM/build (override with UERANSIM=).
set -eu
cd "$(dirname "$0")/../.."
bin=${UERANSIM:-../UERANSIM/build}
[ -x "$bin/nr-gnb" ] && [ -x "$bin/nr-ue" ] || { echo "no UERANSIM in $bin"; exit 1; }
out=$(mktemp -d)
trap 'kill $core $gnb $ue 2>/dev/null || true; rm -rf "$out"' EXIT

wait_for() { for _ in $(seq 50); do grep -q "$2" "$1" && return 0; sleep 0.1; done; return 1; }

cargo build -q --release -p librecn
target/release/librecn --config scripts/ueransim/librecn.toml > "$out/core.log" 2>&1 & core=$!
wait_for "$out/core.log" "AMF amf-1 ready"
"$bin/nr-gnb" -c scripts/ueransim/gnb.yaml > "$out/gnb.log" 2>&1 & gnb=$!
wait_for "$out/gnb.log" "NG Setup procedure is successful"
"$bin/nr-ue" -c scripts/ueransim/ue.yaml > "$out/ue.log" 2>&1 & ue=$!
wait_for "$out/core.log" "released, 0 UE(s) left" || true
sleep 0.5

for f in gnb ue core; do echo "--- $f"; cat "$out/$f.log"; done
grep -q "Initial UE Message\|initial 5GMM Registration Request" "$out/core.log" \
  && grep -q "downlink 5GMM Registration Reject" "$out/core.log" \
  && grep -q "released, 0 UE(s) left" "$out/core.log" \
  && echo "OK: UE signalling over N2 between UERANSIM and LibreCN" || { echo "FAILED"; exit 1; }
