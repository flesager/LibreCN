#!/usr/bin/env sh
# Interop check with UERANSIM: LibreCN's AMF accepts the NG Setup of
# UERANSIM's gNB. UERANSIM is expected next to this repository, built by its
# build.sh: ../UERANSIM/build/nr-gnb (override with UERANSIM=/path/to/build).
set -eu
cd "$(dirname "$0")/../.."
gnb=${UERANSIM:-../UERANSIM/build}/nr-gnb
[ -x "$gnb" ] || { echo "no UERANSIM gNB at $gnb"; exit 1; }
out=$(mktemp -d)
trap 'kill $core $ran 2>/dev/null || true; rm -rf "$out"' EXIT

cargo build -q --release -p librecn
target/release/librecn --config scripts/ueransim/librecn.toml > "$out/core.log" 2>&1 & core=$!
for _ in $(seq 50); do grep -q "AMF amf-1 ready" "$out/core.log" && break; sleep 0.1; done
"$gnb" -c scripts/ueransim/gnb.yaml > "$out/gnb.log" 2>&1 & ran=$!
for _ in $(seq 50); do grep -q "NG Setup procedure is successful" "$out/gnb.log" && break; sleep 0.1; done
sleep 0.5

echo "--- gNB (UERANSIM)"; cat "$out/gnb.log"
echo "--- core (LibreCN)"; cat "$out/core.log"
grep -q "NG Setup procedure is successful" "$out/gnb.log" && grep -q "NG Setup accepted" "$out/core.log" \
  && echo "OK: NG Setup between UERANSIM and LibreCN" || { echo "FAILED"; exit 1; }
