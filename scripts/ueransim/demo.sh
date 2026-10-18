#!/usr/bin/env sh
# A long run to watch in the GUI, in Docker: the core, GNBS gNBs (default
# 2), each in its own container with UES UEs (default 10) beside it, every
# UE with a session on each DNN (internet, ims). The first gNB has two
# cells (NR Cell Identities ...10 and ...11, UERANSIM patch 0005), half its
# UEs in each; the others one. The GUI is on the host,
# http://127.0.0.1:$PORT (default 8080). Then, until Ctrl-C, the network
# keeps busy, each step printed:
# - every 5 s, a few pings from UEs (uplink) and to UEs (from the data
#   network: an idle UE is paged);
# - every 10 s, two UEs released to idle by their gNB;
# - every 30 s, a UE releases its ims session, set up again 10 s later;
# - every 40 s, the network deregisters a UE, which registers again;
# - every 5 s, two new UDP flows of 12 s on random UEs, 20 to 200 Mbit/s,
#   three times out of four downlink, on either DNN: about five flows at a
#   time, over the cells (the PCF's default session AMBR: 500 Mbit/s up,
#   1000 down; UE-AMBR 1000 up, 2000 down); ten iperf3 servers in the
#   core, ports 5202 to 5211.
# Ctrl-C, or `demo.sh stop` from another terminal, removes the containers;
# the core's log is kept in target/demo-core.log.
# See docker.sh for what is expected of the host.
#
#   scripts/ueransim/demo.sh                    # 2 gNBs x 10 UEs
#   GNBS=4 UES=50 PORT=8081 scripts/ueransim/demo.sh
#   scripts/ueransim/demo.sh stop
set -eu
cd "$(dirname "$0")/../.."
if [ "${1:-}" = stop ]; then
  docker ps -aq --filter name='^librecn-(core|gnb-[0-9]+)$' | xargs -r docker rm -f >/dev/null
  docker network rm librecn-interop >/dev/null 2>&1 || true
  echo "demo stopped"
  exit 0
fi
gnbs=${GNBS:-2}
ues=${UES:-10}
patch=scripts/ueransim/patches/0005-gnb-several-cells-each-with-its-radio-address.patch
git -C "${UERANSIM:-../UERANSIM/build}/../src" apply --check -R "$PWD/$patch" 2>/dev/null \
  || { echo "UERANSIM lacks $patch (the first gNB's second cell): apply it, then rebuild"; exit 1; }
port=${PORT:-8080}
total=$((gnbs * ues))
[ "$ues" -le 512 ] || { echo "UES: 512 at most per gNB (UERANSIM's nr-ue)"; exit 1; }
[ "$gnbs" -le 200 ] || { echo "GNBS: 200 at most"; exit 1; }
. scripts/ueransim/docker.sh
# Ctrl-C ends the run through docker.sh's clean-up, the core's log kept.
keep_log() { docker logs librecn-core > target/demo-core.log 2>&1 && echo "the core's log: target/demo-core.log" || true; }
trap 'echo; echo "stopping"; keep_log; exit 0' INT TERM

