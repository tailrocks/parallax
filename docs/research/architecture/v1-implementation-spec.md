# V1 Implementation Spec: The Concrete Contracts

<!-- markdownlint-disable MD013 -->

Research date: 2026-06-12. This is the layer between the concept docs and the first commit: the
concrete schemas, mappings, pins, and conventions an implementing agent needs so that
[v1-scope.md](v1-scope.md) + [simple-ui-v2.md](simple-ui-v2.md) are executable without
re-deriving decisions. Read order for an implementer: **v1-scope (what) → historical v1-build record (sequencing context)
→ this spec (contracts) → simple-ui-v2 (UI) → rust-stack-instrumentation (what arrives)**.
PoC kernels graduate per [poc-evidence-loop-coverage.md](poc-evidence-loop-coverage.md).

Operator note (2026-06-12): Juniper tracing is deferred — the operator instruments his own
resolvers; Parallax only consumes whatever spans arrive.

## 1. Workspace conventions

- Rust edition 2024; toolchain pinned via `rust-toolchain.toml` (current stable; 1.97 at spec
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
  **real SDK emission** (tracing + opentelemetry-otlp) against an in-process server with
  explicitly injected telemetry and metadata capabilities. The in-memory adapter is
  feature-gated test support, absent from product config and normal/release graphs; golden
  bundle tests reuse PoC fixtures.
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
| Metadata | turso (latest; **committed, no fallback engine** — operator decision 2026-06-12: GreptimeDB + Turso are the mandatory stack; no rusqlite flag, no engine swap; in-memory adapter is test-only and absent from normal/release feature graphs) |
| GreptimeDB client | SQL over HTTP API (reqwest) — no native client dependency in V1 |
| CLI | clap 4 |
| Core | serde/serde_json, sha2, regex, anyhow/thiserror |
| Engine | **GreptimeDB latest stable** (1.0.2 at spec time; supervisor resolves latest stable at install, records the resolved version in config and the release manifest) |
| UI | latest `@tanstack/react-start`, latest shadcn CLI/components (Base UI variant), latest Recharts via shadcn charts |

## 2a. Performance principles (operator rule, 2026-06-12)

Ingest is the hot path: **decode once, never clone, move ownership forward.** OTLP requests are
decoded from the wire once; receivers spool by reference and *move* the decoded request into the
worker channel (no `.clone()` on the hot path). The ingest spool is a bounded WAL: write-before-ack
is unchanged, active per-signal NDJSON files rotate at `[retention].spool_max_segment_bytes`, and
the reaper enforces `[retention].spool_max_total_bytes` plus `[retention].spool_max_age_hours`
without deleting active files. Rotated segments are reclaim-eligible because forwarding to the
engine happens synchronously from the ingest channel today; future replay work must narrow that to
segments newer than the last engine-ack watermark. Backlogged perf work, in order: spool raw
protobuf bytes instead of re-serializing to NDJSON (debuggability trade — revisit at M5 with
measurements); intern repeated strings (`service`, names) behind `Arc<str>` in the normalized rows;
batch adapter inserts by size and time window. Every perf claim still goes through measured gate
rows — this section sets the design posture, not numbers.

**Progress visibility (operator rule, 2026-06-12).** The user never waits in silence: long
CLI steps narrate as they happen (download progress with MiB/percent/speed, engine start,
health, table bootstrap), and `parallax serve` ends with a human banner naming every surface —
UI URL, GraphQL, OTLP ports, GreptimeDB endpoint/mode, Turso path, and data dir. New long-running surfaces follow the same
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
mode = "managed"             # managed | external; no product fallback mode
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

Product config and startup reject every storage mode except `managed` and
`external`; external mode requires `greptime_url`. Tests do not encode a hidden
mode. They call an internal composition seam with injected `TelemetryStore` and
`MetadataStore` capabilities. Normal and release builds do not compile the
in-memory adapter.

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
  links             JSON,     -- OTel span links: [{traceId, spanId, attributes}]
  attributes        JSON,
  resource          JSON,
  TIME INDEX (ts),
  PRIMARY KEY (service)
) WITH (ttl = '{traces_ttl}');
-- Pre-existing installs gain links via the same ALTER-at-bootstrap
-- migration mechanism as otel_metrics_points.run_id.

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
  run_id      STRING,        -- promoted parallax.run.id, like spans/logs
  attributes  JSON,
  TIME INDEX (ts),
  PRIMARY KEY (service, name)
) WITH (ttl = '{metrics_ttl}');
-- Pre-existing installs gain run_id via an ALTER TABLE migration at
-- bootstrap (the already-exists error is swallowed).

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

