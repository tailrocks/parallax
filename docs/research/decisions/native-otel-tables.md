# Native GreptimeDB OTLP Tables for Observability Signals

<!-- markdownlint-disable MD013 -->

Decision date: 2026-06-18 · Supersedes the "Why not GreptimeDB's native OTLP tables" stance in
[v1-implementation-spec.md §5](../architecture/v1-implementation-spec.md) for the **raw OTLP
signals** (traces, logs, metrics). Derived Parallax signals stay custom (see §"What stays custom").

> **Decision — adopt GreptimeDB's native OTLP table model for the three raw observability signals
> (traces, logs, metrics), then customize each by `ALTER` for the few columns/indexes Parallax
> needs. Keep the derived signals (`error_events`, `rollups_fingerprint_minute`) and all metadata
> (Turso) custom — GreptimeDB has no native equivalent for them.** Portability to the ClickHouse
> fallback is preserved at the `StorageAdapter` API boundary, *not* at the physical-table level.

This reverses the V1 spec's original "hand-rolled `otel_spans`/`otel_logs`/`otel_metrics_*`" choice
for raw signals. Two things changed since that choice: (1) GreptimeDB's founder confirmed directly
(Slack, 2026-06-18) that the team will **optimize specifically around the native data model** and
recommends it; (2) the engine sub-study already verified the native trace model live and rated it
**better** than the hand-rolled one ([greptimedb-implementation.md](../storage/greptimedb-vs-clickhouse/greptimedb-implementation.md),
pass 119 / Run 86). Riding the native model means Parallax inherits Greptime's optimization roadmap
instead of competing with it.

## The native OTLP model (verified against official docs + live engine)

GreptimeDB auto-creates and maintains these when OTLP flows into its `/v1/otlp/v1/...` endpoints
with the pipeline header. Source: GreptimeDB docs (`user-guide/ingest-data/for-observability/opentelemetry.md`,
`user-guide/traces/data-model.md`) + Run 86/Run 45 live verification.

| Signal | Endpoint + header | Native table | Shape |
| --- | --- | --- | --- |
| **Traces** | `POST /v1/otlp/v1/traces`, header `x-greptime-pipeline-name: greptime_trace_v1` | `opentelemetry_traces` (+ `_services`, `_operations` for the Jaeger API) | 1 row/span. `service_name` = Tag + PRIMARY KEY; `timestamp` = Time Index; `duration_nano`/`timestamp_end` generated. **Every attribute flattened to its own typed column** — `span_attributes.<k>`, `resource_attributes.<k>`, `scope_attributes.<k>` (exception: `resource_attributes.service.name` → `service_name`). `span_events`/`span_links`/compound values → `JSON`. `trace_id`/`parent_span_id`/`service_name` carry a **BLOOM `SKIPPING INDEX`**; table is **`PARTITION ON COLUMNS (trace_id)` (16-way)**. Schema **auto-widens** when a new attribute key appears. |
| **Logs** | `POST /v1/otlp/v1/logs` (table via `x-greptime-log-table-name`, default `opentelemetry_logs`) | `opentelemetry_logs` | `timestamp` (prefers `time_unix_nano`/`observed_time_unix_nano`) = Time Index, `trace_id`, `span_id`, `severity_text`, `body`, attributes as `JSON`. `append_mode='true'`, **NO PRIMARY KEY, NO `trace_id` index** (flat append). |
| **Metrics** | `POST /v1/otlp/v1/metrics` | **one logical table per metric name** (metric engine, Prometheus-compatible) | metric name = table name, auto-created; selected resource attributes kept as tag columns; PromQL-native. **ExponentialHistogram not yet supported.** |

Customization levers on auto-created tables: ingest hints `x-greptime-hints: ttl=…, append_mode=…`;
table-name headers; and — because these are ordinary tables — post-create `ALTER TABLE … ADD COLUMN`
and `ADD … INVERTED INDEX / FULLTEXT INDEX`. The schema auto-grows columns from new attributes.

## What blocks Parallax from the native tables today

The current code (`crates/parallax-storage/src/greptime.rs`, spec §5) hand-rolls six tables and
writes them with `INSERT` SQL over the HTTP API. The gaps between that and the native model:

1. **Parallax *is* the OTLP receiver (proxy-lens), so GreptimeDB never sees OTLP.** Native tables
   only auto-create when OTLP hits *GreptimeDB's* `/v1/otlp/` endpoint with the pipeline header.
   Parallax terminates OTLP itself (derivation/grouping/run-scoping/redaction workers) and then
   `INSERT`s rows. To get native tables, the greptime adapter must either **(A)** re-emit the
   processed OTLP to GreptimeDB's OTLP endpoint (true native, auto-schema, rides Greptime's
   optimizations) or **(B)** hand-write rows into a table whose *layout* matches native. This is the
   load-bearing architectural fork, not a column tweak.
2. **Opaque `JSON` attributes vs. flattened typed columns.** Custom stores `attributes`/`resource`
   as two `JSON` blobs and queries them with `json_get_string(...)`. Native explodes every attribute
   into its own column (`span_attributes.http.request.method`) with per-column codecs and a
   dynamically widening schema. This is the biggest gap: query rewrite (JSON paths → real columns)
   plus accepting an auto-growing wide schema. A hand-written `INSERT` (path B) **cannot reproduce
   dynamic flattening** without an `ALTER` per never-before-seen key — only the OTLP-endpoint path
   (A) gets it for free.
