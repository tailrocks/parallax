# Distributed Model and Scaling (Axis #3)

<!-- markdownlint-disable MD013 -->

Status: pass 10. The scaling axis — single-node ceiling and **horizontal**
scale-out (the operator's primary scaling concern: startups on a tiny single node
that grow to big-company horizontal scale as a *topology change, not a rewrite*).
Mostly architecture-reasoned from source + RFCs; flagged where a real multi-node
run is still owed.

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`).

## Single-node ceiling (vertical)

| | GreptimeDB | ClickHouse |
| --- | --- | --- |
| Vertical scaling | Good — Rust, async, uses cores; LSM + object store. | **Excellent** — a decade of C++/SIMD tuning; saturates many cores and NVMe; famously high single-box throughput. |
| Tier-1 single node | `standalone` binary, metrics-native. | One `clickhouse-server`, plain MergeTree. |
| Verdict | Both fine for a tiny single-node startup. **ClickHouse has the higher vertical ceiling** on one big box (raw scan/aggregate throughput). | |

So at the *low* end both are trivial; at the *one-huge-box* end ClickHouse scales
up further. The decision is about what happens **when one box is not enough**.

## Horizontal scale-out — designed-in vs bolted-on

| Aspect | GreptimeDB | ClickHouse (OSS) |
| --- | --- | --- |
| Shard unit | **Region** (a table is partitioned into regions by partition rules — range/multi-dim, `src/partition/src` `splitter`, `multi_dim`). | **Shard** (a manually-defined cluster of nodes); table is `Distributed` over shards. |
| Who places shards | **Metasrv** auto-places regions on datanodes and can migrate them (RFC `2023-11-07-region-migration`). | **You do** — sharding key + cluster topology are operator-defined, static. |
| Grow past current size | **Repartition** splits/merges regions as data grows (RFC `2025-06-20-repartition`); Metasrv rebalances. | **Manual resharding** — no automatic cross-shard rebalance in OSS (only intra-node JBOD balancing, `MergeTreeData.h:1183`). Growing shard count = manual data redistribution. |
| Compute/storage separation | **Yes, practical**: object store (OpenDAL) + remote WAL (Kafka) make datanodes hold little durable local state → regions migrate cheaply, datanodes are near-elastic. | **OSS: no.** `SharedMergeTree` (elastic, compute/storage-separated, auto-scaling) is **ClickHouse Cloud proprietary — not in OSS** (confirmed: no `SharedMergeTree` in `src/Storages`). OSS is shared-nothing shard+replica. |
| Replication / consensus | Metasrv (Raft-based metadata) + region replication; remote WAL via Kafka. | `ReplicatedMergeTree` + **ClickHouse Keeper (Raft)** per replica set. |
| Read fan-out | Frontend (stateless, scale freely) → `MergeScanExec` fans sub-plans to region datanodes (`src/query/src/dist_plan`). | `Distributed` engine fans the query to shards, merges on the initiator. Read scaling across replicas via **`max_parallel_replicas`** — still **experimental** in 26.x (`allow_experimental_parallel_reading_from_replicas=0` default). |

## The decisive difference for Parallax's growth path

The operator's requirement is **small→large as a topology change, not a rewrite**:

- **GreptimeDB was designed multi-node from the start.** The region is the unit of
  distribution; Metasrv places, migrates, and (via repartition) splits regions as
  load grows. The *same schema* runs on `standalone` (one node) and on a
  Frontend/Datanode/Metasrv cluster — you add datanodes and the control plane
  rebalances. With object storage + remote WAL, datanodes approach stateless, so
  scale-out is closer to "add capacity" than "re-architect." **This is exactly the
  topology-change-not-rewrite property the operator wants.**
  - **Migration mechanism confirmed in source (pass 34):** the region-migration
    procedure (`src/meta-srv/src/procedure/region_migration/`) is **flush_leader →
    downgrade_leader → open_candidate → upgrade_candidate → close_downgraded** — a
    sequence with **no bulk data-transfer step** (its elapsed-time metrics track
    only flush/downgrade/open/upgrade). It flushes the source region to make its
    data durable in shared storage, then the target datanode **opens the same region
    from storage (manifest + SSTs)** and takes over leadership. So migration is
    **ownership reassignment + reopen, not a data copy** — **cheap specifically when
    backed by shared object storage** (the SSTs are already in S3; the target lazy-
    loads them). On local-disk-only storage this property weakens (the target must
    obtain the files). This is the load-bearing mechanism behind cheap rebalancing.
- **ClickHouse OSS scale-out is operator-driven and front-loaded.** You must choose
  the shard count and sharding key up front; there is no automatic resharding in
  OSS, so outgrowing the initial shard layout is a manual, disruptive data-movement
  exercise — closer to a re-architecture than a topology tweak. The elastic answer
  (`SharedMergeTree`) exists only in ClickHouse Cloud, which is **out of scope**
  (proprietary, and the project's self-hosted/open posture + language filter rule
  out depending on a closed cloud engine).

→ **Scaling axis (#3) verdict: GreptimeDB wins the horizontal-scale dimension** —
the operator's stated primary — because scale-out is designed-in (region model +
Metasrv rebalancing + repartition + compute/storage separation), whereas OSS
ClickHouse requires manual sharding decided up front with a painful resharding
wall. **ClickHouse wins the single-node vertical ceiling.** Since the operator
ranks **horizontal first** and vertical-only is a flagged limitation, this axis
favors GreptimeDB for Parallax's startups→big-companies trajectory.

## Honest caveats

- **This is architecture-reasoned, not cluster-measured.** All Docker runs so far
  are single-node smoke. The region-rebalancing smoothness, repartition behavior
  under load, and `MergeScanExec` fan-out latency at scale are **not yet measured**
  — a real multi-node run (both engines) is owed before this is benchmark-confirmed.
- **ClickHouse horizontal scale is proven in production** at enormous scale (many
  shards) — it is not that ClickHouse *can't* scale out, but that doing so is
  operator-managed and resharding is the known pain point. For a fixed, well-sized
  cluster ClickHouse scales horizontally fine.
- **GreptimeDB's distributed maturity is younger.** Region migration/repartition
  are recent (2025 RFCs); their robustness under production churn is less battle-
  tested than ClickHouse's shard model. Designed-in ≠ as-hardened.
- Both use Raft for consensus (Metasrv metadata vs Keeper) — neither has a
  consensus-model disadvantage.

## What still needs a real cluster run

1. **Multi-node read fan-out latency**: `MergeScanExec` (GreptimeDB) vs
   `Distributed` (ClickHouse) on the evidence-bundle queries, 3+ nodes.
2. **Rebalance behavior**: add a datanode to GreptimeDB and observe region
   migration; compare to the manual ClickHouse resharding effort.
3. **Scale-out throughput hold**: does p95 hold as nodes are added (the harness
   scaling protocol)? Owed to `storage-benchmark-prototype.md`.

## Scaling-axis roll-up

| Sub-axis | Winner | Mechanism | Confidence |
| --- | --- | --- | --- |
| Single-node vertical ceiling | ClickHouse | decade-tuned C++ scan/aggregate | arch |
| Horizontal scale-out design | **GreptimeDB** | region model + Metasrv placement/migration + repartition; compute/storage separation via object store + remote WAL | arch |
| Resharding / topology change | **GreptimeDB** | automatic region rebalance vs ClickHouse OSS manual resharding wall | arch |
| Elastic compute/storage separation | **GreptimeDB** (OSS) | OpenDAL + remote WAL; ClickHouse `SharedMergeTree` is Cloud-only | confirmed (OSS absence) |
| Read parallelism across replicas | ~tie | ClickHouse `max_parallel_replicas` (experimental) vs GreptimeDB frontend fan-out | arch |

## Source / evidence

- GreptimeDB: `src/partition/src/{splitter,multi_dim,manager}.rs`; **region-migration procedure `src/meta-srv/src/procedure/region_migration/` (flush_leader → downgrade → open_candidate → upgrade → close; no bulk-copy step)**; RFCs `2023-11-07-region-migration`, `2025-06-20-repartition`, `2025-07-23-global-gc-worker`; dist plan `src/query/src/dist_plan` (`MergeScanExec`); remote WAL `src/log-store/src/kafka`.
- ClickHouse: `src/Storages/StorageDistributed.cpp`, `src/Storages/StorageReplicatedMergeTree.cpp`, `src/Coordination/Keeper*`; `max_parallel_replicas`/`allow_experimental_parallel_reading_from_replicas` (`src/Core/Settings.cpp:7308`); **no** `SharedMergeTree` in `src/Storages` (Cloud-only).
