# Storage Size and Object Cost Gate

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

This is the proof gate for the remaining storage-cost claim:

> GreptimeDB versus ClickHouse storage size and object-storage cost.

The repo already has a retention model and local smoke size numbers. Those are
not enough to choose the storage default because Parallax's real cost depends on
per-signal compression, object count, object-store requests, cold-cache reads,
local cache size, compaction amplification, and egress when agent bundle queries
re-read history.

This gate turns the cost claim into a runnable benchmark extension for
[Storage benchmark prototype](storage-benchmark-prototype.md).

## Current Source Posture

As of 2026-05-25, primary docs support object-storage testing for both storage
candidates, but not a cost winner:

- GreptimeDB 1.0 configuration docs say standalone and datanode storage can use
  local files, AWS S3 and compatible services such as MinIO, Azure Blob Storage,
  and Aliyun OSS
  ([GreptimeDB configuration](https://docs.greptime.com/user-guide/deployments/configuration/)).
- ClickHouse docs say MergeTree and Log family engines can store data on S3 or
  Azure Blob Storage through external disks, and tables can use an S3
  `storage_policy` or `disk`
  ([ClickHouse external disks](https://clickhouse.com/docs/operations/storing-data)).
- ClickHouse release metadata checked on 2026-05-25 shows
  `v26.5.1.882-stable` as the newest feature-stable release and
  `v26.3.12.3-lts` as the newest LTS/latest release. ClickHouse production
  guidance says `stable` is recommended by default while `lts` fits teams that
  cannot upgrade frequently or have simpler secondary workloads
  ([ClickHouse stable release](https://github.com/ClickHouse/ClickHouse/releases/tag/v26.5.1.882-stable),
  [ClickHouse LTS release](https://github.com/ClickHouse/ClickHouse/releases/tag/v26.3.12.3-lts),
  [ClickHouse production version guidance](https://clickhouse.com/docs/faq/operations/production#how-to-choose-between-clickhouse-releases)).
- ClickHouse S3 docs distinguish table functions/engines from S3-backed
  MergeTree; the simple S3 table path lacks primary-index and cache support, so
  the benchmark must use S3-backed MergeTree for the real candidate comparison
  ([ClickHouse S3 integration](https://clickhouse.com/docs/integrations/s3)).
- ClickHouse compression docs say compression is data-distribution dependent and
  tunable through data types, codecs, ordering, and compression algorithms
  ([ClickHouse compression](https://clickhouse.com/docs/data-compression/compression-in-clickhouse)).
- Cloudflare R2 pricing lists Standard storage at `$0.015 / GB-month`,
  operation fees, and free egress
  ([Cloudflare R2 pricing](https://developers.cloudflare.com/r2/pricing/)).
- Backblaze B2 pricing lists free egress up to 3x average monthly storage and
  `$0.01/GB` after that unless traffic goes through listed CDN/compute partners
  ([Backblaze B2 pricing](https://www.backblaze.com/cloud-storage/pricing)).
- AWS S3 pricing separates storage, request/retrieval, and data-transfer costs;
  its own examples show internet data-transfer-out charges can materially exceed
  request costs for read-heavy usage
  ([AWS S3 pricing](https://aws.amazon.com/s3/pricing/)).

Conclusion: object storage is available for both candidate engines. The actual
decision is workload-specific and must be measured with identical generated
data, fair schema tuning, and provider-specific request/egress modeling.

## What Existing Research Already Shows

| Existing note | Useful evidence | Missing proof |
| --- | --- | --- |
| [Retention cost model](retention-cost-model.md) | Shows compressed object storage can make 30-90 day retention cheap, and that egress is the hidden read-heavy cost. | Uses assumed compression/request rates, not measured Parallax data. |
| [Local benchmark results](greptimedb-vs-clickhouse/local-benchmark-results.md) | Run 1 measured ClickHouse at 28.9 MiB and GreptimeDB SST at 38 MiB for 1M spans. | The schema comparison was unfair: ClickHouse had tuned codecs; GreptimeDB used defaults. No object storage or request counts. |
| [Storage benchmark prototype](storage-benchmark-prototype.md) | Defines retained size, compression, and object-store request metrics. | Does not yet spell out the cost gate, source pricing inputs, or failure consequences. |

The local size result is a warning, not a verdict. It says schema/codecs can
move cost enough to change the decision.

## Measurement Definitions

Record size and object-store metrics separately; do not collapse them into one
"GB stored" number:

| Metric | Meaning |
| --- | --- |
| `raw_bytes` | Generated uncompressed input bytes by signal and total. |
| `retained_local_bytes` | Candidate local retained bytes after flush/compaction, excluding transient WAL if it normally truncates. |
| `retained_object_bytes` | Object-store bytes by prefix after flush/compaction. |
| `metadata_bytes` | Candidate metadata, marks, indexes, manifests, and local object metadata. |
| `wal_or_queue_bytes` | Transient WAL/raft/queue bytes at end of run; report separately from retained bytes. |
| `compression_ratio` | `raw_bytes / retained_bytes`, by signal and blended. |
| `object_count` | Number of objects by prefix/table/signal. |
| `avg_object_size` | `retained_object_bytes / object_count`. |
| `put_count` | PUT, multipart upload, copy, and complete operations during ingest/compaction. |
| `get_count` | GET/head/read operations during warm and cold query phases. |
| `list_count` | LIST operations during ingest, compaction, restart, and query. |
| `object_read_bytes` | Bytes read from object storage during query and compaction. |
| `object_write_bytes` | Bytes written to object storage during ingest and compaction. |
| `cache_bytes` | Local SSD cache bytes required to hit warm-latency targets. |
| `egress_bytes` | Bytes that would leave the object-store provider/region in the deployment model. |

For provider cost projection, compute:

```text
monthly_cost =
  storage_gb_month * provider_storage_rate
+ class_a_or_put_requests * provider_write_request_rate
+ class_b_or_get_requests * provider_read_request_rate
+ retrieval_gb * provider_retrieval_rate
+ egress_gb * provider_egress_rate
```

Keep provider rates in each result file because pricing changes. The benchmark
must record the source URL and date for AWS S3, Cloudflare R2, and Backblaze B2.

## Workload Shape

Run the same `smoke` -> `small` -> `medium` progression as the storage benchmark,
but make cost visible after each phase:

| Phase | Required output |
| --- | --- |
| `load write-only` | object writes, object count, retained bytes, WAL bytes, compaction bytes. |
| `flush/compact` | final retained bytes, metadata bytes, object count, write amplification. |
| `query warm` | local-cache hit path, object GET count, bytes read, query latency. |
| `query cold` | restart/drop-cache path, object GET count, bytes read, query latency. |
| `mixed` | compaction/write amplification while Q1-Q6 queries run. |

Dataset requirements:

- realistic metric shapes: monotonic counters, flat gauges, sparse gauges, and
  histogram-like values;
- structured logs with both repetitive fields and high-entropy message bodies;
- spans/traces with high-cardinality IDs and repeated service/route attributes;
- error events with varied stacktraces and repeated frame/module structure;
- CLI and agent rows with bounded stdout/stderr excerpts plus object refs for
  larger payloads.

## Candidate-Specific Rules

### GreptimeDB

Run at least:

1. local file storage;
2. S3-compatible storage against MinIO;
3. the same S3-compatible layout projected to AWS S3, R2, and B2 pricing.

For each table family, record whether the table is append-only, which indexes
are enabled, and whether the schema is the documented baseline or an
anchor/cost-optimized variant. If index maintenance materially increases object
bytes or request counts, report that as part of the storage choice rather than a
separate concern.

### ClickHouse

Run at least:

1. local MergeTree storage;
2. S3-backed MergeTree using `storage_policy` or `disk` against MinIO;
3. the same S3-backed layout projected to AWS S3, R2, and B2 pricing.

Do not compare GreptimeDB object storage against ClickHouse's plain S3 table
function/engine path. The ClickHouse docs warn that simple S3 table paths lack
the primary-index and cache behavior needed for this workload. The storage-cost
comparison must use a real MergeTree storage policy.

Run two ClickHouse schema modes:

- **matched-effort mode:** reasonable LowCardinality, timestamp, and value
  codecs for Parallax columns;
- **default-effort mode:** minimal explicit codec tuning, so the result exposes
  whether operational simplicity changes the cost winner.

Run or explicitly exclude two ClickHouse release tracks:

- **feature-stable mode:** latest checked stable feature train, initially
  `v26.5.1.882-stable`;
- **LTS mode:** latest checked LTS train, initially `v26.3.12.3-lts`, when the
  storage decision is being evaluated for conservative self-hosted operators.

GreptimeDB gets the same fairness rule: do not compare ClickHouse tuned codecs
against unexamined GreptimeDB defaults and then call it an engine verdict.

## Pass Targets

Use these initial gates until measured runs justify calibration:

| Gate | Target |
| --- | --- |
| Cost comparison | GreptimeDB retained size plus modeled object-store cost is <= 1.2x ClickHouse on small tier, or GreptimeDB must win enough speed/operability to justify the premium. |
| Compression | Report by signal; blended ratio below 5x is a schema/data-shape failure requiring investigation. |
| Object fanout | Request costs under the 20 percent re-read model must stay below storage cost for R2/B2 and below 2x storage cost for S3. |
| Object size | Average retained object size below 8 MiB after compaction is a warning; below 1 MiB is a failure unless request costs remain negligible. |
| Cache dependency | Warm-query pass must record the local cache size needed; if cache cost exceeds object-store savings, local SSD may be the better tier. |
| Query/cost coupling | Any size/cost winner that fails the [storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md) cannot become the default. |

Provider-specific read-cost gates:

- R2 or B2 should be the recommended self-hosted object-store target if Parallax
  routinely re-reads more than 10 percent of retained data per month outside the
  storage provider's free/co-located path.
- S3 is acceptable when compute is co-located in the same AWS region and measured
  transfer costs are near zero; otherwise S3 egress must be explicitly shown in
  the deployment cost table.

## Result Record

Every result should include a cost section like:

```json
{
  "candidate": "clickhouse",
  "candidate_version": "26.5.1.882-stable",
  "release_track": "feature-stable|lts",
  "storage_mode": "s3_backed_mergetree",
  "schema_variant": "matched_effort",
  "dataset_tier": "small",
  "dataset_seed": 42,
  "raw_bytes": {"total": 0, "spans": 0, "logs": 0, "metrics": 0, "errors": 0},
  "retained_bytes": {
    "local": 0,
    "object": 0,
    "metadata": 0,
    "wal_or_queue": 0
  },
  "compression_ratio": {"blended": 0.0, "spans": 0.0, "logs": 0.0, "metrics": 0.0},
  "object_store": {
    "provider_model": "r2_standard",
    "object_count": 0,
    "avg_object_size": 0,
    "put_count": 0,
    "get_count_warm": 0,
    "get_count_cold": 0,
    "list_count": 0,
    "object_read_bytes": 0,
    "object_write_bytes": 0,
    "egress_bytes": 0
  },
  "monthly_cost_projection": {
    "storage": 0.0,
    "requests": 0.0,
    "retrieval": 0.0,
    "egress": 0.0,
    "total": 0.0,
    "pricing_source_date": "2026-05-25"
  }
}
```

The comparison table should be readable without JSON:

```text
candidate | storage_mode | schema | retained_GB | ratio | objects | avg_obj_MB | PUT_M | GET_M_cold | R2_$/mo | B2_$/mo | S3_$/mo | verdict
----------|--------------|--------|-------------|-------|---------|------------|-------|------------|---------|---------|---------|--------
greptime  | s3           | base   |             |       |         |            |       |            |         |         |         |
clickhouse| s3_mt        | tuned  |             |       |         |            |       |            |         |         |         |
```

## Decision Consequences

- If GreptimeDB is within the cost target and passes the speed gates, keep it as
  the v0.1 default.
- If GreptimeDB is more than 1.2x ClickHouse on retained size/object cost and
  does not clearly simplify operations, ClickHouse becomes the storage default
  unless metrics/PromQL value outweighs the premium.
- If both candidates are cheap only with large local caches, document a
  two-tier storage profile: local SSD hot window plus object storage for cold
  retention.
- If request costs dominate storage costs, increase compaction segment size,
  batch object writes, or keep the tiny/small deployment on local disk.
- If S3 egress dominates the monthly cost, default self-hosted docs to
  Cloudflare R2, Backblaze B2, or co-located AWS compute instead of generic S3.
- If the object-store run fails freshness or Q6 latency, object storage remains
  a cold-retention tier, not the hot query tier.

## Harness Additions

Add these to `parallax-bench` before quoting storage-cost numbers:

- MinIO access-log parsing for PUT/GET/LIST/object bytes;
- ClickHouse retained-byte collectors using `system.parts`, disk metadata path,
  and object prefix scan;
- GreptimeDB retained-byte collectors using SST/object prefix scan, metadata
  bytes, and WAL bytes;
- per-signal raw-byte accounting from the generator manifest;
- provider pricing table loaded from a versioned TOML file;
- local cache byte measurement for warm-object-store query runs;
- compaction amplification counters: bytes read/written after initial ingest;
- cost report that models 7, 30, and 90 day retention with 0, 10, 20, and 50
  percent monthly re-read rates.

## Related Research

- [Retention cost model](retention-cost-model.md)
- [Storage benchmark prototype](storage-benchmark-prototype.md)
- [Storage benchmark artifact interpretation](storage-benchmark-artifact-interpretation.md)
- [Storage freshness and bundle latency gate](storage-freshness-and-bundle-latency-gate.md)
- [Observability storage benchmark plan](observability-storage-benchmark-plan.md)
- [GreptimeDB storage evaluation](greptimedb-storage-evaluation.md)
- [GreptimeDB vs ClickHouse local benchmark results](greptimedb-vs-clickhouse/local-benchmark-results.md)
- [A5 stack decision ledger](a5-stack-decision-ledger.md) consumes this gate's
  retained-size, object-count, provider-pricing, and cache-dependency rows
  before any storage result can become a stack default.