```

Adapter queries are plain SQL over the HTTP API; every engine-specific statement lives in
`parallax-storage`'s greptime module only.

**⚠ Native OTLP tables — DECIDED 2026-06-18; the hand-rolled DDL below is superseded for raw signals.**
V1 **adopts GreptimeDB's native OTLP model** (`opentelemetry_traces`, `opentelemetry_logs`,
one-table-per-metric metric engine) for the three raw signals. The adapter **forwards raw OTLP straight
to GreptimeDB's `/v1/otlp/` endpoints (Path A)** so native tables auto-create and ride Greptime's
optimizations, and **tees** the same bytes in-process to derive the **custom extension** tables
(`error_events`, `invocation_metric_points`, and `metric_exemplars`). Native
attributes are columns (traces) / JSON (logs); the correlation key is a resource attribute →
`resource_attributes."cli.invocation.id"` on traces (was `parallax.run.id` until 2026-07-17), promoted
via `X-Greptime-Log-Extract-Keys` on logs, and **never a metric tag** (high cardinality). `error_events`,
`invocation_metric_points`, and `metric_exemplars` stay custom because they are derived Parallax product
facts, not raw-signal replacement tables. **GreptimeDB + Turso only** — ClickHouse and Postgres are
comparators, not product targets. **Greenfield:** the `otel_spans`/`otel_logs`/`otel_metrics_*` DDL below is **removed**, not
migrated (research stage, no users). Canonical decision and historical adoption record:
[decisions/native-otel-tables.md](../decisions/native-otel-tables.md) ·
[storage/native-otel-migration-plan.md](../storage/native-otel-migration-plan.md). The DDL block below
is retained only as a record of the legacy shape and the extension-table definitions.

The `metric_exemplars` extension uses `TIMESTAMP(9)` as its time index and
`PRIMARY KEY (service, name)` only. `trace_id` and `run_id` are skipping-indexed
fields; `span_id` and JSON attributes remain fields. Bootstrap recognizes the
historical `(service, name, trace_id, span_id)` key, copies it into a versioned
replacement, verifies row count and bidirectional value equality, then performs
a restart-safe rename cutover. The retained source is never dropped until the
canonical replacement passes verification.

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
CREATE TABLE IF NOT EXISTS invocations (                       -- was `runs`; renamed 2026-07-17
  invocation_id TEXT PRIMARY KEY,
  command       TEXT,
  started_at    INTEGER NOT NULL,
  ended_at      INTEGER,
  exit_code     INTEGER,
  status        TEXT NOT NULL DEFAULT 'running',  -- running | finished | external
  app_mode      TEXT,
  outcome       TEXT
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
CREATE TABLE IF NOT EXISTS issue_occurrences (
  occurrence_id TEXT PRIMARY KEY,
  fingerprint   TEXT NOT NULL,
  observed_at   INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS issue_occurrences_observed_at
  ON issue_occurrences(observed_at);
```

Issue occurrence identity is deterministic and source-neutral: a failure with a
valid trace/span context uses `v1:span:{trace_id}:{span_id}:{fingerprint}` so a
late span/log echo claims the same identity; an uncorrelated log uses
`v1:event:{service}:{ts_nanos}:{fingerprint}` as its log-event identity. Turso
owns this mutable ledger because issue counters, buckets, status, and tags are
already one metadata consistency domain. GreptimeDB continues to store every
derived `error_events` row and owns no mutable dedup state.

Each metadata batch runs in one immediate transaction. It first inserts the
occurrence identity with `ON CONFLICT DO NOTHING`; only a newly claimed identity
updates `issues`, `issue_buckets`, and the tag cache. This makes retries,
concurrent delivery, and restart replays idempotent without collapsing distinct
span/log identities. The transaction prunes ledger rows older than 30 days from
the newest observed event time in the batch, bounding state consistently with
the default derived-error retention horizon.

