# Shared by the Docker interop checks (user-plane.sh, mobility.sh), sourced
# from the repository's root: the image with LibreCN and UERANSIM, a network
# 10.210.0.0/24, containers started with `run`, waits on their logs.
# UERANSIM is expected next to this repository: ../UERANSIM/build (override
# with UERANSIM=), with patches/0001 applied. Needs docker and the host's
# sctp module.
bin=${UERANSIM:-../UERANSIM/build}
[ -x "$bin/nr-gnb" ] && [ -x "$bin/nr-ue" ] && [ -x "$bin/nr-cli" ] || { echo "no UERANSIM in $bin"; exit 1; }
patch=scripts/ueransim/patches/0001-gnb-select-an-AMF-for-protected-initial-messages.patch
git -C "$bin/../src" apply --check -R "$PWD/$patch" 2>/dev/null \
  || { echo "UERANSIM's gNB lacks $patch: git -C $bin/../src apply, then rebuild"; exit 1; }
image=librecn-interop
net=librecn-interop
containers=""
ctx=$(mktemp -d)
remove() {
  for c in $containers; do docker rm -f "$c" >/dev/null 2>&1 || true; done
  docker network rm "$net" >/dev/null 2>&1 || true
}
trap 'remove; rm -rf "$ctx"' EXIT
docker network rm "$net" >/dev/null 2>&1 || true

cargo build -q --release -p librecn
cp docker/* target/release/librecn "$bin/nr-gnb" "$bin/nr-ue" "$bin/nr-cli" "$ctx"
docker build -q -t "$image" "$ctx" >/dev/null
docker network create --subnet 10.210.0.0/24 "$net" >/dev/null

# run NAME IP COMMAND...: the command is PID 1 (LibreCN must get SIGTERM);
# $opts, if set, adds options to docker run (ports, volumes).
run() {
  name=$1 ip=$2; shift 2
  docker rm -f "$name" >/dev/null 2>&1 || true
  containers="$containers $name"
  # No IPv6: the kernel's router solicitations on uesimtun0 would wake the idle UE.
  docker run -d --name "$name" --network "$net" --ip "$ip" --cap-add NET_ADMIN --device /dev/net/tun \
    --sysctl net.ipv6.conf.all.disable_ipv6=1 ${opts:-} "$image" "$@" >/dev/null
}
# wait_for CONTAINER PATTERN: until PATTERN is in its logs (10 s).
wait_for() { for _ in $(seq 100); do docker logs "$1" 2>&1 | grep -q "$2" && return 0; sleep 0.1; done; return 1; }
# wait_count CONTAINER PATTERN N: until PATTERN appears N times in its logs.
wait_count() {
  for _ in $(seq 100); do [ "$(docker logs "$1" 2>&1 | grep -c "$2")" -ge "$3" ] && return 0; sleep 0.1; done; return 1
}
# release [GNB]: the gNB (default UERANSIM-gnb-999-70-1, in librecn-ran) releases its UE: idle.
release() {
  g=${1:-UERANSIM-gnb-999-70-1} c=${2:-librecn-ran}
  id=$(docker exec "$c" nr-cli "$g" -e ue-list | grep -o 'ue-id: [0-9]*' | head -1 | cut -d' ' -f2)
  docker exec "$c" nr-cli "$g" -e "ue-release ${id:-1}" >/dev/null || true
}
