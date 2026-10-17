#!/usr/bin/env sh
# Control-plane load on the whole core (app/examples/load.rs): registrations
# with 5G AKA, NAS security and a PDU session, from test gNBs on SCTP, with
# LibreCN's NFs pinned to 1, 2, 3 and 4 cores. Results go in README.md.
# Needs cores 2 to 11 (LibreCN on 2-5, OAM on 6, gNBs on 8-11).
set -eu
cd "$(dirname "$0")/.."
cargo build -q --release -p librecn --example load
for cores in 1 2 3 4; do
  target/release/examples/load "$cores" 4 5000 64 2>/dev/null   # loaded: 4 gNBs, 256 in flight
  target/release/examples/load "$cores" 1 2000 1 2>/dev/null    # unloaded: one UE at a time
done
