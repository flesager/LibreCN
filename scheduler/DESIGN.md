# LibreCN scheduler: design

Status: **M1 and M2 implemented**: building blocks and test bed (§15), multi-core runtime (§16). M3 not started.

## 1. Goals

- A reactor scheduler hosting **Execution Objects** (EOs). An EO hosts one network function and processes messages **run-to-completion**.
- Runs on Linux, baremetal or in Docker, using the kernel network stack.
- **No allocation on the hot path.** All memory (queues, message buffers, timers, registry) is preallocated when an EO is created, and every pool is measurable.
- **Observable.** Processing time, queue depth, high-water marks and memory use are exposed per EO and collected at runtime by a metric loop.
- **Testable.** Every EO can run unchanged on a single-threaded fake platform with a virtual clock.

Out of scope for now: multi-process messaging, multiple instances of one NF, poll and hybrid modes (UPF milestone), adding or removing EOs at runtime (planned, and the design allows it).

## 2. Overview

```
                 ┌──────────────────────── Runtime ─────────────────────────┐
                 │                                                          │
                 │  Registry (fixed capacity, EoId = index + generation)    │
                 │                                                          │
                 │  ┌──────── Pool "cp" (Event mode) ────────┐              │
                 │  │ worker@core2  worker@core3  worker@core4 │  ...more    │
                 │  │      \            |            /         │  pools     │
                 │  │       ── shared run queue (EO handles) ── │            │
                 │  │       ── epoll: sockets + wakeup eventfd ─│            │
                 │  └───────────────────────────────────────────┘            │
                 └──────────────────────────────────────────────────────────┘

   EO = state + Mailbox
        Mailbox = system lane (header-only) + data lane (MPSC ring)
                  + its own BufferPool (size classes) + EoStats
```

- **Pool**: a group of worker threads, each pinned to a core. An EO belongs to one pool and can run on **any** worker of that pool, but never on two workers at once.
- **Everything is a message**: messages from other EOs, timer expiries, I/O readiness and lifecycle events all arrive in the EO's mailbox. The header says what each one is.

## 3. Identifiers and message header

```rust
#[repr(transparent)]
pub struct EoId(u64);          // low 32 bits: registry index, high 32 bits: generation

#[repr(C)]
pub struct MsgHeader {         // 40 bytes, fixed layout
    pub msg_id:   u32,         // message type, see §3.1
    pub body_len: u32,         // 0 = header-only message
    pub from:     EoId,        // EoId::SYSTEM for timers / I/O / lifecycle
    pub to:       EoId,
    pub corr_id:  u32,         // request/response correlation
    pub flags:    u32,
    pub reserved: u64,         // system messages: timer id, I/O token, ...
}
```

- **Generation**: when an EO is removed and its slot reused, the generation increments. A message addressed to a stale `EoId` is rejected (`Error::UnknownEo`), never delivered to the new EO.
- **System messages are header-only** (`body_len = 0`, data in `reserved`), so they never need a buffer: `EO_START`, `EO_STOP`, `TIMER_EXPIRED`, `IO_READY`.

### 3.1 Message ID

The scheduler reserves **only** the system range `0x0000_0000..=0x0000_FFFF` and treats every other value as opaque.

Convention for the other IDs (defined in `common`, not in the scheduler): **grouped by function, not by NF**.

```
msg_id = family (16 bits) << 16 | message (16 bits)

family = a functional procedure, independent of which NF sends or receives it:
         0x0001 UE registration, 0x0002 PDU session establishment,
         0x0003 N4 session management, 0x0004 NF discovery, ...
message = a message within that procedure (setup request, setup response, release, ...)
```

- A message shared by several NFs (e.g. SMF→UPF and UPF→SMF in the same procedure) keeps one ID and lives in one place.
- The message's role (request, response, notification) goes in `flags`, not in the ID. Together with `corr_id`, this is enough to pair requests and responses.
- All families are declared in one file in `common`, with a test that no ID is used twice.

