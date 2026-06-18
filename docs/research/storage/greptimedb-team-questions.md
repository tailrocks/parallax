# Questions to the GreptimeDB Team

<!-- markdownlint-disable MD013 -->

Status: open · Created 2026-06-18. Detailed questions to review with the GreptimeDB team on the next
sync. They back the native-OTLP adoption decisions in
[native-otel-migration-plan.md](native-otel-migration-plan.md) and
[../decisions/native-otel-tables.md](../decisions/native-otel-tables.md). Each item states our
context, our current assumption, the exact question, why it matters to Parallax, and our fallback if
the answer is "no". Ranked load-bearing first.

## Context for the team (one paragraph)

Parallax is an AI-native error-tracking / observability product (a Sentry/DataDog-style layer) built
on GreptimeDB. Parallax terminates OTLP itself (it is the receiver) and, per the team's guidance,
**forwards raw OTLP straight to GreptimeDB's `/v1/otlp/…` endpoints untouched** so the data lands in
the **native** tables (`opentelemetry_traces`, `opentelemetry_logs`, the metric engine). In the same
pass Parallax derives its own product signals (error grouping / "issues") into a few **custom
extension tables** alongside the native ones. We want to lean on the native model as much as possible
and only extend where native genuinely can't do something. The questions below are about the seams
where we customize native tables.

---

## 1. Custom columns / indexes vs. schema auto-widening (load-bearing)

- **Context.** Native OTLP tables auto-add a column when a new attribute key appears. We also plan to
  `ALTER` these tables to add our own things (an index, or a Parallax-specific column).
- **Our assumption.** Manual `ALTER`-added columns and indexes persist and keep working as the schema
  auto-widens from new OTLP attributes.
- **Question.** Do manually added columns/indexes survive dynamic schema growth? What happens if an
  incoming OTLP attribute maps to a column name that already exists because we added it (type/semantic
  conflict, ingest error, silent coercion)?
- **Why it matters.** Our entire "adopt native, then customize" plan depends on this being safe. If
  auto-widening can drop or conflict with our changes, we need a different strategy.
- **Fallback if no.** Reserve a Parallax namespace/prefix for our columns, or keep derived data only
  in separate extension tables and never `ALTER` native tables.

## 2. Traces OTLP model GA / long-term stability (load-bearing)

- **Context.** We build product features on `opentelemetry_traces` (created via the
  `greptime_trace_v1` pipeline) — span tree, anchored `trace_id` retrieval, the Jaeger-style surfaces.
- **Our assumption.** The trace data model + pipeline are production-grade and stable enough to build a
  product on.
- **Question.** Is `greptime_trace_v1` / `opentelemetry_traces` GA and committed long-term? Is there a
  schema-change / versioning policy for it (e.g. column renames, partition strategy changes)?
- **Why it matters.** A product depends on this table's shape. We need confidence it won't shift under
  us, or a clear migration/versioning contract if it does.
- **Fallback if no.** Pin a GreptimeDB version, gate upgrades behind our own validation, and isolate
  trace reads behind our adapter so a model change is a localized fix.

## 3. Indexing the native logs table after creation (load-bearing)

- **Context.** The auto-created `opentelemetry_logs` is flat append with **no `trace_id` index**, so a
  `trace_id` log lookup scans. In our workload `trace_id` retrieval is the dominant evidence-bundle
  cost, so we need it indexed. We also want full-text search on the log body.
- **Our assumption.** We can `ALTER` the auto-created table to add `trace_id INVERTED INDEX` and a
  `FULLTEXT INDEX` on the body, and ingest keeps working.
- **Question.** Is adding these indexes to the native logs table supported and stable post-create,
  without breaking subsequent OTLP log ingest? Any preferred index type for a high-cardinality
  `trace_id` on an append table (inverted vs skipping)?
- **Why it matters.** This is our one required deviation from the native log schema; the bundle hot
  path needs it.
- **Fallback if no.** Use a custom logs extension table for indexed `trace_id` lookups, or pre-create
  the log table with our indexes and point OTLP ingest at it (see Q4/Q5 on whether ingest accepts a
  pre-created table).

