#!/usr/bin/env sh
# Load on the whole core (app/examples/load.rs): every EO in one pool pinned
# to 1, 2, 3 and 4 cores. Registrations with 5G AKA, NAS security and a PDU
# session from test gNBs on SCTP; 1400-byte packets both ways through the
# UPF, as fast as it goes (also with 84-byte packets) and at 500 Mbit/s
# each way; each alone, then both; then the memory for 1 000 to 40 000
# UEs. Results go in BENCHMARKS.md and the README's capacity table. Needs
# cores 2 to 14 (LibreCN on 2-5, the load on 6-14) and 127.0.0.1:2152 free
# (about 4 minutes).
set -eu
cd "$(dirname "$0")/.."
cargo build -q --release -p librecn --example load
for cores in 1 2 3 4; do
  target/release/examples/load "$cores" 4 5000 64 up 2>/dev/null | grep '^cores'      # 256 in flight, user plane flat out
  target/release/examples/load "$cores" 4 5000 64 up size=84 2>/dev/null | grep '^cores'  # the same, 84-byte packets
  target/release/examples/load "$cores" 4 5000 64 up=500 2>/dev/null | grep '^cores'  # 256 in flight, 500 Mbit/s each way
  target/release/examples/load "$cores" 1 2000 1 2>/dev/null | grep '^cores'          # unloaded: one UE at a time
done
for ues in 250 2500 5000 10000; do                                                 # memory: 1 000 to 40 000 UEs
  target/release/examples/load 2 4 "$ues" 64 2>/dev/null | grep 'memory'
done