## 4. Message bodies: zero copy

A body is a fixed-layout `repr(C)` struct, written **directly into a buffer owned by the receiving EO** and read in place. Nothing is serialized and nothing is copied.

```rust
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]   // zerocopy derives
#[repr(C)]
pub struct PduSessionReq { pub ue_id: u64, pub snssai: u32, pub dnn: [u8; 64], /* ... */ }

impl MsgBody for PduSessionReq { const MSG_ID: u32 = 0x0001_0001; }

// sender
let mut out = ctx.alloc::<PduSessionReq>(smf)?;   // buffer taken from smf's pool, zeroed
out.ue_id = 42;
ctx.send(out)?;                                    // only the slot handle is pushed

// receiver
fn on_message(&mut self, msg: &Message, ctx: &mut Context) {
    if let Some(req) = msg.body::<PduSessionReq>() {  // checks msg_id, length and alignment
        /* ... */
    }
}   // buffer goes back to the pool when the handler returns
```

- `msg.body::<T>()` returns `None` if `msg_id != T::MSG_ID` or the length or alignment doesn't fit, so no unchecked casts.
- `ctx.send_value(to, &value)` is also available for small structs (one copy into the buffer).
- Buffers are aligned to 64 bytes (a cache line), and size classes are multiples of 64.

## 5. Mailbox and memory: one pool per queue

Each EO's mailbox owns **all** the memory sent to that EO, so memory needs can be tracked per EO.

| Part | Structure | Preallocated |
|---|---|---|
| System lane | bounded MPSC ring of headers (small, e.g. 64) | yes |
| Data lane | bounded MPSC ring of `MsgHeader + BufRef` | yes, `capacity` slots |
| BufferPool | N size classes, each one contiguous block + free list (bounded MPMC ring of indices) | yes, `count × size` per class |

- **Allocation happens on the destination's pool.** The sender asks for a buffer from the receiver's pool, in the smallest class that fits the body. If that class is empty, the buffer comes from the next larger class that has one (**fallback**, counted on the requested class). If none has one, the send fails *at the sender* with `Error::PoolExhausted` (counted on the requested class too). Overload therefore shows up exactly at the EO that can't keep up, and a class whose `high_water` reaches its `count` is undersized.
- **Bounded everywhere.** A full lane returns `Error::QueueFull` to the sender. Nothing grows and nothing blocks.
- **The system lane is drained first and is separate from the data lane**, so a data flood can't delay timers or `EO_STOP`. Consequence: system messages can overtake data messages. Data messages stay **FIFO per sender→receiver pair**.
- The queues are our own implementation (Vyukov-style array rings), checked with `loom`.

Configuration when an EO is created:

```rust
EoConfig::new("amf")
    .pool("cp")
    .data_capacity(1024)
    .system_capacity(512)                               // >= max_timers + max_io + 2
    .buffers(&[(256, 512), (2048, 256), (9216, 32)])    // (size, count) per class
    .max_timers(256)
    .max_io(8)
```

### 5.1 Allocation time

Preallocation moves the allocation cost to EO creation, so that cost is measured and kept under control:

- **Prefault**: every region is written once at creation (rings, buffer classes, timer table), so the first messages never trigger page faults.
- **Measured**: the time to create each EO (allocation + prefault) and the bytes reserved appear in `EoStats`. The runtime's total startup time is logged.
- **Off the workers**: creating an EO (at startup, or with `spawn` later) allocates on the caller's thread, never on a pool worker. The EO is published in the registry only once it's ready, so running EOs are never stalled by an allocation.
- **Later options**: `mlock` (no swapping) and huge pages for large pools (UPF).
- **Verified**: a counting global allocator shows zero allocations on the send, receive and timer paths (§11).

## 6. Scheduling (Event mode)

EO run state (atomic): `Idle → Scheduled → Running → (Idle | Scheduled)`.

