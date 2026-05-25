# Storage Benchmark Artifact Interpretation

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Pass Target

Consume the separate benchmark agent's new artifacts without running another
storage benchmark. The goal is to decide what Runs 140-146 prove, what they
falsify, which source-read mechanism claims they strengthen, and which
product/storage claims must stay unproven until the full storage gates run.

## Artifacts Checked

| Artifact | Evidence class | What was inspected |
| --- | --- | --- |
| Commit `19e9604` | Reproducible local benchmark code + Run 140 docs | `bench/four-way/`, 1M-row default, `N >= 50000` enforcement, four builds, 20-query matrix. |
| Commit `1728da7` | Local 5M scale run interpretation | Run 141 docs showing anchored hot path holds while GreptimeDB heavy analytics cross the 300 ms gate. |
| Commit `ead9482` | Local A/B isolation | Run 142 docs showing GreptimeDB dedup aggregation vs append-mode aggregation at 5M. |
| Commit `f3a4023` | Benchmark tier policy + Run 143 docs | Local laptop default lowered to `N=100000`, 5M+ runs moved to explicit server tier, and forced compaction reduced the 5M dedup aggregation penalty. |
| Commit `20140c2` | Primary-source code read + Run 144 docs | GreptimeDB `v1.0.2` TWCS picker, window picker, and compactor source: compaction is time-window scoped and expired SSTs are removed separately from successful merges. |
| Commit `a6107e3` | Consolidation note | The storage verdict's DQ6 section now carries the 5M `v1.1.0-nightly-20260525` dedup-aggregation regression instead of preserving the stale "no regressions" wording from the 1M tier. |
| Commit `5d97084` | Local 100k preliminary validation | Run 145 docs showing the new `N=100000` local default completes without freezing the laptop and keeps all 20 four-way queries interactive, but compresses gaps enough that it is directional only. |
| Commit `72c6498` | Primary-source code read + Run 146 docs | GreptimeDB `v1.0.2` raft-engine WAL source: appends entries as `LogBatch`, forwards `sync_write` into the raft-engine write call, and runs a periodic sync task over `engine.sync()`. |
| `bench/four-way/gen.sh` | Reproducibility source | Generates six logical tables in-engine across all four builds; now defaults `N=100000`; rejects `N < 50000`; flushes GreptimeDB tables. |
| `bench/four-way/bench.sh` | Reproducibility source | Runs the 20-query matrix; `REPS` defaults to 6 and must be recorded per run when docs cite medians. |
| GreptimeDB TWCS source | Primary source | `TwcsPicker` groups files into compaction windows by max timestamp; `WindowedCompactionPicker` splits strict-window compaction by file time spans; the compactor removes expired SSTs even when merge output fails. |
| GreptimeDB raft-engine WAL source | Primary source | `RaftEngineLogStore` owns `sync_write`, `sync_period`, and the raft-engine handle; writes convert entries into a `LogBatch` and call `engine.write(&mut batch, self.sync_write)`, while `SyncWalTaskFunction` calls `engine.sync()`. |
| GitHub release APIs | Current release-track check | GreptimeDB latest GA remains `v1.0.2` published 2026-05-14; ClickHouse latest release endpoint currently returns LTS `v26.3.12.3-lts`, while the benchmarked feature-stable line remains `v26.5.1.882-stable`. |

## What The Artifacts Prove

1. **The benchmark is now reproducible enough to audit.** `bench/four-way/`
   stores the version matrix, table generation, data-size floor, and query set as
   code. This is a major improvement over ad hoc local numbers.
2. **Benchmark tier policy matters.** Local laptop runs are now a small but
   meaningful preliminary tier (`N=100000` default, `N >= 50000` enforced).
   Large `N=5000000+` runs are server-tier only and should be run only when the
   operator explicitly asks. Run 145 validates the local default operationally:
   generation finished in about 10 seconds, did not freeze the laptop, and left
   all 20 query shapes interactive across four builds.
3. **At 1M warm local rows, every measured query is interactive on all four
   builds.** ClickHouse is still faster on most scans/joins/JSON/log-tail shapes;
   GreptimeDB wins or ties last-value, selective full-text, and high-cardinality
   exact distinct. This supports "fit not speed" for the anchored/local warm
   tier, not a production default.
4. **At 100k warm local rows, the benchmark is safe but too compressed for
   magnitude claims.** Run 145 shows all 20 queries at 2-52 ms and nightlies
   roughly equal to stables. It confirms direction and harness health; it does
   not replace the 1M matrix or the 5M scale findings.
5. **At 5M, the distinction matters.** GreptimeDB's anchored/keyed hot path still
   holds: anchored lookup, last-value, and time-range reads remain interactive.
   GreptimeDB's heavy analytical queries cross or approach the 300 ms gate:
   metric aggregations, dynamic JSON, in-DB cross-tier join, and high-card
   distinct. This turns the DQ5 flip trigger from theory into measured local
   evidence: analytics-heavy usage favors ClickHouse.
