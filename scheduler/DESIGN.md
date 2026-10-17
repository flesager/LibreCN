# LibreCN scheduler: design

A reactor scheduler hosting **execution objects** (EOs). An EO hosts one network function and handles messages **run-to-completion**. Linux, baremetal or Docker, kernel network stack.

## 1. Principles

- **SOLID, mostly.** One responsibility per module; EOs depend on the `Context` abstraction, never on the platform running them.
- **Light is right** (Colin Chapman). No allocation on the hot path, few dependencies, few concepts.
- **Perfection is reached when there is nothing left to take away** (Saint-Exupéry). Every feature, option and counter must earn its place; §11 lists what was removed or deliberately left out.

## 2. Model

```
 Runtime
 ├── Registry: fixed table of EOs, EoId = slot index + generation
 └── Pools (e.g. "cp", "up"): 1..64 worker threads each, optionally pinned to cores
      ├── run queue of ready EOs, shared by the pool's workers
      ├── epoll instance for the sockets of the pool's EOs
      └── per worker: timer wheel, wake-up eventfd

 EO = handler object + mailbox
      mailbox = system lane (header-only, drained first) + data lane
                + the buffer pool holding every body sent to this EO
```

- **Everything is a message**: other EOs' messages, timer expiries, socket readiness, polls, lifecycle. The header says which.
- An EO belongs to one pool and runs on **any** of its workers, **never on two at once**, so its state needs no lock.
- The same EO code runs on the multi-core `Runtime` and on the single-threaded `TestBed`.

## 3. Messages

```rust
#[repr(C)]
pub struct MsgHeader {   // 40 bytes, fixed layout
    pub msg_id:   u32,   // 0..=0xFFFF reserved for the scheduler, the rest is opaque
    pub body_len: u32,   // 0 = header-only
    pub from:     EoId,  // EoId::SYSTEM for timers, I/O, polls, lifecycle
    pub to:       EoId,
    pub corr_id:  u32,   // request/response correlation; timer tag; readiness bits
    pub flags:    u32,   // free for the application (convention in `common`)
    pub reserved: u64,   // system messages: timer id, I/O token
}
```

- A stale `EoId` (its slot was reused) is refused with `UnknownEo`, never delivered to the new occupant.
- System messages: `EO_START`, `EO_STOP`, `TIMER_EXPIRED`, `IO_READY`, `EO_POLL`. All header-only, so they never need a buffer.
- **Message-id convention** (in `common`, not the scheduler): `family << 16 | message`, families grouped by **procedure** (registration, PDU session, N4 session, ...), not by NF. The role (request, response, notification) goes in `flags`.

**Bodies are zero-copy**: a fixed-layout struct written directly into a buffer of the **receiver's** pool and read in place.

```rust
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)]   // zerocopy
#[repr(C)]
struct PduSessionReq { ue_id: u64, snssai: u32, dnn: [u8; 64] }
impl MsgBody for PduSessionReq { const MSG_ID: u32 = 0x0002_0001; }

let req = ctx.alloc::<PduSessionReq>(smf)?.with_corr_id(7);   // zeroed buffer in smf's pool
ctx.send(req)?;
// receiver: msg.body::<PduSessionReq>() checks id, length and alignment
```

Data messages are FIFO per sender→receiver pair. System messages overtake data messages.

## 4. Memory

Everything an EO uses is allocated and **prefaulted** when it is created, on the creating thread, never on a worker. Creation time and bytes reserved are in its stats.

| Part | Structure |
|---|---|
| System lane | bounded ring of headers |
| Data lane | bounded ring of header + buffer handle |
| Buffer pool | size classes (64-byte aligned), one contiguous block + free list each |
| Timer and I/O tables, flight recorder | fixed arrays |

- **One pool per mailbox**, so memory needs show per EO.
- A body takes the smallest class that fits. If that class is empty it **falls back** to the next larger one with a free buffer. If none has one, `PoolExhausted`. Fallbacks and failures are counted on the class the body asked for: `high_water == count` plus fallbacks means that class is undersized.
- Full lane: `QueueFull`. Errors go to the sender. Nothing grows, nothing blocks.
- `system_capacity >= max_timers + max_io + 3` is enforced (start, stop, poll).
- The rings are a Vyukov bounded queue with half-step sequence numbers, so capacity 1 works. Checked with loom and proptest.

## 5. Scheduling

Run state per EO: `Idle → Scheduled → Running → Idle | Scheduled | Closed`. Whoever moves it `Idle → Scheduled` queues it, so an EO is queued at most once; only the thread that set `Running` touches its private state.

