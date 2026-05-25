# Storage Benchmark Artifact Interpretation

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Pass Target

Consume the separate benchmark agent's new artifacts without running another
storage benchmark. The goal is to decide what Runs 140-142 prove, what they
falsify, and which product/storage claims must stay unproven until the full
storage gates run.

## Artifacts Checked

| Artifact | Evidence class | What was inspected |
| --- | --- | --- |
| Commit `19e9604` | Reproducible local benchmark code + Run 140 docs | `bench/four-way/`, 1M-row default, `N >= 50000` enforcement, four builds, 20-query matrix. |
| Commit `1728da7` | Local 5M scale run interpretation | Run 141 docs showing anchored hot path holds while GreptimeDB heavy analytics cross the 300 ms gate. |
| Commit `ead9482` | Local A/B isolation | Run 142 docs showing GreptimeDB dedup aggregation vs append-mode aggregation at 5M. |
| `bench/four-way/gen.sh` | Reproducibility source | Generates six logical tables in-engine across all four builds; defaults `N=1000000`; rejects `N < 50000`; flushes GreptimeDB tables. |
| `bench/four-way/bench.sh` | Reproducibility source | Runs the 20-query matrix; `REPS` defaults to 6 and must be recorded per run when docs cite medians. |
| GitHub release APIs | Current release-track check | GreptimeDB latest GA remains `v1.0.2` published 2026-05-14; ClickHouse latest release endpoint currently returns LTS `v26.3.12.3-lts`, while the benchmarked feature-stable line remains `v26.5.1.882-stable`. |

## What The Artifacts Prove

1. **The benchmark is now reproducible enough to audit.** `bench/four-way/`
   stores the version matrix, table generation, data-size floor, and query set as
   code. This is a major improvement over ad hoc local numbers.
2. **At 1M warm local rows, every measured query is interactive on all four
   builds.** ClickHouse is still faster on most scans/joins/JSON/log-tail shapes;
   GreptimeDB wins or ties last-value, selective full-text, and high-cardinality
   exact distinct. This supports "fit not speed" for the anchored/local warm
   tier, not a production default.
3. **At 5M, the distinction matters.** GreptimeDB's anchored/keyed hot path still
   holds: anchored lookup, last-value, and time-range reads remain interactive.
   GreptimeDB's heavy analytical queries cross or approach the 300 ms gate:
   metric aggregations, dynamic JSON, in-DB cross-tier join, and high-card
   distinct. This turns the DQ5 flip trigger from theory into measured local
   evidence: analytics-heavy usage favors ClickHouse.
4. **GreptimeDB table mode is load-bearing.** Run 142 isolates dedup aggregation
   as roughly 8x slower than append-mode aggregation at 5M on the tested metric
   shape. For scrape-style metrics where `(series, ts)` is already unique,
   append mode is the safer default; dedup/`last_non_null` belongs only where
   partial upsert or out-of-order correction is actually needed.
5. **GreptimeDB `v1.1.0-nightly-20260525` is not a clean upgrade signal.** It is
   better on some append/scan paths but regresses the dedup aggregation path at
   5M. The only defensible future claim is "re-test v1.1 GA", not "v1.1 fixes
   GreptimeDB performance."

## What The Artifacts Do Not Prove

These runs do **not** satisfy the storage benchmark prototype, storage freshness
gate, storage cost gate, or A5 stack decision ledger:

- no 25-50 GB small-tier dataset;
- no cold-cache/drop-cache comparison under the full Q1-Q6 bundle workload;
- no native OTLP/Prometheus ingest path comparison for queryable freshness;
- no mixed ingest+query p95/p99, stale bundle rate, or Q6 p95 result rows;
- no S3/MinIO object-store request, egress, cache-size, or provider-cost rows;
- no ClickHouse LTS run, even though GitHub's latest ClickHouse release endpoint
  currently points to the LTS line;
