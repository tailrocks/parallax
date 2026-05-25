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

### Run 5 — 2026-05-25 — freshness + ingest throughput (axis #1)

Full analysis in [`write-path-and-ingestion.md`](write-path-and-ingestion.md).

- **Freshness = tie.** A single synchronous insert was **immediately queryable on
  both** engines (count=1 on the first query after ack); neither needs a
  flush/merge. Per-call ms (CH 288, GT 124) are client/HTTP overhead, not the
  mechanism — they do not rank freshness.
- **ClickHouse 26.x reports `async_insert=1` by default** (busy timeout 50–200 ms):
  small inserts auto-batch → visible after the buffer window, not instantly.
- **Bulk ingest:** ClickHouse 1M spans in 0.575 s (~1.74M rows/s, client wall) vs
  GreptimeDB 0.895 s (~1.12M rows/s, server time). Both >1M rows/s; inconclusive at
  smoke (different measurement bases, non-concurrent).
- **Mechanism difference that matters:** ClickHouse writes one part per INSERT →
  small high-frequency inserts risk "too many parts" → needs batching/async-insert;
  GreptimeDB's LSM memtable absorbs small writes natively. Favors GreptimeDB for
  streaming small-batch telemetry.

### Run 6 — 2026-05-25 — B2: GreptimeDB `trace_id INVERTED INDEX` validation

Tests `benchmarking-the-differences.md` B2: does adding `trace_id INVERTED INDEX`
to GreptimeDB spans close the Run-1 trace-lookup gap? Built `spans_idx` (same 1M
spans, `trace_id STRING INVERTED INDEX`, `append_mode`), flushed (index → Puffin),
re-measured `WHERE trace_id = ?` (warm, min of 3). Parity: 14 rows on all.

| Table | trace lookup | vs |
| --- | --- | --- |
| GreptimeDB `spans_idx` (INVERTED INDEX) | **8 ms** | the fix |
| GreptimeDB `spans` (no index, Run-1 baseline) | 14 ms | un-indexed |
| ClickHouse `spans` (`ORDER BY (trace_id, ts)`) | **2 ms** | sort-prefix seek |

**Reading (honest):** the inverted index **~halved** GreptimeDB's trace lookup
(14→8 ms) — the fix **helps and is confirmed directionally**. But it did **not**
reach ClickHouse parity (still ~4×). Since GreptimeDB's `execution_time_ms` is its
own *server-side* figure (excludes HTTP transport), the residual gap is **real
fixed query-setup overhead** (DataFusion planning + `MergeScanExec` region-scan
setup), not a measurement artifact — at 1M cache-resident rows that fixed floor
(~8 ms) dominates, below which an index cannot push. ClickHouse's leaner native
path floors lower (~2 ms).

**B2 status: partially confirmed.** Index helps; parity not reached *at smoke
scale*. The index's value (pruning) should matter more at larger scale where
actual scanning — not the fixed planning floor — dominates; **re-test at `small`+
and via the GreptimeDB MySQL native protocol** (lower per-query overhead than HTTP)
before concluding. Does not change the verdict (trace lookup is fast enough in
absolute terms — 8 ms — for anchored bundle assembly).

### Run 7 — 2026-05-25 — B9: small-write part behaviour (self-correction)

Tested `benchmarking-the-differences.md` B9: does ClickHouse's one-part-per-INSERT
cause part-explosion on small writes vs GreptimeDB's memtable? Drove 300 single-row
INSERTs (async_insert=0) into ClickHouse; 100 into GreptimeDB.

| Observation | Result |
| --- | --- |
| ClickHouse `NewPart` events (part_log) | **300** — confirms **one part per INSERT** |
| ClickHouse merge events | 61 — background merges ran concurrently |
| ClickHouse **active** parts after | **1** (300 → merged down) |
| `parts_to_throw_insert` default | **3000** |
| GreptimeDB 100 inserts | absorbed in memtable → 1 SST on flush (no per-insert files) |

**Self-correction to passes 9/14.** The mechanism is real (ClickHouse *does* create
one part per insert), **but background merges collapse bounded bursts aggressively**
(300 parts → 1 active), and the throw guard is far away (3000). So "too many parts"
is a **sustained-rate** failure — insert rate persistently exceeding merge
throughput — **not** a per-insert problem, and `async_insert` (default on in 26.x)
mitigates it further. My pass-9 framing overstated it.