A worker loops:
1. apply cancellations sent by other workers, and fire due timers (before **every** turn);
2. collect the pool's socket readiness once per tick while busy;
3. pop an EO and run up to `budget` messages, system lane first; re-queue it if messages remain;
4. with nothing to run: collect readiness, then sleep in `epoll_wait` until the next timer.

`Runtime::shutdown` stops the EOs in reverse creation order, each one finishing before the next is asked: services created first (the log EO, ...) still serve the `on_stop` of the EOs created after them.

**Wake-ups.** A pool keeps a bitmask of sleeping workers. Queuing an EO wakes one through its eventfd, with fences on both sides so no wake-up is lost. Re-queuing its own EO wakes another worker only if more than one EO is waiting. One sleeping worker at a time, the **I/O leader**, also watches the pool's epoll, so a socket event wakes one worker, not all of them. Every awake worker collects readiness before sleeping, so readiness is never left unwatched.

**Poll and hybrid modes.** `ctx.set_polling(true)` makes the platform deliver one `EO_POLL` per turn and keep the EO runnable, so its worker never sleeps (busy polling of rings or non-blocking sockets). An EO that turns polling off after N empty polls, and back on at its next `IO_READY` or message, runs in hybrid mode. The EO decides, because it alone knows its traffic. Polls are not kept in the flight recorder. On the test bed, polls are delivered only on `TestBed::poll()`.

## 6. Timers

- `ctx.arm_timer(after, tag)` inserts into the current worker's hashed wheel (tick 1 ms by default). It never fires early. An expiry is a `TIMER_EXPIRED` system message to the EO, wherever it runs.
- The EO's timer table is the source of truth: a **cancelled timer is never delivered**, even if its expiry is already queued (counted as `timers_stale`).
- The wheel entry of a cancelled timer is removed directly if it is in this worker's wheel; otherwise it goes through the owner's bounded cancel queue, or is left to expire (counted as `lazy_cancels`).
- An expiry refused by a full system lane goes back in the wheel for the next tick.

## 7. I/O

- `ctx.register_io(&socket, Interest::READABLE)` returns a token. Readiness arrives as `IO_READY`, read with `msg.io()`. The EO does the actual `recv`.
- Registrations are one-shot and level-triggered, **re-armed by the platform at the end of the turn** that handled their `IO_READY`: at most one `IO_READY` per registration is waiting, at most one per turn is handled, and the EO need not drain the socket in one turn. Re-arming after each handler let a socket that is always ready (the UPF under traffic) fill the system lane again during the next handler, so the data lane, drained after it, starved (§11).
- The platform keeps a duplicate of the descriptor until `deregister_io` or the EO stops, so re-arming and removing are always safe.
- If an `IO_READY` cannot be queued, every registration of that EO is re-armed at the end of its next turn. Readiness may occasionally be spurious; `WouldBlock` is normal.

## 8. EO API

```rust
pub trait ExecutionObject: Send + 'static {
    fn on_start(&mut self, _ctx: &mut Context<'_>) {}
    fn on_message(&mut self, msg: &Message<'_>, ctx: &mut Context<'_>);
    fn on_stop(&mut self, _ctx: &mut Context<'_>) {}
}
```

| `Context` | |
|---|---|
| `self_id`, `now` | identity, platform time (virtual on the test bed) |
| `alloc` + `send`, `send_value`, `signal` | messages (zero-copy, copied, header-only) |
| `arm_timer`, `cancel_timer` | one-shot timers |
| `register_io`, `deregister_io` | socket readiness |
| `set_polling` | poll / hybrid mode |
| `spawn` | create an EO, in any pool |
| `stop(id)` | stop any EO, this one included: `on_stop` runs next, then it is removed |
| `name`, `eos`, `find(name)` | own name, list of EOs, a well-known service by name |
| `stats`, `stats_and_reset_peaks`, `worker_stats` | metrics of any EO, of the workers |

## 9. Observability and failures

**Per EO** (`stats(id)`): processing time per message (count, total, min, max, histogram in powers of two from 1 µs), turns, budget exhaustions, timers fired and stale, I/O events and stale; for each lane, depth, `high_water`, `rejected`; for each buffer class, `in_use`, `high_water`, `fallbacks`, `alloc_failures`; creation time and bytes; the EO's own counters, named in its `EoConfig` (`counters`) and incremented by `ctx.count(i, n)`, one relaxed atomic add (the UPF's packets and bytes). `stats_and_reset_peaks` resets min, max and high-water marks for periodic collection. The counters are atomics any EO reads through its `Context`, so the metric loop needs no EO of its own: the OAM web server exports them (Prometheus histograms, so no reader resets anything).

