# V1 Implementation Spec: The Concrete Contracts

<!-- markdownlint-disable MD013 -->

Research date: 2026-06-12. This is the layer between the concept docs and the first commit: the
concrete schemas, mappings, pins, and conventions an implementing agent needs so that
[v1-scope.md](v1-scope.md) + [simple-ui-v2.md](simple-ui-v2.md) are executable without
re-deriving decisions. Read order for an implementer: **v1-scope (what) → v1-build-plan (order)
→ this spec (contracts) → simple-ui-v2 (UI) → rust-stack-instrumentation (what arrives)**.
PoC kernels graduate per [poc-evidence-loop-coverage.md](poc-evidence-loop-coverage.md).

Operator note (2026-06-12): Juniper tracing is deferred — the operator instruments his own
resolvers; Parallax only consumes whatever spans arrive.

## 1. Workspace conventions

- Rust edition 2024; toolchain pinned via `rust-toolchain.toml` (current stable; 1.96 at spec
  time). Workspace at repo root: `crates/*` + `ui/` + existing `poc/` (frozen).
- Lints: `cargo clippy --workspace --all-targets -- -D warnings`; `cargo fmt --check` in CI —
  both strict, zero tolerated warnings (operator rule, 2026-06-12).
- Test runner: **cargo-nextest** (`cargo nextest run --workspace`; operator rule, 2026-06-12).
  The gated real-engine test stays behind nextest's ignored filter.
- Modernity rule (operator, 2026-06-12): follow the latest recommended practices of every
  ecosystem touched — Rust (current idioms, edition 2024), TypeScript/React/TanStack/shadcn
  (their current official guidance) — re-checked whenever a layer is touched.
- Errors: `thiserror` in library crates, `anyhow` at binary edges. No `unwrap()` outside tests.
- Tests: unit beside code; integration tests under `crates/parallax-server/tests/` driven by
  **real SDK emission** (tracing + opentelemetry-otlp) against an in-process server with the
  in-memory storage adapter; golden bundle tests reuse PoC fixtures.
- Logging: the server uses `tracing` itself; never exports its own telemetry to itself by
  default (loop guard).

## 2. Dependency versions — policy: always latest (operator, 2026-06-12)

**Rule: use the latest stable version of everything, everywhere.** The table below is NOT a
freeze — it is the **known-mutually-compatible floor verified on 2026-06-12**. At implementation
start (and on every later dependency touch) the agent resolves the **latest mutually-compatible
stable set** — "latest" in the OTel ecosystem means the matched release train (otel core ⇄
tracing-opentelemetry ⇄ middleware crates move in lockstep; never mix trains) — and **updates
this table to the resolved set in the same commit**. Pre-release/RC versions only when no stable
exists for a required piece.

| Area | Compatible floor (2026-06-12) |
| --- | --- |
| Runtime | tokio 1.x, axum 0.8, tonic 0.14, tower 0.5 |
| OTel ingest types | opentelemetry-proto 0.32 (`gen-tonic`, `with-serde`) |
| GraphQL server | **Juniper 0.17** (operator instruction, 2026-06-12 — the library he uses in his own services; replaces async-graphql). Axum integration is a ~20-line hand-rolled handler (`juniper::http::GraphQLRequest` → `execute` → JSON), avoiding integration-crate version skew. GraphQL `Int` is i32: counts cross the API saturated to i32. Schema-level depth/complexity enforcement is not built into Juniper — resolver-level limit caps apply now; query-cost middleware is M5 hardening. |
| Metadata | turso (latest; feature-flag fallback: rusqlite) |
| GreptimeDB client | SQL over HTTP API (reqwest) — no native client dependency in V1 |
| CLI | clap 4 |
| Core | serde/serde_json, sha2, regex, anyhow/thiserror |
| Engine | **GreptimeDB latest stable** (1.0.2 at spec time; supervisor resolves latest stable at install, records the resolved version in config and the release manifest) |
| UI | latest `@tanstack/react-start`, latest shadcn CLI/components (Base UI variant), latest Recharts via shadcn charts |

## 2a. Performance principles (operator rule, 2026-06-12)

