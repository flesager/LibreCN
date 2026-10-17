# LibreCN, and UERANSIM for the interop checks, in one image: the same base
# as the build host (the binaries are built there, not here).
# Built by scripts/ueransim/user-plane.sh, which stages the binaries.
FROM ubuntu:26.04
RUN apt-get update \
 && apt-get install -y --no-install-recommends curl iproute2 iputils-ping libsctp1 libstdc++6 \
 && rm -rf /var/lib/apt/lists/* \
 && mkdir -p /etc/iproute2 \
 && { cp /usr/share/iproute2/rt_tables /etc/iproute2/ 2>/dev/null || printf '255\tlocal\n254\tmain\n253\tdefault\n0\tunspec\n' > /etc/iproute2/rt_tables; }
# UERANSIM's UE adds its routing table to /etc/iproute2/rt_tables, which
# recent iproute2 no longer installs there.
COPY librecn nr-gnb nr-ue nr-cli /usr/local/bin/
COPY core.sh core.toml gnb.yaml ue.yaml /etc/librecn/
