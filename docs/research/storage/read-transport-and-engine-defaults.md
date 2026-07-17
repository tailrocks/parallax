# Read transport and engine defaults — measurement spike (plan 090)

<!-- markdownlint-disable MD013 -->

- **Research date:** 2026-07-11
- **Engine under test:** GreptimeDB `1.1.2` (`SELECT version()` → `1.1.2`)
- **Harness:** [`poc/read-transport-bench/`](../../../poc/read-transport-bench/)
- **Raw JSON:** [`poc/read-transport-bench/results/`](../../../poc/read-transport-bench/results/)
- **Planned at:** commit `df81d86` (inventory read from the historical PR #19
  implementation working tree after items 084/085)

## Product adoption (plan 091 — 2026-07-11)

**GO landed.** Heavy GreptimeDB typed reads now use HTTP
`format=arrow&compression=zstd` via `GreptimeStore::sql_arrow` /
`sql_with_schema_arrow` (transport in
`crates/parallax-greptime/src/greptime/transport.rs`, decode in
`crates/parallax-greptime/src/arrow_sql.rs`).
Domain callers still consume `Vec<Vec<serde_json::Value>>` / `SqlResult` — no
GraphQL or UI contract change.

| Path | Wire format |
|------|-------------|
| `select_spans`, `select_logs` / `logs_search`, `traces_search` page, `histogram_*`, metric/log/signal series, service-map edges, batched spans-by-runs | **Arrow + zstd** |
| DDL/admin, `information_schema`, `LIMIT 0` schema probes, single-row `COUNT(*)`, raw SQL playground, other tiny probes | **`greptimedb_v1` JSON** (`sql` / `sql_with_schema`) |

Uncompressed Arrow is intentionally never used on the product path (090:
`logs_search` IPC was ~7.8× larger than JSON without zstd). `cargo tree -p
parallax-storage -i rustls` stays empty; `arrow-ipc` uses the `zstd` crate only.

## Purpose

Historically all Parallax product reads went through GreptimeDB HTTP `/v1/sql`
returning `greptimedb_v1` JSON (then in
`crates/parallax-storage/src/greptime.rs`; concrete GreptimeDB I/O now lives in
`parallax-greptime`). The 2026-07-10 audit verified that the engine also offers:

1. `format=arrow` (+ optional `compression=zstd|lz4`) on the same endpoint
2. MySQL (:24002) / Postgres (:24003) wires with **session-cached prepared plans**
3. Native `RANGE`/`ALIGN` time-bucket syntax (vs `date_bin` + `GROUP BY`)
4. Auto-created `opentelemetry_traces` default **16 partitions by `trace_id`**
   (`trace_table_partitions` hint can shrink it)

This spike measured those candidates on the real read mix and issued
GO / NO-GO / REVISIT verdicts. The spike itself made no product changes; its
Arrow+zstd GO was subsequently implemented in `parallax-greptime`.

## Method

- Same-engine A/B (not four-way cross-engine; four-way rule does not apply).
- Synthetic laptop-tier dataset via in-engine `range()` (same discipline as
  `bench/four-way/gen.sh`), **N = 100,000** rows per table (≥ 50k floor).
- Tables are **inventory-shaped**, not native OTLP auto-create (OTLP/protobuf
  seeding at 100k was out of scope for this harness). Partition A/B uses SQL
  `PARTITION ON COLUMNS (trace_id)` with **4 regions** as a multi-region proxy
  for the native 16-way default — labelled as such in every partition table.
- Client: `poc/read-transport-bench` (tokio + reqwest `native-tls`,
  `arrow-ipc` + zstd, `mysql_async` with **no TLS features** — plaintext
  localhost; `cargo tree -i rustls` empty).
- Warmup 5, reps 50 per cell. Wall-clock is client-observed (network + server +
  decode). Decode is client-side only (JSON parse or Arrow IPC stream read).