Ingest is the hot path: **decode once, never clone, move ownership forward.** OTLP requests are
decoded from the wire once; receivers spool by reference and *move* the decoded request into the
worker channel (no `.clone()` on the hot path). Backlogged perf work, in order: spool raw
protobuf bytes instead of re-serializing to NDJSON (debuggability trade — revisit at M5 with
measurements); intern repeated strings (`service`, names) behind `Arc<str>` in the normalized
rows; batch adapter inserts by size and time window. Every perf claim still goes through
measured gate rows — this section sets the design posture, not numbers.

**Progress visibility (operator rule, 2026-06-12).** The user never waits in silence: long
CLI steps narrate as they happen (download progress with MiB/percent/speed, engine start,
health, table bootstrap), and `parallax serve` ends with a human banner naming every surface —
UI URL, GraphQL, OTLP ports, storage mode, data dir. New long-running surfaces follow the same
rule.

## 2b. UI delivery (decided against the real build, 2026-06-12)

TanStack Start builds in **SPA mode** (`tanstackStart({ spa: { enabled: true } })`) producing
`ui/dist/client/` with `_shell.html` + assets; route loaders run client-side against the
same-origin `/graphql` (the dev server proxies it to :4000, so dev and prod behave alike). The
server mounts the dist directory as the API listener's fallback service (`ServeDir` with the
shell as fallback) — autodetected at `ui/dist/client` for dev checkouts, overridable via
`[server].ui_dist`, API-only with a hint when absent. **Release packaging embeds the same dist
into the binary behind an `embed-ui` cargo feature (rust-embed) at M-packaging** — disk serving
is the dev/default path, embedding is the distribution path.

## 3. Ports and process layout (collision fix)

GreptimeDB standalone defaults to :4000–:4003, colliding with the planned Parallax API port.
**Decision:** Parallax keeps **:4000** (API + UI + OTLP/HTTP on one axum listener; OTLP/gRPC on
:4317 via tonic; :4318 redirects to :4000's OTLP routes or binds separately — implementer's
choice, document in `doctor`); the **managed GreptimeDB child runs on shifted ports
24000–24003**, written into the child's config by the supervisor (Parallax owns the child's
config file entirely; `~/.parallax/greptime/config.toml`). `--greptime-url` mode uses whatever
the user provides. `parallax doctor` checks all five ports.

## 4. `~/.parallax/config.toml` (all keys, with defaults)

```toml
[server]
bind = "127.0.0.1"          # --bind to widen
api_port = 4000              # GraphQL + UI + OTLP/HTTP
otlp_grpc_port = 4317
otlp_http_port = 4318

[storage]
mode = "managed"             # managed | external | none
greptime_url = ""            # used when mode = "external"
greptime_version = "latest"  # resolves to latest stable at install; resolved version recorded here
data_dir = "~/.parallax"

[retention]
traces_ttl = "7d"
logs_ttl = "7d"
metrics_ttl = "14d"
error_events_ttl = "30d"

[limits]
bundle_max_tokens = 10000
graphql_max_depth = 8
graphql_max_complexity = 1000
```

## 5. GreptimeDB DDL (created by the storage adapter on first start)

Conventions: time index on the event timestamp; `service` as a tag (PRIMARY KEY) column for
locality; high-cardinality identifiers (`trace_id`) as fields with an inverted index where
available; attribute maps as `JSON` columns with hot keys promoted to real columns; TTL from
config interpolated into `WITH (ttl = …)`.

**Learned against the real engine (2026-06-12, v1.0.2):** every identifier is double-quoted —
`service`, `name`, `value`, `count`, `sum`, `source` are reserved words in GreptimeDB's parser;
JSON values insert via `parse_json('…')` and read back via `json_to_string(…)`; `CAST("ts" AS
BIGINT)` in projections must be aliased (DataFusion unique-name rule); the HTTP SQL API returns
`{"output":[…]}` on success (no `code` field) and `{"code":…,"error":…}` on failure; the
`.sha256sum` release asset is a bare hash. The DDL below is normative as written; the adapter
applies the quoting.