1. **Post**: push to the mailbox. If a compare-and-swap moves the EO from `Idle` to `Scheduled`, push its handle onto the pool's **run queue** and wake a worker.
2. **Run**: a worker pops the EO, sets `Running`, and handles up to `budget` messages, system lane first.
3. **Yield**: if messages remain, set `Scheduled` and push the EO back to the end of the run queue. Otherwise set `Idle`, then **recheck** the mailbox: if a message arrived in between and the swap `Idle → Scheduled` succeeds, push the EO again. No wakeup is lost.

Invariants:
- An EO is in the run queue at most once. The run queue capacity is therefore the registry capacity, so it can never be full.
- Exclusivity: only the worker that set `Running` touches the EO's state, so handlers never need locks.

**Sleep and wakeup** (as implemented, see §16).
- Each worker has its own wake-up eventfd. A pool keeps a bitmask of its sleeping workers; whoever queues an EO clears one bit and writes that worker's eventfd. A worker sets its bit, rechecks the run queue, then sleeps (fences on both sides, so no wakeup is lost).
- Each pool has one epoll instance for its EOs' sockets. One sleeping worker at a time, the **I/O leader**, also watches it; a readiness event therefore wakes one worker, not all. Every awake worker polls it before sleeping, so readiness is never left unwatched.
- The sleep timeout is the worker's next timer deadline.
- A ready socket becomes an `IO_READY` system message (`reserved` = I/O token) to the EO that owns it; the EO does the actual `read`/`recv`.

**Poll and hybrid modes** (UPF milestone): same loop, but `epoll_wait(timeout = 0)` and no sleeping (Poll), or spinning for X µs and then sleeping (Hybrid). The loop is written from the start with a "poll sources" step so these modes slot in without a rewrite.

## 7. Timers

- Each worker has a **timer wheel** (default tick 1 ms, configurable). `ctx.arm_timer(after, tag)` inserts into the current worker's wheel, without a lock.
- At expiry, the worker posts a `TIMER_EXPIRED` system message (`reserved` = TimerId) to the EO, wherever it is.
- `ctx.cancel_timer(id)` disarms the timer in the **EO's timer table** (preallocated, `max_timers` entries, only accessed while the EO is running). That alone guarantees a cancelled timer is never delivered: an expiry that still arrives is dropped before the handler.
- The wheel entry is then removed to free its slot: directly if the wheel is the current worker's, otherwise through the owning worker's **cancel queue** (bounded). If that queue is full, the entry just stays until it expires (counted as `lazy_cancels`).
- Workers check their wheel before every turn, so a long turn on another EO never delays an expiry by more than that one turn.
- An expiry refused by a full system lane is put back in the wheel for the next tick, never lost.

## 8. EO API

```rust
pub trait ExecutionObject: Send + 'static {
    fn on_start(&mut self, _ctx: &mut Context<'_>) {}
    fn on_message(&mut self, msg: &Message, ctx: &mut Context<'_>);
    fn on_stop(&mut self, _ctx: &mut Context<'_>) {}
}
```

`Context` is the EO's only way to reach the outside world, and it has the same shape on the real runtime and on the test bed:

| Call | Purpose |
|---|---|
| `self_id()`, `now()` | identity, clock (virtual on the test bed) |
| `alloc::<T>(to)`, `send(out)`, `send_value(to, &v)` | messaging |
| `arm_timer(after, tag)`, `cancel_timer(id)` | timers |
| `register_io(fd, interest)`, `deregister_io(token)` | sockets → `IO_READY` |
| `stats(eo)` | read another EO's metrics (used by the metric EO) |
| `stop_self()` | lifecycle |
| `stats_and_reset_peaks(eo)` | same, then reset min/max/high-water |
| `spawn(cfg, eo)` | *later (M3)*: add an EO from inside an EO; `Runtime::add` already works while traffic flows |

## 9. Metrics and execution time

Each EO has an `EoStats` block made of atomics. Only the worker running the EO writes to it; anyone can read it without a lock.

