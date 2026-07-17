# Server-Tier Benchmark Runbook (owed measurements)

<!-- markdownlint-disable MD013 -->

Status: **Run 240 (2026-07-17)** — harness-handoff for measurements that **must not
run on a laptop** (operator rule: N≥1M/5M freezes four containers on Mac).
Laptop work (Runs 173–239) saturated smoke; this note is the **copy-paste contract**
for the next server session.

Pins (re-check before run): GT **`v1.1.3`** + nightly **`v1.2.0-nightly-*`**, CH
**`26.6.1.1193`** + **`head`** — see comparison README.

## Always four builds

Every number lands in [`four-way-version-comparison.md`](four-way-version-comparison.md)
with all four columns + Faster + link to this run in
[`local-benchmark-results.md`](local-benchmark-results.md) (or a `server-` prefix
log if preferred).

## Tier A — N=1,000,000 (minimum server meaningful)

```bash
cd bench
docker compose -f compose.yml pull
docker compose -f compose.yml up -d
# wait healthy x4
N=1000000 bash four-way/gen.sh
REPS=8 bash four-way/bench.sh | tee /tmp/four-way-1m.txt
# paste matrix into four-way-version-comparison.md + local-benchmark-results
```

**Must capture:** full harness rows especially:

- high-group-agg / count-distinct
- metric-agg-flat / counter-rate-panel / **last-value**
- fulltext-broad vs selective
- dynamic-attr-jsonb vs **json2**
- cross-tier-join

## Tier B — N=5,000,000 (dedup-agg + scan magnitude)

Only after Tier A. Same harness if tables fit RAM; else nightlies sequential
start/stop:

```bash
# optional: only stable pair first, then nightly pair
N=5000000 bash four-way/gen.sh   # may need larger disk
REPS=5 bash four-way/bench.sh
```

**Priority retests:**

1. Dedup-agg / last-value shape (pre-GA nightly regression history)
2. Broad full-text scan ratio
3. Storage size per signal (`compression-and-cost.md`)

## Tier C — MinIO cold GB

```bash
bench/s3/run-s3-stack.sh up   # pins must match compose
# load ≥1M spans (or multi-GB logs)
# force cold: GT rm -rf /greptimedb_data/cache/* && restart
# CH: SYSTEM DROP FILESYSTEM CACHE; DROP MARK CACHE; DROP UNCOMPRESSED CACHE
# delta GT: opendal_http_*GetObject on /metrics
# delta CH: system.events S3GetObject + ReadBufferFromS3Bytes
```

See [`caching-and-cold-warm.md`](caching-and-cold-warm.md) recipe (Run 238).

## Product inputs still blocking flip-rule close

1. Fill shares in [`workload-mix-decision-input.md`](workload-mix-decision-input.md)
2. Vendor trial quotes (list rates in [`managed-cloud-vs-self-host.md`](managed-cloud-vs-self-host.md) Run 221)
3. RPO D2/D3 drills ([`product-rpo-runbook.md`](product-rpo-runbook.md))

## Not done criteria

This runbook existing ≠ server work done. Comparison stays open until operator
stops the research loop.