```sql
CREATE TABLE IF NOT EXISTS otel_spans (
  ts                TIMESTAMP(9) NOT NULL,
  service           STRING,
  trace_id          STRING,
  span_id           STRING,
  parent_span_id    STRING,
  name              STRING,
  kind              STRING,
  status_code       STRING,
  status_message    STRING,
  duration_ns       BIGINT,
  run_id            STRING,
  scope_name        STRING,
  attributes        JSON,
  resource          JSON,
  TIME INDEX (ts),
  PRIMARY KEY (service)
) WITH (ttl = '{traces_ttl}');

CREATE TABLE IF NOT EXISTS otel_logs (
  ts             TIMESTAMP(9) NOT NULL,
  service        STRING,
  severity_num   INT,
  severity_text  STRING,
  body           STRING,
  trace_id       STRING,
  span_id        STRING,
  run_id         STRING,
  scope_name     STRING,
  attributes     JSON,
  resource       JSON,
  TIME INDEX (ts),
  PRIMARY KEY (service)
) WITH (ttl = '{logs_ttl}');

-- One table per point class keeps queries simple in V1.
CREATE TABLE IF NOT EXISTS otel_metrics_points (   -- gauges + sums
  ts          TIMESTAMP(3) NOT NULL,
  service     STRING,
  name        STRING,
  value       DOUBLE,
  is_monotonic BOOLEAN,
  attributes  JSON,
  TIME INDEX (ts),
  PRIMARY KEY (service, name)
) WITH (ttl = '{metrics_ttl}');

CREATE TABLE IF NOT EXISTS otel_metrics_histograms (
  ts            TIMESTAMP(3) NOT NULL,
  service       STRING,
  name          STRING,
  count         BIGINT,
  sum           DOUBLE,
  bucket_counts JSON,
  bounds        JSON,
  attributes    JSON,
  TIME INDEX (ts),
  PRIMARY KEY (service, name)
) WITH (ttl = '{metrics_ttl}');

CREATE TABLE IF NOT EXISTS error_events (
  ts           TIMESTAMP(9) NOT NULL,
  service      STRING,
  fingerprint  STRING,
  error_type   STRING,
  message      STRING,
  stacktrace   STRING,
  source       STRING,           -- span_exception | span_status | log_record | log_exception
  trace_id     STRING,
  span_id      STRING,
  attributes   JSON,
  TIME INDEX (ts),
  PRIMARY KEY (service, fingerprint)
) WITH (ttl = '{error_events_ttl}');

CREATE TABLE IF NOT EXISTS rollups_fingerprint_minute (
  bucket_ts    TIMESTAMP(0) NOT NULL,
  service      STRING,
  fingerprint  STRING,
  count        BIGINT,
  TIME INDEX (bucket_ts),
  PRIMARY KEY (service, fingerprint)
) WITH (ttl = '{error_events_ttl}');
```

Adapter queries are plain SQL over the HTTP API; every engine-specific statement lives in
`parallax-storage`'s greptime module only.

**Why not GreptimeDB's native OTLP tables** (`opentelemetry_traces`, pipeline-fed
`opentelemetry_logs`, one-table-per-metric): GreptimeDB can ingest OTLP directly at
`/v1/otlp/...` and auto-create its own layouts, but Parallax deliberately does not use that
path. Parallax **is** the OTLP receiver (the proxy-lens architecture decision, 2026-05-25):
ingest flows through Parallax's derivation/grouping/run-scoping workers, and the adapter writes
the tables above. What this buys: an **engine-portable schema** (the ClickHouse fallback gets
the identical layout through its own adapter), the `run_id` column and other Parallax semantics
as first-class columns, and one stable contract behind `StorageAdapter`. What it costs: we
forgo Greptime's built-in trace view/Jaeger-compatible query layer and its
PromQL-ergonomic per-metric tables — V1 charts query our single metrics tables with SQL
aggregates instead. Revisit-trigger: if V2 wants PromQL compatibility, a parallel write into
Greptime's native metric model (or its OTLP metrics endpoint) can be added inside the greptime
adapter without touching the product contract.

## 6. Turso (metadata) DDL

