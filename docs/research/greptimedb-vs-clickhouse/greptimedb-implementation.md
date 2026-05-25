# Parallax on GreptimeDB — Concrete Implementation Design

<!-- markdownlint-disable MD013 -->

Status: pass 12. The buildable storage design for the **recommended** engine
(`verdict-which-to-choose.md`): full schema, ingest path, exact retrieval, object
storage, and operational shape — for the whole Parallax signal set. Builds on the
seed DDL in `storage-benchmark-prototype.md` and applies the empirical lessons
from `local-benchmark-results.md` (Runs 1–5). DDL syntax verified against the
pinned source.

Pin: GreptimeDB `v1.0.2` (`0ef5451`). DDL features confirmed in
`src/sql/src/parsers/create_parser.rs` (`INVERTED`/`FULLTEXT`/`SKIPPING INDEX`),
`src/store-api/src/region_request.rs` (`append_mode`, `ttl`), and
`src/store-api/src/metric_engine_consts.rs` (`physical_metric_table`,
`on_physical_table`).

## Design principles (each justified from internals)

1. **`trace_id` gets an `INVERTED INDEX` on every signal it anchors** (spans, logs,
   error_events, frontend_events, agent_actions). This is the **direct fix for the
   Run-1 finding** (trace lookup 16 ms un-indexed vs 2 ms keyed): the inverted
   index gives point-lookup on `trace_id` **without** making it a primary-key tag
   (which would explode series cardinality, ~71k+ traces → tiny series). Idiomatic
   GreptimeDB high-cardinality-lookup pattern.
2. **Primary key = low-cardinality query tags** (e.g. `service`, `project`,
   `fingerprint`) → bounded series count, efficient `TimeSeries`/`PartitionTree`
   memtable; high-cardinality lookups go through indexes, not the PK.
3. **`append_mode = 'true'`** on all append-only signals (events/spans/logs) →
   skips the dedup/last-row merge (`greptimedb-internals.md`), lowering write/read
   cost; matches GreptimeDB's own event-table default.
4. **`FULLTEXT INDEX`** on free-text (`message`) for log/error search.
5. **Metrics via the metric engine** (logical→physical) + OTLP/Prom remote write →
   native PromQL (Run 3 capability win).
6. **Dynamic OTLP attributes → `JSON` column**; promote a hot attribute to a tag or
   `SKIPPING INDEX` only when a query needs it (Q5).
7. **`ttl` per table** + object storage for cheap re-readable retention.

## Schema (real DDL)

