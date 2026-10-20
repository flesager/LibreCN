#!/usr/bin/env sh
# User-plane interop check with UERANSIM, in Docker (UERANSIM's UE needs
# root for uesimtun0): the core in one container, UERANSIM's gNB and UE in
# another. The UE registers, gets its two PDU sessions (DNNs internet and
# ims), then pings the UPF's N6 addresses (10.45.0.1 through uesimtun0,
# 10.46.0.1 through uesimtun1): GTP-U on N3, upf0 on N6; the PCF puts the
# answers from 10.45.0.1 on an extra QoS flow (docker/core.toml). iperf3
# then measures the session AMBR the PCF gives (10 Mbit/s up, 20 down, the
# UE-AMBR 1000), offering UDP at 3/2 of it: the UPF holds the downlink to it
# (UERANSIM's gNB limits the downlink to the UE-AMBR only), the uplink both
# do. (UDP: the rates are policed, and TCP over this sub-millisecond RTT
# gets about half of them, see nf/README.md.) The gNB
# The internet session's downlink comes from the core's own container, so
# the UPF carries it (the fallback: the kernel leaves the host's packets);
# so does the ims session, in the kernel (the fast path: XDP uplink, TC
# downlink): its AMBR (10 Mbit/s up, 20 down) is measured too, and the MBR
# of its PCC rule's flow (UDP from port 5203: 5 Mbit/s, classified there),
# with a data network in a third container (the kernel leaves the core's
# own packets to the UPF), which the core reaches through its one
# interface, as a host with one network card; it sends complete checksums,
# as the internet does (a container's are left to its interface: the fast
# path would pass them to the UPF). The gNB
# then releases the UE (idle): the next ping brings it back with a Service
# Request, its session reactivated (UERANSIM may drop the packet that woke
# it: one answer at least is expected). Released again, the UE is then
# pinged from the data network (the core's container, through upf0): the
# UPF parks the packets and reports them, the AMF pages the UE, whose
# Service Request brings its session back with the parked packets. Released
# a last time, the UE registers periodically when T3512 expires (30 s in
# docker/core.toml), accepted without a new authentication (UERANSIM sets
# the follow-on request: the AMF keeps the connection).
# See docker.sh for what is expected of the host.
set -eu
cd "$(dirname "$0")/../.."
. scripts/ueransim/docker.sh
opts="--sysctl net.ipv4.ip_forward=1"
run librecn-core 10.210.0.10 /etc/librecn/core.sh
opts=""
run librecn-dn 10.210.0.30 sh -c 'ethtool -K eth0 tx off >/dev/null && ip route add 10.46.0.0/16 via 10.210.0.10 \
  && iperf3 -s -D -p 5203 && exec iperf3 -s -p 5202'
wait_for librecn-core "AMF amf-1 ready"
run librecn-ran 10.210.0.20 sh -c 'nr-gnb -c /etc/librecn/gnb.yaml > /tmp/gnb.log 2>&1 & sleep 1; exec nr-ue -c /etc/librecn/ue.yaml'
wait_for librecn-ran "TUN interface\[uesimtun0" || true

wait_for librecn-ran "TUN interface\[uesimtun1" || true
sleep 1 # UERANSIM logs the interface up before it reads from it
ping=$(docker exec librecn-ran ping -I uesimtun0 -c 3 -W 2 10.45.0.1 2>&1 || true)
# The second session, on the second DNN (ims).
ims_ping=$(docker exec librecn-ran ping -I uesimtun1 -c 3 -W 2 10.46.0.1 2>&1 || true)
# The session AMBR, measured: UDP for 4 s each way, the receiver's rate in Mbit/s.
mbps() {
  docker exec librecn-ran iperf3 -c 10.45.0.1 -B 10.45.0.2 -t 4 -f m -u "$@" 2>&1 \
    | awk '/receiver/ { for (i = 2; i <= NF; i++) if ($i == "Mbits/sec") print $(i - 1) }'
}
uplink=$(mbps -b 15M)
downlink=$(mbps -b 30M -R)
# The ims session, in the fast path, to the data network's container. Its
# UE address is the second session's (10.46.0.x), read from uesimtun1.
ims_ue=$(docker exec librecn-ran ip -4 -o addr show uesimtun1 | awk '{ split($4, a, "/"); print a[1] }')
ims_mbps() {
  docker exec librecn-ran iperf3 -c 10.210.0.30 -B "$ims_ue" -t 4 -f m -u "$@" 2>&1 \
    | awk '/receiver/ { for (i = 2; i <= NF; i++) if ($i == "Mbits/sec") print $(i - 1) }'
}
ims_uplink=$(ims_mbps -p 5202 -b 15M)
ims_downlink=$(ims_mbps -p 5202 -b 30M -R)
ims_flow=$(ims_mbps -p 5203 -b 15M -R)
release
wait_for librecn-core "deactivated (the UE is idle)" || true
idle_ping=$(docker exec librecn-ran ping -I uesimtun0 -c 3 -W 2 10.45.0.1 2>&1 || true)
release
wait_count librecn-core "deactivated (the UE is idle)" 2 || true
paged_ping=$(docker exec librecn-core ping -c 3 -W 2 10.45.0.2 2>&1 || true)
release
for _ in $(seq 60); do
  [ "$(docker logs librecn-core 2>&1 | grep -c "registered: imsi-999700000000001")" -ge 2 ] && break
  sleep 1
