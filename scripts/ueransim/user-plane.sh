#!/usr/bin/env sh
# User-plane interop check with UERANSIM, in Docker (UERANSIM's UE needs
# root for uesimtun0): the core in one container, UERANSIM's gNB and UE in
# another. The UE registers, gets its two PDU sessions (DNNs internet and
# ims), then pings the UPF's N6 addresses (10.45.0.1 through uesimtun0,
# 10.46.0.1 through uesimtun1): GTP-U on N3, upf0 on N6; the PCF puts the
# answers from 10.45.0.1 on an extra QoS flow (docker/core.toml). The gNB
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
run librecn-core 10.210.0.10 /etc/librecn/core.sh
wait_for librecn-core "AMF amf-1 ready"
run librecn-ran 10.210.0.20 sh -c 'nr-gnb -c /etc/librecn/gnb.yaml > /tmp/gnb.log 2>&1 & sleep 1; exec nr-ue -c /etc/librecn/ue.yaml'
wait_for librecn-ran "TUN interface\[uesimtun0" || true

wait_for librecn-ran "TUN interface\[uesimtun1" || true
sleep 1 # UERANSIM logs the interface up before it reads from it
ping=$(docker exec librecn-ran ping -I uesimtun0 -c 3 -W 2 10.45.0.1 2>&1 || true)
# The second session, on the second DNN (ims).
ims_ping=$(docker exec librecn-ran ping -I uesimtun1 -c 3 -W 2 10.46.0.1 2>&1 || true)
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
echo "--- UPF counters"; echo "$metrics"
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
  && echo "$core" | grep -q "1 extra flow(s)" \
  && echo "OK: UERANSIM's UE pinged through LibreCN's user plane on two DNNs (answers on the PCF's extra flow), before and after idle; pinged while idle: paged; periodic registration" \
  || { echo "FAILED"; exit 1; }