## Dataset

| Table | Rows | Notes |
|-------|-----:|-------|
| `opentelemetry_traces` | 100,000 | ~200 spans / `trace_id` (`t0`…`t499`) |
| `opentelemetry_logs` | 100,000 | JSON attrs + FULLTEXT body |
| `http_server_request_duration_seconds_count` | 100,000 | metric-engine-like count sibling |
| `http_server_request_duration_seconds_bucket` | 100,000 | `le` buckets |
| `traces_p1` | 100,000 | 1 region (partition proxy low) |
| `traces_p16` | 100,000 | **4** SQL RANGE regions (multi-region proxy; **not** native 16-hash) |

Seed timestamps are `TIMESTAMP(3)` milliseconds starting at `1716000000000`.
Inventory windows use matching ms bounds (native OTLP product tables use
nanosecond `timestamp` literals — shape of SQL is the same; payload width
differs on auto-widened native columns).

## Step 1 — Frozen query inventory

Extracted from then-live `crates/parallax-storage/src/greptime.rs` (post-084,
with 085 working-tree histogram rewrites present). Current query builders and
analytics live in `parallax-greptime`. Six heaviest shapes:

| id | Surface | Provenance | SQL shape (frozen in harness) |
|----|---------|------------|-------------------------------|
| `select_spans` | Trace detail / span tree | `select_spans_sql` + `select_spans` | `SELECT * FROM opentelemetry_traces WHERE trace_id = 't1' … LIMIT 500` |
| `logs_search` | Logs page (500 rows) | `select_logs_sql` + `logs_search` + `log_filter_clauses` | CAST ts + COALESCE service + `json_to_string` attrs, ORDER BY ts DESC LIMIT 500 |
| `traces_search` | Trace list page | `traces_search_sql` + `traces_search` | Window `ROW_NUMBER` root pick + span_count join + page LIMIT 50 |
| `metric_series` | Metric rate numerator | `histogram_count_series_sql` | `date_bin` + `SUM(greptime_value)` GROUP BY bucket |
| `histogram_buckets` | Histogram quantile inputs | `histogram_quantile_bucket_sql` (085 windowed MAX) | `date_bin` + `MAX(greptime_value)` GROUP BY bucket, le |
| `service_summaries` | Service list RED-ish | `service_summaries` | GROUP BY service + `approx_percentile_cont(duration_nano, 0.95)` |

Verbatim SQL lives in `poc/read-transport-bench/src/lib.rs` (`INVENTORY`).

## Step 3 — HTTP format A/B (p50 / p95 wall ms, bytes, decode)

Engine `1.1.2`, N=100k, reps=50, warmup=5. Source:
`poc/read-transport-bench/results/bench.json`.

| Query | Rows | JSON p50 / p95 (ms) | Arrow p50 / p95 | Arrow+zstd p50 / p95 | JSON p50 B | Arrow p50 B | zstd p50 B | Notes |
|-------|-----:|---------------------|-----------------|----------------------|-----------:|------------:|-----------:|-------|
| `select_spans` | 200 | 9.1 / 42.6 | 8.4 / 43.9 | 8.7 / 43.6 | 17 157 | 18 504 | 6 600 | Small page; wall ~flat; zstd shrinks ~2.6× |
| `logs_search` | 500 | **324 / 423** | **258 / 314** | 281 / 459 | 91 080 | **711 048** | **22 984** | Arrow wins wall (~20%); **uncompressed Arrow much larger** than JSON; zstd wins bytes |
| `traces_search` | 50 | 94.9 / 158.5 | **67.6 / 101.7** | **60.9 / 92.4** | 3 134 | 5 064 | 2 696 | Arrow/zstd clear wall win on this plan shape |
| `metric_series` | 1 667 | 6.3 / 49.6 | 4.9 / 37.5 | 5.4 / 39.3 | 36 861 | 27 784 | 5 448 | Mild wall win; zstd ~6.8× smaller than JSON |
| `histogram_buckets` | 8 334 | 8.9 / 48.5 | **6.7 / 42.0** | **6.9 / 29.8** | 220 970 | 204 168 | **24 968** | Best zstd story (~9× vs JSON); decode 1.04 → 0.05 ms |
| `service_summaries` | 12 | 11.3 / 34.2 | 11.9 / 38.1 | 9.9 / 13.7 | 746 | 1 800 | 1 544 | Tiny result — noise; keep JSON fine |

