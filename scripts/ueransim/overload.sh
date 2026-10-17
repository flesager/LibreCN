#!/usr/bin/env sh
# Overload check with UERANSIM: an AMF with room for 2 UE contexts and 3
# UEs. The second UE fills the table (over 90 %): the gNB gets Overload
# Start; the AMF drops the third UE's Initial UE Message. The two UEs then
# switch off: under 80 %, the gNB gets Overload Stop. UERANSIM's gNB only
# logs both (it acts on neither). UERANSIM is expected next to this
# repository: ../UERANSIM/build (override with UERANSIM=).
set -eu
cd "$(dirname "$0")/../.."
bin=${UERANSIM:-../UERANSIM/build}
[ -x "$bin/nr-gnb" ] && [ -x "$bin/nr-ue" ] && [ -x "$bin/nr-cli" ] || { echo "no UERANSIM in $bin"; exit 1; }
out=$(mktemp -d)
trap 'kill $core $gnb $ue 2>/dev/null || true; rm -rf "$out"' EXIT

wait_for() { for _ in $(seq 100); do grep -q "$2" "$1" && return 0; sleep 0.1; done; return 1; }
wait_count() { for _ in $(seq 100); do [ "$(grep -c "$2" "$1")" -ge "$3" ] && return 0; sleep 0.1; done; return 1; }

# The registration check's core, with 3 subscribers and room for 2 UEs.
sed -e 's/^opc = "E8ED289DEBA952E4283B54E88E6183CA"$/&\ncount = 3/' \
    -e 's/^network_name = .*$/&\nmax_ues = 2/' scripts/ueransim/librecn.toml > "$out/core.toml"
grep -q "max_ues = 2" "$out/core.toml" && grep -q "count = 3" "$out/core.toml" || { echo "core config not derived"; exit 1; }

cargo build -q --release -p librecn
target/release/librecn --config "$out/core.toml" > "$out/core.log" 2>&1 & core=$!
wait_for "$out/core.log" "AMF amf-1 ready"
"$bin/nr-gnb" -c scripts/ueransim/gnb.yaml > "$out/gnb.log" 2>&1 & gnb=$!
wait_for "$out/gnb.log" "NG Setup procedure is successful"
"$bin/nr-ue" -c scripts/ueransim/ue.yaml -n 3 > "$out/ue.log" 2>&1 & ue=$!
wait_for "$out/gnb.log" "AMF overload start received" || true
wait_count "$out/core.log" "registered: imsi-99970000000000" 2 || true
for imsi in $(grep -o "registered: imsi-[0-9]*" "$out/core.log" | cut -d' ' -f2 | sort -u); do
  "$bin/nr-cli" "$imsi" -e "deregister switch-off" >> "$out/cli.log" 2>&1 || true
done
wait_for "$out/gnb.log" "AMF overload stop received" || true

for f in gnb ue cli core; do echo "--- $f"; cat "$out/$f.log"; done
grep -q "AMF overload start received" "$out/gnb.log" \
  && grep -q "overloaded: 2 of 2 UE contexts, Overload Start to 1 gNB(s)" "$out/core.log" \
  && grep -q "UE table full (2 UEs)" "$out/core.log" \
  && grep -q "AMF overload stop received" "$out/gnb.log" \
  && grep -q "overload over:" "$out/core.log" \
  && echo "OK: a full AMF told UERANSIM's gNB to hold back (Overload Start), then to resume (Overload Stop)" \
  || { echo "FAILED"; exit 1; }