- no metadata-store, ingest-log, setup, restart, redaction, or integration rows;
- no production hardware profile. These are local Docker warm-cache artifacts,
  with four containers sharing a host and different timing bases by engine.

Therefore, these artifacts can support `smoke_only` storage evidence and schema
decisions. They cannot produce `greptime_prototype_default`,
`clickhouse_storage_default`, `dual_storage_open`, or `phase1_stack_pass` in A5.

## Claim Updates

| Prior wording risk | Corrected wording |
| --- | --- |
| "GT-nightly has no regressions." | "GT-nightly has no regressions at the 1M warm tier, but Run 141/142 found a 5M dedup aggregation regression." |
| "Every query is below the 300 ms gate." | "Every 1M warm query is below the gate; at 5M, GreptimeDB heavy analytics cross the gate while anchored/keyed hot paths remain interactive." |
| "Metrics should use GreptimeDB dedup/metric mode by default." | "Use append mode for scrape-style unique metrics when aggregation is load-bearing; reserve dedup/`last_non_null` for true correction/upsert semantics." |
| "Benchmark confirms GreptimeDB as the default." | "Benchmark confirms GreptimeDB remains plausible for Parallax's anchored hot path, while ClickHouse is the fallback/default for analytics-heavy usage until the full gates decide." |

## Decision Impact

Keep GreptimeDB as the **prototype-fit candidate**, not as a measured stack
default. The new evidence strengthens both sides:

- GreptimeDB still fits Parallax's intended anchored evidence-bundle path.
- ClickHouse is clearly safer if Parallax drifts into ad hoc analytics, wide
  dynamic JSON, broad log search, or in-database cross-tier joins at scale.
- A storage adapter boundary remains mandatory because the measured flip trigger
  is now real, not hypothetical.
- Future storage docs should separate 1M local warm, 5M local warm, small-tier
  cold/object-store, and A5 stack-proof claims.

## Remaining Uncertainty

- The `bench/four-way` harness is valuable but narrower than the Rust
  `parallax-bench` prototype. It does not yet emit the JSONL result rows that A5
  expects.
- Run 142 used a local A/B table copy; it isolates the dedup path, but the real
  native metric engine / Prometheus remote-write path still needs a v1.1 GA
  re-test.
- Object-store economics and cold selective reads are still the cost decision's
  highest-risk gap.

## Next Evidence Gap

Run the full mixed-load Q6 freshness gate and the object-store cost gate before
turning these numbers into a storage default. The next storage-specific
falsification target is: **does GreptimeDB keep Q6 p95 under 300 ms under mixed
native ingest with cold/object-store reads, and does its object-count advantage
offset ClickHouse's scan efficiency?**

## Sources

- [Local benchmark results](greptimedb-vs-clickhouse/local-benchmark-results.md)
- [Four-way version comparison](greptimedb-vs-clickhouse/four-way-version-comparison.md)
- [Four-way benchmark harness](../../bench/four-way/README.md)
- [A5 stack decision ledger](a5-stack-decision-ledger.md)
- [Storage benchmark prototype](storage-benchmark-prototype.md)
- [Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md)
- [Storage size and object cost gate](storage-size-and-object-cost-gate.md)
- [GreptimeDB `v1.0.2` release](https://github.com/GreptimeTeam/greptimedb/releases/tag/v1.0.2)
- [ClickHouse `v26.5.1.882-stable` release](https://github.com/ClickHouse/ClickHouse/releases/tag/v26.5.1.882-stable)
- [ClickHouse `v26.3.12.3-lts` release](https://github.com/ClickHouse/ClickHouse/releases/tag/v26.3.12.3-lts)

## Bottom Line

Runs 140-142 made the storage evidence better and less comfortable. The anchored
Parallax hot path still supports the GreptimeDB fit thesis, but the 5M results
prove that analytics-heavy usage is a ClickHouse-shaped workload and that
GreptimeDB table mode can dominate version choice. Treat the new benchmark code
as strong smoke evidence and schema guidance, not as an A5 pass.