**Refined claim:** GreptimeDB's memtable-absorption write-path advantage is **real
but narrower** — it matters for *sustained* high-frequency tiny writes that outpace
ClickHouse's merge rate (where ClickHouse needs async-insert/batching tuning and
GreptimeDB does not). For bounded/moderate small-write bursts, ClickHouse copes via
merges + async insert. Confirming the *sustained* failure needs a rate-ramp test
(insert faster than merges keep up until 3000) — proposed for the harness.

**B9 status: done, refined** (mechanism confirmed; severity downgraded to a
sustained-rate concern).

### Run 8 — 2026-05-25 — B10 (partial): GreptimeDB object storage on MinIO

First object-storage run. Stood up MinIO + bucket `greptimedb` on an isolated
network; ran a GreptimeDB `v1.0.2` standalone with `[storage] type = "S3"`,
`endpoint = http://…minio:9000`, path-style, against MinIO; ingested the 1M spans,
flushed. (Config via `docker create` + `docker cp` + `docker start` — bind-mounts
don't reach the orbstack daemon.)

| Observation | Result |
| --- | --- |
| GreptimeDB-S3 startup | clean — logs confirm `store: S3(bucket: greptimedb)`; healthy in ~4 s |
| Ingest 1M spans → flush | OK (COPY 950 ms server-side), 1,000,000 rows queryable |
| **MinIO footprint** | **36 MiB across 4 objects** |
| vs local-disk SST (Run 1) | 38 MiB — **no object-storage size penalty** (same Parquet SST) |

**Findings (cost axis #2):**

1. **GreptimeDB object-store-native is real and clean** — one `[storage]` block,
   data lands in S3 directly as Parquet. Empirically confirms the verdict's
   "object-store-native" claim (vs ClickHouse's S3-disk-under-a-policy).
2. **Few, large objects (4 for 1M rows)** → **request-efficient on S3**: fewer
   GET/PUT/LIST, so lower per-request cost amplification — the thing that dominates
   object-store bills for a re-read-heavy engine (`retention-cost-model.md`).
   ClickHouse Wide parts store **one object per column per part** → many more,
   smaller objects → more requests; this is the contrast to measure next.

**B10 status: partial.** GreptimeDB side done. **Still owed:** ClickHouse `s3`
disk + storage-policy run on the same MinIO (object count + bytes), and actual
GET/PUT/LIST counts (MinIO audit log / `mc admin trace`) during ingest and during
cold-cache Q1–Q6 — the real request-cost comparison. Cold-read egress too.

### Run 9 — 2026-05-25 — B10 complete: ClickHouse vs GreptimeDB object layout on MinIO

Stood up a ClickHouse `v26.5.1.882` with an `s3` disk + `storage_policy='s3only'`
against the **same MinIO**, loaded the same 1M spans, `OPTIMIZE FINAL`. Compared
the object layout to GreptimeDB-S3 (Run 8).

| Engine | Objects in S3 | S3 bytes used | Active logical bytes |
| --- | --- | --- | --- |
| **GreptimeDB** | **4** | 37 MiB | 37 MiB |
| **ClickHouse** | **74** | 63 MiB | 31.82 MiB (1 Wide part) |

**Findings (cost axis #2 — the decisive object-store result):**

1. **Object count: GreptimeDB 4 vs ClickHouse 74 (~18×).** ClickHouse's Wide part
   stores **one S3 object per column** (+ marks + metadata), so a single table
   becomes dozens of objects; GreptimeDB writes a few large Parquet objects. **This
   is the object-store-economics advantage**: per-request pricing dominates an
   object-store bill, and a cold read in ClickHouse must issue **many more S3 GETs**
   (one per needed column file) than GreptimeDB's few-Parquet-file reads. Confirms
   the verdict's "object-store-native" claim with a hard number.
2. **Size nuance — a partial reversal.** Active logical data: ClickHouse 31.82 MiB
   < GreptimeDB 37 MiB (ClickHouse's tuned spans codecs win on the high-card hex
   columns, consistent with Run 1's local result). But ClickHouse's **raw S3 usage
   was 63 MiB** — nearly 2× its logical — because pre-`OPTIMIZE` merge parts' S3
   objects are **not yet garbage-collected** (ClickHouse S3 cleanup is async). So
   ClickHouse on object storage carries **transient space amplification** from
   merge garbage until cleanup runs — an operational cost GreptimeDB's LSM-flush
   model largely avoids.

**B10 status: done** (object layout + footprint). Remaining refinement: actual
GET/PUT/LIST **request counts** during cold-cache Q1–Q6 (MinIO audit / `mc admin
trace`) to quantify the request-cost gap the 4-vs-74 object split implies — but the
object-count proxy already shows the direction decisively.

Cleanup: the MinIO + GreptimeDB-S3 + ClickHouse-S3 containers and `pbench-s3`
network are torn down after this run (ephemeral; nothing committed).

### Run 10 — 2026-05-25 — B7: realistic-cardinality log-text compression

Re-ran log compression with **realistic high-entropy text** (500k rows, **99%
unique messages**: templated with embedded UUIDs/IDs/latencies + stack-trace
lines), fixing Run 4's synthetic-cardinality distortion (Run 4 had 10 distinct
messages).

| Schema | Total on disk | Notes |
| --- | --- | --- |
| GreptimeDB `logs_real` (default ZSTD-all) | **25 MiB** | Parquet + table-wide ZSTD |
| ClickHouse `logs_real` (only `message` ZSTD; ids default **LZ4**) | 35.53 MiB | trace_id 15.3M + span_id 7.7M dominate (LZ4 on hex) |
| ClickHouse `logs_real_z` (**ZSTD on all string cols**) | **24.24 MiB** | trace_id 15.3→7.8M, span_id 7.7→3.9M |

**Finding — corrects both earlier framings:**

- Run-4's GreptimeDB logs win was **not** purely a synthetic artifact: with
  realistic 99%-unique text GreptimeDB **still wins at defaults** (25 vs 35.5 MiB).
- **But the win is a default-codec effect, not engine superiority.** ClickHouse's
  per-column default is **LZ4**; the high-cardinality hex `trace_id`/`span_id`
  columns compress poorly under LZ4. Switching them to ZSTD drops ClickHouse to
  **24.24 MiB ≈ GreptimeDB's 25 MiB** — essentially a **tie when both tuned**.
- **Operational nuance:** GreptimeDB ZSTDs everything automatically → good log
  compression with **zero tuning**; ClickHouse needs explicit per-column `CODEC(ZSTD)`
  on high-card columns to match (its default LZ4 leaves ~30% on the table here).

**B7 status: done.** Realistic-log compression is a **tie at matched effort**,
**GreptimeDB-favored out-of-the-box**. Reinforces the pass-8 "compression is a
tuning-dependent wash" conclusion with realistic data, plus the defaults nuance.

### Run 11 — 2026-05-25 — B5: high-cardinality metrics (40k series, 8M rows)

Re-ran the metric path at the prototype's real cardinality (40 services × 1000
instances = **40,000 series**, 200 points each = 8M rows), vs Run 3's 1,200 series.
Plain time-series table on both (not the metric engine / PromQL path).

| Measure | ClickHouse | GreptimeDB |
| --- | --- | --- |
| Bulk ingest 8M rows | 0.669 s (~12M rows/s) | 2.98 s (~2.7M rows/s) |
| Retained size | 57.42 MiB | 62 MiB |
| **`avg by service`, 5-min buckets (SQL group-by)** | **65 ms** | **638 ms (~10×)** |
| single-series lookup | 3 ms | 9 ms |

**Significant refinement of the metrics finding.** At 1,200 series (Run 3) the SQL
range-aggregation was a near-tie (16 vs 12 ms); at **40k series / 8M rows it is
~10× in ClickHouse's favour** (65 vs 638 ms), and ClickHouse ingested ~4.5× faster.
This is **predicted by the internals** — ClickHouse's decade-tuned vectorized C++
group-by over a columnar scan is the throughput bar (`clickhouse-internals.md`),
and Run-3's near-tie was a **small-scale / cache-resident artifact** (the
fixed-overhead floor, not throughput). At real volume the scan-aggregate engine
gap shows.

**Consequence (sharpens the verdict's metrics pillar):** GreptimeDB's metrics
advantage is **PromQL-nativeness + native ingest (capability), NOT aggregation
speed at volume.** For heavy metric *analytics* at scale, ClickHouse is materially
faster (~10×) — it just can't speak PromQL. So "metrics → GreptimeDB" holds **only
on the capability/ingest axis**, not on raw query latency at volume.

Caveat: this is the SQL group-by (ClickHouse's core strength), not GreptimeDB's
native PromQL planner or the metric engine (logical→physical) — a PromQL-path run
+ the metric engine could differ and is owed. But for SQL-shape metric aggregation,
the volume result is clear. Also a **preview of B1**: at 8M rows ClickHouse's scan
engine already shows ~10×; the cold GB–TB log/trace scan likely shows it larger.

**B5 status: done** (SQL aggregation); PromQL-path + metric-engine high-card run owed.

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
