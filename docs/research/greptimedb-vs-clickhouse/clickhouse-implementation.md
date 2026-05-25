# Parallax on ClickHouse — Concrete Implementation Design

<!-- markdownlint-disable MD013 -->

Status: pass 13 (design) + pass 74 (**whole schema built live**, Run 46). The buildable
storage design for the **rejected-but-viable** alternative (`verdict-which-to-choose.md`):
full schema, ingest path, exact retrieval, object storage, operational shape — kept
structurally parallel to `greptimedb-implementation.md` so the differences are directly
comparable. Builds on the seed DDL in `storage-benchmark-prototype.md` and the Docker runs
(`local-benchmark-results.md`). **DDL executed on live ClickHouse `v26.5.1.882` (Run 46):
all 7 tables + rollup MV build; one fix applied (`text` tokenizer `'default'`→`splitByNonAlpha`).**

Pin: ClickHouse `v26.5.1.882-stable` (`5b96a8d8`). Confirmed against source:
skip-index types `minmax/set/tokenbf_v1/ngrambf_v1/sparse_grams/bloom_filter/text/
vector_similarity` (`src/Storages/MergeTree/MergeTreeIndices.cpp:172-195`),
`async_insert` **default `true`** (`src/Core/Settings.cpp:6365`), `JSON` type now
**stable** (experimental flag obsoleted, `Settings.cpp:8217`).

## Design principles (justified from internals)

1. **`ORDER BY` is the only "index" that prunes cheaply** — put the dominant query
   key first. Spans key on `trace_id` (Run-2 plan: anchored lookup → `Granules: 1`).
   Tables not keyed on `trace_id` get a **`bloom_filter` skip index** on it so Q1/Q4
   anchored lookups don't full-scan.
2. **Per-column codecs**, matched to data: `DoubleDelta`+`ZSTD` for timestamps,
   `Gorilla`+`ZSTD` for float gauges, `LowCardinality` for service/level/status,
   `ZSTD` for free text (Run 4 showed counter 7.3×, gauge 78×).
3. **Native `text` index** on `message` for log/error substring search (the new
   inverted index; `tokenbf_v1` as the conservative fallback). **Tokenizer must be a
   real name** — valid in 26.5.1 are `splitByNonAlpha` (word search, used here),
   `splitByString`, `array`; `'default'`/`'standard'`/`'ngram'` are **rejected** (Run 46).
4. **`JSON` type** (now stable) for dynamic OTLP attributes.
5. **Metrics: `AggregatingMergeTree` + materialized view** for rollups — there is
   **no PromQL**, so a PromQL→SQL translation layer is required in front (the cost).
6. **`async_insert` (default on in 26.x)** batches small writes server-side →
   avoids part-explosion; visibility delayed ~50–200 ms.
7. **TTL move to an S3 disk** for hot-local/cold-object tiering.

## Schema (real DDL)