6. **GreptimeDB table mode, compaction state, and TWCS window count are
   load-bearing.** Run 142 isolates dedup aggregation as roughly 8x slower than
   append-mode aggregation in the less-compacted 5M state; Run 143 shows forced
   compaction drops GreptimeDB stable's dedup aggregation from about 314 ms to
   about 60 ms, while append mode stays faster at about 40 ms and avoids
   compaction dependence. Run 144 makes the mechanism more precise: forced
   compaction can collapse within-window state, but a long-retention table still
   keeps at least one SST per TWCS window, so a dedup reader can still merge
   across windows. For scrape-style metrics where `(series, ts)` is already
   unique, append mode is still the safer load-bearing default; dedup/
   `last_non_null` belongs where partial upsert or out-of-order correction is
   actually needed.
7. **GreptimeDB `v1.1.0-nightly-20260525` is not a clean upgrade signal.** It is
   better on some append/scan paths but regresses the dedup aggregation path at
   5M. Run 143 makes the compaction-state sensitivity visible; Run 144 shows why
   many-window metric tables remain structurally different from a single compacted
   window; commit `a6107e3` carries that caveat into the storage verdict. The
   defensible future claim is "re-test v1.1 GA", not "v1.1 fixes GreptimeDB
   performance."
8. **GreptimeDB's cheap-retention claim is stronger than a benchmark number.**
   The Run 144 source read shows TWCS windows are the compaction boundary and the
   compactor includes expired SSTs in removals separately from successful merge
   outputs. That makes whole-SST TTL drop a structural GreptimeDB advantage,
   while object-store request counts and cold-read cost are still unmeasured for
   the full Parallax gate.
9. **GreptimeDB's strict-durability advantage is now source-grounded.** Run 75's
   measured delta said GreptimeDB `sync_write=true` was roughly 10x cheaper than
   ClickHouse per-part fsync on the local smoke path. Run 146 explains the
   mechanism in `v1.0.2`: GreptimeDB batches WAL entries into raft-engine
   `LogBatch` records, passes `sync_write` into `engine.write`, and also exposes
   `sync_period` as a periodic sync path. This supports the architectural claim
   that strict local durability fsyncs an append-log path rather than a
   multi-file part. It does not replace crash/restart testing or a mixed native
   ingest run.

## What The Artifacts Do Not Prove

These runs do **not** satisfy the storage benchmark prototype, storage freshness
gate, storage cost gate, or A5 stack decision ledger:

- no 25-50 GB small-tier dataset;
- no cold-cache/drop-cache comparison under the full Q1-Q6 bundle workload;
- no native OTLP/Prometheus ingest path comparison for queryable freshness;
- no mixed ingest+query p95/p99, stale bundle rate, or Q6 p95 result rows;
- no storage durability fault test showing acknowledged rows survive
  crash/restart under the declared GreptimeDB `sync_write`/`sync_period` mode
  or the declared ClickHouse fsync/replication mode;
- no S3/MinIO object-store request, egress, cache-size, or provider-cost rows;
- no ClickHouse LTS run, even though GitHub's latest ClickHouse release endpoint
  currently points to the LTS line;
- no metadata-store, ingest-log, setup, restart, redaction, or integration rows;
- no production hardware profile. Runs 140-143 and 145 are local Docker
  warm-cache artifacts, with four containers sharing a host and different timing
  bases by engine; Run 143 explicitly demotes large local runs and moves
  `N=5000000+` to an operator-requested server tier. Runs 144 and 146 are
  source-read mechanism evidence, not new timing, cost, or fault-injection runs.

Therefore, these artifacts can support `smoke_only` storage evidence and schema
decisions. They cannot produce `greptime_prototype_default`,
`clickhouse_storage_default`, `dual_storage_open`, or `phase1_stack_pass` in A5.

## Claim Updates

| Prior wording risk | Corrected wording |
| --- | --- |
| "GT-nightly has no regressions." | "GT-nightly has no regressions at the 1M warm tier, but Runs 141/142 found a 5M dedup aggregation regression; re-test v1.1 GA, compaction states, and many-window metric tables." |
| "Every query is below the 300 ms gate." | "Every 100k/1M warm query is below the gate; at 5M, GreptimeDB heavy analytics cross the gate while anchored/keyed hot paths remain interactive. Server-tier runs own future large absolute numbers." |
| "Metrics should use GreptimeDB dedup/metric mode by default." | "Use append mode for scrape-style unique metrics when aggregation is load-bearing; reserve dedup/`last_non_null` for true correction/upsert semantics, and measure compaction-state plus TWCS-window-count sensitivity." |
| "Strict durability is only a smoke timing claim." | "Run 75 timing is now source-grounded in GreptimeDB's append-log sync path, but A5 still needs a version-pinned durability-mode run that records `sync_write`, `sync_period`, ClickHouse fsync/replication settings, crash/restart loss counts, and mixed-ingest p95/p99." |
| "Benchmark confirms GreptimeDB as the default." | "Benchmark confirms GreptimeDB remains plausible for Parallax's anchored hot path, while ClickHouse is the fallback/default for analytics-heavy usage until the full gates decide." |

