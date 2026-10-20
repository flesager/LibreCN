#!/bin/sh
# Builds the UPF's fast path, upf.bpf.c, into upf.bpf.o (checked in, so
# that cargo needs no clang): with the host's clang if it has one and
# libbpf's headers, else in a small Docker image with both.
#
#   nf/upf/bpf/build.sh
set -eu
cd "$(dirname "$0")"
build='clang -O2 -g -target bpf -mcpu=v3 -Wall -Werror -I/usr/include/$(uname -m)-linux-gnu -c upf.bpf.c -o upf.bpf.o && llvm-strip -g upf.bpf.o'
if command -v clang >/dev/null && command -v llvm-strip >/dev/null && [ -f /usr/include/bpf/bpf_helpers.h ]; then
  sh -c "$build"
else
  printf 'FROM ubuntu:26.04\nRUN apt-get update && apt-get install -y --no-install-recommends clang llvm libbpf-dev linux-libc-dev && rm -rf /var/lib/apt/lists/*\n' \
    | docker build -q -t librecn-bpf - >/dev/null
  docker run --rm -u "$(id -u):$(id -g)" -v "$PWD:/src" -w /src librecn-bpf sh -c "$build"
fi
ls -l upf.bpf.o
