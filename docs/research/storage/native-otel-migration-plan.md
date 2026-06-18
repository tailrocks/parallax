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
- **The proxy stays — but its reason is derivation, not multi-store routing.** All telemetry goes
  through Parallax first. For GreptimeDB it is a **very thin proxy** that forwards OTLP straight to
  Greptime's production-ready `/v1/otlp/` API (untouched on the raw-signal path) while teeing the same
  bytes in-process to derive `error_events` (Q2). The proxy is kept even though Greptime could receive
  OTLP directly because Parallax *is* the product entry point and the derivation tee, **not** because
  of a multi-store boundary. ClickHouse is **deferred** (Q5): GreptimeDB is the single focus; the
  `StorageAdapter` trait may remain (cheap, already exists, memory adapter for tests) but
  portability-to-ClickHouse is **no longer a design constraint**.
- **Rationale for thin-forward to Greptime:** GreptimeDB's team optimizes specifically around the
  native model (Ning Sun, Slack 2026-06-18); forwarding untouched lets Parallax inherit that roadmap
  for free, and the native OTLP API is GA/production-ready.
- **Issue grouping (Sentry-style) is the part Parallax must own** — operator's claim, verified below.
- **Native-first principle (operator, 2026-06-18).** Use the native GreptimeDB approach for the
  *entire* OpenTelemetry stack — nothing outside native for OTel signals. Only where something is
  **genuinely not capable** of being done natively do we ask "what is the minimal extension?" — an
  additional table or column alongside native — and even then we **extend, never replace** native.
  Default answer to every storage question is "do it the native way."
- **No migration, ever (operator, 2026-06-18).** Parallax is in the research state with no users (the
  operator runs it locally, fresh data on each spawn). We freely refactor / re-implement / redefine.
  There is **no data migration, no backfill, no dual-write, no parity-before-delete** — just rebuild
  on the native model. This is a *refactor* plan, not a data-migration plan.