## Decision Impact

Keep GreptimeDB as the **prototype-fit candidate**, not as a measured stack
default. The new evidence strengthens both sides:

- GreptimeDB still fits Parallax's intended anchored evidence-bundle path.
- ClickHouse is clearly safer if Parallax drifts into ad hoc analytics, wide
  dynamic JSON, broad log search, or in-database cross-tier joins at scale.
- A storage adapter boundary remains mandatory because the measured flip trigger
  is now real, not hypothetical.
- GreptimeDB's durability fit is stronger for a no-loss-on-crash single-node
  profile because the source-backed strict path is an append-log sync, but A5
  must still pin durability settings and run crash/restart loss tests before the
  product can claim strict durability.
- Future storage docs should separate local preliminary (`N=100000` default),
  historical 1M/5M local warm artifacts, operator-requested server large-tier,
  small-tier cold/object-store, and A5 stack-proof claims. Run 145 belongs in
  the preliminary bucket, not the canonical comparison bucket.

## Remaining Uncertainty

- The `bench/four-way` harness is valuable but narrower than the Rust
  `parallax-bench` prototype. It does not yet emit the JSONL result rows that A5
  expects.
- Runs 142-144 isolate the dedup path, compaction sensitivity, and TWCS window
  mechanism, but the real native metric engine / Prometheus remote-write path
  still needs a v1.1 GA re-test on the server tier.
- Run 146 source-grounds the WAL sync path, but it does not prove the durability
  contract under process crash, OS crash, object-store mode, Kafka WAL mode, or
  mixed ingest with concurrent bundle queries.
- Object-store economics and cold selective reads are still the cost decision's
  highest-risk gap.

## Next Evidence Gap

Run the full mixed-load Q6 freshness gate and the object-store cost gate before
turning these numbers into a storage default. The next storage-specific
falsification target is: **does GreptimeDB keep Q6 p95 under 300 ms under mixed
native ingest with cold/object-store reads, while preserving acknowledged rows
under the declared durability mode, and does its object-count advantage offset
ClickHouse's scan efficiency?**

## Sources

- [Local benchmark results](greptimedb-vs-clickhouse/local-benchmark-results.md)
- [Four-way version comparison](greptimedb-vs-clickhouse/four-way-version-comparison.md)
- [Four-way benchmark harness](../../bench/four-way/README.md)
- [A5 stack decision ledger](a5-stack-decision-ledger.md)
- [Storage benchmark prototype](storage-benchmark-prototype.md)
- [Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md)
- [Storage size and object cost gate](storage-size-and-object-cost-gate.md)
- [GreptimeDB `v1.0.2` TWCS picker source](https://github.com/GreptimeTeam/greptimedb/blob/v1.0.2/src/mito2/src/compaction/twcs.rs)
- [GreptimeDB `v1.0.2` compactor source](https://github.com/GreptimeTeam/greptimedb/blob/v1.0.2/src/mito2/src/compaction/compactor.rs)
- [GreptimeDB `v1.0.2` raft-engine WAL log store source](https://github.com/GreptimeTeam/greptimedb/blob/v1.0.2/src/log-store/src/raft_engine/log_store.rs)
- [GreptimeDB `v1.0.2` raft-engine WAL config source](https://github.com/GreptimeTeam/greptimedb/blob/v1.0.2/src/common/wal/src/config/raft_engine.rs)
- [GreptimeDB `v1.0.2` Kafka WAL datanode config source](https://github.com/GreptimeTeam/greptimedb/blob/v1.0.2/src/common/wal/src/config/kafka/datanode.rs)
- [GreptimeDB `v1.0.2` release](https://github.com/GreptimeTeam/greptimedb/releases/tag/v1.0.2)
- [ClickHouse `v26.5.1.882-stable` release](https://github.com/ClickHouse/ClickHouse/releases/tag/v26.5.1.882-stable)
- [ClickHouse `v26.3.12.3-lts` release](https://github.com/ClickHouse/ClickHouse/releases/tag/v26.3.12.3-lts)

## Bottom Line

Runs 140-146 made the storage evidence better and less comfortable. The anchored
Parallax hot path still supports the GreptimeDB fit thesis, but the 5M results
prove that analytics-heavy usage is a ClickHouse-shaped workload and that
GreptimeDB table mode, compaction state, and TWCS window count can dominate
version choice. Run 145 confirms the laptop-safe smoke tier, but also reinforces
why small local timings are directional only. Runs 144 and 146 now source-ground
two GreptimeDB mechanism claims, TTL window drops and append-log durability, but
they are still mechanism evidence. Treat the new benchmark code and source-read
evidence as strong smoke/schema guidance, not as an A5 pass.
