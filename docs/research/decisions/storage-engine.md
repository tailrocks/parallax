# Storage Engine — GreptimeDB vs ClickHouse

<!-- markdownlint-disable MD013 -->

Decision date baseline: 2026-05-29 (reconciles the engine sub-study to the current operator brief).

> **Decision — current lean GreptimeDB, NOT yet settled.** Keep **both engines behind one
> `StorageAdapter`**; never hard-code engine magic into the schema or the evidence-bundle
> contract. ClickHouse is the fallback and the faster raw analytical engine. The lean is
> GreptimeDB because Parallax's hot path is *anchored* evidence-bundle retrieval (all signals
> for one `trace_id`/`fingerprint`), where **both engines are interactive (≪300 ms at every
> tested scale)** — so ClickHouse's scan-speed lead is off the hot path and the decision turns
> on **cost + Rust + self-hosted**, where GreptimeDB leads. This is finalized only when the
> sized cost numbers and the self-host-vs-managed-cloud call land (below).

This is the condensed current verdict. The **full record** — ~170 benchmark runs, a source-level
teardown of both engines, the four-build version matrix, and the per-pass history — lives in
[../storage/greptimedb-vs-clickhouse/](../storage/greptimedb-vs-clickhouse/) (start at
[verdict-which-to-choose.md](../storage/greptimedb-vs-clickhouse/verdict-which-to-choose.md);
history in [run-log.md](../storage/greptimedb-vs-clickhouse/run-log.md); cross-build matrix in
[four-way-version-comparison.md](../storage/greptimedb-vs-clickhouse/four-way-version-comparison.md)).

## Decision questions (DQ1–DQ6), in one table

| # | Question | Answer (mechanism-grounded) |
| --- | --- | --- |
| DQ1 | Where is **GreptimeDB** genuinely better? | Metrics/PromQL-native (GA + default-on); small-write/upsert ingest ergonomics (LSM, no "too many parts"); horizontal scale-out by design (region auto-rebalance, compute/storage separation, no bulk-copy migration); read-time dedup → correct latest-state on a plain query; OTLP schema-drift auto-adds typed columns; retention = whole-SST drop (cheap by default); object-storage-native (fewer objects → wins cold *full* scans); replayable WAL; cardinality-insensitive metric *ingest* (~flat 1k→1M series). |
| DQ2 | Where is **ClickHouse** genuinely better? | Selective log/trace scan + full-text; time-DESC log-tail locality; generic wide-scan/aggregate throughput (decade-tuned C++ vectorized engine, ~2–3× warm metric-agg); per-column codecs; dynamic-attribute JSON path queries (~8× with the required `.:Type` cast); projections (a 2nd physical order); in-DB anchored cross-tier joins; cold *selective* object-store reads (sparse-granule egress); schema-mistake tolerance. The gap **widens with scale** (5M+). |
| DQ3 | Can ClickHouse replace GreptimeDB? | **Yes, technically** — stored every signal, identical bundles — at the cost of a PromQL+OTLP compatibility layer (experimental/collector-only on CH), manual sharding (OSS `SharedMergeTree` is Cloud-only), and an ingest-batching layer. |
| DQ4 | Can GreptimeDB replace ClickHouse? | **Yes** — ran Q1–Q6 with identical results; accept slower heavy ad-hoc log/trace scans. Parallax's anchored hot path is **not latency-bound** (Q6 composite ≪300 ms on both). |
| DQ5 | Which to choose for Parallax today? | **GreptimeDB** on workload fit (metrics-native, ingest/upsert ergonomics, retention cost, scale-out) + the Rust tiebreak; ClickHouse's wins are real but less central to anchored retrieval. |
| DQ6 | Better long-term *investment*? | **GreptimeDB** — the speed gap is **closable engineering, not a physics wall** (seven of eight advantages are pure engineering; the two heaviest ride the shared **DataFusion** scan and **Parquet-Variant** JSON roadmaps), and it is the **Rust, open-source substrate the operator can contribute to** rather than wait on (C++). |