- **Processing**: messages handled; total, min and max time per message; a histogram with power-of-two buckets (from 1 µs up to over 1 s). Measured with `Instant` around each `on_message` call.
- **Scheduling**: number of runs, budget exhaustions, time spent in the run queue (scheduling latency).
- **Mailbox**: current depth, `high_water`, `rejected` for each lane.
- **Memory**: for each buffer class, buffers in use, `high_water`, `alloc_failures`, bytes reserved.
- **Creation**: allocation + prefault time, and bytes reserved (§5.1).
- **Worker**: busy time vs idle time and EOs run, for each worker (used to verify load balancing).

API (same on `Runtime` and `TestBed`, and in `Context` for EOs):
- `stats(id) -> EoStats`
- `stats_and_reset_peaks(id) -> EoStats`: reads, then resets the min/max/high-water values, for periodic collection.
- `Runtime::worker_stats() -> Vec<WorkerStats>`: per worker, turns, messages, busy and idle time, wake-ups, I/O events, timers, cross-worker cancellations.

The **metric loop is itself an EO**, woken by a periodic timer, that reads snapshots of all EOs. It's provided as an example and will later be the source for exporting (Prometheus, logs, ...).

## 9.1 Failure handling: always find the root cause

Default policy: **report, then abort** (`panic = "abort"`). Docker or systemd restarts the process. An optional mode isolates the EO instead: the EO is marked `Faulted` and the others keep running (needs `panic = "unwind"`). The report is the same in both modes.

- Before each handler call, the worker records in a thread-local **which EO** is running and **which message** it's processing.
- Each EO keeps a **flight recorder**: a preallocated ring holding the headers of its last N messages (default 32, about 40 bytes each, copied at no real cost).
- A **panic hook** writes a report before aborting:
  - EO name and `EoId`, worker and core;
  - the full header of the message being processed (`msg_id`, `from`, `corr_id`, ...);
  - the flight recorder (the message history that led up to the panic);
  - a snapshot of the EO's `EoStats` (queue depth, memory, ...);
  - the panic message, its location, and a backtrace.
- The report goes to stderr (collected by Docker or journald) and, optionally, to a file. Core dumps can be enabled for post-mortem debugging.

## 10. Test bed (fake platform)

A single-threaded `TestBed` hosting real EOs, unchanged:

- `add(cfg, eo) -> EoId`, plus **probe** EOs that record everything they receive, for assertions.
- `inject(to, msg)`, `run_until_idle()`, `step()` (one message, fully deterministic).
- **Virtual clock**: `advance(Duration)` fires timers in order. Nothing depends on real time.
- I/O: real local sockets (loopback UDP, `socketpair`); `poll_io()` / `wait_io(timeout)` collect readiness without running anything. Fully fake sockets come later.
- Same metrics API as the runtime.

## 11. Test plan

**Units**
- MPSC / MPMC rings: FIFO order, full and empty behaviour, wrap-around; `loom` models for concurrent push/pop; `proptest` against a `VecDeque` model.
- BufferPool: alloc/free, exhaustion, every buffer returned, alignment.
- Timer wheel: ordering, cancel, re-arm, long delays (multiple wheel turns).
- EoId: a stale generation is rejected.
- `body::<T>()`: wrong id, length or alignment returns `None`.

**Test bed**: request/response between two EOs, timers on the virtual clock, cancel, stop.

**Required by the spec**
1. **Load balancing**: a pool of 4 workers, 16 EOs, each message doing about 50 µs of work.
   - Every worker gets a share of the busy time within a set tolerance.
   - **Exclusivity**: an "inside handler" flag checked in each EO never sees two workers at once.
   - **Ordering**: FIFO per sender→receiver pair holds even when an EO migrates between workers.
2. **Overload**: flood one EO beyond its capacity.
   - Senders get `QueueFull` / `PoolExhausted`; the `rejected` and `alloc_failures` counters are exact; `high_water == capacity`.
   - **No heap allocation** on the send or receive path, checked with a counting global allocator.
   - Timers and `EO_STOP` are still delivered during the flood (system lane).
   - **Recovery**: after the flood, depth returns to 0 and every buffer is back in the pool (no leak).