**Decode (p50 ms, client):** JSON dominates only when rows are wide/tall
(`histogram_buckets` 1.04 ms JSON vs 0.05 Arrow). For 12-row summaries decode is
noise.

**Parity:** row counts matched across HTTP formats for every inventory query.

### HTTP takeaway

- **`format=arrow&compression=zstd` is the right default for heavy read pages**
  (logs page, histogram bucket pulls, dense span fetches, metric series).
- Uncompressed Arrow is **not** a free win: on `logs_search` the IPC stream was
  ~7.8× larger than JSON (binary fixed-width / validity buffers) even though
  wall was better — always pair Arrow with zstd (or lz4) on the wire.
- Keep `greptimedb_v1` JSON for schema probes, counts, and tiny result sets
  (decode + Arrow reader startup not worth it).

## Step 4 — MySQL wire prepared statements

Client: `mysql_async 0.37` with `default-features` only (**no** `rustls*`, no
TLS — plaintext `127.0.0.1:24002`). Pool reconnect measured ~0.06–0.22 ms
(local, empty plan cache miss path).

| Query | Prepared? | Notes |
|-------|-----------|-------|
| `select_spans` | prepared OK | Returned **0 rows** over MySQL wire despite HTTP returning 200 (catalog/session quirk under this harness — **row parity UNRELIABLE**) |
| `logs_search` | **PREPARE FAIL** | `json_to_string` type coercion fails on MySQL protocol path |
| `traces_search` | **PREPARE FAIL** | planner: field access on Utf8 (window/join projection) |
| `metric_series` | **PREPARE FAIL** | `INTERVAL` unit parse rejected as prepare statement |
| `histogram_buckets` | **PREPARE FAIL** | same `INTERVAL` prepare rejection |
| `service_summaries` | **PREPARE FAIL** | `approx_percentile_cont` coercion fail on MySQL path |

**Verdict signal:** even before performance, **5/6 inventory queries cannot be
prepared** on the MySQL wire with the SQL Parallax actually issues. The wire
is not a drop-in replacement for HTTP `/v1/sql` for this dialect surface.
Adoption cost is **L** (new client pool, TLS policy, dual path, dialect
matrix) with **failed prepare** on the heavy shapes.

Mark measured wall numbers for MySQL as **not product-comparable** (empty
result for the one prepare that succeeded). Plan-cache benefit is real in
engine source, but unusable until SQL is rewritten for the wire dialect or
the prepare path is fixed upstream.

## Step 5 — RANGE vs `date_bin` (spot check)

Same metric series shape, 20 timed reps + `EXPLAIN ANALYZE`.

| Form | p50 wall ms | Rows | Plan highlight |
|------|------------:|-----:|----------------|
| `date_bin` + `GROUP BY` (product) | **5.51** | 1 667 | `AggregateExec` partial/final + `date_bin` gby |
| `SUM(...) RANGE '60s' … ALIGN '60s'` | **5.10** | 1 667 | `RangeSelectExec` (align=60000ms) |

Plans differ (`RangeSelectExec` vs hash aggregate); timings are within noise at
N=100k / 8k scanned series rows. **No adoption pressure** to rewrite product
`date_bin` calls — keep the portable form unless a later large-scale bench
shows RANGE winning on tall series.

## Step 6 — Partition count (1 vs multi-region proxy)