Counters (`event_count`, `last_seen`) are updated by the ingest worker on each newly claimed derived error
event; the same transaction increments the minute-grained `issue_buckets` rollup that feeds the
trend sparkline (`issueTrend` sums it into coarser steps in SQL) and merges the event's scalar
attributes into the bounded `tags` cache (`{key: {value: count}}`; ≤16 keys, ≤8 values per key,
values ≤64 chars, `exception.*` excluded). Invocations whose `cli.invocation.id` first appears in
telemetry without a CLI `invocationStart` are auto-registered by the worker with status `external`
(first-seen timestamp as `started_at`) so invocation-scoped UI/CLI lookups work for foreign
invocation ids (the jackin follow-up; renamed from `parallax.run.id`/`runStart` on 2026-07-17).

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
| `resource/span.attributes["cli.invocation.id"]` (+ `session.id`) | **the invocation correlation key.** Resolved signal-attr first (root span / log attrs — jackin shape), then resource attr. On native traces it is a `span_attributes."cli.invocation.id"` column and free JSON `resource_attributes."cli.invocation.id"`; on native logs it is promoted via `X-Greptime-Log-Extract-Keys`; invocation-scoped metric points land in the `invocation_metric_points` extension (never a native metric tag — high cardinality). Supersedes the legacy `parallax.run.id`→`run_id` column (retired 2026-07-17, forward-only). Decision + sources: [capture/run-id-standardization.md](../capture/run-id-standardization.md) |

`TraceId` is a transparent model value at external boundaries: OTLP requires
exactly 16 non-zero bytes, GraphQL/CLI accept exactly 32 non-zero hexadecimal
characters, and text normalizes to lowercase. Persisted and wire values remain
the same lowercase strings; storage row fields are intentionally not swept in
the boundary pilot.

> **⚠ 2026-06-18 (native-OTLP decision):** the right-hand custom-table targets above (`otel_spans`,
> `otel_logs`, `otel_metrics_*`) are **superseded** — raw signals now land in GreptimeDB's native tables
> (`opentelemetry_traces`/`opentelemetry_logs`/metric engine) via OTLP forward; invocation-scoped
> metrics go to the `invocation_metric_points` extension (was `run_metric_points`), trace/span metric
> exemplars go to `metric_exemplars`, and derived error rows go to `error_events`. The correlation key
> is a resource attribute (column on traces, extract-key column on logs, never a metric tag). See
> [decisions/native-otel-tables.md](../decisions/native-otel-tables.md).
>
> **⚠ 2026-07-17 (unified-CLI observability, plans 156–161):** the correlation key is now
> `cli.invocation.id` (resolved signal-attr first, then resource-attr), not the retired
> `parallax.run.id`; the run-scoped extension table is `invocation_metric_points`. The legacy
> `run_metric_points` table is **dropped at bootstrap** (forward-only, no migration). See
> [capture/run-id-standardization.md](../capture/run-id-standardization.md) and
> [decisions/native-otel-tables.md](../decisions/native-otel-tables.md).

Fingerprinting and derivation logic: graduate `poc/evidence-loop/src/{derive,fingerprint}.rs`
semantics (both exception encodings; normalization rules; 16-hex fingerprint).
As of 2026-07-08, new fingerprints normalize top-frame line/column/path variance,
broaden volatile-token normalization for short hex IDs, jackin container slugs,
and uid/gid pairs, and prefer producer-stated `error.type` plus
`jackin.operation` when present. This is a forward-only cutover: existing issue
rows keep their old fingerprints and age out under normal retention instead of
being regrouped in place.

## 8. GraphQL SDL (the V1 core, as implemented by Juniper)

Dialect conventions (decided against the real build, 2026-06-12 — Juniper, not async-graphql):
**nanosecond timestamps cross the API as strings** (`tsNanos`/`fromNanos`/`toNanos` — GraphQL
`Int` is i32 and `Float` loses precision); **JSON crosses as a JSON-encoded `String!`**
(`attributes`, `resource`, `tags`, `layout`); **filters are flat arguments**, not input
objects (Juniper input objects buy nothing over named args for this surface); counts saturate
to i32. Where the original draft said `Time`/`JSON` scalars and `TimeRange`/`*Filter` inputs,
this implemented dialect is the contract.