```sql
-- 1. Spans (anchored by trace_id; append-only)
CREATE TABLE spans (
  ts             TIMESTAMP TIME INDEX,
  trace_id       STRING INVERTED INDEX,        -- Q1/Q4 point lookup (Run-1 fix)
  span_id        STRING,
  parent_span_id STRING,
  service        STRING,
  name           STRING,
  duration_ms    DOUBLE,
  status         STRING,
  attributes     JSON,                          -- dynamic OTLP span attrs
  PRIMARY KEY (service, name)                    -- low-card series tags
) WITH (append_mode = 'true', ttl = '30d');

-- 2. Logs (full-text + trace anchor; append-only)
CREATE TABLE logs (
  ts        TIMESTAMP TIME INDEX,
  service   STRING,
  level     STRING,
  message   STRING FULLTEXT INDEX WITH (analyzer = 'English', case_sensitive = 'false'),
  trace_id  STRING INVERTED INDEX,
  span_id   STRING,
  attributes JSON,
  PRIMARY KEY (service, level)
) WITH (append_mode = 'true', ttl = '30d');

-- 3. Error events (issue/fingerprint history + trace anchor)
CREATE TABLE error_events (
  ts            TIMESTAMP TIME INDEX,
  project       STRING,
  environment   STRING,
  release       STRING,
  fingerprint   STRING,
  error_type    STRING,
  message       STRING FULLTEXT INDEX WITH (analyzer = 'English'),
  trace_id      STRING INVERTED INDEX,
  span_id       STRING,
  panic_location STRING,
  handled       BOOLEAN,
  attributes    JSON,
  PRIMARY KEY (project, fingerprint)             -- Q2/Q3 by (project,fingerprint)
) WITH (append_mode = 'true', ttl = '90d');

-- 4. Metrics — physical wide table under the metric engine; logical tables
--    auto-created by OTLP / Prometheus remote write (one logical table per metric).
CREATE TABLE greptime_physical_metrics (
  ts        TIMESTAMP TIME INDEX,
  greptime_value DOUBLE,
  PRIMARY KEY ()
) ENGINE = metric WITH (physical_metric_table = '');
-- logical metric tables are created on ingest:
--   ... ENGINE = metric WITH (on_physical_table = 'greptime_physical_metrics');

-- 5. Deploy markers (low volume; release timeline)
CREATE TABLE deploy_markers (
  ts          TIMESTAMP TIME INDEX,
  project     STRING,
  environment STRING,
  release     STRING,
  commit_sha  STRING,
  attributes  JSON,
  PRIMARY KEY (project, environment)
) WITH (append_mode = 'true', ttl = '365d');

-- 6. CLI invocations
CREATE TABLE cli_invocations (
  ts         TIMESTAMP TIME INDEX,
  session_id STRING INVERTED INDEX,
  user_id    STRING SKIPPING INDEX,             -- Q5 high-card filter
  command    STRING,
  exit_code  INT,
  duration_ms DOUBLE,
  trace_id   STRING INVERTED INDEX,
  attributes JSON,
  PRIMARY KEY (command)
) WITH (append_mode = 'true', ttl = '90d');

-- 7. Agent actions
CREATE TABLE agent_actions (
  ts          TIMESTAMP TIME INDEX,
  session_id  STRING INVERTED INDEX,
  action_type STRING,
  trace_id    STRING INVERTED INDEX,
  tool        STRING,
  attributes  JSON,
  PRIMARY KEY (action_type)
) WITH (append_mode = 'true', ttl = '90d');

-- 8. Frontend events (cross-tier: trace_id propagates into backend spans)
CREATE TABLE frontend_events (
  ts          TIMESTAMP TIME INDEX,
  app         STRING,
  session_id  STRING INVERTED INDEX,
  event_type  STRING,
  trace_id    STRING INVERTED INDEX,            -- Q4 cross-tier join key
  url         STRING,
  user_id     STRING SKIPPING INDEX,
  attributes  JSON,
  PRIMARY KEY (app, event_type)
) WITH (append_mode = 'true', ttl = '30d');
```

## Indexing strategy

| Column | Index | Serves | Mechanism |
| --- | --- | --- | --- |
| `trace_id` (spans/logs/errors/frontend/agent) | `INVERTED INDEX` | Q1, Q4 anchored lookup | point lookup without series explosion (Run-1 fix) |
| `(project, fingerprint)` (errors) | PRIMARY KEY tags | Q2, Q3 | series key = sort prefix |
| `message` (logs/errors) | `FULLTEXT INDEX` | log/error search | tokenized inverted (Puffin) |
| `user_id` (cli/frontend) | `SKIPPING INDEX` | Q5 high-card filter | min/max-style granule skip |
| `ts` (all) | TIME INDEX | every time-window query | file/row-group time pruning |

## Ingest path

| Signal | Path |
| --- | --- |
| Metrics | **Prometheus remote-write** or **OTLP metrics** → metric engine auto-creates logical tables. Native, no translation. |
| Spans / logs / errors | **OTLP** (traces/logs) → mapped to the tables above; or gRPC/SQL insert. |
| Deploy / CLI / agent / frontend | SQL insert / gRPC from the Parallax collectors. |

Freshness: visible-on-write via `committed_sequence` (Run 5) — no flush barrier.
Small high-frequency writes absorbed by the memtable (no part-explosion;
`write-path-and-ingestion.md`).

## Retrieval (Q1–Q6, GreptimeDB dialect)

