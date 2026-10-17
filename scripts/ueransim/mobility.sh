#!/usr/bin/env sh
# Idle mobility with UERANSIM, in Docker: the core in one container,
# UERANSIM's gNB A (tracking area 1) and the UE in a second, gNB B (tracking
# area 2) in a third. The UE registers under gNB A, gets its sessions, goes
# idle; gNB B starts, gNB A stops. The UE, now under gNB B, out of its
# registration area, sends a mobility registration (accepted from its kept
# context, no new authentication). The data network then pings the UE: the
# UPF reports the downlink data, the AMF sets the session up through gNB B
# (the UE is connected: no paging). UERANSIM's UE, when connected, sends its
# uplink data without asking for its session's user plane, so the ping comes
# from the network's side. See docker.sh for what is expected of the host.
set -eu
cd "$(dirname "$0")/../.."
. scripts/ueransim/docker.sh
run librecn-core 10.210.0.10 /etc/librecn/core.sh
wait_for librecn-core "AMF amf-1 ready"
# The UE searches gNB A (beside it) and gNB B (its container's address).
run librecn-ran 10.210.0.20 sh -c '
  sed "s/^  - 127.0.0.1\$/&\n  - 10.210.0.21/" /etc/librecn/ue.yaml > /tmp/ue.yaml
  nr-gnb -c /etc/librecn/gnb.yaml > /tmp/gnb.log 2>&1 & echo $! > /tmp/gnb.pid
  sleep 1; exec nr-ue -c /tmp/ue.yaml'
wait_for librecn-ran "TUN interface\[uesimtun0" || true
sleep 1
ping_a=$(docker exec librecn-ran ping -I uesimtun0 -c 3 -W 2 10.45.0.1 2>&1 || true)
release
wait_for librecn-core "deactivated (the UE is idle)" || true

# gNB B: another gNB id, tracking area 2, its own addresses.
run librecn-ran2 10.210.0.21 sh -c '
  sed -e "s/^nci: .*/nci: '"'"'0x000000020'"'"'/" -e "s/^tac: .*/tac: 2/" -e "s/^linkIp: .*/linkIp: 10.210.0.21/" \
      -e "s/^ngapIp: .*/ngapIp: 10.210.0.21/" -e "s/^gtpIp: .*/gtpIp: 10.210.0.21/" /etc/librecn/gnb.yaml > /tmp/gnb.yaml
  exec nr-gnb -c /tmp/gnb.yaml'
wait_for librecn-ran2 "NG Setup procedure is successful" || true
docker exec librecn-ran sh -c 'kill $(cat /tmp/gnb.pid)'
for _ in $(seq 30); do
  docker logs librecn-core 2>&1 | grep -q "registration from idle (Mobility)" && break
  sleep 1
done
wait_count librecn-core "registered: imsi-999700000000001" 2 || true
ping_b=$(docker exec librecn-core ping -c 3 -W 2 10.45.0.2 2>&1 || true)
docker stop -t 5 librecn-core >/dev/null

echo "--- gnb A"; docker exec librecn-ran cat /tmp/gnb.log 2>/dev/null || true
echo "--- gnb B"; docker logs librecn-ran2 2>&1
echo "--- ue"; docker logs librecn-ran 2>&1
echo "--- core"; docker logs librecn-core 2>&1
echo "--- ping under gNB A"; echo "$ping_a"
echo "--- ping under gNB B"; echo "$ping_b"
core=$(docker logs librecn-core 2>&1)
echo "$ping_a" | grep -q " 0% packet loss" \
  && echo "$core" | grep -q "registration from idle (Mobility): no new authentication" \
  && [ "$(echo "$core" | grep -c "registered: imsi-999700000000001")" -ge 2 ] \
  && echo "$core" | grep -q "N2 to UERANSIM-gnb-999-70-1 lost" \
  && echo "$ping_b" | grep -q " 0% packet loss" \
  && echo "$core" | grep -q "UE 2: PDU session 1 reactivated" \
  && echo "OK: UERANSIM's idle UE moved to another gNB and tracking area, registered there, and was reached through it" \
  || { echo "FAILED"; exit 1; }
