# Four-Way Version Comparison — Every Performance Measurement Across All 4 Builds

<!-- markdownlint-disable MD013 -->

Status: created 2026-05-25 (Run 131). The operator asked for **one clear comparison**: how fast is
each load-bearing query on **all four builds** —
- **GT-stable** = GreptimeDB **v1.0.2** (latest stable, production-OK)
- **GT-nightly** = GreptimeDB **v1.1.0-nightly-20260525** (latest nightly, unreleased)
- **CH-stable** = ClickHouse **v26.5.1.882** (latest stable feature line, non-LTS)
- **CH-head** = ClickHouse **v26.6.1.127** (`clickhouse:head`, the unreleased nightly)

**Method (no-tricks, reproducible).** Two fresh standalone containers (GT-nightly :4100, CH-head
:8124) ran alongside the v1.0.2 / 26.5 bench. **Identical data on all four**, generated natively via
`range()` (GT) / `numbers()` (CH): `spans1m` (1M; trace_id 70k-card, INVERTED on GT / `ORDER BY
(trace_id,ts)` on CH), `m2m` (2M / 40k series), `logs1m` (1M + fulltext bloom/tokenbf index on
`message`), `errs` (1M, trace_id-keyed), `sj` (200k JSON). GT tables flushed (settled-state reads).
Warm, **median of 5 reps**. GT = `execution_time_ms`; CH = `clickhouse-client --time`.

## The matrix (median ms; lower = faster)

Median ms, lower = faster. **Faster** = which engine wins this query (both interactive — every cell
≪ 300 ms). **Details** links the curated mechanism note + the reproducible run(s) in the run log.

| Query (Parallax view) | GT v1.0.2 | GT v1.1-nightly | CH 26.5 | CH 26.6-head | Faster | Details |
| --- | ---: | ---: | ---: | ---: | --- | --- |
| **Anchored lookup** (`trace_id`, evidence-bundle hot path) | 8 | 7 | 3 | 2 | CH ~3× (both ≪ gate) | [read-path](read-path-indexing-and-execution.md) · [Runs 16/95/99/130/131](local-benchmark-results.md) |
| **Unindexed scan** (`span_id` point, full scan) | 18 | 13 | 4 | 3 | CH ~4× | [exec-engine](query-execution-engine.md) · [Runs 102/130/131](local-benchmark-results.md) |
| **TopK** (`ORDER BY duration LIMIT 10`) | 6 | 7 | 6 | 4 | ~tie | [exec-engine](query-execution-engine.md) · [Runs 106/131](local-benchmark-results.md) |
| **Trace-explorer** (`status=error AND dur>250` + sort) | 13 | 16 | 8 | 10 | CH ~1.6× | [trace-tree](trace-span-tree.md) · [Runs 127/131](local-benchmark-results.md) |
| **Metric-agg flat** (`avg(val) GROUP BY service`) | 23 | 18 | 13 | 11 | CH ~1.6× | [exec-engine](query-execution-engine.md) · [Runs 96/124/125/131](local-benchmark-results.md) |
| **Metric bucketed line** (1-min `date_bin`) | 31 | 23 | 17 | 15 | CH ~1.5× | [exec-engine](query-execution-engine.md) · [Runs 96/131](local-benchmark-results.md) |
| **Counter-rate panel** (5-min `max-min(counter)`) | 35 | 25 | 23 | 19 | CH ~1.3× | [promql/metrics](promql-and-metrics-query.md) · [Runs 113/131](local-benchmark-results.md) |
| **Last-value** ("current value" per series) | **5** | **5** | 10 | 11 | **GT ~2×** | [exec-engine](query-execution-engine.md) · [Runs 109/131](local-benchmark-results.md) |
| **Latency p99 by service** (`quantile(0.99)`) | 15 | 12 | 8 | 8 | CH ~1.5–2× | [exec-engine](query-execution-engine.md) · [Run 135](local-benchmark-results.md) |
| **Full-text selective** (exact token, 1 row) | 7 | 8 | 9 | 7 | ~tie | [indexing](indexing-internals.md) · [Runs 98/131](local-benchmark-results.md) |
| **Full-text broad** (~143k matches, this corpus) | 24 | 24 | 16 | 16 | CH ~1.5× *here*; **~12× canonical** | [indexing](indexing-internals.md) · [Runs 98/131/133](local-benchmark-results.md) |
| **Log-tail** (`service` + `ts DESC LIMIT 100`) | 17 | 13 | 3 | 3 | CH ~5× | [per-signal](per-signal-verdict.md) · [Runs 107/131](local-benchmark-results.md) |
| **Issue-list** (`GROUP BY fingerprint` + top-50) | 16 | 13 | 7 | 9 | CH ~1.8× | [verdict DQ1](verdict-which-to-choose.md) · [Runs 119/131](local-benchmark-results.md) |
| **Dynamic-attr JSON** (path GROUP BY, typed cast) | 48 | 48 | 5 | 5 | CH ~10× | [schema-evolution](schema-evolution-and-dynamic-columns.md) · [Runs 104/129/131](local-benchmark-results.md) |
| **Cross-tier join** (anchored `spans ⋈ errs`) | 65 | 36 | 3 | 3 | CH ~12–20× | [read-path](read-path-indexing-and-execution.md) · [Runs 81/103/131](local-benchmark-results.md) |
| **Ingest** (`INSERT…SELECT` 1M; *not* native path) | 719 | 623 | 201 | 170 | CH ~3.5× (synthetic) | [write-path](write-path-and-ingestion.md) · [Runs 101/132](local-benchmark-results.md) |
| **Storage** (1M rows, compressed) | 2.0 MiB | 2.0 MiB | 1.69 MiB | 1.69 MiB | CH ~1.2× | [compression/cost](compression-and-cost.md) · [Runs 100/132](local-benchmark-results.md) |
| **Cardinality-insensitive ingest** (GT append, 12 vs 1M series) | 527/457 | 489/401 | n/a | n/a | **GT (flat/cap-free)** | [metric-cardinality](metric-cardinality.md) · [Runs 84/101/132](local-benchmark-results.md) |

