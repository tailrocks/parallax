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
- **Q3 — Metrics. LEAN: forward all three signals uniformly** (traces+logs+metrics → native OTLP
  endpoints; the thin-forward is identical for all). Migrate the *read* layer incrementally; keep an
  explicit-bucket fallback until native ExponentialHistogram lands. (Native metric tables are still
  SQL-queryable, so PromQL rewrite can be gradual, not a blocker.)
- **Q4 — Existing data. LEAN: greenfield** (research stage — drop custom tables, start native fresh; no
  backfill).
- **Q5 — ClickHouse fallback. DECIDED (operator, 2026-06-18): do not keep ClickHouse as a boundary for
  now. Full focus on GreptimeDB.** Multi-store becomes a goal only if a concrete benefit appears — for
  now there is no clear benefit, so portability-to-ClickHouse is **not** a design constraint. The
  `StorageAdapter` trait may stay (it already exists, with the memory adapter for tests), but the
  design is free to use Greptime-native features (Flow, `digest`, HLL, uddsketch). This reverses the
  prior "ClickHouse is the fallback" lean in [decisions/storage-engine.md](../decisions/storage-engine.md)
  and [decisions/v1-storage-adapter-vision.md](../decisions/v1-storage-adapter-vision.md) for V1 scope.
- **Q6 — `run_id`. LEAN: use the native flattened column** `resource_attributes.parallax.run.id`;
  repoint all `run_id` queries. (Confirm the exact name end-to-end before deleting custom DDL.)
- **Q7 — Custom columns under auto-widening + traces GA.** Open — needs Greptime input: do `ALTER`-added
  columns/indexes survive dynamic schema growth, and what is the traces-OTLP long-term stability story.
  Raise on the next Greptime sync.
