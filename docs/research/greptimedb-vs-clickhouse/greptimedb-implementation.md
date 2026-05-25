# Parallax on GreptimeDB — Concrete Implementation Design

<!-- markdownlint-disable MD013 -->

Status: pass 12 (design) + pass 73 (**whole schema built live**, Run 45) + pass 94
(**native auto-schema vs custom decided live**, Run 57) + pass 116 (**Q4 retrieval
corrected for the join-pushdown gap**, Runs 81/82 — direct in-DB join full-scans; use
subquery pre-filter or app-side assembly). The buildable
storage design for the **recommended** engine (`verdict-which-to-choose.md`): full schema,
ingest path, exact retrieval, object storage, and operational shape — for the whole Parallax
signal set. Builds on the seed DDL in `storage-benchmark-prototype.md` and applies the
empirical lessons from `local-benchmark-results.md` (Runs 1–5). **The DDL below was executed
against a live GreptimeDB `v1.0.2` (Run 45): all 9 tables (8 signals + a logical metric
table) build, with the `trace_id INVERTED INDEX` and `message FULLTEXT INDEX` confirmed
attached via `SHOW CREATE TABLE`.** Two corrections vs the original syntax-only design were
caught and applied — see the note above the DDL (reserved-keyword quoting) and table 4 (no
empty `PRIMARY KEY ()`).

Pin: GreptimeDB `v1.0.2` (`0ef5451`). DDL features confirmed in
`src/sql/src/parsers/create_parser.rs` (`INVERTED`/`FULLTEXT`/`SKIPPING INDEX`),
`src/store-api/src/region_request.rs` (`append_mode`, `ttl`), and
`src/store-api/src/metric_engine_consts.rs` (`physical_metric_table`,
`on_physical_table`) — and now end-to-end live (Run 45).

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

## Native out-of-the-box schema vs this custom design (adopt-vs-custom — Run 57, live)

Before any custom DDL: what does GreptimeDB auto-create with **zero schema work**
when a standard client just sends telemetry? Verified live (Run 57) by hitting the
native ingest endpoints on a clean table and reading `SHOW CREATE TABLE`:

| Signal | Native ingest (zero DDL) | Auto-created structure (live) | Adopt or customize? |
| --- | --- | --- | --- |
| **Metrics** | InfluxDB line `POST /v1/influxdb/write` (HTTP 204); also Prom remote-write / OTLP | tags → `PRIMARY KEY` (e.g. `(service, env)`); fields **auto-typed** (`count`→`BIGINT`, `latency_ms`→`DOUBLE`); auto `greptime_timestamp TIMESTAMP(9) TIME INDEX`; `merge_mode='last_non_null'` (partial-upsert last-non-null per series+ts); **one table per measurement** | **ADOPT.** This *is* a correct metric table — tags-as-PK bounds series, PromQL runs on it, last-non-null gives upsert. No custom DDL needed unless a specific PK order is wanted. |
| **Logs** | `greptime_identity` pipeline `POST /v1/ingest?…&pipeline_name=greptime_identity` (HTTP 200), JSON body | auto `greptime_timestamp TIMESTAMP(9) TIME INDEX`; **every JSON key → `STRING` column** (`level`, `message`, `service`, `trace_id`, `span_id`); `append_mode='true'`; **NO `PRIMARY KEY`, NO index on `trace_id`/`message`** (flat append) | **ADOPT-then-CUSTOMIZE.** Append-mode + auto-timestamp are right; the **one shortfall is the missing anchor index** — a `trace_id` log lookup on the native table **scans** (no index). Parallax adds `trace_id INVERTED INDEX` (+ `message FULLTEXT`) — exactly the deviation principle 1/4 already specify. Run 56 showed `trace_id` retrieval is the evidence-bundle's dominant cost, so this index is load-bearing, not optional. |
| **Traces** | OTLP `POST /v1/otlp/v1/traces` — **protobuf only** | not hand-verifiable: JSON **rejected live** (HTTP 400 `"OTLP endpoint only supports 'application/x-protobuf'"`, Run 57) — needs a real OTLP exporter/collector. Native table is `opentelemetry_traces` (per docs), flattening spans | **OWED + likely CUSTOMIZE.** Whether the native `opentelemetry_traces` indexes `trace_id` for the anchored lookup is **unverified** (protobuf blocker — route a collector-fed check to the harness). Parallax's custom `spans` table indexes `trace_id` explicitly regardless. |