**Per worker** (`Runtime::worker_stats`): turns, messages, busy time, idle time (current sleep included), wake-ups, I/O events, timers, remote and lazy cancellations.

**Failures**: a panic in a handler produces a report with the EO, thread and CPU, the message being handled, the previous headers (flight recorder, 32 by default), the EO's stats, the panic location and a backtrace. Policies: `Abort` (default: report then abort, let Docker or systemd restart), `Isolate` (EO marked faulted, the others run on), `Propagate` (test bed: the panic reaches the test).

## 10. Testing

- **Test bed**: single thread, virtual clock (`advance`, `advance_clock`), explicit `step`, `run_until_idle`, `poll_io` and `poll`; real loopback sockets; `Probe` EO that records what it gets.
- **Units**: ring (proptest model, loom), buffer pool, timer wheel and tables, I/O table, histogram.
- **Runtime, real threads**: load balancing (4 workers, 16 EOs: fair share, exclusivity, per-sender order under migration), overload from 4 threads (exact counters, fallback, timers overtake data, recovery without leak), execution time and metric loop, timers never early and never delivered once cancelled, UDP echo, busy and hybrid polling, isolated fault, stop and shutdown, spawn across pools, pinning.
- **Zero allocation** on the test bed message path and on worker threads between messages (counting allocator, proven to catch an injected allocation).
- Stress (`scripts/stress.sh`, outside `cargo test` so the trunk stays fast): repeated release runs, runs restricted to 1 and 2 CPUs, loom.

```
cargo test --workspace          # everything, ~2 s
scripts/stress.sh [RUNS]        # slow checks
cargo bench -p librecn-scheduler
```

Benchmarks (release, dev machine): ring push+pop ~7 ns; one message on the test bed ~135 ns (alloc, send, schedule, handle, timing); round trip between two EOs on two workers ~800 ns.

Dependencies: `rustix` (epoll, eventfd, affinity), `zerocopy`, `thiserror`; tests only: `proptest`, `loom`, `criterion`. No async runtime.

## 11. Decisions and lessons

**Taken away**
- *Pool modes* (`Event` / `Poll` / `Hybrid` with spin settings): replaced by one EO flag, `set_polling` (§5).
- *Message roles* (`flags::REQUEST`, ...): an NF convention the scheduler never reads, moved to `common`.
- *`MetricCollector`*: metrics are read in place by whoever needs them (§9); the collector survives as a test helper.
- *`stop_self`*: `stop(ctx.self_id())`.
- *"Poll I/O every 32 turns"*: readiness is polled once per tick and before sleeping.
- *Duplicates*: the two EO-creation paths are now one (`Registry::create`, also behind `ctx.spawn`); `OutMsg` setters that duplicated the builders; public readiness bits that duplicated `IoReady`'s methods; `Time::elapsed_since_start`; public `raw()` on opaque handles.

**Deliberately not done**
- *Zero-copy hand-off of a received buffer to another EO.* It would put buffers of one EO's pool in another's mailbox, so each slot would carry its owner and pay a reference count on every forward. That breaks "an EO's pool holds what is sent to it". Copying a packet costs tens of nanoseconds; to be reconsidered with a real UPF profile.
- *Multiple instances of one NF, multi-process messaging, a file sink for failure reports, `tracing`*: not needed yet.

**Learned the hard way**
- The textbook Vyukov ring breaks at capacity 1; the property test found it.
- `EPOLLEXCLUSIVE` is refused on a nested epoll descriptor, which gave the I/O leader.
- Timers checked only every 32 turns were 100 ms late under long turns; the overload test found it.
- Worker idle time that only counted finished sleeps reported 0 for a worker asleep for a long time.
- Re-arming a socket right after its handler starved the data lane on two workers or more: with two sockets always ready, each was harvested again by another worker while the EO handled the other, so the system lane never ran dry. The UPF under traffic held the SMF's N4 requests for up to 1.4 s, and 3 or 4 cores stalled registrations; the whole-core load benchmark found it. Sockets are now re-armed at the end of the turn.
- `is_alive(id)` turns false when the EO closes; `alive()` stops counting it once its resources are released, slightly later.

**Open**
- Every send clones an `Arc` and a cache miss takes a per-slot read lock; revisit with NF benchmarks.
- Removal policy for faulted EOs (their slot is kept for the report).
