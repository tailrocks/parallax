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

### Run 2 — 2026-05-25 — evidence-bundle join (Q1 + Q4), EXPLAIN plans

Same containers/dataset as Run 1, plus `logs` (214,287 rows) and `error_events`
(2,226 rows) generated to share the spans' `trace_id`/`span_id` (one real pair per
trace). ClickHouse `logs ORDER BY (service, ts)`, `error_events ORDER BY (project,
fingerprint, ts)`; GreptimeDB per seed DDL `PRIMARY KEY` equivalents.

**Correctness parity (anchor `trace_id` with an error): PASS**

| Query | ClickHouse | GreptimeDB |
| --- | --- | --- |
| Q1 `trace_context` (UNION spans+logs+error) | 18 rows (14+3+1) | 18 rows |
| Q4 `cross_tier` (spans LEFT JOIN error_events ON trace_id, span_id) | 14 rows, 1 matched error | 14 rows, 1 matched error |

**Query latency (warm, min of 3)** — same smoke-scale caveat as Run 1.

| Query | ClickHouse | GreptimeDB |
| --- | --- | --- |
| Q1 trace_context | 4 ms | 24 ms |
| Q4 cross_tier join | 3 ms | 54 ms |

**EXPLAIN plans — the real mechanism evidence (scale-independent):**

ClickHouse Q4 (`EXPLAIN actions=1`):

```text
Join (JOIN FillRightFirst)
Algorithm: SpillingHashJoin(ConcurrentHashJoin)
Clauses: [(trace_id, span_id) = (trace_id, span_id)]
  ReadFromMergeTree (default.spans)        Granules: 1   Prewhere: trace_id = '3fb2…'
  ReadFromMergeTree (default.error_events) Granules: 1   Prewhere: trace_id = '3fb2…'
```

GreptimeDB Q4 (`EXPLAIN`):

```text
SortPreservingMergeExec
  HashJoinExec: mode=Partitioned, join_type=Left, on=[(trace_id,trace_id),(span_id,span_id)]
    RepartitionExec: Hash([trace_id, span_id], 10)
      FilterExec: trace_id = '3fb2…'   <- MergeScanExec (spans region)
    RepartitionExec: Hash([trace_id, span_id], 10)
      FilterExec: trace_id = '3fb2…'   <- MergeScanExec (error_events region)
```

**What the plans confirm (and one correction to pass 3):**

1. **ClickHouse:** `FillRightFirst` + `SpillingHashJoin(ConcurrentHashJoin)`
   confirms the broadcast/concurrent-hash + grace-spill family from
   `clickhouse-internals.md`. The anchor `trace_id` became a **PREWHERE** and the
   **sparse index pruned to `Granules: 1`** on the spans side — empirical proof of
   the pass-3 PREWHERE + 8192-row-granule-skip mechanism.
2. **GreptimeDB:** `HashJoinExec: mode=Partitioned` + `RepartitionExec Hash(…,10)`
   confirms the **partitioned hash join** (repartition both sides) from pass 3 —
   the structure that scales to large↔large joins.
3. **Both engines propagate the anchor constant to BOTH join inputs.** ClickHouse
   pushed `trace_id='…'` to the `error_events` scan as a PREWHERE (`Granules: 1`);
   GreptimeDB pushed `FilterExec: trace_id='…'` to *both* region scans. **This
   corrects pass 3**, which implied ClickHouse's broadcast join must build the
   whole right table — for a *constant-anchored* join it does not; the optimizer
   propagates the equi-join constant and prunes both sides first.
4. **Consequence for Parallax (important):** the evidence-bundle queries (Q1–Q6)
   are **always anchored** on a specific `trace_id`/`fingerprint`. Both engines
   reduce each side to a tiny set *before* the join, so the join-algorithm
   difference (broadcast vs partitioned) is **largely irrelevant for Parallax's
   actual query pattern**. The "join strategy decides the winner" framing applies
   only to *un-anchored large↔large* joins, which Parallax does not run for bundle
   assembly. This downgrades the join from "the deciding factor" to "not a
   differentiator for the anchored pattern" — the **key placement** (so the anchor
   prunes cheaply) matters far more, which Run 1 already showed.

**Claims checked (Run 2)**

| Claim | Result | Status |
| --- | --- | --- |
| ClickHouse PREWHERE + sparse-index granule skip on key-anchored read | spans pruned to `Granules: 1` | **confirmed (plan)** |
| GreptimeDB uses partitioned hash join (pass 3) | `mode=Partitioned` in plan | **confirmed (plan)** |
| GreptimeDB pushes the anchor filter into region scans (pass 3) | `FilterExec` on both `MergeScanExec` inputs | **confirmed (plan)** |
| ClickHouse broadcast join must build whole right side (pass 3) | constant propagated → right side pruned to 1 granule | **contradicted (plan)** for anchored joins |
| Evidence-bundle join algorithm decides the winner | both prune before join on anchored queries | **refined**: not a differentiator for anchored Parallax queries |
| Cross-engine bundle correctness (Q1/Q4 identical) | 18 / 14 rows both | **confirmed** |

### Run 3 — 2026-05-25 — metrics signal + PromQL nativeness

Tests the operator hypothesis's strongest GreptimeDB claim: metrics + PromQL.
Dataset: 864,000 points, 1,200 series (12 services × 100 instances), one value
every 30 s over 6 h, a smooth random walk. Same containers.

