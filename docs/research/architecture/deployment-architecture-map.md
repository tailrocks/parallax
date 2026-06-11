# Deployment Architecture Map: Local, Own Server, Cloud

<!-- markdownlint-disable MD013 -->

Research date: 2026-06-11. Companion to the [V1 build plan](v1-build-plan.md): the same one
binary and one API in all three pictures — only the **profile** changes where state lives and
how clients reach it. Profiles: `--profile local`, `--profile server`, `--profile cloud`
(the build plan's M3 ships `server` and `cloud` as presets of one server-side family).

## 0. Who holds what (the rule that never changes)

| Store | Role | Holds | Why this split |
| --- | --- | --- | --- |
| **GreptimeDB** | The telemetry evidence engine — high-volume, append-only, columnar | Spans, log records, metric points, derived `error_event` rows, rollup counters (fingerprint × minute) | This data is written once, queried by anchor (`trace_id`/`fingerprint`/time window), compressed 10×-class, aged by TTL. Columnar engines exist for exactly this. |
| **Turso** (local/dev) → **Postgres** (server/cloud option) | The product-state store — low-volume, mutable, relational | Projects, ingest/API tokens, **issues** (fingerprint → status/first/last seen/count/assignee), runs, retention & redaction policies, audit records, (later: outcome rows) | Issue state is OLTP: it gets UPDATEd (resolved, assigned, regressed). Columnar stores hate row updates — Sentry built a whole "replacements" pipeline to fake them in ClickHouse. We keep mutable state relational from day one. |
| **Local disk** | Spool + engine data in local/server profiles | Ingest WAL/outbox segments, GreptimeDB data dir, Turso file, raw-ref blobs with TTL | Cheap durability for the write path and the local tiers. |
| **Object storage** (cloud profile) | The only long-term copy | GreptimeDB SSTs (engine-native S3 backend), raw-ref archive, **pinned evidence slices** (bundle-cited raw data that must outlive TTL), bundle JSON archive | ~$0.021/GB-month, 1× copy (no replica multiplication) — the cost vertex of the [impossible triangle](../00-vision/north-star-autonomous-fix-loop.md). |

Clients never see any of these directly. CLI, UI, agents, MCP — everything goes through the one
Parallax API ([api-concept.md](api-concept.md)).

## 1. Angle A — the local developer (`--profile local`)

The daily-driver setup. One command; Parallax supervises everything; nothing to configure.

```text
 developer laptop ────────────────────────────────────────────────────────────────┐
 │                                                                                │
 │   your apps under development              coding agent (Claude Code, …)       │
 │   ┌──────────────┐ ┌──────────────┐        ┌──────────────────────────┐        │
 │   │ rust service │ │ cli tool /   │        │  parallax CLI            │        │
 │   │ (tracing +   │ │ TUI (jackin) │        │  (context: local =       │        │
 │   │  otel-otlp)  │ │              │        │   http://127.0.0.1:4000) │        │
 │   └──────┬───────┘ └──────┬───────┘        └────────────┬─────────────┘        │
 │          │ OTLP gRPC :4317 │ OTLP HTTP :4318            │ GraphQL/HTTP :4000   │
 │          ▼                 ▼                            ▼                      │
 │   ┌─────────────────────────────────────────────────────────────────┐          │
 │   │                    parallax serve  (ONE process)                │          │
 │   │                                                                 │          │
 │   │  tonic OTLP receiver ──► WAL spool ──► derive / fingerprint /   │          │
 │   │  axum OTLP-HTTP + API               group / rollup workers      │          │
 │   │  GraphQL API  ◄── bundle builder (bound + redact + hypotheses)  │          │
 │   │  local UI (later, same port)                                    │          │
 │   │                                                                 │          │
 │   │  child-process supervisor ───────────────┐                      │          │
 │   └──────────────┬───────────────────────────┼──────────────────────┘          │
 │                  │ SQL (in-process)          │ spawns + health-checks          │
 │                  ▼                           ▼                                 │
 │   ┌────────────────────────┐   ┌──────────────────────────────────┐            │
 │   │ Turso file             │   │ greptime standalone (child proc) │            │
 │   │ ~/.parallax/meta.db    │   │ data: ~/.parallax/greptime/      │            │
 │   │ projects, issues, runs,│   │ spans, logs, metrics,            │            │
 │   │ tokens, policies       │   │ error_events, rollups            │            │
 │   └────────────────────────┘   └──────────────────────────────────┘            │
 └────────────────────────────────────────────────────────────────────────────────┘
```

**Setup experience:**

```bash
brew install parallax            # or one static binary download
parallax serve                   # default --profile local --manage-greptime
# in your app's environment:
export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317
cargo run                        # telemetry flows; panics become grouped issues
parallax run inspect <run_id>    # or your agent does this
```

- **GreptimeDB here:** a *managed child process* (`greptime standalone start`), installed via
  the Greptime brew tap or downloaded/pinned by Parallax; Parallax owns spawn, health, restart,
  and version pin. Escape hatches: `--greptime-url` (use one you already run) and
  `--no-greptime` (tiny Turso-only fallback for demos — bounded telemetry, not the default).
- **Turso here:** one in-process file. No server, no socket, nothing to operate.
- No auth in local profile (loopback only by default). Short TTLs, manual prune.
- The agent uses the same CLI you do; `--context local` is the implicit default.

## 2. Angle B — your own server (`--profile server`)

The growing-startup setup the old world solved with Sentry+Grafana+Loki. Here: one binary as a
service, local disks, same API, reached from every laptop kubectl-style.

```text
 your server (VM / bare metal) ───────────────────────────────────────────────┐
 │                                                                            │
 │   ┌──────────────────────────────────────────────────────────────┐         │
 │   │            parallax serve --profile server  (systemd/container)        │
 │   │  OTLP :4317/:4318  (x-parallax-project-token required)       │         │
 │   │  GraphQL API + UI :4000 (TLS via your proxy)                 │         │
 │   │  workers: derive / group / rollup / triggers (issue-only)    │         │
 │   └───────┬──────────────────────────────┬───────────────────────┘         │
 │           │                              │                                 │
 │           ▼                              ▼                                 │
 │   ┌───────────────────────┐   ┌───────────────────────────────────┐        │
 │   │ metadata store        │   │ GreptimeDB standalone             │        │
 │   │ Turso file (default)  │   │ (managed child OR --greptime-url  │        │
 │   │ or Postgres           │   │  to a separate host)              │        │
 │   │ (--metadata postgres) │   │ data on local SSD, per-signal TTL │        │
 │   └───────────────────────┘   └───────────────────────────────────┘        │
 │           ▲ backups: file snapshot / pg_dump      ▲ disk snapshots         │
 └───────────┼───────────────────────────────────────┼────────────────────────┘
             │                                       │
   ──────────┼───────────────────────────────────────┼──────────────
             │ OTLP + tokens                         │
 ┌───────────┴─────────────┐            ┌────────────┴────────────┐
 │ your deployed services  │            │ developer laptops       │
 │ (Rust backends, frontends│           │ parallax CLI:           │
 │  via OTLP-HTTP, CI deploy│           │  context "prod" = URL + │
 │  events parallax.deploy) │           │  API token; UI in browser│
 └─────────────────────────┘            └─────────────────────────┘
```

**Setup experience:**

```bash
# on the server
parallax serve --profile server --data-dir /var/lib/parallax   # one unit file
parallax project create checkout    # prints ingest token
# on each laptop
parallax context add prod --url https://parallax.internal --token <api-token>
parallax --context prod issue list
```

- **GreptimeDB here:** still standalone (managed child by default; point at a separate host with
  `--greptime-url` when telemetry volume wants its own box). Data on local SSD; retention by
  per-signal TTL; whole-SST drops make retention cheap.
- **Turso vs Postgres here:** Turso file remains the zero-ops default; switch to Postgres
  (`--metadata postgres://…`) when you want operational durability guarantees (backups,
  replication, multiple Parallax processes later). The adapter boundary makes this a config
  change, not a migration project — though the Turso production gates
  ([metadata-store.md](../decisions/metadata-store.md)) say Postgres is the safer answer once
  this server matters.
- Auth is on: per-project ingest tokens for OTLP, API tokens for humans/agents.
- Deploy events arrive from CI (`parallax.deploy.v0`) and make deploy-adjacent regressions
  visible (trigger creates/escalates the issue — nothing auto-dispatches).

## 3. Angle C — cloud machine + cloud storage (`--profile cloud`)

The retention-economics setup: compute is disposable, **object storage is the only copy**.

```text
 cloud account ───────────────────────────────────────────────────────────────────┐
 │                                                                                │
 │   cloud VM (disposable compute)                                                │
 │   ┌──────────────────────────────────────────────────────────────┐             │
 │   │        parallax serve --profile cloud                        │             │
 │   │  OTLP :4317/:4318 (tokens)  ·  GraphQL API + UI :4000 (TLS)  │             │
 │   │  workers: derive / group / rollup / triggers                 │             │
 │   │  evidence pinning: bundle-cited raw slices → object storage  │             │
 │   └───────┬───────────────────────────────┬──────────────────────┘             │
 │           │                               │                                    │
 │           ▼                               ▼                                    │
 │   ┌─────────────────────┐   ┌──────────────────────────────────────┐           │
 │   │ managed Postgres    │   │ GreptimeDB (standalone now,          │           │
 │   │ (RDS/Cloud SQL/...) │   │  distributed later — same seam)      │           │
 │   │ recommended default │   │  storage backend = OBJECT STORAGE    │           │
 │   │ for this profile;   │   │  local NVMe = cache + memtables only │           │
 │   │ Turso file still    │   └──────────────┬───────────────────────┘           │
 │   │ fine for one-VM     │                  │ engine-native S3 API              │
 │   └─────────────────────┘                  ▼                                   │
 │                              ┌─────────────────────────────────────┐           │
 │                              │ object storage (S3 / R2 / GCS / B2) │           │
 │                              │  • GreptimeDB SSTs (1× copy)        │           │
 │                              │  • raw-ref archive (TTL'd)          │           │
 │                              │  • PINNED evidence slices + bundles │           │
 │                              │    (outlive TTL — audit trail)      │           │
 │                              │  • lifecycle: hot → IA → archive    │           │
 │                              └─────────────────────────────────────┘           │
 └────────────────────────────────────────────────────────────────────────────────┘
              ▲ OTLP + deploy events                    ▲ GraphQL + UI
   your services (any language, std OTel)     laptops / agents: parallax --context cloud …
```

**Setup experience:**

```bash
parallax serve --profile cloud \
  --object-store s3://acct-parallax-evidence?region=… \   # passed through to GreptimeDB
  --metadata postgres://…                                  # managed Postgres recommended
```

- **GreptimeDB here:** the same engine with its **object-storage backend** — SSTs live in
  S3/R2/GCS, the VM's NVMe is only cache and memtables. This is why the engine was chosen: the
  1× shared-object-store design (no N× replica copies) is the architecture half of the cost
  story ([storage-cost-and-tiering.md](../storage/greptimedb-vs-clickhouse/storage-cost-and-tiering.md)).
  Kill the VM, start another, point it at the same bucket — the evidence is the bucket.
  Egress-free providers (R2/B2) fit re-read-heavy use best
  ([size-and-object-cost.md](../storage/size-and-object-cost.md)).
- **Metadata here:** managed Postgres is the recommended default (issue state and tokens deserve
  the cloud's backup/replication machinery); a Turso file remains acceptable for a single-VM
  setup with snapshots.
- **Tiering and pinning:** raw telemetry ages out by per-signal TTL; rollups stay; every raw
  slice a bundle cites is pinned to the bucket first — bills shrink, audit trails don't break.
- **The growth seam (rung 3, designed-under, not built):** the same profile later splits into
  ingest nodes / API nodes / worker pools and GreptimeDB standalone → distributed (frontend +
  datanodes over the same bucket), with an optional stream (Iggy) between ingest and workers.
  Topology change, not rewrite — the seams exist in the crate layout from M0.

## 4. One picture, three placements

| | A: local | B: own server | C: cloud |
| --- | --- | --- | --- |
| Parallax process | `parallax serve` (foreground) | systemd/container unit | disposable VM/container |
| GreptimeDB | managed child, data in `~/.parallax/greptime/` | managed child or separate host, local SSD | object-storage backend; NVMe = cache only |
| Metadata | Turso file | Turso file → Postgres option | managed Postgres (Turso ok for one VM) |
| Long-term evidence | short TTL, prune | SSD + TTL, disk snapshots | object storage, lifecycle tiers, pinned slices |
| Auth | none (loopback) | ingest + API tokens, TLS at proxy | same + cloud IAM for the bucket |
| Clients | same-machine CLI/agent/UI | kubectl-style remote contexts | same as B |
| Setup | one command | one unit + one context | one command + bucket + Postgres URL |

Same binary, same API, same bundle, same CLI grammar everywhere — `--context` picks the world,
the profile picks where state lives, and the [StorageAdapter](../decisions/v1-storage-adapter-vision.md)
keeps every one of these placements swappable.