```sql
CREATE TABLE IF NOT EXISTS issues (
  fingerprint   TEXT PRIMARY KEY,
  title         TEXT NOT NULL,          -- error_type: normalized message head
  error_type    TEXT NOT NULL,
  culprit       TEXT,                   -- top stack frame
  service       TEXT NOT NULL,
  status        TEXT NOT NULL DEFAULT 'open',   -- open | resolved
  first_seen    INTEGER NOT NULL,       -- unix nanos
  last_seen     INTEGER NOT NULL,
  event_count   INTEGER NOT NULL DEFAULT 0,
  last_trace_id TEXT,
  tags          TEXT NOT NULL DEFAULT '{}'      -- JSON: top tag values cache
);
CREATE TABLE IF NOT EXISTS runs (
  run_id      TEXT PRIMARY KEY,
  command     TEXT,
  started_at  INTEGER NOT NULL,
  ended_at    INTEGER,
  exit_code   INTEGER,
  status      TEXT NOT NULL DEFAULT 'running'   -- running | finished | external
);
CREATE TABLE IF NOT EXISTS dashboards (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  layout      TEXT NOT NULL,    -- JSON: [{metric, agg, chart, title, groupBy?, quantile?, w?}]
  created_at  INTEGER NOT NULL,
  updated_at  INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS settings ( key TEXT PRIMARY KEY, value TEXT NOT NULL );
CREATE TABLE IF NOT EXISTS issue_buckets (
  fingerprint TEXT NOT NULL,
  bucket_ts   INTEGER NOT NULL,   -- minute-aligned unix millis
  count       INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (fingerprint, bucket_ts)
);
```

Counters (`event_count`, `last_seen`) are updated by the ingest worker on each derived error
event; the same upsert increments the minute-grained `issue_buckets` rollup that feeds the
trend sparkline (`issueTrend` sums it into coarser steps in SQL) and merges the event's scalar
attributes into the bounded `tags` cache (`{key: {value: count}}`; ≤16 keys, ≤8 values per key,
values ≤64 chars, `exception.*` excluded). Runs whose `parallax.run_id` first appears in
telemetry without a CLI `runStart` are auto-registered by the worker with status `external`
(first-seen timestamp as `started_at`) so run-scoped UI/CLI lookups work for foreign run ids
(the jackin follow-up, 2026-06-12).

## 7. OTLP → storage mapping (the load-bearing rows)

| OTLP (proto) | Column |
| --- | --- |
| `resource.attributes["service.name"]` | `service` (every table) |
| full resource attribute list | `resource` JSON |
| span `trace_id`/`span_id`/`parent_span_id` (bytes) | lowercase hex strings |
| span `start_time_unix_nano` | `ts`; `end-start` → `duration_ns` |
| span `status.code` | `STATUS_CODE_*` string |
| span events named `exception` | error_events row (source `span_exception`) |
| span status ERROR w/o exception event | error_events row (source `span_status`) |
| log `severity_number >= 17` or `exception.*` attrs | error_events row (`log_record`/`log_exception`) |
| log `body.string_value` | `body` |
| metric gauge/sum data points | `otel_metrics_points` (one row per point; `is_monotonic` from sum) |
| metric histogram data points | `otel_metrics_histograms` |
| `resource.attributes["parallax.run_id"]` | **promoted to a real `run_id` column** on `otel_spans`/`otel_logs` (the key contains a dot, making JSON-path filtering fragile; a column makes run-scoped reads exact and fast) |

Fingerprinting and derivation logic: graduate `poc/evidence-loop/src/{derive,fingerprint}.rs`
verbatim semantics (both exception encodings; normalization rules; 16-hex fingerprint).

## 8. GraphQL SDL (the V1 core, as implemented by Juniper)

Dialect conventions (decided against the real build, 2026-06-12 — Juniper, not async-graphql):
**nanosecond timestamps cross the API as strings** (`tsNanos`/`fromNanos`/`toNanos` — GraphQL
`Int` is i32 and `Float` loses precision); **JSON crosses as a JSON-encoded `String!`**
(`attributes`, `resource`, `tags`, `layout`); **filters are flat arguments**, not input
objects (Juniper input objects buy nothing over named args for this surface); counts saturate
to i32. Where the original draft said `Time`/`JSON` scalars and `TimeRange`/`*Filter` inputs,
this implemented dialect is the contract.