3. **Parallax-promoted first-class columns (`run_id`).** Today `parallax.run.id` is promoted to a
   real `run_id` column on spans/logs/metrics and is queried directly. Under native flattening it
   becomes `resource_attributes.parallax.run.id` (still a column — *not* a hard blocker), but every
   `run_id` query/filter must be remapped to the native column name.
4. **Metrics shape mismatch.** Custom = two unified tables (`otel_metrics_points`,
   `otel_metrics_histograms`) with `name` as a column and `PRIMARY KEY(service, name)`, queried with
   SQL `date_bin` aggregates. Native = one table *per metric name* via the metric engine, queried
   with PromQL. Adopting native means rewriting the metric query layer **and** handling the
   **ExponentialHistogram gap** (native unsupported → keep an explicit-bucket fallback).
5. **No anchor index on native logs.** Native `opentelemetry_logs` is flat append with no `trace_id`
   index, so a `trace_id` log lookup **scans** — and Run 56 showed `trace_id` retrieval is the
   evidence bundle's dominant cost. Fixable by `ALTER`-adding `trace_id INVERTED INDEX` (+ `message
   FULLTEXT`) after auto-create; this is the "adopt-then-customize" deviation, not a wall.
6. **Engine portability (the ClickHouse fallback).** ClickHouse has **no native OTLP ingest** — only
   GreptimeDB accepted the live native writes. So a GreptimeDB-native physical layout cannot be the
   one shared schema across engines. Portability must move *up* to the `StorageAdapter` API; the
   ClickHouse adapter hand-rolls an equivalent layout behind the same contract. (This is the cost the
   spec §5 already named, now made explicit.)
7. **Derived signals have no native form.** `error_events` and `rollups_fingerprint_minute` come from
   Parallax's fingerprinting/grouping — they are product semantics, not OTLP signals. GreptimeDB's
   native model has nothing for them. Not a *blocker* to going native; they are simply out of native
   scope and stay custom.

## Recommendation — adopt-then-customize, per signal

Match the engine sub-study's per-signal verdict (pass 119) and the founder's guidance:

- **Traces → ADOPT native `opentelemetry_traces`.** It is strictly better than hand-rolled
  `otel_spans`: bloom-indexed `trace_id` **and** 16-way `trace_id` partitioning prune anchored
  lookups (~1/16 partitions + bloom skip) without paying PK-cardinality cost. `ALTER`-add only the
  cross-signal columns native lacks (e.g. `fingerprint`).
- **Logs → ADOPT native `opentelemetry_logs`, then `ALTER`-add `trace_id INVERTED INDEX` +
  `message FULLTEXT INDEX`.** That one deviation fixes the dominant evidence-bundle cost.
- **Metrics → ADOPT the native metric engine (PromQL-native — a GreptimeDB DQ1 strength).** Keep a
  custom explicit-bucket histogram path until native ExponentialHistogram lands.
- **`error_events`, `rollups_fingerprint_minute` → KEEP custom.** No native equivalent; these are
  the Parallax layer.

**How the adapter writes native (the path-A vs path-B fork):**

- **Path A — re-emit OTLP to GreptimeDB's `/v1/otlp/` endpoint after Parallax processing.** Gains
  auto-schema flattening, dynamic column growth, the Jaeger/PromQL surfaces, and — the founder's
  point — Greptime's *ongoing* optimizations land on Parallax's data for free. Cost: less direct
  `INSERT` control; an extra serialize hop; native layout is GreptimeDB-only.
- **Path B — hand-write rows into native-*shaped* tables via SQL.** Keeps `INSERT` control but must
  reproduce attribute flattening manually (`ALTER` per new key) and will *not* automatically inherit
  Greptime's native-model optimizations. Weaker exactly where Ning's advice points.

**Lean: Path A for the GreptimeDB adapter**, because the whole reason to go native is to ride the
engine team's optimization roadmap, which Path B forfeits. Portability is preserved by keeping the
`StorageAdapter` contract backend-neutral — the ClickHouse adapter stays hand-rolled (Path B-style)
behind the same API. No product contract (evidence bundle, context API, grouping) may depend on the
GreptimeDB-native physical shape; it depends only on the adapter API.

## What stays custom (and why that's not a contradiction)

Going native for raw OTLP does **not** dissolve the proxy-lens decision (2026-05-25). Parallax still
terminates OTLP to derive `error_events`, fingerprint, run-scope, and redact. The change is only
*where the raw signals land*: instead of hand-rolled tables, the post-processed OTLP is written into
GreptimeDB's native tables, and the Parallax-only derived signals continue as custom tables. Parallax
remains the receiver; GreptimeDB owns the raw-signal physical schema.

## Open items before this is fully settled

- Confirm Path A preserves Parallax semantics end-to-end: that `parallax.run.id` survives as
  `resource_attributes.parallax.run.id` and that redaction happens *before* re-emit.
- Decide the metric-engine cutover: PromQL rewrite of the current SQL-aggregate query layer, and the
  ExponentialHistogram fallback lifetime.
- Measure native-table anchored retrieval vs the current hand-rolled tables on the same corpus
  (extend the four-build matrix) before deleting the custom span/log DDL.
- Raise on the next GreptimeDB sync: native-table custom-column/index stability across schema
  auto-widening, and the ExponentialHistogram timeline.
