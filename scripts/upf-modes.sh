#!/usr/bin/env sh
# The UPF's forwarding: per mode (event, poll, hybrid) with one UE, then per
# number of UEs and packet size, uplink and downlink, with the UPF's pool on
# 1 to CORES cores (default 4). Results go in nf/upf/README.md and README.md.
# Needs cores 2 to 10 and 127.0.0.80-81:2152 free (about a minute per core
# count).
set -eu
cd "$(dirname "$0")/.."
cargo build -q --release -p librecn-upf --example forwarding
target/release/examples/forwarding "$@" 2>/dev/null | grep '^|'
for cores in $(seq 1 "${CORES:-4}"); do
    echo "UPF pool on $cores core(s):"
    UPF_CORES=$cores target/release/examples/forwarding ues 2>/dev/null | grep '^|'
done
