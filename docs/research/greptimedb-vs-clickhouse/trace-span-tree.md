# Trace Span-Tree Reconstruction (Traces Signal)

<!-- markdownlint-disable MD013 -->

Status: pass 49. The "span trees" item of the Traces signal (checklist #4 / scenario
matrix): given a `trace_id`, how does each engine produce the parent→child span
hierarchy that a trace view (and Parallax's evidence bundle) needs? Source-reasoned +
live (Run 27). The short answer: it reduces to a question already settled (the anchored
fetch) plus a capability tie (recursive CTE).

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`),
re-confirmed latest stable 2026-05-25.

## Two ways to build a span tree

A span tree is `span_id ← parent_span_id` edges within one `trace_id`. There are two
implementation strategies, and which one Parallax uses decides what matters:

1. **Flat anchored fetch + app-side tree build (the dominant pattern).** `SELECT
   span_id, parent_span_id, … WHERE trace_id = X ORDER BY ts` returns all spans of the
   trace; the *application* (or trace UI — this is what Jaeger/Tempo do) assembles the
   tree in memory. The DB does **no recursion** — its job is a cheap anchored scan.
2. **In-DB recursive walk (`WITH RECURSIVE`).** Walk parent→child in SQL to compute
   depth, descendants of a span, or critical path *inside* the engine. Needed only for
   server-side tree analytics, not for "show me the trace."

## Live findings (Run 27)

- **Recursive CTE works on BOTH engines.** `WITH RECURSIVE` returned correctly on
  ClickHouse (native) **and** GreptimeDB (via DataFusion) — both gave `15` for the
  canonical `1..5` sum, and both executed the real span-tree recursive join over the
  1M-row `spans` table (CH ~7 ms, GreptimeDB ~8 ms server-side; the synthetic dataset
  isn't a clean parent chain, so depth grouping was trivial, but the recursive join
  *ran* on both). → **in-DB span-tree analytics is a capability tie**; DataFusion gives
  GreptimeDB recursive CTE for free.
- **The flat anchored fetch is the real hot path, and it's the anchored-lookup
  question already settled.** All 14 spans of one `trace_id`: **ClickHouse 4 ms**
  (the `spans` `ORDER BY (trace_id, ts)` sort-key prefix → spans are physically
  contiguous, one granule range), **GreptimeDB ~24–54 ms** (seed PK is `(service,name)`,
  so `trace_id` is served by the **inverted index** + a fixed HTTP/setup floor — Run
  1/6). So span-tree *retrieval* performance = the anchored-fetch performance, where
  **ClickHouse's sort-key locality wins** unless GreptimeDB makes `trace_id` the key
  prefix.

## Side by side

| | GreptimeDB | ClickHouse |
| --- | --- | --- |
| Flat anchored fetch (`WHERE trace_id=X`) | inverted index on `trace_id` + scan (Run 6: 8–24 ms) | `ORDER BY (trace_id, ts)` sort-key locality → 1 granule (Run 2: ~4 ms) |
| App-side tree build | ✓ (returns flat spans) | ✓ (returns flat spans) |
| In-DB recursive CTE | ✓ via DataFusion (Run 27, ~8 ms) | ✓ native (Run 27, ~7 ms) |
| Span-tree retrieval winner | — | **ClickHouse** (sort-key locality), unless GT keys `trace_id` |

## Axis consequence

- **Speed (axis #1):** span-tree retrieval is **not a new differentiator** — it is the
  anchored `trace_id` fetch (ClickHouse edge via sort-key locality; GreptimeDB
  competitive once `trace_id` is keyed/indexed, as the Parallax GreptimeDB DDL does)
  plus app-side assembly. In-DB recursion is a **tie** (both ~7–8 ms). Neither moves
  the verdict.
- **Parallax fit:** Parallax assembles the bundle's trace context from the flat fetch
  (Q1, already measured as part of the Q6 composite — not latency-bound, Run 16), and
  can use recursive CTE on *either* engine for server-side tree analytics if ever
  needed. So traces add no blocker in either direction; the trace-context sub-query is
  the anchored fetch the verdict already accounts for.

## Honest caveats

- The synthetic `spans` dataset isn't a connected parent chain, so the recursive
  *depth distribution* wasn't exercised — only that the recursive join executes on both
  engines. A clean-tree dataset would quantify recursion latency vs depth (a
  `benchmarking-the-differences.md` candidate if Parallax ever needs in-DB tree ops).
- GreptimeDB's flat-fetch latency carries the HTTP/MySQL-protocol + fixed-overhead floor
  seen throughout (Run 1/6); the gap is fixed overhead + key placement, not algorithmic.
- Recursive-CTE latency at large fan-out / deep trees isn't measured; at smoke depth
  both are sub-10 ms.

## Source / evidence

- Live (Run 27): `WITH RECURSIVE` on both engines; flat fetch CH 4 ms / GT 54 ms (HTTP)
  on the 1M-row `spans` table.
- Cross-refs: `read-path-indexing-and-execution.md` (anchored join/scan, PREWHERE),
  `per-signal-verdict.md` (Traces · `trace_id` point lookup row), `indexing-internals.md`
  (GreptimeDB inverted index vs ClickHouse sort-key locality), Run 2/6/16.