## Why the lean is GreptimeDB even though ClickHouse is faster

Two lenses once reached opposite defaults; the resolved query mix breaks the tie toward GreptimeDB:

- **Fit + long-term-investment lens → GreptimeDB.** Rust (operator-contributable), object-store-native
  cost, metrics/PromQL-native, scale-out by design; its speed deficits are closable on shared roadmaps.
- **Parallax-as-proxy lens → once leaned ClickHouse.** Because Parallax itself owns OTLP
  ingest/routing/conversion (operator architecture decision, 2026-05-25), GreptimeDB's native-ingest
  edge is neutralized, leaving retrieval speed + build-on-top ecosystem (SigNoz/Uptrace/HyperDX/
  ClickStack) — both ClickHouse wins.
- **The resolver — query mix is RESOLVED (operator 2026-05-29): anchored-bundle-retrieval-dominant.**
  The hot path fetches all signals for one `trace_id`/`fingerprint`/issue to assemble a bundle, not
  broad ad-hoc analytics. On that path **both engines are interactive at every tested scale**, so
  ClickHouse's raw-speed lead is **not decisive for Parallax**. The decision therefore turns on
  **cost + Rust**, where GreptimeDB leads — not on analytical-scan speed, where ClickHouse leads.

## What must close before this is settled

1. **Sized cost numbers on a real server tier** — $/GB retained, per-signal compression, and
   **multi-replica object-storage cost** (GreptimeDB 1× shared S3 vs OSS ClickHouse N× replica
   copies). The operator's #1 priority and the least-measured axis. Evidence:
   [../storage/size-and-object-cost.md](../storage/size-and-object-cost.md).
2. **Cold-read latency at GB–TB from object storage** — the one regime that could still surprise an
   anchored workload. Evidence: [../storage/freshness-and-latency.md](../storage/freshness-and-latency.md).
3. **Self-hosted vs managed cloud** — strictly self-hosted at scale favors GreptimeDB's 1× object copy
   + compute/storage separation; if ClickHouse Cloud (`SharedMergeTree`) is acceptable, that erases
   GreptimeDB's cost-economics edge.
4. **Re-test on GreptimeDB v1.1 GA** (expected Q2 2026 — narrows the dynamic-JSON gap and may move the
   metrics path; the v1.1 *nightly* is uneven, even regressing 5M dedup-aggregation). Re-pin and re-run
   the load-bearing speed/cost benchmarks when it ships.

## The flip rule (honest guardrail)

Absent a surprise in (1)–(2) or a "yes" to managed cloud in (3), the anchored workload + cost + Rust
point at **GreptimeDB**. **But** if the sized cost numbers come back **at parity** *and* a **managed
path is acceptable**, ClickHouse's ecosystem + speed make it the safer pick — let the **numbers**, not
the Rust preference, settle it. A secondary flip (from the sub-study): if the real query mix turns out
**analytics-/ad-hoc-scan-dominated** *and* GreptimeDB's cold-scan latency at GB–TB is materially worse,
ClickHouse's read-path advantage becomes central.

## Standing maintenance

- Keep both engines behind one `StorageAdapter` trait; no engine magic in the schema or bundle contract.
- Query mix is **resolved** (anchored-retrieval-dominant); the remaining finalizers are the sized cost
  numbers and the self-host-vs-managed-cloud call, not another query-shape model.
- Re-pin versions and re-verify load-bearing claims on each new stable release (GreptimeDB v1.1 GA next).

## Related records

- Stack roll-up that gates this becoming a stack default: [stack-decision.md](stack-decision.md) (A5).
- Relational metadata store (separate from the columnar engine): [metadata-store.md](metadata-store.md).
- Parity/closability analysis and alternatives survey:
  [../storage/greptimedb-vs-clickhouse/greptimedb-parity-roadmap.md](../storage/greptimedb-vs-clickhouse/greptimedb-parity-roadmap.md),
  [../storage/greptimedb-vs-clickhouse/platform-fit-and-alternatives.md](../storage/greptimedb-vs-clickhouse/platform-fit-and-alternatives.md).