```sql
-- Q1 trace_context: spans + logs + error for a trace (INVERTED INDEX on trace_id)
SELECT 'span' AS kind, span_id, name, duration_ms, status, NULL AS message
  FROM spans WHERE trace_id = ?
UNION ALL SELECT 'log', span_id, NULL, NULL, level, message
  FROM logs WHERE trace_id = ?
UNION ALL SELECT 'error', span_id, error_type, NULL, NULL, message
  FROM error_events WHERE trace_id = ?;

-- Q2 issue_context: PK (project,fingerprint) range
SELECT min(ts) first_seen, max(ts) last_seen, count(*) n
  FROM error_events WHERE project = ? AND fingerprint = ?;

-- Q3 release_regression: fingerprints in R absent in prior release
SELECT DISTINCT fingerprint FROM error_events
 WHERE project = ? AND release = ?
   AND fingerprint NOT IN (
     SELECT fingerprint FROM error_events WHERE project = ? AND release = ?);

-- Q4 cross_tier: frontend trace -> backend spans + errors (trace_id INVERTED INDEX)
SELECT s.service, s.name, s.duration_ms, e.error_type, e.message
  FROM spans s LEFT JOIN error_events e
    ON e.trace_id = s.trace_id AND e.span_id = s.span_id
 WHERE s.trace_id = ? ORDER BY s.ts;

-- Q5 high_cardinality: filter by user over window (SKIPPING INDEX / JSON attr)
SELECT count(*) FROM spans
 WHERE ts >= ? AND ts < ? AND json_get_string(attributes, 'user') = ?;

-- Q6 bundle = Q1 + Q2 + Q3 for the anchor, assembled client-side.

-- Metrics (native PromQL, not SQL):
--   GET /v1/prometheus/api/v1/query_range?query=avg by (service)(http_req_latency)&start=&end=&step=
```

Key/index usage: Q1/Q4 use the `trace_id` inverted index (point lookup, Run-2
plan confirmed filter pushdown into the region scan); Q2/Q3 use the
`(project,fingerprint)` PK prefix; Q5 uses the `SKIPPING INDEX` or falls back to a
JSON-attribute scan over the time window; metrics use the native PromQL planner.

## Object storage and retention

```toml
# datanode config — object storage native (OpenDAL)
[storage]
type = "S3"          # or "Oss" / "Gcs"
bucket = "parallax-telemetry"
root   = "greptimedb"
cache_path = "/var/cache/greptimedb"   # local read cache in front of S3 (default on)
```

- Hot data served from the local read cache; cold re-reads hit S3 (cost in
  `retention-cost-model.md`).
- Per-table `ttl` expires old data; longer `ttl` on `error_events`/`deploy_markers`
  (issue history) than on high-volume `spans`/`logs`.
- Time-window compaction (TWCS) keeps recent windows tight for fast recent-time
  queries.

## Operational shape

- **Tier-1 (startup):** one `greptime standalone` binary + an object-store bucket.
  Metrics-native, OTLP-native; no Kafka, no coordinator. Smallest viable footprint.
- **Scale-out (same schema):** Frontend (stateless, scale freely) + Datanodes
  (hold regions) + Metasrv (placement) + optional Kafka remote WAL. Tables
  partition into regions; Metasrv rebalances; **no schema change** — topology
  change, not rewrite (`distributed-and-scaling.md`).

## Build-validation status

Runs 1–5 already exercised the `spans`/`logs`/`error_events`/metrics schemas and
Q1/Q4 on a live GreptimeDB `v1.0.2` with cross-engine parity. **Not yet built:**
the `INVERTED INDEX`-on-`trace_id` variant (the Run-1 fix) — the next Docker run
should create spans with `trace_id INVERTED INDEX` and re-measure the trace lookup
(expect it to close the 16 ms→~2 ms gap). Routed to `benchmarking-the-differences.md`.

## Side-by-side

The ClickHouse counterpart is `clickhouse-implementation.md` (next pass); the two
are kept structurally parallel so the design differences — native metric engine +
inverted-index lookups + object-store-native vs MergeTree `ORDER BY` + skip indexes
+ S3-disk-policy — are directly comparable.
