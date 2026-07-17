# V2 supported server profile (contract v1)

- **Status:** Implemented supported profile; plan 115 residual closed 2026-07-17
  ([residual-closure](../validation/2026-07-plan-115-v2-server-profile/residual-closure-2026-07-17.md))
- **Contract version:** 1
- **Decision date:** 2026-07-17
- **Approved by:** alexey@chainargos.com (unblock directive)
- **Plan owner:** plan 115
- **Depends on:** plan 109 auth minimal (DONE), plan 102 release four-target
  proof (DONE)

## Decision

Parallax supports **exactly one** V2 server profile for the first non-local
deployment: a **single-node operator box** that co-hosts GreptimeDB (managed or
external URL), Turso metadata, the Parallax binary, and optional local UI
assets. There is no multi-node HA profile, no alternate storage engines, and no
in-memory product mode.

| Field | Contract |
| --- | --- |
| Topology | 1 VM/host, one `parallax serve` process, one GreptimeDB, one Turso file |
| Storage | GreptimeDB + Turso only; native OTLP tables for raw signals |
| TLS | OS native TLS only (never rustls); terminate TLS at the OS / reverse proxy for remote binds |
| Auth | Shipped plan 109 bearer contract required when `server.bind` is non-loopback |
| Ingest tokens | Optional OTLP project token deferred until measured need; default open OTLP on trusted network only |
| Ports (defaults) | API/UI `4000`, OTLP gRPC `4317`, OTLP HTTP `4318` |
| Health | `GET /health` open; ready banner names every surface |
| Retention | Config-driven TTLs and shipped `parallax prune` (closed plan 116); no silent infinite growth |
| Backup | Offline file-level: Greptime data dir + Turso `meta.db` + config; restore = stop → replace → start |
| Unsupported | Postgres/ClickHouse/SQLite product engines, multi-writer Turso, multi-region active-active, rustls, hidden fallback stores |

## Hardware floor (minimum supported)

| Resource | Floor | Notes |
| --- | ---: | --- |
| CPU | 4 vCPU | Ingest + query + managed Greptime |
| RAM | 8 GiB | 16 GiB recommended when co-hosting playground / multi-backend lab |
| Disk | 100 GiB SSD | Size for TTL window × ingest rate; monitor free space |
| Network | 1 Gbps class | Local/LAN first; WAN via operator TLS edge |

These floors are **support gates**, not marketing SLOs. Workloads that exceed
them are unsupported until a later measured profile is written.

## Workload envelope (v1 claim)

| Signal | Envelope (single-node claim) |
| --- | --- |
| Sustained OTLP ingest | ≤ 5k spans/s **or** ≤ 20k log records/s (whichever hits first) |
| Concurrent GraphQL clients | ≤ 10 interactive humans/agents |
| Issue registry | ≤ 100k open fingerprints |
| Spool | Config max segment/total/age; durable ack before worker |

Exceeding the envelope is not a bug report against this profile; open a new
measured profile or shrink load.

## Trust boundary

```text
[ agents / CLI / UI ] --bearer--> [ parallax :4000 ]
[ OTLP SDKs ] -------------------> [ parallax :4317/:4318 ]
                                       |
                                       +--> GreptimeDB (native tables)
                                       +--> Turso meta.db
[ GitHub webhooks ] --HMAC------> [ /webhooks/github ] (optional, disabled default)
[ Sentry SDK ] --public key-----> [ /api/<project>/envelope ] (optional, disabled default)
```

- Remote binds **must** use the shipped bearer contract; validation fails otherwise.
- Webhooks and Sentry adapter stay **opt-in** and off by default.
- No product path may enable rustls or ship a custom CA bundle as the only trust store.

## Operability

| Concern | v1 rule |
| --- | --- |
| Progress | Long-running CLI narrates startup, downloads, ready banner |
| Drain | Graceful shutdown aborts listeners then drains ingest workers |
| Upgrade | Replace binary + config; engines upgrade via managed supervisor or operator external upgrade |
| Rollback | Restore prior binary + data dirs from backup |
| Doctor | `parallax doctor` reports storage/auth/spool; deploy-context inventory shipped with plan 121 (DONE) |

## Plan 110 gate

Plan 110 (ingest concurrency) may open **only** after a load packet on this
profile proves the single worker is the bottleneck (not disk/network/Greptime).
Until then, single-worker remains mandatory.

## Implementation status

| Slice | State |
| --- | --- |
| ADR (this file) | landed |
| Validated server config composition + rehearsals | landed (live + residual packets 2026-07-17) |
| Release package install on supported targets | landed (plan 102 four-target + host install dogfood) |
| Remote CLI contexts + TLS edge | landed (HTTPS edge dogfood) |
| OTLP ingest tokens | deferred — open OTLP on trusted network only; optional tokens if public remote ingest opens |
| Plan 110 concurrency | **closed** — single-worker retained; load packet under envelope without worker stage isolation |

## STOP

- Alternate DB engines, rustls, unauthenticated non-loopback bind, or speculative
  concurrency before measurement.