- **Goal of this doc:** decide exactly what is native, what (if anything) must be a custom extension,
  and how — then re-implement on native models.

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
| `otel_metrics_points` + `otel_metrics_histograms` (2 unified tables, SQL aggregates) | metric engine, one table per metric (PromQL) | **Replace, fully native (Q3 = A).** Forward metrics OTLP → endpoint. Rewrite chart reads (native metric tables are SQL-queryable, so SQL→PromQL is gradual). **ExponentialHistogram is the one native gap** — rely on explicit-bucket histograms (OTel SDK default, fully native); add a minimal extension table *only if* exp-histograms actually appear (native-first principle). |
| `error_events` (Parallax fingerprinting) | — none — | **Keep custom.** Product semantics, no native form. |
| `rollups_fingerprint_minute` | — none — | **Keep custom.** Derived. |
| (new) run-scoped metrics | — none (metric engine can't hold run_id) — | **Add custom extension `run_metric_points`** (Approach 2, Q6): append table, `run_id STRING SKIPPING INDEX`, `append_mode='true'`, `flat` SST. Greptime's blessed high-card pattern. Metric engine stays run_id-free. |
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

## Build steps (greenfield — no migration, Q4)

1. **Delete the hand-rolled raw-signal DDL + write paths** (`otel_spans`, `otel_logs`,
   `otel_metrics_*`) outright. No parity gate, no backfill — fresh data each spawn.
2. Add the OTLP-forward path in the greptime adapter: re-emit received traces/logs/metrics to
   GreptimeDB's `/v1/otlp/v1/...` endpoints with the right pipeline/table headers + `x-greptime-hints`
   (ttl, append_mode).
3. Bootstrap deviations once after first auto-create: `ALTER` native logs to add `trace_id INVERTED
   INDEX` + body `FULLTEXT`; `ALTER` native traces for any extra column we keep.
4. Repoint reads (`greptime.rs` SELECTs) to native column names (`span_attributes.*`,
   `resource_attributes.parallax.run.id`, `duration_nano`, etc.). Rewrite metric reads against the
   per-metric native tables (SQL first; PromQL where it helps).
5. Wire the derivation tee (Q2): parse forwarded OTLP in-process → `error_events`. Keep `error_events`
   + `rollups` as custom extension tables (native has no equivalent — native-first principle).
6. Create the `run_metric_points` extension table (Q6, Approach 2) for run-scoped metrics; route
   per-run metric points there (run_id column), keep aggregate metrics on the native metric engine.
7. Only if a further native gap actually bites (e.g. exp-histograms): add another minimal extension.

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

## Grouping: what Parallax wants vs. what GreptimeDB gives (deep map)

Sentry-style issue tracking is a pipeline. Mapping each stage to its owner shows the clean seam:
**GreptimeDB owns stateless aggregation math over append-only signals; Parallax owns fingerprint
intelligence + stateful issue identity/lifecycle + bundle assembly + alerting.**

| Stage | What it does | Owner | GreptimeDB mechanism / note |
| --- | --- | --- | --- |
| Raw store | spans / logs / metrics at rest | **Greptime** | native OTLP tables |
| Error extraction | find exception events, error spans (`STATUS_CODE_ERROR`), ERROR/FATAL logs | **Parallax (tee)** | could be a Greptime query, but the tee already holds the bytes (one pass) |
| Message normalization | strip variable parts → stable template | Greptime-capable; canonical in Parallax | `digest` processor (presets: numbers/uuid/ip/quoted/bracketed + regex). **Flat string only** |
| Stacktrace fingerprint | normalize frames, in-app vs lib, top-N → key | **Parallax only** | not expressible in `digest`/SQL — the real grouping intelligence |
| Custom grouping rules | fingerprint overrides, merge rules | **Parallax only** | app logic |
| Fingerprint hash | template/type/frames → hash | either | SQL `sha512`; canonical algo stays in Parallax |
| Issue identity | find-or-create issue by fingerprint, merge/unmerge | **Parallax (Turso)** | mutable OLTP — never a timeseries store |
| Issue lifecycle | status (resolved/ignored/regressed), assign, snooze | **Parallax (Turso)** | mutable state; no native form |
| Count / first-seen / last-seen | per fingerprint | **Greptime-capable (Flow)** | `GROUP BY fingerprint → count, min(ts), max(ts)` |
| Trend sparkline | count per fingerprint per window | **Greptime (Flow)** | == `rollups_fingerprint_minute`, server-side |
| Users/sessions affected | approximate unique count | **Greptime-capable** | `hll` / `approx_distinct` (HyperLogLog) |
| Latency / percentiles per issue or span | p50/p95/p99 | **Greptime-capable** | `uddsketch` Flow **directly over `opentelemetry_traces`** (docs "extend-trace") |
| Tag-value distribution | top values per key | **Greptime-capable** | `GROUP BY tag` (watch cardinality) |
| Issue search / filter | by status + tags + time | **Hybrid** | state from Turso; signal filters can hit Greptime |
| Evidence-bundle assembly | join error → spans → logs → metrics → deploys | **Parallax** | cross-signal orchestration |
| Alerting / notify | new / regressed / threshold | **Parallax** | GreptimeDB OSS has no built-in alerting |

**Where Greptime genuinely helps Parallax:** it can offload all the *counting* — issue counts, trend
rollups, unique-users (HLL), latency percentiles (uddsketch) — as continuous Flows over the native
tables, so Parallax doesn't re-scan to recompute aggregates. That is real leverage, not nothing.

**Where the operator's instinct holds:** the *intelligence* (stacktrace fingerprinting, custom
grouping) and the *state* (issue identity + lifecycle) must be Parallax — because a timeseries store
cannot express structured-frame fingerprinting or mutable issue state, **not** because of portability.
With ClickHouse deferred (Q5), Greptime-native acceleration (`digest`, Flow, HLL, uddsketch) is now
*more* freely usable — there is no second engine to hold the design back. Still, keep the **canonical
fingerprint algorithm + issue state authoritative in Parallax** for control and correctness; treat
Greptime Flows/sketches as a derived acceleration layer Parallax owns and can recompute. Greptime
accelerates; Parallax decides.

## Implementation roadmap — current code → required changes

Mapped against the live code (2026-06-18). The good news: the ingest worker **already tees** —
`worker.rs::process` normalizes *and* derives from the same OTLP request — so Q2's structure exists.
What changes is the **write target** (custom INSERT → native OTLP forward) and the **read layer**
(custom columns → native columns). The `TelemetryStore` *read* signatures are the stable boundary the
GraphQL API depends on, so most of `parallax-api` is untouched.

### As-is flow

```text
OTLP HTTP/gRPC (otlp_http.rs / otlp_grpc.rs)
  → decode protobuf → spool.append → mpsc → worker.rs::process
       Traces:  normalize_traces → SpanRow[];  derive_from_traces → errors
                register_runs; live broadcast;  store.write_spans (INSERT custom otel_spans)
                record_errors → metadata.upsert_issue + store.write_error_events
       Logs:    normalize_logs → LogRow[];  derive_from_logs;  store.write_logs (INSERT otel_logs)
       Metrics: normalize_metrics → points+histograms; store.write_metric_points/_histograms (INSERT)
  Reads: parallax-api GraphQL → TelemetryStore read methods → SELECT custom otel_* tables
```

### Target flow