```graphql
// The SDL below is the V1-launch core preserved as a contract record. The
// authoritative, always-current SDL is GENERATED from the Juniper schema by
// `cargo xtask ui graphql export` and checked into
// [`ui/graphql/schema.graphql`](../../../ui/graphql/schema.graphql) (drift-checked by
// `cargo xtask ui graphql check`). Treat that file as the source of truth; the
// sketch here is historical and intentionally partial.
//
// Re-verified 2026-07-17 against `crates/parallax-api/src/lib.rs` and
// `ui/graphql/schema.graphql`: the live schema is Juniper code-first with
// **76 Query fields, 14 Mutation fields, and ZERO Subscription fields**
// (`RootNode<Query, Mutation, EmptySubscription>`).
// Beyond the V1 core below it has grown: `overview`, `serviceMap`/`serviceRed`/
// `serviceCatalog`, `ecosystem` service topology, `invocations`/`invocation`/
`invocationFacets`/`observedInvocations`, `sessions`/`agentSession`/`story`/
`screenVisits`/`uiActions`/`backgroundCycles`/`jobs`/`conversations`/`evidenceGaps`,
`investigations`/`investigation`/`savedViews`, `alertRules`/`alertRule`/
`alertRuleStates`/`alertIncidents`/`alertIncident`/`alertDestinations`/`alertChecks`,
`testCases`/`testCase`, trace analytics (`traceEvents`/`linkedTraces`/
`traceCriticalPath`/`traceCompare`/`traceFacets`/`traceDurationStats`/`tracesPage`),
log analytics (`logsAround`/`logCountSeries`/`logFacets`/`logPatterns`), and a
metrics explorer (`metricCatalog`/`metricQuery`/`metricLabels`/`metricLabelValues`/
`metricExemplars`/`runtimeSnapshot`/`histogramQuantile`).
//
// **Run → invocation rename (2026-07-17, plans 156–161):** `runs`/`run`/
// `runStart`/`runFinish`/`tracesByRun`/`logsByRun` are gone; the live fields are
// `invocations`/`invocation`/`invocationStart`/`invocationFinish`/
// `tracesByInvocation`/`logsByInvocation`/`invocationMetrics`, keyed on
// `cli.invocation.id` (see [capture/run-id-standardization.md](../capture/run-id-standardization.md)).
// Live tail is SSE, not subscriptions (see the live-tail note below).
//
// Historical V1-launch core (field set as shipped at V1; names and shapes that
// have since been renamed are retained only to read older notes):
type Query {
  health: String!
  version: String!
  invocations(limit: Int = 50): [Invocation!]!        # was `runs`; renamed 2026-07-17
  invocation(invocationId: String!): Invocation        # was `run(runId:)`
  issues(...): IssueList!
  issue(fingerprint: String!): Issue
  issueTrend(fingerprint: String!, hours: Int = 24, stepSeconds: Int = 3600): [TrendPoint!]!
  trace(traceId: String!): Trace
  tracesByInvocation(invocationId: String!, limit: Int = 200): [TraceSummary!]!  # was tracesByRun
  traces(...): [TraceSummary!]!
  logs(...): [LogRecord!]!
  logsByTrace(traceId: String!): [LogRecord!]!
  logsByInvocation(invocationId: String!, limit: Int = 500): [LogRecord!]!        # was logsByRun
  metricNames(prefix: String): [String!]!
  services: [String!]!
  metricSeries(name:, ..., invocationId: String, ...): [Series!]!   # was runId
  histogramQuantile(...): [Point!]!
  serviceOverview(...): ServiceOverview!
  bundle(fingerprint: String, invocationId: String, traceId: String, maxTokens: Int = 10000): BundleOut
  dashboards: [Dashboard!]!
  dashboard(id: String!): Dashboard
  sql(query: String!): SqlResult!
}
type Mutation {
  issueSetStatus(fingerprint: String!, status: String!): Issue!
  dashboardSave(...): Dashboard!
  dashboardDelete(id: String!): Boolean!
  invocationStart(invocationId: String!, command: String, appMode: String, startedAtNanos: String!): Boolean!
  invocationFinish(invocationId: String!, endedAtNanos: String!, exitCode: Int!, outcome: String): Boolean!
}
// ... (Issue, ErrorEvent, Trace, Span, TraceSummary, LogRecord, Series, Point,
//      TrendPoint, Invocation [was Run], Dashboard, ServiceOverview, BundleOut,
//      SqlResult) — see the generated schema.graphql for current shapes.
```