```sql
-- 1. Spans — trace_id as ORDER BY prefix (fast anchored lookup, Run-2)
CREATE TABLE spans (
  ts DateTime64(3) CODEC(DoubleDelta, ZSTD),
  trace_id String, span_id String, parent_span_id String,
  service LowCardinality(String), name LowCardinality(String),
  duration_ms Float64 CODEC(Gorilla, ZSTD),
  status LowCardinality(String),
  attributes JSON
) ENGINE = MergeTree
ORDER BY (trace_id, ts)
PARTITION BY toYYYYMMDD(ts)              -- align parts to a day so TTL drops whole parts
TTL toDateTime(ts) + INTERVAL 30 DAY
SETTINGS ttl_only_drop_parts = 1;        -- whole-part drop, no row-level rewrite (see retention-and-ttl.md)

-- 2. Logs — service-ordered; text index for search; bloom for trace lookup
CREATE TABLE logs (
  ts DateTime64(3) CODEC(DoubleDelta, ZSTD),
  service LowCardinality(String), level LowCardinality(String),
  message String CODEC(ZSTD),
  trace_id String, span_id String,
  attributes JSON,
  INDEX idx_msg   message  TYPE text(tokenizer = 'splitByNonAlpha') GRANULARITY 1,
  INDEX idx_trace trace_id TYPE bloom_filter GRANULARITY 1
) ENGINE = MergeTree
ORDER BY (service, ts)
TTL toDateTime(ts) + INTERVAL 30 DAY;

-- 3. Error events — (project,fingerprint) prefix for Q2/Q3
CREATE TABLE error_events (
  ts DateTime64(3) CODEC(DoubleDelta, ZSTD),
  project LowCardinality(String), environment LowCardinality(String),
  release LowCardinality(String), fingerprint String,
  error_type LowCardinality(String),
  message String CODEC(ZSTD),
  trace_id String, span_id String,
  panic_location String, handled UInt8,
  attributes JSON,
  INDEX idx_trace trace_id TYPE bloom_filter GRANULARITY 1,
  INDEX idx_msg   message  TYPE text(tokenizer = 'splitByNonAlpha') GRANULARITY 1
) ENGINE = MergeTree
ORDER BY (project, fingerprint, ts)
TTL toDateTime(ts) + INTERVAL 90 DAY;

-- 4. Metrics — raw + a rollup via AggregatingMergeTree + MV. NO PromQL.
CREATE TABLE metrics_raw (
  ts DateTime64(3) CODEC(DoubleDelta, ZSTD),
  metric LowCardinality(String),
  labels_hash UInt64,
  labels JSON,
  value Float64 CODEC(Gorilla, ZSTD)
) ENGINE = MergeTree
ORDER BY (metric, labels_hash, ts)
TTL toDateTime(ts) + INTERVAL 15 DAY;

CREATE TABLE metrics_5m (
  ts DateTime CODEC(DoubleDelta, ZSTD), metric LowCardinality(String),
  labels_hash UInt64,
  avg_state AggregateFunction(avg, Float64),
  max_state AggregateFunction(max, Float64)
) ENGINE = AggregatingMergeTree
ORDER BY (metric, labels_hash, ts)
TTL ts + INTERVAL 400 DAY;

CREATE MATERIALIZED VIEW metrics_5m_mv TO metrics_5m AS
SELECT toStartOfInterval(ts, INTERVAL 5 MINUTE) ts, metric, labels_hash,
       avgState(value) avg_state, maxState(value) max_state
FROM metrics_raw GROUP BY ts, metric, labels_hash;

-- 5-8. deploy_markers / cli_invocations / agent_actions / frontend_events
CREATE TABLE frontend_events (
  ts DateTime64(3) CODEC(DoubleDelta, ZSTD),
  app LowCardinality(String), session_id String,
  event_type LowCardinality(String),
  trace_id String, url String, user_id String,
  attributes JSON,
  INDEX idx_trace trace_id TYPE bloom_filter GRANULARITY 1,
  INDEX idx_user  user_id  TYPE bloom_filter GRANULARITY 1
) ENGINE = MergeTree
ORDER BY (app, event_type, ts)
TTL toDateTime(ts) + INTERVAL 30 DAY;
-- deploy_markers ORDER BY (project, environment, ts);
-- cli_invocations ORDER BY (command, ts) + bloom on session_id/trace_id;
-- agent_actions   ORDER BY (action_type, ts) + bloom on session_id/trace_id.
```

## Indexing strategy

| Column | Mechanism | Serves |
| --- | --- | --- |
| `trace_id` (spans) | `ORDER BY` prefix → sparse primary index | Q1/Q4 (1-granule seek, Run-2) |
| `trace_id` (logs/errors/frontend/…) | `bloom_filter` skip index | Q1/Q4 when not the sort prefix |
| `(project, fingerprint)` (errors) | `ORDER BY` prefix | Q2/Q3 |
| `message` | native `text` index (or `tokenbf_v1`) | log/error search |
| `user_id` | `bloom_filter` skip index | Q5 high-card filter |
| `ts` | part `PARTITION BY` (e.g. day) + sort key | time pruning |

## Ingest path

| Signal | Path |
| --- | --- |
| Metrics | **No native PromQL / Prom remote-write** → run an OTLP/Prometheus **collector/exporter** that writes `metrics_raw`; the MV builds `metrics_5m`. A **PromQL→SQL layer** is required to serve PromQL. |
| Spans / logs / errors | **OTLP collector → ClickHouse exporter** (no native OTLP ingest). |
| Deploy / CLI / agent / frontend | INSERT from collectors; `async_insert` (default) batches small writes. |

Freshness: synchronous insert is visible-on-write (Run 5); `async_insert` (default
on) delays small-write visibility ~50–200 ms in exchange for avoiding
part-explosion.

## Retrieval (Q1–Q6, ClickHouse dialect)

