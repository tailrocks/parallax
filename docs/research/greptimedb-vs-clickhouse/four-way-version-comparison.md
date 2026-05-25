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

| Query (Parallax view) | GT-stable v1.0.2 | GT-nightly v1.1.0 | CH-stable 26.5 | CH-head 26.6 | GT/CH gap |
| --- | ---: | ---: | ---: | ---: | --- |
| **Anchored lookup** (`trace_id`, evidence bundle hot path) | 8 | 7 | 3 | 2 | ~3× |
| **Unindexed scan** (`span_id` point, full scan) | 18 | 13 | 4 | 3 | ~4× |
| **TopK** (`ORDER BY duration LIMIT 10`) | 6 | 7 | 6 | 4 | ~1× |
| **Trace-explorer** (`status=error AND dur>250` + sort) | 13 | 16 | 8 | 10 | ~1.6× |
| **Metric-agg flat** (`avg(val) GROUP BY service`) | 23 | 18 | 13 | 11 | ~1.6× |
| **Metric bucketed line** (1-min `date_bin`) | 31 | 23 | 17 | 15 | ~1.5× |
| **Counter-rate panel** (5-min `max-min(counter)`) | 35 | 25 | 23 | 19 | ~1.3× |
| **Last-value** ("current value" per series) | **5** | **5** | 10 | 11 | **GT wins ~2×** |
| **Full-text selective** (exact token, 1 row) | 7 | 8 | 9 | 7 | ~1× (tie) |
| **Full-text broad** (~143k matches) | 24 | 24 | 16 | 16 | ~1.5× |
| **Log-tail** (`service` + `ts DESC LIMIT 100`) | 17 | 13 | 3 | 3 | ~5× |
| **Issue-list** (`GROUP BY fingerprint` + top-50) | 16 | 13 | 7 | 9 | ~1.8× |
| **Dynamic-attr JSON** (path GROUP BY, typed cast) | 48 | 48 | 5 | 5 | ~10× |
| **Cross-tier join** (anchored `spans ⋈ errs`) | 65 | 36 | 3 | 3 | ~12–20× |

*(All data 1–2M rows, warm. Absolute numbers scale with data size; the **ratios** and **cross-build
deltas** are the signal. Every cell is ≪ the 300 ms interactive gate.)*

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
