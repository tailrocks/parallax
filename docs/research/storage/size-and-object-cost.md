# Storage Size and Object-Store Cost

> Object storage is available for both candidate engines (GreptimeDB and ClickHouse), but the storage-cost winner is still an open, workload-specific gate: it must be measured with identical generated data, fair schema tuning (no tuned ClickHouse codecs vs. unexamined GreptimeDB defaults), and provider-specific request/egress modeling across AWS S3, Cloudflare R2, and Backblaze B2. What is already decided is the economics: compressed object-storage retention is cheap — single-digit to low-hundreds of dollars per month from the tiny to large tier at 90-day retention, roughly 100x (two orders of magnitude) under ingest-priced SaaS such as Observe (~$0.49/GB) or SigNoz Cloud (~$0.30/GB). The non-obvious design finding, also decided, is that because Parallax re-reads history to build agent context, object-store egress pricing matters as much as storage pricing, so self-hosted deployments should default to a zero/low-egress store (R2 or B2) or co-locate compute rather than generic S3. Still open and gated by runnable benchmark measurement: per-signal compression ratios on real Parallax data, object counts, PUT/GET/LIST request costs, cold-read bytes, compaction amplification, local cache size, and the resulting provider cost projection — plus the cost-comparison pass target (GreptimeDB retained size + modeled object cost <= 1.2x ClickHouse on the small tier, or a clear speed/operability win) and the coupling rule that any size/cost winner failing the storage freshness and bundle-latency gate cannot become the default. Local smoke numbers (ClickHouse 28.9 MiB vs. GreptimeDB SST 38 MiB for 1M spans) are a warning, not a verdict, because that schema comparison was unfair. Provider list prices are current as of 2026-05-25 and are order-of-magnitude planning inputs, not quotes.

This note consolidates the following previously-separate research files, each preserved in full below:

- `storage-size-and-object-cost-gate.md`
- `retention-cost-model.md`

## Storage Size and Object Cost Gate

_Provenance: merged verbatim from `storage-size-and-object-cost-gate.md` (2026-05-29 restructure)._

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

### Purpose

This is the proof gate for the remaining storage-cost claim:

> GreptimeDB versus ClickHouse storage size and object-storage cost.

The repo already has a retention model and local smoke size numbers. Those are
not enough to choose the storage default because Parallax's real cost depends on
per-signal compression, object count, object-store requests, cold-cache reads,
local cache size, compaction amplification, and egress when agent bundle queries
re-read history.

This gate turns the cost claim into a runnable benchmark extension for
[Storage benchmark prototype](benchmark-plan.md).

