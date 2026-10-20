#!/usr/bin/env sh
# The UPF's uplink with and without its fast path (XDP), between two
# containers over veth (nf/upf/examples/fast_path.rs): the UPF on core 2,
# the receive path of its N3 interface (veth's NAPI, in a thread, where XDP
# runs) on core 3, the gNB's three senders on cores 8 to 10. The fast path
# carries the sessions, or passes them to the UPF (the fallback: sessions
# with a PCC rule, which it leaves to the UPF). The UEs' packets are forwarded out of a
# dummy device standing for the data network. Per packet size and mode:
# what LibreCN carried, and how busy its two cores were (results in
# nf/upf/README.md). The UPF's container is privileged and sees the host's
# processes, to pin the kernel's NAPI thread: for this benchmark only.
#
#   scripts/upf-fast.sh            # UES=100 SECONDS=5
set -eu
cd "$(dirname "$0")/.."
cargo build -q --release -p librecn-upf --example fast_path
bin=$PWD/target/release/examples/fast_path
printf 'FROM ubuntu:26.04\nRUN apt-get update && apt-get install -y --no-install-recommends iproute2 procps util-linux && rm -rf /var/lib/apt/lists/*\n' \
  | docker build -q -t librecn-bench - >/dev/null
net=librecn-bench
cleanup() {
  docker rm -f librecn-bench-upf librecn-bench-gnb >/dev/null 2>&1 || true
  docker network rm "$net" >/dev/null 2>&1 || true
}
trap cleanup EXIT
cleanup
docker network create --subnet 10.211.0.0/24 "$net" >/dev/null
ues=${UES:-100}
seconds=${SECONDS:-5}
echo "| mode | packet | carried | UPF core busy | receive core busy |"
echo "|---|---|---|---|---|"
for size in 84 1400; do
  for mode in fast fallback; do
    docker run -d --name librecn-bench-upf --network "$net" --ip 10.211.0.10 --privileged --pid=host \
      --sysctl net.ipv4.ip_forward=1 -v "$bin:/fast_path:ro" librecn-bench sh -c "
        ip tuntap add dev upf0 mode tun && ip addr add 10.45.0.1/16 dev upf0 && ip link set upf0 up
        ip link add n6 type dummy && ip addr add 10.99.0.1/24 dev n6 && ip link set n6 up
        exec /fast_path upf 10.211.0.10 $ues $mode $seconds" >/dev/null
    for _ in $(seq 100); do docker logs librecn-bench-upf 2>&1 | grep -q ready && break; sleep 0.1; done
    # veth's receive path (its NAPI, which XDP brings) in a thread of its own, on core 3.
    docker exec librecn-bench-upf sh -c 'echo 1 > /sys/class/net/eth0/threaded
      for p in $(pgrep "^napi/eth0-"); do taskset -pc 3 "$p" >/dev/null; done'
    docker run -d --name librecn-bench-gnb --network "$net" --ip 10.211.0.20 --cpuset-cpus 8-10 \
      -v "$bin:/fast_path:ro" librecn-bench /fast_path gnb 10.211.0.10 "$ues" "$size" 3 >/dev/null
    docker wait librecn-bench-upf >/dev/null
    line=$(docker logs librecn-bench-upf 2>&1 | grep carried || echo "carried nothing")
    docker rm -f librecn-bench-upf librecn-bench-gnb >/dev/null
    mode=$([ "$mode" = fast ] && echo "fast path" || echo "fallback")
    echo "$line" | awk -v mode="$mode" -v size="$size" '{
      kpps = $2; printf "| %s | %d B | %d kpps, %.2f Gbit/s | %s | %s |\n", mode, size, kpps, kpps * size * 8 / 1e6, $7 " %", $12 " %" }'
  done
done