3. **Execution time API**: an EO that spins for a known time (e.g. 1 ms).
   - min/max/total/histogram are consistent with that time.
   - The metric EO collects snapshots periodically; `snapshot_and_reset_peaks` works.

4. **Failure report**: an EO that panics on a given message. The report identifies the EO, the message header, and the previous messages in the right order. Tested in unwind mode so the test can inspect the report.
5. **Allocation time**: creation time and bytes reserved are reported for each EO; zero allocations on the hot path.

**Benchmarks** (`criterion`): send→receive latency, throughput for 1 to N workers, cost of the measurement itself.

## 12. Crate layout and dependencies

```
LibreCN/
├── Cargo.toml            # workspace
├── scheduler/            # crate `librecn-scheduler`
│   ├── src/  id.rs header.rs message.rs queue.rs pool.rs mailbox.rs timer.rs io.rs
│   │         stats.rs metrics.rs failure.rs recorder.rs config.rs
│   │         core.rs registry.rs exec.rs context.rs eo.rs runtime/ testbed/
│   ├── tests/            # runtime (load balancing, overload, ...), test bed, failure, no_alloc
│   └── benches/          # criterion
├── common/               # placeholder
└── nf/                   # placeholder
```

| Dependency | Use |
|---|---|
| `rustix` | epoll, eventfd, CPU affinity: Linux syscalls with safe wrappers |
| `zerocopy` | derives that make the zero-copy bodies safe |
| `thiserror` | error types |
| `tracing` | logging, kept out of hot paths (not used yet) |
| dev: `loom`, `proptest`, `criterion` | tests and benchmarks only |

No Tokio and no async in the scheduler.

## 13. Milestones

- **M1, core and test bed**: ids, header, bodies, rings (+ loom), BufferPool, mailbox, stats, timer wheel, EO trait / Context, TestBed, unit tests and test-bed tests.
- **M2, multi-core runtime**: pools, pinned workers, run queue, Event mode with epoll, I/O registration, metric EO, then the **load-balancing, overload and execution-time tests**, and benchmarks.
- **M3, next**: Poll mode, then Hybrid (UPF); `ctx.spawn` (adding EOs from inside an EO); passing a buffer from one EO to another without copying (UPF forwarding).

## 14. Decisions

1. Handler panic: report, then abort by default; isolation is optional. The root cause must always be identifiable (§9.1).
2. Timer tick: 1 ms by default, configurable.
3. Dependencies in §12: approved. Allocation time is measured and kept off the hot path (§5.1).
4. `msg_id`: only the system range is reserved; the rest is grouped by functional procedure (§3.1).

## 15. Milestone 1: implementation notes

Decisions made while implementing, and limits to address in M2.

- **Ring capacity 1.** The textbook Vyukov queue needs capacity ≥ 2: with one slot, "published at p" and "free for p + capacity" have the same sequence value, and a push into a full ring overwrites the value. The property test found it. Sequences now count in half steps (free = `2p`, published = `2p + 1`), so every capacity ≥ 1 works.
- **Buffer classes fall back** to larger ones when empty (changed after M1 review, see §5). `fallbacks` and `alloc_failures` are counted on the class the body asked for.
- **Processing stats count every handled message**, system messages included (`on_start`, timers, ...). A failure report's stats snapshot is taken during the panic, so it counts the messages completed *before* the faulty one.
- **Timer cancellation.** The wheel entry is removed in O(1), checked by generation. An expiry already queued is dropped at delivery and counted as `timers_stale`. Cross-worker cancellation: see §7 and §16.
- **Test bed additions.** `FaultPolicy::Propagate` (the test bed default: the panic reaches the test) and `advance_clock`, which moves the clock and queues expired timers without running any EO, to build exact interleavings.


Test commands:

