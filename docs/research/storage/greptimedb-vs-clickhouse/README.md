# GreptimeDB vs ClickHouse — Deep Internals Comparison

<!-- markdownlint-disable MD013 -->

White-box, source-level comparison of **GreptimeDB** and **ClickHouse** for the Parallax storage
layer: how each works internally, which design decisions make each fast or slow, and — for
Parallax's signals (metrics, logs, traces, anchored evidence-bundle correlation) — which to build
on, and why.

> **Conclusion:** current lean **GreptimeDB, not yet settled** (fit + cost + Rust; ClickHouse is
> the faster analytical engine but its lead is off Parallax's anchored hot path). Read the one-page
> [`verdict-which-to-choose.md`](verdict-which-to-choose.md); the product decision is in
> [`../../decisions/storage-engine.md`](../../decisions/storage-engine.md); the full run-by-run
> history and detailed synthesis are in [`run-log.md`](run-log.md).

This sub-study is driven by the loop brief
[`prompts/greptimedb-vs-clickhouse-internals.md`](../../../../prompts/greptimedb-vs-clickhouse-internals.md).

## How this fits with the rest of storage research

This is the **white-box** layer — the *why* behind the *what* the other notes establish:

- [`../evaluation.md`](../evaluation.md) — strategy/fit evaluation (reasons *about* the systems).
- [`../benchmark-plan.md`](../benchmark-plan.md) — the benchmark plan + runnable black-box harness (holds veto over the default).
- [`../size-and-object-cost.md`](../size-and-object-cost.md) and [`../freshness-and-latency.md`](../freshness-and-latency.md) — the cost and latency proof gates.

A benchmark number the internals cannot explain is a flag that one of them is wrong.

## Version pins (re-check and bump every pass)

| System | Pinned version | Source commit | Notes |
| --- | --- | --- | --- |
| GreptimeDB | `v1.0.2` (GA 2026-05-14) | `0ef54511f710f0ef2c05941c8c600bb4c1fd46c8` | Latest GA; `v1.1.0-nightly` exists but is not stable. |
| ClickHouse | `v26.5.1.882-stable` | commit `5b96a8d8a5e2f4800b43a780911a39dc5a666e1c` | Latest stable feature line; LTS line is `v26.3.12.3-lts`. |

## Method

- Compare the latest stable release of each system; record exact versions and the source commit SHA in every note.
- Orient on architecture docs, then confirm load-bearing claims against the cloned source (GreptimeDB Rust, ClickHouse C++); cite file:line. When docs and code disagree, trust the code.
- Every "X is faster" claim carries a *because* (mechanism) and a *scenario* (signal, query shape, cardinality, cache state, single-node vs scaled).
- Benchmarks run on all four builds (GT stable+nightly, CH stable+nightly) and update [`four-way-version-comparison.md`](four-way-version-comparison.md).

## Evaluation axes (priority order)

1. **Speed** — ingest-to-queryable freshness and evidence-bundle/correlation latency under concurrent ingest+query.
2. **Cost** — retained size/compression by signal, object-vs-local economics, compute per GB and per query class.
3. **Scaling** — single-node ceiling and horizontal scale-out (horizontal first; vertical-only is a flagged limitation).

## Note index (the evidence layer)

**Verdict and history**
- [`verdict-which-to-choose.md`](verdict-which-to-choose.md) — one-page current verdict (DQ1–DQ6 + flip rule).
- [`run-log.md`](run-log.md) — run-by-run status timeline, per-note status, and detailed verdict synthesis.
- [`open-questions-and-gaps.md`](open-questions-and-gaps.md) — gap ledger: what is NOT yet addressed, prioritized.

**Mechanism teardowns**
- [`greptimedb-internals.md`](greptimedb-internals.md) / [`clickhouse-internals.md`](clickhouse-internals.md) — architecture + code-path teardown of each engine.
- [`write-path-and-ingestion.md`](write-path-and-ingestion.md) — ingest → durable → queryable, and the freshness consequence.
- [`read-path-indexing-and-execution.md`](read-path-indexing-and-execution.md) — query planning, indexing, execution, scan-vs-skip, joins.
- [`query-execution-engine.md`](query-execution-engine.md) — CH C++ vectorized pipeline vs GT DataFusion-over-Arrow (the throughput gap).
- [`indexing-internals.md`](indexing-internals.md) — index file formats (GT Puffin sidecar vs CH per-part `.idx`).
- [`compaction-and-merge.md`](compaction-and-merge.md) — TWCS vs size-tiered merge; write amplification.
- [`caching-and-cold-warm.md`](caching-and-cold-warm.md) — cache hierarchies and the cold-vs-warm divergence.
- [`wal-and-durability.md`](wal-and-durability.md) — GT WAL (raft-engine/Kafka) vs CH no-WAL part-commit.
- [`dedup-and-update-semantics.md`](dedup-and-update-semantics.md) — read-time dedup vs `ReplacingMergeTree`.
- [`deletes-and-mutations.md`](deletes-and-mutations.md) — corrections / GDPR-erase / updates.
- [`schema-evolution-and-dynamic-columns.md`](schema-evolution-and-dynamic-columns.md) — OTLP attribute drift, ALTER cost, JSON storage.
- [`retention-and-ttl.md`](retention-and-ttl.md) — whole-file drop vs row rewrite.
- [`projections-and-access-paths.md`](projections-and-access-paths.md) — CH projections vs GT secondary indexes.
- [`metric-cardinality.md`](metric-cardinality.md) — high-cardinality metric storage and ingest.
- [`promql-and-metrics-query.md`](promql-and-metrics-query.md) — PromQL planning paths; the "no PromQL" drift correction.
- [`trace-span-tree.md`](trace-span-tree.md) — span-tree reconstruction (flat fetch vs recursive CTE).
- [`rollup-and-continuous-aggregation.md`](rollup-and-continuous-aggregation.md) — GT Flow vs CH MV + AggregatingMergeTree.
- [`compression-and-cost.md`](compression-and-cost.md) — layout, codecs, compression by signal, index cost.
- [`distributed-and-scaling.md`](distributed-and-scaling.md) — single-node ceiling and horizontal-scale design.
- [`storage-cost-and-tiering.md`](storage-cost-and-tiering.md) — CH performance/local-first vs GT S3-native/cost-first; hot/cold hybrid.

**Per-signal, benchmarks, and public claims**
- [`per-signal-verdict.md`](per-signal-verdict.md) — scenario matrix: metrics vs logs vs traces vs evidence-bundle correlation.
- [`benchmarking-the-differences.md`](benchmarking-the-differences.md) — per-difference targeted benchmark design (B1–B15).
- [`local-benchmark-results.md`](local-benchmark-results.md) — empirical log of local Docker runs (env, pins, numbers).
- [`four-way-version-comparison.md`](four-way-version-comparison.md) — consolidated matrix: every load-bearing query × 4 builds.
- [`public-performance-claims.md`](public-performance-claims.md) — public benchmark claims rated against code + local runs.
- [`vendor-claims-audit.md`](vendor-claims-audit.md) — audit of GreptimeDB's own marketing/comparison pages.
- [`otel-arrow-ingest-assessment.md`](otel-arrow-ingest-assessment.md) — OTel-Arrow (OTAP) ingest assessment.

**Implementation designs and roadmap**
- [`greptimedb-implementation.md`](greptimedb-implementation.md) / [`clickhouse-implementation.md`](clickhouse-implementation.md) — concrete Parallax-on-X design (schema, ingest, queries, retention).
- [`platform-fit-and-alternatives.md`](platform-fit-and-alternatives.md) — proxy lens, alternatives survey, the metadata/error-grouping split.
- [`greptimedb-parity-roadmap.md`](greptimedb-parity-roadmap.md) — per ClickHouse advantage, the borrowed concept → code change → effort tier → Parallax verdict.

## Source repositories (read, do not vendor into this repo)

- GreptimeDB (Rust): <https://github.com/GreptimeTeam/greptimedb>
- ClickHouse (C++): <https://github.com/ClickHouse/ClickHouse>
