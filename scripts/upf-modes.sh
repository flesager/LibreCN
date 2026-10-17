#!/usr/bin/env sh
# The UPF alone on one core: per mode (event, poll, hybrid) with one UE, then
# per number of UEs and packet size, uplink and downlink. Results go in
# nf/upf/README.md and BENCHMARKS.md. Needs cores 2 to 10 and
# 127.0.0.80-81:2152 free (about a minute).
set -eu
cd "$(dirname "$0")/.."
cargo build -q --release -p librecn-upf --example forwarding
target/release/examples/forwarding "$@" 2>/dev/null | grep '^|'
target/release/examples/forwarding ues 2>/dev/null | grep '^|'