# The core: docker/core.toml's NFs, a subscriber per UE, the default policy.
cat > "$ctx/demo.toml" <<TOML
[[runtime.pools]]
name = "cp"
workers = 1
[[runtime.pools]]
name = "up"
workers = 1
[oam]
listen = "0.0.0.0:8080"
[[nf]]
kind = "udm"
name = "udm"
[[nf.params.subscribers]]
supi = "imsi-999700000000001"
count = $total
k = "465B5CE8B199B49FAA5F0A2EE238A6BC"
opc = "E8ED289DEBA952E4283B54E88E6183CA"
[[nf]]
kind = "ausf"
name = "ausf"
[[nf]]
kind = "upf"
name = "upf"
pool = "up"
[nf.params]
n3 = "10.210.0.10"
n6 = "upf0"
max_sessions = $((2 * total + 16))
[[nf]]
kind = "pcf"
name = "pcf"
[nf.params.default]
ue_ambr_mbps = { uplink = 1000, downlink = 2000 }
[[nf.params.default.sessions]]
ambr_mbps = { uplink = 500, downlink = 1000 }
[[nf.params.policies]]           # the first UE: pings from the data network on their own flow
supi = "imsi-999700000000001"
[[nf.params.policies.sessions]]
dnn = "internet"
ambr_mbps = { uplink = 500, downlink = 1000 }
[[nf.params.policies.sessions.flows]]
five_qi = 7
filters = [{ direction = "downlink", remote = "10.45.0.1", protocol = 1 }]
[[nf]]
kind = "smf"
name = "smf"
[nf.params]
dnns = [{ dnn = "internet", pool = "10.45.0.0/16" }, { dnn = "ims", pool = "10.46.0.0/16" }]
[[nf]]
kind = "amf"
name = "amf-1"
[nf.params]
n2 = "10.210.0.10:38412"
max_ues = $((total + 64))
TOML
opts="-p 127.0.0.1:$port:8080 -v $ctx/demo.toml:/etc/librecn/demo.toml:ro"
run librecn-core 10.210.0.10 /etc/librecn/core.sh /etc/librecn/demo.toml
opts=""
wait_for librecn-core "AMF amf-1 ready" || { docker logs librecn-core; exit 1; }
echo "GUI: http://127.0.0.1:$port ($gnbs gNB(s) x $ues UE(s) starting; Ctrl-C to stop)"
# The traffic's servers, one flow each at a time (core.sh's own serves 5201).
for p in $(seq 5202 5211); do docker exec -d librecn-core iperf3 -s -p "$p"; done

# gNB g (1, 2...): 10.210.0.(20 + g), gNB id g, tracking area g, and its UEs.
for g in $(seq "$gnbs"); do
  ip=10.210.0.$((20 + g))
  first=$(((g - 1) * ues + 1))
  # The first gNB's second cell: radio address 127.0.0.2, half its UEs
  # (a second nr-ue, searching that cell only, its TUN devices named apart).
  cell2=: ue2=: here=$ues
  if [ "$g" -eq 1 ] && [ "$ues" -ge 2 ]; then
    here=$((ues / 2))
    # Two nr-ue add their routing tables to rt_tables at once, with the same
    # numbers: every table is there first, numbered apart.
    cell2="printf 'extraCells:\n  - nci: $(printf 0x%09x $((g << 4 | 1)))\n    linkIp: 127.0.0.2\n' >> /tmp/gnb.yaml
    for i in \$(seq 0 $((2 * ues))); do echo \"\$((1000 + i)) rt_uesimtun\$i\"; echo \"\$((3000 + i)) rt_uesimc2tun\$i\"; done >> /etc/iproute2/rt_tables"
    ue2='sed -e "s/^  - 127.0.0.1\$/  - 127.0.0.2/" -e "\$a tunName: uesimc2tun" /etc/librecn/ue.yaml > /tmp/ue2.yaml
    nr-ue -c /tmp/ue2.yaml -n '$((ues - here))' -i imsi-99970'$(printf %010d $((first + here)))' -t 200 > /tmp/ue2.log 2>&1 &'
  fi
  run "librecn-gnb-$g" "$ip" sh -c "
    sed -e \"s/^nci: .*/nci: '$(printf 0x%09x $((g << 4)))'/\" -e 's/^tac: .*/tac: $g/' \
        -e 's/^ngapIp: .*/ngapIp: $ip/' -e 's/^gtpIp: .*/gtpIp: $ip/' /etc/librecn/gnb.yaml > /tmp/gnb.yaml
    $cell2
    nr-gnb -c /tmp/gnb.yaml > /tmp/gnb.log 2>&1 &
    sleep 1
    $ue2
    exec nr-ue -c /etc/librecn/ue.yaml -n $here -i imsi-99970$(printf %010d $first) -t 200"
done

