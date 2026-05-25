# Storage Freshness and Bundle Latency Gate

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This is the proof gate for the two remaining storage-speed claims that the local
Docker smoke runs did not prove:

1. GreptimeDB ingest-to-queryable freshness for mixed logs, traces, metrics, and
   errors.
2. Evidence-bundle query latency under concurrent ingest.

This document narrows those claims into a runnable gate for
[Storage benchmark prototype](storage-benchmark-prototype.md). The existing smoke
runs confirmed row-count correctness, key-placement sensitivity, anchored join
plans, and GreptimeDB PromQL nativeness. They did not prove that freshly ingested
production evidence becomes visible quickly enough while the system is also
serving agent-context bundle queries.

## Current Source Posture

As of 2026-05-25, primary docs support the shape of the benchmark but do not
settle the performance question:

- GreptimeDB 1.0 positions itself as a unified observability database for
  metrics, logs, and traces, with SQL, PromQL, Prometheus remote write, OTLP,
  Jaeger, MySQL, and PostgreSQL protocol support
  ([GreptimeDB docs](https://docs.greptime.com/)).
- GreptimeDB's trace ingestion docs say traces use OpenTelemetry OTLP/HTTP and
  that trace docs are still experimental; trace data is queryable through SQL
  once stored
  ([GreptimeDB trace ingestion/query docs](https://docs.greptime.com/user-guide/traces/read-write/)).
- GreptimeDB's quick start demonstrates metrics, logs, traces, cross-signal SQL
  joins, native PromQL, and PromQL embedded in SQL via TQL
  ([GreptimeDB quick start](https://docs.greptime.com/getting-started/quick-start/)).
- GreptimeDB's schema guide says table design materially affects write and query
  performance, recommends append-only tables when dedup/delete are unnecessary,
  and recommends skipping indexes for high-cardinality columns like `trace_id`
  and `request_id`
  ([GreptimeDB table design](https://docs.greptime.com/user-guide/deployments-administration/performance-tuning/design-table/)).
- ClickHouse insert docs say async inserts buffer data in memory and flush when
  size, time, or query-count thresholds fire; if async inserts are used, the
  recommended safe mode is `async_insert=1,wait_for_async_insert=1`, while
  `wait_for_async_insert=0` is risky because clients may miss errors and lose
  backpressure
  ([ClickHouse insert strategy](https://clickhouse.com/docs/best-practices/selecting-an-insert-strategy)).
- ClickHouse release metadata checked on 2026-05-25 shows
  `v26.5.1.882-stable` as the newest feature-stable release and
  `v26.3.12.3-lts` as the newest LTS/latest release. ClickHouse production
  guidance says `stable` is recommended by default while `lts` fits slower
  upgrade policies and simpler secondary workloads
  ([ClickHouse stable release](https://github.com/ClickHouse/ClickHouse/releases/tag/v26.5.1.882-stable),
  [ClickHouse LTS release](https://github.com/ClickHouse/ClickHouse/releases/tag/v26.3.12.3-lts),
  [ClickHouse production version guidance](https://clickhouse.com/docs/faq/operations/production#how-to-choose-between-clickhouse-releases)).
- ClickHouse MergeTree docs say primary-key conditions can trim ranges for fast
  reads, and PREWHERE docs say ClickHouse can automatically move filters into
  PREWHERE to reduce I/O
  ([MergeTree docs](https://clickhouse.com/docs/engines/table-engines/mergetree-family/mergetree),
  [PREWHERE docs](https://clickhouse.com/docs/optimize/prewhere)).

Conclusion: both candidates can plausibly serve Parallax-shaped anchored reads,
but freshness and mixed-load query latency are empirical, version-specific
questions. Public docs cannot answer them.

## What The Smoke Runs Already Proved

The smoke runs in
[Local Benchmark Results](greptimedb-vs-clickhouse/local-benchmark-results.md)
are useful but intentionally narrow:

| Run | Proved | Did not prove |
| --- | --- | --- |
| Run 1: 1M spans | Correctness parity, fixed-overhead latency floors, `trace_id` key-placement sensitivity. | Freshness, mixed-load behavior, cold-cache behavior, object storage, metrics/log/error mix. |
| Run 2: Q1/Q4 joins | Cross-engine correctness and optimizer behavior for anchored trace joins. | Full Q6 bundle latency, concurrent ingest penalty, stale/incomplete bundles. |
| Run 3: metrics | GreptimeDB native PromQL over the metrics table; SQL metric-aggregation parity. | Mixed metrics/log/trace/error ingest, realistic metric compression, Q6 latency while metrics ingest continues. |
| Run 140: four-way 1M local warm | Reproducible four-build matrix, `N >= 50000` enforcement, and 20 query shapes; every 1M warm query is interactive. | Small-tier 25-50 GB, cold/object-store, native ingest, mixed Q6 p95/p99, stale bundles, ClickHouse LTS. |
| Run 141: four-way 5M local warm | Anchored/keyed hot path remains interactive; heavy analytical queries cross or approach the 300 ms gate on GreptimeDB. | Whether full Q6 under mixed native ingest remains under budget; object-store/cold behavior; production hardware. |
| Run 142: GreptimeDB dedup vs append A/B | Dedup-mode aggregation is much slower than append mode at 5M for unique metric-like data; table mode is load-bearing. | Native metric-engine/Prometheus path under v1.1 GA; correctness tradeoff for out-of-order correction workloads. |
| Run 143: benchmark tier policy | Local laptop default is now `N=100000`, with `N=5000000+` reserved for operator-requested server runs; forced compaction reduced stable GreptimeDB dedup aggregation from about 314 ms to about 60 ms. | Server-tier large run, full mixed native ingest, and compaction-state-sensitive metric path under v1.1 GA. |

The useful correction from Run 2 is now part of this gate: Parallax bundle
queries are anchored, so key/index placement and per-anchor pruning matter more
than generic large-to-large join algorithms.

## Measurement Definitions

Use the same timestamp names across all candidates:

| Field | Meaning |
| --- | --- |
| `t_emit` | Synthetic event time stamped by the generator before enqueue/write. |
| `t_ingest_start` | Adapter starts writing the batch containing the event. |
| `t_ingest_ack` | Candidate acknowledges the write to the client. |
| `t_visible` | First point query observes the exact row by stable id. |
| `t_bundle_start` | Q6 bundle request starts for an anchor. |
| `t_bundle_done` | Q6 bundle is assembled, row-count checked, and serialized. |

Primary metrics:

- `freshness = t_visible - t_emit`
- `ack_to_visible = t_visible - t_ingest_ack`
- `bundle_latency = t_bundle_done - t_bundle_start`
- `stale_bundle_rate = bundles missing rows that become visible inside the
  configured freshness window`
- `mixed_penalty = query_p95_mixed / query_p95_query_only`

Report p50, p95, and p99 per signal type and per query class. Do not rely only
on server-reported timings; record client wall-clock timing for every write,
poll, and Q1-Q6 query.

## Workload Shape

Run `smoke` first for correctness, then `small` for the first decision gate:

| Dimension | Small-tier target |
| --- | --- |
| Retained data | 25-50 GB generated candidate input before compression. |
| Duration window | 30 days synthetic wall-clock span. |
| Query workers | At least 4 concurrent workers running Q1-Q6. |
| Ingest workers | Enough workers to maintain target generated write rate without client-side sleeps dominating. |
| Cache modes | warm and cold; cold means candidate restart plus OS page-cache drop where permitted. |
| Signal mix | spans, logs, metrics, error events, deploy markers, frontend events, CLI invocations, and agent actions. |

The query workload is the existing `QueryClass` set from
[Storage benchmark prototype](storage-benchmark-prototype.md):

- Q1 `trace_context`
- Q2 `issue_context`
- Q3 `release_regression`
- Q4 `cross_tier`
- Q5 `high_cardinality`
- Q6 `bundle`

Freshness probes must cover all row families that can enter a bundle:

- spans by `(trace_id, span_id, event_id)`
- logs by `(trace_id, span_id, log_id)`
- error events by `(project, fingerprint, event_id)`
- metric points by `(metric_name, label set, ts, sample_id)`
- deploy markers by `(project, release, deploy_id)`
- frontend events by `(session_id, trace_id, event_id)`
- CLI invocations by `(invocation_id)`
- agent actions by `(agent_session_id, action_id)`

## Candidate-Specific Rules

### GreptimeDB

Run at least two schema modes:

1. **Documented observability baseline:** append-only tables, `trace_id` and
   `request_id` as skipping indexes where relevant, low-cardinality primary-key
   tags, and native PromQL for metrics.
2. **Anchor-optimized variant:** keep tables append-only, but test
   `sst_format='flat'` or a trace-anchor-oriented key/index layout for spans,
   logs, and error events if the baseline cannot meet Q1/Q6 latency.

Measure both SQL ingest for controlled synthetic batches and OTLP ingest for the
real Parallax path. GreptimeDB remains the default only if the real OTLP path
does not add unacceptable visibility delay relative to SQL/bulk ingest.

Run 142 adds a schema requirement: for scrape-style metric tables where
`(series, ts)` uniqueness is guaranteed, test an append-mode variant. Dedup or
`last_non_null` remains valid for partial-upsert and out-of-order correction, but
it cannot be assumed safe for aggregation-heavy metric reads at 5M+ without
fresh v1.1 GA evidence. Run 143 narrows the finding: forced compaction can make
stable dedup aggregation acceptable, but append mode still avoids both compaction
dependence and the nightly dedup regression.

### ClickHouse

Run at least two insert modes:

1. **Synchronous batched insert baseline:** client batches enough rows to avoid
   tiny-part pathology, then records freshness after write ack.
2. **Async insert safe mode:** `async_insert=1,wait_for_async_insert=1`, with
   flush thresholds recorded in the result file.

Run or explicitly exclude two release tracks:

1. **Feature-stable baseline:** latest checked stable feature train, initially
   `v26.5.1.882-stable`.
2. **LTS baseline:** latest checked LTS train, initially `v26.3.12.3-lts`, for
   conservative self-hosted operators that prefer slower upgrade cadence.

Do not accept `wait_for_async_insert=0` as the default Parallax ingest mode even
if it improves client ack latency. It weakens error visibility and backpressure,
which are part of the product contract for debugging evidence.

ClickHouse schemas must put anchor fields in sort/order/index layouts that match
the bundle queries. The existing smoke run already showed that unindexed
`trace_id` makes the comparison meaningless.

## Pass Targets

Use the initial numeric gates from
[Storage benchmark prototype](storage-benchmark-prototype.md) until real small
runs justify calibration:

| Gate | Target |
| --- | --- |
| Freshness p95, mixed load, small tier | <= 5 s ingest-to-queryable |
| Evidence-bundle Q6 p95, warm, small tier | <= 300 ms |
| Trace-context Q1 p95, warm, small tier | <= 100 ms |
| Concurrent penalty | mixed-load query p95 <= 2x query-only p95 |
| Correctness | every anchor returns the same linked row set across candidates after the freshness window |

Additional triage thresholds:

- Freshness p99 above 15 s is a release blocker even if p95 passes.
- Q6 p99 above 1 s requires either a schema/index change or a precomputed bundle
  index before agent workflows rely on it.
- Any nonzero stale bundle rate after the freshness window is a correctness
  failure, not a latency warning.

## Result Record

Every run should write a `results.json` entry with at least:

```json
{
  "candidate": "greptime",
  "candidate_version": "1.0.2",
  "release_track": "ga|feature-stable|lts",
  "mode": "mixed",
  "schema_variant": "append_only_skip_index",
  "insert_mode": "otlp_http",
  "dataset_tier": "small",
  "dataset_seed": 42,
  "ingest_rate_rows_per_second": 0,
  "query_workers": 4,
  "freshness_ms": {
    "spans": {"p50": 0, "p95": 0, "p99": 0},
    "logs": {"p50": 0, "p95": 0, "p99": 0},
    "metrics": {"p50": 0, "p95": 0, "p99": 0},
    "error_events": {"p50": 0, "p95": 0, "p99": 0}
  },
  "queries_ms": {
    "Q1": {"p50": 0, "p95": 0, "p99": 0},
    "Q6": {"p50": 0, "p95": 0, "p99": 0}
  },
  "mixed_penalty": {"Q1": 0.0, "Q6": 0.0},
  "stale_bundle_rate": 0.0,
  "timeouts": 0
}
```

The report table should make the decision visible without reading JSON:

```text
candidate | mode | fresh_p95 | fresh_p99 | Q1_p95 | Q6_p95 | Q6_p99 | mixed_penalty_Q6 | stale_rate | verdict
----------|------|-----------|-----------|--------|--------|--------|------------------|------------|--------
greptime  | OTLP |           |           |        |        |        |                  |            |
clickhouse| sync |           |           |        |        |        |                  |            |
clickhouse| async_wait |     |           |        |        |        |                  |            |
```

## Decision Consequences

Use the result to narrow the architecture:

- If GreptimeDB passes freshness, Q1, Q6, and mixed-penalty gates while staying
  close enough on cost, keep it as the v0.1 default.
- If GreptimeDB fails freshness or Q6 and ClickHouse passes, use ClickHouse as
  the first production storage default and keep GreptimeDB only for metrics or a
  later retry.
- If both candidates pass only with schema-specific precomputation, build a
  `issue_evidence_index` / `bundle_index` worker and stop claiming raw ad hoc
  cross-signal queries are enough.
- If both candidates fail the small-tier freshness gate, narrow the MVP to
  Sentry-compatible errors plus bounded trace/log snippets and defer full
  metrics/log retention.
- If ClickHouse only passes through `wait_for_async_insert=0`, treat that as a
  failed storage default for Parallax's debugging-evidence contract.

## Harness Additions

Add these features to the runnable benchmark before trusting another speed
verdict:

- `load --mode mixed --with-query --freshness-probes`
- per-signal stable ids in generated rows;
- a polling loop that records first visibility without forcing flushes;
- Q6 bundle correctness checks against the generated manifest;
- separate `query-only`, `write-only`, and `mixed` phases;
- result fields for candidate settings, schema variant, insert mode, flush
  thresholds, and client-side timing source;
- explicit stale-bundle detection by re-querying missing refs after the
  freshness window.

## Related Research

- [Storage benchmark prototype](storage-benchmark-prototype.md)
- [Storage benchmark artifact interpretation](storage-benchmark-artifact-interpretation.md)
- [Observability storage benchmark plan](observability-storage-benchmark-plan.md)
- [GreptimeDB storage evaluation](greptimedb-storage-evaluation.md)
- [GreptimeDB vs ClickHouse local benchmark results](greptimedb-vs-clickhouse/local-benchmark-results.md)
- [A5 stack decision ledger](a5-stack-decision-ledger.md) consumes this gate's
  freshness and Q6 latency rows before any storage result can become a stack
  default.
- [Evidence bundle and open schema specification](evidence-bundle-and-schema.md)
