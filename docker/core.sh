#!/bin/sh
# The core's container: N6 is the TUN device upf0, the UPF's side of the
# UE network (10.45.0.1/16); then LibreCN.
set -eu
ip tuntap add dev upf0 mode tun
ip addr add 10.45.0.1/16 dev upf0
ip link set upf0 up
exec librecn --config /etc/librecn/core.toml