# Helpers, on the containers.
api() { docker exec librecn-core curl -s "$@"; }
gnb() { echo "librecn-gnb-$(shuf -i 1-"$gnbs" -n 1)"; }
# A UE address with a session, from the SMF.
ue_ip() { api http://127.0.0.1:8080/api/nfs/smf/sessions | grep -o '"ue_ip":"10.45[0-9.]*"' | cut -d'"' -f4 | shuf -n 1; }
alive() { docker inspect -f '{{.State.Running}}' librecn-core 2>/dev/null | grep -q true; }
say() { echo "$(date +%T) $*"; }

t=0
released=""
flow=5202
while alive; do
  sleep 5
  t=$((t + 5))
  # Pings: from two UEs to the UPF's N6 side, and from the data network to two UEs.
  for _ in 1 2; do
    c=$(gnb)
    tun=$(docker exec "$c" ip -o link 2>/dev/null | grep -o 'uesim[c0-9]*tun[0-9]*' | shuf -n 1 || true)
    [ -n "$tun" ] && docker exec "$c" ping -I "$tun" -c 1 -W 2 10.45.0.1 >/dev/null 2>&1 &
    ip=$(ue_ip || true)
    [ -n "$ip" ] && docker exec librecn-core ping -c 1 -W 3 "$ip" >/dev/null 2>&1 &
  done
  if [ $((t % 10)) -eq 0 ]; then
    c=$(gnb)
    name=UERANSIM-gnb-999-70-${c#librecn-gnb-}
    for id in $(docker exec "$c" nr-cli "$name" -e ue-list 2>/dev/null | grep -o 'ue-id: [0-9]*' | cut -d' ' -f2 | shuf -n 2); do
      docker exec "$c" nr-cli "$name" -e "ue-release $id" >/dev/null 2>&1 || true
    done
    say "$c: two UEs released to idle"
  fi
  if [ $((t % 30)) -eq 0 ]; then
    c=$(gnb)
    ue=$(docker exec "$c" nr-cli --dump 2>/dev/null | grep '^imsi-' | shuf -n 1 || true)
    if [ -n "$ue" ]; then
      docker exec "$c" nr-cli "$ue" -e "ps-release 2" >/dev/null 2>&1 || true
      released="$c $ue"
      say "$ue releases its ims session"
    fi
  elif [ $((t % 30)) -eq 10 ] && [ -n "$released" ]; then
    set -- $released
    docker exec "$1" nr-cli "$2" -e "ps-establish IPv4 --sst 1 --dnn ims" >/dev/null 2>&1 || true
    say "$2 sets its ims session up again"
    released=""
  fi
  if [ $((t % 40)) -eq 0 ]; then
    supi=$(api http://127.0.0.1:8080/api/nfs/amf-1/ues | grep -o '"supi":"imsi-[0-9]*"' | cut -d'"' -f4 | shuf -n 1 || true)
    if [ -n "$supi" ]; then
      api -X DELETE -d '{"reregister": true}' "http://127.0.0.1:8080/api/nfs/amf-1/ues/$supi" >/dev/null || true
      say "the network deregisters $supi (it registers again)"
    fi
  fi
  # Two new flows every tick, each on the next server port: a UE's address
  # on either DNN, to that DNN's N6 side (10.45.0.1 or 10.46.0.1).
  for _ in 1 2; do
    c=$(gnb)
    bind=$(docker exec "$c" ip -4 -o addr 2>/dev/null | grep -o 'inet 10\.4[56]\.[0-9.]*' | cut -d' ' -f2 | shuf -n 1 || true)
    [ -n "$bind" ] || continue
    rate=$(shuf -i 20-200 -n 1)
    way=$(shuf -n 1 -e down down down up)
    reverse=""
    [ "$way" = down ] && reverse=-R
    docker exec -d "$c" iperf3 -c "${bind%.*.*}.0.1" -B "$bind" -p "$flow" -u -b "${rate}M" -t 12 $reverse
    say "UE $bind: 12 s of UDP at $rate Mbit/s $way"
    flow=$((flow == 5211 ? 5202 : flow + 1))
  done
done
say "the core is gone"
keep_log
