#!/bin/sh
# LibreCN's container: the configuration (/etc/librecn/librecn.toml if one
# is mounted, else written from the LIBRECN_* settings of .env), N6 (the TUN
# device upf0 with the UE pool's first address, NAT to the outside), then
# the core. At stop, N6 is undone: with the host's network, upf0 and the NAT
# rules are the host's.
set -eu

say() { echo "librecn-entrypoint: $*"; }
fail() { say "$*"; exit 1; }

# The host's main address: the one its default route leaves from.
default_address() { ip -4 route get 1.1.1.1 2>/dev/null | awk '{for (i = 1; i < NF; i++) if ($i == "src") print $(i + 1)}'; }
default_device() { ip -4 route get 1.1.1.1 2>/dev/null | awk '{for (i = 1; i < NF; i++) if ($i == "dev") print $(i + 1)}'; }

address=${LIBRECN_ADDRESS:-$(default_address)}
mcc=${LIBRECN_MCC:-001}
mnc=${LIBRECN_MNC:-01}
sst=${LIBRECN_SST:-1}
sd=${LIBRECN_SD:-}
dnn=${LIBRECN_DNN:-internet}
pool=${LIBRECN_UE_POOL:-10.45.0.0/16}
nat=${LIBRECN_NAT:-yes}
gui=${LIBRECN_GUI:-127.0.0.1:8080}
log=${LIBRECN_LOG:-info}
[ -n "$address" ] || fail "no address for N2 and N3: set LIBRECN_ADDRESS in .env"

# What the host must give: a TUN device (N6), IP forwarding (NAT). SCTP (N2):
# the kernel loads its module when the AMF opens its socket, unless the
# host forbids it (the AMF then says so).
[ -c /dev/net/tun ] || fail "no /dev/net/tun: the container needs it (compose.yaml: devices)"
if [ "$nat" = yes ] && [ "$(cat /proc/sys/net/ipv4/ip_forward)" != 1 ]; then
  fail "IP forwarding is off: on the host, run 'sudo sysctl -w net.ipv4.ip_forward=1' (or set LIBRECN_NAT=no)"
fi

# The UE pool's first address is the UPF's N6 side.
net=${pool%/*}
len=${pool#*/}
gateway=${net%.*}.$((${net##*.} + 1))

config=/etc/librecn/librecn.toml
if [ ! -f "$config" ]; then
  config=/run/librecn.toml
  slice="{ sst = $sst }"
  [ -z "$sd" ] || slice="{ sst = $sst, sd = \"$sd\" }"
  cat > "$config" <<TOML
# Written by librecn-entrypoint from .env; mount your own file at
# /etc/librecn/librecn.toml to replace it (see app/librecn.example.toml).
[oam]
listen = "$gui"
log_level = "$log"
state_dir = "/var/lib/librecn"   # subscribers and policies added in the GUI

[[nf]]
kind = "udm"
name = "udm"                     # no subscriber: add them in the GUI (Provisioning)

[[nf]]
kind = "ausf"
name = "ausf"

[[nf]]
kind = "upf"
name = "upf"
[nf.params]
n3 = "$address"
n6 = "upf0"

[[nf]]
kind = "pcf"
name = "pcf"

[[nf]]
kind = "smf"
name = "smf"
[nf.params]
dnns = [{ dnn = "$dnn", pool = "$pool" }]
slices = [$slice]

[[nf]]
kind = "amf"
name = "amf-1"
[nf.params]
n2 = "$address:38412"
plmn = { mcc = "$mcc", mnc = "$mnc" }
slices = [$slice]
network_name = { full = "LibreCN", short = "LibreCN" }
TOML
  say "N2 $address:38412, N3 $address:2152, PLMN $mcc/$mnc, DNN $dnn ($pool), GUI http://$gui"
fi

# The NAT rules: the UEs' packets leave with the host's address; FORWARD
# lets them through (Docker sets its policy to DROP). nat_rules -I adds
# them (once), nat_rules -D removes them.
nat_rules() {
  for rule in "-t nat POSTROUTING -s $pool ! -o upf0 -j MASQUERADE" "FORWARD -i upf0 -j ACCEPT" \
              "FORWARD -o upf0 -j ACCEPT"; do
    table=filter
    case $rule in -t*) table=nat rule=${rule#-t nat } ;; esac
    if [ "$1" = -I ]; then
      iptables -t "$table" -C $rule 2>/dev/null || iptables -t "$table" -I $rule
    else
      iptables -t "$table" -D $rule 2>/dev/null || true
    fi
  done
}

# N6: upf0 (kept if it exists), then NAT.
ip link show upf0 >/dev/null 2>&1 || ip tuntap add dev upf0 mode tun
ip addr replace "$gateway/$len" dev upf0
ip link set upf0 up
if [ "$nat" = yes ]; then
  nat_rules -I
  say "N6: upf0 $gateway/$len, NAT to the outside through $(default_device)"
else
  say "N6: upf0 $gateway/$len, no NAT"
fi

undo() {
  [ "$nat" != yes ] || nat_rules -D
  ip link del upf0 2>/dev/null || true
}

# The core in the background, so that its stop (docker stop: SIGTERM) undoes N6.
librecn --config "$config" &
core=$!
stopping=no
trap 'stopping=yes; kill -TERM "$core" 2>/dev/null || true' TERM INT
status=0
wait "$core" || status=$?
# The signal interrupted wait: wait again, for the core's own status.
if [ "$stopping" = yes ]; then
  status=0
  wait "$core" || status=$?
fi
undo
exit "$status"
