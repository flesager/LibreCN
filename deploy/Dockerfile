# LibreCN's image, built from source for amd64 and arm64: a static binary
# (musl) on Alpine, about 23 MB. Rust is compiled on the build machine's own
# architecture (no emulation: fast); only the small runtime stage runs on
# the target's. musl's allocator costs about a third of the registration
# rate (BENCHMARKS.md): accepted for a small core (the owner's choice).
#
#   docker buildx build -f deploy/Dockerfile --platform linux/amd64,linux/arm64 .
FROM --platform=$BUILDPLATFORM rust:1.95-trixie AS build
ARG BUILDARCH
ARG TARGETARCH
# The target's triple and, when it is not the build machine's, its linker
# (Rust's musl targets bring their own C runtime).
RUN case "$TARGETARCH" in \
      amd64) echo x86_64-unknown-linux-musl > /triple; cross=gcc-x86-64-linux-gnu ;; \
      arm64) echo aarch64-unknown-linux-musl > /triple; cross=gcc-aarch64-linux-gnu ;; \
      *) echo "unsupported architecture $TARGETARCH" >&2; exit 1 ;; \
    esac \
 && rustup target add "$(cat /triple)" \
 && if [ "$TARGETARCH" != "$BUILDARCH" ]; then \
      apt-get update && apt-get install -y --no-install-recommends $cross && rm -rf /var/lib/apt/lists/*; \
    fi
# No debug information in the image: the release profile keeps some.
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-linux-gnu-gcc \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc \
    CARGO_PROFILE_RELEASE_DEBUG=false \
    CARGO_PROFILE_RELEASE_STRIP=symbols
WORKDIR /src
COPY . .
RUN triple=$(cat /triple) \
 && if [ "$TARGETARCH" = "$BUILDARCH" ]; then unset CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER; fi \
 && cargo build --release --locked -p librecn --target "$triple" \
 && cp "target/$triple/release/librecn" /librecn

FROM alpine:3.22
# ip (N6's TUN device), iptables (NAT to the outside).
RUN apk add --no-cache iproute2 iptables
COPY --from=build /librecn /usr/local/bin/librecn
COPY deploy/entrypoint.sh /usr/local/bin/librecn-entrypoint
VOLUME /var/lib/librecn
ENTRYPOINT ["librecn-entrypoint"]
