# OAM: operations, administration and maintenance

The non-3GPP EOs. They run in the `oam` pool, so OAM traffic never takes a
worker from an NF. NFs reach them through `librecn-common` (`Logger`,
`Alarms`), which finds them by name and degrades gracefully when they are
absent.

| EO | Name | Role |
|---|---|---|
| `LogSink` | `log` | writes every EO's log records to stdout as JSON lines |
| `AlarmManager` | `alarm` | active alarms (X.733 severities), every change logged |
| `Control` | `control` | starts NFs from a deployment, stops EOs, raises a critical alarm when an EO faults |
| `Server` | `server` | web API and GUI (HTTP/1.1, one request per connection) |

Metrics have no EO: the scheduler's counters are read in place, and the server
renders them for Prometheus (`/metrics`).

## Logs

```rust
struct Amf { log: Logger }                       // Logger::new(Level::Debug) to see everything
log!(self.log, ctx, Info, "ue {} registered", ue_id);
```

- Levels: `VIP` (never filtered: an NF started), `FATAL`, `ERROR`, `INFO`, `DEBUG`.
- Formatted on the stack into a 256-byte record (text cut at 240 bytes, flagged `"truncated":true`) and sent to the log EO. No allocation, and a slow output never blocks the NF.
- No log EO: the line is printed to stderr, same format. Log EO overloaded: the record is dropped and counted in its stats, except `VIP` and `FATAL`, which are printed.
- Line: `{"ts":"2026-10-17T09:30:00.123Z","level":"INFO","eo":"amf-1","msg":"ue 7 registered"}`.
- In tests: `let (sink, captured) = LogSink::capture();` then `captured.lines()`.

## Alarms

```rust
self.alarms.raise(ctx, LINK_DOWN, Severity::Major, "N2 link to gNB 12 down");
self.alarms.clear(ctx, LINK_DOWN);
```

An alarm is keyed by its source EO and an id chosen by that EO; raising it again updates it. Up to 64 active.

## Deployments

TOML or JSON, the same in the configuration file, in the GUI and in `POST /api/nfs`:

```toml
[[nf]]
kind = "demo"        # a factory registered with Control::kind
name = "demo-1"      # unique among running EOs
pool = "cp"          # optional
[nf.params]          # optional, for the factory
period_ms = 500
```

The answer lists what started and what failed, NF by NF. OAM EOs cannot be stopped through the API.

## Web API

| Request | Answer |
|---|---|
| `GET /` | GUI |
| `GET /api/eos` | EOs, with their stats |
| `GET /api/workers` | workers |
| `GET /api/alarms` | active alarms |
| `GET /api/kinds` | NF kinds |
| `POST /api/nfs` | start NFs (deployment as body) |
| `POST /api/eos/{id}/stop` | stop an EO |
| `GET /metrics` | Prometheus text |

It listens on `127.0.0.1:8080` by default; there is no authentication yet.

## Between the OAM EOs

Requests are small fixed bodies or header-only messages (`proto`); each gets a
`Reply` (16 KiB) carrying the JSON of the web answer, which the server passes
through. A requester needs a 16 KiB buffer class to receive replies.
