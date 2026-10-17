#!/usr/bin/env sh
# The UPF's forwarding in each mode (event, poll, hybrid): latency,
# throughput, idle CPU. Results go in nf/upf/README.md. Needs cores 2 to 8
# and 127.0.0.80-81:2152 free (a few seconds per mode).
set -eu
cd "$(dirname "$0")/.."
cargo build -q --release -p librecn-upf --example forwarding
target/release/examples/forwarding "$@" 2>/dev/null
