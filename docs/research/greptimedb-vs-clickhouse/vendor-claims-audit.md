# Vendor-Claims Audit — greptime.com Comparison Page + Blogs vs Our Findings

<!-- markdownlint-disable MD013 -->

Status: created 2026-05-25 (Run 106). The operator's GreptimeDB-over-ClickHouse leaning was
partly anchored on GreptimeDB's own marketing pages — chiefly
[`greptime.com/compare/click_house`](https://greptime.com/compare/click_house) plus a set of
blogs. He asked for a **clarity/accuracy/manipulation audit**: do these pages hold up against our
105 independent local benchmark runs, or did marketing frame the decision? Goal — decide on
**tech-proven numbers, not marketing**. This note rates each load-bearing vendor claim
(`AGREES` / `CONFLICTS` / `EXAGGERATES` / `OMITS-CONTEXT` / `NEW` / `UNVERIFIABLE`) against our
data, through the Parallax lens, and folds in what the pages got right that we had missed.

Pins for our side: GreptimeDB `v1.0.2`, ClickHouse `v26.5.1.882` (105 runs, `local-benchmark-results.md`).

## Headline verdict — the page did NOT manipulate the decision toward a wrong conclusion

**The compare page sells GreptimeDB on *fit, storage, economics, and native protocols* — exactly
where our independent benchmarks also place GreptimeDB's wins — and it conspicuously does NOT claim
raw-query-speed superiority over ClickHouse** (the one claim that would contradict our data). The
companion log-monitoring blog even **explicitly concedes ClickHouse is faster** on unstructured
keyword search. So the *load-bearing direction* of the vendor material is **accurate and consistent
with our verdict** ("GreptimeDB on fit, not speed"). The decision does not rest on a manipulation.

Where the pages DO spin, it is **peripheral to the Parallax decision** and, importantly, the spin
runs toward "GT is comparable / cheaper / simpler," never a false "GT is faster on analytics." The
few "GT faster" numbers that *do* appear are exactly the kind our runs already neutralized
(unstated index/config) — so the correct response is to **discount those specific numbers and keep
relying on the fit/economics/direction claims**, which are solid.

## The compare page (`/compare/click_house`) — claim-by-claim

| Vendor claim | Type | Vs our findings | Note |
| --- | --- | --- | --- |
| Log storage ~50% larger on CH (2.6 GB vs 1.3 GB); compression 26% vs 13% | benchmark | **AGREES** | Matches our storage-fit direction (Run 100: GT wins high-card metrics; logs ~wash but GT competitive). Single unstated dataset — direction fair, exact number not reproducible. |
| Timestamp-first layout vs "time is just another column" | mechanism | **AGREES** | Code-confirmed; GT's genuine time-series FIT advantage. |
| SQL + **GA PromQL** vs CH "experimental PromQL (rate/delta/increase)" | feature | **AGREES** | Matches Runs 23/44/50/62/105 + `promql-and-metrics-query.md`. Decision-relevant, fair. |
| Native **OTLP** all-signals + native **Jaeger** query API vs CH collector/plugin | feature | **AGREES** | Matches Runs 25/32. CH OTLP is collector-only, Jaeger external plugin. Fair, core fit edge. |
| Dynamic schema (auto-add columns) vs CH `ALTER TABLE` | feature | **AGREES** but **OMITS-CONTEXT** | True (Run 18) — but the page omits that **CH is 13–57× FASTER querying JSON attribute paths** once present (Run 104). Half the story. |
| "No middleware" / "4–8 components vs 1 database" | marketing-puff | **OMITS-CONTEXT** | Real operational-simplicity point, but cherry-counted; CH ingest-buffering is a common pattern, not a hard requirement, and GT benefits from buffering at scale too. |
| Poizon case: "P99 latency seconds → milliseconds" | benchmark | **OMITS-CONTEXT (misleading)** | **Strongest spin.** This compares GT against a *multi-stage ETL pipeline*, **not** against tuned ClickHouse. Do not read it as a GT-vs-CH speed result. |
| "Replace ClickHouse in under a week" + migration steps | marketing-puff | irrelevant | Migration-effort claim, untestable, not decision-relevant. |
| (Absent) any "GT faster on analytical queries" claim | — | **notable** | The page does **not** make the one claim our data would refute. Tells against the "manipulation" worry. |

## Blogs — load-bearing claims

| Source | Claim | Vs our findings | Note |
| --- | --- | --- | --- |
| **Log-monitoring** (2025-04-01) | Unstructured keyword search ~2547 ms GT vs ~2080 ms CH — "ClickHouse is faster" | **AGREES** | Honest concession; matches our broad-term ~12× direction. Deflects to lower CPU%. |
| Log-monitoring | Structured keyword 22.8 ms GT vs 52 ms CH; COUNT 6 vs 46 ms (GT faster) | **CONFLICTS** | **No index/tokenizer/query config disclosed** → the same GT-indexed-vs-CH-default / `matches()`-vs-`matches_term()` artifact we diagnosed (Runs 48–49). Unreproducible; discount. |
| Log-monitoring | Write TPS GT 185k vs CH 166k; lower CPU/mem on GT | **NEW** | We didn't measure head-to-head write TPS; plausible, no hardware/config stated. Economics angle. |
| Log-monitoring | JSONBench 1B docs: GT **1st cold, 4th hot** | **AGREES** | Cold win = object-store/scan-startup fit; "4th hot" quietly concedes CH-class engines win **warm** — consistent with our "CH faster warm." Hero-framed on cold only. |
| **Ingestion-protocol benchmark** (2026-03-24, **v1.0 GA**) | gRPC Bulk(Arrow) **2.68M rows/s** (peak 3.3M); SDK 1.17M; InfluxDB-LP 889k; OTLP-Logs 621k | **AGREES** | Confirms our ">1M rows/s bulk" + native multi-protocol (no collector). Single-node M4 Max, disclosed as relative-not-absolute. |
| Ingestion benchmark | gRPC Bulk **+37.3% FASTER at 1M vs 100k series**; SST size flat ~491 MB regardless of cardinality | **AGREES (key)** | Directly corroborates our **ingest cardinality-insensitivity** win (Runs 84/101) — on **v1.0 GA**. Strongest independent confirmation of a GT pillar. |
| Ingestion benchmark | MySQL/Postgres INSERT ~72k rows/s ("up to ~37× slower") | **OMITS-CONTEXT** | The 37× rides a disclosed SQL-at-fixed-low-concurrency handicap; vendor admits pool size fixes it. Mild self-serving framing, disclosed. |
| **TimescaleDB benchmark** (2025-12-09, **beta.2**) | GT wins 13/15 TSBS queries, "up to 67×"; 18× compression | **EXAGGERATES / OMITS-CONTEXT** | TSBS cpu-only is aggregation-heavy → favors columnar GT vs **row-store Postgres**; says **nothing** about GT-vs-ClickHouse (both columnar; our runs show **CH beats GT** on these shapes). Don't read as GT-absolute-fast. |
| TimescaleDB benchmark | GT **LOSES lastpoint 8.7×**, groupby-orderby-limit 6× to TimescaleDB | **AGREES (telling)** | GT's lastpoint/point-lookup is **not** inherently fast — matches our finding that the anchored path is a *tie only because both are ≪ the 300 ms gate*, not because GT is fast. Honest of them to include. |
| **Prometheus-3** (2026-05-09) | PromQL implemented **in Rust** (own `promql-parser` + DataFusion engine, not a Go wrapper); ~100% compliance | **AGREES** | Matches our native-PromQL finding; the Rust/DataFusion stack is the contributable-engine reason behind DQ6. Compliance % is NEW/plausible. |
| Prometheus-3 | Remote Write 1.0 GA; **RW 2.0 NOT GA**, exemplars unsupported | **NEW (honest gap)** | Good-faith disclosure of limits. |
| Prometheus-3 | "60% fewer bytes / 90% fewer allocations" | **OMITS-CONTEXT** | Those are **Prometheus's own RW2.0** numbers, not GreptimeDB's. Not a GT win. |
| **Observability 2.0** (2025-04-25) | "Unified wide events" — store raw high-fidelity events, derive metrics/logs/traces | **AGREES (direction)** | Substantive thesis (label is contested even by its originator). **Aligns strongly with Parallax** (multi-signal-one-engine; evidence-bundle-by-`trace_id` *is* the wide-event pattern for AI debugging). |
| **Agent-friendly infra** (2026-04-08) | **MCP Server v0.3** (SQL/TQL/PromQL, read-only, masking), prompt templates, Agent Skills, GenAI observability (OTel `gen_ai.*`, cross-signal JOINs, wide events) | **NEW (direction, hot)** | Shipped, not just blogged. Strongest **direction-alignment** with Parallax's AI-native goal. Read-path agent tooling, not a hot-path speed feature. |
| **Coroot** (2025-08-21) | "billions of index points per second" | **UNVERIFIABLE / EXAGGERATES** | No methodology; undefined unit. Puff. (eBPF/APM is Coroot's, not GT's.) |
| **DeepSeek-AI** (2025-02-11) | — | **irrelevant** | DeepSeek is the *monitored app*, not AI-inside-GT. GreptimeDB has **no** AI intelligence layer (no text-to-SQL/anomaly/RAG) — that layer is **Parallax's to build**. Logo/tutorial piece. |
| Unified-metrics+log (2025-01-26) | — | thin | Feature tutorial, zero benchmarks/comparison. Promotional-by-omission. |

## What the pages got right that we had MISSED or under-recorded (fold into our notes)

1. **GreptimeDB has a *disk* index-file cache distinct from its in-memory index caches** (index-cache
   guide). We had documented only the in-memory inverted/bloom/vector + index-result cache
   (`cache/index/`). There is *also* an on-disk independent index-file cache (since v1.0.0-beta.1),
   preloaded newest-first, default ~20% of write cache, tunable `index_cache_percent`. Refines
   `caching-and-cold-warm.md` / `indexing-internals.md`. (Net: GT's cold-tier index story is a bit
   stronger than we recorded.)
2. **OTel-Arrow is Phase-2 / experimental, NOT GA** (OTel-Arrow-Rust post). Only baseline OTLP
   (HTTP/gRPC) is GA. Do not claim OTel-Arrow as a shipped advantage. (Recorded in
   `public-performance-claims.md`.)
3. **Shipped scan-engine gap-closing, live-verified on our v1.0.2:**
   - **"100× TopK" (RC2): dynamic filter pushdown into the Mito scan** (`ORDER BY … LIMIT`
     28.86 s → 0.21 s in their test). **Re-verified live (Run 106): `ORDER BY duration_ms DESC LIMIT
     10` on 1M rows = GT ~20 ms vs CH ~7 ms (~3×, both ≪ 300 ms)** — GT is *not* full-sorting, so the
     TopK pushdown **is in our v1.0.2 binary** and our benchmarks already benefit. Built on
     **DataFusion runtime dynamic filters** — exactly the "closable via the DataFusion roadmap"
     mechanism behind DQ6. This is independent confirmation, not just a blog claim.
   - **Flat SST default (v1.0 GA): write ~4×, query latency up to ~10× on TSBS @2M series** — a
     scan-format redesign for high cardinality. Since our containers are v1.0.2 GA, our scan numbers
     (Run 102, etc.) already include it.
4. **JSON Type v2 (field-level index, dynamic fields) is committed for v1.1 / Q2 2026** (roadmap).
   This is the **JSON-shredding gap (parity-roadmap #4) being addressed upstream** — directly
   relevant since Run 104 found the dynamic-attr JSON gap had *widened* to ~57×. The gap that widened
   is on the roadmap to narrow.

## Honest caveat to the DQ6 "gaps are closable" thesis (the pages also temper it)

The **published 2026 roadmap does NOT explicitly commit** to JIT/SIMD vectorization, PREWHERE,
join-input pushdown, or projections/alternate-orderings — the specific gaps in `greptimedb-parity-
roadmap.md`. Those ride **upstream DataFusion**, not GreptimeDB-owned roadmap items. This is a real
softening of the thesis: closing several of our gaps depends on the broader Arrow/DataFusion
community, not GreptimeDB's committed plan. **Counter-evidence that it still works:** the 100× TopK
and Flat-SST wins shipped *without* being roadmap-headlined (via DataFusion dynamic filters + an
internal SST redesign), proving GreptimeDB does land such scan-engine wins. So the thesis holds —
the gaps are engineering, not physics — but the *delivery path* is "DataFusion upstream + opportunistic
GreptimeDB release wins," not "a GreptimeDB roadmap line item per gap." DQ6 updated accordingly.

## Bottom line for the Parallax decision

- **No reason to flip toward ClickHouse from this material.** The vendor pages reinforce, not
  undermine, our verdict: GreptimeDB is sold — and independently measures — as a **fit / storage /
  economics / native-protocol / direction** choice, not a raw-speed one. The pages never claim the
  analytical-speed superiority our data would refute.
- **Discount the few "GT faster" query numbers** (structured-keyword, Poizon, TimescaleDB 67×) —
  they are unstated-config, GT-vs-non-ClickHouse, or workload-stacked. Our runs are the authority on
  GT-vs-CH speed: **CH faster on raw analytics, both interactive on Parallax's anchored hot path.**
- **The direction signals are genuine and strengthen DQ6**: native-Rust PromQL/OTLP/Jaeger, the
  unified-wide-events thesis (mirrors Parallax's design), shipped agent/MCP tooling, and *live-proven*
  scan-engine gap-closing (TopK on our v1.0.2). The bet — GreptimeDB on fit + a closable, Rust-
  contributable engineering gap — is intact, with the roadmap-commitment caveat above.

## Cross-refs

`verdict-which-to-choose.md` (DQ6, the investment decision), `greptimedb-parity-roadmap.md` (the gap
list + closability), `public-performance-claims.md` (the broader public-claims triangulation),
`promql-and-metrics-query.md` (Run 105), `metric-cardinality.md` (Runs 84/101), `local-benchmark-results.md`
(Run 106). Vendor sources audited: the compare page + 15 blogs (2025-01 through 2026-05).