```sql
-- Q1 trace_context (trace_id = ORDER BY prefix on spans; bloom on logs/errors)
SELECT 'span' AS kind, span_id, name, duration_ms, status, NULL AS message
  FROM spans WHERE trace_id = ?
UNION ALL SELECT 'log', span_id, NULL, NULL, level, message FROM logs WHERE trace_id = ?
UNION ALL SELECT 'error', span_id, error_type, NULL, NULL, message FROM error_events WHERE trace_id = ?;

-- Q2 issue_context
SELECT min(ts) first_seen, max(ts) last_seen, count() n
  FROM error_events WHERE project = ? AND fingerprint = ?;

-- Q3 release_regression
SELECT DISTINCT fingerprint FROM error_events
 WHERE project = ? AND release = ?
   AND fingerprint NOT IN (SELECT fingerprint FROM error_events WHERE project = ? AND release = ?);

-- Q4 cross_tier (Run-2: SpillingHashJoin, anchor propagated to both sides)
SELECT s.service, s.name, s.duration_ms, e.error_type, e.message
  FROM spans s LEFT JOIN error_events e ON e.trace_id = s.trace_id AND e.span_id = s.span_id
 WHERE s.trace_id = ? ORDER BY s.ts;

-- Q5 high_cardinality (bloom_filter on user_id, or JSON path)
SELECT count() FROM spans
 WHERE ts >= ? AND ts < ? AND attributes.user = ?;

-- Q6 bundle = Q1+Q2+Q3 assembled client-side.

-- Metrics range-agg (SQL; PromQL must be translated to this shape):
SELECT metric, toStartOfInterval(ts, INTERVAL 5 MINUTE) t, avgMerge(avg_state)
  FROM metrics_5m WHERE metric = ? AND ts BETWEEN ? AND ? GROUP BY metric, t ORDER BY t;
```

## Object storage and retention

```xml
<!-- storage policy: hot local volume -> cold S3 volume -->
<storage_configuration>
  <disks>
    <s3><type>s3</type><endpoint>https://…/parallax/</endpoint></s3>
  </disks>
  <policies><hot_cold><volumes>
    <hot><disk>default</disk></hot>
    <cold><disk>s3</disk></cold>
  </volumes></hot_cold></policies>
</storage_configuration>
```

```sql
ALTER TABLE spans MODIFY TTL
  toDateTime(ts) + INTERVAL 2 DAY TO VOLUME 'cold',
  toDateTime(ts) + INTERVAL 30 DAY DELETE
SETTINGS storage_policy = 'hot_cold';
```

- Hot data on local NVMe; cold tiered to S3 by TTL move. (Zero-copy replication is
  **off by default** and fragile — leave off.)
- This is more configuration than GreptimeDB's object-store-native default
  (`compression-and-cost.md`), and S3 is a *disk under a policy*, not the home.
- **Retention must be partition-aligned.** Every table above needs `PARTITION BY
  toYYYYMMDD(ts)` (coarser — `toYYYYMM(ts)` — for the 400d `AggregatingMergeTree`
  rollup) **plus** `SETTINGS ttl_only_drop_parts = 1`, so TTL DELETE drops whole parts
  instead of rewriting surviving rows on every TTL merge. Without it ClickHouse defaults
  to **row-level** expiry (write-amp ∝ surviving data). GreptimeDB needs no equivalent
  tuning — TWCS time-windows make expiry a whole-SST drop by default. Mechanism +
  source in `retention-and-ttl.md`.

## Operational shape

- **Tier-1 (startup):** one `clickhouse-server` + plain MergeTree. Smallest
  footprint; but the metrics path needs an external OTLP collector + PromQL layer
  to be useful for Parallax.
- **Scale-out:** `Distributed` engine over **manually-defined shards** +
  `ReplicatedMergeTree` + Keeper. Sharding key + shard count chosen **up front**;
  no automatic resharding in OSS (`distributed-and-scaling.md`) — growing past the
  initial layout is a manual data-move.

## What ClickHouse needs that GreptimeDB does not (the replaceability cost)

1. **OTLP collector + PromQL→SQL layer** — metrics/traces/logs do not land
   natively; build and operate the pipeline.
2. **Explicit skip indexes** for `trace_id` on every non-prefix table.
3. **Sharding decided up front** + manual resharding to grow.
4. **`async_insert`/batching** discipline for small writes (default-on mitigates).

In exchange you get the faster log/trace scan engine, the higher vertical ceiling,
and the broader hand-tunable codec set. The verdict weighs this and still chooses
GreptimeDB on *fit*; this note proves ClickHouse is a fully buildable fallback.

## Build-validation status

**Whole schema built live on ClickHouse `v26.5.1.882-stable` (Run 46),** parallel to the
GreptimeDB build (Run 45). All 7 tables + the rollup MV created clean in a scratch database;
`JSON` builds bare (stable, no experimental flag), `CODEC(DoubleDelta/Gorilla, ZSTD)`,
`LowCardinality`, `bloom_filter` skip indexes, `ttl_only_drop_parts`, the
`AggregatingMergeTree` + `avgState/maxState` MV, and JSON-path access (`attributes.user`) all
accepted. **One real defect caught:** the `text` index `tokenizer = 'default'` is invalid →
`Unknown tokenizer: 'default'`; fixed to `splitByNonAlpha` (valid set: `splitByNonAlpha`,
`splitByString`, `array`). This is a much smaller drift than the GreptimeDB side (Run 45: 7
reserved-keyword columns + the metric-table PK) — ClickHouse's DDL was nearly correct as
written. S3-disk tiering (the `<storage_configuration>` + `TTL … TO VOLUME`) still needs a
MinIO-backed run — routed to `benchmarking-the-differences.md`.