## 4. Adding Parallax-specific columns to native tables

- **Context.** We may want a Parallax column on a native table (e.g. `fingerprint` on
  `opentelemetry_traces`) that OTLP itself doesn't carry.
- **Our assumption.** `ALTER ADD COLUMN` is the supported way, and OTLP ingest leaves a column it
  doesn't know about untouched (NULL), without erroring.
- **Question.** Is `ALTER ADD COLUMN` the blessed path for product-specific columns on native OTLP
  tables? Does OTLP ingest tolerate/ignore columns outside the OTLP mapping?
- **Why it matters.** Decides whether we enrich native tables in place or keep all derived fields in
  separate extension tables.
- **Fallback if no.** Keep all Parallax-derived fields in extension tables keyed by `trace_id` /
  `span_id` and join at read time.

## 5. Promoting a log attribute to a real column

- **Context.** Native logs keep attributes as JSON by default. We need `parallax.run.id` as a real,
  queryable column on logs (it is core to run-scoped retrieval).
- **Our assumption.** The `X-Greptime-Log-Extract-Keys: parallax.run.id` ingest header promotes that
  attribute to its own column, and the column can be indexed.
- **Question.** Confirm `X-Greptime-Log-Extract-Keys` promotes a (resource/log) attribute to a real
  column on `opentelemetry_logs`, and that we can add an index on the promoted column. Does it work
  for resource-level attributes specifically?
- **Why it matters.** Run-scoped log queries should hit a column, not a JSON path.
- **Fallback if no.** Query the attribute via JSON path functions, accepting slower run-scoped log
  reads, or store run-scoped pointers in an extension table.

## 6. High-cardinality metrics — confirm the recommended pattern

- **Context.** We need "metrics for one CLI run" (`run_id` is high-cardinality — a new value per run).
  We will **not** put `run_id` on the metric engine (it would create one series per run = cardinality
  explosion). Instead we plan a separate **append/event table** with `run_id STRING SKIPPING INDEX`
  (modeled on the `http_logs_v4` high-cardinality example), keeping the metric engine for low-card
  aggregates only.
- **Our assumption.** This events-table-with-skipping-index pattern is the recommended GreptimeDB way
  to handle per-entity high-cardinality "metrics".
- **Question.** Is this the pattern you'd recommend? Is there any native per-entity / high-cardinality
  metric mechanism we're missing (e.g. a way to scope metric-engine data by a high-card key without
  the series blow-up)?
- **Why it matters.** Confirms our one custom metrics extension is the right shape before we build it.
- **Fallback if no.** Reconstruct run-scoped metrics purely from time windows (run start/end) and from
  span-derived aggregates over `opentelemetry_traces`.

## 7. OTLP forward performance from a proxy

- **Context.** Parallax re-emits received OTLP into GreptimeDB's `/v1/otlp/…` endpoints at volume,
  rather than using the SQL insert path.
- **Our assumption.** The OTLP endpoints are the right high-throughput ingest path for a forwarding
  proxy.
- **Question.** Any guidance for a proxy forwarding OTLP at volume — gRPC vs HTTP, batching, payload
  compression, connection/concurrency tuning, expected throughput vs the SQL-insert path?
- **Why it matters.** Forwarding is now our only write path for raw signals; we want it efficient.
- **Fallback if no.** Tune batch sizes empirically and benchmark gRPC vs HTTP ourselves.

## 8. ExponentialHistogram support timeline

- **Context.** The native metric engine does not yet support OTLP ExponentialHistogram. We will rely
  on explicit-bucket histograms and only add a minimal extension if exponential ones appear.
- **Our assumption.** Explicit-bucket histograms are fully supported; exponential support is on the
  roadmap.
- **Question.** Is there a timeline for native ExponentialHistogram support?
- **Why it matters.** Determines how long any explicit-bucket fallback needs to live.
- **Fallback if no.** Keep a small custom histogram extension table for exponential histograms until
  native support lands.

---

## After the call

Record answers inline under each question, then graduate the resolved ones into
[native-otel-migration-plan.md](native-otel-migration-plan.md) (Q7) and the build plan, and update any
fallback that becomes the chosen path.
