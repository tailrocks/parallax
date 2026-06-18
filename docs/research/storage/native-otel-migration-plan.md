# Migration to GreptimeDB Native OTLP Tables — Working Plan

<!-- markdownlint-disable MD013 -->

Status: **living working doc** (started 2026-06-18). This is the iteration surface for moving
Parallax off its hand-rolled telemetry tables onto GreptimeDB's native OTLP model. It is *not* a
settled ADR — decisions get debated here, and only the resolved ones graduate into
[decisions/native-otel-tables.md](../decisions/native-otel-tables.md) and the
[v1-implementation-spec.md §5](../architecture/v1-implementation-spec.md). Open questions live at the
bottom and are answered with the operator over time.

## Direction (operator, 2026-06-18)

- **Use GreptimeDB native tables directly.** Adopt the native OTLP model for the raw signals.
- **The proxy stays — as a storage-layer router, not just an OTLP receiver.** All telemetry goes
  through Parallax first. The proxy *decides* per backend: for GreptimeDB it is a **very thin proxy**
  that forwards OTLP straight through to Greptime's production-ready `/v1/otlp/` API (untouched on the
  raw-signal path); for a future ClickHouse (or other) profile it processes/writes itself. This is
  why the proxy is kept even though Greptime could receive OTLP directly — it preserves the
  multi-store `StorageAdapter` boundary (GreptimeDB now, ClickHouse later).
- **Rationale for thin-forward to Greptime:** GreptimeDB's team optimizes specifically around the
  native model (Ning Sun, Slack 2026-06-18); forwarding untouched lets Parallax inherit that roadmap
  for free, and the native OTLP API is GA/production-ready.
- **Issue grouping (Sentry-style) is the part Parallax must own** — operator's claim, verified below.
- **Goal of this doc:** decide exactly what migrates, what stays custom, and how — then execute the
  migration to native models.

## Why native (recap of the finding)

- GreptimeDB's founder confirmed the team will optimize around the **internal data model** and
  recommends it; ecosystem products (e.g. hebo.ai) build on top.
- The engine sub-study already verified the native trace model live and rated it **better** than the
  hand-rolled `otel_spans` (bloom-indexed `trace_id` + 16-way `trace_id` partitioning) —
  [greptimedb-implementation.md](greptimedb-vs-clickhouse/greptimedb-implementation.md), pass 119 / Run 86.

## The native OTLP model (verified — official docs + live engine)

| Signal | Endpoint + header | Native table | Shape |
| --- | --- | --- | --- |
| **Traces** | `POST /v1/otlp/v1/traces`, `x-greptime-pipeline-name: greptime_trace_v1` | `opentelemetry_traces` (+ `_services`, `_operations` for Jaeger API) | 1 row/span. `service_name` = Tag + PK; `timestamp` = Time Index; `duration_nano`/`timestamp_end` generated. **Every attribute → its own typed column**: `span_attributes.<k>`, `resource_attributes.<k>`, `scope_attributes.<k>` (except `resource_attributes.service.name` → `service_name`). `span_events`/`span_links`/compound → `JSON`. `trace_id`/`parent_span_id`/`service_name` = BLOOM `SKIPPING INDEX`. `PARTITION ON COLUMNS (trace_id)` (16-way). Schema **auto-widens** on new attribute keys. |
| **Logs** | `POST /v1/otlp/v1/logs` (`x-greptime-log-table-name`, default `opentelemetry_logs`) | `opentelemetry_logs` | `timestamp` (prefers `time_unix_nano`/`observed_time_unix_nano`), `trace_id`, `span_id`, `severity_text`, `body`, attributes as `JSON`. `append_mode='true'`. **No PK, no `trace_id` index** (flat append). |
| **Metrics** | `POST /v1/otlp/v1/metrics` | **one logical table per metric name** (metric engine, Prometheus-compatible) | metric name = table name, auto-created; selected resource attributes kept as tag columns; PromQL-native. **ExponentialHistogram not yet supported.** |

