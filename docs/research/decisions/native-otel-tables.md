# Native GreptimeDB OTLP Tables for Observability Signals

<!-- markdownlint-disable MD013 -->

Decision date: 2026-06-18 · Supersedes the "Why not GreptimeDB's native OTLP tables" / hand-rolled
`otel_*` stance in [v1-implementation-spec.md §5](../architecture/v1-implementation-spec.md). The
full implementation roadmap and the per-question decision log live in
[../storage/native-otel-migration-plan.md](../storage/native-otel-migration-plan.md); the open
vendor questions in [../storage/greptimedb-team-questions.md](../storage/greptimedb-team-questions.md).

> **Decision — V1 is GreptimeDB-only and adopts GreptimeDB's native OTLP model.** Parallax forwards
> raw OTLP straight to GreptimeDB's native tables (`opentelemetry_traces`, `opentelemetry_logs`, the
> per-metric metric engine) and **tees** the same bytes in-process to derive its product signals
> (error grouping / "issues") into a few **custom extension tables** (`error_events`,
> `run_metric_points`, `metric_exemplars`). **Hard rule:** raw observability signals must stay
> in GreptimeDB-native tables. Parallax may extend native tables and may keep derived product extension
> tables, but must not replace native logs/traces/metrics with hand-rolled raw-signal tables. **ClickHouse
> is deferred** — no longer a V1 fallback or a design constraint (revisit only if a concrete benefit
> appears). No data migration (greenfield; research stage).

## Hard rule and escalation path

This decision is binding for agents and future implementation plans:

- **Always use GreptimeDB native signal tables.** Logs use `opentelemetry_logs`; traces use
  `opentelemetry_traces` plus GreptimeDB's native trace helper tables; metrics use the native
  per-metric metric-engine tables. The same rule applies to future GreptimeDB-native OTLP signals.
- **Extend native before inventing tables.** Prefer native extension points: OTLP pipeline headers,
  `X-Greptime-Log-Extract-Keys`, native schema auto-widening, `ALTER TABLE` columns/indexes, SQL/PromQL
  functions, Flows, and upstream GreptimeDB changes.
- **Custom raw-signal tables are a stop condition.** If a plan appears to require a custom table that
  stores raw logs, traces, metrics, profiles, or equivalent observability records, stop and produce a
  research packet first. The packet must cover latest stable, latest nightly/source, official docs,
  live spike results where feasible, tradeoffs lost by leaving native tables, and candidate upstream
  changes.
