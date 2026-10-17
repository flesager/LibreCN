#!/usr/bin/env sh
# Slow checks, kept out of `cargo test` so the trunk stays fast:
# repeated release runs of the threaded tests, runs on 1 and 2 CPUs, loom.
# Usage: scripts/stress.sh [RUNS]   (default 20)
set -eu
runs=${1:-20}
cd "$(dirname "$0")/.."

cargo test --workspace --release --no-run -q
bins=$(cargo test -p librecn-scheduler --release --no-run --message-format=json -q 2>/dev/null \
  | grep -o '"executable":"[^"]*\(runtime\|polling\|no_alloc\)-[^"]*"' | cut -d'"' -f4)
[ -n "$bins" ] || { echo "no threaded test binaries found"; exit 1; }

for i in $(seq "$runs"); do
  cargo test --workspace --release -q > /dev/null || { echo "release run $i failed"; exit 1; }
done
echo "release: $runs runs ok"

for cpus in 0 0,1; do
  for bin in $bins; do
    for i in 1 2 3; do
      taskset -c "$cpus" "$bin" -q > /dev/null || { echo "$bin failed on CPUs $cpus"; exit 1; }
    done
  done
  echo "CPUs $cpus: ok"
done

RUSTFLAGS="--cfg loom" CARGO_TARGET_DIR=target/loom \
  cargo test -p librecn-scheduler --release --lib -q queue::loom_tests
echo "loom: ok"