- ClickHouse: `http_req_latency (ts DateTime64(3) CODEC(DoubleDelta,ZSTD),
  service LowCardinality, instance LowCardinality, value Float64 CODEC(Gorilla,
  ZSTD)) ENGINE=MergeTree ORDER BY (service, instance, ts)`.
- GreptimeDB: `http_req_latency (ts TIME INDEX, service, instance, value DOUBLE,
  PRIMARY KEY (service, instance))` — a plain time-series table.

**Correctness parity: PASS** — SQL 5-min range-aggregation grouped by service:
both return 864 groups (12 × 72 buckets); svc-0 first-bucket `avg(value)` =
**106.2274** on both.

**PromQL nativeness — the capability gap (most important result):**

| | GreptimeDB | ClickHouse |
| --- | --- | --- |
| Native PromQL | **Yes** — `GET /v1/prometheus/api/v1/query_range?query=avg by (service)(http_req_latency)` returned 12 series × 73 points directly over the plain table. | **No** — no PromQL engine. Must translate PromQL→SQL in an external layer. |
| Range query model | Native `query_range` (start/end/step) + PromQL functions (`rate`, `avg_over_time`, `… by (label)`). | Hand-written SQL with `toStartOfInterval` + `groupArray`/window funcs. |

This is a **capability difference, not just a speed delta**. If Parallax exposes
PromQL or ingests Prometheus remote-write, GreptimeDB does it natively; ClickHouse
requires building and maintaining a PromQL compatibility layer. **Confirmed** —
the clearest GreptimeDB advantage found so far, and it is on the metrics signal
exactly as the hypothesis predicted.

**Latency (warm, min of 3, smoke scale — indicative)**

| Query | ClickHouse | GreptimeDB |
| --- | --- | --- |
| SQL 5-min range-agg by service | 12 ms | 16 ms |
| Native PromQL `avg by (service)` (wall-clock incl. HTTP) | n/a | 48 ms |

GreptimeDB is **within ~1.3× of ClickHouse on the metric aggregation** — far
closer than the 2–3× gap it showed on log/trace scans (Run 1). Consistent with
metrics being GreptimeDB's design center. Still cache-resident scale; directional.

**Compression NOT meaningfully tested this run.** The ClickHouse `value` column
(Gorilla+ZSTD) compressed to 6.03 MiB for 864k float64 (~6.6 MB raw) — i.e. almost
no compression, because the synthetic random-walk values are high-entropy. Real
metrics (flat gauges, monotonic counters, repeated values) are exactly what
Gorilla/DoubleDelta target and compress 5–10×. **Re-run float compression with
realistic metric shapes** before any cost conclusion.

**Claims checked (Run 3)**

| Claim | Result | Status |
| --- | --- | --- |
| GreptimeDB PromQL-native; ClickHouse not | PromQL works on GreptimeDB, absent in ClickHouse | **confirmed** |
| GreptimeDB competitive/faster on metric aggregation | 16 ms vs 12 ms (within 1.3×) | *plausible* — directional, cache-resident scale |
| Gorilla codec shrinks float metrics | not exercised (incompressible synthetic data) | *inconclusive* — redo with realistic shapes |
| Cross-engine metric-aggregation correctness | 864 groups + 106.2274 both | **confirmed** |

### Run 4 — 2026-05-25 — per-signal compression (cost axis)

Measured retained size for all loaded tables (flushed / `OPTIMIZE FINAL`) plus a
realistic counter+gauge metric table. **Full analysis in
[`compression-and-cost.md`](compression-and-cost.md).** Headline: **no blanket
compression winner** — ClickHouse wins tuned counters (`DoubleDelta` 7.3×), flat
gauges (`Gorilla` 78×) and high-cardinality random strings (`spans` 28.9 vs 38
MiB); GreptimeDB wins dictionary-friendly low-card columns (`logs` 5.5 vs 10.24
MiB ⚠ synthetic) and high-entropy floats where Gorilla backfires
(`http_req_latency` 5.1 vs 6.31 MiB). Cost is closer to a tie than pass 2 implied;
object-store $ (MinIO) still unmeasured.

## Next runs (to make the numbers mean something)

1. **Bigger tier** (`small` ≈ 25–50 GB, cold cache) so scans exceed cache and the
   vectorized-engine + granule-skip mechanisms actually bite. Drop OS page cache
   between cold runs.
2. **Matched-codec/object-cost gate**: run the
   [storage size and object cost gate](../storage-size-and-object-cost-gate.md)
   so retained bytes, object counts, request costs, cache needs, and egress are
   measured rather than inferred from the tiny local-disk smoke result.
3. **Full mixed-load Q6 gate**: run the
   [storage freshness and bundle latency gate](../storage-freshness-and-bundle-latency-gate.md)
   with all bundle signals, per-signal freshness probes, and concurrent ingest.
4. **Metrics float compression with realistic shapes** (flat gauges, monotonic
   counters, repeated values) to actually exercise Gorilla/DoubleDelta vs
   GreptimeDB Parquet — Run 3's random-walk data was incompressible. (PromQL
   nativeness + aggregation latency already done in Run 3.)
5. **Fairer GreptimeDB timing** via the MySQL native protocol, not HTTP.
6. **Object-storage path** (MinIO) for both — add to `bench/compose.yml`; cost
   interpretation belongs to the
   [storage size and object cost gate](../storage-size-and-object-cost-gate.md).

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