Customization levers: ingest hints `x-greptime-hints: ttl=…, append_mode=…`; table-name headers; and
post-create `ALTER TABLE … ADD COLUMN` / `ADD … INVERTED INDEX / FULLTEXT INDEX` (they are ordinary
tables). Schema auto-grows columns from new attributes.

## Current → target (per signal)

| Today (custom, `greptime.rs`) | Target (native) | Migration action |
| --- | --- | --- |
| `otel_spans` (PK `service`, attrs as JSON) | `opentelemetry_traces` | **Replace.** Forward traces OTLP → endpoint. `ALTER`-add any cross-signal column we still need (`fingerprint`?). Native is strictly better here. |
| `otel_logs` (PK `service`, attrs as JSON) | `opentelemetry_logs` | **Replace + customize.** Forward logs OTLP → endpoint, then `ALTER`-add `trace_id INVERTED INDEX` + `message/body FULLTEXT INDEX` (the one native shortfall; Run 56 = dominant bundle cost). |
| `otel_metrics_points` + `otel_metrics_histograms` (2 unified tables, SQL aggregates) | metric engine, one table per metric (PromQL) | **Replace, with a fallback.** Forward metrics OTLP → endpoint. Rewrite chart query layer SQL→PromQL. **ExponentialHistogram gap** → keep an explicit-bucket path until native supports it. *(Open question Q3.)* |
| `error_events` (Parallax fingerprinting) | — none — | **Keep custom.** Product semantics, no native form. |
| `rollups_fingerprint_minute` | — none — | **Keep custom.** Derived. |
| Turso metadata (issues, runs, dashboards) | — none — | **Keep.** Unaffected. |

## The architecture tension to resolve first

The operator's "forward OTLP straight through **without doing anything**" is the right move for raw
storage — but Parallax's product value (derived `error_events`, fingerprinting, run-scoping,
**redaction**) needs the OTLP *content*. So "forward untouched" cannot mean "forward and forget."
The realistic shape is a **tee / fan-out**:

```text
OTLP in ──► [Parallax receiver]
               ├──► forward raw OTLP ──► GreptimeDB /v1/otlp/...  (native tables, untouched)
               └──► derive (error_events, fingerprint, run-scope) ──► custom tables
```

Two things this forces a decision on (Q1, Q2 below): **redaction** (if raw OTLP is stored untouched,
redaction-before-storage is gone — is that acceptable, or does redaction move to query-time / opt-in?),
and **derivation source** (derive from the in-flight OTLP on the tee, vs. read back from native tables).

## Draft migration steps (subject to the open questions)

1. Add an OTLP-forward path in the greptime adapter: re-emit received traces/logs/metrics to
   GreptimeDB's `/v1/otlp/v1/...` endpoints with the right pipeline/table headers + `x-greptime-hints`
   (ttl, append_mode).
2. Bootstrap deviations once after first auto-create: `ALTER` native logs to add `trace_id INVERTED
   INDEX` + body `FULLTEXT`; `ALTER` native traces for any extra column we keep.
3. Repoint reads (`greptime.rs` SELECTs) to native column names (`span_attributes.*`,
   `resource_attributes.parallax.run.id`, `duration_nano`, etc.). Rewrite metric reads to PromQL.
4. Keep `error_events` + `rollups` custom; wire derivation to the tee (Q2).
5. Measure native vs hand-rolled anchored retrieval on the same corpus (extend the four-build matrix)
   **before** deleting the custom span/log DDL.
6. Delete the hand-rolled span/log/metric DDL + write paths once parity is proven.

## Claim verification — "grouping must be Parallax; can't delegate to GreptimeDB" (2026-06-18)

Operator claim: *the proxy stays as a storage router; for Greptime it is thin-forward; but
issue/error grouping (Sentry-style) must be processed by Parallax because no system can aggregate
this for us and there's no straightforward way to delegate it to GreptimeDB.* Verified against
GreptimeDB capabilities (official docs, v1.0):

