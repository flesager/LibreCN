#!/usr/bin/env sh
# Interop check with UERANSIM: NG Setup, then UERANSIM's UE registers with
# LibreCN (AMF, AUSF, UDM), its SUCI concealed with ECIES Profile A and
# deciphered by the UDM: EAP-AKA' (the subscriber's method; patch 0003 lets
# UERANSIM's UE read AT_KDF), NAS security mode, Registration Accept
# in Initial Context Setup and Complete; it then establishes its PDU session
# (SMF, UPF, PDU Session Resource Setup; the PCF gives the UE-AMBR and the
# session AMBR of its subscriber, and a GBR voice flow: its QoS rule and
# flow description to the UE, its GBR information to the gNB), and a second
# one on the second DNN
# (ims; both in ue.yaml: on the host, UERANSIM's UE bars its own nr-cli
# ps-establish, its RRC thread missing the 100 ms UAC check). The UE gets the
# network's names, then a new 5G-GUTI (POST on the AMF's web API, a
# Configuration Update Command it acknowledges; nr-cli status shows it).
# The AMF then announces a new capacity to the gNB (AMF Configuration
# Update), and the PCF a new UE-AMBR (POST on its web API), which the AMF
# sets at the gNB (UE Context Modification).
# Without root, the UE cannot create
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
for patch in scripts/ueransim/patches/0003-*.patch scripts/ueransim/patches/0004-*.patch; do
  git -C "$bin/../src" apply --check -R "$PWD/$patch" 2>/dev/null \
    || { echo "UERANSIM lacks $patch: git -C $bin/../src apply, then rebuild"; exit 1; }
done
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
wait_for "$out/core.log" "PDU session 2 set up" || true
web=$(grep -o 'http://127.0.0.1:[0-9]*' "$out/core.log" | head -1)
# A new 5G-GUTI (Configuration Update Command, acknowledged).
curl -s -X POST "$web/api/nfs/amf-1/ues/imsi-999700000000001/guti" >> "$out/cli.log"
wait_for "$out/core.log" "5G-GUTI reallocated" || true
guti=$(grep -o "5G-GUTI reallocated: 5G-GUTI [0-9/]* [0-9:]* [0-9a-f]*" "$out/core.log" | head -1 | awk '{print $NF}')
"$bin/nr-cli" imsi-999700000000001 -e status > "$out/info.log" 2>&1 || true
# The AMF's capacity (AMF Configuration Update), the PCF's UE-AMBR (UE Context Modification).
curl -s -X POST "$web/api/nfs/amf-1/capacity" -d '{"relative_capacity": 100}' >> "$out/cli.log"
wait_for "$out/core.log" "AMF Configuration Update acknowledged" || true
curl -s -X POST "$web/api/nfs/pcf/policies" -d '{"supi": "imsi-999700000000001",
  "ue_ambr_mbps": {"uplink": 50, "downlink": 200},
  "sessions": [{"dnn": "internet", "ambr_mbps": {"uplink": 50, "downlink": 200}}]}' >> "$out/cli.log"
wait_for "$out/core.log" "UE Context Modification done" || true
cli "ps-release 1"
wait_count "$out/core.log" "PDU session 1 set up" 2 || true
id=$(curl -s "$web/api/nfs/smf/sessions" | tr '{' '\n' | grep '"pdu_session_id":1,' | grep -o '"id":[0-9]*' | cut -d: -f2)
curl -s -X DELETE "$web/api/nfs/smf/sessions/$id" >> "$out/cli.log"
wait_count "$out/core.log" "PDU session 1 released\$" 2 || true
sleep 0.5
curl -s -X DELETE "$web/api/nfs/amf-1/ues/imsi-999700000000001" -d '{"reregister": true}' >> "$out/cli.log"
wait_count "$out/core.log" "registered: imsi-999700000000001" 2 || true
wait_count "$out/core.log" "PDU session 1 set up" 3 || true
sleep 0.5
cli "deregister switch-off"
wait_for "$out/core.log" "released, 0 UE(s) left" || true

for f in gnb ue cli info core; do echo "--- $f"; cat "$out/$f.log"; done
grep -q "Initial Registration is successful" "$out/ue.log" \
  && grep -q "SUCI of Profile A" "$out/core.log" \
  && grep -q "EAP-AKA' challenge" "$out/core.log" \
  && grep -q "registered: imsi-999700000000001" "$out/core.log" \
  && grep -q "PDU Session establishment is successful PSI\[1\]" "$out/ue.log" \
  && grep -q "PDU Session establishment is successful PSI\[2\]" "$out/ue.log" \
  && grep -q "PDU session 2 to ims, UE 10.46.0.2" "$out/core.log" \
  && [ "$(grep -c "Configuration Update Command received" "$out/ue.log")" -ge 2 ] \
  && [ -n "$guti" ] && grep -qi "$guti" "$out/info.log" \
  && grep -q "AMF configuration update received" "$out/gnb.log" \
  && grep -q "AMF Configuration Update acknowledged" "$out/core.log" \
  && grep -q "AM policy, UE-AMBR 100 / 500 Mbit/s" "$out/core.log" \
  && grep -q "SM policy for PDU session 1 to internet: 5QI 9, ARP 8, AMBR 50 / 200 Mbit/s, 1 extra flow(s)" "$out/core.log" \
  && grep -q "new UE-AMBR 50 / 200 Mbit/s, to its gNB" "$out/core.log" \
  && grep -q "UE Context Modification Request received" "$out/gnb.log" \
  && grep -q "UE Context Modification done" "$out/core.log" \
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