**Decision:** GreptimeDB's adopt-native path is **genuinely usable with zero DDL for
metrics** and **near-usable for logs** — the *only* forced deviation is adding the
`trace_id`/`message` indexes the native log schema omits, which Parallax's anchored hot
path requires. That is a small, well-motivated customization (name the shortfall: *no
anchor index on the native append table*), not a wholesale redesign. **ClickHouse has no
native-ingest equivalent** (no OTLP/Influx receiver — re-confirmed: only GreptimeDB
accepted the live native writes here; ClickHouse needs an OTel Collector + ClickHouse
exporter, i.e. the "ClickStack" defaults, or a hand-defined schema). So there is **no
"zero schema work" path on ClickHouse** — its native-vs-custom question is always
"adopt the collector-exporter schema vs define your own", never "use what the DB
auto-makes". This is a real ergonomics edge for GreptimeDB on the ingest/onboarding axis.

## Schema (real DDL)

```sql
-- NOTE: GreptimeDB v1.0.2 reserves a BROAD keyword set — Run 85 found ~28/42 common
-- observability column names rejected unquoted: id, value, timestamp, user, name, status,
-- level, message, service, release, url, method, count, type, source, target, date, start,
-- end, key, index, group, order, table, version, event, action, result (ClickHouse rejects
-- only `index` of these). RULE: **quote every column identifier** ("col") in GreptimeDB DDL
-- — it is the safe default given how many observability names are reserved. (Not reserved:
-- host, duration, environment, project, fingerprint, error_type, span_id, trace_id,
-- attributes, time.) Bench tables carry a bare `service` only because the ingest protocols
-- (OTLP/InfluxDB line) auto-create schema outside the SQL parser. Verified live (Runs 45, 85).

-- 1. Spans (anchored by trace_id; append-only)
CREATE TABLE spans (
  ts             TIMESTAMP TIME INDEX,
  trace_id       STRING INVERTED INDEX,        -- Q1/Q4 point lookup (Run-1 fix)
  span_id        STRING,
  parent_span_id STRING,
  "service"      STRING,
  "name"         STRING,
  duration_ms    DOUBLE,
  "status"       STRING,
  attributes     JSON,                          -- dynamic OTLP span attrs
  PRIMARY KEY ("service", "name")                -- low-card series tags
) WITH (append_mode = 'true', ttl = '30d');

-- 2. Logs (full-text + trace anchor; append-only)
CREATE TABLE logs (
  ts        TIMESTAMP TIME INDEX,
  "service" STRING,
  "level"   STRING,
  "message" STRING FULLTEXT INDEX WITH (analyzer = 'English', case_sensitive = 'false'),
  trace_id  STRING INVERTED INDEX,
  span_id   STRING,
  attributes JSON,
  PRIMARY KEY ("service", "level")
) WITH (append_mode = 'true', ttl = '30d');

-- 3. Error events (issue/fingerprint history + trace anchor)
CREATE TABLE error_events (
  ts            TIMESTAMP TIME INDEX,
  project       STRING,
  environment   STRING,
  "release"     STRING,
  fingerprint   STRING,
  error_type    STRING,
  "message"     STRING FULLTEXT INDEX WITH (analyzer = 'English'),
  trace_id      STRING INVERTED INDEX,
  span_id       STRING,
  panic_location STRING,
  handled       BOOLEAN,
  attributes    JSON,
  PRIMARY KEY (project, fingerprint)             -- Q2/Q3 by (project,fingerprint)
) WITH (append_mode = 'true', ttl = '90d');

-- 4. Metrics — physical wide table under the metric engine; logical tables
--    auto-created by OTLP / Prometheus remote write (one logical table per metric).
--    NOTE: no empty PRIMARY KEY () — invalid syntax in v1.0.2 (Run 45); omit it.
CREATE TABLE greptime_physical_metrics (
  ts             TIMESTAMP TIME INDEX,
  greptime_value DOUBLE
) ENGINE = metric WITH ("physical_metric_table" = '');
-- logical metric tables are created on ingest (or explicitly):
--   CREATE TABLE http_req_latency (ts TIMESTAMP TIME INDEX, greptime_value DOUBLE,
--     "service" STRING PRIMARY KEY) ENGINE = metric
--     WITH (on_physical_table = 'greptime_physical_metrics');

-- 5. Deploy markers (low volume; release timeline)
CREATE TABLE deploy_markers (
  ts          TIMESTAMP TIME INDEX,
  project     STRING,
  environment STRING,
  "release"   STRING,
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
  "url"       STRING,
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

-- Q4 cross_tier: frontend trace -> backend spans + errors.
-- IMPORTANT (Run 81/82): a *direct* `spans s LEFT JOIN error_events e WHERE s.trace_id=?`
-- does NOT push the anchor into the join-input scan on GreptimeDB v1.0.2 — it FULL-SCANS
-- spans (~54 ms / 1M rows) because the inverted index is not consulted for a join input.
-- Fix: pre-filter the anchored side in a subquery so the index prunes (~21 ms), OR assemble
-- app-side like Q1/Q6 (anchored fetch each signal + join in app — Parallax's pattern, fastest).
SELECT s.service, s.name, s.duration_ms, e.error_type, e.message
  FROM (SELECT * FROM spans WHERE trace_id = ?) s
  LEFT JOIN error_events e ON e.trace_id = s.trace_id AND e.span_id = s.span_id
 ORDER BY s.ts;

-- Q5 high_cardinality: filter by user over window (SKIPPING INDEX / JSON attr)
SELECT count(*) FROM spans
 WHERE ts >= ? AND ts < ? AND json_get_string(attributes, 'user') = ?;

-- Q6 bundle = Q1 + Q2 + Q3 for the anchor, assembled client-side.

-- Metrics (native PromQL, not SQL):
--   GET /v1/prometheus/api/v1/query_range?query=avg by (service)(http_req_latency)&start=&end=&step=
```

