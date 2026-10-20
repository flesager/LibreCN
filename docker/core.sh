#!/bin/sh
# The core's container: N6 is the TUN device upf0, the UPF's side of the
# UE network (10.45.0.1/16), with an iperf3 server there; then LibreCN,
# with the configuration given (default /etc/librecn/core.toml).
set -eu
ip tuntap add dev upf0 mode tun
ip addr add 10.45.0.1/16 dev upf0
ip addr add 10.46.0.1/16 dev upf0   # the second DNN (ims, core.toml)
ip link set upf0 up
iperf3 -s -D -B 10.45.0.1   # user-plane.sh measures the session AMBR
exec librecn --config "${1:-/etc/librecn/core.toml}"