*(All data 1–2M rows, warm, median-of-5. Absolute numbers scale with data size — the **ratios** and
**cross-build deltas** are the signal. **Every query cell is ≪ the 300 ms interactive gate** on all
four builds. Click a row's Details for the mechanism write-up + the reproducible run.)*

*Broad-term full-text caveat (Run 133): the ~1.5× shown is on this synthetic corpus + a `tokenbf_v1`
index. On the **canonical full-text bench** (`logs_b1`, 5M rows, 699k matches) the broad-term gap is
**~12×** (CH ~7 ms / GT ~85 ms) — GreptimeDB's broad-term cost is scan-bound and grows with the
matched-row set × scale. Use ~12× as the load-bearing broad-term number; selective full-text stays a
~tie either way.*

## What this says — version by version

**1. GT-nightly v1.1.0 vs GT-stable v1.0.2 — a modest, broad improvement; no regressions.**
GT-nightly is **equal-or-faster on every query**, consistently so on the heavier ones:
- Metric aggregations **~20–30% faster** (flat 23→18, bucketed 31→23, rate 35→25).
- Cross-tier join **~1.8× faster** (65→36 ms) — though still far from CH's pushdown (3 ms).
- Unindexed scan, log-tail, issue-list slightly faster; anchored/TopK/full-text/JSON/last-value ≈ equal.
- **No regressions.** So v1.1 is a real (if modest) step — strongest on aggregation + join — likely
  the Flat-SST / metric-engine / execution work on the v1.1 roadmap. It does **not** close any gap to
  ClickHouse (JSON still ~10×, join still ~12×), and does **not** change dynamic-attr JSON (48 ms,
  JSON Type v2 not yet helping `json_get_int`). *(Caveat: ~20–30% is modest and partly noise-adjacent;
  the direction is consistent across the 3 agg queries, so it's a real small win, not a fluke.)*

**2. CH-head 26.6 vs CH-stable 26.5 — perf-flat, marginally faster aggs, but stricter.**
CH-head matches or marginally beats CH-stable (~15% on aggs: 13→11, 17→15, 23→19); within noise
elsewhere. The **real 26.6 change is correctness, not speed**: it **enforces** the typed-subcolumn
cast in JSON GROUP BY (`Code 44` without it) where 26.5 allowed a lax no-cast ~1 ms path. So adopt the
`.:Int64` cast (≈5 ms) — that is the fair, forward-compatible form.

**3. GreptimeDB vs ClickHouse — the gaps hold on all builds.**
- **ClickHouse faster** on: anchored ~3×, scan ~4×, log-tail ~5× (sort-key locality), dynamic-attr
  JSON ~10×, cross-tier in-DB join ~12–20× (pushdown). All real, all **interactive on GreptimeDB**.
- **GreptimeDB wins** last-value ~2× (time-sorted layout vs `argMax` full scan) and **ties** selective
  full-text (~7–8 ms, the right backend/function).
- **Near-parity** (~1.3–1.6×): metric aggs, bucketed, rate, trace-explorer, issue-list, broad full-text.
- **Everything ≪ 300 ms on every build** — for Parallax's queries, all four are interactive; the
  decision stays *fit not speed* (the speed gaps never cross the interactive gate).

## Ingest + storage (the non-query "performance" axes; Run 132)

| Measure | GT-stable v1.0.2 | GT-nightly v1.1.0 | CH-stable 26.5 | CH-head 26.6 |
| --- | ---: | ---: | ---: | ---: |
| **Ingest** `INSERT…SELECT` 1M rows (ms) | 719 | 623 | 201 | 170 |
| **Storage** 1M-row table (compressed) | 2.0 MiB | 2.0 MiB | 1.69 MiB | 1.69 MiB |
| **Cardinality-insensitivity** (GT ingest, append: low-card vs 1M-series) | 527 / 457 ms | 489 / 401 ms | — | — |

- **Ingest (this synthetic `INSERT…SELECT` path):** ClickHouse ~3.5× faster than GreptimeDB
  (170–201 vs 623–719 ms / 1M). **Caveat:** this is *not* the native ingest path — GreptimeDB's real
  ingest story is **native OTLP/gRPC bulk** (>1 M rows/s, Run 101's vendor-confirmed 2.68 M/s) +
  **cardinality-insensitivity**, not `INSERT…SELECT`. As a *relative* cross-build measure: both
  nightlies ~13–15% faster (GT 719→623, CH 201→170).
- **Storage:** ClickHouse ~1.2× smaller on this high-card-string table (2.0 vs 1.69 MiB) — ZSTD +
  sort-key locality. **No nightly change** either side. (Per-column-pattern; GreptimeDB wins
  high-card metric storage, Run 100.)
- **Cardinality-insensitivity holds on both GreptimeDB versions:** with `append_mode`, ingesting 1M
  rows at **1M distinct series** is **≈ or faster than** at 12 series (457 vs 527 ms stable; 401 vs
  489 nightly) — cap-free ingest, the load-bearing GreptimeDB ingest pillar (Runs 84/101), **unchanged
  in v1.1**. (ClickHouse has no dedup-merge, so its analog is the LowCardinality-overflow storage cost,
  Run 101 — not re-run here.)

## Bottom line for the operator

- **Nightlies don't change the decision.** GT v1.1 gives a modest broad speedup (aggs ~25%, join
  ~1.8×) with no regressions; CH 26.6 is perf-flat + stricter. The GreptimeDB-vs-ClickHouse gaps —
  and the verdict — are the same on all four builds.
- **The only build worth waiting for is GT v1.1 GA** (when JSON Type v2 may land properly and could
  cut the ~10× dynamic-attr gap — it does **not** in the 20260525 nightly).
- **Where ClickHouse genuinely leads** (re-confirmed on the newest of both): in-DB cross-tier join,
  dynamic-attr JSON, log-tail, raw anchored/scan latency — all of which Parallax either avoids
  (app-side correlation, anchored fetch) or absorbs (all interactive).

## Cross-refs

Detailed single-engine runs: `local-benchmark-results.md` (Run 131 logs this matrix; Runs 96–127 the
per-query deep dives; Runs 129/130 the prior 2-query and 4-way checks). Decision: `verdict-which-to-
choose.md`. Closability: `greptimedb-parity-roadmap.md`. **Re-run on GreptimeDB v1.1 GA** (`local-
benchmark-results.md` Run 128 flagged the trigger). Reproduce: re-spin the two nightly containers,
rebuild the 5 tables via `range()`/`numbers()`, run the 14 queries warm ×5 (medians above).
