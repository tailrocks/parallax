# Verdict — Which To Choose, And Why

<!-- markdownlint-disable MD013 -->

Status: standing decision, continually sharpened (current through **pass 103**; passes 86–87 /
Runs 48–49 **dissolved most of the full-text gap** — the ~18× was a backend/function
misconfiguration: with the correct pairing, selective full-text is ~6 ms (tantivy+`matches`)
and ~8 ms (bloom+`matches_term`) vs ClickHouse ~3 ms; residual is broad-term analytics only —
see the flip-trigger correction below. **Pass 103 folded in Runs 55–66:** added two ClickHouse
edges — **cold *selective* object-store reads** (scatter-vs-cluster, Runs 55/63) and
**dynamic-attribute path queries ~13×** (Run 61) — and refined the metric-agg gap to **~2–3×
warm** (Run 67). Offsetting GreptimeDB wins re-confirmed: full-text cost tie (Runs 51–52),
concurrent-ingest non-blocking (Run 53), object *count* + warm-cache re-reads (Runs 54–55),
Q6 not-latency-bound (Run 56), native zero-DDL ingest (Run 57), upsert/DELETE ergonomics
(Runs 59/66), PromQL GA (Run 62), cheap retention (Run 64). None flip the recommendation.)
Synthesizes the internals teardowns (all 10 subsystems + rollup, retention,
schema-evolution, dedup, WAL/durability, execution-engine, indexing, PromQL, metric
cardinality, span-tree, projections, deletes/mutations, async-insert, zero-copy
replication), the per-signal matrix, Docker Runs 1–46, and public-claims triangulation.
The runnable `storage-benchmark-prototype.md` holds final veto; this verdict states the
mechanism-grounded recommendation and the triggers that would flip it. **The white-box
smoke comparison is now comprehensive** — all 10 checklist subsystems, every named
ClickHouse/GreptimeDB lead, the Q1–Q6 evidence-bundle set, and all 9 public claims are
covered; **both implementation designs are now verified buildable live** (Runs 45–46); the
remaining open questions are **harness-gated** (scale/cold/multi-node), listed below. Pins
re-verified current through pass 75 (no newer stable on either side: GreptimeDB v1.1.0 is
nightly-only; ClickHouse 26.5.x is the highest *feature* line — newer-dated 26.3/26.4 tags
are older-line LTS/backport patches).

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`).

## Headline

**Recommended: GreptimeDB** — but **not because it is the fastest engine. It is
not.** ClickHouse is faster for high-volume log/trace analytics, by clear,
code-confirmed mechanisms. GreptimeDB is the recommendation because its *design
aligns with Parallax's dominant axes*: metrics/PromQL-native, fresh-on-write with
small-write ingest ergonomics, horizontal scale-out designed-in, object-storage
native, and Rust (tiebreak). This is a **fit decision, not a speed decision** —
and the honest correction to the operator hypothesis below makes that explicit.

**A second lens — the long-term *investment* decision (DQ6) — reaches the same answer
from a different direction.** ClickHouse's speed lead is a **closable engineering gap, not
a physics wall** (the parity-roadmap's per-gap test finds no architectural wall; the two
heaviest gaps ride the shared DataFusion + Parquet-Variant roadmaps), and GreptimeDB is the
**Rust, open-source substrate the operator can actually contribute to** — whereas ClickHouse's
C++ engine is one he can only wait on. So the long-term bet *reinforces* the fit choice:
GreptimeDB's deficits are engineering/time, not physics, and they are closable in the
language the operator will invest in. Full reasoning in **Decision question 6** below.

**The "fit not speed" thesis is now anchored on the query that matters most.** Pass
35 measured the full anchored evidence-bundle composite (Q6 = Q1+Q2+Q3, Run 16): CH
~10 ms vs GreptimeDB ~33 ms, **both far under the 300 ms gate** — so for Parallax's
dominant retrieval, **engine choice is not latency-bound**. **Re-verified Run 99 (no drift,
numbers better): CH ~5 ms / GT ~16 ms** on the 3-signal bundle for one trace_id (warmer
containers; same ~3× ratio, both still ≪ 300 ms) — the load-bearing anchor reproduces. The
decision therefore rests on the *fit* pillars below (metrics-native, ingest/upsert ergonomics,
retention cost, scaling), exactly where GreptimeDB leads — not on the analytical-scan latency
where ClickHouse leads but which Parallax's anchored pattern rarely hits.

## The operator hypothesis, tested honestly

> Hypothesis: "GreptimeDB will be the fastest, then ClickHouse."

**Refuted as a raw-speed claim; partially true on a capability basis.**

- On **query latency** for logs, traces, and generic analytical scans, **ClickHouse
  is faster** — finer 8,192-row granule vs GreptimeDB's 102,400-row Parquet row
  group, PREWHERE late materialization, a mature inverted text index,
  `LowCardinality`, and a decade-tuned C++ vectorized engine with lower fixed
  per-query overhead (Runs 1–2; `read-path-indexing-and-execution.md`,
  `clickhouse-internals.md`).
- GreptimeDB's metrics edge is **PromQL-native *capability* + native ingest, not
  query speed**: at 40k series / 8M rows ClickHouse's SQL aggregation is **~2–3× faster
  warm** (Run 37: CH 50 ms vs GT 107 ms = 2×; Run 67 re-verify: CH 32 ms vs GT 99 ms = 3×
  as CH's JIT warms, GT stable — **corrected down from the ~10× of Run 11,
  which was a cold/first-run GreptimeDB scan, not the warm gap**; cold-regime gap is
  larger). **Hardened further (Run 44): even GreptimeDB's *own* native PromQL path is
  ~5× slower than its own SQL** (`avg by(service)` ≈590 ms vs ≈120 ms vs CH SQL ≈65 ms) —
  the PromQL planner pays a near-fixed `SeriesDivide`/`SeriesNormalize` series-sort setup
  (a single-step instant eval costs as much as a 20-step range) that a streaming SQL
  hash-agg avoids. So the raw metric-agg ordering is **CH SQL > GT SQL > GT PromQL**.
  GreptimeDB ties only on **freshness** (both visible-on-write, Run 5). So even on
  metrics, "GreptimeDB fastest" is false for aggregation *latency at volume* — it
  wins on PromQL **maturity/ergonomics** (GA, default-on, the expressiveness of range
  vectors/`rate`/lookback) vs ClickHouse's **experimental** 26.x PromQL (`TimeSeries`
  engine, off by default) — a real lead, but narrower than the old "ClickHouse has no
  PromQL" framing (corrected pass 44, `promql-and-metrics-query.md`).

So the ordering "GreptimeDB fastest, then ClickHouse" does not hold for the
analytical query shapes; it inverts. The design decisions that cause it: ClickHouse
optimized the *read path for selective columnar scans* (sparse index + PREWHERE +
SIMD); GreptimeDB optimized for *metric-native ingest + time-series model + object
storage*, accepting a younger DataFusion scan engine.

## Decision question 1 — where is GreptimeDB genuinely better, and why?

| Area | Mechanism | Confidence |
| --- | --- | --- |
| **Metrics / PromQL** | Native PromQL planner (custom DataFusion nodes) + Prom remote-write + metric engine, **GA + default-on**. **Corrected pass 44:** ClickHouse 26.x *does* have PromQL (`prometheusQuery[Range]` over the experimental `TimeSeries` engine), but **experimental, off by default, setup-heavy** — so the win is now **maturity/ergonomics, not capability** (`promql-and-metrics-query.md`). | plan+live (Run 3, Run 23) |
| **Write ergonomics** | LSM memtable absorbs high-frequency small writes; no ClickHouse "too many parts". Native OTLP/Prom ingest, no collector. | arch+Run 5 |
| **Horizontal scaling** | Region model + Metasrv auto-rebalance + repartition + compute/storage separation (object store + remote WAL) → topology change, not rewrite. **Region migration confirmed in source (pass 34)** = flush→downgrade→open_candidate→upgrade→close, **no bulk-copy step** — ownership reassignment + reopen-from-object-storage, cheap precisely because SSTs already live in S3. | arch+source (multi-node run owed) |
| **Latest-state / upsert reads** | Dedup is **read-time** (`DedupReader` in the scan path): `last_row` / `last_non_null` (per-field partial-upsert merge) → "current issue status / deploy marker / metric last-value" is correct on a **plain query**, no keyword. ClickHouse `ReplacingMergeTree` dedups only at merge/`FINAL` (dupes visible until then). Concrete win on the Q2 issue-history sub-query. | source+measured (Run 19) |
| **Schema evolution (OTLP drift)** | Ingest **auto-adds typed columns** (`create_or_alter_tables_on_demand`) — a new attribute lands with zero migration; ClickHouse rejects unknown-column inserts (needs JSON or a managed ALTER). Both `ADD COLUMN` are metadata-only. | source+measured (Run 18) |
| **Retention cost** | TTL = **whole-SST drop** (TWCS time-windowing → no read/rewrite; `compactor.rs:581`); ClickHouse default `ttl_only_drop_parts=0` **rewrites survivors** (Run 17: read 1M / rewrote 500k) unless tuned (`PARTITION BY` time + `ttl_only_drop_parts=1`). Cheap-by-default vs cheap-if-configured. | source+measured (Run 17) |
| **Object-storage-native** | OpenDAL default + read cache; cheap re-readable retention first-class. Fewer *total* objects (4 vs 74, Runs 8–9) → wins full-scan cold reads. Cold GET cost is query-shape-dependent (measured both ways): full scan GreptimeDB fewer (26 vs 57, Run 15 — wins the JSONBench regime); **anchored lookup ClickHouse fewer (5 vs 22, Run 14)** — Parallax's pattern. Read cache → warm re-reads local on both. | measured (layout + cold GETs both shapes) |
| **Durability / crash safety** | Has a **replayable WAL** (raft-engine local, tunable `sync_write`; or **Kafka remote → durability decoupled from the datanode**, the same mechanism that makes migration cheap). ClickHouse MergeTree has **no WAL** (obsolete in 26.x) — durability = unsynced part-on-disk (`fsync_after_insert=0`) + replicas; a single-node crash loses unflushed parts. | source+live (Run 20) |
| **High-cardinality metric *ingest*** (rate + ergonomics) | Metric engine `__tsid` (label-set hash) over a shared physical wide table + PartitionTree memtable (dict-encoded label sets, **no per-series cap**) — cap-free ingest, many logical metrics → one physical table, no `ORDER BY` tuning. **Measured (Run 84): GreptimeDB ingest is cardinality-INSENSITIVE** — 1k→1M series ~flat (357→381 ms, ~1.07×) vs **ClickHouse ~2.6× slower** (`LowCardinality` overflow + more `ORDER BY` keys). The clearest GreptimeDB high-card win is the ingest axis. ClickHouse `LowCardinality` caps at 8,192 then degrades **gracefully** (still < plain `String` at 200k — Run 76, not the "cliff explosion" first framed). **⚠ Corrected (Runs 76–79): high-card *storage* is CARDINALITY-DEPENDENT (a crossover)** — ClickHouse `LowCardinality` wins low–mid (1k ~1.12×, 200k ~1.24×) but **GreptimeDB wins at ~1M unique series ~1.34×** (CH `LowCardinality` blows up to 16.51 MiB all-unique vs GT 12.36; the metric engine's `__tsid` is overhead not a saving). And **aggregation latency → ClickHouse ~2–3× warm** (Run 37/67). So GreptimeDB's high-card win is **operability/no-cap + extreme-cardinality storage**, ClickHouse's is **moderate-cardinality storage + agg speed.** | source+live (Runs 26, 76–79) |
| **Corrections (UPDATE) / upsert** | UPDATE = re-insert `(PK,ts)` → dedup last-wins = a **cheap GA upsert**, no setup; ClickHouse UPDATE = heavy `ALTER UPDATE` part rewrite (lightweight update is experimental + needs a per-table block-number column). DELETE is ~parity (both read-filtered). | source+live (Run 29) |
| Freshness | Visible-on-write (tie with ClickHouse, not a win). | smoke |

## Decision question 2 — where is ClickHouse genuinely better, and why?

| Area | Mechanism | Confidence |
| --- | --- | --- |
| **Log/trace selective scan + full-text search** | 8,192 granule + PREWHERE + inverted text index + LowCardinality + C++ SIMD vectorized pipeline. | arch+smoke (Runs 1–2) |
| **Generic wide scan / aggregate throughput** | Decade-tuned vectorized engine — the OLAP-scan bar. Mechanism (pass 42): 65k-row blocks (8× DataFusion's batch) + LLVM-JIT expressions/aggregation + bespoke SIMD kernels + specialized hash aggregation vs GreptimeDB's DataFusion-over-Arrow; explains Runs 11–12. | arch+live (`query-execution-engine.md`) |
| **Vertical single-node ceiling** | Saturates many cores + NVMe on one big box. | arch |
| **Per-column codec tuning** | Hand-picked `DoubleDelta`/`Gorilla`/etc. (counter 7.3×, gauge 78×, Run 4). | smoke (Run 4) |
| **Dynamic-attribute path queries** | `JSON` type stores each path as a **typed columnar subcolumn** (`attributes.k` reads only that subcolumn); GreptimeDB `Json` is a binary blob + `json_get_*` per-row parse. **Measured ~13× (6 ms vs 78 ms, 100k, Run 61)** for an *unpredictable* attribute path. Two caveats: CH subcolumns are `Dynamic`-typed (GROUP BY needs a `.:Type` cast), and GreptimeDB closes it for *known* hot attrs by promoting them to typed columns (automatic-CH vs manual-GreptimeDB). Bites only on ad-hoc paths at volume. | source+measured (Runs 18, 61) |
| **Multi-ordering scans (projections)** | A **projection** stores a 2nd physical `ORDER BY` inside each part, optimizer-picked transparently → fast sequential scans on an alternate key (e.g. `service`-time *and* `trace_id`) from one table. GreptimeDB has no equivalent (indexes give positions, not a 2nd physical order). Cost: ~2× storage per normal projection (Run 28). | source+live (Run 28) |
| **Cross-tier anchored *in-DB* join** | ClickHouse pushes the anchor (`trace_id='X'`) through a `LEFT JOIN` into the scan and prunes (`Granules 1` + PREWHERE) → ~4 ms. **GreptimeDB does NOT push a left-side filter through the join** (Run 81) — `EXPLAIN ANALYZE` shows a full 1M-row `spans_idx` scan → ~54 ms (~13×); a predicate-pushdown-into-join optimizer gap. Fixable: subquery pre-filter (~21 ms) or app-side correlation (Parallax's pattern — anchored fetch each signal + join in app, avoids the in-DB join). So a *direct* in-DB cross-tier join favours ClickHouse; rewrite/app-side neutralises it (all < 300 ms gate). **Re-verified Run 103, no drift:** CH ~3 ms / GT direct ~53 ms (~17×, full-scan) / GT subquery-prefilter ~19 ms. | source+live (Runs 30, 81, 103) |
| **Cold *selective* object-store reads** (~10× with partitioning, ~80× without) | ClickHouse `ORDER BY (trace_id, ts)` clusters the anchor at **zero cardinality cost** → cold anchored read ~1 granule (**294 KiB**). GreptimeDB non-partitioned scatters `trace_id` → ~whole SST (**~23 MiB**, ~80×, Runs 55/63). **But `PARTITION ON COLUMNS(trace_id)` cuts it to ~2.8 MiB (16-way, Run 88) → ~10× gap** — a cardinality-free anchor-locality lever the native `opentelemetry_traces` ships by default; finer partitioning narrows more. GreptimeDB's persistent read cache also keeps the common warm path at ~0 S3. So the cold-selective-egress gap is real but **~10× (partitioned), not ~80×**, and only on genuinely cold/evicted reads. | measured (Runs 55, 63, 87, 88) |
| Query latency at smoke scale | Won every non-metric query (2–4 ms vs 9–54 ms) — but cache-resident, fixed-overhead-dominated. | smoke |

## Decision question 3 — can ClickHouse replace GreptimeDB for Parallax?

**Yes, technically** — it stored every Parallax signal and returned identical
evidence bundles (Runs 1–4 parity PASS), and the **full ClickHouse schema is now
verified buildable on 26.5.1** (Run 46 — JSON/codecs/`text` index/AggregatingMergeTree
MV all build; one `text`-tokenizer fix). But three design decisions impose real cost:

1. **Observability protocols are experimental or external, not GA-native.** All three
   are **GA-native + default-on in GreptimeDB** (OTLP metrics/logs/traces Run 25; PromQL
   Runs 23–24; **Jaeger query API Run 32** — `/v1/jaeger/api/services` live). On
   ClickHouse 26.x each is *assembled*: OTLP via a collector (no native receiver, pass
   46); PromQL via the **experimental, off-by-default `TimeSeries` engine**
   (`prometheusQuery[Range]` table functions + the 12-function `timeSeries*ToGrid`
   family — broad PromQL coverage, *not* "limited to rate/delta/increase"; pass 44 /
   Run 50 — *not* "absent" anymore, but off by default); Jaeger via the **external
   `jaeger-clickhouse` storage
   plugin** (pass 55). So Parallax would depend on experimental/external observability
   paths, vs GreptimeDB's GA-native trio. A maturity/ergonomics cost now, not a hard
   capability blocker.
2. **Horizontal scale-out is manual** (shard count + sharding key up front; no OSS
   auto-resharding; `SharedMergeTree` is Cloud-only). Outgrowing the initial layout
   is a disruptive data-move — friction against the startups→big-companies path.
3. **Part-explosion** on streaming small writes → a batching/async-insert layer is
   required to stay healthy.

→ ClickHouse can replace GreptimeDB **at the cost of** a PromQL+OTLP compatibility
layer, a sharding/ops burden, and an ingest-batching layer.

**Trajectory (passes 44–51) — the gaps are narrowing, mostly experimentally.**
ClickHouse 26.x is *actively* closing the observability gaps: it added PromQL
(`prometheusQuery[Range]`, pass 44), Prometheus remote-write (TimeSeries engine),
lightweight `DELETE` (GA-default mask, pass 51), and an experimental lightweight
`UPDATE` (pass 51) — all things earlier framed as "absent." But the pattern is
consistent: each lands **experimental and/or setup-gated** (TimeSeries off by default,
lightweight update needs a per-table block-number column), while OTLP ingest is
**still collector-only** (pass 46). So the
replaceability *cost is trending down* — but today it is "depend on experimental
metrics/correction paths" rather than "GA-native," and **GreptimeDB's are GA now**.
This is a live trend to re-evaluate on every ClickHouse version bump (the method's
per-pass re-check exists for exactly this); the *direction* favors ClickHouse closing
the gap over time, the *present state* still favors GreptimeDB for shipping today.

## Decision question 4 — can GreptimeDB replace ClickHouse for Parallax?

**Yes** — it stored every signal and ran Q1–Q6 with identical results, and the **full
GreptimeDB schema is verified buildable on v1.0.2** (Run 45 — `INVERTED`/`FULLTEXT`/
`SKIPPING` indexes + metric engine all build, after quoting 7 reserved-keyword columns
and dropping the metric-table's empty `PRIMARY KEY ()`). The cost:

1. **Slower heavy log/trace analytics** (younger DataFusion engine, coarser
   granule, no PREWHERE-equivalent late materialization yet). **But Parallax's
   dominant retrieval is *anchored* evidence-bundle assembly** (always filtered by
   `trace_id`/`fingerprint`), where both engines prune the anchor first and the
   gap shrinks (Run 2). **Now measured end-to-end (Run 16): the full anchored
   composite Q6 is ~33 ms on GreptimeDB vs ~10 ms on ClickHouse — both ≪ the 300 ms
   gate, GreptimeDB's gap being 3-way-UNION fixed overhead, not algorithmic.** So on
   the query that matters most this blocker is **not latency-bound** — it is far less
   central than the raw log-scan numbers suggest.
2. **Younger analytical/distributed maturity** — region migration/repartition are
   2025-era; less battle-tested than ClickHouse's shard model.
3. **Schema discipline required**: `trace_id` must be in the primary key / indexed
   or point lookups scan (Run 1: 16 ms vs 2 ms) — fixable in the schema design.

→ GreptimeDB can replace ClickHouse for Parallax's workload, accepting slower
ad-hoc large-scale log/trace search.

**The full Q1–Q6 evidence-bundle set is now measured at smoke** (Q1/Q2/Q3 + composite
Run 16; Q4 cross-tier join Run 30; Q5 high-card filter Run 31). Pattern: the
**anchored** bundle queries (Q1–Q4, Q6) are *not latency-bound* on either engine
(both ≪ the 300 ms gate); the only place ClickHouse pulls clearly ahead is the
**unindexed scan** shape (Q5 — **re-verified warm Run 102: ~2–5× shape-dependent**, not the
~10× Run 31 reported, which was cold/HTTP-wall inflated; ~5× pure point-filter scan compressing
to ~2× once aggregation work is added; all ≪ 300 ms at 1M; plus ad-hoc log search Run 12) — which
Parallax avoids by anchoring and indexing. So "GreptimeDB slower" is real **only** for
scan-shaped queries Parallax does not run on the hot path.

## Decision question 5 — which to choose

**GreptimeDB**, for Parallax specifically.

- **Language filter**: both pass (Rust / C++ allowed; no JVM/interpreted).
  **Rust breaks ties → GreptimeDB** (per `AGENTS.md` + storage evaluation).
- **Workload fit** is the deciding factor. Parallax = metrics + logs + traces +
  *anchored* evidence-bundle correlation, streaming OTLP/Prom ingest, cheap
  re-readable object-store retention, tiny-single-node → horizontal growth, Rust-
  first, self-hosted/open. GreptimeDB wins or ties the axes that dominate this
  profile (metrics-native, freshness/ingest ergonomics, horizontal scaling,
  object-store economics); ClickHouse's wins (log/trace scan latency, vertical
  ceiling) are real but less central to *anchored* retrieval and come with the
  PromQL/OTLP/sharding cost above.
- **No third system** is warranted: neither a mechanism gap nor the language filter
  opens room for one; both candidates cover the workload.
- **No hybrid by default.** A GreptimeDB+ClickHouse split would put logs/search on
  ClickHouse and the rest on GreptimeDB — but that splits Parallax's cross-signal
  evidence-bundle correlation (the hot path) across two engines and doubles ops. Only
  justified if a benchmark shows ad-hoc log search is both heavy *and* standalone. The
  better route to "clear winner for all cases" is closing GreptimeDB's few gaps — see
  **`greptimedb-parity-roadmap.md`** (the gaps are execution-integration, mostly on the
  DataFusion roadmap or contributable in Rust, not architectural; Tier-A schema/Flow work
  already wins Parallax's anchored workload today).

### Recommendation, tradeoffs, and the rejected alternative

- **Choose GreptimeDB.** Tradeoffs accepted: slower heavy ad-hoc log/trace search;
  younger analytical/distributed maturity; must design `trace_id`/`fingerprint`
  into keys/indexes.
- **Rejected: ClickHouse.** It is the faster analytical engine and more mature, with
  a higher vertical ceiling — but for Parallax it requires building a PromQL+OTLP
  compatibility layer, manual sharding with a resharding wall, and an ingest-
  batching layer, and it is C++ not Rust. Chosen-against on *fit*, not on merit.

### The trigger that would flip this

If a benchmark shows that (a) Parallax's real query mix is dominated by
**large-scale, cold-cache, ad-hoc log/trace search** (not anchored bundle
assembly), **and** (b) GreptimeDB's cold-scan latency at GB–TB is materially worse
(not just the smoke-scale fixed-overhead gap), then ClickHouse's read-path
advantage becomes central and the choice flips — accepting the PromQL/OTLP layer
as the cost of doing business. This is the single most important thing the larger
benchmark must settle.

**Historical update (Run 12, measured at 5M logs, both indexed; warm-re-verified Run 38;
superseded by Runs 48-49):** condition (b) once looked **partly confirmed** — ClickHouse
full-text log search appeared **~18×** faster (7 ms vs 129 ms) and full count-by-`level`
scans ~4× (Run 39, warm-verified). That was useful because it proved the difference was
not a cold-cache artifact, but Runs 48-49 later showed the full-text number was a
backend/function artifact, not a real index-maturity gap. Keep only the surviving lesson:
if Parallax's mix is **broad log/trace scan-dominated**, the flip can still be real. But
Parallax's designed pattern is *anchored* bundle assembly (keyed lookups), and selective
full-text is now competitive with the right backend/function pairing. Validate the
assumption (what fraction of real Parallax queries are broad ad-hoc search vs anchored
retrieval) — it is the load-bearing question, not the old 18× number.

**Major correction (Run 48): the ~18× was largely a query-form artifact.** `logs_b1`'s
fulltext index is `backend='bloom'`, and Run 12 queried it with **`matches()`** (the
tantivy *query-syntax* function) — which does **not** push to a bloom index, so it
**full-scanned 5M rows** (EXPLAIN ANALYZE `output_rows: 5000000`), fixed regardless of
selectivity (even a 1-row-match term took ~150 ms). With the **correct pairing** —
**`matches_term()`** (exact term) on the bloom index — GreptimeDB **prunes** (scan
`output_rows: 1`) and selective exact-term search is **~8 ms warm, ~2–3× ClickHouse's
~3 ms, not 18×.** So for Parallax's *actual* incident-search pattern — an SRE grepping a
specific request-id (an exact term) — **GreptimeDB is competitive (~8 ms), not 18× slower.**
After Run 48, the large gap only applied to (a) the `matches()`/bloom mismatch (use the
tantivy backend for query-syntax), or (b) broad-term scans matching many rows (~12×,
scan-engine territory = Improvement #2). This **substantially narrowed the flip trigger**:
the verdict's one big ClickHouse win shrank to "wrong backend/function pairing or broad-term
analytics," not the everyday exact-term incident grep. Detail in
`local-benchmark-results.md` Run 48 + `greptimedb-parity-roadmap.md` #1.

**Closed (Run 49): the query-syntax path is also fast.** A tantivy-backed index makes
`matches()` (query syntax) **prune** — selective ~6 ms warm (EXPLAIN `output_rows: 1`), vs
the ~150 ms full-scan on a bloom index. So **both** selective full-text paths are
sub-perceptible with the correct backend: **tantivy + `matches()` ~6 ms**, **bloom +
`matches_term()` ~8 ms**, vs ClickHouse ~3 ms (~2×). The ~18× was **100 % a backend/function
misconfiguration**, not a full-text-maturity gap. **Net: ClickHouse's log-search advantage
dissolves for interactive/selective search on both query types; the only residual is
broad-term analytics (scan engine).** Parallax guidance: tantivy backend for query-syntax,
bloom for exact-term grep — both fast. This is the strongest narrowing yet of the verdict's
one large ClickHouse win.

**Re-verified (Run 98, no drift) — all three legs reproduce on the current containers:** selective
exact-term (1 match) bloom + `matches_term` = CH ~3 ms / GT ~10 ms (~3×, both sub-perceptible); the
`matches()`-on-bloom artifact still full-scans ~155 ms (proving the ~18× was the pairing, not the
engine); broad-term (699k matches) CH ~7 ms / GT ~88 ms (~12×, scan-bound = parity-roadmap #2). The
finding is stable: selective grep competitive, broad-term scan the only real residual.

## Decision question 6 — which is the better long-term *investment*?

DQ1–5 answer "which fits Parallax's workload today." This answers a different, sharper
question the operator raised: **over the next several years, which engine is the better
thing to invest in** — given that ClickHouse is faster *now* and more mature, but the
operator will invest engineering in **Rust and not C++**? Two sub-questions decide it:
*(A) is the speed gap closable or a permanent wall?* and *(B) who can actually close it?*

**(A) Is the gap closable, or fundamental like physics?** — *Closable.* The operator's own
analogy: some gaps are like Singapore↔US latency vs Singapore↔China — pure geography, no
engineering crosses them. The parity-roadmap's per-gap **physics-wall test** finds **none of
ClickHouse's advantages is that kind of wall**: seven of eight are pure engineering (same
vectorized-columnar-over-Arrow model, ClickHouse merely a decade further along the *same*
curve), #6 is *time-only* (maturity, closes on a calendar), and #5 (the PK=sort=series
conflation behind cold selective egress) is the lone design-*flavoured* one — already
**defused to an engineering choice** by `trace_id` partitioning + a re-sorted copy (Runs
87/88 cut it from ~80× to ~10×). Decisively, the two heaviest gaps ride **shared industry
roadmaps**: scan/agg throughput (#2) is on the **DataFusion** codegen/SIMD/batch roadmap,
and dynamic-attr JSON (#4) is the **Parquet Variant/shredding** direction — so much of the
closing work is *already in flight by others*, and GreptimeDB inherits it on a dependency
bump. **ClickHouse's raw-speed lead is therefore a depreciating asset, not a moat.**

**(B) Who can move the engine?** — This is the operator's decisive lever, and it is
*asymmetric*. GreptimeDB and DataFusion are **open-source Rust**; a gap there is one the
operator (and AI-assisted contribution, which is markedly stronger at Rust than C++ — cf.
Bun's Zig→Rust move for performance + maintainability) can **actually land a PR against**.
ClickHouse's C++ engine is contributable in principle but **not by this operator in
practice** ("I will not invest in C++; I will invest in Rust"). A gap you can close is
categorically different from one you can only wait on — so the *same* benchmark gap has
opposite strategic meaning depending on which engine carries it. Contributions land in a
**shared Arrow/DataFusion ecosystem**, not a private fork, so the effort compounds and is
partly shared with the upstream community.

**Design trajectory / growth potential** (judge the *direction*, not the snapshot): the
**Postgres-overtook-MySQL** precedent — the better-architected-for-the-domain system passes
a more-mature incumbent once effort compounds. For an AI-native observability/debugging
context engine, GreptimeDB *is* the domain-native design: metrics+logs+traces in **one**
engine, **object-store-native** economics, **horizontal-scale-designed-in**, **cardinality-
insensitive ingest** (Run 84: ~flat 1k→1M series vs CH ~2.6×), and **Arrow/DataFusion
extensibility** as a contribution surface. ClickHouse is a superb *general* OLAP engine that
*added* observability (experimental PromQL, collector-only OTLP, plugin Jaeger — DQ3). The
direction favours GreptimeDB for *this* domain.

**Cost + scalability, now and projected:** object-store tiering vs local-NVMe replicas
(GreptimeDB 1× shared S3 copy vs OSS-ClickHouse N× replica copies — open Q#6); cardinality-
free ingest; **small→large is a topology change, not a rewrite** ([[scaling-trajectory]]) —
the startups-first/tiny-single-node path grows to horizontal without re-platforming, which
ClickHouse's manual sharding + Cloud-only `SharedMergeTree` does not match in OSS.

**The honest risk of the bet (stated plainly, no cheerleading):** betting on GreptimeDB
assumes the engineering gaps *actually get closed* — by the operator, by the GreptimeDB
team, and by the DataFusion community. ClickHouse is faster today, more mature, and has
momentum. **If that investment does not materialize**, the raw-speed gap persists
indefinitely on ad-hoc analytics (though *never* the observability-native **fit** gap, which
is structural in ClickHouse's favour-of-GreptimeDB direction). So:

- **Bet GreptimeDB** if you (a) value the Rust-contributable substrate, (b) believe the
  domain-native design compounds, and (c) will actually invest — which the operator states
  he will. The gaps are closable, much of the work is shared/in-flight, and Parallax's
  *anchored* hot path is already a sub-300 ms tie (Tier-A wins it today, DQ4/roadmap).
- **Bet ClickHouse** only if the real workload turns out **analytics-/ad-hoc-scan-dominated**
  (the DQ5 flip trigger) *and* you want maximum speed today with **zero** engine investment.

**Investment verdict: GreptimeDB is the stronger long-term bet for Parallax** — its deficits
are engineering/time not physics, its design is the better domain trajectory, and uniquely
it is the substrate this Rust-investing operator can *improve* rather than merely consume.
This **reinforces** the DQ5 fit recommendation from the investment angle: same answer, now
also defensible as "the gap is closable, by us, in the language we'll invest in." It is not
unconditional — the DQ5 flip trigger (analytics-dominated mix) still governs, and the bet's
honest precondition is sustained contribution.

## Open questions handed to the benchmark (veto power)

`storage-benchmark-prototype.md` must settle, at `small`+ tier, cold cache,
concurrent ingest+query:

0. **JSONBench cold-run at 1B docs** (public claim #6). **Mechanism locally
   confirmed (Run 15):** on a cold *full scan* GreptimeDB issued fewer S3 GETs (26
   vs 57) — its few-large-objects layout wins cold scan/wide reads, exactly the
   JSONBench mechanism. **Still owed:** the 1B-doc *cold latency* at scale (only the
   GET-count mechanism is verified locally). This cold/object-store regime is one
   Parallax touches for retention re-reads.
1. **Cold-cache GB–TB log/trace scan latency** — how much slower is GreptimeDB
   beyond the cache-resident smoke floor? (Could flip Q5.) Runs 48-49 dissolved the old
   selective full-text ~18×; the remaining owed number is broad-term and unanchored
   scan latency at GB-TB scale.
2. **Object-store cost on equal footing** (MinIO) — **largely answered, now with a
   measured two-sided cold result (Run 55/B10):** retained bytes ~tie (compression
   wash); object count GreptimeDB 3 vs CH 74 (Run 54, re-verified). **Cold-read is
   regime-split:** for a cold *anchored* lookup, **egress strongly favours ClickHouse**
   (~294 KiB granule reads vs GreptimeDB ~23 MiB whole-SST, ~80× — small-SST-inflated,
   at-scale owed), while **request count favours GreptimeDB** (9 vs 18 GETs) and
   **warm/repeat re-reads favour GreptimeDB** (write-through persistent local cache →
   ~0 S3 after first touch). ⚠ **Reproduction conflict flagged:** the anchored cold
   GET-count *direction* did **not** reproduce — Run 14 had CH 5 < GT 22, Run 55 has
   GT 9 < CH 18; GET count is **SST/part-state-dependent and unstable**, so the robust
   differentiator is **egress bytes** (mechanism: CH sparse-granule reads vs GreptimeDB
   Parquet whole-SST/row-group), not GET count. Wide cold *scan* still favours
   GreptimeDB (GT 26 < CH 57, Run 15 — the JSONBench regime). Remaining: at-scale
   selective-cold egress + a realistic mixed bundle workload.
3. **Concurrent ingest+query freshness p95** — **penalty answered (Run 13):** both
   pass the ≤2× gate (CH 1.55×, GT 1.38×). The precise mixed-load *freshness p95*
   (stamp-emit→poll) still owed.
4. **Multi-node scale-out hold** — does p95 hold as nodes are added; GreptimeDB
   region rebalance vs ClickHouse resharding effort. **Untested (needs multi-node
   harness).**
5. **Realistic-cardinality compression** — **answered (Run 10):** realistic
   99%-unique log text → tie at matched codecs (GreptimeDB 25 vs CH 24.24 MiB),
   GreptimeDB-favored out-of-the-box.
6. **Multi-replica object-store cost** (B-new, pass 57) — does OSS ClickHouse HA on S3
   really pay N× storage (zero-copy not-production-ready) vs GreptimeDB's 1× shared
   copy? Mechanism source-confirmed (Run 34); the $ delta at N replicas is owed.
7. **Strict-durability throughput cost** (pass 41) — `sync_write=true` (GreptimeDB) vs
   `fsync_after_insert=1` (ClickHouse): the ingest-rate hit when forcing per-write
   durability. **Directionally measured (Run 75/B15):** GreptimeDB **~+1.7 ms/write (~3%)**
   — one sequential WAL-append fsync — vs ClickHouse **~+18 ms/part (~20%)** — whole-part
   fsync (column files + dir). **Strict-durable ingest is ~10× cheaper on GreptimeDB**
   (architectural: append-log fsync ≪ part fsync). The *sustained* strict-durable throughput
   ceiling at scale is still a sized-harness number; the per-write cost ratio is settled.
8. **High-cardinality metric storage at volume** (B13) — **answered, curve complete (Runs
   76–79):** a **crossover** — ClickHouse `LowCardinality` wins low–mid (1k 8.18/9.18 ~1.12×,
   200k 9.64/11.99 ~1.24×) but **GreptimeDB wins at 1M unique series (12.36 vs CH 16.51,
   ~1.34×)** — CH `LowCardinality` blows up all-unique, GreptimeDB scales gently. The cliff
   is graceful; the metric-engine `__tsid` is overhead not a saving. So storage winner is
   cardinality-dependent; GreptimeDB's edge = ingest ergonomics + extreme cardinality, CH's
   = moderate-cardinality bytes + agg speed. *Remaining:* the curve at larger row volume and
   ingest-rate under cap-free vs cap-managed are sized-harness extensions.

**These are the complete remaining gaps** — every smoke/source-answerable question is
closed; #0–#8 all require the larger-tier / cold-cache / multi-node / sized harness, and
are the prototype's domain (it holds veto). The white-box loop has done its job for the
mechanism layer; further sharpening waits on harness numbers (or a version bump that
ships a new mechanism — re-checked each pass).

## Supporting notes

- Mechanisms: `greptimedb-internals.md`, `clickhouse-internals.md`,
  `read-path-indexing-and-execution.md`, `write-path-and-ingestion.md`,
  `compression-and-cost.md`, `distributed-and-scaling.md`,
  `compaction-and-merge.md`, `caching-and-cold-warm.md`,
  `rollup-and-continuous-aggregation.md`, `retention-and-ttl.md`,
  `schema-evolution-and-dynamic-columns.md`, `dedup-and-update-semantics.md`,
  `wal-and-durability.md`, `query-execution-engine.md`, `indexing-internals.md`,
  `promql-and-metrics-query.md`, `metric-cardinality.md`, `trace-span-tree.md`,
  `projections-and-access-paths.md`, `deletes-and-mutations.md`.
- Matrix: `per-signal-verdict.md`. Empirical: `local-benchmark-results.md`
  (Runs 1–46; recent: 37 metric-agg ~2× warm, 43 rollup live, 44 native-PromQL ~5×
  slower than GT SQL, 45–46 both impl schemas built live). Public claims:
  `public-performance-claims.md`. Targeted cases:
  `benchmarking-the-differences.md` (B1–B15; B14 multi-replica S3 cost, B15
  strict-durability throughput added pass 59 as harness-handoff specs for open Q#6/#7).
- Build designs: `greptimedb-implementation.md`, `clickhouse-implementation.md`.
- Reproducible object-store stack: `bench/s3/`.