`sql` exposes the telemetry engine's full read query power (the logs page's
escape hatch and the agent's power tool): one statement, gated to read-only
prefixes (SELECT/WITH/SHOW/DESCRIBE/EXPLAIN/TQL). It is engine-dialect SQL —
not part of the portable contract — and exists because the V1 profile is
loopback, single-user, no-auth; the V2 server profile must revisit it behind
authz before any non-local exposure.

**Live tail endpoints (SSE, not GraphQL).** `GET /v1/logs/stream` and
`GET /v1/traces/stream` on the API port serve Server-Sent Events fed by the
ingest worker's broadcast channels (published only while subscribers exist —
the hot path stays clone-free). Live is explicitly narrower than the polling
queries, by design and per industry practice (Datadog Live Tail, Loki `tail`):
**per-row predicates only, no time ranges, no aggregation, no SQL.** Filters
are query params mirroring the polling vocabulary where it applies to a single
row — logs: `service`, `severity_min`, `q`, `trace_id`, `invocationId`; traces (a
finished-span feed): `service`, `min_duration_ms`, `errors_only`, `q`,
`trace_id`, `invocationId`. Each SSE frame is a JSON array of matching rows; lagging
consumers drop batches (broadcast semantics = tail semantics). Rationale and
sources: [live-telemetry-streaming.md](live-telemetry-streaming.md). The CLI
mirror is `--follow` on `parallax logs`/`parallax traces`, with `--for <window>`
to watch a fixed window and report the match count (the agent verification
loop). `parallax invocation watch <invocation_id>` combines both streams
invocation-scoped (interleaved `[log]`/`[span]` lines), mirroring the invocation
hub's explicit Go-live mode (invocation-filtered SSE tails + 5 s metric/status
repolls). (The `run_id`/`run watch` spellings were retired in the 2026-07-17
rename.)