Key/index usage: **Q1** uses the `trace_id` inverted index (the UNION's per-table
`WHERE trace_id=?` each prune — point lookup). **Q4 needs the subquery-prefilter form
above** (or app-side assembly): a *direct* join does **not** push the anchor into the
join-input scan, so the index isn't used and spans is full-scanned (Run 81/82 — corrects
the earlier "Q4 uses the index" claim; the standalone `WHERE` prunes, the join input does
not). Q2/Q3 use the `(project,fingerprint)` PK prefix; Q5 uses the `SKIPPING INDEX` or
falls back to a JSON-attribute scan over the time window; metrics use the native PromQL
planner. **Design rule for Parallax retrieval: anchor each signal with its own
`WHERE trace_id=?` (index-pruned) and join app-side — never a direct in-DB join on an
indexed anchor.**

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
  `compression-and-cost.md` / `caching-and-cold-warm.md`).
- Per-table `ttl` expires old data; longer `ttl` on `error_events`/`deploy_markers`
  (issue history) than on high-volume `spans`/`logs`. TTL expiry is a **whole-SST
  drop** (TWCS time-windowing → no rewrite); mechanism + ClickHouse contrast in
  `retention-and-ttl.md`.
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

**Whole schema built live on GreptimeDB `v1.0.2` (Run 45).** All 8 signal tables + 1
logical metric table created clean in a scratch database; `SHOW CREATE TABLE` confirmed
`trace_id INVERTED INDEX` (spans) and `message FULLTEXT INDEX` (logs) attached, and the
logical→physical metric-engine link (`on_physical_table`) works. Two real defects in the
original syntax-only design were caught and fixed:

1. **Reserved-keyword columns must be quoted.** `service`, `name`, `status`, `level`,
   `release`, `url`, `message` are reserved in v1.0.2's SQL parser (`Cannot use keyword
   '…' as column name`); the DDL now quotes them (`"col"`; backticks also work). This is a
   real implementer gotcha — the bench tables dodge it only because OTLP/line-protocol
   ingest auto-creates schema outside the SQL parser.
2. **No empty `PRIMARY KEY ()`** on the metric-engine physical table — invalid syntax;
   omit the clause entirely.

Earlier (Runs 1–6) exercised the trace-lookup variants: `trace_id INVERTED INDEX` closes
the Run-1 16 ms→~8 ms gap (Run 6). Remaining live work is data-loaded Q1–Q6 on *this exact*
schema (vs the bench's simplified tables) — routed to `benchmarking-the-differences.md`.

## Side-by-side

The ClickHouse counterpart is `clickhouse-implementation.md` (next pass); the two
are kept structurally parallel so the design differences — native metric engine +
inverted-index lookups + object-store-native vs MergeTree `ORDER BY` + skip indexes
+ S3-disk-policy — are directly comparable.
