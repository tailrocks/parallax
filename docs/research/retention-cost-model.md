# Retention Cost Model

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Purpose

The prompt's cost axis explicitly asks for "retention math: cost to keep N days
or weeks of real data," and the core belief is that cheap, durable retention is
"close to a requirement, because the value depends on being able to keep and
re-extract history without cost anxiety." The corpus argued this qualitatively;
this note quantifies it, and produces one non-obvious design finding: because
Parallax **re-reads** history to build agent context, **object-store egress
pricing matters as much as storage pricing**, which changes the recommended
backend.

Numbers are 2026 list prices; treat as order-of-magnitude, not quotes.

## Inputs

Object storage (per GB-month storage, plus egress):

| Backend | Storage $/GB-mo | Egress | Note |
| --- | --- | --- | --- |
| AWS S3 Standard | ~$0.023 | ~$0.09/GB | Egress is the killer for re-read-heavy workloads. |
| Cloudflare R2 | ~$0.015 | **$0** | Zero egress — ideal for re-extracting history. |
| Backblaze B2 | ~$0.006 | free up to 3× stored/mo, then ~$0.01/GB | Cheapest storage + generous egress. |

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
[storage benchmark prototype](storage-benchmark-prototype.md).

Sources: [Backblaze pricing comparison](https://www.backblaze.com/cloud-storage/pricing),
[Cloudflare R2 pricing](https://developers.cloudflare.com/r2/pricing/),
[S3 real cost analysis](https://leanopstech.com/blog/aws-s3-pricing-2026/),
[ClickHouse compression](https://clickhouse.com/docs/data-compression/compression-in-clickhouse),
[Observe per-GB pricing](https://www.observeinc.com/pricing).

## Worked Model

Stored/day = ingest/day ÷ 10 (compression). Retained = stored/day × days.

| Tier | Ingest/day (uncompressed) | Stored/day (~10×) | 30-day stored | 90-day stored |
| --- | --- | --- | --- | --- |
| Tiny (startup) | 2 GB | 0.2 GB | 6 GB | 18 GB |
| Small–mid team | 50 GB | 5 GB | 150 GB | 450 GB |
| Large | 1 TB | 100 GB | 3 TB | 9 TB |

Monthly **storage** cost at 90-day retention:

| Tier | 90-day stored | S3 | R2 | B2 |
| --- | --- | --- | --- | --- |
| Tiny | 18 GB | ~$0.41 | ~$0.27 | ~$0.11 |
| Small–mid | 450 GB | ~$10.35 | ~$6.75 | ~$2.70 |
| Large | 9 TB | ~$207 | ~$135 | ~$54 |

The headline: **90 days of real, mixed telemetry costs single-digit dollars per
month for a small team, and ~$50–200/month even at 1 TB/day ingest.** That is the
"keep history without cost anxiety" belief, quantified and true — on object
storage with good compression.

## The Egress Finding (Non-Obvious)

Parallax's value is **re-extracting** history to assemble evidence bundles. That
is read traffic, and if compute is not co-located with storage, reads become
egress. Model the small–mid tier re-reading 20% of retained data per month for
agent/human context (450 GB × 20% = 90 GB/mo):

| Backend | Storage/mo | Egress/mo (90 GB) | Total |
| --- | --- | --- | --- |
| S3 | ~$10.35 | ~$8.10 | ~$18.45 |
| R2 | ~$6.75 | $0 | ~$6.75 |
| B2 | ~$2.70 | free (under 3×) | ~$2.70 |

Egress nearly doubles the S3 bill and scales with how *useful* Parallax is (more
agent investigations = more reads). For a re-read-heavy context engine, **prefer
zero/low-egress object stores (R2, B2)** for self-hosted deployments, or
**co-locate query compute with the bucket** so reads never egress. This is a real
design input the qualitative docs missed: the cheaper-storage/cheaper-egress
providers are strictly better for *this* workload than S3, and the operator's
self-hosting ethos makes provider choice fully in their control.

## Contrast With Ingest-Priced SaaS (Quantifies The Wedge)

Cloud observability prices on **ingest**, not retention. At the small–mid tier
(50 GB/day = ~1,500 GB/month ingested):

- Observe-style logs at ~$0.49/GB ingested → ~$735/month for logs alone.
- SigNoz Cloud at ~$0.30/GB → ~$450/month.

Versus Parallax self-hosted retention at **single-digit dollars/month** for the
same data kept 90 days (compute extra, but on a cheap VM). The retention-cost
advantage is roughly **two orders of magnitude**. This is the concrete economic
core of the "self-hosted, no cost anxiety, keep everything" thesis and a direct
input to [business model and economics](business-model-and-economics.md): the
selling point is cost ownership, and the number is ~100× on retention.

Caveat: this compares Parallax *retention* cost to SaaS *ingest* pricing — not
apples-to-apples on features, and it excludes Parallax's own compute. But it is
exactly why a cost-conscious self-hoster defects from per-GB SaaS.

## Hidden Costs And Honest Caveats

- **Request costs.** Object stores charge per PUT/GET. Writing many tiny segments
  or reading many tiny objects can make requests dominate storage at low volume.
  Mitigate: batch into larger segments/parts; this is a real GreptimeDB/ClickHouse
  object-store tuning parameter, not free.
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

## Recommendation

- **Object-storage-first retention** is correct and cheap; it makes long
  retention a non-issue economically, which is the precondition for the AI-context
  value. Confirmed.
- For self-hosted deployments, **default to a zero/low-egress object store
  (Cloudflare R2 or Backblaze B2) or co-located compute**, not S3, because
  Parallax's read pattern turns S3 egress into a usage-scaling tax.
- **Tiny tier:** local disk (tens of GB at 90 days) — object storage is optional
  until volume or durability needs grow.
- Feed real per-signal compression ratios and request/egress counts from the
  [storage benchmark prototype](storage-benchmark-prototype.md) back into this
  model before quoting any number externally.

## Relationship To Other Research

- [Storage benchmark prototype](storage-benchmark-prototype.md) — measures the
  compression ratios, request counts, and object-store costs this model assumes.
- [Business model and economics](business-model-and-economics.md) — the ~100×
  retention-cost advantage is the cost-ownership selling point.
- [Technical implementation concept](technical-implementation-concept.md) — the
  GreptimeDB object-storage decision this quantifies.
- [Verdict](verdict.md) — the "cheap durable retention" precondition for the AI
  context thesis.

## Bottom Line

Keeping months of real telemetry is cheap — single-digit to low-hundreds of
dollars per month across tiny-to-large tiers on compressed object storage, ~100×
under ingest-priced SaaS. The one design correction this analysis forces:
because Parallax re-reads history to build context, choose a zero/low-egress
object store or co-locate compute, or S3 egress quietly taxes the product's own
usefulness.