Pagination/row caps are resolver-level (500 rows; issue scans capped at 1000) — Juniper has no
schema-level depth/complexity middleware; the `[limits]` config keys wait on the M5 query-cost
middleware. `serviceOverview` resolves request/error/latency series from well-known request
metrics (`http.server.request.duration`, `rpc.server.duration` — first candidate with data wins)
with graceful absence (empty series + the gap surfaced — feeds instrumentation suggestions).
`runtimeSnapshot` is the runtime lane and discovers native metric tables in the supported
runtime families (`process.*`, `system.*`, `jvm.*`, `tokio.runtime.*`, `container.*`, and
`db.client.connection.*`). Rust playground runtime scenarios should prefer the emitted
`tokio.runtime.*` names; `process.*` remains a supported family for hosts/CLI/SDKs that emit it.
`bundle` accepts exactly one anchor: `fingerprint` (issue), `invocationId` (invocation-anchored:
the invocation's traces, logs, and grouped issues), or `traceId`.

**Bundle correlation sections (`metric_window`).** The bundle is the
correlation artifact — every anchor assembles **trace + logs + metric
windows** together (the scope §1 acceptance wording). `metric_windows[]`
entries carry `{metric, scope ("invocation"|"service"), window {from_nanos,
to_nanos, step_seconds}, points [{ts_nanos, value}], stats {min, max, avg,
last}}` for supported runtime metrics such as `process.cpu.utilization`,
`process.memory.usage`, and `tokio.runtime.alive_tasks`: invocation anchors use the
invocation's own invocation-scoped points over its lifespan (5 s steps); issue/trace
anchors use a ±5-minute window around the anchor event (30 s steps),
invocation-scoped when the anchor's spans carry an invocation id, service-scoped otherwise.
Windows are bounded (≤60 points per metric), participate in the canonical
hash like every section, and absent instruments contribute no entry —
graceful absence, surfaced through `missing_evidence`.

## 9. UI page → query map

| Page | Queries |
| --- | --- |
| Issues list | `issues(service, status, query, sort, …)` (+ per-row `trend` already embedded) |
| Issue detail | `issue` (tags/latestEvent/events), `issueTrend`, `logsByTrace` breadcrumbs, `bundle(fingerprint:)` for the CLI snippet, `issueSetStatus` |
| Service overview | `serviceOverview` (+ `services` for the selector) |
| Custom dashboard | `dashboards`/`dashboard` + N × `metricSeries(groupBy?)`/`histogramQuantile`; builder uses `metricNames`; `dashboardSave`/`dashboardDelete` |
| Trace view | `trace(traceId)` + `logsByTrace`; entry from paste, issue event, or a run's `tracesByRun(runId)` |
| Traces browse | `traces(service?, fromNanos?, toNanos?, minDurationMs?, errorOnly?, query?)`; Live mode switches to the `/v1/traces/stream` span feed |
| Logs | `logs(traceId?, runId?, service?, severityMin?, query?, …)` + `logCountSeries` histogram; Live mode switches to `/v1/logs/stream` |
| SQL workbench | `sql(query:)` on a dedicated page: schema browser (information_schema), cross-signal example joins, ⌘⏎, localStorage history — the escape hatch from predefined pages (logs/traces/metrics/error_events in one statement) |
| Runs | `runs` / `run(runId)` (errorCount/traceCount/issues) + `tracesByRun` + `logsByRun` + `bundle(runId:)` preview |

## 10. CLI output contract

Every read command supports `--format table|json|md` (default `table` on TTY, `json` when
piped). `issue context` defaults to `md` (agent-facing). Exit codes: 0 ok, 1 error, 2 not-found.
`invocation start -- <cmd>` propagates the child's exit code.

**`invocation start` OTLP forwarding (compare mode).** `parallax invocation
start` injects the full standard OTel env (`OTEL_EXPORTER_OTLP_*` for all
signals + protocols), `OTEL_RESOURCE_ATTRIBUTES=cli.invocation.id=<id>` (plus
`session.id`), `PARALLAX_INVOCATION_ID`, and a W3C `TRACEPARENT` into the child
(`crates/parallax-cli/src/commands/forwarding.rs::forward_resource_attrs`). The
carrier is the context of an exported agent-session span that remains open for
the wrapped command, so test runners and other context-aware children can join
one invocation trace. The API reports both active OTLP receiver ports; the
wrapper also injects `PARALLAX_OTLP_HTTP_TRACES_ENDPOINT` for
HTTP/protobuf-only clients. The destination is resolved: `--otlp-forward
<url|rotel|off>` flag > `PARALLAX_OTLP_FORWARD` env > a pre-existing
`OTEL_EXPORTER_OTLP_ENDPOINT` (respected, not clobbered) > the Parallax default
`http://127.0.0.1:4317`. When forwarding (compare mode) the injected resource
attrs also carry `parallax.lab=1` + `deployment.environment.name` (from
`PARALLAX_ENV`, default `lab`) for cross-tool alignment; the OTLP protocol
follows the endpoint port (`:4318`→http, else grpc). In Rotel compare mode the
HTTP traces endpoint uses Rotel's paired `:4318` receiver, preserving fan-out
for browser exporters while the standard child endpoint remains gRPC on `:4317`.
`--print-env` prints the env block and exits (dry-run). Config-file surface
(`[invocation].otlp_forward`) is deferred — v1 is env + flag only. This is the
lab hook in [`otlp-fanout-comparison-lab.md`](../validation/otlp-fanout-comparison-lab.md).
**(2026-07-17 rename)** this was `parallax run start` injecting
`parallax.run.id`/`PARALLAX_RUN_ID`; the unified-CLI observability program
(plans 156–161) renamed both the verb and the correlation key, with no legacy
fallback.

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
   data-dir size per table, active spool sizes, rotated segment counts, and configured spool caps.

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
execution contract for all unfinished implementation work is
[`plans/IMPLEMENTATION.md`](../../../plans/IMPLEMENTATION.md), with the active
index in [`plans/README.md`](../../../plans/README.md).