done
metrics=$(docker exec librecn-core curl -s http://127.0.0.1:8080/metrics | grep 'eo="upf"' | grep counter_total || true)
docker stop -t 5 librecn-core >/dev/null

echo "--- gnb"; docker exec librecn-ran cat /tmp/gnb.log 2>/dev/null || docker cp librecn-ran:/tmp/gnb.log - 2>/dev/null || true
echo "--- ue"; docker logs librecn-ran 2>&1
echo "--- core"; docker logs librecn-core 2>&1
echo "--- ping"; echo "$ping"
echo "--- ping on the second DNN"; echo "$ims_ping"
echo "--- ping after idle"; echo "$idle_ping"
echo "--- ping of the idle UE (paging)"; echo "$paged_ping"
echo "--- iperf3, UDP at 15 up and 30 down (Mbit/s; session AMBR 10 up, 20 down)"; echo "uplink ${uplink:-none}, downlink ${downlink:-none}"
echo "--- iperf3 on ims, in the fast path, UDP at 15 up and 30 down (session AMBR 10 up, 20 down)"
echo "uplink ${ims_uplink:-none}, downlink ${ims_downlink:-none}; its PCC rule's flow (MBR 5), at 15 down: ${ims_flow:-none}"
echo "--- UPF counters"; echo "$metrics"
# within RATE AMBR FLOOR: at most the AMBR, 5 % for the 100 ms bursts, at
# least FLOOR of it. The uplink is policed by UERANSIM's gNB too, its own way.
within() { awk -v v="$1" -v max="$2" -v floor="$3" 'BEGIN { exit !(v != "" && v >= max * floor && v <= max * 1.05) }'; }
core=$(docker logs librecn-core 2>&1)
echo "$ping" | grep -q " 0% packet loss" \
  && echo "$ims_ping" | grep -q " 0% packet loss" \
  && echo "$idle_ping" | grep -Eq " [1-9][0-9]* received" \
  && echo "$core" | grep -q "resumed after" \
  && echo "$core" | grep -q "PDU session 1 reactivated" \
  && echo "$paged_ping" | grep -Eq " [1-9][0-9]* received" \
  && echo "$core" | grep -q "paged for PDU session 1" \
  && echo "$core" | grep -q "registration from idle (Periodic)" \
  && [ "$(echo "$core" | grep -c "registered: imsi-999700000000001")" -ge 2 ] \
  && echo "$metrics" | grep -Eq 'counter="downlink_packets"} [1-9]' \
  && echo "$metrics" | grep -Eq 'counter="extra_flow_packets"} [1-9]' \
  && within "$uplink" 10 0.8 && within "$downlink" 20 0.95 \
  && echo "$metrics" | grep -Eq 'counter="rate_limited_packets"} [1-9]' \
  && echo "$core" | grep -q "1 extra flow(s)" \
  && echo "$core" | grep -q "fast path on eth0" \
  && echo "$metrics" | grep -Eq 'counter="fast_path_packets"} [1-9][0-9]{3}' \
  && within "$ims_uplink" 10 0.8 && within "$ims_downlink" 20 0.9 && within "$ims_flow" 5 0.9 \
  && echo "OK: UERANSIM's UE pinged through LibreCN's user plane on two DNNs (answers on the PCF's extra flow, the session AMBR held: $uplink / $downlink Mbit/s; in the fast path: $ims_uplink / $ims_downlink, its flow's MBR $ims_flow), before and after idle; pinged while idle: paged; periodic registration" \
  || { echo "FAILED"; exit 1; }