### Current Source Posture

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
- GreptimeDB `v1.0.2` TWCS/compactor source shows time-window scoped compaction
  and separate expired-SST removal. This supports the cheap whole-SST TTL
  mechanism, but it does not measure object counts, request costs, or cold reads
  for Parallax's full data mix
  ([TWCS picker source](https://github.com/GreptimeTeam/greptimedb/blob/v1.0.2/src/mito2/src/compaction/twcs.rs),
  [compactor source](https://github.com/GreptimeTeam/greptimedb/blob/v1.0.2/src/mito2/src/compaction/compactor.rs)).
- Cloudflare R2 pricing, last updated 2026-04-21 and checked 2026-05-25, lists
  Standard storage at `$0.015 / GB-month`, Class A operations at
  `$4.50 / million requests`, Class B operations at `$0.36 / million requests`,
  and free direct egress
  ([Cloudflare R2 pricing](https://developers.cloudflare.com/r2/pricing/)).
- Backblaze B2 pricing checked 2026-05-25 lists pay-as-you-go storage starting
  at `$6.95 / TB / month`, free transactions, free egress up to 3x average
  monthly storage, and `$0.01/GB` after that unless traffic goes through listed
  CDN/compute partners
  ([Backblaze B2 pricing](https://www.backblaze.com/cloud-storage/pricing)).
- AWS S3 pricing separates storage, request/retrieval, and data-transfer costs.
  The official AWS Price List API checked 2026-05-25 for `us-east-1` lists S3
  Standard first-50-TB storage at `$0.023 / GB-month`, PUT/COPY/POST/LIST at
  `$0.005 / 1,000`, GET/all-other requests at `$0.004 / 10,000`, and AWS data
  transfer out to the internet starting at `$0.09 / GB` for the first 10 TB
  beyond the global free tier
  ([AWS S3 pricing](https://aws.amazon.com/s3/pricing/),
  [Amazon S3 price list API](https://pricing.us-east-1.amazonaws.com/offers/v1.0/aws/AmazonS3/current/us-east-1/index.json),
  [AWS data-transfer price list API](https://pricing.us-east-1.amazonaws.com/offers/v1.0/aws/AWSDataTransfer/current/us-east-1/index.json)).

Conclusion: object storage is available for both candidate engines. The actual
decision is workload-specific and must be measured with identical generated
data, fair schema tuning, and provider-specific request/egress modeling.

### What Existing Research Already Shows

| Existing note | Useful evidence | Missing proof |
| --- | --- | --- |
| [Retention cost model](size-and-object-cost.md) | Shows compressed object storage can make 30-90 day retention cheap, and that egress is the hidden read-heavy cost. | Uses assumed compression/request rates, not measured Parallax data. |
| [Local benchmark results](greptimedb-vs-clickhouse/local-benchmark-results.md) | Run 1 measured ClickHouse at 28.9 MiB and GreptimeDB SST at 38 MiB for 1M spans. | The schema comparison was unfair: ClickHouse had tuned codecs; GreptimeDB used defaults. No object storage or request counts. |
| [Storage benchmark artifact interpretation](benchmark-plan.md) | Run 144 source-read confirms GreptimeDB's expired-SST removal path is structural under TWCS. | Does not measure provider request counts, cold-read bytes, compaction amplification, or ClickHouse tuned partition/drop behavior for the full Parallax dataset. |
| [Storage benchmark prototype](benchmark-plan.md) | Defines retained size, compression, and object-store request metrics. | Does not yet spell out the cost gate, source pricing inputs, or failure consequences. |

The local size result is a warning, not a verdict. It says schema/codecs can
move cost enough to change the decision.

### Measurement Definitions

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

### Workload Shape

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

### Candidate-Specific Rules

#### GreptimeDB

Run at least:

1. local file storage;
2. S3-compatible storage against MinIO;
3. the same S3-compatible layout projected to AWS S3, R2, and B2 pricing.

For each table family, record whether the table is append-only, which indexes
are enabled, and whether the schema is the documented baseline or an
anchor/cost-optimized variant. If index maintenance materially increases object
bytes or request counts, report that as part of the storage choice rather than a
separate concern.

#### ClickHouse

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

### Pass Targets

Use these initial gates until measured runs justify calibration:

| Gate | Target |
| --- | --- |
| Cost comparison | GreptimeDB retained size plus modeled object-store cost is <= 1.2x ClickHouse on small tier, or GreptimeDB must win enough speed/operability to justify the premium. |
| Compression | Report by signal; blended ratio below 5x is a schema/data-shape failure requiring investigation. |
| Object fanout | Request costs under the 20 percent re-read model must stay below storage cost for R2 and below 2x storage cost for S3; for B2's current free-transaction model, object fanout is still a latency and provider-portability warning. |
| Object size | Average retained object size below 8 MiB after compaction is a warning; below 1 MiB is a failure unless request costs remain negligible. |
| Cache dependency | Warm-query pass must record the local cache size needed; if cache cost exceeds object-store savings, local SSD may be the better tier. |
| Query/cost coupling | Any size/cost winner that fails the [storage freshness and bundle latency gate](freshness-and-latency.md) cannot become the default. |

Provider-specific read-cost gates:

- R2 or B2 should be the recommended self-hosted object-store target if Parallax
  routinely re-reads more than 10 percent of retained data per month outside the
  storage provider's free/co-located path. B2 currently advertises free
  transactions, so object fanout is mainly a latency/portability risk there, not
  a published request-fee risk.
- S3 is acceptable when compute is co-located in the same AWS region and measured
  transfer costs are near zero; otherwise S3 egress must be explicitly shown in
  the deployment cost table.

### Result Record

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

### Decision Consequences

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

### Harness Additions

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

### Related Research

- [Retention cost model](size-and-object-cost.md)
- [Storage benchmark prototype](benchmark-plan.md)
- [Storage benchmark artifact interpretation](benchmark-plan.md)
- [Storage freshness and bundle latency gate](freshness-and-latency.md)
- [Observability storage benchmark plan](benchmark-plan.md)
- [GreptimeDB storage evaluation](evaluation.md)
- [GreptimeDB vs ClickHouse local benchmark results](greptimedb-vs-clickhouse/local-benchmark-results.md)
- [A5 stack decision ledger](../decisions/stack-decision.md) consumes this gate's
  retained-size, object-count, provider-pricing, and cache-dependency rows
  before any storage result can become a stack default.

## Retention Cost Model

_Provenance: merged verbatim from `retention-cost-model.md` (2026-05-29 restructure)._

_(Shared note — see the Storage Size and Object Cost Gate section above.)_

Research date: 2026-05-25

### Purpose

The prompt's cost axis explicitly asks for "retention math: cost to keep N days
or weeks of real data," and the core belief is that cheap, durable retention is
"close to a requirement, because the value depends on being able to keep and
re-extract history without cost anxiety." The corpus argued this qualitatively;
this note quantifies it, and produces one non-obvious design finding: because
Parallax **re-reads** history to build agent context, **object-store egress
pricing matters as much as storage pricing**, which changes the recommended
backend.

Numbers are current list prices checked on 2026-05-25; treat as
order-of-magnitude planning inputs, not quotes.

### Inputs

Object storage (per GB-month storage, plus egress):

| Backend | Storage $/GB-mo | Egress | Note |
| --- | --- | --- | --- |
| AWS S3 Standard | ~$0.023 | ~$0.09/GB | Egress is the killer for re-read-heavy workloads. |
| Cloudflare R2 | ~$0.015 | **$0** | Zero egress — ideal for re-extracting history. |
| Backblaze B2 | ~$0.00695 | free up to 3× stored/mo, then ~$0.01/GB | Cheapest storage + generous egress; public page also says transactions are free. |

Compression (observability data, columnar + ZSTD + delta/Gorilla timestamps):

| Signal | Typical ratio | Note |
| --- | --- | --- |
| Metrics | 10–50× | Delta/Gorilla on regular series compress extremely well. |
| Logs | 5–12× | ZSTD(3) on structured logs; repetitive fields help. |
| Traces/spans | 5–10× | Many short attributes; trace/span IDs compress modestly. |
| Error events | 3–6× | Stacktraces/messages are text-heavy and varied. |

Blended assumption used below: **~10× compression** across a mixed
log/trace/metric/error workload. ZSTD(1) gives 3–4×, ZSTD(3) 4–5×, ZSTD(9) 6–8×
on generic data; observability columns with delta encodings push the blended
figure higher. Calibrate per real data in the
[storage benchmark prototype](benchmark-plan.md).

Sources:
[Backblaze B2 pricing](https://www.backblaze.com/cloud-storage/pricing),
[Cloudflare R2 pricing](https://developers.cloudflare.com/r2/pricing/),
[AWS S3 pricing](https://aws.amazon.com/s3/pricing/),
[Amazon S3 price list API](https://pricing.us-east-1.amazonaws.com/offers/v1.0/aws/AmazonS3/current/us-east-1/index.json),
[AWS data-transfer price list API](https://pricing.us-east-1.amazonaws.com/offers/v1.0/aws/AWSDataTransfer/current/us-east-1/index.json),
[ClickHouse compression](https://clickhouse.com/docs/data-compression/compression-in-clickhouse),
[Observe per-GB pricing](https://www.observeinc.com/pricing).

### Worked Model

Stored/day = ingest/day ÷ 10 (compression). Retained = stored/day × days.

| Tier | Ingest/day (uncompressed) | Stored/day (~10×) | 30-day stored | 90-day stored |
| --- | --- | --- | --- | --- |
| Tiny (startup) | 2 GB | 0.2 GB | 6 GB | 18 GB |
| Small–mid team | 50 GB | 5 GB | 150 GB | 450 GB |
| Large | 1 TB | 100 GB | 3 TB | 9 TB |

Monthly **storage** cost at 90-day retention:

| Tier | 90-day stored | S3 | R2 | B2 |
| --- | --- | --- | --- | --- |
| Tiny | 18 GB | ~$0.41 | ~$0.27 | ~$0.13 |
| Small–mid | 450 GB | ~$10.35 | ~$6.75 | ~$3.13 |
| Large | 9 TB | ~$207 | ~$135 | ~$62.55 |

The headline: **90 days of real, mixed telemetry costs single-digit to low-tens
of dollars per month for a small team, and ~$60–200/month even at 1 TB/day
ingest.** That is the "keep history without cost anxiety" belief, quantified
and true — on object storage with good compression.

### The Egress Finding (Non-Obvious)

Parallax's value is **re-extracting** history to assemble evidence bundles. That
is read traffic, and if compute is not co-located with storage, reads become
egress. Model the small–mid tier re-reading 20% of retained data per month for
agent/human context (450 GB × 20% = 90 GB/mo):

| Backend | Storage/mo | Egress/mo (90 GB) | Total |
| --- | --- | --- | --- |
| S3 | ~$10.35 | ~$8.10 | ~$18.45 |
| R2 | ~$6.75 | $0 | ~$6.75 |
| B2 | ~$3.13 | free (under 3×) | ~$3.13 |

Egress nearly doubles the S3 bill and scales with how *useful* Parallax is (more
agent investigations = more reads). For a re-read-heavy context engine, **prefer
zero/low-egress object stores (R2, B2)** for self-hosted deployments, or
**co-locate query compute with the bucket** so reads never egress. This is a real
design input the qualitative docs missed: the cheaper-storage/cheaper-egress
providers are strictly better for *this* workload than S3, and the operator's
self-hosting ethos makes provider choice fully in their control.

### Contrast With Ingest-Priced SaaS (Quantifies The Wedge)

Cloud observability prices on **ingest**, not retention. At the small–mid tier
(50 GB/day = ~1,500 GB/month ingested):

- Observe-style logs at ~$0.49/GB ingested → ~$735/month for logs alone.
- SigNoz Cloud at ~$0.30/GB → ~$450/month.

Versus Parallax self-hosted retention at **single-digit dollars/month** for the
same data kept 90 days (compute extra, but on a cheap VM). The retention-cost
advantage is roughly **two orders of magnitude**. This is the concrete economic
core of the "self-hosted, no cost anxiety, keep everything" thesis and a direct
input to [business model and economics](../validation/business-model.md): the
selling point is cost ownership, and the number is ~100× on retention.

Caveat: this compares Parallax *retention* cost to SaaS *ingest* pricing — not
apples-to-apples on features, and it excludes Parallax's own compute. But it is
exactly why a cost-conscious self-hoster defects from per-GB SaaS.

### Hidden Costs And Honest Caveats

- **Request costs.** R2 and S3 charge per PUT/GET-class operation; B2's current
  public pay-as-you-go page says transactions are free. Writing many tiny
  segments or reading many tiny objects can still make requests dominate storage
  on R2/S3 and can hurt latency/provider portability even on B2. Mitigate: batch
  into larger segments/parts; this is a real GreptimeDB/ClickHouse object-store
  tuning parameter, not free.
- **Compute is not counted here.** Ingest, compaction, and query CPU/RAM run on a
  VM whose cost is separate. At the tiny/small tiers this is one cheap box; the
  storage math is the easy part.
- **Compaction read/write amplification.** Background compaction re-reads and
  re-writes data; on S3 that is more requests (and egress if cross-region).
- **Compression ratio is workload-specific.** 10× is a planning assumption;
  high-cardinality attribute-heavy traces compress worse, regular metrics far
  better. The benchmark must measure per-signal ratios on real data.
- **Hot cache.** Acceptable query latency on object storage usually needs a local
  SSD cache; that cache is a cost and a freshness/latency factor (the storage
  benchmark's cold-vs-hot axis).

### Recommendation

- **Object-storage-first retention** is correct and cheap; it makes long
  retention a non-issue economically, which is the precondition for the AI-context
  value. Confirmed.
- For self-hosted deployments, **default to a zero/low-egress object store
  (Cloudflare R2 or Backblaze B2) or co-located compute**, not S3, because
  Parallax's read pattern turns S3 egress into a usage-scaling tax.
- **Tiny tier:** local disk (tens of GB at 90 days) — object storage is optional
  until volume or durability needs grow.
- Feed real per-signal compression ratios and request/egress counts from the
  [storage size and object cost gate](size-and-object-cost.md) back
  into this model before quoting any number externally.

### Relationship To Other Research

- [Storage benchmark prototype](benchmark-plan.md) — measures the
  compression ratios, request counts, and object-store costs this model assumes.
- [Storage size and object cost gate](size-and-object-cost.md) —
  specifies the pass/fail gate and provider-cost projection for those
  measurements.
- [Business model and economics](../validation/business-model.md) — the ~100×
  retention-cost advantage is the cost-ownership selling point.
- [Technical implementation concept](../architecture/implementation-concept.md) — the
  GreptimeDB object-storage decision this quantifies.
- [Verdict](../decisions/go-no-go.md) — the "cheap durable retention" precondition for the AI
  context thesis.

### Bottom Line

Keeping months of real telemetry is cheap — single-digit to low-hundreds of
dollars per month across tiny-to-large tiers on compressed object storage, still
roughly two orders of magnitude under ingest-priced SaaS. The one design
correction this analysis forces:
because Parallax re-reads history to build context, choose a zero/low-egress
object store or co-locate compute, or S3 egress quietly taxes the product's own
usefulness.