```graphql
type Query {
  health: String!
  version: String!
  runs(limit: Int = 50): [Run!]!
  run(runId: String!): Run
  issues(service: String, status: String, query: String,
         fromNanos: String, toNanos: String,          # window on lastSeen
         tagKey: String, tagValue: String,
         sort: IssueSort = LAST_SEEN, limit: Int = 50, offset: Int = 0): IssueList!
  issue(fingerprint: String!): Issue
  issueTrend(fingerprint: String!, hours: Int = 24, stepSeconds: Int = 3600): [TrendPoint!]!
  trace(traceId: String!): Trace
  tracesByRun(runId: String!, limit: Int = 200): [TraceSummary!]!
  logs(traceId: String, runId: String, service: String,
       fromNanos: String, toNanos: String, severityMin: Int,
       query: String, limit: Int = 500): [LogRecord!]!   # unified browse; newest first
  logsByTrace(traceId: String!): [LogRecord!]!           # anchored reads, time ascending
  logsByRun(runId: String!, limit: Int = 500): [LogRecord!]!
  metricNames(prefix: String): [String!]!
  services: [String!]!
  metricSeries(name: String!, fromNanos: String!, toNanos: String!, service: String,
               groupBy: String, stepSeconds: Int = 60, agg: String = "avg"): [Series!]!
  histogramQuantile(name: String!, fromNanos: String!, toNanos: String!, q: Float!,
                    service: String, stepSeconds: Int = 60): [Point!]!
  serviceOverview(service: String!, fromNanos: String!, toNanos: String!,
                  stepSeconds: Int = 60): ServiceOverview!
  bundle(fingerprint: String, runId: String, traceId: String,
         maxTokens: Int = 10000): BundleOut               # exactly one anchor
  dashboards: [Dashboard!]!
  dashboard(id: String!): Dashboard
  sql(query: String!): SqlResult!    # raw read-only engine SQL (see note)
}
type Mutation {
  issueSetStatus(fingerprint: String!, status: String!): Issue!   # open | resolved
  dashboardSave(name: String!, layout: String!, id: String): Dashboard!
  dashboardDelete(id: String!): Boolean!
  runStart(runId: String!, command: String, startedAtNanos: String!): Boolean!
  runFinish(runId: String!, endedAtNanos: String!, exitCode: Int!): Boolean!
}

enum IssueSort { LAST_SEEN FIRST_SEEN EVENTS TREND }   # TREND = last-24h occurrence sum

type IssueList { items: [Issue!]!, total: Int! }   # total capped at the 1000-row scan window
type Issue {
  fingerprint: String!, title: String!, errorType: String!, culprit: String,
  service: String!, status: String!, firstSeenNanos: String!, lastSeenNanos: String!,
  eventCount: Int!, lastTraceId: String, tags: String!,            # JSON: {key: {value: count}}
  trend: [TrendPoint!]!,                                           # last 24h, hourly
  latestEvent: ErrorEvent,
  events(limit: Int = 50, fromNanos: String, toNanos: String): [ErrorEvent!]!
}
type ErrorEvent { tsNanos: String!, service: String!, fingerprint: String!, errorType: String!,
  message: String!, stacktrace: String, source: String!, traceId: String!, spanId: String!,
  attributes: String! }
type Trace { traceId: String!, spans: [Span!]! }
type Span { tsNanos: String!, service: String!, traceId: String!, spanId: String!,
  parentSpanId: String, name: String!, kind: String!, statusCode: String!,
  statusMessage: String!, durationNs: String!, runId: String, scopeName: String!,
  attributes: String!, resource: String! }
type TraceSummary { traceId: String!, rootName: String, service: String, tsNanos: String!,
  durationNs: String!, spanCount: Int!, errorCount: Int! }   # errorCount = ERROR-status spans
type LogRecord { tsNanos: String!, service: String!, severityNum: Int!, severityText: String!,
  body: String!, traceId: String!, spanId: String!, runId: String, attributes: String! }
type Series { groupValue: String, points: [Point!]! }   # groupValue null when ungrouped
type Point { tsNanos: String!, value: Float! }
type TrendPoint { tsNanos: String!, count: Int! }
type Run { runId: String!, command: String, startedAtNanos: String!, endedAtNanos: String,
  exitCode: Int, status: String!,                  # running | finished | external
  errorCount: Int!, traceCount: Int!, issues: [Issue!]! }
type Dashboard { id: String!, name: String!, layout: String!, updatedAtNanos: String! }
type ServiceOverview { cpu: [Point!]!, memory: [Point!]!,
  requestRate: [Point!]!, latencyP50: [Point!]!, latencyP95: [Point!]!, latencyP99: [Point!]!,
  errorRate: [Point!]! }
type BundleOut { json: String!, markdown: String!, canonicalHash: String! }
type SqlResult { columns: [String!]!, rows: [String!]!, rowCount: Int! }  # rows are JSON arrays
```

`sql` exposes the telemetry engine's full read query power (the logs page's
escape hatch and the agent's power tool): one statement, gated to read-only
prefixes (SELECT/WITH/SHOW/DESCRIBE/EXPLAIN/TQL). It is engine-dialect SQL —
not part of the portable contract — and exists because the V1 profile is
loopback, single-user, no-auth; the V2 server profile must revisit it behind
authz before any non-local exposure.

