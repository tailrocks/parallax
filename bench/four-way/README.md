# Four-Way Benchmark Harness (reproducible)

Stored-as-code benchmark for the GreptimeDB-vs-ClickHouse comparison, run on **all four builds** per
the [AGENTS.md](../../AGENTS.md) "Benchmarking Rule":

1. **GreptimeDB stable** (`greptimedb`, :4000) — `greptime/greptimedb:v1.0.2`
2. **GreptimeDB nightly** (`greptimedb-nightly`, :4100) — `greptime/greptimedb:v1.1.0-nightly-…`
3. **ClickHouse stable, non-LTS** (`clickhouse`, :8123) — `clickhouse/clickhouse-server:26.5.1.882`
4. **ClickHouse nightly** (`clickhouse-head`, :8124) — `clickhouse/clickhouse-server:head`

This is the **preliminary local check**; the same scripts re-run on a sized server later. Everything
is code — no ad-hoc out-of-the-box generation — so what was tested, how the data was spawned, and how
to reproduce is all here.

## Run it

```bash
docker compose -f bench/compose.yml up -d        # bring up all four builds (wait for healthy)
bench/four-way/gen.sh                            # generate identical data on all four (N=100,000 default)
bench/four-way/bench.sh                           # print the 4-way matrix (median of 6 warm reps)
docker compose -f bench/compose.yml down -v      # tear down
```

Tunables (env): `N` = rows per table (gen.sh; **minimum 50,000**, default **100,000**); `REPS` =
warm reps per query (bench.sh, default 6). Container-name overrides: `GT_STABLE`, `GT_NIGHTLY`,
`CH_STABLE`, `CH_HEAD` (default to the compose names).

## Data-size policy — LOCAL small (preliminary), SERVER large (detailed)

Two tiers:
- **LOCAL (laptop): small but meaningful — default `N=100,000`** (≥50,000 enforced). This is a
  **preliminary comparison only**. Running big `N` (millions) with 4 DB containers **freezes a
  MacBook** — don't. Keep local runs small; tear the nightly containers down between runs.
- **SERVER: large, detailed — `N=5,000,000+`** (`N=5000000 bench/four-way/gen.sh`). The proper
  performance test runs on a server with headroom, not the dev laptop.

`gen.sh` enforces `N >= 50,000` (no toy benchmarks) and builds all six tables at the same `N` for an
internally consistent matrix. Absolute ms at the small local tier are fixed-overhead-dominated — the
**cross-build ratios** are the preliminary signal; trust absolute numbers from the server tier.

**Laptop caution:** do NOT keep all four containers standing with big data on a laptop. For a local
check: `docker start` the nightlies → `gen.sh` (small) → `bench.sh` → `docker stop` the nightlies.

## What's generated (`gen.sh`) — natively, no CSV transport

Data is generated in-engine via GreptimeDB `range()` / ClickHouse `numbers()` (identical logical
data on all four). Six tables modelling Parallax's signals:

| Table | Models | GreptimeDB layout | ClickHouse layout |
| --- | --- | --- | --- |
| `spans1m` | spans (events) | `PK(service)` + `trace_id` INVERTED + `append_mode` | `ORDER BY (trace_id, ts)` |
| `m2m` | metrics | `PK(service,instance)` (dedup) | `ORDER BY (service,instance)` |
| `logs1m` | logs | `PK(service)` + `message` FULLTEXT(bloom) + `append_mode` | `ORDER BY (service,ts)` + `tokenbf_v1` index |
| `sj` | dynamic-attr JSON | `JSON` + `append_mode` | `JSON` |
| `errs` | errors (join side) | `PK(service)` + `trace_id` INVERTED + `append_mode` | `ORDER BY (trace_id, ts)` |
| `tsr` | time-range scan | `TIME INDEX ts` + `append_mode` | `ORDER BY ts` |

GreptimeDB tables are flushed after load (`ADMIN flush_table`) so reads come from SST (settled state),
not the memtable. Schema choices follow the blueprint rules (low-card PK + append_mode for event
signals — `greptimedb-implementation.md` principles 2/3; Runs 114/117).

## What's measured (`bench.sh`) — 20 queries

Every query in [`../../docs/research/storage/greptimedb-vs-clickhouse/four-way-version-comparison.md`](../../docs/research/storage/greptimedb-vs-clickhouse/four-way-version-comparison.md): anchored lookup, unindexed scan, TopK,
trace-explorer, high-group agg, count-distinct (low/high card), latency histogram, metric-agg
(flat/bucketed/rate/last-value/p99), full-text (selective/broad), log-tail, issue-list, dynamic-attr
JSON, cross-tier join, time-range scan. Per-engine SQL where dialects differ (`count(*)`/`count()`,
`date_bin`/`toStartOfInterval`, `last_value`/`argMax`, `approx_percentile_cont`/`quantile`,
`matches_term`/`hasToken`, `json_get_int`/`.:Int64`). Output = median ms per (query × build); update
the comparison doc from it.

## Notes / caveats

- **Ingest path:** `gen.sh` uses `INSERT…SELECT` (synthetic) — NOT the native OTLP/gRPC bulk path.
  For ingest-throughput numbers use the native protocols; this harness measures query latency on a
  fixed dataset.
- **Full-text broad-term** at `N=1M` is ~1.2–1.5× (this corpus); the canonical broad-term gap is
  ~12× at 5M on `logs_b1` (Run 133) — bump `N` to widen.
- **Nightly tags roll daily.** Bump the `*-nightly` / `:head` tags in `compose.yml` and re-run
  `gen.sh` to re-pin.
- The `bench/s3/` sibling holds the object-storage (MinIO) stack for cold-read / cost passes.
