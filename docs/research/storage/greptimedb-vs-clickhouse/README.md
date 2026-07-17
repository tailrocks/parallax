# GreptimeDB vs ClickHouse — Deep Internals Comparison

<!-- markdownlint-disable MD013 -->

White-box, source-level comparison of **GreptimeDB** and **ClickHouse** for the Parallax storage
layer: how each works internally, which design decisions make each fast or slow, and — for
Parallax's signals (metrics, logs, traces, anchored evidence-bundle correlation)
— what risks and upstream opportunities the comparison exposes.

> **Current authority:** GreptimeDB + Turso are mandatory; ClickHouse is a
> comparator only, never a fallback. The historical study selected GreptimeDB on
> fit + cost + Rust while measuring ClickHouse as the faster analytical engine.
> Read the one-page
> [`verdict-which-to-choose.md`](verdict-which-to-choose.md); the product decision is in
> [`../../decisions/storage-engine.md`](../../decisions/storage-engine.md); the full run-by-run
> history and detailed synthesis are in [`run-log.md`](run-log.md).

This sub-study is driven by the loop brief
[`prompts/greptimedb-vs-clickhouse-internals.md`](../../../../prompts/greptimedb-vs-clickhouse-internals.md).

## How this fits with the rest of storage research

This is the **white-box** layer — the *why* behind the *what* the other notes establish:

- [`../evaluation.md`](../evaluation.md) — strategy/fit evaluation (reasons *about* the systems).
- [`../benchmark-plan.md`](../benchmark-plan.md) — benchmark plan + runnable black-box harness (qualifies claims and risks; it cannot change the stack).
- [`../size-and-object-cost.md`](../size-and-object-cost.md) and [`../freshness-and-latency.md`](../freshness-and-latency.md) — the cost and latency proof gates.

A benchmark number the internals cannot explain is a flag that one of them is wrong.

## Version pins (re-check and bump every pass)

| System | Pinned version | Source commit | Notes |
| --- | --- | --- | --- |
| GreptimeDB stable | **`v1.1.3`** (GA 2026-07-17) | `63ef18a74a640135b983db6332226f90f9ae2b24` | **Run 173 bump** from `v1.0.2`. Do **not** pin `v1.1.0` alone — critical JSON upgrade bug; use ≥`v1.1.1`. |
| GreptimeDB nightly | **`v1.2.0-nightly-20260713`** | `c12f40cec232dda23429a0995d70bb4a230a562c` (reports `1.2.0`) | Rolls; re-bump dated tag each pass. |
| ClickHouse stable | **`v26.6.1.1193-stable`** | `840482cdca4e574927c1853900043b81d0687d00` | **Run 173 bump** from `v26.5.1.882`. Latest **feature** line (not LTS). `v26.5.5.8-stable` is a newer *patch* of the older 26.5 line. |
| ClickHouse nightly | **`clickhouse/clickhouse-server:head`** | reports **`26.7.1.1097`** | Rolls daily. |

*Prior pins preserved in run history:* GT `v1.0.2` / CH `v26.5.1.882` through Run 172.

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
- [`multi-tenancy-and-isolation.md`](multi-tenancy-and-isolation.md) — tenant isolation, RBAC, row policies, quotas, and proxy-owned auth.

**Per-signal, benchmarks, and public claims**
- [`per-signal-verdict.md`](per-signal-verdict.md) — scenario matrix: metrics vs logs vs traces vs evidence-bundle correlation.
- [`benchmarking-the-differences.md`](benchmarking-the-differences.md) — per-difference targeted benchmark design (B1–B15).
- [`local-benchmark-results.md`](local-benchmark-results.md) — empirical log of local Docker runs (env, pins, numbers).
- [`four-way-version-comparison.md`](four-way-version-comparison.md) — consolidated matrix: every load-bearing query × 4 builds.
- [`public-performance-claims.md`](public-performance-claims.md) — public benchmark claims rated against code + local runs.
- [`vendor-claims-audit.md`](vendor-claims-audit.md) — audit of GreptimeDB's own marketing/comparison pages.
- [`otel-arrow-ingest-assessment.md`](otel-arrow-ingest-assessment.md) — OTel-Arrow (OTAP) ingest assessment.

**Historical implementation studies and improvement assessment**
- [`greptimedb-implementation.md`](greptimedb-implementation.md) / [`clickhouse-implementation.md`](clickhouse-implementation.md) — comparative schema, ingest, query, and retention studies; ClickHouse is not a product target.
- [`platform-fit-and-alternatives.md`](platform-fit-and-alternatives.md) — proxy lens, alternatives survey, the metadata/error-grouping split.
- [`greptimedb-parity-roadmap.md`](greptimedb-parity-roadmap.md) — research assessment of possible GreptimeDB/upstream improvements, not an active Parallax implementation roadmap.

## Source repositories (read, do not vendor into this repo)

- GreptimeDB (Rust): <https://github.com/GreptimeTeam/greptimedb>
- ClickHouse (C++): <https://github.com/ClickHouse/ClickHouse>