Pagination/row caps are resolver-level (500 rows; issue scans capped at 1000) — Juniper has no
schema-level depth/complexity middleware; the `[limits]` config keys wait on the M5 query-cost
middleware. `serviceOverview` resolves from well-known metric names (`process.cpu.*`,
`process.memory.*`, `http.server.request.duration`, `rpc.server.duration` — first candidate
with data wins) with graceful absence (empty series + the gap surfaced — feeds instrumentation
suggestions). `bundle` accepts exactly one anchor: `fingerprint` (issue), `runId`
(run-anchored: the run's traces, logs, and grouped issues), or `traceId`.

## 9. UI page → query map

| Page | Queries |
| --- | --- |
| Issues list | `issues(service, status, query, sort, …)` (+ per-row `trend` already embedded) |
| Issue detail | `issue` (tags/latestEvent/events), `issueTrend`, `logsByTrace` breadcrumbs, `bundle(fingerprint:)` for the CLI snippet, `issueSetStatus` |
| Service overview | `serviceOverview` (+ `services` for the selector) |
| Custom dashboard | `dashboards`/`dashboard` + N × `metricSeries(groupBy?)`/`histogramQuantile`; builder uses `metricNames`; `dashboardSave`/`dashboardDelete` |
| Trace view | `trace(traceId)` + `logsByTrace`; entry from paste, issue event, or a run's `tracesByRun(runId)` |
| Logs | `logs(traceId?, runId?, service?, severityMin?, query?, …)` |
| Runs | `runs` / `run(runId)` (errorCount/traceCount/issues) + `tracesByRun` + `logsByRun` + `bundle(runId:)` preview |

## 10. CLI output contract

Every read command supports `--format table|json|md` (default `table` on TTY, `json` when
piped). `issue context` defaults to `md` (agent-facing). Exit codes: 0 ok, 1 error, 2 not-found.
`run start -- <cmd>` propagates the child's exit code.

## 11. GreptimeDB supervision contract

1. Resolve binary: `storage.mode=managed` → look in `<data_dir>/bin/greptime`, then `$PATH`;
   if absent, download the release for the host triple from GitHub releases (resolving
   `latest` via the API, **falling back to the pinned floor version when the API is
   unreachable**), verify the bare-hash `.sha256sum`, install to `<data_dir>/bin/`.
2. Write child config (ports 24000–24003, data dir, `--rpc-bind-addr 127.0.0.1`).
3. Spawn `greptime standalone start -c …`; health = HTTP `/health` on 24000 with timeout;
   restart with backoff on crash; stop on `parallax serve` shutdown.
4. **Orphan safety** (verified 2026-06-12 — a SIGKILLed serve leaves the child alive on the
   engine ports, and the next serve would otherwise health-check that foreign-data-dir orphan
   while its own child crash-loops): the supervisor writes `<data_dir>/greptime.pid` on every
   (re)spawn; on start it reaps a still-alive pidfile process (only if `ps` confirms it is a
   greptime binary), then preflight-binds port 24000 and refuses to start if a foreign
   listener holds it. `parallax serve` handles SIGTERM as cleanly as Ctrl-C; the pidfile is
   removed on clean shutdown.
5. `doctor` reports: binary path + version + checksum status, child pid/health, port checks,
   data-dir size per table, spool backlog.

## 12. What stays out of this spec on purpose

Internal module layout inside crates, exact axum route tree, UI component file structure
(follow shadcn blocks), worker channel sizes, GreptimeDB index tuning (V1 ships defaults;
benchmarks own tuning claims). The implementing agent decides these inside the conventions
above. Anything that would change a *contract* in this file gets changed **here first**, then
in code.

## 13. Readiness statement

With this spec, [v1-scope.md](v1-scope.md) (inventory + acceptance) and
[simple-ui-v2.md](simple-ui-v2.md) (UI) are implementable end-to-end: schemas, mappings, ports,
pins, API shape, and supervision are decided; the PoC supplies derivation/fingerprint/bundle
semantics; acceptance is the dogfood scenarios in v1-scope §1. The recommended long-running
instruction for an implementing agent is recorded in [prompts/README.md](../../../prompts/README.md)
alongside the other runbooks.
