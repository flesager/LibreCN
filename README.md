# LibreCN
Open implementation of a 5GCore in Rust based on a pattern reactor scheduler and design for embedded target in barematal and docker

## Layout

| Directory | Content |
|---|---|
| [`scheduler/`](scheduler/) | Reactor scheduler hosting run-to-completion execution objects; design in [`scheduler/DESIGN.md`](scheduler/DESIGN.md) |
| [`common/`](common/) | Code shared by the network functions (placeholder) |
| [`nf/`](nf/) | Network functions, one crate each (empty for now) |

## Build and test

Rust 1.88 or newer. Linux only (epoll, eventfd, CPU affinity).

```
cargo build --workspace
cargo test --workspace
```