```text
OTLP in → otlp_http/grpc (keep raw Bytes alongside decoded request) → spool → mpsc → worker
   Traces:  store.forward_traces(raw)  → POST greptime /v1/otlp/v1/traces (greptime_trace_v1)
            + derive_from_traces → error_events (tee, unchanged)  + register_runs + live
   Logs:    store.forward_logs(raw)    → POST /v1/otlp/v1/logs (extract-keys: parallax.run.id)
            + derive_from_logs (tee)
   Metrics: store.forward_metrics(raw) → POST /v1/otlp/v1/metrics (native metric engine, NO run_id tag)
            + write run_metric_points from normalized points carrying run_id (Approach 2)
   Reads: GraphQL → SAME TelemetryStore read signatures → SELECT native tables (new column names)
```

### Change list by file

| File | Current role | Required change |
| --- | --- | --- |
| `parallax-storage/src/greptime.rs` | hand-rolled DDL + INSERT + custom-table SELECTs | **Largest change.** (1) `bootstrap`: drop the `otel_spans/otel_logs/otel_metrics_*` CREATE; keep CREATE for extension tables (`error_events`, `rollups_fingerprint_minute`, new `run_metric_points`). (2) Add `forward_traces/forward_logs/forward_metrics` → `POST {base_url}/v1/otlp/v1/…` with headers (`x-greptime-pipeline-name: greptime_trace_v1`, `x-greptime-log-table-name`, `x-greptime-log-extract-keys: parallax.run.id`, `x-greptime-hints: ttl=…,append_mode=true`). (3) Rewrite every read to native columns: traces `service_name`/`duration_nano`/`span_attributes.*`/`resource_attributes.*`; logs `body` + JSON attrs + extracted `parallax.run.id`; metrics → per-metric native tables (+ `information_schema.tables` for `metric_names`). (4) Add `run_metric_points` write + read. |
| `parallax-storage/src/adapter.rs` | `TelemetryStore` trait | **Reshape the write side:** replace `write_spans/write_logs/write_metric_points/write_histograms` with `forward_traces/forward_logs/forward_metrics` (raw OTLP), add `write_run_metric_points` + a run-scoped metric read. **Read signatures stay** (API-stable). *(Sub-decision: how the in-memory test adapter satisfies a raw-OTLP write — see open impl questions.)* |
| `parallax-server/src/worker.rs` | normalize + derive + write | Swap `store.write_*` for `store.forward_*` (pass the raw request/bytes). Keep `normalize_*` (still needed for live-tail, `register_runs`, `derive_from_logs`, and run-metric extraction). Metrics arm: forward raw **and** write `run_metric_points` from normalized points that carry `run_id`. |
| `parallax-server/src/otlp_http.rs` / `otlp_grpc.rs` | decode → spool → queue | Keep the **original `Bytes`** next to the decoded request and hand both to the worker, so forwarding re-emits the original payload (no re-encode) — honors the zero-copy ingest rule. |
| `parallax-core/src/normalize.rs` | OTLP → rows | Keep. `run_id` already read from resource attr `parallax.run.id` (matches Q6). Still feeds live-tail, run registration, derivation, and run-metric points. |
| `parallax-core/src/model.rs` | row DTOs | `SpanRow`/`LogRow` remain the read/live DTO (map native query rows into them). Add `RunMetricPointRow`. |
| `parallax-server/src/serve.rs` | calls `bootstrap` | Adjust bootstrap: native tables auto-create on first OTLP, so the `ALTER` (logs `trace_id INVERTED` + `body FULLTEXT`) must run **after** the table exists — handle ordering (pre-touch the table, or ALTER lazily/idempotently after first forward). Create extension tables up front. |
| `parallax-api/src/lib.rs` | GraphQL resolvers | Mostly unchanged (read signatures stable). Verify metric resolvers handle per-metric native tables and that run-scoped metric queries hit `run_metric_points`. |
| `parallax-server/src/greptime_supervisor.rs` / `config.rs` | manage local engine, TTLs | Forward target = the managed local Greptime HTTP base URL (already known). TTLs now ride `x-greptime-hints` on forward + `WITH(ttl)` on extension tables. |
| `poc/evidence-loop/*` | frozen reference | Untouched (frozen). Update `crates` tests for the new write/read shapes. |

### Open implementation questions (resolve during build)

- **IQ1 — trait write shape + memory adapter.** Native forward needs the raw OTLP request; the
  in-memory test adapter wants normalized rows. Options: (a) `forward_*` takes raw OTLP and the memory
  adapter decodes internally; (b) keep both a normalized-write path (memory/tests) and a raw-forward
  path (greptime). Lean (a) for one contract, but confirm test ergonomics.
- **IQ2 — bootstrap/ALTER ordering.** Auto-created native tables don't exist until first ingest, but
  the log indexes need `ALTER`. Decide: pre-create the native log table explicitly with our columns +
  indexes (does Greptime then accept OTLP into it?), or ALTER idempotently right after the first
  forward succeeds.
