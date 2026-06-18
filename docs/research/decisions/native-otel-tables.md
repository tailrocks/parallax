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
> `rollups_fingerprint_minute`, `run_metric_points`). **Native-first:** use native for the entire OTel
> stack; only where native genuinely cannot do something do we add a minimal extension — we **extend,
> never replace** native. **ClickHouse is deferred** — no longer a V1 fallback or a design constraint
> (revisit only if a concrete benefit appears). No data migration (greenfield; research stage).

## Why

1. **The GreptimeDB team optimizes around the native model** and recommends it (founder, Slack
   2026-06-18); ecosystem products build on it. Forwarding raw OTLP into the native tables means
   Parallax inherits that optimization roadmap instead of competing with hand-rolled tables.
2. **The engine sub-study already verified the native trace model live and rated it better** than the
   hand-rolled `otel_spans` — bloom-indexed `trace_id` + 16-way `trace_id` partitioning
   ([greptimedb-implementation.md](../storage/greptimedb-vs-clickhouse/greptimedb-implementation.md),
   pass 119 / Run 86).
3. **GreptimeDB-only focus (no ClickHouse boundary) removes the portability constraint**, so the
   design is free to use Greptime-native features (Flow, `digest`, HLL, uddsketch). See the V1-scope
   update in [storage-engine.md](storage-engine.md) and [v1-storage-adapter-vision.md](v1-storage-adapter-vision.md).

## The native OTLP model (verified — official docs + live engine)

GreptimeDB auto-creates and maintains these when OTLP flows into its `/v1/otlp/v1/...` endpoints.
Source: GreptimeDB docs (`for-observability/opentelemetry.md`, `traces/data-model.md`) + Run 45/86.

| Signal | Endpoint + header | Native table | Shape |
| --- | --- | --- | --- |
| **Traces** | `POST /v1/otlp/v1/traces`, `x-greptime-pipeline-name: greptime_trace_v1` | `opentelemetry_traces` (+ `_services`, `_operations`) | 1 row/span. `service_name` = Tag + PK; `timestamp` = Time Index; `duration_nano`/`timestamp_end` generated. **Every attribute → its own column** (`span_attributes.<k>`, `resource_attributes.<k>`, `scope_attributes.<k>`; except `resource_attributes.service.name` → `service_name`). `span_events`/`span_links`/compound → `JSON`. `trace_id`/`parent_span_id`/`service_name` BLOOM `SKIPPING INDEX`; `PARTITION ON COLUMNS (trace_id)` 16-way. Schema **auto-widens**. |
| **Logs** | `POST /v1/otlp/v1/logs` (`x-greptime-log-table-name`, default `opentelemetry_logs`) | `opentelemetry_logs` | `timestamp`, `trace_id`, `span_id`, `severity_text`, `body`, attributes as `JSON`. `append_mode='true'`. **No PK, no `trace_id` index** (flat append). |
| **Metrics** | `POST /v1/otlp/v1/metrics` | **one table per metric name** (metric engine) | metric name = table name; selected resource attrs = tag columns; PromQL-native. **ExponentialHistogram unsupported.** |

Customization levers: `x-greptime-hints: ttl=…, append_mode=…`; table-name + `X-Greptime-Log-Extract-Keys`
headers; post-create `ALTER TABLE … ADD COLUMN` / `ADD … INVERTED INDEX | FULLTEXT INDEX | SKIPPING INDEX`.

## Per-signal decisions (adopt-then-customize)

- **Traces → ADOPT native `opentelemetry_traces`.** Strictly better than hand-rolled. `ALTER`-add only
  cross-signal columns native lacks (e.g. `fingerprint`).
- **Logs → ADOPT native `opentelemetry_logs`**, then `ALTER`-add `trace_id INVERTED INDEX` + body
  `FULLTEXT` (native's one shortfall; `trace_id` retrieval is the dominant bundle cost, Run 56).
- **Metrics → ADOPT the native metric engine fully (PromQL-native).** Rely on explicit-bucket
  histograms; add a minimal extension only if ExponentialHistogram appears.
- **`run_id`** — emitted as a resource attribute. Traces: free column `resource_attributes.parallax.run.id`.
  Logs: promote to a column via `X-Greptime-Log-Extract-Keys: parallax.run.id`. Metrics: **never a
  metric tag** (high-cardinality → series explosion).
- **Run-scoped metrics → custom extension `run_metric_points`** (append table, `run_id STRING SKIPPING
  INDEX`, `append_mode`, `flat` SST) — GreptimeDB's own high-cardinality pattern; the metric engine
  stays run_id-free.
- **`error_events`, `rollups_fingerprint_minute` → KEEP custom.** Product semantics; no native form.

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
