# GreptimeDB Parity Roadmap — What To Implement To Make It A Clear Winner For All Cases

<!-- markdownlint-disable MD013 -->

Status: pass 76 (new). Answers the operator question: GreptimeDB wins Parallax on *fit*
(`verdict-which-to-choose.md`), but ClickHouse is genuinely ahead on a few capabilities —
**what would GreptimeDB have to implement, against its actual internals, to be the
unambiguous choice for every query shape?** For each ClickHouse advantage: the
mechanism, the concrete change in GreptimeDB's real subsystems, the effort tier, and
whether it is a *design* gap or an *integration* gap.

Pins: GreptimeDB `v1.0.2` (`0ef5451`, DataFusion `=52.1`), ClickHouse
`v26.5.1.882-stable` (`5b96a8d8`), re-verified latest stable 2026-05-25.

## The load-bearing finding (why this is mostly tractable)

GreptimeDB's index **toolkit is richer**, not poorer (FST+roaring inverted, Lucene-class
tantivy full-text, configurable-granularity bloom — `indexing-internals.md`). So its
measured losses are **execution-integration gaps, not architectural ones**
(`query-execution-engine.md`): a younger DataFusion-over-Arrow scan engine and a
multi-step index→scan path, versus ClickHouse's decade-tuned C++ pipeline. **Integration
gaps close by engineering; architecture gaps would require redesign.** Almost everything
below is the former — and because GreptimeDB and DataFusion are open-source Rust (the
project's north star), the operator can *contribute* the fixes, not just wait for them.

## The gaps and what closes each

| # | ClickHouse advantage | Mechanism (why CH wins) | What GreptimeDB implements — against its real structure | Tier | Gap kind |
| --- | --- | --- | --- | --- | --- |
| 1 | **Full-text log search ~18× (warm, Run 12/38)** | Coarse `text` posting-list prune **inside** the C++ pipeline, then vectorized `hasToken` confirm on 65k-row blocks — index lookup + confirm are one fast path. | **Fuse the tantivy hit-set into the scan.** GreptimeDB already has the richer index (Puffin `greptime-fulltext-index-v1`, tantivy 0.24); the cost is the load-blob → apply → map-to-row-group → DataFusion-scan round trip. Implement: (a) keep the FST/posting-lists **warm-cached** (avoid per-query Puffin blob reload); (b) push the tantivy row/segment hit-set directly into a Parquet **row-selection** the scan node consumes without re-materializing. Files: `src/mito2/src/sst/index/fulltext_index.rs` (applier), the mito2 Parquet reader, the DataFusion scan node. | **B** | integration |
| 2 | **Generic scan/aggregate throughput ~2–4×** | 65,409-row blocks (8× DataFusion's 8,192) + **LLVM-JIT** expressions/aggregation + bespoke SIMD kernels + specialized adaptive hash tables. | (a) **Raise the `RecordBatch` size** in `SessionConfig` (`src/query/src/query_engine/state.rs:126`) toward 32–64k so vectors amortize overhead and feed SIMD; (b) **expression + aggregation codegen** — DataFusion's is young/narrow vs LLVM JIT; (c) **specialized SIMD aggregation** (two-level hash for high-card, fixed-width-key kernels). Mostly **upstream DataFusion** — GreptimeDB inherits as DF improves, or contributes. | **B** | integration (upstream DataFusion) |
| 3 | **Late materialization (PREWHERE)** | Reads cheap filter columns first → selection vector → decodes only surviving rows of wide columns. GreptimeDB reads full row-groups (filter pushdown, but no PREWHERE-equivalent yet — `query-execution-engine.md`). | **Late materialization in the mito2 Parquet reader.** Parquet supports page-index + row-filter pushdown; DataFusion `ParquetExec` has partial `RowFilter`/page-skipping. Implement: evaluate predicates on minimal columns, build a row-selection, decode wide/`Json` columns only for survivors. Files: `src/mito2/src/sst/parquet/reader.rs` + DataFusion ParquetExec row-filter. | **B** | integration |
| 4 | **Dynamic-attribute path queries (JSON)** | `JSON` type stores each path as a **typed columnar subcolumn** → `attributes.user` reads one subcolumn. GreptimeDB `Json` is a **binary blob** (jsonb) + `json_get_*` per-row parse (`schema-evolution-and-dynamic-columns.md`). | **Shred JSON paths into Parquet subcolumns.** Adopt the emerging Parquet **Variant/shredding** layout: at flush, write hot/declared attribute paths as their own typed Parquet columns; push `attributes.k` access down to a subcolumn scan instead of a per-row blob parse. Files: a shredded column type + mito2 SST writer + DataFusion pushdown. **Biggest storage-format change here** — borders on design, but Parquet's variant work makes it integration, not a rewrite. | **B** | integration / format |
| 5 | **Projections (alternate physical `ORDER BY`)** | A projection stores a 2nd sort order **inside each part**, optimizer-picked → fast sequential scan on an alternate key from one table (Run 28). GreptimeDB indexes give *positions*, not a 2nd physical order. | Two paths: (a) **today, Tier A workaround** — maintain a re-sorted derived table via **Flow** (`CREATE FLOW … SINK TO …` keyed on the alternate order; Run 43 proved Flow works) — extra storage, app-managed; (b) **true parity, Tier B** — mito2 writes an alternate-sorted SST copy per region with planner auto-pick. Files: `src/mito2` SST writer + region metadata + DataFusion planner rule. | **A** (workaround) / **B** (native) | integration |
| 6 | **Vertical single-node ceiling + analytical/merge maturity** | A decade of single-box scan tuning; battle-tested merges. | Inherited via #2 (engine) + time/battle-testing. Not a discrete feature. | **C** | maturity |

## The three tiers (the decision-useful summary)

- **Tier A — close it in Parallax today, no engine change.** Index `trace_id`/`fingerprint`
  (Run 45 design), **Flow pre-aggregation** for dashboards (Run 43, ~5–6× read speedup,
  neutralizes the ~2× SQL-agg gap), **SQL not PromQL** for hot aggregations (Run 44: GT SQL
  ~5× faster than GT's own PromQL), FULLTEXT-index + anchored log search, and the Flow
  alternate-ordering workaround (#5a). **This already makes GreptimeDB a clear winner for
  Parallax's *anchored* workload** — the gaps below only bite on heavy *ad-hoc* analytics.
- **Tier B — contribute upstream (Rust, open-source) to win *all* cases.** Items 1–5: batch
  size, JIT, SIMD, PREWHERE late materialization, JSON shredding, native projections,
  index↔scan fusion. These are what "clear winner even for large-scale ad-hoc log/trace
  search and heavy scans" requires. Several (#2, #3) ride the **DataFusion roadmap** and
  arrive without Parallax work; the rest are GreptimeDB-specific but **engineering, not
  redesign**, and PR-able given the Rust-first north star.
- **Tier C — accept or wait.** Distributed/analytical battle-testing; matures with time.

## The alternative to closing the gap: a hybrid

If the Tier-B work is not worth it and ad-hoc log search proves heavy, the alternative is a
**GreptimeDB (system-of-record + correlation) + ClickHouse (log-search sidecar)** split.
Honest cost: it **splits Parallax's cross-signal evidence-bundle correlation across two
engines** — exactly the hot path the whole product is built on — turning anchored bundle
assembly into a cross-engine fan-out, plus doubling the ops surface (against the
startups-first/tiny-single-node trajectory). Only justified if a benchmark shows log search
is both **heavy** and **standalone** (not joined with other signals). Never split the bundle.
See `verdict-which-to-choose.md` (DQ5) and `distributed-and-scaling.md`.

## Bottom line

For Parallax's **anchored** workload, **Tier A alone makes GreptimeDB a clear winner today**
— no engine changes needed. To make it the unambiguous choice for **every** shape including
heavy ad-hoc analytics, the remaining gaps (1–5) are **execution-integration**, mostly on
the **DataFusion roadmap** or contributable in Rust — *not* architectural limits. That is the
encouraging answer to "what must GreptimeDB implement": engineering, not redesign. Validate
first which gaps Parallax's real query mix actually hits before investing in Tier B.

## Open questions handed to the benchmark

- Quantify the post-Tier-A residual: with FULLTEXT + warm index cache, how much of the ~18×
  log-search gap remains? (Owed to a tuned re-run; `benchmarking-the-differences.md`.)
- Does raising the DataFusion batch size in `SessionConfig` measurably narrow the ~2× agg
  gap at 40k series? (A cheap local probe — propose as a new case.)
- JSON-shredding payoff: attribute-path query latency, blob-parse vs subcolumn — at volume.

## Source / evidence

- Mechanisms: `query-execution-engine.md` (batch/JIT/SIMD/PREWHERE), `indexing-internals.md`
  (Puffin/tantivy vs `text`, integration-not-format), `read-path-indexing-and-execution.md`
  (pushdown/skip), `schema-evolution-and-dynamic-columns.md` (JSON blob vs subcolumn),
  `projections-and-access-paths.md` (Run 28), `rollup-and-continuous-aggregation.md` (Flow).
- Empirical: `local-benchmark-results.md` Runs 11/12/37/38 (the gaps), 43 (Flow), 44 (PromQL
  vs SQL), 45 (GreptimeDB schema build).
- Decision context: `verdict-which-to-choose.md`. Loop target: `prompts/greptimedb-vs-clickhouse-internals.md`
  ("Closing The Gap").