- **IQ3 — zero-copy forward.** Forward the original spooled `Bytes` rather than re-encoding the decoded
  proto (AGENTS zero-copy ingest rule). Confirm the spool already holds the exact bytes to replay.
- **IQ4 — metric read rewrite depth.** How much of the metric read layer goes SQL-over-native vs
  PromQL now (Q3 said SQL-first, PromQL where it helps).

## Open questions → current decisions / leans

- **Q1 — Redaction (A6). DECIDED (operator, 2026-06-18): forward raw OTLP as-is, no redaction, straight
  to Greptime's OTLP API.** Consequence to record: raw telemetry is stored **unredacted at rest** in
  GreptimeDB. Acceptable for the self-hosted / local-first V1 (operator controls the data). **Revisit
  trigger:** a managed / multi-tenant / cloud profile re-opens this — redaction would move onto the
  forward path or to ingest-side scrubbing there.
- **Q2 — Derivation source. DECIDED (operator, 2026-06-18): tee in-flight.** When the proxy receives
  OTLP it does two things in one pass: (1) forward the bytes to Greptime untouched, (2) parse the same
  bytes in memory → extract errors → fingerprint → write `error_events`. No second round trip, no lag,
  no reading back what we just wrote. The rejected alternative ("read-back": forward only, then query
  Greptime later to pull errors back out) was simpler on the forward path but paid redundant I/O + lag.
- **Q3 — Metrics. DECIDED (operator, 2026-06-18): fully native (Option A).** Forward all three signals
  uniformly (traces+logs+metrics → native OTLP endpoints); nothing outside native for OTel. Rewrite the
  metric read layer against per-metric native tables (SQL first, PromQL where it helps). Per the
  native-first principle, the only native gap (ExponentialHistogram) is handled by relying on
  explicit-bucket histograms; a minimal extension table is considered *only if* exp-histograms appear.
- **Q4 — Existing data. DECIDED (operator, 2026-06-18): greenfield, no migration.** No users, fresh
  data each spawn — delete custom tables and rebuild native outright. No backfill, dual-write, or
  parity gate.
- **Q5 — ClickHouse fallback. DECIDED (operator, 2026-06-18): do not keep ClickHouse as a boundary for
  now. Full focus on GreptimeDB.** Multi-store becomes a goal only if a concrete benefit appears — for
  now there is no clear benefit, so portability-to-ClickHouse is **not** a design constraint. The
  `StorageAdapter` trait may stay (it already exists, with the memory adapter for tests), but the
  design is free to use Greptime-native features (Flow, `digest`, HLL, uddsketch). This reverses the
  prior "ClickHouse is the fallback" lean in [decisions/storage-engine.md](../decisions/storage-engine.md)
  and [decisions/v1-storage-adapter-vision.md](../decisions/v1-storage-adapter-vision.md) for V1 scope.
- **Q6 — `run_id`. DECIDED (operator, 2026-06-18).** Emit `parallax.run.id` as a **resource
  attribute**. Per signal:
  - **Traces:** flattens to the real column `resource_attributes.parallax.run.id` — free, queryable
    (no problem).
  - **Logs:** promote to a real column at ingest via `X-Greptime-Log-Extract-Keys: parallax.run.id`
    (else it lives in the logs JSON attributes) — no problem.
  - **Metrics:** **never put `run_id` on the metric engine** — it is high-cardinality (a new value
    every run) and a metric tag = one series per run = cardinality explosion (own research Run
    114/115: cost scales with series count; ~1-point series are the catastrophic 5×–80× case). The
    metric engine carries only low-card tags (service/name).
  - **Run-scoped metrics → Approach 2 (events table).** Add a custom append table (e.g.
    `run_metric_points`): `ts, run_id STRING SKIPPING INDEX, service, name, value, attributes`,
    `append_mode='true'`, `flat` SST. `run_id` is a **column, not a tag**, so high cardinality is free
    (events behave like logs/spans). This is **GreptimeDB's own documented high-cardinality pattern**
    (`http_logs_v4`: `request_id STRING SKIPPING INDEX`), so it honors native-first — the metric engine
    stays for aggregates; this is an *added* table, not a replacement. (Time-window reconstruction via
    Turso `RunRecord` start/end and span-derived metrics over native traces remain available as
    no-storage complements, but Approach 2 is the chosen primary for exact per-run metrics.)
- **Q7 — Questions for the GreptimeDB team (open — needs their input).** Eight detailed questions
  (custom columns/indexes vs schema auto-widening, indexing native logs post-create, adding Parallax
  columns to native traces, log attribute promotion, traces OTLP GA/stability, high-card metric
  pattern confirmation, OTLP forward performance, ExponentialHistogram timeline) live in their own
  doc, ready to review on the next sync: **[greptimedb-team-questions.md](greptimedb-team-questions.md)**.
