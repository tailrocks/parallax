# Trace Span-Tree Reconstruction (Traces Signal)

<!-- markdownlint-disable MD013 -->

Status: pass 49, **corrected pass 104 (Run 68)**. The "span trees" item of the Traces
signal (checklist #4 / scenario matrix): given a `trace_id`, how does each engine produce
the parent→child span hierarchy that a trace view (and Parallax's evidence bundle) needs?
Source-reasoned + live (Runs 27, 68). The short answer: it reduces to a question already
settled (the anchored fetch). **Correction (Run 68): the in-DB recursive CTE is NOT a tie
— GreptimeDB v1.0.2 errors on the table-self-join recursion span trees need; ClickHouse
does it. Practical impact is low (the dominant pattern is the flat fetch + app-side build,
which works on both), but it is a genuine ClickHouse capability edge.**

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

- **Recursive CTE — counter form works on both; the span-tree (table-self-join) form
  works on ClickHouse but FAILS on GreptimeDB v1.0.2 (Run 68 — corrects Run 27).** The
  *canonical counter* `WITH RECURSIVE t AS (SELECT 1 UNION ALL SELECT n+1 … n<5)` runs on
  both (GreptimeDB returned `n=5`). **But the span-tree pattern** — the recursive term
  joins the base table to the recursive relation (`… SELECT c.sid … FROM tree_gt c JOIN t
  ON c.pid=t.sid`) — **errors on GreptimeDB** with `Schema error: project index N out of
  bounds` (reproduced for both 1-column and 2-column recursive projections), while
  **ClickHouse executes it correctly** (a clean 3-node `root→child→grandchild` chain
  returned `count=3, max_depth=2`). So Run 27's "the recursive join *ran* on both" was the
  *counter* form, not the table-self-join span tree: **GreptimeDB v1.0.2 cannot do in-DB
  span-tree recursion** (a DataFusion recursive-CTE projection limitation). *(Also found:
  GreptimeDB loads the root's empty `parent_span_id` as **NULL**, not `''` — a base-case
  predicate must use `IS NULL`.)*
- **The flat anchored fetch is the real hot path, and it's the anchored-lookup
  question already settled.** All 14 spans of one `trace_id`: **ClickHouse 4 ms**
  (the `spans` `ORDER BY (trace_id, ts)` sort-key prefix → spans are physically
  contiguous, one granule range), **GreptimeDB ~14 ms server-time** (Run 40 — *not* the
  24–54 ms HTTP-wall; the seed `spans` has PK `(service,name)` so `trace_id` is a full
  scan, and the HTTP wall added ~40 ms; with the `trace_id INVERTED INDEX` Parallax's
  design adds, ~8 ms, Run 6). So span-tree *retrieval* performance = the anchored-fetch
  performance (both ≪ the 300 ms gate), where
  **ClickHouse's sort-key locality wins** unless GreptimeDB makes `trace_id` the key
  prefix.

## Side by side

| | GreptimeDB | ClickHouse |
| --- | --- | --- |
| Flat anchored fetch (`WHERE trace_id=X`) | inverted index on `trace_id` + scan (Run 6: 8–24 ms) | `ORDER BY (trace_id, ts)` sort-key locality → 1 granule (Run 2: ~4 ms) |
| App-side tree build | ✓ (returns flat spans) | ✓ (returns flat spans) |
| In-DB recursive CTE — counter | ✓ (Run 68, `n=5`) | ✓ native |
| In-DB recursive CTE — **span-tree (table self-join)** | ✗ **errors** "project index out of bounds" (Run 68, v1.0.2) | ✓ (Run 68, 3 rows / depth 2) |
| Span-tree retrieval winner | — | **ClickHouse** (sort-key locality + the only in-DB recursion), unless GT keys `trace_id` |

## Axis consequence

- **Speed (axis #1):** span-tree *retrieval* is **not a new differentiator** — it is the
  anchored `trace_id` fetch (ClickHouse edge via sort-key locality; GreptimeDB
  competitive once `trace_id` is keyed/indexed, as the Parallax GreptimeDB DDL does)
  plus app-side assembly. **Capability (corrected): in-DB span-tree recursion is a
  ClickHouse edge, not a tie** — GreptimeDB v1.0.2 cannot run the table-self-join
  recursive CTE (Run 68). **But practical impact is low:** the dominant pattern is the
  flat fetch + app-side tree build (what Jaeger/Tempo do), which works on both; in-DB
  recursion is only needed for server-side tree analytics (critical-path, descendant
  rollups).
- **Parallax fit:** Parallax assembles the bundle's trace context from the flat fetch
  (Q1, already measured as part of the Q6 composite — not latency-bound, Run 16) and
  builds the tree app-side, so the recursion gap **does not block** Parallax. It would
  only bite if Parallax wanted *in-DB* tree analytics on GreptimeDB — then the options
  are: do it app-side, or (the parity angle) wait for GreptimeDB's DataFusion recursive
  CTE to support the table-self-join form. A minor, mechanism-grounded ClickHouse
  capability edge; does not move the verdict.

## Honest caveats

- The recursion finding is now from a **clean 3-node tree** (Run 68), not the synthetic
  `spans` (whose parent links don't form a connected tree from the root) — that is why
  the earlier synthetic recursive query returned degenerate counts. The clean tree shows
  ClickHouse traverses (3 rows / depth 2) and GreptimeDB v1.0.2 errors on the
  table-self-join form. Recursion latency-vs-depth at scale is unmeasured (moot for
  GreptimeDB until the CTE form is supported).
- GreptimeDB's flat-fetch latency carries the HTTP/MySQL-protocol + fixed-overhead floor
  seen throughout (Run 1/6); the gap is fixed overhead + key placement, not algorithmic.
- Recursive-CTE latency at large fan-out / deep trees isn't measured; at smoke depth
  both are sub-10 ms.

## Source / evidence

- Live (Run 27): flat fetch CH 4 ms / GT 54 ms (HTTP) on the 1M-row `spans` table.
- Live (Run 68): counter `WITH RECURSIVE` works on both (GT `n=5`); **table-self-join
  span-tree recursion — ClickHouse 3 rows/depth 2, GreptimeDB v1.0.2 `Schema error:
  project index out of bounds` (1-col and 2-col)**; flat fetch re-verified CH ~2 ms /
  GT ~15 ms; GreptimeDB root `parent_span_id` stored as NULL not `''`.
- Cross-refs: `read-path-indexing-and-execution.md` (anchored join/scan, PREWHERE),
  `per-signal-verdict.md` (Traces · `trace_id` point lookup row), `indexing-internals.md`
  (GreptimeDB inverted index vs ClickHouse sort-key locality), Run 2/6/16.
