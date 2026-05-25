# GreptimeDB vs ClickHouse — Deep Internals Comparison

<!-- markdownlint-disable MD013 -->

Status: in progress (produced by an indefinite research loop). All planned notes
now drafted (passes 1–14); the loop continues **deepening** — executing the
targeted benchmark cases (`benchmarking-the-differences.md`: B2 trace-id index,
B1 cold GB–TB scan, B7/B10 cost, B8 concurrent freshness, B11 multi-node) and
sharpening the verdict as their numbers land. Provisional verdict: **GreptimeDB on
fit** (metrics-native + ingest/freshness ergonomics + horizontal scaling + Rust),
**not on raw speed** (ClickHouse leads log/trace query latency).

## Purpose

This folder holds a deep, under-the-hood technical comparison of **GreptimeDB**
and **ClickHouse** for the Parallax storage layer. It answers one question, at the
level of the actual implementation rather than marketing:

> How does each system work internally, which design decisions make each one fast
> or slow, and — for Parallax's signals (metrics, logs, traces, and cross-signal
> evidence-bundle correlation) — which should we build on, and why?

It is driven by the loop brief
[`prompts/greptimedb-vs-clickhouse-internals.md`](../../../prompts/greptimedb-vs-clickhouse-internals.md),
which runs indefinitely and deepens these notes one subsystem at a time until the
operator stops it.

## How this fits with the existing storage research

This is the **white-box** layer. It explains the *why* behind the *what* the other
documents establish:

- [`../greptimedb-storage-evaluation.md`](../greptimedb-storage-evaluation.md) —
  strategy/fit evaluation (reasons *about* the systems).
- [`../observability-storage-benchmark-plan.md`](../observability-storage-benchmark-plan.md)
  — what to measure and why.
- [`../storage-benchmark-prototype.md`](../storage-benchmark-prototype.md) — the
  runnable black-box harness that produces numbers and holds veto power over the
  default storage choice.

The benchmark shows *that* one system is faster; this folder must explain *why*,
from the data structures and code paths — and the two must agree. A benchmark
number the internals cannot explain is a flag that one of them is wrong.

## Version pins (re-check and bump every pass)

As of 2026-05-25 (re-verified through pass 25 — pins still current; GreptimeDB
`v1.1.0` exists only as nightly, not GA; ClickHouse `v26.5.1.882-stable` still
latest stable):

| System | Pinned version | Source commit | Notes |
| --- | --- | --- | --- |
| GreptimeDB | `v1.0.2` (GA 2026-05-14) | `0ef54511f710f0ef2c05941c8c600bb4c1fd46c8` | Latest GA; `v1.1.0-nightly` exists but is not stable. |
| ClickHouse | `v26.5.1.882-stable` | tag obj `fae722ba…`; **commit read `5b96a8d8a5e2f4800b43a780911a39dc5a666e1c`** | Latest stable; LTS line is `v26.3.12.3-lts` (`f118ee7c3b4c1a57dde6a389e5c3e29080f38c5d`). |

## Method

- Compare the latest stable release of each system; record exact versions and the
  source commit SHA read in every note (version-freshness rule).
- Read the architecture docs to orient, then confirm load-bearing claims against
  the cloned source (GreptimeDB in Rust, ClickHouse in C++). Cite file paths and
  commits. When docs and code disagree, trust the code.
- Every "X is faster" claim carries a *because* (a concrete mechanism) and a
  *scenario* (signal, query shape, cardinality, cache state, single-node vs
  scaled).
- Verify the operator hypothesis (GreptimeDB fastest, then ClickHouse) honestly;
  a fully-explained result that contradicts it is the most valuable outcome.

## Evaluation axes (priority order)

1. Speed — ingest-to-queryable freshness and evidence-bundle/correlation query
   latency under concurrent ingest+query.
2. Cost — retained size and compression by signal, object-vs-local economics,
   compute per ingested GB and per query class.
3. Scaling — single-node ceiling and horizontal scale-out (horizontal first;
   vertical-only is a flagged limitation).

## Planned notes

These are produced and grown by the loop; this index is updated as they land.

