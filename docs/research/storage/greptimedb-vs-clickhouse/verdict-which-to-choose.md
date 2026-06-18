# Verdict — Which To Choose, And Why

<!-- markdownlint-disable MD013 -->

One-page current verdict. The full reasoning, ~170 benchmark runs, per-pass history, and the
detailed DQ tables with run citations are in [`run-log.md`](run-log.md). The product-level
decision and the "what must close to finalize" list are in
[`../../decisions/storage-engine.md`](../../decisions/storage-engine.md).

Pins (re-verify each pass): GreptimeDB `v1.0.2` (GA 2026-05-14, `0ef5451`); ClickHouse
`v26.5.1.882-stable` (`5b96a8d8`). v1.1.0 is nightly-only; 26.5.x is the latest stable *feature* line.
**Re-verified 2026-05-29 (GitHub releases): no drift — GreptimeDB v1.1 still nightly-only (no GA), ClickHouse 26.5 still the highest stable feature line.**

## Headline

> **⚠ V1 scope update (2026-06-18):** Parallax V1 has **decided GreptimeDB-only on the native OTLP
> model**; ClickHouse is deferred (not a V1 fallback or design constraint, revisit only on a concrete
> benefit). This document remains the white-box engine comparison and the future-option record — it is
> evidence, not the V1 decision. Canonical: [../../decisions/native-otel-tables.md](../../decisions/native-otel-tables.md).

**Recommended: GreptimeDB — current lean, not yet settled.** Not because it is the fastest
engine; it is not. ClickHouse is faster for high-volume log/trace analytics by clear,
code-confirmed mechanisms, and the gap *widens* with scale. GreptimeDB is recommended because its
design aligns with Parallax's dominant axes (metrics/PromQL-native, fresh-on-write small-write
ingest, horizontal scale-out by design, object-storage-native cost, Rust) **and** because
Parallax's hot path is *anchored* evidence-bundle retrieval, where both engines are interactive
(≪300 ms at every tested scale) — so ClickHouse's raw-speed lead is off the hot path. This is a
**fit + cost + investment decision, not a speed decision.**

Reconciliation note (2026-05-29): an intermediate "proxy lens" once tilted the default toward
ClickHouse (because Parallax owns OTLP ingest/routing/conversion, neutralizing GreptimeDB's
native-ingest edge, leaving retrieval speed + build-on-top ecosystem where ClickHouse wins). That
tilt is **re-weighted back to GreptimeDB** now that the query mix is resolved as
anchored-retrieval-dominant (operator, 2026-05-29): with the hot path interactive on both engines,
the decision turns on cost + Rust, where GreptimeDB leads. Keep both engines behind one
`StorageAdapter`; ClickHouse is the fallback.

## Decision questions (DQ1–DQ6)

| # | Question | Answer |
| --- | --- | --- |
| DQ1 | Where is **GreptimeDB** genuinely better? | Metrics/PromQL-native (GA + default-on); small-write/upsert ingest ergonomics (LSM, no part-explosion); horizontal scale-out by design (region auto-rebalance, no bulk-copy migration); read-time dedup → correct latest-state on a plain query; OTLP schema-drift auto-adds typed columns; retention = whole-SST drop; object-storage-native (fewer objects → wins cold full scans); replayable WAL; cardinality-insensitive metric *ingest*. |
| DQ2 | Where is **ClickHouse** genuinely better? | Selective log/trace scan + full-text; time-DESC log-tail locality; generic wide-scan/aggregate throughput (~2–3× warm metric-agg); per-column codecs; dynamic-attribute JSON path queries (~8× with the required `.:Type` cast); projections (a 2nd physical order); in-DB anchored cross-tier joins; cold *selective* object-store reads; schema-mistake tolerance. Gap **widens with scale** (5M+). |
| DQ3 | Can ClickHouse replace GreptimeDB? | **Yes, technically** — at the cost of a PromQL+OTLP compatibility layer (experimental/collector-only on CH), manual sharding (OSS `SharedMergeTree` is Cloud-only), and an ingest-batching layer. |
| DQ4 | Can GreptimeDB replace ClickHouse? | **Yes** — ran Q1–Q6 with identical results; accept slower heavy ad-hoc log/trace scans. The anchored hot path is **not latency-bound** (Q6 composite ≪300 ms on both). |
| DQ5 | Which to choose for Parallax today? | **GreptimeDB** on workload fit + the Rust tiebreak; ClickHouse's wins are real but less central to anchored retrieval. |
| DQ6 | Better long-term *investment*? | **GreptimeDB** — the speed gap is **closable engineering, not a physics wall** (7/8 advantages are pure engineering; the two heaviest ride the shared DataFusion scan and Parquet-Variant JSON roadmaps), and it is the **Rust substrate the operator can contribute to** rather than wait on (C++). |

## The flip rule

- **Primary (cost/cloud):** if the sized cost numbers come back **at parity** *and* a **managed
  cloud path is acceptable**, ClickHouse's ecosystem + speed make it the safer pick — let the
  numbers, not the Rust preference, settle it.
- **Secondary (workload):** if Parallax's real query mix turns out **analytics-/ad-hoc-scan-
  dominated** (not anchored bundle assembly) *and* GreptimeDB's cold-scan latency at GB–TB is
  materially worse, ClickHouse's read-path advantage becomes central and the choice flips —
  accepting the PromQL/OTLP layer as the cost of doing business.

## What must still close (handed to the benchmark, which holds veto)

1. **Sized cost numbers on a server tier** — $/GB retained, per-signal compression, and
   **multi-replica object-storage cost** (GreptimeDB 1× shared S3 vs OSS ClickHouse N× replicas).
   The operator's #1 priority; see [`../size-and-object-cost.md`](../size-and-object-cost.md).
2. **Cold-read latency at GB–TB from object storage** — see [`../freshness-and-latency.md`](../freshness-and-latency.md).
3. **Self-hosted vs managed-cloud** — ClickHouse Cloud (`SharedMergeTree`) would erase GreptimeDB's cost edge.
4. **Re-test on GreptimeDB v1.1 GA** (Q2 2026 — JSON Type v2 narrows the dynamic-attr gap; the v1.1
   nightly is uneven and even regresses 5M dedup-aggregation). Re-pin and re-run the load-bearing benchmarks.

The complete open-question ledger (#0–#8, with mechanism status) and the per-pass run history are in
[`run-log.md`](run-log.md) and [`open-questions-and-gaps.md`](open-questions-and-gaps.md); the
runnable harness ([`../benchmark-plan.md`](../benchmark-plan.md)) holds final veto.

## Supporting notes

Mechanism teardowns, per-signal matrix, benchmarks, implementation designs, the parity roadmap, and
the four-build version matrix are indexed in [`README.md`](README.md).