HTTP JSON only, reps=50. `traces_p1` = 1 region; `traces_p16` = **4** SQL
RANGE partitions on `trace_id` (proxy — **native OTLP default is 16 hash
partitions** via `trace_table_partitions`).

| Query | 1-region p50 / p95 (ms) | 4-region p50 / p95 | Rows | Notes |
|-------|-------------------------|--------------------|-----:|-------|
| spans-by-trace (`t1`) | 12.0 / 13.8 | **5.3 / 6.0** | 200 | Multi-region **faster** (partition prune on equality) |
| traces_search page | 30.2 / 34.5 | 23.8 / 32.8 | 50 vs **unstable** | 4-region page row counts drifted (0…50) across reps |

**Caveats (honest):**

- This is **not** the native 16-way hash layout from OTLP auto-create.
- Multi-region `traces_search` showed **row-count instability** under this
  synthetic layout (harness logged drift). Do not treat 4-region search wall
  times as a clean A/B.
- Partition hint only affects **fresh** data dirs / new tables; existing
  tables keep their partitioning.
- Ingest-side region memory / region count metrics: 1 vs 4 regions observed in
  `information_schema.region_peers`; no meaningful laptop memory pressure at
  N=100k.

## Decision table

| Candidate | Measured delta (this run) | Adoption cost | Verdict |
|-----------|---------------------------|---------------|---------|
| HTTP `format=arrow` (+ **zstd**) | Logs page ~20% faster wall; histogram decode 20×; zstd wire ~4–9× smaller on tall pages. Uncompressed Arrow can be **larger** than JSON. | **S** per endpoint (change `sql()` response path + `arrow-ipc` decode; keep JSON for tiny results) | **GO — implemented** (former plan 091) |
| MySQL/PG prepared statements | 5/6 inventory SQLs **fail PREPARE** on MySQL wire; remaining path had empty/unreliable rows in harness | **L** (pool, dual transport, dialect rewrite, TLS policy) | **NO-GO** for product path today; re-open only if engine prepare dialect matches HTTP SQL or we rewrite queries |
| `RANGE`/`ALIGN` vs `date_bin` | ~same wall at laptop N; different plan | Cosmetic / query-by-query | **NO-GO** (keep `date_bin`) |
| `trace_table_partitions=1` (laptop) | Point lookup faster multi-region; search A/B confounded; proxy ≠ native 16 | **S** but fresh dirs only | **REVISIT-AT-SCALE** — do **not** change product default from this proxy; remeasure on native OTLP auto-create 1 vs 16 with ≥500k spans |

## Sources

- GreptimeDB HTTP API — `format=arrow`, `compression=zstd|lz4`
  ([docs protocols/http](https://docs.greptime.com/user-guide/protocols/http/))
- RANGE/ALIGN SQL
  ([docs query-data/sql](https://docs.greptime.com/user-guide/query-data/sql/))
- Engine facts verified 2026-07-10 in plan 090 (MySQL/PG plan cache in
  `servers/src/mysql/handler.rs` / `postgres/handler.rs`;
  `DEFAULT_PARTITION_NUM_FOR_TRACES`; Arrow result buffers full response)
- Live measurement 2026-07-11, GreptimeDB 1.1.2 standalone, aarch64 Linux
  harness host, data dir `/tmp/parallax-090-measure`

## Re-run

```bash
# Start greptime standalone (or parallax serve) on :24000 / :24002
export GREPTIME_HTTP=http://127.0.0.1:24000
export GREPTIME_MYSQL=mysql://127.0.0.1:24002/public
cd poc/read-transport-bench
cargo test
cargo run --release -- seed --n 100000
cargo run --release -- bench --reps 50 --out results/bench.json
cargo run --release -- partition-bench --reps 50 --out results/partition.json
cargo run --release -- range-check --out results/range.json
```

Pin `SELECT version()` on every re-run; re-seed after major engine bumps.
