# GreptimeDB Parity Roadmap — What To Implement To Make It A Clear Winner For All Cases

<!-- markdownlint-disable MD013 -->

Status: pass 76 (new) + pass 77 (gaps #2/#3 source-verified) + pass 78 (gap #1
source-corrected) + pass 79 (**expanded to detailed per-improvement what/why/how**, framed
as borrowed-concept → GreptimeDB structure → value, per operator) + pass 80 (#4 JSON
binary-jsonb + per-row `json_get` source-confirmed; #2 batch-size has no runtime knob,
live-probed) + pass 81 (**added Improvement #7** — per-column codec parity; source-grounded:
user data columns default to `PLAIN`+ZSTD, floats miss the Gorilla-class encoding) + pass 82
(#5 projections **source-confirmed absent**; **added the user-first ranking** + per-improvement
user stories + explicit "does this make GreptimeDB the clear winner?" verdicts, per operator —
the reason to add anything is solving a real Parallax user's problem) + pass 83 (user-story /
clear-winner bullet now on **every** improvement #1–#7 — format complete; honest verdict:
only #1 flips a real common user moment, #2–#5,#7 are footnotes for Parallax's usage) + pass 84
(format correction: #6 now carries the same user-story verdict, explicitly as a maturity caveat,
not an implementable feature) + pass 85 (**Run 47 isolated the full-text gap** via live metrics:
the fulltext index apply is ~0.15 ms / ~0.1 % of the query → the ~18× is the **post-index scan**,
so #1's primary lever is the scan engine (#2/#3), the tantivy cache is second-order) + pass 86
(**Run 48 — the ~18× was a query-form artifact**: `matches()` on a `backend='bloom'` index
full-scans; `matches_term()` prunes → selective exact-term is **~8 ms / ~2–3×**, not 18×. #1
downgraded to a Tier-A usage fix; verdict flip-trigger narrowed) + pass 87 (**Run 49**: tantivy
backend `matches()` **also prunes** ~6 ms → query-syntax path fast too; full-text gap is fully
a backend/function pairing artifact, residual = broad-term analytics only) + pass 115 (**added
Improvement #8** — push an equality filter into an *indexed join input*: Run 81/82 found GreptimeDB
full-scans a join's anchored table (`output_rows: 1M`, both INNER+LEFT) instead of using the inverted
index, an optimizer pushdown gap; Tier-A workaround = subquery pre-filter / app-side correlation) + pass 101 (**Improvement
#5 deepened with the Run 63 root cause**: GreptimeDB's PK = sort = series identity, so it can't
cluster by a high-card anchor like `trace_id` without series blowup, while ClickHouse `ORDER BY`
decouples sort from identity at zero cost — this is *also* the root of the cold-selective-read egress
loss (Run 55/63), so #5 now closes two gaps; `order_by` table option live-rejected, Run 65) + pass 96 (**added the physics-wall
closability test** — a per-gap engineering/fundamental/time-only verdict answering the
operator's investment question: *no gap is a physics wall*; 7/8 are engineering, #6 is
time, #5 is the lone design-flavoured one and is defused by `trace_id` partitioning; the
two heaviest (#2, #4) ride shared industry roadmaps; investment synthesis → `verdict-which-
to-choose.md` DQ6). This is **the dedicated,
standalone file** answering "what can GreptimeDB improve, why, and how" — the summary table
scans, the detailed sections below carry the code-oriented specifics. Answers the operator
question: GreptimeDB wins Parallax on *fit*
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

**Pass-77 source check strengthens this** (both load-bearing engine claims verified against
v1.0.2, not just reasoned): gap #2 — the query `SessionConfig` (`state.rs:126-128`) sets only
`with_target_partitions`, never `batch_size`, so DataFusion's 8,192 default genuinely holds (a
one-line change to raise). Gap #3 — the mito2 reader already prunes row-groups/pages (indexes +
page index) and post-decode-filters rows, but has **no arrow `RowFilter`**, so the PREWHERE fix
is *wiring an arrow primitive that already exists*, not inventing late materialization. Both are
integration, exactly as the thesis predicts.

**Pass-78 source check corrected gap #1** (drift caught in this roadmap's own first draft):
GreptimeDB **already** in-memory-caches the inverted, bloom, and vector indexes + an
index-application result cache (`cache/index/`), so "warm-cache the FST" was already done.
The residual cost was thought to be tantivy-specific, because there is no dedicated
`FulltextIndexCache`, but Run 47 then showed the fulltext index apply itself was sub-ms.

**Pass-86 live check corrected the user-level impact again:** Run 48 showed the benchmarked
`logs_b1` table used `backend='bloom'`, while Run 12/38 queried it with `matches()` — the
tantivy query-syntax function, not the exact-term bloom path. That combination full-scans
5M rows. With the correct pairing, `matches_term()` + bloom, GreptimeDB prunes to the single
matching row and returns selective exact-term search in ~8 ms warm (~2-3× ClickHouse, both
sub-perceptible). **Pass-87 / Run 49 closed the last piece:** a tantivy-backed index makes
`matches()` (query syntax) **prune** too — selective ~6 ms warm (EXPLAIN `output_rows: 1`). So
**both** selective full-text paths are fast with the correct backend (tantivy+`matches` ~6 ms,
bloom+`matches_term` ~8 ms), and the ~18× was **100 % a backend/function misconfiguration**.
The only residual is **broad-term scans that match many rows** (scan engine, #2). This moves #1
from "engine work required for incident grep" to "usage/schema guard only; scan-engine work
solely if broad-term log analytics is common."

## Closable, fundamental, or time-only? — the physics-wall test

The operator's investment question reduces to one thing: **is ClickHouse's speed lead a
permanent moat (architectural physics, like Singapore↔US latency vs Singapore↔China —
unimprovable by any amount of engineering) or a depreciating asset (a decade of C++
hand-tuning that Rust/DataFusion can amortize)?** Each gap gets one of three verdicts:

- **Engineering** — same architectural model (vectorized columnar over Arrow); the other
  side is just further along the *same* curve. Someone writes the Rust; it is not a wall.
- **Fundamental** — a real architectural wall the current design cannot cross without a
  redesign. This is the only verdict that would justify "stay on ClickHouse forever."
- **Time-only** — not code, not physics: maturity / battle-testing, closes on a calendar.

| # | Gap | Verdict | Who closes it / leverage | Physics wall? |
| --- | --- | --- | --- | --- |
| 1 | Full-text log search | **Engineering** (selective already dissolved — usage) | Tier-A function/backend pairing today; broad-term → #2 | No |
| 2 | Scan/agg throughput 2–4× | **Engineering** | **On the DataFusion roadmap** (batch size, expr/agg codegen, SIMD) — shared-ecosystem, benefits every Arrow engine; GreptimeDB inherits on the `datafusion` bump | No |
| 3 | PREWHERE late materialization | **Engineering — SHIPPED + MEASURED WORKING (Runs 121/122)** | GreptimeDB **v1.0.2 added its own `prefilter.rs` framework** (read filter columns first → refined row selection → read the rest), wired into the Flat read path (`file_range.rs: has_flat_primary_key_prefilter`); **measured pruning the wide-column decode ~3× (selective ~16 ms vs ~50 ms full-decode, Run 122)**. PK/partition-scoped; residual vs CH (~5×) is general throughput, not missing late-materialization. General arbitrary-column PREWHERE is the remaining Tier-B piece. | No |
| 4 | Dynamic-attr JSON | **Engineering / format** | Parquet **Variant/shredding** spec is the industry direction; integration along a spec, not a new type | No |
| 5 | Alternate ordering / cold selective egress | **Engineering (mitigated)** + a design *residue* | `PARTITION ON COLUMNS(trace_id)` already cuts cold egress to ~1/N at no PK-cardinality cost (Run 87/88); a re-sorted Flow copy closes the rest. Full sort/identity *decoupling* would be a region-model redesign — but the partition+copy **sidesteps** it | **Closest candidate — but defused.** The PK=sort=series conflation is the one structural edge; partitioning + copy reduce it to an engineering choice, not a wall |
| 6 | Vertical ceiling / merge maturity | **Time-only (Tier C)** | Calendar + production hardening; GreptimeDB's structural answer is horizontal scale-out + object store | No (not engineering either) |
| 7 | Per-column codecs | **Engineering** | Parquet ships `BYTE_STREAM_SPLIT`/`DELTA_BINARY_PACKED`; extend `customize_column_config` to user columns (Tier B) | No |
| 8 | Join-pushdown into indexed input | **Engineering** | Tier-A subquery pre-filter today; optimizer rule upstream (Tier B) | No |

**Verdict: there is no physics wall.** Seven of eight gaps are pure engineering; #6 is time;
#5 is the only one with an architectural *flavour* (PK=sort=series identity), and it is
already **defused to an engineering choice** by `trace_id` partitioning + a re-sorted copy
(Runs 87/88). Critically, the two heaviest gaps (#2 scan/agg, #4 JSON) ride **shared
industry roadmaps** (DataFusion codegen/SIMD; Parquet Variant) — so closing them is *partly
someone else's work already in flight*, and any Parallax/operator contribution lands in an
ecosystem, not a private fork. This is the Postgres-overtook-MySQL shape: the engine that
was never architecturally behind, catching a faster-now incumbent as effort compounds.
The investment synthesis (cost, scalability, Rust-contributability, the honest risk) lives
in `verdict-which-to-choose.md` → "Long-term investment decision."

## User-first ranking — does any of this actually make GreptimeDB the clear winner?

The reason to add anything is **solving a real Parallax user's problem**, not parity for its
own sake. Parallax's users are developers / SREs / AI agents debugging an incident: the hot
flow is **anchored evidence-bundle assembly** (fetch everything for a `trace_id`/`fingerprint`),
which is *already fast on GreptimeDB* (Q6 ~33 ms, Run 16). Ranked by who-actually-feels-it:

1. **Improvement 1 (full-text log search) — Runs 48-49 downgraded it from
   "clear-winner-maker" to "already competitive, just configure it right."** User story:
   *an SRE greps logs for a request-id or quoted phrase during an incident* — a selective
   search. With **tantivy + `matches()`** GreptimeDB returns query-syntax selective search in
   **~6 ms**, and with **bloom + `matches_term()`** exact-term search in **~8 ms** (~2×
   ClickHouse, all sub-perceptible). The reported ~18× was a query-form artifact (`matches()`
   on a bloom index full-scans — Run 48; Run 49 proves tantivy+`matches()` prunes). So incident
   grep needs **no engine work** (Tier-A usage/schema guard). The big gap remains only for
   **broad-term analytics** (~12×, scan engine #2) — not everyday selective incident grep.
2. **Improvement 4 (JSON attribute queries) — matters only for unplanned attribute analytics.**
   User story: *a user groups errors by `http.status_code` or filters spans by `user.id`
   across last week.* Felt only at volume on **undeclared** attributes; the Tier-A answer
   (promote hot attributes to real columns) wins the common case. Footnote unless attribute
   exploration is core.
3. **Improvement 2 (scan/agg engine) — Tier-A Flow pre-agg already wins the user-facing case.**
   User story: *a user opens a high-cardinality metric dashboard.* Flow pre-aggregation makes
   that fast (Run 43); the raw-engine gap only bites on *unplanned* heavy aggregation, which
   Parallax users rarely run interactively. Footnote for the common flow.
4. **Improvements 3 (PREWHERE), 5 (projections), 7 (codecs) — footnotes for Parallax users.**
   PREWHERE helps wide-row selective scans the anchored path avoids; projections serve a
   second scan order Parallax rarely needs (anchors on `trace_id`/`fingerprint`); codecs are
   **invisible to users** (a storage-cost second-order lever, and compression is a near-wash).
   None makes GreptimeDB a "clear winner" of a user-felt case.

**Honest bottom line (user-first):** for Parallax's actual hot path, GreptimeDB is *already*
the clear winner — none of these is required. **Runs 48-49 strengthened this further:** even the
incident log-search case (#1, once thought the one clear-winner-maker) is **already competitive
~6-8 ms with the right function/backend** (`matches`+tantivy or `matches_term`+bloom) — a
Tier-A usage choice, not an engine build. So **no improvement here is a must-do for Parallax's
common user moments.** The remaining real gap is broad-term/ad-hoc analytics.
**Validate the query mix first** (what fraction of real Parallax queries are ad-hoc log search
vs anchored retrieval); invest in Tier-B engine work
only if that fraction is high. Anything else is mechanism elegance without user impact.

## The gaps and what closes each

| # | ClickHouse advantage | Mechanism (why CH wins) | What GreptimeDB implements — against its real structure | Tier | Gap kind |
| --- | --- | --- | --- | --- | --- |
| 1 | **Log search gap mostly dissolved by Runs 48-49** | ClickHouse still has a fast integrated `text`/`hasToken` path, but the old ~18× GreptimeDB result was `matches()` on a bloom-backed index, which full-scanned. Correct selective pairings prune: `matches()` + tantivy returns ~6 ms warm and `matches_term()` + bloom returns ~8 ms warm. | **First fix usage/schema, then measure broad residuals.** Document Parallax log-search DDL/query rules: tantivy backend + `matches()` for query-syntax/phrase/relevance search; bloom backend + `matches_term()` for exact request-id/token grep. Add planner/UX guardrails so a bloom index is not queried through the non-pruning function. Broad-term scans still route to #2 scan-engine work. A dedicated `FulltextIndexCache` is not a current priority because Run 47 measured index apply at ~0.15 ms. | **A** (usage/schema guard) / **B** (broad scan) | usage / integration |
| 2 | **Generic scan/aggregate throughput ~2–4×** | 65,409-row blocks (8× DataFusion's 8,192) + **LLVM-JIT** expressions/aggregation + bespoke SIMD kernels + specialized adaptive hash tables. | (a) **Raise the `RecordBatch` size** in `SessionConfig` — **source-confirmed (pass 77): `state.rs:126-128` sets only `with_target_partitions`, never `batch_size`, so DataFusion's 8,192 default holds** → raise toward 32–64k so vectors amortize overhead and feed SIMD; (b) **expression + aggregation codegen** — DataFusion's is young/narrow vs LLVM JIT; (c) **specialized SIMD aggregation** (two-level hash for high-card, fixed-width-key kernels). Mostly **upstream DataFusion** — GreptimeDB inherits as DF improves, or contributes. | **B** | integration (upstream DataFusion) |
| 3 | **Late materialization (PREWHERE)** | Decodes cheap filter columns first → row mask → decodes wide columns only for surviving rows. | **Add column-staged late materialization to the mito2 reader.** Source-confirmed (pass 77): GreptimeDB already **prunes** row-groups/pages (`RowGroupSelection` from Puffin fulltext/inverted indexes + the Parquet **page index**, `reader.rs`) and then **post-decode**-filters rows (`PruneReader::precise_filter`, `read/prune.rs:119`) — so within a surviving row-group it decodes **all projected columns before dropping rows**. There is **no arrow `RowFilter`** in the reader (grep = 0) → no column-staging. Fix: wire the pushed-down predicate into arrow's **`RowFilter`** (`ParquetRecordBatchReaderBuilder::with_row_filter`, which arrow-rs already provides) so filter columns decode first, build a selection, and wide/`Json` columns materialize only for survivors. File: `src/mito2/src/sst/parquet/reader.rs`. **Arrow ships the primitive → integration, not a new algorithm.** | **B** | integration |
| 4 | **Dynamic-attribute path queries (JSON)** | `JSON` type stores each path as a **typed columnar subcolumn** → `attributes.user` reads one subcolumn. GreptimeDB `Json` is a **binary blob** (jsonb) + `json_get_*` per-row parse (`schema-evolution-and-dynamic-columns.md`). | **Shred JSON paths into Parquet subcolumns.** Adopt the emerging Parquet **Variant/shredding** layout: at flush, write hot/declared attribute paths as their own typed Parquet columns; push `attributes.k` access down to a subcolumn scan instead of a per-row blob parse. Files: a shredded column type + mito2 SST writer + DataFusion pushdown. **Biggest storage-format change here** — borders on design, but Parquet's variant work makes it integration, not a rewrite. | **B** | integration / format |
| 5 | **Projections / alternate physical `ORDER BY`** (decouple sort from PK/series identity) | A projection stores a 2nd sort order **inside each part**, optimizer-picked → fast scan on an alternate key (Run 28); CH `ORDER BY(trace_id,ts)` also clusters the high-card anchor at **zero cardinality cost**. GreptimeDB's **PK = sort = series identity**, so it can't cluster by `trace_id` without 71k-series blowup (Run 63), and indexes give *positions* not a 2nd order. **Now also the root of cold-read egress** (Run 55/63: scattered anchor → cold read pulls whole SST). | (a) **Tier A** — Flow re-sorted copy (`SINK TO …`, Run 43); a `trace_id`-sorted copy also fixes cold-read egress; (b) **Tier B** — mito2 alternate-sorted SST copy + planner auto-pick (`src/mito2` SST writer + region metadata + DataFusion rule). Full sort/identity decoupling = redesign; the copy sidesteps it. | **A** / **B** (copy) | integration (copy); design (full decouple) |
| 7 | **Per-column codecs (`CODEC(Gorilla/DoubleDelta/T64)`)** | Hand-picked per-column codecs match each column's shape — `Gorilla` on float gauges (Run 4: 78×), `DoubleDelta` on monotonic counters (7.3×). | **Type-aware Parquet encodings + a column codec DDL option.** Source (pass 81): mito2's writer **already** sets per-column encodings for *internal* columns (`writer.rs:387-391`: `ts`/`seq` → `DELTA_BINARY_PACKED` ≈ DoubleDelta, `op_type` → UNCOMPRESSED) but **user data columns default to `Encoding::PLAIN` + table-wide ZSTD** (`writer.rs:433-434`) — so a `Float64` gauge gets **no Gorilla/float-split encoding**. Fix: in the writer's `customize_column_config` (`writer.rs:371`), pick **type-aware encodings** for user columns (floats → Parquet **`BYTE_STREAM_SPLIT`** ≈ Gorilla; monotonic ints → `DELTA_BINARY_PACKED`); optionally expose a per-column codec option in DDL. Parquet already ships these encodings → integration. | **B** | integration |
| 6 | **Vertical single-node ceiling + analytical/merge maturity** | A decade of single-box scan tuning; battle-tested merges. | Inherited via #2 (engine) + time/battle-testing. Not a discrete feature. | **C** | maturity |
| 8 | **Push an equality filter into an *indexed join input*** | CH pushes a join input's filter into its scan → index/PREWHERE prunes *before* the join (Q4 `Granules 1`, Run 30/81). | **Optimizer fix:** push a join input's equality predicate to the `TableScan` as an *index-eligible* filter so the inverted index prunes, instead of landing as a post-scan `FilterExec` → full scan. Run 81/82: direct cross-tier join full-scans 1M (`output_rows: 1,000,000`, ~54 ms, both INNER+LEFT) vs CH 4 ms; standalone `WHERE` prunes (14). `src/query/src/optimizer` + DataFusion `push_down_filter` → reach the region scan's index path. | **A** (subquery/app-side today) **/ B** (optimizer) | integration |

## Detailed improvements (borrowed concept · what · why · how)

Each improvement is framed as: a **concept borrowed** from the system that does it well
(almost always ClickHouse), then **how that concept lands in GreptimeDB's real structure**
(mito2 region engine, DataFusion `=52.1`, Puffin index, OpenDAL) to provide value for
Parallax. Source read at GreptimeDB `v1.0.2` (`0ef5451`).

### Improvement 1 — Full-text query/backend selection + hit-set→row-selection fusion

- **Borrowed concept (ClickHouse):** the `text` index isn't just *stored* well — its
  posting-list lookup and the `hasToken` confirmation run **in the same vectorized
  pipeline**, on warm in-memory structures, over 65k-row blocks. The portable idea is
  "keep the search index hot in memory and hand its matches straight to the scan," not
  "copy ClickHouse's index format" (GreptimeDB's tantivy index is already richer).
- **What:** add an in-memory cache for the tantivy full-text reader/segments, and feed its
  matched row-set directly into the Parquet row-selection so only survivors are decoded.
- **Why — substantially corrected by Run 48:** the reported **~18× warm** gap (Run 12/38) was
  **largely a query-form/backend artifact.** `logs_b1`'s fulltext index is `backend='bloom'`,
  and Run 12 used **`matches()`** (the tantivy *query-syntax* function), which does **not** push
  to a bloom index → **full 5M scan** (EXPLAIN ANALYZE `UnorderedScan output_rows: 5000000`),
  fixed regardless of selectivity (even a 1-match term = ~150 ms). With the **correct pairing —
  `matches_term()`** (exact term) on the bloom index — GreptimeDB **prunes** (scan
  `output_rows: 1`) and selective exact-term search is **~8 ms warm (~2–3× ClickHouse's ~3 ms,
  not 18×)**; broad-term (333k matches) is ~85 ms (~12×, scan-engine territory = #2).
  GreptimeDB already in-memory-caches the inverted/bloom/vector indexes + results
  (`cache/index/`, confirmed live Run 47), so the index *apply* is sub-ms — the residual cost is
  the scan, and only when the index doesn't prune (wrong function) or the term matches many rows.
- **How — reordered by Run-48 impact:** (1) **Tier-A usage fix (no engine change, the real
  answer for the user story):** for exact-term incident grep, use **`matches_term()` + the
  `bloom` backend** — already ~8 ms, competitive with ClickHouse. (2) For **query-syntax/phrase**
  search, use the **tantivy backend** (`backend='tantivy'`) — **Run 49 confirms `matches()`
  prunes there** (selective ~6 ms warm, EXPLAIN `output_rows: 1`), also competitive; no longer
  an open question. (3) **Broad-term** scans (~12×) are
  the scan engine (Improvement #2: bigger batches/JIT). (4) Index→scan fusion
  (`src/mito2/src/sst/index/fulltext_index/applier.rs` → arrow `RowFilter`) still helps sparse
  matches. A dedicated `FulltextIndexCache` is moot — the apply is already sub-ms.
- **Tier:** **A** (usage: `matches`+tantivy or `matches_term`+bloom — the user-story fix) +
  **B** (scan engine, for broad terms). **Integration / usage**, not redesign.
- **User story & clear-winner:** *an SRE paged at 2 a.m. greps logs for a request-id /
  `payment timeout` across a service over the last hour to find the failing path.* **Runs 48-49
  downgrade this from "the one clear-winner-maker" to "already competitive":** with
  `matches()` + tantivy GreptimeDB does selective query-syntax search in **~6 ms**, and with
  `matches_term()` + bloom it does exact-term search in **~8 ms** — *not* 18× slower. So the
  incident user story needs **no engine work**, just the right function/backend (Tier-A usage).
  The big gap only remains for **broad-term analytics** (~12×, scan engine #2), not everyday
  selective incident grep.
- **Value here:** **mostly a usage fix, not a build.** Use `matches()` + `backend='tantivy'`
  for query-syntax/phrase search and `matches_term()` + `backend='bloom'` for exact-term search
  — competitive today, no engine change. Invest in the scan engine (#2) only if broad-term log
  analytics proves frequent. Net: #1 drops
  from "the improvement that flips a user moment" to "configure it right and GreptimeDB is fine
  for incident grep" — strengthening the verdict further.

### Improvement 2 — Bigger execution batch + expression/aggregation JIT + SIMD aggregation

- **Borrowed concept (ClickHouse):** throughput comes from **wide vectors** (65,409-row
  blocks ≈ 8× DataFusion's 8,192), **LLVM-JIT** of expression trees and aggregation
  (`compile_expressions`/`compile_aggregate_expressions`), bespoke **SIMD** kernels, and
  **specialized adaptive hash tables** (two-level for high cardinality). Portable ideas:
  fewer, larger batches; compile hot expressions; specialize aggregation.
- **What:** raise GreptimeDB's RecordBatch size and adopt DataFusion's codegen/SIMD
  aggregation as it matures (or contribute it).
- **Why:** generic scan/aggregate is ClickHouse-faster ~2–4× (Run 11→37: metric agg ~2×
  warm; full scans ~4×, Run 39). Source (pass 77): the query `SessionConfig`
  (`src/query/src/query_engine/state.rs:126-128`) sets only `with_target_partitions` and
  **never `batch_size`**, so DataFusion's 8,192 default holds — small vectors carry more
  per-batch overhead and feed SIMD worse than ClickHouse's 65k blocks.
- **How (code-oriented):** (1) ~~**one-line, cheap to try:** add `.with_batch_size(32768)`~~ — **Run
  124 DISPROVED this as the lever:** ClickHouse at GreptimeDB's 8,192 block is still ~3× faster, so
  raising the batch size alone closes ~nothing. (`state.rs:126-128` still sets only
  `with_target_partitions`, no `batch_size`; `SET …batch_size` rejected — Run 123, no runtime knob —
  but it is not worth the code change for the agg gap.) (2) **The actual lever — expression + aggregate
  CODEGEN and specialized SIMD grouping** — DataFusion's is young/narrow vs ClickHouse's LLVM-JIT;
  this is the real ~2–3× source (Run 124). Track the DataFusion roadmap; GreptimeDB inherits on the
  `datafusion = "=52.x"` bump (`Cargo.toml`). (3) SIMD aggregation kernels: upstream arrow/DataFusion.
- **Tier:** **B** — but the cheap part (batch size) is a **non-lever** (Run 124); the real work is
  JIT/SIMD/codegen, which **rides upstream DataFusion** (slow, not GreptimeDB-owned). **Integration**,
  but the slowest-closing gap in the roadmap.
- **User story & clear-winner:** *a user opens a high-cardinality metric dashboard, or runs
  an ad-hoc "top-20 services by error rate, last 24h".* **Flow pre-aggregation (Tier-A)
  already makes the dashboard fast (Run 43), so this does NOT make GreptimeDB a clear winner
  of the common flow** — only of *unplanned* heavy aggregation, which Parallax users rarely
  run interactively. Footnote unless analytics-heavy usage emerges.
- **Value here:** narrows the heavy-scan/aggregation gap; mostly matters for ad-hoc
  analytics, not the anchored hot path. ~~The batch-size probe is the cheapest experiment in
  this whole roadmap — do it first to size the win.~~ **CORRECTED (Run 124): batch size is NOT
  the lever.** Lowering ClickHouse's `max_block_size` to GreptimeDB's 8,192 barely changed CH's agg
  (~37→~38 ms; even 2,048 = ~43 ms), and CH at 8,192 is **still ~3× faster than GreptimeDB (~38 vs
  ~116 ms)** — so the ~2–3× gap is **independent of block size**. The driver is **JIT-compiled
  aggregation + SIMD hash-agg kernels (the expensive, upstream-DataFusion codegen path)**, not the
  cheap config tweak. Raising GreptimeDB's batch_size alone would **not** close the gap; deprioritize
  the batch-size probe and treat #2 as the slow, DataFusion-core-dependent gap (Run 123: still
  untouched in v1.0.2).

### Improvement 3 — Column-staged late materialization (PREWHERE) via arrow `RowFilter`

- **Borrowed concept (ClickHouse):** **PREWHERE** — decode only the cheap filter columns
  first, build a row mask, then decode the wide columns **only for surviving rows**. The
  portable primitive is "predicate-during-decode + late column materialization," which
  Parquet/arrow-rs already supports.
- **What:** decode filter columns first in the mito2 Parquet reader and late-materialize
  wide/`Json` columns only for rows that pass.
- **Why:** source (pass 77) shows GreptimeDB already prunes **row-groups/pages**
  (`RowGroupSelection` from Puffin indexes + the Parquet **page index**) and then
  **post-decode**-filters rows (`PruneReader::precise_filter`, `src/mito2/src/read/prune.rs:119`)
  — so within a surviving row-group it decodes **all projected columns before dropping
  rows**. There is **no arrow `RowFilter`** in the reader (grep = 0). For wide rows
  (spans/logs with big `message`/`attributes`) that wastes decode on rows the filter drops.
- **How (code-oriented):** in `src/mito2/src/sst/parquet/reader.rs`, build an arrow
  `RowFilter` from the pushed-down `Predicate` (the predicate is already plumbed —
  `predicate(...)` builder at line 185) and attach it via
  `ParquetRecordBatchReaderBuilder::with_row_filter`. arrow-rs ships `RowFilter` /
  `ArrowPredicate` — so this is **wiring an existing primitive**, decode the predicate
  columns first and let arrow late-materialize the rest.
- **Tier:** **B**. **Integration** (no new algorithm).
- **User story & clear-winner:** *a user filters spans/logs by a selective predicate (e.g.
  `status_code = 500`) over a wide window where each row is wide (big `message`/`attributes`).*
  Saves decoding wide columns for rows the filter drops. **Not a clear-winner of a common
  case** — the anchored bundle already prunes to tiny row sets; this only helps *non-anchored*
  selective scans. Footnote (pairs with #1 — the tantivy hit-set becomes the `RowFilter`).
- **Value here:** helps wide-row selective scans (log/trace filters that aren't fully
  anchored); modest for the anchored bundle (already tiny row sets). Pairs with Improvement
  1 (the tantivy hit-set becomes the `RowFilter`).

### Improvement 4 — Shredded (typed-subcolumn) JSON for OTLP attributes

- **Borrowed concept (ClickHouse):** the `JSON` type stores each discovered path as a
  **typed columnar subcolumn**, so `attributes.user` reads exactly one subcolumn with full
  codec/skip benefits. Portable idea: **shred** the dynamic document into columns instead
  of keeping one opaque blob.
- **What:** store hot/declared attribute paths as their own Parquet columns and push path
  access down to a subcolumn scan.
- **Why:** GreptimeDB's `Json` is a **binary blob** (jsonb) read with `json_get_*`
  **per-row parse** — every `attributes.k` filter parses the whole blob for every row, vs
  ClickHouse reading one pre-split subcolumn. **Source-confirmed (pass 80, v1.0.2):** the
  `Json` type is stored via `BinaryVectorBuilder` (`src/datatypes/src/types/json_type.rs`);
  `json_get_*` is a DataFusion **scalar UDF** that calls `jsonb::get_by_path(...)` element-wise
  over the binary column (`src/common/function/src/scalars/json/json_get.rs`); the
  `JsonGetRewriter` (`json_get_rewriter.rs`) is only a **logical function-canonicalization**
  (a DataFusion `FunctionRewrite`), **not** a subcolumn pushdown — there are no subcolumns to
  push to. (v1.0.2 has a `JsonNativeType`→`Struct`/`List` typed *representation* for value
  conversion, but storage stays binary jsonb.) Axis: cost + speed on dynamic-attribute
  queries (Q5-shaped).
- **How (code-oriented):** adopt the emerging **Parquet Variant/shredding** layout: at
  flush in the mito2 SST writer (`src/mito2/src/sst/parquet/`), split declared/hot paths of
  a `Json` column into typed Parquet leaf columns; in the read path, lower
  `json_get_*(attributes,'k')` to a direct subcolumn projection so only that leaf decodes.
  This is the **largest change** here — a storage-format addition — but Parquet's variant
  work makes it an integration along an established spec, not a from-scratch type.
- **Tier:** **B**. **Integration / format** (borders on design — flag if it grows).
- **User story & clear-winner:** *a user groups errors by `http.status_code`, or filters by
  an arbitrary, undeclared OTLP attribute (`attributes.http.route = '/checkout'`) across last
  week.* Per-row jsonb parse hurts at volume. **Clear-winner only for unplanned
  arbitrary-attribute analytics** — the Tier-A answer (promote hot attributes to real columns)
  wins the planned/common case. Footnote unless attribute exploration is core to the product.
- **Value here:** only matters if Parallax does heavy arbitrary-attribute filtering at
  volume; for declared hot attributes, promoting them to real columns in the schema (Tier-A
  today) captures most of the benefit without the format work.

### Improvement 5 — Alternate physical ordering (projection-equivalent)

- **Borrowed concept (ClickHouse):** a **projection** stores a *second* physical `ORDER BY`
  inside each part, optimizer-picked transparently → one table serves two access orders
  (e.g. `service`-time **and** `trace_id`) at sequential-scan speed. Portable idea: keep a
  second physically-sorted copy and pick it automatically.
- **What:** give GreptimeDB an alternate-ordering structure for tables queried on two keys.
- **Why:** GreptimeDB's secondary indexes give *positions*, not a second physical sort
  order (Run 28, `projections-and-access-paths.md`) — a scan on a non-primary key can't get
  ClickHouse's projection locality. **Source-confirmed absent (pass 82, v1.0.2):** no
  `PROJECTION` keyword in `src/sql/src/parsers/{create_parser,alter_parser}.rs` (grep = 0),
  and the `AlterTableOperation` enum has no projection/alternate-ordering variant (only
  Add/Drop/ModifyColumn, Rename, Set/UnsetTableOptions, Set/UnsetIndex{Fulltext,Inverted,
  Skipping}); **live-reconfirmed (Run 65, v1.0.2): an `order_by` table option is rejected
  ("Unrecognized table option key")** — there is genuinely no engine-native second physical
  order. **The deeper root (Run 63): GreptimeDB's `PRIMARY KEY` *is* its sort key *and* its
  series identity** — so clustering by a high-card anchor like `trace_id` (which *would*
  speed reads — proven 39 ms scattered → 14 ms trace_id-clustered, Run 63) is impossible
  without making `trace_id` the PK, which **explodes series cardinality** (71k+ traces). The
  ClickHouse concept being borrowed is precisely the **decoupling of physical sort order
  (`ORDER BY`) from row/series identity** — CH clusters by the anchor at **zero cardinality
  cost** because it has no series model; GreptimeDB cannot.
- **Why it now matters more than "scan order" (Run 55/63 — cold-read egress):** the missing
  alternate ordering is *also* the cause of GreptimeDB's **cold-selective-read egress** loss
  — with `trace_id` scattered (not the sort key), an anchored **cold** read touches ~all row
  groups → pulls ~the whole SST from object storage (Run 55: ~23 MiB vs ClickHouse's ~294 KiB
  granule read). So this improvement closes *two* gaps: alternate scan locality (Run 28)
  **and** cold-tier anchored-read egress (Run 55/63).
- **User story & clear-winner:** *(a)* a user wants a service's span timeline (`service`-time)
  **and** the trace bundle (`trace_id` order) fast; *(b)* an SRE re-opens a **months-old,
  cache-evicted** incident and greps one `trace_id` from cold object storage. **Does it make
  GreptimeDB a clear winner? Only for case (b), and only when reads are genuinely cold** —
  GreptimeDB's persistent local read cache (Run 55) keeps **recent/hot** data warm, so the
  anchored hot path on recent bundles already serves locally at ~0 S3. So this stays a
  **footnote unless Parallax's cold-tier *selective* re-read pattern is frequent** — the
  common case (recent, cache-warm) doesn't bite. Honest: most-improved is a rare cold case.
- **How (code-oriented):** (a) **Tier-A today:** maintain a re-sorted derived table with a
  **Flow** (`CREATE FLOW … SINK TO alt_table` keyed on the alternate order — Flow verified
  Run 43); the app/optimizer queries whichever table matches — *a `trace_id`-sorted copy
  also fixes the cold-read egress (anchored cold reads hit the clustered copy → prune to
  ~1 row group)*. Extra storage, no engine change. (b) **Tier-B native parity:** have the
  mito2 SST writer emit an alternate-sorted copy per region + region metadata + a DataFusion
  planner rule to auto-pick — a larger build. *(Note: a true secondary sort key on the
  primary data — decoupling sort from the PK/series identity — would be a region-model
  **redesign**, not integration; the alternate-sorted **copy** sidesteps that, staying
  integration.)*
- **Tier:** **A** (Flow workaround) / **B** (native copy). **Integration** (via copy; full
  sort/identity decoupling would be design).
- **Value here:** **downgraded further (Run 87): GreptimeDB already has a cheap
  anchor-locality lever — `PARTITION ON COLUMNS(trace_id)`** — that partly closes the
  cold-read motivation *without* this improvement. Measured: an anchored read on a
  trace_id-partitioned table prunes to ~1/N partitions (11 ms vs 39 ms at 8-way; ~1/N cold
  egress) at no PK-cardinality cost, and the native `opentelemetry_traces` ships 16-way by
  default (Run 86). So the residual that *only* a true alternate-sort (#5) would add is
  granule-level (vs partition-level) anchor locality — a small increment over partitioning.
  Combined with partitioning being free, #5 is a **footnote**; invest only if a real query
  mix proves frequent cold selective re-reads that 16-way partitioning + the read cache
  don't already handle.

### Improvement 7 — Type-aware Parquet encodings (per-column codec parity)

- **Borrowed concept (ClickHouse):** per-column **`CODEC()` chains** — `Gorilla` for float
  gauges, `DoubleDelta` for monotonic counters/timestamps, `T64`, etc. The portable idea is
  "pick the encoding that matches each column's *shape*," and it is general: **Parquet already
  defines equivalent column encodings** (`BYTE_STREAM_SPLIT` for floats, `DELTA_BINARY_PACKED`
  for monotonic ints, dictionary/RLE) — so this is not ClickHouse-specific.
- **What:** select type-aware Parquet encodings for user data columns (and optionally expose a
  per-column codec option in DDL), instead of defaulting every data column to `PLAIN` + ZSTD.
- **Why:** compression is mostly a **wash** (GreptimeDB's auto Parquet+ZSTD ties or beats
  ClickHouse out-of-the-box on logs/dict-friendly columns — Run 10), **but** ClickHouse wins
  where a specialized codec matches: float gauges via `Gorilla` (Run 4: gauge 84.7 KiB from
  6.59 MiB raw = **78×**) and monotonic counters via `DoubleDelta` (7.3×). Source (pass 81):
  GreptimeDB's writer (`src/mito2/src/sst/parquet/writer.rs`) **already** customizes internal
  columns (`:387-391` — `ts`/`seq` → `DELTA_BINARY_PACKED`, the DoubleDelta-equivalent for the
  time index, which is why GreptimeDB's `ts` compression is competitive) but sets user data
  columns to **`Encoding::PLAIN` + ZSTD** (`:433-434`). A flat/slow-moving `Float64` gauge
  therefore misses the Gorilla-class win ClickHouse gets. Axis: cost (#2), and only the
  numeric-metric columns — second-order for Parallax's overall bytes.
- **How (code-oriented):** extend `customize_column_config` (`writer.rs:371`, already the place
  that sets `set_column_encoding(...)` per internal column) to choose encodings by **column
  type/semantic**: `Float64` field → Parquet **`BYTE_STREAM_SPLIT`** (splits mantissa/exponent
  byte planes → ZSTD compresses flat floats far better, the Gorilla analog); monotonic integer
  field → `DELTA_BINARY_PACKED`; low-card string → keep dictionary. The writer already imports
  `parquet::basic::Encoding` and calls `set_column_encoding`, so this is **filling in a
  per-column policy that the code is structured for** (there is even a `TODO` at `:430` to set
  proper encodings for internal columns). Optionally add a `CREATE TABLE … col Float64
  CODEC(...)`-style DDL option mapping to the same `Encoding`.
- **Tier:** **B**. **Integration** — Parquet ships the encodings; the writer already does
  per-column config; this extends the policy to user columns.
- **User story & clear-winner:** **no user story — invisible to users.** It surfaces only as
  a smaller cloud storage bill on metric-float volume, never as a faster query or a capability
  a Parallax user reaches for. **Does not make GreptimeDB a clear winner of any user-felt
  case** — pure second-order cost footnote.
- **Value here:** low/second-order. Compression is a near-wash and a 1.3–1.9× local-disk delta
  is not the cost driver (object-store request economics dominate — `compression-and-cost.md`).
  Worth it mainly for **metric float columns** if/when retained metric volume is large;
  otherwise leave GreptimeDB's zero-tuning default, which is itself an ergonomics win.

### Improvement 6 — Vertical ceiling + analytical/merge maturity

- **Borrowed concept:** none portable — this is ClickHouse's decade of single-box scan
  tuning and battle-tested merges.
- **What / How:** inherited via Improvement 2 (engine throughput) plus time and
  production hardening; not a discrete feature to implement.
- **Tier:** **C** (accept or wait). **Maturity**, not integration.
- **User story & clear-winner:** *a user runs a large single-node deployment into high
  retained data, high query concurrency, or merge/compaction pressure and expects predictable
  p99 behavior without moving to a distributed topology.* This is a real operator pain, but
  it is not a GreptimeDB feature Parallax can add. **It does not make GreptimeDB a clear
  winner; it is the main ClickHouse maturity caveat to measure or accept.** GreptimeDB's
  answer is horizontal scale-out plus object storage, so the practical gate is whether the
  Parallax tiny/small tier ever reaches this ceiling before it wants scale-out anyway.
- **Value here:** GreptimeDB's answer is horizontal scale-out + object storage (the
  verdict's scaling pillar), not chasing ClickHouse's vertical ceiling.

### Improvement 8 — Push an equality predicate into an *indexed* join input

- **Borrowed concept (ClickHouse):** push a join input's filter into its scan and let the
  scan's index prune *before* the join runs. ClickHouse's Q4 cross-tier join prunes the
  anchored side to `Granules 1` + PREWHERE → reads ~nothing before joining (Run 30/81).
  Portable idea: a join input that has an equality predicate on an indexed column should
  use the index, not full-scan.
- **What:** make GreptimeDB's optimizer push a join input's equality filter
  (`s.trace_id='X'`) down to the `spans_idx` `TableScan` **as an index-eligible predicate**,
  so the inverted index prunes the join input instead of full-scanning it.
- **Why:** **Run 81/82** — a direct cross-tier `spans ⋈ error_events` join anchored on
  `trace_id` **full-scans all 1M spans** on GreptimeDB (`EXPLAIN ANALYZE` `output_rows:
  1,000,000`, ~54 ms) vs ClickHouse ~4 ms; confirmed for **both INNER and LEFT** joins,
  while the *standalone* `WHERE trace_id='X'` prunes to `output_rows: 14`. So the inverted
  index works — it is just **not consulted when the table is a join input**: the pushed
  filter lands as a post-scan `FilterExec` on the `MergeScanExec` output rather than as an
  index-eligible scan predicate. Axis: speed (#1), on cross-tier correlation.
- **How (code-oriented):** in the query optimizer (`src/query/src/optimizer` + the
  DataFusion `push_down_filter` rule), ensure a predicate pushed to a join input reaches the
  region/`MergeScanExec` scan in its **index-applicable filter set** (the same path a plain
  `WHERE` takes), not merely as a `FilterExec` above the scan. The subquery rewrite
  (`FROM (SELECT * FROM spans_idx WHERE trace_id='X') s …`) already lands the filter as the
  scan's own → prunes (`output_rows: 14`, ~21 ms, Run 81), which localizes the fix to
  "make the join-pushed filter take that same index path."
- **Tier:** **A** (today: subquery pre-filter, or app-side correlation — Parallax's pattern)
  **/ B** (optimizer-rule fix upstream). **Integration** — the index exists and works; this
  is pushdown plumbing into the join-input scan, not architecture.
- **User story & clear-winner:** *an SRE/agent runs a cross-tier correlation as one in-DB
  join (`spans ⋈ error_events ON trace_id` during an incident).* On GreptimeDB the direct
  join full-scans (slow, and worse at volume); the fix makes it prune. **But Parallax's
  evidence-bundle assembly is *app-side* (anchored fetch each signal + join in app, Q6 =
  Q1+Q2+Q3 not in-DB joins), so this rarely bites — footnote-priority unless Parallax adds
  direct in-DB cross-tier joins.** Real win for in-DB-join users; for Parallax, the Tier-A
  app-side pattern already sidesteps it.

## The three tiers (the decision-useful summary)

- **Tier A — close it in Parallax today, no engine change.** Index `trace_id`/`fingerprint`
  (Run 45 design), **Flow pre-aggregation** for dashboards (Run 43, ~5–6× read speedup,
  neutralizes the ~2× SQL-agg gap), **SQL not PromQL** for hot aggregations (Run 44: GT SQL
  ~5× faster than GT's own PromQL), tantivy fulltext + `matches()` or bloom fulltext +
  `matches_term()` for selective log search, and the Flow
  alternate-ordering workaround (#5a). **This already makes GreptimeDB a clear winner for
  Parallax's *anchored* workload** — the gaps below only bite on heavy *ad-hoc* analytics.
- **Tier B — contribute upstream (Rust, open-source) to win *all* cases.** Items 1–5 (and
  #7 only if retained metric-float cost becomes material): batch size, JIT, SIMD, PREWHERE
  late materialization, JSON shredding, native projections, index↔scan fusion, and type-aware
  Parquet encodings. Items 1–5 are what "clear winner even for large-scale ad-hoc log/trace
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

**Gap-closing is already happening, live-verified (Run 106 / `vendor-claims-audit.md`):**
GreptimeDB's RC2 **"100× TopK"** (dynamic filter pushdown into the Mito scan, built on **DataFusion
runtime dynamic filters**) is present in our `v1.0.2` — `ORDER BY … LIMIT 10` on 1M = ~20 ms, not a
full sort. **Flat SST** (v1.0 GA default) is a shipped high-cardinality scan-format redesign (write
~4×, TSBS query latency up to ~10×). And **JSON Type v2** (field-level index / dynamic fields) is
**roadmap-committed for v1.1 / Q2 2026** — directly narrowing #4 (the dynamic-attr gap, **~8–12× with
the typed-subcolumn cast** — Runs 129/130; the ~57× at Run 104 was ClickHouse 26.5's lax no-cast path,
removed in 26.6; GT v1.1-nightly shows no change yet, so JSON Type v2's win is still owed to v1.1 GA).
**Honest caveat:** #2's JIT/SIMD, #3 PREWHERE, #5 projections, #8 join-pushdown
are **not** explicit GreptimeDB-roadmap line items — they ride upstream DataFusion + opportunistic
release wins (as TopK did). Engineering, not physics — but partly community-paced, not solely
GreptimeDB-owned.

## Open questions handed to the benchmark

- Quantify the Run-48/49 residuals: broad-term `matches_term()`/`matches()` latency at
  larger/cold scale, especially `SELECT *` shapes rather than `count(*)`.
- Does raising the DataFusion batch size in `SessionConfig` measurably narrow the ~2× agg
  gap at 40k series? (A cheap local probe — propose as a new case.)
- JSON-shredding payoff: attribute-path query latency, blob-parse vs subcolumn — at volume.

## Source / evidence

- Mechanisms: `query-execution-engine.md` (batch/JIT/SIMD/PREWHERE), `indexing-internals.md`
  (Puffin/tantivy vs `text`, integration-not-format), `read-path-indexing-and-execution.md`
  (pushdown/skip), `schema-evolution-and-dynamic-columns.md` (JSON blob vs subcolumn),
  `projections-and-access-paths.md` (Run 28), `rollup-and-continuous-aggregation.md` (Flow).
- Empirical: `local-benchmark-results.md` Runs 11/12/37/38 (the gaps), 43 (Flow), 44 (PromQL
  vs SQL), 45 (GreptimeDB schema build), 47 (full-text gap = post-index scan, not the index
  apply — index apply ~0.15 ms via `greptime_index_apply_elapsed{type=fulltext_index}`),
  48 (`matches()` on a `backend='bloom'` index full-scans 5M, EXPLAIN `output_rows: 5000000`;
  `matches_term()` prunes, `output_rows: 1` → selective exact-term ~8 ms / ~2–3× CH, not 18×),
  **49 (tantivy backend `matches()` prunes too, `output_rows: 1`, selective ~6 ms — both
  full-text paths fast with correct backend; ~18× was 100% a pairing artifact)**.
- Source (pass 77, v1.0.2): `src/query/src/query_engine/state.rs:126-128` (SessionConfig =
  `with_target_partitions` only, no `batch_size`); `src/mito2/src/sst/parquet/reader.rs`
  (`RowGroupSelection` + `PageIndexPolicy` + `prune_row_groups_by_{fulltext,inverted}_index`;
  **no `RowFilter`**); `src/mito2/src/read/prune.rs:119` (`PruneReader::precise_filter` =
  post-decode row filtering).
- Source (pass 78, v1.0.2): `src/mito2/src/cache/index/` = `{inverted_index, bloom_filter_index,
  vector_index, result_cache}.rs` (**no `fulltext_index` cache**); `src/mito2/src/cache.rs`
  (`InvertedIndexCache`/`BloomFilterIndexCache`/`VectorIndexCache`/`PuffinMetadataCache`);
  `src/mito2/src/sst/index/fulltext_index/applier.rs` (`TantivyFulltextIndexSearcher` over
  `SstPuffinDir` + `dir_cache_hit/miss`; bloom variant uses `BloomFilterIndexCacheRef`).
- Source (pass 80, v1.0.2): `src/datatypes/src/types/json_type.rs` (`Json` via
  `BinaryVectorBuilder` = jsonb binary); `src/common/function/src/scalars/json/json_get.rs`
  (`jsonb::get_by_path` per-row scalar UDF) + `json_get_rewriter.rs` (`JsonGetRewriter` =
  logical `FunctionRewrite`, not subcolumn pushdown). Live: `SET …batch_size` →
  `Unsupported set variable` (no runtime knob for #2).
- Source (pass 81, v1.0.2): `src/mito2/src/sst/parquet/writer.rs` — `customize_column_config`
  (`:371`) sets internal-column encodings (`:387-391` `ts`/`seq` → `DELTA_BINARY_PACKED`,
  `op_type` → UNCOMPRESSED) but user data columns default to `Encoding::PLAIN` + ZSTD
  (`:433-434`); `TODO` at `:430`. Confirms #7: no Gorilla-class float encoding on user columns,
  no per-column codec DDL.
- Source (pass 82, v1.0.2): `src/sql/src/parsers/{create_parser,alter_parser}.rs` — **no
  `PROJECTION` keyword** (grep = 0); `AlterTableOperation` = Add/Drop/ModifyColumn, Rename,
  Set/UnsetTableOptions, Set/UnsetIndex{Fulltext,Inverted,Skipping} — no projection/alternate-
  ordering op. Confirms #5: no engine-native second physical order.
- Decision context: `verdict-which-to-choose.md`. Loop target: `prompts/greptimedb-vs-clickhouse-internals.md`
  ("Closing The Gap").