```
cargo test --workspace                     # unit, property, integration, doc tests
cargo test --workspace --release           # same, optimised (panic = abort for binaries)
RUSTFLAGS="--cfg loom" CARGO_TARGET_DIR=target/loom \
  cargo test -p librecn-scheduler --release --lib queue::loom_tests
```

## 16. Milestone 2: implementation notes

What the multi-core runtime does, and what was learned building it.

**Structure.** `Runtime::new(RuntimeConfig { pools, .. })` starts every worker (optionally pinned with `sched_setaffinity`) and fails if one cannot start or be pinned. The test bed and the runtime share the same EO state (`EoCore`), registry and turn logic (`exec::run_turn`), so behaviour verified on the test bed is the runtime's.

**EO-private state.** The EO object, flight recorder, timer table and I/O table sit in `UnsafeCell`s inside `EoCore`. Only the thread that moved the EO to `Running` touches them; the Acquire/Release run-state transitions order successive runners. This is the only `unsafe` contract outside the ring and the buffer pool.

**Registry and lookups.** One read lock per slot. Each worker caches `Arc<EoCore>` per slot, so the message path does not take the lock (a cached EO is checked by id and state). The test bed has no cache: its ~135 ns per message includes the lock.

**I/O.**
- Registrations are one-shot and level-triggered, re-armed by the platform once the handler returns: an EO never has more than one `IO_READY` waiting per registration (which bounds the system lane: `system_capacity >= max_timers + max_io + 2`), and need not drain a socket in one go.
- The platform keeps a duplicate of every registered descriptor (`F_DUPFD_CLOEXEC`), so re-arming and removing are always safe; the socket stays open until it is deregistered or the EO stops.
- If an `IO_READY` cannot be queued anyway, every registration of the EO is re-armed at the end of its next turn.
- Readiness may occasionally be spurious (a slot reused while an event was in flight): EOs must treat `WouldBlock` as normal.

**Found while testing.**
- `EPOLLEXCLUSIVE` is refused when the watched descriptor is itself an epoll instance, which was the original plan for sharing the pool's sockets between workers. Replaced by the I/O leader (§6).
- Timers were only checked between batches of 32 turns. With 3 ms turns, expiries were delayed by about 100 ms; the overload test caught it. Timers are now checked before every turn; I/O is polled after 32 turns or once per tick.
- A worker re-queuing its own EO woke a sleeping worker that found nothing to do (50 useless wake-ups in the overload test). A re-queue now wakes another worker only if more than one EO is waiting.
- `is_alive(id)` turns false when the EO closes; `alive()` stops counting it slightly later, once its resources are released. Both are documented.

**Results.** 64 tests, run repeatedly (60 runs in release, 20 in debug, also restricted to 1 and 2 CPUs with `taskset`) without failure:
- load balancing: 4 workers, 16 EOs, 6200 messages of 50 µs: every worker gets at least half its fair share of busy time, no EO ever runs on two workers at once, per-sender order holds while EOs migrate;
- overload from 4 threads: exact counters, fallback and exhaustion observed, timers overtake queued data, full recovery without leak;
- execution time and metric loop on real time;
- timers never early, cancelled timers never fire whichever worker cancels;
- UDP echo through the pool poller, one datagram per readiness;
- isolated fault, stop and shutdown (`on_stop` for every EO), EOs added while traffic flows, pinned workers stay on their cores;
- zero heap allocations on worker threads between messages (verified to detect an injected allocation).

Benchmarks (`cargo bench`, this machine, release): ring push+pop ~7 ns; one message on the test bed ~135 ns; round trip between two EOs on two workers ~815 ns.

**Open for M3 and later.**
- Poll and Hybrid pool modes (UPF).
- `ctx.spawn`, and the EO removal policy for faulted EOs (their slot is kept for the report today).
- Zero-copy buffer hand-off between EOs.
- Registry lookups take a read lock on a cache miss, and every send clones an `Arc` (one atomic on a shared line). Revisit with benchmarks once NFs exist.
- The failure report can only go to stderr; a file sink is still to do.