| File | Scope | Status |
| --- | --- | --- |
| `README.md` | Index, method, version pins, status. | seeded |
| `greptimedb-internals.md` | GreptimeDB architecture and code-path teardown. | drafted (pass 1: topology + mito2 storage engine; pass 32: metric-engine logical→physical layout confirmed live — `__table_id`/`__tsid` + label-column union in one physical region set, avoids per-metric region explosion) |
| `clickhouse-internals.md` | ClickHouse architecture and code-path teardown. | drafted (pass 2: topology + MergeTree part/granule/mark, skip indexes, codecs, merge variants; deeper KeyCondition/merge-selector/text-index/S3-cache dives pending) |
| `write-path-and-ingestion.md` | Ingest → durable → queryable, both systems, with the freshness consequence. | drafted (pass 9 + Run 5: freshness = tie (both visible-on-write, no flush barrier); GreptimeDB write-path edge = LSM absorbs small writes (no ClickHouse part-explosion) + native OTLP/Prom ingest; bulk throughput both >1M rows/s; concurrent freshness pending; pass 33: native InfluxDB-line ingest confirmed live — schema-on-write auto-creates the table (tags→PK, field→DOUBLE, auto TIME INDEX, merge_mode=last_non_null), no DDL/collector; OTLP metrics is protobuf-only (JSON rejected)) |
| `read-path-indexing-and-execution.md` | Query planning, indexing, execution, scan-vs-skip, joins. | drafted (pass 3: pushdown, scan/skip order, PREWHERE vs row-group pruning, join strategy; pass 5: join verdict corrected by Run 2 EXPLAIN — both engines prune the anchor before joining, so join algo is not a differentiator for anchored evidence-bundle queries) |
| `rollup-and-continuous-aggregation.md` | Rollup/correlation tooling: GreptimeDB Flow engine (streaming + batching) vs ClickHouse MV + AggregatingMergeTree, for Parallax metric downsampling + issue rollups. | drafted (pass 27: wash with opposite tilts — GreptimeDB Flow cleaner/metric-native (CREATE FLOW … SINK TO … EXPIRE/EVAL) vs ClickHouse MV+AggregatingMergeTree more mature but per-block + -State/-Merge ceremony; neither moves the verdict) |
| `caching-and-cold-warm.md` | Subsystem #7: cache hierarchies + the cold-vs-warm divergence mechanism — explains why warm small-scale runs favor ClickHouse but cold object-store re-reads can favor GreptimeDB (the regime Parallax lives in). | drafted (pass 24: CH 5GiB mark cache + uncompressed OFF, local-disk-tuned; GreptimeDB object-store read cache + few-object layout = few cold S3 GETs; mechanism behind JSONBench cold-run; magnitude owed to bigger/cold runs) |
| `compaction-and-merge.md` | Subsystem #5: GreptimeDB TWCS (time-window) vs ClickHouse SimpleMergeSelector (size-tiered), write amplification, read-speed/freshness effect. | drafted (pass 23: TWCS bounds write-amp on aged time-series — sealed windows never re-merged — vs ClickHouse O(log N) size-tiered re-merge toward few 150GB parts for fast full scans; ties to B9/B10) |
| `compression-and-cost.md` | Layout, codecs, compression by signal, retention-cost consequence. | drafted (pass 8: measured per-table/per-column sizes — NO blanket winner, per-column-pattern; ClickHouse wins tuned counter/gauge/high-card-string, GreptimeDB wins dict-friendly + noisy-float; cost ~tie; object-store MinIO run + realistic-cardinality redo pending; pass 15 Run 6 (B2): GreptimeDB trace_id INVERTED INDEX cut lookup 14→8 ms but not to ClickHouse's 2 ms — residual is fixed query-setup floor, re-test at scale + native protocol); pass 16 Run 7 (B9): self-correction — ClickHouse part-explosion is a sustained-rate failure not per-insert (300 inserts→1 active via merges, guard=3000), GreptimeDB write-path edge real but narrower); pass 17 Run 8 (B10 partial): GreptimeDB-S3 on MinIO = 1M spans in 36 MiB / 4 objects (object-store-native confirmed, request-efficient); ClickHouse-S3 + request counts owed); pass 18 Run 9 (B10 done): same MinIO 1M spans = GreptimeDB 4 objects vs ClickHouse 74 (~18× fewer → request-efficient), measured object-store cost edge for GreptimeDB); pass 19 Run 10 (B7): realistic 99%-unique log text — GreptimeDB 25M vs ClickHouse 35.5M at defaults but 24.24M with ZSTD-all → tie at matched effort, GreptimeDB wins out-of-the-box (ClickHouse default LZ4 on high-card ids)); pass 20 Run 11 (B5): 40k-series/8M-row metric aggregation = ClickHouse 65ms vs GreptimeDB 638ms (~10×) — Run-3 near-tie was a small-scale artifact; GreptimeDB metrics edge = PromQL capability NOT agg speed at volume); pass 21 Run 12 (B1 flip-trigger, 5M logs both indexed): full-text search CH 7ms vs GT 130ms (~18×), selective keyed filter a tie (4 vs 5ms) — log-search-at-volume strongly favors ClickHouse; verdict holds conditional on anchored-retrieval workload); pass 26 Run 13 (B8 concurrent): both pass ≤2× penalty gate (CH 1.55×, GreptimeDB 1.38×) — neither blocks reads on ingest; absolute agg at 11M still ~5× ClickHouse) |
| `distributed-and-scaling.md` | Single-node ceiling and horizontal-scale design of each. | drafted (pass 10: ClickHouse wins vertical single-node ceiling; GreptimeDB wins horizontal — region model + Metasrv rebalance + repartition + compute/storage separation vs ClickHouse OSS manual sharding (SharedMergeTree is Cloud-only); arch-reasoned, multi-node run owed) |
| `greptimedb-implementation.md` | Concrete Parallax-on-GreptimeDB design: full schema, ingest path, exact retrieval queries, object-storage/retention layout. | drafted (pass 12: full buildable DDL for all 8 signals — trace_id INVERTED INDEX (Run-1 fix), append_mode, FULLTEXT on message, metric engine + PromQL, JSON attrs, ttl/object-store; Q1–Q6 in dialect; standalone→cluster same schema. DDL syntax source-verified) |
| `clickhouse-implementation.md` | Concrete Parallax-on-ClickHouse design: full schema, ingest path, exact retrieval queries, object-storage/retention layout. | drafted (pass 13: full buildable DDL for all 8 signals — ORDER BY keys + per-column codecs (Gorilla/DoubleDelta/LowCardinality), native text index + bloom_filter for trace_id, JSON attrs, AggregatingMergeTree+MV for metrics, S3-disk TTL tiering; Q1–Q6; replaceability cost = OTLP collector + PromQL→SQL layer + manual sharding. async_insert/JSON/text-index source-verified) |
| `per-signal-verdict.md` | Scenario matrix: metrics vs logs vs traces vs evidence-bundle correlation. | drafted (pass 7: full matrix synthesizing passes 1-6 — ClickHouse leads logs/traces/anchored-bundle latency, GreptimeDB wins metrics/PromQL capability + ties metric agg; cost/scaling cells open; honest read = hypothesis not holding on raw latency, GreptimeDB's edge is metrics-native + object-store fit) |
| `benchmarking-the-differences.md` | Per-difference targeted benchmark design (hypothesis, workload, metric, pass/fail, prerequisites); routes runnable cases into the benchmark prototype. | drafted (pass 14: 11 targeted cases B1–B11 from all prior findings, prioritized; B2 trace-id-index runnable now, B1 cold GB–TB scan = the verdict flip-trigger; harness-gap list routed to the prototype) |
| `local-benchmark-results.md` | Empirical log of local Docker runs: env, pinned image tags, dataset, queries, measured numbers, and which published claim each run confirms or refutes. | drafted (pass 4 Run 1: spans smoke, parity PASS, trace-lookup schema asymmetry; pass 5 Run 2: evidence-bundle Q1/Q4 join parity PASS + EXPLAIN plans confirm PREWHERE/granule-skip + partitioned-hash + anchor-constant pushdown on both → join algo not a differentiator for anchored queries; pass 6 Run 3: metrics — PromQL-native on GreptimeDB vs absent on ClickHouse (capability gap), metric agg within 1.3× (16 vs 12 ms), float compression redo pending; bigger/cold tiers pending) |
| `public-performance-claims.md` | Method-#4 deliverable: public benchmark claims (ClickBench, JSONBench, vendor + independent) gathered and rated against code + local runs. | drafted (pass 22: claims triangulate with local runs — CH wins hot ingest/agg/log-search, GreptimeDB object-store-native + PromQL; KEY counterpoint = GreptimeDB #1 on ClickHouse's JSONBench cold-run at 1B docs, the regime closest to Parallax's cold re-reads — vendor-reported, to reproduce) |
| `verdict-which-to-choose.md` | Final synthesized decision and the mechanism-level reasoning. | drafted v1 (pass 11: recommends **GreptimeDB on FIT not speed** — hypothesis "fastest" refuted (ClickHouse faster on log/trace latency), GreptimeDB chosen for metrics-native + ingest/freshness ergonomics + horizontal-scaling + object-store + Rust; both replaceability answers + flip-trigger + benchmark veto questions; sharpen as impl/cold-scale runs land) |

## Source repositories (read, do not vendor into this repo)

- GreptimeDB (Rust): <https://github.com/GreptimeTeam/greptimedb>
- ClickHouse (C++): <https://github.com/ClickHouse/ClickHouse>
