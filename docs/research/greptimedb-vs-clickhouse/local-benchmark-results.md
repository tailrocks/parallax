# Local Benchmark Results — Docker Smoke Runs

<!-- markdownlint-disable MD013 -->

Empirical log of local Docker runs. **Every number here is an indicative
laptop/dev-box smoke result, not a production verdict** (per the brief's honesty
rule). Numbers exist to confirm/refute the mechanism predictions in the internals
notes and the public performance claims — not to settle the choice. The runnable
`parallax-bench` harness in `storage-benchmark-prototype.md` holds the real veto.

## Run log

### Run 1 — 2026-05-25 — spans smoke, local disk, warm cache

**Environment**

| Item | Value |
| --- | --- |
| Host | Linux container dev box (orbstack); Docker 29.5.0, compose v5.1.3 |
| Compose | `bench/compose.yml` (local disk, no MinIO yet) |
| GreptimeDB | `greptime/greptimedb:v1.0.2` — standalone, default config (no codec tuning) |
| ClickHouse | `clickhouse/clickhouse-server:26.5.1.882` |
| Dataset | 1,000,000 synthetic spans, 14 spans/trace (~71k traces), 12 services, 3% error, seed 42. Identical CSV loaded into both. ~129 MB raw TSV. |
| Schema | The seed DDL from `storage-benchmark-prototype.md` spans table, **minus** the JSON/Map `attributes` column (added in a later run). ClickHouse `ENGINE=MergeTree ORDER BY (trace_id, ts)`; GreptimeDB `PRIMARY KEY (service, name)`, `ts TIME INDEX`. |
| Measurement | ClickHouse: `clickhouse-client --time` on `FORMAT Null`, min of 3 (warm). GreptimeDB: HTTP `/v1/sql` server-reported `execution_time_ms`, min of 3 (warm). |
| Caveat | Queries run **inside** the containers via `docker exec` (sandbox blocks host→container published ports). |

**Correctness parity (gate — must pass before trusting latency): PASS**

| Check | ClickHouse | GreptimeDB |
| --- | --- | --- |
| `count()` | 1,000,000 | 1,000,000 |
| `count WHERE status='error'` | 29,731 | 29,731 |
| spans for one `trace_id` | 14 | 14 |
| `avg(duration_ms)` | 24.96 | 24.96 |

**Retained on-disk size (after `OPTIMIZE FINAL` / `flush_table`)**

| Engine | Retained data | Note |
| --- | --- | --- |
| ClickHouse | **28.9 MiB** (1 part; 27.9 MiB compressed vs 101 MiB uncompressed ≈ 3.6×) | Schema uses tuned codecs: `ts CODEC(DoubleDelta,ZSTD)`, `LowCardinality` on service/name/status. |
| GreptimeDB | **38 MiB** SST (Parquet) | + 46 MiB WAL (transient, raft-engine; truncates in steady state — **not** counted as retained) + 2.1 MiB metadata. Default Parquet codecs, **no codec tuning** in the seed DDL. |

→ ClickHouse ~24% smaller on this dataset, **but** the comparison is codec-tuned
(ClickHouse) vs defaults (GreptimeDB) — a *schema-tuning* gap, not purely engine.
Re-run with matched codec effort before drawing a cost conclusion.

**Query latency (warm, min of 3)**

| Query | ClickHouse | GreptimeDB | Read |
| --- | --- | --- | --- |
| `count(), avg(duration_ms)` (full scan+agg) | 4 ms | 11 ms | both scan 1M rows |
| `count WHERE status='error'` (selective, off-key) | 3 ms | 9 ms | neither has status in key |
| `count WHERE trace_id=…` (point lookup) | **2 ms** | **16 ms** | **schema asymmetry** (below) |
| `GROUP BY service` | 4 ms | 12 ms | low-card group-by |

## What these numbers do and do not show

**Honest reading — ClickHouse won every query here, but interpret with care:**

1. **Scale is trivial (1M rows, ~30 MB).** The whole dataset is cache-resident.
   These are **fixed-overhead / minimum-latency floors**, NOT scan throughput.
   They cannot confirm or refute the at-scale scan claims (the interesting regime
   is GB–TB, cold cache). *Inconclusive at this scale* for the throughput claims.
2. **The `trace_id` lookup gap (2 ms vs 16 ms) is the predicted schema
   asymmetry, not raw engine speed.** ClickHouse's seed schema puts `trace_id`
   first in `ORDER BY`, so the sparse primary index seeks ~1 granule. GreptimeDB's
   seed schema keys on `(service, name)` with `trace_id` un-indexed, so it scans.
   This **confirms the pass-3 prediction** that trace-context retrieval is decided
   by key placement — and flags that the GreptimeDB Parallax schema must put
   `trace_id` in the primary key / add an index (feeds `greptimedb-implementation.md`).
3. **Measurement is only roughly comparable.** ClickHouse `--time` is native-client
   query time; GreptimeDB `execution_time_ms` is its own server-side figure over
   HTTP. Close enough to read direction at this scale, not for a precise ratio.

**Claims checked**

| Claim | Result | Status |
| --- | --- | --- |
| ClickHouse faster on selective/scan queries (pass 3 mechanism) | Faster here, but at cache-resident scale only | *inconclusive at this scale* (direction consistent) |
| Trace lookup decided by sort-key placement (pass 3) | 2 ms (keyed) vs 16 ms (un-keyed) | **confirmed** |
| ClickHouse smaller on-disk (codec breadth, pass 2) | 28.9 vs 38 MiB | *workload-specific* — tuned vs default codecs; re-run matched |
| GreptimeDB metric/PromQL advantage | not tested (no metrics signal in this run) | pending |
| Evidence-bundle large↔large join advantage | not tested | pending |

## Next runs (to make the numbers mean something)

1. **Bigger tier** (`small` ≈ 25–50 GB, cold cache) so scans exceed cache and the
   vectorized-engine + granule-skip mechanisms actually bite. Drop OS page cache
   between cold runs.
2. **Matched-codec GreptimeDB schema** (or accept defaults on both) for a fair
   size/cost comparison.
3. **Add the signals + the join**: load `logs` + `error_events`, run Q1/Q4
   evidence-bundle joins by `trace_id` — the query that decides the choice.
4. **Metrics signal + PromQL** vs ClickHouse AggregatingMergeTree — the predicted
   GreptimeDB strength, currently untested.
5. **Fairer GreptimeDB timing** via the MySQL native protocol, not HTTP.
6. **Object-storage path** (MinIO) for both — add to `bench/compose.yml`.

These route into `benchmarking-the-differences.md` (case design) and the runnable
`parallax-bench` harness (`storage-benchmark-prototype.md`), which owns the real veto.

## Reproduce

```bash
docker compose -f bench/compose.yml up -d
# generate spans.csv (seed 42, 1M rows, 14/trace) — see bench generator
# ClickHouse: CREATE TABLE spans ... ENGINE=MergeTree ORDER BY (trace_id, ts);
#   INSERT INTO spans FROM INFILE '/tmp/spans.csv' FORMAT CSV
# GreptimeDB: CREATE TABLE spans (... PRIMARY KEY ("service","name")); ts TIME INDEX
#   COPY spans FROM '/tmp/spans_h.csv' WITH (FORMAT='CSV')   # needs header row
docker compose -f bench/compose.yml down -v   # cleanup (data dirs are gitignored)
```