- **Consult GreptimeDB before breaking native.** Ning Sun ([Greptime cofounder & CTO](https://greptime.com/about))
  has directly recommended that Parallax design around native tables because that is where GreptimeDB
  focuses performance and compatibility work. Before creating a GreptimeDB pull request or adopting a
  custom raw-signal table, consult Ning / the GreptimeDB team with the research packet so the fix
  aligns with their roadmap instead of wasting their review time.
- **Derived extension tables remain allowed.** Tables like `error_events`, `run_metric_points`, and
  `metric_exemplars` are Parallax product facts, not raw OTel replacements. They stay allowed only when
  they are derived from native signal data or in-process tees and are documented here.

## Why

1. **The GreptimeDB team optimizes around the native model** and recommends it (Ning Sun / GreptimeDB
   team guidance, 2026-06-18 and reaffirmed 2026-07-09); ecosystem products build on it. Forwarding
   raw OTLP into the native tables means Parallax inherits that optimization roadmap instead of
   competing with hand-rolled tables.
2. **The engine sub-study already verified the native trace model live and rated it better** than the
   hand-rolled `otel_spans` — bloom-indexed `trace_id` + 16-way `trace_id` partitioning
   ([greptimedb-implementation.md](../storage/greptimedb-vs-clickhouse/greptimedb-implementation.md),
   pass 119 / Run 86).
3. **GreptimeDB-only focus (no ClickHouse boundary) removes the portability constraint**, so the
   design is free to use Greptime-native features (Flow, `digest`, HLL, uddsketch). See the V1-scope
   update in [storage-engine.md](storage-engine.md) and [v1-storage-adapter-vision.md](v1-storage-adapter-vision.md).

## The native OTLP model (verified — official docs + live engine)

GreptimeDB auto-creates and maintains these when OTLP flows into its `/v1/otlp/v1/...` endpoints.
Source: GreptimeDB docs
([OTLP](https://docs.greptime.com/user-guide/ingest-data/for-observability/opentelemetry/),
[trace data model](https://docs.greptime.com/user-guide/traces/data-model/)) + Run 45/86.

| Signal | Endpoint + header | Native table | Shape |
| --- | --- | --- | --- |
| **Traces** | `POST /v1/otlp/v1/traces`, `x-greptime-pipeline-name: greptime_trace_v1` | `opentelemetry_traces` (+ `_services`, `_operations`) | 1 row/span. `service_name` = Tag + PK; `timestamp` = Time Index; `duration_nano`/`timestamp_end` generated. **Every attribute → its own column** (`span_attributes.<k>`, `resource_attributes.<k>`, `scope_attributes.<k>`; except `resource_attributes.service.name` → `service_name`). `span_events`/`span_links`/compound → `JSON`. `trace_id`/`parent_span_id`/`service_name` BLOOM `SKIPPING INDEX`; `PARTITION ON COLUMNS (trace_id)` 16-way. Schema **auto-widens**. |
| **Logs** | `POST /v1/otlp/v1/logs` (`x-greptime-log-table-name`, default `opentelemetry_logs`) | `opentelemetry_logs` | `timestamp`, `trace_id`, `span_id`, `severity_text`, `body`, attributes as `JSON`. `append_mode='true'`. **No PK, no `trace_id` index** (flat append). |
| **Metrics** | `POST /v1/otlp/v1/metrics` | **one table per metric name** (metric engine) | metric name = table name; selected resource attrs = tag columns; PromQL-native. **ExponentialHistogram unsupported.** |

Customization levers: `x-greptime-hints: ttl=…, append_mode=…`; table-name + `X-Greptime-Log-Extract-Keys`
headers; post-create `ALTER TABLE … ADD COLUMN` / `ADD … INVERTED INDEX | FULLTEXT INDEX | SKIPPING INDEX`.

## Per-signal decisions (adopt-then-customize)

- **Traces → ADOPT native `opentelemetry_traces`.** Strictly better than hand-rolled. The canonical
  fingerprint-to-trace/span relation is the derived `error_events` record; do
  not add a native trace `fingerprint` column. Any legacy nullable column is
  inert and unsupported until a separately live-proven, data-safe migration.
- **Logs → ADOPT native `opentelemetry_logs`**, with Plan 084 corrections (pre-create + extract-keys
  + SKIPPING on `trace_id`; body FULLTEXT is native-default on engine ≥1.1).
- **Metrics → ADOPT the native metric engine fully (PromQL-native).** Rely on explicit-bucket
  histograms; add a minimal extension only if ExponentialHistogram appears.
- **`cli.invocation.id` / `session.id`** (plan 156 neutral contract; replaces the retired
  `parallax.run.id` resource attribute). Lookup priority on ingest: **signal attributes first**
  (root-span / log attrs — jackin shape), then **resource attributes** (generic wrappers). The
  legacy `parallax.run.id` key is never read, written, or COALESCE'd.
  - Traces: span-attr column and/or free JSON `resource_attributes."cli.invocation.id"` (and
    `session.id` likewise); query helpers in `greptime_sql` / `trace_store` /
    `invocation_store`.
  - Logs: promote via `X-Greptime-Log-Extract-Keys:
    service.name,cli.invocation.id,session.id,event.name,observed_ts_nanos` with pre-created
    SKIPPING INDEX columns on `opentelemetry_logs`.
  - Metrics: **never metric tags** (high-cardinality → series explosion).
- **Invocation-scoped metrics → custom extension `invocation_metric_points`** (append table,
  `invocation_id STRING SKIPPING INDEX`, `append_mode`, `flat` SST) — GreptimeDB's own
  high-cardinality pattern; the metric engine stays invocation_id-free. Fresh installs create
  only the new table; bootstrap migrates legacy `run_metric_points` rows when present, then
  drops that table.
- **`error_events`, `invocation_metric_points`, `metric_exemplars` → KEEP custom.** Product
  semantics; no native raw-signal replacement. `metric_exemplars` is keyed only by the
  low-cardinality `(service, name)` pair; trace/invocation correlation identifiers are fields
  with skipping indexes only where reads justify them. Existing legacy exemplar tables migrate
  through a verified replacement-and-rename sequence, preserving the source until row and value
  parity succeeds.

## Plan 084 — deterministic native logs schema (verified 2026-07-11, GreptimeDB v1.1.2)

Live engine verification:

| Fact | Evidence |
| --- | --- |
| Version | `SELECT version()` → `1.1.2` |
| Body FULLTEXT native | Auto-create includes bloom FULLTEXT on `body` |
| extract-keys race | Keys become PK TAGs if columns do not pre-exist (incl. high-card `observed_ts_nanos`) |
| Pre-create fix | Pre-create native-shaped schema with FIELD deviations; extract-keys reuses types |
| Service column | extract-keys column is **`service.name`** (not traces `service_name`); TAG + COALESCE reads |
| `trace_id` index | SKIPPING (bloom), not INVERTED |
| Body search | `matches_term` term match (not substring); memory adapter stays substring |
| TTL reconcile | `ALTER TABLE … SET 'ttl'` on bootstrap for listed tables; per-metric natives excluded |
| Query timeout | `X-Greptime-Timeout: 60s` on SQL; reqwest client 70s |

Native extension (pre-create + extract-keys + ALTER), not a custom raw-signal table.

## Write path — Path A (decided)

The greptime adapter **re-emits raw OTLP to GreptimeDB's `/v1/otlp/` endpoints** (Path A), so native
tables auto-create, attributes flatten, the schema auto-widens, and Greptime's optimizations land on
Parallax's data for free. The rejected Path B (hand-write native-shaped rows via SQL) could not
reproduce dynamic attribute flattening and would forfeit those optimizations. Parallax stays the OTLP
receiver: it **tees** in-process to derive `error_events`/issues (no read-back), and — per the
forward-as-is decision — **does not redact on the forward path** (raw telemetry is stored unredacted
at rest; acceptable for the self-hosted/local V1, revisited only for a managed/cloud profile).

## Grouping division of labor

GreptimeDB can offload the *counting* (issue counts, trend rollups via Flow, unique-users via HLL,
percentiles via uddsketch over native tables). Parallax owns the *intelligence* (stacktrace
fingerprinting, custom grouping) and the *state* (issue identity + mutable lifecycle, in Turso) —
because a timeseries store cannot express those, not for portability. Treat Greptime Flows/sketches
as an acceleration layer Parallax owns and can recompute; the canonical fingerprint + issue state stay
authoritative in Parallax. Detail: [../storage/native-otel-migration-plan.md](../storage/native-otel-migration-plan.md).

## Status of earlier open items

- `run_id` → resource attribute, decided (Q6). Redaction-before-re-emit → **not done** by decision (Q1).
- Metric cutover → fully native, SQL-first reads, PromQL where it helps; exp-histogram fallback only if
  needed (Q3).
- Greenfield, so **no measure-before-delete / no migration** (Q4) — delete custom `otel_*` DDL outright.
- Vendor confirmations (custom columns/indexes vs auto-widening, traces GA, etc.) tracked in
  [../storage/greptimedb-team-questions.md](../storage/greptimedb-team-questions.md).