**Verdict: ~50–60% real. The conclusion (process in Parallax) is right; the stated reason (Greptime
can't do grouping) is partly wrong. The strongest reason is portability, not incapability.**

What GreptimeDB **can** do toward grouping (so the "no system can" framing is overstated):

- **Message normalization** — the `digest` pipeline processor strips variable content on ingest
  (`numbers`, `uuid`, `ip`, `quoted`, `bracketed` presets + custom regex) → a stable message
  *template*. That is exactly Sentry-style *log-message* grouping-key extraction, native.
- **Fingerprint hashing** — SQL `sha512` (+ other hash funcs); VRL processor can derive fields on
  ingest. A template → hash is expressible.
- **Issue rollups/counts** — the **Flow engine** does continuous `GROUP BY key, time_window →
  count/min/max/avg` into a sink table. `rollups_fingerprint_minute` is reproducible server-side as a
  Flow, no app code.

What GreptimeDB genuinely **cannot** (correctly) do → must stay Parallax:

- **Stacktrace-based fingerprinting** (Sentry's strongest grouping): normalize frames, in-app vs
  library, group by top-N frames. `digest` works on a flat string, not structured frames. App-side.
- **Mutable issue state machine** — status (resolved/ignored/regressed), assignment, merge/unmerge,
  snooze. OLTP mutable state → Turso metadata, never an append/timeseries store.
- **Cross-signal evidence-bundle assembly** + custom grouping-rule overrides. App orchestration.

**The decisive reason the operator is still right:** even where Greptime *could* group (digest +
Flow), doing it *inside* Greptime via pipelines/Flow **couples the grouping logic to GreptimeDB and
breaks the storage-router** — ClickHouse (the kept fallback) cannot run Greptime pipelines or Flow.
So to keep the multi-store boundary, the **canonical fingerprint + grouping must live in Parallax**
regardless of Greptime's capability. Greptime's digest/Flow are best treated as an *optional
in-adapter optimization*, not the source of truth.

Corollary for "read data back to group": with the proxy tee, fingerprinting can run on the
**in-flight OTLP** (no read-back needed). Read-back (or a Greptime Flow feeding a sink Parallax
reads) is an *alternative*, not a requirement — it trades hot-path simplicity for query/lag cost.
This maps directly to Q2.

Also verified: **the native OTLP API is GA/production-ready** for logs and metrics; traces OTLP
(`greptime_trace_v1`) is documented and live-verified (Run 86) but newer — confirm long-term
stability on the next Greptime sync (Q7).

## Open questions (answer with operator, iterate)

- **Q1 — Redaction (A6).** If raw OTLP is forwarded untouched to Greptime, redaction-before-storage is
  lost. Acceptable for V1 (redact at query/projection time, or rely on SDK-side scrubbing)? Or must the
  forward path redact first (then it's not "untouched")?
- **Q2 — Derivation source.** Derive `error_events`/fingerprints from the in-flight OTLP on the tee
  (before/parallel to forwarding), or read them back from the native tables asynchronously?
- **Q3 — Metrics now or later.** Adopt the native metric engine + PromQL in this migration, or keep the
  custom metrics tables for V1 and migrate metrics in a later pass? (ExponentialHistogram gap + query
  rewrite cost.)
- **Q4 — Existing data.** Research stage → greenfield (drop custom tables, start native fresh), or write
  a backfill/dual-write window?
- **Q5 — ClickHouse fallback.** Native commitment is GreptimeDB-only at the physical layer. Keep the
  `StorageAdapter` ClickHouse boundary alive (portability cost), or formally narrow V1 to GreptimeDB and
  revisit ClickHouse later?
- **Q6 — `run_id` access.** Confirm `parallax.run.id` lands as `resource_attributes.parallax.run.id`
  under native flattening, and repoint all `run_id` queries to that column name.
- **Q7 — Custom columns under auto-widening.** Confirm `ALTER`-added columns/indexes on native tables
  survive GreptimeDB's dynamic schema growth (raise on next Greptime sync).
