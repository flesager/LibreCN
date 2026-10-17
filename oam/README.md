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
| `GET`, `POST`, `DELETE /api/nfs/{name}/{path}` | the resources of NF `name`, served by the NF itself |
| `GET /metrics` | Prometheus text |

It listens on `127.0.0.1:8080` by default; there is no authentication yet.
`[oam] log_level` (default `info`) sets every EO's log threshold.

### NF resources

The server forwards `/api/nfs/{name}/{path}` to the EO `name` as a
`common::manage::Manage` request (method, path, body; 8 KiB) and returns its
`Reply` (status and JSON), without knowing the NF. An NF serves what it
chooses; an NF without such resources has no 8 KiB buffer class, so the
server answers 503 at once. Today:

| NF | Request | Answer |
|---|---|---|
| UDM | `GET subscribers` | `[{"supi", "amf", "sqn"}]`: keys never leave the UDM |
| UDM | `POST subscribers` | adds one subscriber or an array, fields of the configuration (`supi`, `k`, `opc`, optional `amf`, `sqn`): 201, 409 if one exists |
| UDM | `DELETE subscribers/{supi}` | 200, or 404 |
| SMF | `GET sessions` | `[{"id", "supi", "pdu_session_id", "ue_ip", "dnn", "state"}]` |
| SMF | `DELETE sessions/{id}` | releases the session (the UE told through its AMF): 200, or 404 |
| AMF | `GET ues` | `{"connected", "registered", "idle", "ues": [...]}`, the first 64 UEs (`supi`, `cm`, `state`, `tmsi`, `sessions`) |
| AMF | `GET ues/{supi}` | that UE, or 404 |
| AMF | `POST ues/{supi}/ambr`, body `{"uplink_mbps", "downlink_mbps"}` | the connected UE's UE-AMBR, to its gNB (UE Context Modification) and with its sessions: 202, 400, 404 |
| AMF | `POST capacity`, body `{"relative_capacity"}` | the AMF's relative capacity, announced to every gNB (AMF Configuration Update): 202, 400 |
| AMF | `POST ues/{supi}/guti` | a new 5G-GUTI for the connected UE (Configuration Update Command, acknowledged): 202, 409 if idle or in another procedure, 404 |
| AMF | `DELETE ues/{supi}`, body `{"reregister": true}` optional | the network deregisters the UE: 202 if it is connected (Deregistration Request), 200 if idle (locally), 409 if in another procedure, 404 |

The GUI's Subscribers view uses them.

## Between the OAM EOs

Requests are small fixed bodies or header-only messages (`proto`); each gets a
`Reply` (16 KiB, `common::manage`) carrying the status and JSON of the web
answer, which the server passes through. A requester needs a 16 KiB buffer class to receive replies.
