# GreptimeDB vs ClickHouse — Run Log and Detailed History

<!-- markdownlint-disable MD013 -->

This is the **working history and detailed evidence** for the engine sub-study. The current
one-page conclusion lives in [`verdict-which-to-choose.md`](verdict-which-to-choose.md); the
product-level decision is in [`../../decisions/storage-engine.md`](../../decisions/storage-engine.md).
Nothing was deleted in the 2026-05-29 restructure — the run-by-run status timeline, the per-note
status, and the pre-restructure verdict synthesis are preserved verbatim below.

---

## Part A — sub-study status timeline and per-note status (former README body)

# GreptimeDB vs ClickHouse — Deep Internals Comparison

<!-- markdownlint-disable MD013 -->

Status: produced by an indefinite research loop; **white-box analysis comprehensive
(through pass ~75)**. All 10 checklist subsystems + every named ClickHouse/GreptimeDB
lead are torn down against source; the Q1–Q6 evidence-bundle set is measured; the 9
public claims are triangulated (the "ClickHouse has no PromQL" one was caught drifting —
26.x added experimental PromQL); and the load-bearing latency numbers were re-verified
warm + HTTP-fair (one correction: the metric-agg gap is **~2× warm**, not the ~10× a
cold/first-run measurement showed). 32 mechanism + synthesis notes + 171 local runs + B1–B15 cases. Recent: **Run 171 — projection re-verify (no drift) + verdict consistency fix + GAP LEDGER** — CH projection `p_svc` serves a non-primary `WHERE service` query (`ReadFromMergeTree (p_svc)` Granules 2/24, Run 28/71 holds); fixed the verdict's stale dynamic-attr framing to match Run 168 (cast enforced on 26.5 too, not 26.6-only); created `open-questions-and-gaps.md` consolidating what's NOT addressed (top: Parallax's un-characterized **workload mix** + **managed-cloud-vs-self-host** — the two inputs that most move the decision). **Run 170 — REPO-CONSISTENCY SWEEP (clean)** — no dead cross-references (the Run-169 dedup left no broken links; remaining `continuous-aggregation-and-rollups.md` mentions are intentional history-prose); corrected the note count (was "27", actual = 31 topical notes excluding the index/run-log/four-way-matrix). Repo-health verified. **Run 169 — REPO CLEANUP + count-distinct re-verify** — merged a duplicate note (`continuous-aggregation-and-rollups.md`, which I created in Run 149 unaware the canonical `rollup-and-continuous-aggregation.md` had existed since pass 27) into the canonical + redirected all refs + deleted the dup (28→27 notes). Count-distinct re-verified, no drift: both engines exact `count(distinct trace_id)`=71,429 (match) + HLL approx (CH `uniq`=71,563 / GT `hll_count`=71,319, <0.2% error) = capability parity. **Run 168 — LIVE (exec, 50k JSON): dynamic-attr DRIFT CORRECTION** — on 26.5.1.882 `GROUP BY attrs.region` → `Code 44` (cast enforced), `::String` cast works; `WHERE attrs.user_id` filter is cast-free. So the prior "26.5 lax-no-cast GROUP BY, 26.6 removes it" is WRONG — the cast is a 26.5+ GROUP-BY requirement, and the cast-free fast path was the FILTER (not a lax GROUP BY). Gap ~8–12× with cast (`schema-evolution-and-dynamic-columns.md`). **Run 167 — LIVE re-verify (exec): delete/redaction (Run 29/66), no drift** — GT `DELETE FROM`, CH `ALTER DELETE`/lightweight `DELETE FROM` all remove the row. GDPR nuance: both delete LOGICALLY first (tombstone/mask → read-filter), physical removal deferred to compaction/merge → for a right-to-be-forgotten deadline, **force compaction** (GT `compact_table` / CH `OPTIMIZE FINAL`), don't rely on lazy purge (`deletes-and-mutations.md`). **Run 166 — LIVE re-verify (exec): freshness (visible-on-write), no drift** — CH direct insert immediately visible (1→2), GT memtable visible-on-write (no flush). Resolves Run 160: the grouped-error MV incremental non-reflection was MV-target timing, NOT base freshness (which is immediate); freshness tie holds (`write-path-and-ingestion.md`). **Run 165 — LIVE re-verify (exec): span-tree recursive-CTE (Run 68/97), no drift** — GT table-self-join recursive CTE still errors (`project index out of bounds`); CH `WITH RECURSIVE` runs. Proxy-lens: Parallax IS the app layer → builds span trees app-side from the flat keyed fetch (prunes on both, Run 158) → the GT recursive-CTE gap is **fully irrelevant to Parallax** (`trace-span-tree.md`). **Run 164 — LIVE re-verify (exec): PromQL nativeness (no drift) + proxy-lens nuance** — GT `/v1/prometheus/api/v1/query` GA-native (envelope returns); CH `TimeSeries` engine still experimental-gated (`allow_experimental_time_series_table=1`), `prometheusQuery` needs `(ts_table,promql)` args (my bare call's UNKNOWN_FUNCTION was an arg mismatch, not drift). New framing: **native PromQL is LESS proxy-neutralized than native OTLP** (translating SQL←PromQL is expensive) → GT's GA-native PromQL is a *more durable* surviving edge (`promql-and-metrics-query.md`). **Run 163 — LIVE re-verify (exec): dedup/upsert semantics (Run 19), no drift** — CH `ReplacingMergeTree` plain SELECT shows dups until `FINAL`; GT dedups exact `(PK,ts)` dups at READ + latest-state via `last_value`. Clarified the dedup-MODEL distinction: CH = upsert-table (one row/key, eventual); GT = time-series ((PK,ts) points, read-time). Latest-state correct-by-default on GT, needs FINAL/argMax on CH → reinforces mutable→relational-store split (`dedup-and-update-semantics.md`). **Run 162 — LIVE (exec, 200k×2): cardinality-cliff STORAGE re-verify — no cliff, per-series cost ~parity** — isolated 1k→50k distinct: CH `LowCardinality` grew only ~4× for 50× cardinality (graceful, re-confirms Run 76); per-added-series ~parity (CH 14.0 / GT 15.6 bytes). Reconciles the narrative: GT "cardinality-insensitive" = ingest throughput + no-hard-cap, NOT storage density (storage is cardinality-dependent on both; Run 79 crossover) (`metric-cardinality.md`). **Run 161 — STRATEGY (operator redirect): storage-architecture thesis verified + cost made co-equal + hybrid evaluated** — CH=performance/local-first (real S3 cold penalty ~2000×, S3=cold tier, N× copies; Cloud distributed-cache closes it at $ premium); GT=S3-native (1× shared copy, elastic). S3 ~3.5× cheaper than EBS gp3 × N×-vs-1× × density = GT's bulk cost edge. **ClickHouse optimizes time-to-answer on hot data; GreptimeDB optimizes $/GB on deep data.** Hybrid (CH live + GT historical) = Phase-2 cost optimization, not Day-1 (doubles ops). New note `storage-cost-and-tiering.md`; benchmark prompt reframed (cost co-equal). **Run 160 — LIVE build (exec): concrete Sentry-style grouped-error rollup on ClickHouse** (AggregatingMergeTree + MV: `countState/minState/maxState/argMaxState` → `-Merge`) — computes correctly (fp-135=21, matches Run 156); GreptimeDB does it via Flow (Run 149) or query-time `last_value`. Both columnar engines build the rollup *aggregate*; only mutable workflow state needs the relational store. Caveat: incremental-on-insert not freshly re-confirmed (async_insert buffering + early cleanup) (`rollup-and-continuous-aggregation.md`). **Run 159 — LIVE storage re-verify (exec, larger tier): GreptimeDB denser on metrics+logs, ClickHouse on traces** — metrics_hc 8M GT 38.6 vs CH 57.42 MiB (~1.49×); logs_b1 5M GT 258 (incl 18 idx) vs CH 399 MiB (~1.55×); spans 1M CH 28.9 vs GT 37.4 (~1.3×, no drift). Metric winner is **shape-dependent** (counter/gauge→CH, labeled-series+plain-value→GT). GT denser on the high-volume bulk → concrete basis of its cost edge (stacked on 1×-vs-N× replication) (`compression-and-cost.md`). **Run 158 — LIVE re-verify (exec): the dominant-query pillar (anchored evidence-bundle)** — anchored `trace_id` fetch prunes IFF `trace_id` keyed on that signal, on **both** engines (keyed: GT `spans_idx` 14 rows / CH `spans` `Granules 1/123`; un-keyed `logs_b1`: CH `611/611`, GT `scan_cost 429ms`/49 ranges = full scan). Schema blueprint: **key `trace_id`/`fingerprint` on every signal**. Methodological fix: GT scan `output_rows` = post-filter emission, NOT rows-read — gauge by `scan_cost`/`file_ranges` (`per-signal-verdict.md`). **Run 157 — LIVE re-verify (exec, real 5M `logs_b1`): full-text index-pruning mechanism holds, no drift** — selective term (UUID frag, 1 match) prunes on both (GT bloom 5M→~51,200 rows; CH text `Granules 1/611`); broad term (`timeout` 699k) prunes on neither (every granule has matches → GT scans 5M, CH `611/611`) → scan-bound, CH vectorized wins. Flip-trigger (broad log search→CH) holds; selective is a tie (CH finer granule 8k vs 51k) (`per-signal-verdict.md`). **Run 156 — LIVE capability check (exec): build-on-top is an ECOSYSTEM gap, not a SQL-capability gap** — both engines compute the Sentry-style grouped-error rollup (GT `last_value(... ORDER BY ts)` / CH `argMax`, identical results) AND evidence-bundle window ranking (`row_number() OVER …`, identical). So GreptimeDB is **not capability-blocked** for the grouped-error requirement; CH's "build on top" edge is ecosystem/maturity, not query expressibility (`platform-fit-and-alternatives.md`). **Run 155 — object-storage economics under the proxy lens** (+ ENV: localhost benchmarking is structurally blocked — agent capsule is network-isolated from Docker ports/bridge; only `docker exec` reaches engines; restart didn't fix). CH *can* tier to S3 (storage policies + TTL MOVE, the ClickStack cost path) → closes most of the raw cold-storage gap; GT's edge **narrows but persists** on 1× vs N× replication + elastic near-stateless compute (`SharedMergeTree` Cloud-only) = an **OSS-vs-Cloud, self-hosted-HA** distinction (`distributed-and-scaling.md`). **Run 154 — LIVE re-verify (exec): join-pushdown gap reproduces, isolated sharper** — GT `trace_id` index prunes a *plain* filter (not 1M), but the `LEFT JOIN` defeats pushdown (`spans_idx` scans 1,000,000) while CH prunes (`Granules 1/123`, ~10k rows). It's predicate-through-join propagation (Run 148 `Join=NonCommutative`), not a broken index; subquery-prefilter + app-side correlation both prune → sidestepped, not a blocker (`per-signal-verdict.md` Q4). **Run 153 — STRATEGY (operator redirect): the Parallax-proxy lens** — Parallax owns OTLP ingest/routing/conversion, so GreptimeDB's native-protocol/ingest-ergonomics edge (Runs 150–152) is **neutralized**. Re-scoring on what survives (retrieval speed + build-on-top ecosystem) tilts the default to **ClickHouse** (the de-facto unified-obs backend: SigNoz/Uptrace/HyperDX/ClickStack); GreptimeDB stays for the metrics-cardinality / self-hosted-1×-S3-economics / auto-rebalance bet. No alternative (OpenObserve=competitor-platform, Quickwit=logs-only, InfluxDB3, VictoriaMetrics/Logs, StarRocks/Doris) beats them as an embeddable backend. Grouped-errors/metadata → relational store (**Turso default / Postgres fallback**, already chosen), NOT the columnar engine. Slowness=engine (closable), cheapness=object-store architecture — separate levers. New note `platform-fit-and-alternatives.md`. **Run 152 — SOURCE+LIVE (gentle): native TRACES → completes the adopt-native trio** — OTLP traces auto-map to a span table (`trace_id,span_id,parent_span_id,service_name,duration_nano,span_attributes,resource_attributes,…`, `otlp/trace.rs`) + auxiliary service/operation tables backing a **Jaeger-native read API** (`/v1/jaeger/api/services` HTTP 200 live). GT = OTLP-in→Jaeger-out zero-glue; ClickHouse needs external collector + custom schema + `jaeger-clickhouse` plugin. **Metrics/logs/traces all adopt-native on GT** (`write-path-and-ingestion.md`). **Run 151 — SOURCE+LIVE (gentle): native log ingestion = built-in pipeline (ETL) engine** — `src/pipeline` = processors+transforms; **`greptime_identity`** JSON→auto-schema **live-proven** (POST 2 JSON logs → table auto-created `[greptime_timestamp,latency_ms bigint,level,msg,service]`; new `trace_id` field auto-added = schema-on-write). ClickHouse has no in-db log-parsing pipeline → needs external collector + pre-modelled schema. Adopt-native-logs edge to GT (`write-path-and-ingestion.md`). **Run 150 — SOURCE+LIVE (gentle): metric engine = multiplexer over Mito2** — `DataRegion` (wide physical table, auto `AddColumn`) + `MetadataRegion` (moka-cached logical→physical map); **live-verified** two logical tables (`la(host)`, `lb(host,dc)`) sharing one physical region → cols `[__table_id,__tsid,dc,host,ts,val]`, `dc` auto-added, `__table_id` row-isolation. "10k metrics ≠ 10k tables" live-proven = the adopt-native-metrics backbone (`metric-cardinality.md`). **Run 149 — SOURCE+LIVE (gentle): continuous aggregation** — GT `CREATE FLOW` (streaming + batching, dedicated Flownode; `flows` catalog live) vs CH Materialized Views (insert-triggered + refreshable, `allow_experimental_refreshable_materialized_view=1` live). Both close *recurring* rollups → the agg-gap bites **only ad-hoc** analytics; small maturity edge to CH (insert-MV battle-tested). New note `rollup-and-continuous-aggregation.md`. **Run 148 — SOURCE (gentle): GT distributed read fan-out (`dist_plan`)** — `Categorizer` pushes scans/filters/steppable-aggs/sorts/limits to datanodes (steppable agg → two-stage partial+frontend-merge), but **`Join` = `NonCommutative`** → frontend fan-in; so the agg-gap **parallelizes** at scale while the join-gap **worsens** distributed (keeps the app-side-correlation blueprint). **Run 147 — SOURCE (gentle): PartitionTree memtable** dict-encodes label sets (mem-budgeted PK dictionary, no per-series cap) — grounds the cardinality-insensitive ingest win (Run 84/101) vs ClickHouse LowCardinality's 8192 cap
interactive, gaps compressed (~1.5–3×, JSON ~12×) — fixed-overhead-dominated, DIRECTIONAL only; magnitude + the v1.1
regression + the >300 ms analytical crossovers show only at the 1M–5M server tier. Canonical = the 1M matrix. **Run 144
— SOURCE (gentle): TWCS** grounds two findings — `TwcsPicker` compacts only WITHIN time windows, so a time-spanning
table keeps ≥1 SST/window → dedup-agg merges across them (the Run-142 ~8× cost; single-window → 1 SST → fast, Run 117),
and each window = own SST → TTL-expired window drops WHOLE (the cheap-retention pillar, now structural). **Run 143 —
benchmark-tier policy** (Mac froze on 5M: LOCAL small/100k preliminary, SERVER large/5M+ on request only). Earlier: **Run 142 — isolated
the 5M dedup-agg finding**: (A) dedup-mode metric agg is **~8× slower than append at 5M** on BOTH versions (m2m dedup
314 ms vs m2m_ap append 40 ms, same data) — the Run-117 dedup-merge cost at scale; blueprint nuance: use append_mode for
scrape-style metrics where (series,ts) is unique. (B) v1.1-nightly regressed the dedup path specifically (~2.8×: 314→867)
while improving append (40→26) — re-test on v1.1 GA. **Run 141 — 5M tier**
(the trust-the-numbers tier): GreptimeDB's anchored/keyed hot path HOLDS (anchored ~14 ms, last-value ~10 ms ≪ gate),
but heavy ANALYTICAL queries CROSS 300 ms at 5M (metric-agg 315–1021 ms, JSON 330 ms, in-DB join 659 ms) while ClickHouse
stays fast — scan/agg gaps WIDEN with scale (the DQ5 flip-trigger). ⚠ Surfaced a **v1.1-nightly dedup-agg regression**
(~2.5× slower than stable on the dedup metric table at 5M, persists post-compaction, append-mode unaffected; invisible at
1M) — re-test on v1.1 GA. **Run 143 — benchmark tier policy**: local laptop runs are now small
preliminary (`N=100k` default, ≥50k enforced), while `N=5M+` is server-only on explicit operator
request; forced compaction reduced stable dedup agg 314→~60 ms, so append still wins but the penalty
is compaction-sensitive. **Runs 139–140 — REPRODUCIBLE 4-way harness** (`bench/four-way/`): every benchmark stored as
code — `compose.yml` (4 builds) + `gen.sh` + `bench.sh` (20 queries × 4 builds); the matrix reproduces
from it. Earlier:
**Run 138 — time-range scan** (CH ~3 / GT ~5–9 ms, both time-prune; GT-nightly ~40% faster). **Run 137 — high-group-count agg 4-way** (`GROUP BY trace_id`, 70k groups): GT 21 / CH ~13 ms (~1.5×) — no high-group cliff on
GreptimeDB (the group-by path is well-behaved, unlike the dedup high-card-PK path); nightlies ≈ stables (CH-head's
initial 26 ms was a warmup/contention artifact, re-verified to ~13 ms). **Run 136 — count-distinct / cardinality panel 4-way**: low-card distinct CH ~1.7× (GT 20 / CH 12 ms) but high-card EXACT distinct
GreptimeDB ties/wins (1M unique: GT 30–33 / CH 36–37 ms; CH approx `uniq` HLL ~10 ms fastest); nightlies ≈ stables, all
≪ 300 ms. (Per the new AGENTS.md rule: every benchmark now runs on all 4 builds + updates four-way-version-comparison.md.) **Run 135 — latency-percentile panel** (p99 by service): CH ~11 ms / GT ~21 ms (~1.9×); the p50/p95/p99 panel = CH ~11 ms (one pass)
/ GT ~28 ms (~2.5×, GT's approx_percentile_cont scales per-percentile, no shared sketch) — both interactive; blueprint:
use a single-call/Flow-sketch for multi-percentile panels on GT. **Run 134 — SOURCE: Flat SST** (v1.0 GA): stores tag/PK columns as RAW dictionary-encoded columns alongside the encoded composite key, so
tag-keyed group-by/filter reads the raw column directly (no per-row composite-key decode) — the foundation behind the
prefilter (Run 121) + the tag-keyed agg improvements; dedup cost unchanged. **Run 133 — reconciled the broad-term full-text gap**: ~12× REPRODUCES on the canonical logs_b1 (5M, 699k matches: CH ~7 ms / GT ~85 ms, scan-
bound); Run-131's ~1.5× was a different corpus + CH tokenbf index (not comparable). Canonical broad-term gap = ~12×;
selective full-text stays a ~tie. **Run 132 — 4-way INGEST + STORAGE** completing the matrix: CH ~3.5× faster on synthetic INSERT…SELECT ingest (not the native path; GT's
real ingest = native bulk + cardinality-insensitivity) + ~1.2× smaller storage; both nightlies ~13–15% faster, no other
change; GT cardinality-insensitivity holds on both versions (1M-series ingest ≈ 12-series). **Run 131 — FULL 14-query 4-way matrix** (operator-requested "one clear comparison", `four-way-version-comparison.md`): every
load-bearing query across GT v1.0.2 / v1.1-nightly / CH 26.5 / 26.6-head. GT-nightly equal-or-faster on all at 1M (aggs
~20–30%, join 65→36 ms; no regressions at that tier), CH-head perf-flat+stricter; GreptimeDB wins last-value ~2× + ties selective
full-text, ClickHouse leads anchored/scan/log-tail/JSON/join — all ≪ 300 ms on every build, verdict identical. **Run 130 — COMPREHENSIVE 4-way re-check** (operator-requested, all load-bearing speed findings across GT v1.0.2 / v1.1-nightly /
CH 26.5 / 26.6-head on identical fresh data): **nightlies ≈ their stables** (within noise — no clear v1.1/26.6 perf
change; corrects Run 129's "~25% agg" to within-noise), and the GT-vs-CH gaps (anchored ~4–6×, scan ~5×, agg ~2×,
JSON ~10× with the cast) are IDENTICAL on all four — verdict holds on every build. **Run 129 — 4-WAY NIGHTLY comparison** (operator-requested): GT v1.1.0-nightly + CH 26.6-head vs the v1.0.2/26.5 stables. GT v1.1 ~25%
faster flushed metric-agg (~18 vs ~24 ms @2M), NO JSON-attr change (~56 ms); CH 26.6 perf-flat but now ENFORCES the
typed-subcolumn cast in GROUP BY — correcting Run 104's dynamic-attr ~57× to **~8×** (the ~57× was 26.5's lax no-cast
path). Nightlies don't move the headline; verdict holds. **Run 128 — pins RE-VERIFIED via release pages** (not just asserted): GreptimeDB v1.0.2 = latest stable (v1.1.0 is nightly-only, not GA),
ClickHouse v26.5.1.882 = highest feature line (later-dated 26.2/26.3/26.4 tags are older-line LTS backports); both
current, no bump. Future trigger: GreptimeDB v1.1 GA (Q2 — JSON Type v2 narrows the dynamic-attr gap) = re-pin + re-run
104/96/122. **Run 127 — trace-explorer "slow error spans"** query (find+rank slowest errored spans): CH ~10 ms / GT ~24 ms (~2.4×), both
interactive — completes trace-query coverage (anchored waterfall ~18 ms + this search ~24 ms); every core Parallax view
across all four signals is now confirmed sub-perceptible on GreptimeDB. **Run 126 — native metric engine**: physical table (`greptime_timestamp`/`greptime_value`) creates cleanly, but logical tables are
AUTO-created via Prometheus remote-write/OTLP ingestion, NOT hand-DDL (explicit `ENGINE=metric WITH(on_physical_table=)`
is finicky about time/value column mapping) — adopt-native-metrics nuance: use the ingestion path to auto-provision.
**Run 125 — agg gap is NOT JIT either**: CH with compile_aggregate_expressions=0 is still ~3.7× faster than GT (~31 vs ~116 ms; JIT-off was
even faster on a heavy agg). So the ~2–3× metric-agg gap is NEITHER batch (124) NOR JIT (125) — it's diffuse vectorized-
execution maturity (SIMD kernels + hash-agg), the slowest-closing gap (no single PR; accrues as DataFusion's core
matures). Engineering not physics, but the longest timeline. **Run 124 CORRECTED parity #2**: lowering ClickHouse's max_block_size to GreptimeDB's 8192 barely changed CH's agg (~37→~38 ms) and CH is
STILL ~3× faster (~38 vs ~116 ms) — so batch size is NOT the agg-gap lever; the roadmap's "raise batch_size = cheapest
win" is disproven, the gap is JIT/SIMD/codegen (the expensive upstream-DataFusion part). **Run 123 — the OTHER side of gap-closing (honest balance)**: #2 batch_size is STILL untouched in v1.0.2 (state.rs sets no with_batch_size,
8192 default holds, SET still rejected) — so the ~2–3× agg-throughput gap has NOT moved, unlike the SST-layer wins
(TopK/Flat-SST/prefilter). Gap-closing is uneven: GreptimeDB ships what it owns in the scan layer; the execution-core
throughput knobs ride upstream DataFusion. **Run 122 MEASURED the prefilter working** — GT selective wide-row scan prunes the wide-column decode ~3× (~16 ms vs ~50 ms full-decode), so
the v1.0.2 `prefilter.rs` (Run 121) is shipped AND functional; parity #3 PREWHERE shifted from "catastrophic full-decode"
to "the same ~5× throughput gap as every scan" (residual is engine throughput, not missing late-materialization), both
interactive. **Run 121 — SOURCE gap-closing**: GreptimeDB v1.0.2 shipped `prefilter.rs`, a PREWHERE-style late-materialization framework (parity #3 was
"missing" at pass-77, now PARTIALLY CLOSED — PK/partition-scoped, wired into the Flat read path). Third source-confirmed
shipped scan-engine gap-closing (after TopK + Flat SST) — validates the DQ6 "closable in Rust, being closed" thesis.
**Run 120 re-verified the native observability-protocol trio LIVE** (DQ3): GreptimeDB OTLP receiver (HTTP 400=exists) + PromQL API (200) +
Jaeger query API (200) all GA/default-on, vs ClickHouse PromQL still experimental + OFF by default
(`allow_experimental_time_series_table=0`) with OTLP collector-only + Jaeger external-plugin — adopting CH still costs a
PromQL+OTLP+Jaeger compat layer; GT's are turnkey. **Run 119 — issue-list query** (Sentry-style group-errors-by-fingerprint, the #1 error-tracker view): CH ~10 ms / GT ~26 ms (~2.6×, scan-agg
class), both interactive — rounds out Parallax core-query coverage (anchored bundle, trace waterfall, log tail/search,
metric panels, issue list): CH ~2–7× faster on analytical shapes but EVERY core view is sub-perceptible on GreptimeDB.
**Run 118 CORRECTED a Run-114 overclaim** — ClickHouse does NOT have a symmetric schema-discipline trap: a wrong (high-card-first) ORDER BY
costs CH only ~11% storage with no scan/lookup penalty (no read-path dedup-merge), vs GreptimeDB's ~16–44× hot-scan
catastrophe for the analogous PK mistake. CH is markedly more schema-mistake-tolerant — a fair operability point added
to "where ClickHouse is genuinely better" (DQ2). **Run 117 — SOURCE + live** grounded the Run-114 dedup gotcha in GreptimeDB v1.0.2 code (`flat_merge`/`seq_scan.rs:224`): the penalty is
per-series-boundary work in the merge/dedup of overlapping sorted runs, concentrated in the unflushed MEMTABLE — a
high-card-PK dedup table scans ~1235 ms while memtable-resident but ~28 ms once flushed to a single SST (~44×). So it
hits HOT/recent data (what observability queries most); append_mode is the universal fix. **Run 116 re-verified freshness** — both visible-on-write (5/5 trials each, no flush barrier): GT from the LSM memtable, CH on insert-ack; a
tie, not a decision axis (default sync path; async batching on either side trades freshness for throughput). **Run 115 refined the Run-114 dedup gotcha** — the penalty scales with SERIES COUNT (merge boundaries) not rows: cheap for metric labels (40k
series / ~200 rows each → ~110 ms) and catastrophic only for per-event ids (1M single-row series → ~1220 ms), so
dedup/`last_non_null` is FINE for the metric engine (validates "ADOPT native metric engine"); the gotcha is confined to
needing append_mode on EVENT tables. **Run 114 — BLUEPRINT GOTCHA quantified**: GreptimeDB default-dedup mode + high-cardinality PK = ~16× slower scans (the DedupReader merges
every series), and a high-card PK itself ~5× — so the naive `PK(span_id)+default` is ~80× slower than the correct
`PK(service,name)+append_mode` (which the blueprint already specifies). append_mode is mandatory for high-card event
signals; reserve dedup for low-card upsert signals. **Run 113 — counter-rate panel** (the #1 observability metric query): CH ~12 ms / GT ~19 ms (~1.6×, smallest agg gap — shrinks as per-row compute
grows), both interactive. Completes the metric-panel picture: last-value GT-wins ~2.4× (109), rate ~1.6×, bucketed line
~2× (96), flat avg ~3× (96) — across real dashboards GT ranges from winning to ~3× behind, all interactive. **Run 112 re-verified concurrent ingest+query** — under sustained ingest (1M rows) neither engine blocks reads (anchored query flat: CH
2→2 ms, GT 10→10 ms, ~1.0× penalty both) and neither explodes storage (CH merged 50 inserts→2 parts, GT absorbed 1M in
the LSM memtable, sst_num=0); Parallax's continuous-ingest-while-querying mode is safe on both. **Run 111 refined retention/TTL** — narrower gap than "CH always rewrites": ClickHouse drops a *fully-expired* part cheaply (verified),
rewriting only a *boundary* part (expired+live mixed) or a non-time-ordered part, so time-ordered ingestion is cheap on
both; GreptimeDB's edge is zero-config TWCS vs CH cheap-when-time-partitioned (blueprint already does it), and GT TTL
purge is eventual/background not on-demand. **Run 110 re-verified schema-on-write / OTLP-drift** — GreptimeDB InfluxDB-line write of a new tag+field auto-adds the columns (HTTP 204,
zero migration, old rows NULL-backfilled) while ClickHouse rejects unknown-column inserts (`Code: 16
NO_SUCH_COLUMN_IN_TABLE`, needs ALTER or a JSON column with the ~13–57× query penalty); a real GT ingest-ergonomics win
for drifting telemetry. **Run 109 — GreptimeDB WINS the last-value/"current value" stat-panel query** ~2.4× (GT ~17 ms / CH ~41 ms): time-sorted layout makes
latest-per-series a cheap tail read vs CH's argMax full-scan — so the vendor "GT loses lastpoint to TimescaleDB" does
NOT carry to ClickHouse (engine-relative); a real GT metric-speed win alongside cardinality-insensitive ingest. **Run 108 verified the Run-107 blueprint claim** — PK(service) is only ~10% faster than PK(service,level) for the log-tail (~27 vs ~30 ms), so
keying by service is a minor optimization, NOT the main lever; the ~7× gap to ClickHouse is the structural #5
sort-locality, not PK choice (honest correction to Run 107's overstatement). **Run 107 benchmarked the log-explorer hot path** (service time-DESC tail + errors-in-window): CH ~4/~10 ms vs GT ~28/~60 ms (~6–7×, the
larger warm gap, from CH's `(service,ts)` sort-key locality — a concrete instance of the #5 alternate-ordering gap);
both ≪ 300 ms so GT is interactive; blueprint fix = key GT logs by `service` not `service,level`. **Run 106 audited
GreptimeDB's own marketing pages** (`compare/click_house` + 15 blogs, `vendor-claims-audit.md`) — verdict: the page
sells GT on fit/storage/economics/native-protocols and **never claims raw-analytical-speed superiority** (the one thing
our data would refute), so it did NOT manipulate the decision; misleading bits (Poizon seconds→ms = GT-vs-ETL,
structured-keyword "GT faster" = unstated config, TSBS 67× = vs row-store Postgres) are peripheral. Live-verified the
RC2 "100× TopK" dynamic-filter pushdown on our v1.0.2 (GT ~20 ms / CH ~7 ms ORDER BY…LIMIT) — concrete DQ6 gap-closing
evidence. Corrections folded: disk index-file cache exists too; OTel-Arrow is experimental not GA; JSON Type v2 (v1.1/Q2)
will narrow the Run-104 attr gap. **Run 105 re-verified PromQL vs SQL** — GT native PromQL ~675 ms vs GT SQL ~120 ms vs CH SQL ~55 ms on `avg by(service)` (~5.6× GT-SQL,
ordering CH SQL > GT SQL > GT PromQL, Run 44 reproduces); metrics→GT is capability/ergonomics not speed, and a wide
PromQL range is OVER the 300 ms gate so drive hot panels with SQL/Flow. **Run 104 re-verified dynamic-attr JSON path queries — gap WIDENED to ~57×** (CH ~1 ms / GT ~57 ms @200k, was ~13× at Run 61): CH's 26.x
new-`JSON`-type typed-subcolumn read matured (~6→~1 ms) while GT's per-row jsonb parse is unchanged; bites only on
undeclared-attribute analytics at volume — Tier-A column-promotion + anchored bundle fetch sidestep it. **Run 103
re-verified the cross-tier in-DB join pushdown** — CH prunes the anchor through the join (~3 ms) while GT full-scans the input
(~53 ms, ~17×, parity-roadmap #8 optimizer gap); the subquery-prefilter workaround cuts GT to ~19 ms, and Parallax's
app-side correlation sidesteps it entirely (all ≪ 300 ms; Run 81 reproduces). **Run 102 re-verified the unindexed/ad-hoc scan gap warm** — ~2–5× shape-dependent at 1M (point-filter ~5×, full-scan aggregation ~2×), NOT
Run 31's ~10× which was cold/HTTP-wall inflated (confirms the Run 40 correction); all ≪ 300 ms at 1M, so the DQ5
flip trigger needs GB–TB cold scale, not just an ad-hoc shape. **Run 101 re-verified ingest cardinality-insensitivity** — going 12→1M distinct series, GreptimeDB ingest slows only ~1.16× (≈flat) vs
ClickHouse ~1.53× (plain String) / ~2.6× (idiomatic LowCardinality, Run 84); GT has no cardinality knob to mis-size
(metric-engine `__tsid`), the clearest high-card GT win is the ingest axis. **Run 100 re-verified storage/compression** across all four signal tables — no blanket winner, pattern-dependent: GT wins high-card metrics
(`metrics_hc` 8M 38.6 < CH 57.3 MiB, the Run-79 crossover on a real table), CH wins low-card metrics via codecs
(`metrics_real` 1.01 < GT 1.89 MiB, Gorilla — parity #7) + spans; logs ~wash; raw bytes second-order to object-store
request economics. **Run 99 re-verified THE load-bearing anchor** — the Q6 evidence-bundle composite (all signals for one trace_id) is still not latency-bound on
either engine: CH ~5 ms / GT ~16 ms warm, both ≪ 300 ms (faster than Run 16's 10/33, same ~3× ratio, no drift) — the
"fit not speed" thesis holds. **Run 98 re-verified full-text log search end-to-end** (selective exact-term ~3× competitive: CH ~3 ms / GT ~10 ms via bloom+`matches_term`;
the ~18× `matches()`-on-bloom artifact still full-scans ~155 ms; broad-term ~12× scan-bound — Runs 48–49 reproduce, no
drift; adopt-native-logs = ADOPT structure + ADD message fulltext, carry the pairing rule). **Run 97 re-verified the
trace-waterfall hot path** (flat span fetch CH ~3 ms / GT ~18–20 ms warm, both ≪ 300 ms; in-DB table-self-join recursive
CTE still errors on GT v1.0.2 while pure recursive works — Run 68 reproduces, no drift; app-side tree build is the
dominant pattern so adopt-native-traces stands). **Run 96 re-verified the metric-agg warm gap** (~3× flat `avg by service`, ~2× the realistic bucketed line-chart panel — the gap is
query-shape-dependent: scan-bound ~3× ceiling, compute-heavier panels trend to ~2×; both sub-300 ms warm on GreptimeDB,
interactive either way). **Run 95 closed the anchored-tie-at-scale question** — a clean 5M-*distinct*-trace partitioned test: GreptimeDB anchored lookup ~7 ms warm
(flat-to-faster than 1M) vs ClickHouse ~3 ms, both ≪ the 300 ms gate, so the tie does not widen with scale. **The
verdict now also carries DQ6 — the long-term *investment* decision** (is the speed gap closable or a physics wall?):
the parity-roadmap's per-gap physics-wall test finds *no* architectural wall (7/8 engineering, #6 time-only, #5 a
design-flavoured residue defused by `trace_id` partitioning), the two heaviest gaps ride shared DataFusion + Parquet-
Variant roadmaps, and GreptimeDB is the Rust substrate the operator can contribute to — so the long-term bet reinforces
the fit choice. Earlier: Run 44 closed the twice-owed
metrics item (GreptimeDB's *native PromQL* path is ~5× slower than its own SQL at 40k series
— a `SeriesNormalize` fixed-setup cost, so metrics→GreptimeDB is capability, never speed);
Runs 45–46 **built both implementation schemas live** (the "buildable design" bar) —
GreptimeDB needed 7 reserved-keyword columns quoted + the metric-table PK fixed, ClickHouse
needed only the `text` tokenizer fixed (`'default'`→`splitByNonAlpha`). Both designs now
verified buildable; ClickHouse S3-disk tiering is the remaining live gap.
**Verdict (sharpened through pass 87): GreptimeDB on fit** — metrics/PromQL-native (GA
vs ClickHouse's experimental), ingest/freshness/upsert ergonomics, object-store +
replication economics, horizontal scaling, Rust — **not on raw speed** (ClickHouse leads
SQL scan/aggregation by its vectorized C++ engine, and broad-term log analytics;
but the headline "~18× full-text" was a **backend/function misconfiguration** — Runs 48–49:
`matches()` on a bloom index full-scans, but with the correct pairing selective full-text is
~6 ms (tantivy+`matches`) / ~8 ms (bloom+`matches_term`) vs CH ~3 ms = **~2×, not 18×**;
log-search dissolves for interactive search, residual is broad-term analytics only; and
Parallax's *anchored* evidence-bundle hot path is **not latency-bound** on either). The
remaining open questions are **harness-gated** (multi-node, 1B-doc cold, sized
high-card storage, strict-durability, multi-replica S3 cost) — handed to
`storage-benchmark-prototype.md`. The loop now runs in maintenance/drift-watch: per-pass
version re-check + re-verification.

## Purpose

This folder holds a deep, under-the-hood technical comparison of **GreptimeDB**
and **ClickHouse** for the Parallax storage layer. It answers one question, at the
level of the actual implementation rather than marketing:

> How does each system work internally, which design decisions make each one fast
> or slow, and — for Parallax's signals (metrics, logs, traces, and cross-signal
> evidence-bundle correlation) — which should we build on, and why?

It is driven by the loop brief
[`prompts/greptimedb-vs-clickhouse-internals.md`](../../../prompts/greptimedb-vs-clickhouse-internals.md),
which runs indefinitely and deepens these notes one subsystem at a time until the
operator stops it.

## How this fits with the existing storage research

This is the **white-box** layer. It explains the *why* behind the *what* the other
documents establish:

- [`../greptimedb-storage-evaluation.md`](../greptimedb-storage-evaluation.md) —
  strategy/fit evaluation (reasons *about* the systems).
- [`../observability-storage-benchmark-plan.md`](../observability-storage-benchmark-plan.md)
  — what to measure and why.
- [`../storage-benchmark-prototype.md`](../storage-benchmark-prototype.md) — the
  runnable black-box harness that produces numbers and holds veto power over the
  default storage choice.

The benchmark shows *that* one system is faster; this folder must explain *why*,
from the data structures and code paths — and the two must agree. A benchmark
number the internals cannot explain is a flag that one of them is wrong.

## Version pins (re-check and bump every pass)

As of 2026-05-25 (re-verified through pass 91 — pins still current; GreptimeDB
`v1.1.0`/`v1.0.0` newer tags are nightly-only, GA stays `v1.0.2`; ClickHouse
`v26.5.1.882-stable` still the latest stable *feature* line — later-dated
`v26.2.19.43`/`v26.4.3.37` are lower-line LTS/backport patches, not higher):

| System | Pinned version | Source commit | Notes |
| --- | --- | --- | --- |
| GreptimeDB | `v1.0.2` (GA 2026-05-14) | `0ef54511f710f0ef2c05941c8c600bb4c1fd46c8` | Latest GA; `v1.1.0-nightly` exists but is not stable. |
| ClickHouse | `v26.5.1.882-stable` | tag obj `fae722ba…`; **commit read `5b96a8d8a5e2f4800b43a780911a39dc5a666e1c`** | Latest stable; LTS line is `v26.3.12.3-lts` (`f118ee7c3b4c1a57dde6a389e5c3e29080f38c5d`). |

## Method

- Compare the latest stable release of each system; record exact versions and the
  source commit SHA read in every note (version-freshness rule).
- Read the architecture docs to orient, then confirm load-bearing claims against
  the cloned source (GreptimeDB in Rust, ClickHouse in C++). Cite file paths and
  commits. When docs and code disagree, trust the code.
- Every "X is faster" claim carries a *because* (a concrete mechanism) and a
  *scenario* (signal, query shape, cardinality, cache state, single-node vs
  scaled).
- Verify the operator hypothesis (GreptimeDB fastest, then ClickHouse) honestly;
  a fully-explained result that contradicts it is the most valuable outcome.

## Evaluation axes (priority order)

1. Speed — ingest-to-queryable freshness and evidence-bundle/correlation query
   latency under concurrent ingest+query.
2. Cost — retained size and compression by signal, object-vs-local economics,
   compute per ingested GB and per query class.
3. Scaling — single-node ceiling and horizontal scale-out (horizontal first;
   vertical-only is a flagged limitation).

## Planned notes

These are produced and grown by the loop; this index is updated as they land.

| File | Scope | Status |
| --- | --- | --- |
| `README.md` | Index, method, version pins, status. | seeded |
| `greptimedb-internals.md` | GreptimeDB architecture and code-path teardown. | drafted (pass 1: topology + mito2 storage engine; pass 32: metric-engine logical→physical layout confirmed live — `__table_id`/`__tsid` + label-column union in one physical region set, avoids per-metric region explosion) |
| `clickhouse-internals.md` | ClickHouse architecture and code-path teardown. | drafted (pass 2: topology + MergeTree part/granule/mark, skip indexes, codecs, merge variants; deeper KeyCondition/merge-selector/text-index/S3-cache dives pending) |
| `write-path-and-ingestion.md` | Ingest → durable → queryable, both systems, with the freshness consequence. | drafted (pass 9 + Run 5: freshness = tie (both visible-on-write, no flush barrier); GreptimeDB write-path edge = LSM absorbs small writes (no ClickHouse part-explosion) + native OTLP/Prom ingest; bulk throughput both >1M rows/s; pass 90 Run 53: **concurrent ingest+query re-verified** — neither blocks reads (CH 1.0–1.13×, GT 1.15–1.19× penalty, tighter than Run 13's 1.38–1.55×); anchored hot path ~immune on both (point lookup flat under ingest), scan-agg absorbs contention; CH held 17 parts / GT 1 SST under sustained load (no explosion); loads unmatched (matched-rate owed); pass 33: native InfluxDB-line ingest confirmed live — schema-on-write auto-creates the table (tags→PK, field→DOUBLE, auto TIME INDEX, merge_mode=last_non_null), no DDL/collector; OTLP metrics is protobuf-only (JSON rejected); pass 56 Run 33: async-insert mechanism — CH `AsynchronousInsertQueue` buffers small writes server-side, flush triggers 10MiB/450 queries/50-200ms adaptive busy-timeout → solves part-explosion but data invisible+non-durable until flush (≤200ms window; wait=1 blocks client, wait=0 = loss window), vs GreptimeDB LSM memtable = same absorption but queryable+durable on write (no window); pass 46 Run 25 re-verified OTLP at CH 26.5 — **no drift**: ClickHouse still has no native OTLP receiver (no otlp/otel function, no OTLP handler in src/Server), needs an OTel Collector; GreptimeDB native OTLP metrics/traces/logs (`http/otlp.rs`, live 400=exists). Contrast: ClickHouse's 26.x protocol investment went to Prometheus, not OTLP) |
| `read-path-indexing-and-execution.md` | Query planning, indexing, execution, scan-vs-skip, joins. | drafted (pass 3: pushdown, scan/skip order, PREWHERE vs row-group pruning, join strategy; pass 5: join verdict corrected by Run 2 EXPLAIN — both engines prune the anchor before joining, so join algo is not a differentiator for anchored evidence-bundle queries) |
| `rollup-and-continuous-aggregation.md` | Rollup/correlation tooling: GreptimeDB Flow engine (streaming + batching) vs ClickHouse MV + AggregatingMergeTree, for Parallax metric downsampling + issue rollups. | drafted (pass 27: wash with opposite tilts — GreptimeDB Flow cleaner/metric-native (CREATE FLOW … SINK TO … EXPIRE/EVAL) vs ClickHouse MV+AggregatingMergeTree more mature but per-block + -State/-Merge ceremony; neither moves the verdict; **pass 106 Run 70 re-verified: both produce identical minute+svc rollups (15·2/5·1/30·1); new freshness tilt to CH — push-MV materializes synchronously inside the INSERT vs GT Flow batched (FLUSH_FLOW/interval)**) |
| `caching-and-cold-warm.md` | Subsystem #7: cache hierarchies + the cold-vs-warm divergence mechanism — explains why warm small-scale runs favor ClickHouse but cold object-store re-reads can favor GreptimeDB (the regime Parallax lives in). | drafted (pass 24: CH 5GiB mark cache + uncompressed OFF, local-disk-tuned; GreptimeDB object-store read cache + few-object layout = few cold S3 GETs; mechanism behind JSONBench cold-run; **pass 92 Run 55/B10 MEASURED cold anchored read — two-sided, corrects the prediction:** request count → GreptimeDB (9 vs 18 GETs), but cold **egress** for a *selective* query → ClickHouse ~80× (294 KiB granules vs ~23 MiB whole-SST; small-SST-inflated, at-scale owed); **warm/repeat → GreptimeDB** (write-through persistent local cache survives restart → ~0 S3 after first touch). Wide cold scan still favors GreptimeDB (JSONBench regime); selective cold favors CH) |
| `compaction-and-merge.md` | Subsystem #5: GreptimeDB TWCS (time-window) vs ClickHouse SimpleMergeSelector (size-tiered), write amplification, read-speed/freshness effect. | drafted (pass 23: TWCS bounds write-amp on aged time-series — sealed windows never re-merged — vs ClickHouse O(log N) size-tiered re-merge toward few 150GB parts for fast full scans; ties to B9/B10) |
| `deletes-and-mutations.md` | Corrections / GDPR-erase / updates: ClickHouse mutations (+ lightweight delete) vs GreptimeDB tombstone+upsert. | drafted (pass 51, source+live Run 29: **DELETE ≈ parity** — ClickHouse lightweight `DELETE FROM` = `_row_exists=0` mask mutation (GA, default; live part `_2` bump, not a surviving-row rewrite) caught up to GreptimeDB's tombstone+read-filter (`filter_deleted`); both read-immediate. **UPDATE → GreptimeDB** — GA zero-setup upsert (re-insert + dedup last-wins) vs ClickHouse heavy `ALTER UPDATE` part rewrite; **pass 96 Run 59 re-verified dedup + proved the partial-upsert gap: GreptimeDB `last_non_null` merges per-field (a=10,b=hello) vs ClickHouse RMT `FINAL` whole-row replace LOSES the field (a=NULL,b=hello)**; CH lightweight UPDATE exists (`enable_lightweight_update=1`) but **experimental + needs per-table `enable_block_number_column=1`** (live-rejected on a plain table). LSM-native correction ergonomics edge GreptimeDB, strongest on UPDATE; honest CH lightweight-delete catch-up; no verdict flip; **pass 102 Run 66 re-verified DELETE parity + pinned UPDATE: GreptimeDB has NO `UPDATE` statement ("not supported"), update = (PK,ts)-keyed upsert (same ts overwrites, new ts = new version) — append/upsert model not relational row-update; CH lightweight UPDATE still gated**) |
| `projections-and-access-paths.md` | Serving multiple access paths from one table: ClickHouse projections vs GreptimeDB secondary indexes. | drafted (pass 50, source+live Run 28: **ClickHouse projections** = a second physical `ORDER BY` (or pre-agg) inside each part, optimizer-transparent — live `EXPLAIN` chose `ReadFromMergeTree (p_service)` for a service query on a trace_id-ordered table; **storage ~doubles** per normal projection (2→4 MiB measured). **GreptimeDB has no projections** (parser rejects) — uses secondary indexes (inverted/skipping) = row-positions at index size. Tradeoff: projections win **scan-by-alternate-ordering** (no GT equivalent) at ~2× storage; GT index leaner for **anchored point/filter** (Parallax's shape). No verdict flip; sharpens read-path/cost; **pass 107 Run 71 re-verified: storage 2.41→4.52 MiB (~1.9×), optimizer picks `ReadFromMergeTree(p_b)` Granules 1/62, GT rejects PROJECTION; linked to Run 63 — a trace_id-ordered projection is how CH could also get cold-anchor clustering, the alternate-order GT structurally lacks**) |
| `trace-span-tree.md` | Traces signal: span-tree reconstruction (flat anchored fetch + app build vs in-DB recursive CTE). | drafted (pass 49, live Run 27 — **CORRECTED pass 104 Run 68**: recursive CTE is NOT a tie — counter form works on both, but the **table-self-join span-tree CTE errors on GreptimeDB v1.0.2** ("project index out of bounds") while ClickHouse runs it (3 rows/depth 2) → in-DB span-tree recursion is a CH capability edge. Dominant pattern is the **flat anchored fetch** (all spans of a trace_id, app builds the tree) = the anchored-lookup question already settled: CH 4ms (`ORDER BY (trace_id,ts)` sort-key locality) vs GT ~54ms HTTP (inverted-index lookup + fixed floor). Span-tree *retrieval* is NOT a new differentiator — reduces to anchored fetch (CH edge); but in-DB span-tree *recursion* is a CH edge (GT can't). Low impact: dominant pattern = flat fetch + app-side build (works on both); doesn't move verdict) |
| `metric-cardinality.md` | Checklist lead #6: how each engine physically stores many series (high-cardinality metrics). | drafted (pass 48, source+live Run 26: **GreptimeDB metric engine built for high card** — `__tsid` label-set hash (perf-critical, has its own bench) over a shared physical wide table + PartitionTree memtable (dict-encoded label sets, sharded, multi-partition, no per-series cap). **ClickHouse `LowCardinality` dict caps at 8192** distinct (live), then "writes in an ordinary method" = the high-cardinality cliff; needs careful `ORDER BY` or the experimental TimeSeries engine. Two-sided: high-card **storage/ingest ergonomics → GreptimeDB**, high-card **aggregation latency → ClickHouse** (~2× warm, Run 37; the Run 11 "~10×" was a cold/first-run artifact, larger cold). **pass 112 Run 76 advanced B13 (200k-series storage): CH `LowCardinality` 9.64 MiB vs GT plain-table 11.99 MiB (~1.24×, CH wins raw storage); the 8192 cliff is GRACEFUL (LC 1.53 MiB still < String 1.99 MiB, not an explosion). So GT high-card edge = ingest ergonomics (no cap), NOT storage; **pass 113 Run 77 closed B13: metric engine = 12.63 MiB (NOT smaller than plain 11.99 — `__tsid` is overhead on top of labels), so CH `LowCardinality` 9.64 wins high-card storage ~1.3× over BOTH GT layouts; corrects Run 26 "storage→GT"; GT high-card win is operability/no-cap only, agg latency→CH too; **pass 114 Run 79 CURVE found a CROSSOVER: CH `LowCardinality` wins low-mid (1k 8.18 vs 9.18; 200k 9.64 vs 11.99) but GT wins at 1M unique series (12.36 vs CH 16.51 ~1.34× — CH `LowCardinality` blows up all-unique). Storage winner is cardinality-dependent; **pass 117 Run 84 closed high-card INGEST: GT cardinality-INSENSITIVE (1k→1M series ~flat 357→381ms) vs CH ~2.6× slower — GT's clearest high-card win is the ingest axis. Full picture: ingest→GT, storage→crossover, agg→CH, no-cap→GT**) |
| `promql-and-metrics-query.md` | The PromQL planning path + a verdict-material re-verification of "ClickHouse has no PromQL". | drafted (pass 44, source+live Run 23 — **verdict drift caught**: ClickHouse 26.x **does** have PromQL (`prometheusQuery`/`prometheusQueryRange` table functions over the experimental `TimeSeries` engine; `allow_experimental_time_series_table` default 0) — "no PromQL" REFUTED. **GreptimeDB** PromQL is GA+default-on: custom DataFusion plan nodes (`InstantManipulate`/`RangeManipulate`/`SeriesNormalize`/`SeriesDivide`/`HistogramFold`/`Absent`/`prom_rate`) via `PromExtensionPlanner`, Prom HTTP API + `TQL`, live-confirmed zero-setup. Re-rated: metrics win = **GA-ergonomic vs experimental-off-by-default-setup-heavy**, not present-vs-absent; narrows but doesn't flip verdict; corrected verdict/per-signal/write-path. **Pass 45 Run 24 measured the maturity gap end-to-end:** ClickHouse `TimeSeries` has **no direct INSERT/SELECT** ("not supported yet"), ingest is **remote-write-only**, query **table-function-only** — `prometheusQuery`/`Range` execute `rate()` but need a remote-write client to feed; GreptimeDB ran `TQL EVAL rate(...)` to **real values** (0.72/1.17) after a zero-ceremony influx-line load. Capability present both; maturity/ergonomics gap large+concrete; **pass 98 Run 62 re-verified live — no drift: GT PromQL GA zero-setup → real values (50.77 / 49.98) on a plain table; CH `allow_experimental_time_series_table=0`, `TimeSeries` INSERT/SELECT still "not supported yet" (remote-write+table-function only). Verdict #1 pillar stable**) |
| `indexing-internals.md` | Checklist #3 storage half: index file formats (GreptimeDB Puffin sidecar vs ClickHouse `.idx` per part) + the richer≠faster paradox. | drafted (pass 43, source+live Run 22: **GreptimeDB = one `.puffin` sidecar per SST** (same UUID as `.parquet`) holding all indexes as blobs — inverted (`fst`+`roaring`, true term→rows secondary index), full-text (`tantivy` 0.24 Lucene-class, or `fastbloom` variant), bloom skipping; granularity configurable/fine. **ClickHouse = `primary.cidx` sparse primary + one `skp_idx_<name>.idx`+`.cmrk4` per skip index per part**, `GRANULARITY×8192` coarse granule-pruning. Runs 48-49 corrected the old full-text gap: `matches()` on bloom full-scans, while the correct pairings prune — tantivy+`matches()` ~6 ms and bloom+`matches_term()` ~8 ms. Remaining gap = broad-term scan integration; **pass 108 Run 72 re-verified index file formats live: GT `.puffin` in `index/` subdir (1/SST, all indexes as blobs) vs CH `text`=`.dct.idx` 97MB dict + `.pst.idx` 81MB postings + skip + marks (true inverted index), 37 files/part → the root of the Run-54 object-count gap (2 files/SST vs 37/part)**) |
| `query-execution-engine.md` | Checklist #4 execution half: ClickHouse bespoke C++ vectorized pipeline (block/JIT/SIMD) vs GreptimeDB DataFusion-over-Arrow — the mechanism behind the remaining measured throughput gaps. | drafted (pass 42, source+live Run 21: **ClickHouse** 65409-row blocks (~8×), `max_threads` pipeline lanes, **LLVM JIT** `compile_expressions`+`compile_aggregate_expressions=1` live, specialized adaptive hash aggregation, PREWHERE → the scan/aggregate throughput bar. **GreptimeDB** DataFusion `=52.1` over Arrow `RecordBatch` (~8192), `target_partitions`+custom `ParallelizeScan`, `MergeScanExec` fan-out, younger codegen — competitive but trades raw kernel speed for **extensibility** (PromQL/metric-engine plug-in nodes = the metrics-native win). Anchored Q6 and exact-term log grep stay not-throughput-bound; gap bites on ad-hoc large scans. DataFusion improving fast — re-check on bumps) |
| `platform-fit-and-alternatives.md` | **Strategic re-decision (operator redirect, Run 153):** the Parallax-proxy lens, alternatives survey, the metadata/error-grouping split, and "why slower vs why cheaper." | drafted (Run 153). **Proxy lens:** Parallax owns OTLP ingest/routing/conversion → GreptimeDB's native-protocol/ingest-ergonomics edge (Runs 150–152) is **neutralized**; re-scoring on what survives (retrieval speed + build-on-top ecosystem, both ClickHouse wins; object-store economics + cardinality + auto-rebalance, GreptimeDB) tilts the default to **ClickHouse** (de-facto unified-obs backend — SigNoz/Uptrace/HyperDX/ClickStack all build the platform layer on CH, exactly what Parallax is). GreptimeDB reserved for the metrics-cardinality/PromQL · self-hosted-1×-S3-economics · mandatory-auto-rebalance bet. **Alternatives:** none beats CH/GT as an *embeddable backend* in the language filter — OpenObserve is a competitor *platform* not a DB, Quickwit is logs/traces-only, InfluxDB3 metrics-centric, VictoriaMetrics/Logs are split products, StarRocks/Doris carry JVM-FE/ops risk. **Data model:** metrics/logs/traces/raw-errors → columnar (CH/GT); **Sentry-style grouped errors + metadata (mutable/relational/OLTP) → the relational store already chosen: Turso default / Postgres fallback**, NOT the columnar engine (Sentry's CH "replacements consumer" is the warning); cold tier → object storage. **Slower≠because-of-remote-storage:** the query gap is the *engine* (DataFusion vs CH C++/JIT, measured on local disk, closable); the cost win is the *object-store architecture* (compute/storage separation + 1× vs N× S3 + elastic compute = fewer always-on servers) — separate levers. Operator's "fewer servers, cheaper storage" intuition is GreptimeDB's strongest *surviving* argument under the proxy) |
| `storage-cost-and-tiering.md` | **Storage-architecture thesis + the cost axis + the hot/cold hybrid (operator redirect, Run 161).** Verifies CH=performance/local-first vs GT=S3-native/cost-first; adds cost (not just speed); evaluates CH-live+GT-historical. | drafted (Run 161). **Thesis verified:** ClickHouse performance-first (real S3 cold-read penalty ~2000× cold vs warm-cache; S3=cold tier in OSS, hot-cold tiering is CH's own pattern; N× HA copies; *correction:* not "broken on S3" — tier-vs-native, Cloud distributed-cache closes it at $ premium). GreptimeDB S3-native (SSTs→object store at 1× shared copy, local hot cache, elastic near-stateless compute). **Cost axis (4 components):** $/GB (S3 $0.023 ≈ 3.5× cheaper than EBS gp3 $0.08) × replication (1× GT vs N× CH-OSS) × density (GT ~1.5× denser metrics+logs); compute-for-SLA (always-on CH hot vs elastic GT); cold-read GET/egress; ops (Keeper/resharding vs metasrv/Kafka). **Which for what:** ClickHouse optimizes time-to-answer on hot data; GreptimeDB optimizes $/GB on deep data. **Hybrid:** coherent (CH hot speed + GT 1×-S3 cold cost, proxy routes by age) but doubles ops + needs cross-boundary federation → **Phase-2 cost optimization, not Day-1**; start single-engine internal tiering, adopt the hybrid on sized $ numbers. Sized/tuned $ owed to server tier. |
| `open-questions-and-gaps.md` | **Gap ledger (Run 171, operator asked "what have we missed?").** Consolidates what's NOT addressed, prioritized. | drafted (Run 171). The engine comparison is settled + re-verified; the gaps are: (1) **Parallax's real workload mix** — the un-characterized deciding input the whole verdict is gated on (anchored-retrieval→GT fine vs ad-hoc-analytics→CH); needs product intent, not benchmarks; (2) server-tier benchmarks (timing/$/cold/multi-node, deferred + network-blocked); (3) Parallax's own layers (proxy/ingest, query API, evidence-bundle assembly) — above the engine, untouched; (4) cross-cutting (multi-tenancy/isolation, auth, backup/DR); (5) **managed-cloud vs self-host** — ClickHouse Cloud/GreptimeCloud change the cost/ops calculus (CH Cloud erases much of GT's object-store edge); may be most decisive after workload mix; (6) decided-but-not-designed (metadata schema, hybrid federation, production DDL); (7) full ops-complexity picture. Resolve #1 + #5 first (need operator/product intent). |
| `wal-and-durability.md` | Checklist #2 durability path: GreptimeDB WAL (raft-engine local / Kafka remote) vs ClickHouse no-WAL part-commit + fsync defaults; the Kafka-WAL scaling enabler. | drafted (pass 41, source+live Run 20: **GreptimeDB has a replayable WAL** — raft-engine local (`sync_write=false` default, tunable; live `.raftlog` 128MiB segments) or **Kafka remote → durability decoupled from datanode = the cheap-migration / compute-storage-separation enabler** behind the scaling verdict. **ClickHouse MergeTree has no WAL** (in-memory-parts WAL obsolete in 26.x); durability = part on disk, `fsync_after_insert=0`/`fsync_part_directory=0` live (not fsynced), `async_insert=1`+`wait_for_async_insert=1` live; crash = unflushed parts lost, relies on `ReplicatedMergeTree`+Keeper. Both default throughput-over-fsync; only GreptimeDB has a replay log. Durability+scaling edge GreptimeDB; not a query-speed factor; **pass 111 Run 75 advanced B15 (strict-durability cost): GT `sync_write=true` ~+1.7ms/write (~3%) vs CH `fsync_after_insert=1` ~+18ms/part (~20%) → strict-durable ingest ~10× cheaper on GT (append-log fsync ≪ part fsync)**; **pass 105 Run 69 re-verified live — GT WAL active (1.4 GB raftlog); CH `in_memory_parts_enable_wal`/`write_ahead_log_*` runtime `is_obsolete=1`, only Compact/Wide parts, no WAL files → no functional WAL confirmed**) |
| `dedup-and-update-semantics.md` | Latest-state/upsert reads: GreptimeDB read-time dedup (`merge_mode`) vs ClickHouse merge-time `ReplacingMergeTree`. | drafted (pass 39, source+measured Run 19: **GreptimeDB dedups at READ** via `DedupReader` in the scan path — `last_row` (default) / `last_non_null` (per-field partial-upsert merge) / `filter_deleted`; plain query always correct, `append_mode` opts out. **ClickHouse `ReplacingMergeTree` dedups at MERGE/`FINAL` only** — plain SELECT showed 2 dup rows until `FINAL`/`OPTIMIZE`. Latest-state queries (issue status, deploy marker, metric last-value) correct-by-default on GreptimeDB; ClickHouse needs `FINAL` (cost ∝ covering parts) or `argMax`/`AggregatingMergeTree`. Ergonomics+correctness edge GreptimeDB on upsert signals; append signals a tie; reinforces not flips verdict) |
| `schema-evolution-and-dynamic-columns.md` | Subsystem #10: how each absorbs evolving OTLP attributes — ALTER cost, schema-on-write, JSON storage. | drafted (pass 38, source+measured Run 18: both `ADD COLUMN` metadata-only (CH 5ms no part rewrite; GT flush+`RegionChange` manifest, no SST rewrite); **GreptimeDB ingest auto-adds typed columns** (`create_or_alter_tables_on_demand` — live: city/humidity/wind appeared, old rows null) while **ClickHouse rejects unknown-column inserts**; JSON storage differs — CH = per-path typed subcolumns (columnar, `attributes.k2` reads one subcolumn) vs GT = single binary blob + `json_get_*` per-row parse. Ingest-ergonomics edge GreptimeDB (zero-touch drift, risk=column explosion); dynamic-attr path-query edge ClickHouse (columnar JSON, cap=`max_dynamic_paths`); not a raw-speed flip; **pass 97 Run 61 measured the dynamic-attr path query: CH ~6 ms vs GT ~78 ms (~13×) via typed subcolumn — but CH subcolumns are `Dynamic`-typed (GROUP BY needs `.:String` cast), storage a tie (1.0 vs 1.1 MiB), and GT closes it for known hot attrs by promoting to typed columns**) |
| `retention-and-ttl.md` | Cost axis #2 lever: how old telemetry expires — whole-file drop vs row rewrite. | drafted (pass 36, source-confirmed: GreptimeDB TTL = whole-SST drop via TWCS time-windowing, no read/rewrite — `compactor.rs:581` "expired SSTs … don't depend on merge success"; ClickHouse default `ttl_only_drop_parts=false` → **row-level** TTL merge rewrites surviving rows (`merge_with_ttl_timeout`=4h), cheap whole-part drop needs `PARTITION BY` time + `ttl_only_drop_parts=1`; cheap-by-default GreptimeDB vs cheap-if-configured ClickHouse; applied DDL correction to clickhouse-implementation.md; **pass 37 Run 17 measured it**: CH `part_log` default TTL=`TTLDeleteMerge` read 1M/rewrote 500k survivors (50 MiB) vs tuned `TTLDropMerge` 0 rewritten; GT `ttl=5s` 1 SST→0 after compact (no rewrite file); refinements: `merge_with_ttl_timeout`=4h is a repeat floor not initial delay (CH evicted in seconds), GT filters TTL at read+flush+compaction; write-amp magnitude at volume owed to harness; **pass 100 Run 64 re-verified + refined: CH wholly-expired part → `TTLDropMerge` 0-rewrite (cheap even at default), mixed part → `TTLDeleteMerge` read 1M/write 500k (rewrite confirmed); GT read-time TTL filter → 0 live rows immediately. Rewrite penalty is boundary-parts-only; GT cheap-by-default via TWCS**) |
| `compression-and-cost.md` | Layout, codecs, compression by signal, full-text index cost, retention-cost consequence. | drafted (pass 8: measured per-table/per-column sizes — NO blanket winner, per-column-pattern; ClickHouse wins tuned counter/gauge/high-card-string, GreptimeDB wins dict-friendly + noisy-float; cost ~tie; object-store MinIO run + realistic-cardinality redo pending; **passes 88–89 Runs 51–52: full-text INDEX storage cost** — index *family* is the axis, identical on both engines: inverted (GreptimeDB tantivy 148 MiB / ~6 ms vs ClickHouse `text` 170 MiB / 3 ms, ~13% apart, both 60–75% overhead) vs **bloom = TIE** (CH `tokenbf_v1` 19 MiB / 8 ms vs GreptimeDB bloom 18 MiB / 9 ms at matched 1% fpr). Corrected own Run 51 over-claim: the "~9×" was bloom-vs-inverted, and a first CH tokenbf 57 MiB was a 3× oversizing artifact — at fair fpr the bloom tier ties (it's fpr math). Bloom-vs-inverted (~9× smaller, ~2–3× slower) is the real lever, **engine-neutral**; GreptimeDB's only edge is ergonomics (one `FULLTEXT WITH(backend=)` knob vs CH `text`/`tokenbf` split); pass 15 Run 6 (B2): GreptimeDB trace_id INVERTED INDEX cut lookup 14→8 ms but not to ClickHouse's 2 ms — residual is fixed query-setup floor, re-test at scale + native protocol); pass 16 Run 7 (B9): self-correction — ClickHouse part-explosion is a sustained-rate failure not per-insert (300 inserts→1 active via merges, guard=3000), GreptimeDB write-path edge real but narrower); pass 17 Run 8 (B10 partial): GreptimeDB-S3 on MinIO = 1M spans in 36 MiB / 4 objects (object-store-native confirmed, request-efficient); ClickHouse-S3 + request counts owed); pass 18 Run 9 (B10 done): same MinIO 1M spans = GreptimeDB 4 objects vs ClickHouse 74 (~18× fewer → request-efficient), measured object-store cost edge for GreptimeDB); pass 19 Run 10 (B7): realistic 99%-unique log text — GreptimeDB 25M vs ClickHouse 35.5M at defaults but 24.24M with ZSTD-all → tie at matched effort, GreptimeDB wins out-of-the-box (ClickHouse default LZ4 on high-card ids)); pass 20 Run 11 (B5): 40k-series/8M-row metric aggregation = ClickHouse 65ms vs GreptimeDB 638ms (~10×) — **CORRECTED pass 62 Run 37: warm steady-state is CH 50ms vs GT 107ms (~2×); the 638ms/~10× was a cold/first-run GreptimeDB scan, not the warm gap; ~2× fits the exec-engine mechanism (pass 42) — ClickHouse still wins SQL agg but ~2× warm not 10×, cold-regime larger**; GreptimeDB metrics edge = PromQL capability NOT agg speed); pass 21 Run 12 (B1 flip-trigger, 5M logs both indexed): full-text search CH 7ms vs GT 130ms (~18×), selective keyed filter a tie (4 vs 5ms) — log-search-at-volume strongly favors ClickHouse; verdict holds conditional on anchored-retrieval workload); pass 26 Run 13 (B8 concurrent): both pass ≤2× penalty gate (CH 1.55×, GreptimeDB 1.38×) — neither blocks reads on ingest; absolute agg at 11M still ~5× ClickHouse); Runs 16–30 (passes 35–53): Q6 composite (CH 10ms/GT 33ms, not latency-bound), retention write-amp (Run 17), schema-evolution (Run 18), dedup (Run 19), durability defaults (Run 20), exec-engine knobs (Run 21), index file formats (Run 22), PromQL drift (Runs 23–24), metric-cardinality (Run 26), span-tree recursion (Run 27), projections (Run 28), deletes/mutations (Run 29), **Q4 cross-tier join (Run 30; CORRECTED Run 81: CH 4ms / GT 54ms ~13× — GT does NOT push the left-side trace_id filter through the LEFT JOIN, full-scans 1M (EXPLAIN output_rows:1M); a pushdown-into-join optimizer gap, NOT the HTTP/repartition artifact Run 30 claimed; fixable by subquery pre-filter ~21ms or app-side correlation)**; Run 31 Q5 high-card filter (unindexed span_id full-scan: CH 10ms/GT 95ms ~10× — vectorized engine wins scans; indexed=anchored tie) → **Q1–Q6 smoke set complete**) |
| `distributed-and-scaling.md` | Single-node ceiling and horizontal-scale design of each. | drafted (pass 10: ClickHouse wins vertical single-node ceiling; GreptimeDB wins horizontal — region model + Metasrv rebalance + repartition + compute/storage separation vs ClickHouse OSS manual sharding (SharedMergeTree is Cloud-only); arch-reasoned, multi-node run owed; pass 34: region-migration mechanism confirmed in source — flush→downgrade→open_candidate→upgrade→close, no bulk-copy step = ownership reassignment + reopen-from-storage, cheap when object-store-backed); pass 57 Run 34: replication storage economics — OSS ClickHouse `ReplicatedMergeTree` stores N full S3 copies for N replicas (zero-copy replication `allow_remote_fs_zero_copy_replication=0` + source "Don't use … not ready"/EXPERIMENTAL; `SharedMergeTree` Cloud-only), vs GreptimeDB object-store-native = 1 shared S3 copy per region (HA via leadership/metadata, not data copy) → cheaper+simpler HA at scale; **pass 110 Run 74 runtime-confirmed the OSS-scale-out-is-manual claims: `SharedMergeTree`→UNKNOWN_STORAGE (Cloud-only), `ReplicatedMergeTree`→NO_ZOOKEEPER (needs Keeper), zero-copy replication default 0; multi-node hold still harness-gated**) |
| `greptimedb-implementation.md` | Concrete Parallax-on-GreptimeDB design: full schema, ingest path, exact retrieval queries, object-storage/retention layout. | drafted (pass 12: full buildable DDL for all 8 signals — trace_id INVERTED INDEX (Run-1 fix), append_mode, FULLTEXT on message, metric engine + PromQL, JSON attrs, ttl/object-store; Q1–Q6 in dialect; standalone→cluster same schema. DDL syntax source-verified; **pass 94 Run 57 added the live native-vs-custom decision** — **pass 118 Run 85 DDL-ergonomics: GreptimeDB reserves ~28/42 common observability column names (value/timestamp/user/status/level/message/service/name/id/type/source/target/event/action/result/…) vs ClickHouse 1/28 — blueprint rule: quote every GT column identifier; small CH DDL-authoring edge, not a blocker**; native auto-schemas verified live (metrics Influx→tags-as-PK+auto-typed+last_non_null = ADOPT; logs identity-pipeline→all-STRING+append, no anchor index = ADOPT-then-add trace_id/message index; **pass 119 Run 86 CLOSED native traces: hand-built OTLP protobuf → `opentelemetry_traces` auto-created (via `greptime_trace_v1` pipeline) — `trace_id`/`service_name` bloom-indexed, PK `service_name`, PARTITION ON COLUMNS(`trace_id`) 16-way, full OTLP fields + JSON events/links → ADOPT native for traces; the trace_id partitioning is also an anchor-locality lever without PK-cardinality cost (refines Run 63/65)**). ClickHouse has no zero-DDL native path (collector-mediated) — GreptimeDB onboarding-ergonomics edge) |
| `clickhouse-implementation.md` | Concrete Parallax-on-ClickHouse design: full schema, ingest path, exact retrieval queries, object-storage/retention layout. | drafted (pass 13: full buildable DDL for all 8 signals — ORDER BY keys + per-column codecs (Gorilla/DoubleDelta/LowCardinality), native text index + bloom_filter for trace_id, JSON attrs, AggregatingMergeTree+MV for metrics, S3-disk TTL tiering; Q1–Q6; replaceability cost = OTLP collector + PromQL→SQL layer + manual sharding. async_insert/JSON/text-index source-verified) |
| `per-signal-verdict.md` | Scenario matrix: metrics vs logs vs traces vs evidence-bundle correlation. | drafted (pass 7: full matrix synthesizing passes 1-6 — ClickHouse leads logs/traces/anchored-bundle latency, GreptimeDB wins metrics/PromQL capability + ties metric agg; cost/scaling cells open; honest read = hypothesis not holding on raw latency, GreptimeDB's edge is metrics-native + object-store fit; **pass 93 Run 56 re-verified Q6 composite — no drift (CH ~10 ms / GT ~30 ms vs Run 16's 10/33), both ≪300 ms gate; gap isolated to Q1 trace_id retrieval floor, correlation/assembly is a tie**) |
| `benchmarking-the-differences.md` | Per-difference targeted benchmark design (hypothesis, workload, metric, pass/fail, prerequisites); routes runnable cases into the benchmark prototype. | drafted (pass 14: 11 targeted cases B1–B11 from all prior findings, prioritized; B2 trace-id-index runnable now, B1 cold GB–TB scan = the verdict flip-trigger; harness-gap list routed to the prototype; grown to **B1–B15** — B12 JSONBench 1B cold, B13 high-card metric storage (pass 48), **B14 multi-replica S3 cost / zero-copy gap + B15 strict-durability throughput (pass 59)** — the latter two complete precise harness-handoff specs for the verdict's open questions #6/#7) |
| `local-benchmark-results.md` | Empirical log of local Docker runs: env, pinned image tags, dataset, queries, measured numbers, and which published claim each run confirms or refutes. | drafted (pass 4 Run 1: spans smoke, parity PASS, trace-lookup schema asymmetry; pass 5 Run 2: evidence-bundle Q1/Q4 join parity PASS + EXPLAIN plans confirm PREWHERE/granule-skip + partitioned-hash + anchor-constant pushdown on both → join algo not a differentiator for anchored queries; pass 6 Run 3: metrics — PromQL-native on GreptimeDB vs absent on ClickHouse (capability gap), metric agg within 1.3× (16 vs 12 ms), float compression redo pending; bigger/cold tiers pending) |
| `public-performance-claims.md` | Method-#4 deliverable: public benchmark claims (ClickBench, JSONBench, vendor + independent) gathered and rated against code + local runs. | drafted (pass 22: claims triangulate with local runs — CH wins hot ingest/agg/log-search, GreptimeDB object-store-native + PromQL; KEY counterpoint = GreptimeDB #1 on ClickHouse's JSONBench cold-run at 1B docs, the regime closest to Parallax's cold re-reads — vendor-reported, to reproduce. **Re-verified pass 47** against Runs 1-25 + web sweep: only drift = claim #7 "PromQL absent in ClickHouse" corrected (now early-stage/limited to rate/delta/increase, triangulated incl. Greptime's own comparison page); OTLP no-drift; #1-6,#8-9 hold; JSONBench cold-run still un-reproduced (blog dates 2025-03). **Pass 55 Run 32 closed claim #7's last sub-claim**: GreptimeDB native GA Jaeger query API live (`/v1/jaeger/api/services` 200, `http/jaeger.rs` services/operations/traces+tag-search) vs ClickHouse external jaeger-clickhouse plugin — all 3 GreptimeDB protocols (OTLP/PromQL/Jaeger) now verified native-GA, ClickHouse none native) |
| `vendor-claims-audit.md` | Audit of GreptimeDB's own marketing/comparison pages (`compare/click_house` + 15 blogs) vs our 105-run findings, per the operator's "decide on tech not marketing" ask. | drafted (Run 106): verdict — the compare page sells GT on fit/storage/economics/native-protocols and **never claims raw-analytical-speed superiority** (the one claim our data refutes), so it did NOT manipulate the decision; log-monitoring blog *concedes* CH faster on keyword search; ingestion benchmark independently confirms cardinality-insensitivity on v1.0 GA. Misleading-but-peripheral: Poizon "seconds→ms" (GT-vs-ETL not GT-vs-CH), structured-keyword "GT faster" (unstated config = the matches()/bloom artifact), TSBS "67×" (vs row-store Postgres, agg-stacked). Live-verified RC2 "100× TopK" dynamic-filter pushdown on our v1.0.2 (GT ~20 ms / CH ~7 ms). Corrections: disk index-file cache exists *in addition to* in-memory caches; OTel-Arrow is Phase-2/experimental not GA; JSON Type v2 (v1.1/Q2 2026) narrows the Run-104 attr gap. Net: reinforces "GT on fit not speed," no flip |
| `otel-arrow-ingest-assessment.md` | Assessment of OTel-Arrow (OTAP) ingest after the operator flagged it as a possible "huge benefit." | drafted: the benefit is **transport-only** (~30% bandwidth traces / 50–70% logs-metrics vs OTLP+zstd, the "10×" is vs uncompressed) and costs +5–43% CPU; **experimental on every layer** (OTel collector beta + not in OTLP spec + needs a collector hop since SDKs don't emit it; GreptimeDB "initial support"; ClickHouse no native OTAP). The Arrow-on-wire→Arrow-in-engine zero-copy story that would favour GT structurally is **aspirational, not shipping** (both engines re-encode on ingest today). **Self-hosted killer caveat:** co-located collector↔DB evaporates the network-egress savings. Verdict: **not a near-term GT-vs-CH factor**, a future scaling-phase + direction-alignment signal; track GT Phase-2 zero-copy DataFusion ingest. Use plain GA OTLP today |
| `four-way-version-comparison.md` | Operator-requested consolidated matrix: every load-bearing query × 4 builds (GT v1.0.2, GT v1.1.0-nightly, CH 26.5.1.882, CH 26.6.1.127-head), identical data, reproducible through `bench/four-way/`. | drafted and current through Runs 140-143: 20 queries at 1M (all interactive; CH faster on most; GT wins/ties last-value, selective full-text, high-card exact distinct), plus 5M scale section showing the split: GT anchored/keyed hot path holds, but heavy analytics cross the 300 ms gate and v1.1-nightly regresses the dedup aggregation path; Run 143 adds local-small/server-large policy and compaction nuance. Re-test pending: GT v1.1 GA for JSON Type v2 and dedup-path regression. |
| `verdict-which-to-choose.md` | Final synthesized decision and the mechanism-level reasoning. | drafted, sharpened through **pass 40** (recommends **GreptimeDB on FIT not speed** — hypothesis "fastest" refuted (ClickHouse faster on log/trace latency), GreptimeDB chosen for metrics-native + ingest/freshness/upsert ergonomics + retention cost + horizontal-scaling + object-store + Rust; both replaceability answers + flip-trigger + benchmark veto questions. Pass 40 folded in: Q6 composite measured NOT latency-bound (Run 16) anchoring "fit not speed" on the dominant query; read-time dedup (Run 19); schema-on-write auto-columns (Run 18); whole-SST retention drop (Run 17); region-migration no-bulk-copy mechanism. ClickHouse edges added: dynamic-attr columnar-JSON path queries. **Pass 52 folded in passes 41-51**: durability/WAL (GT replayable WAL+Kafka decouple), exec-engine (CH 65k-block+JIT throughput), high-card storage (GT metric engine vs CH LowCardinality 8192 cliff), corrections (GT upsert vs CH rewrite; DELETE parity), projections (CH multi-ordering scans), + the **trajectory note**: ClickHouse closing observability gaps (PromQL/lightweight delete+update) but experimentally/setup-gated while GreptimeDB's are GA — replaceability cost trending down, present state still favors GreptimeDB. **Pass 58**: folded Q4/Q5 (bundle complete), Jaeger (GA-native observability trio OTLP+PromQL+Jaeger vs CH assembled), async-insert, zero-copy replication economics (1× vs N× S3); **declared the white-box smoke comparison comprehensive** (all 10 subsystems + named leads + Q1–Q6 + 9 claims) with remaining open questions crystallized as harness-gated (#0–#8: 1B-doc cold, GB-scale cold latency, multi-node hold, multi-replica S3 $, strict-durability cost, sized high-card storage). **Pass 103 folded in Runs 55–67:** two added ClickHouse edges — cold *selective* object-store reads (scatter-vs-cluster, Runs 55/63) + dynamic-attr path queries ~13× (Run 61) — metric-agg refined to ~2–3× warm (Run 67); offsetting GT wins re-confirmed (full-text cost tie, non-blocking concurrency, object-count, Q6, native ingest, upsert/DELETE, PromQL GA, cheap retention); recommendation unchanged. **Pass 96 added DQ6 — the long-term *investment* decision**: distinct from "fits today," it asks which engine to bet on for years given the operator invests in Rust not C++. Finds CH's speed lead is a **closable engineering gap, not a physics wall** (per the roadmap's per-gap test), GreptimeDB is the **Rust substrate the operator can contribute to** (asymmetric lever — AI writes Rust > C++, Bun Zig→Rust precedent), the **design trajectory** is domain-native (Postgres-overtook-MySQL shape), and the honest risk is stated (the bet depends on sustained contribution). Investment verdict **reinforces** the fit choice: same answer, now also "the gap is closable, by us, in the language we'll invest in") |
| `greptimedb-parity-roadmap.md` | The dedicated, detailed **what/why/how** doc for improving the recommended engine: per ClickHouse advantage, the borrowed concept → code-oriented change against GreptimeDB's real internals → effort tier → **Parallax user story + "does it make GreptimeDB the clear winner?"** verdict. | drafted + sharpened through **pass 87**. 7 improvements (full-text usage/schema guard, scan/agg engine batch+JIT, PREWHERE, JSON shredding, projections, per-column codecs, maturity); **#1–5,#7 source-grounded vs `v1.0.2`** with file:line cites. Findings: gaps are **execution-integration, not architecture** (GreptimeDB's index toolkit is *richer*) → closable by engineering, mostly on the DataFusion roadmap or contributable in Rust; Tier A (schema/Flow/SQL-not-PromQL) already wins Parallax's anchored workload today. **User-first ranking (sharpened by Runs 48-49): no improvement is a must-do for Parallax's common user moments. **Pass 115 (Run 81/82) added #8** — push an equality filter into an indexed join input: GreptimeDB full-scans a join's anchored table (INNER+LEFT, `output_rows:1M`) instead of using the inverted index; Tier-A workaround = subquery pre-filter / app-side correlation, Tier-B = optimizer fix; footnote for Parallax (app-side assembly sidesteps it). **Pass 101 (Run 63/65) deepened #5** with its root cause — GreptimeDB PK = sort = series identity, so it can't cluster by a high-card anchor (`trace_id`) without series blowup while ClickHouse `ORDER BY` decouples sort from identity for free; this is *also* the root of the cold-selective-read egress loss (Run 55/63), so #5 now closes two gaps (still a footnote: persistent cache wins the warm hot path).** Even #1 (incident log search) is already competitive — selective query-syntax via tantivy+`matches()` is **~6 ms** and exact-term via bloom+`matches_term()` is **~8 ms**, not 18×. The real gap shrinks to broad-term/ad-hoc analytics. Validate the real query mix before any Tier-B engine work. **Pass 96 added the physics-wall closability test** answering the operator's investment question: a per-gap engineering/fundamental/time-only verdict finds **no gap is a physics wall** — 7/8 are engineering (same vectorized-columnar-over-Arrow model, CH just further along the same curve), #6 is time-only, #5 (PK=sort=series) is the lone design-flavoured one and is defused by `trace_id` partitioning; the two heaviest (#2 scan/agg, #4 JSON) ride **shared industry roadmaps** (DataFusion codegen/SIMD; Parquet Variant) so GreptimeDB inherits much of the work → CH's speed lead is a *depreciating asset, not a moat*. Investment synthesis in `verdict-which-to-choose.md` DQ6. |

## Source repositories (read, do not vendor into this repo)

- GreptimeDB (Rust): <https://github.com/GreptimeTeam/greptimedb>
- ClickHouse (C++): <https://github.com/ClickHouse/ClickHouse>


---

## Part B — detailed verdict synthesis with per-run citations (former verdict-which-to-choose.md, pre-restructure)

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

> **⚠ Reconsideration — the Parallax-proxy lens re-weights this toward ClickHouse (Run 153, operator
> 2026-05-25).** The headline below leans GreptimeDB largely on **ingest-nativeness** (native
> OTLP/PromQL/Jaeger, schema-on-write, small-write ergonomics). The operator has since fixed the
> architecture: **Parallax is the first layer — a proxy that implements OTLP itself, routes, and
> converts before writing to any backend.** That **neutralizes GreptimeDB's marquee advantage** (the
> DB no longer needs to speak OTLP/Jaeger/Prom natively — Parallax does, and translates). Stripping
> ingest ergonomics from the scorecard, the two axes Parallax *cannot* paper over are **retrieval
> speed** and **build-on-top ecosystem**, and **ClickHouse wins both** (and is the de-facto unified-obs
> backend: SigNoz/Uptrace/HyperDX/ClickStack). So under the proxy lens **ClickHouse becomes the
> pragmatic default**, and GreptimeDB is reserved for a specific bet (metrics-cardinality/PromQL as a
> product surface · self-hosted 1×-S3 HA economics at scale · mandatory zero-ops auto-rebalance).
> Full reasoning, the alternatives survey (OpenObserve/Quickwit/InfluxDB3/VictoriaMetrics/StarRocks —
> none beats them as an embeddable backend), and the **grouped-errors/metadata → relational store
> (Turso default / Postgres fallback, already chosen), NOT the columnar engine** split
> are in [`platform-fit-and-alternatives.md`](platform-fit-and-alternatives.md). Treat that note as the
> current top-level lean; the fit/closability analysis below remains valid *as analysis* but is
> re-weighted by the proxy.

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
| **Retention cost** | TTL = **whole-SST drop** (TWCS time-windowing → no read/rewrite; `compactor.rs:581`); ClickHouse default `ttl_only_drop_parts=0` **rewrites survivors** (Run 17: read 1M / rewrote 500k) unless tuned (`PARTITION BY` time + `ttl_only_drop_parts=1`). Cheap-by-default vs cheap-if-configured. **Refined Run 111: the gap is NARROWER than "CH always rewrites" — CH drops a *fully-expired* part cheaply (whole-part drop, verified); the rewrite cost is only the *boundary* part (expired+live mixed) or a non-time-ordered part. For time-ordered ingestion both are cheap; GT's edge is zero-config (TWCS auto-windows) vs CH cheap-when-time-partitioned (blueprint already does this). Also: GT TTL purge is eventual/background, not on-demand.** | source+measured (Runs 17, 111) |
| **Object-storage-native** | OpenDAL default + read cache; cheap re-readable retention first-class. Fewer *total* objects (4 vs 74, Runs 8–9) → wins full-scan cold reads. Cold GET cost is query-shape-dependent (measured both ways): full scan GreptimeDB fewer (26 vs 57, Run 15 — wins the JSONBench regime); **anchored lookup ClickHouse fewer (5 vs 22, Run 14)** — Parallax's pattern. Read cache → warm re-reads local on both. | measured (layout + cold GETs both shapes) |
| **Durability / crash safety** | Has a **replayable WAL** (raft-engine local, tunable `sync_write`; or **Kafka remote → durability decoupled from the datanode**, the same mechanism that makes migration cheap). ClickHouse MergeTree has **no WAL** (obsolete in 26.x) — durability = unsynced part-on-disk (`fsync_after_insert=0`) + replicas; a single-node crash loses unflushed parts. | source+live (Run 20) |
| **High-cardinality metric *ingest*** (rate + ergonomics) | Metric engine `__tsid` (label-set hash) over a shared physical wide table + PartitionTree memtable (dict-encoded label sets, **no per-series cap**) — cap-free ingest, many logical metrics → one physical table, no `ORDER BY` tuning. **Measured (Run 84): GreptimeDB ingest is cardinality-INSENSITIVE** — 1k→1M series ~flat (357→381 ms, ~1.07×) vs **ClickHouse ~2.6× slower** (`LowCardinality` overflow + more `ORDER BY` keys). The clearest GreptimeDB high-card win is the ingest axis. ClickHouse `LowCardinality` caps at 8,192 then degrades **gracefully** (still < plain `String` at 200k — Run 76, not the "cliff explosion" first framed). **⚠ Corrected (Runs 76–79): high-card *storage* is CARDINALITY-DEPENDENT (a crossover)** — ClickHouse `LowCardinality` wins low–mid (1k ~1.12×, 200k ~1.24×) but **GreptimeDB wins at ~1M unique series ~1.34×** (CH `LowCardinality` blows up to 16.51 MiB all-unique vs GT 12.36; the metric engine's `__tsid` is overhead not a saving). And **aggregation latency → ClickHouse ~2–3× warm** (Run 37/67). So GreptimeDB's high-card win is **operability/no-cap + extreme-cardinality storage**, ClickHouse's is **moderate-cardinality storage + agg speed.** | source+live (Runs 26, 76–79) |
| **Corrections (UPDATE) / upsert** | UPDATE = re-insert `(PK,ts)` → dedup last-wins = a **cheap GA upsert**, no setup; ClickHouse UPDATE = heavy `ALTER UPDATE` part rewrite (lightweight update is experimental + needs a per-table block-number column). DELETE is ~parity (both read-filtered). | source+live (Run 29) |
| Freshness | Visible-on-write (tie with ClickHouse, not a win). | smoke |

## Decision question 2 — where is ClickHouse genuinely better, and why?

| Area | Mechanism | Confidence |
| --- | --- | --- |
| **Log/trace selective scan + full-text search** | 8,192 granule + PREWHERE + inverted text index + LowCardinality + C++ SIMD vectorized pipeline. | arch+smoke (Runs 1–2) |
| **Log-explorer time-DESC tail** | `ORDER BY (service, ts)` sort-key locality serves "recent logs for a service ordered by time" as a direct tail-of-run read. **Run 107: service-tail CH ~4 ms / GT ~28 ms (~7×); errors-in-window CH ~10 ms / GT ~60 ms (~6×)** — a concrete instance of the #5 alternate-ordering gap (GT's PK=sort=series can't give time-within-service locality). **Both ≪ 300 ms (GT interactive)**; blueprint: prefer `PK(service)` over `(service,level)` for tail tables — but **Run 108 verified that is only ~10%** (~27 vs ~30 ms), a minor lever; the ~7× gap to CH is the structural #5 sort-locality, not the PK. | live (Runs 107, 108) |
| **Generic wide scan / aggregate throughput** | Decade-tuned vectorized engine — the OLAP-scan bar. Mechanism (pass 42): 65k-row blocks (8× DataFusion's batch) + LLVM-JIT expressions/aggregation + bespoke SIMD kernels + specialized hash aggregation vs GreptimeDB's DataFusion-over-Arrow; explains Runs 11–12. | arch+live (`query-execution-engine.md`) |
| **Vertical single-node ceiling** | Saturates many cores + NVMe on one big box. | arch |
| **Per-column codec tuning** | Hand-picked `DoubleDelta`/`Gorilla`/etc. (counter 7.3×, gauge 78×, Run 4). | smoke (Run 4) |
| **Dynamic-attribute path queries** | `JSON` type stores each path as a **typed columnar subcolumn** (`attributes.k` reads only that subcolumn); GreptimeDB `Json` is a binary blob + `json_get_*` per-row parse. **Measured ~13× (6 ms vs 78 ms, 100k, Run 61); re-verified Run 104 — gap WIDENED to ~57× (CH ~1 ms / GT ~57 ms @200k) as CH's 26.x new-`JSON`-type subcolumn read matured (~6→~1 ms) while GT's per-row jsonb parse is unchanged.** **Re-measured 4-way Run 129 + CORRECTED:** ClickHouse **26.6 enforces** the typed-subcolumn cast in GROUP BY (`.:Int64`; `Code 44` without it), which 26.5 allowed implicitly at ~1 ms. With the **required/idiomatic cast** CH is **~7 ms**, so the fair dynamic-attr gap is **~8× (GT ~56 ms / CH ~7 ms), not ~57×**. **RE-CORRECTED Run 168 (live on 26.5.1.882):** the `.:Type` cast in JSON GROUP BY is enforced on **26.5 too** (`Code 44` without it), **not 26.6-only** — so there is no "26.5 lax GROUP BY path." The cast-free ~1 ms / ~57× measurement was the **FILTER** path (`WHERE attrs.k=…`, cast-free on both versions), misread as a GROUP BY. GT v1.1.0-nightly shows **no change** (~56 ms). So state it as **~8× for a GROUP BY with the required `.:Type` cast (both 26.5 + 26.6); the JSON *filter* is cast-free and fast on both**, for an *unpredictable* attribute path. Two caveats unchanged: GROUP BY on CH subcolumns may need a `.:Type` cast, and GreptimeDB closes it for *known* hot attrs by promoting them to typed columns (automatic-CH vs schema-on-write GreptimeDB). Bites only on ad-hoc *undeclared* paths at volume; Parallax's anchored bundle fetch and Tier-A column-promotion sidestep it. | source+measured (Runs 18, 61, 104) |
| **Multi-ordering scans (projections)** | A **projection** stores a 2nd physical `ORDER BY` inside each part, optimizer-picked transparently → fast sequential scans on an alternate key (e.g. `service`-time *and* `trace_id`) from one table. GreptimeDB has no equivalent (indexes give positions, not a 2nd physical order). Cost: ~2× storage per normal projection (Run 28). | source+live (Run 28) |
| **Cross-tier anchored *in-DB* join** | ClickHouse pushes the anchor (`trace_id='X'`) through a `LEFT JOIN` into the scan and prunes (`Granules 1` + PREWHERE) → ~4 ms. **GreptimeDB does NOT push a left-side filter through the join** (Run 81) — `EXPLAIN ANALYZE` shows a full 1M-row `spans_idx` scan → ~54 ms (~13×); a predicate-pushdown-into-join optimizer gap. Fixable: subquery pre-filter (~21 ms) or app-side correlation (Parallax's pattern — anchored fetch each signal + join in app, avoids the in-DB join). So a *direct* in-DB cross-tier join favours ClickHouse; rewrite/app-side neutralises it (all < 300 ms gate). **Re-verified Run 103, no drift:** CH ~3 ms / GT direct ~53 ms (~17×, full-scan) / GT subquery-prefilter ~19 ms. | source+live (Runs 30, 81, 103) |
| **Cold *selective* object-store reads** (~10× with partitioning, ~80× without) | ClickHouse `ORDER BY (trace_id, ts)` clusters the anchor at **zero cardinality cost** → cold anchored read ~1 granule (**294 KiB**). GreptimeDB non-partitioned scatters `trace_id` → ~whole SST (**~23 MiB**, ~80×, Runs 55/63). **But `PARTITION ON COLUMNS(trace_id)` cuts it to ~2.8 MiB (16-way, Run 88) → ~10× gap** — a cardinality-free anchor-locality lever the native `opentelemetry_traces` ships by default; finer partitioning narrows more. GreptimeDB's persistent read cache also keeps the common warm path at ~0 S3. So the cold-selective-egress gap is real but **~10× (partitioned), not ~80×**, and only on genuinely cold/evicted reads. | measured (Runs 55, 63, 87, 88) |
| **Schema-mistake tolerance (operability)** | ClickHouse is **markedly more forgiving** of a bad schema choice. **Run 118: a wrong (high-card-first) `ORDER BY` costs only ~11% storage with NO scan/lookup penalty** (CH scans all rows regardless of sort key; no read-path dedup-merge). GreptimeDB's analogous mistake — high-card PK + default dedup on an event table — is **~16–44× slower scans on hot/memtable data** (Runs 114/117) because `PK = sort = series = dedup-unit` are coupled. The right GreptimeDB design (low-card PK + `append_mode`) is known + in the blueprint, but the margin for error is far smaller. A real ClickHouse operability edge. | live (Run 118 vs 114/117) |
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

**Condition (b) now has scale evidence (Run 141, 5M warm tier):** GreptimeDB's heavy
*analytical* queries **cross the 300 ms gate at 5M** (metric-agg 315–1021 ms, dynamic-attr JSON
330 ms, in-DB join 659 ms) and the scan gaps **widen with scale** (unindexed scan ~4×@1M →
~10×@5M) while ClickHouse stays fast (~20 ms). So an **analytics-/ad-hoc-scan-dominated mix
already favours ClickHouse at 5M** — (b) is trending confirmed; the GB–TB *cold* magnitude is the
last piece owed to the sized/server harness. **Crucially, (a) still does NOT hold for Parallax:**
the **anchored/keyed hot path stays interactive at 5M** (anchored ~14 ms, last-value ~10 ms,
time-range ~13 ms — all ≪ gate), so the dominant evidence-bundle workload keeps the GreptimeDB
recommendation. The flip is real *only if the real query mix turns out analytics-dominated* — which
the workload-mix question (the operator's to answer) decides.

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

**Live/source-verified gap-closing, not just theory — now THREE shipped examples:** (1) RC2
**"100× TopK"** — dynamic filter pushdown into the Mito scan via **DataFusion runtime dynamic
filters** — **present in our v1.0.2** (`ORDER BY … LIMIT 10` on 1M = GT ~20 ms, not a ~100 ms+ full
sort; Run 106). (2) **Flat SST** (v1.0 GA default: write ~4×, query latency up to ~10× on high-card
TSBS) — shipped scan-format redesign. (3) **`prefilter.rs` — PREWHERE-style late materialization**
(parity-roadmap #3, which was "missing" at pass-77): v1.0.2 shipped GreptimeDB's own prefilter
framework ("read filter columns first → refined row selection → read the rest"), **wired into the
Flat read path**, PK/partition-scoped so far (Run 121, source-confirmed). Three of the scan-engine
parity gaps closing in shipped GreptimeDB Rust — concrete proof the engineering path works exactly
as the thesis predicts (PREWHERE/#3 was itself "missing" at pass-77, now shipped — Runs 121/122). **Honest
caveat — gap-closing is UNEVEN (the 2026 roadmap + Run 123 temper the thesis):** GreptimeDB has shipped the
improvements it owns in its **SST/scan layer** (Flat SST, TopK dynamic-filter pushdown, the prefilter/late-
materialization), but the **raw-vectorized-throughput** gap (#2: batch size, JIT, SIMD) is **untouched in
v1.0.2** — Run 123 re-confirmed `SessionConfig` still sets no `batch_size` (8192 default) and `SET batch_size`
is still rejected, so the **~2–3× aggregation gap has not moved** and won't until GreptimeDB plumbs the batch
size or **upstream DataFusion** codegen/SIMD matures. Join-input pushdown (#8) and projections (#5) are
likewise not roadmap-committed. So: the gaps are engineering not physics, and the SST-layer ones are closing
fast, but the **execution-core throughput gap depends on DataFusion** (the part neither GreptimeDB nor the
operator fully controls) — don't extrapolate the prefilter/TopK velocity to "the agg gap closes soon." The
dynamic-attr JSON gap (#4, *widened* to ~57× at Run 104) **is** roadmap-committed: **JSON Type v2
(field-level index, dynamic fields), v1.1 / Q2 2026.**

**⚠ The v1.1 *nightly* is uneven, not uniformly better (Runs 141/142):** at 1M it's a modest broad
improvement, and it helps the join path (cross-tier 65→36 ms); but at **5M it REGRESSES the dedup-table
aggregation ~2.5×** (metric-agg 315→782 ms; append-mode unaffected — isolated to the dedup-merge path).
Since the metric engine uses dedup-like `last_non_null`, **don't assume v1.1 is a free upgrade for
metrics-at-scale — re-test on v1.1 GA.** The append-mode escape hatch (Run 142: ~8× faster dedup-agg at
scale, scrape-style metrics) sidesteps both the regression and the dedup cost regardless of version.

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

*(A soft future amplifier on this axis — **OTel-Arrow** — was assessed separately
(`otel-arrow-ingest-assessment.md`) after the operator flagged it: its Arrow-native ingest plays to
GreptimeDB's stack and could become a real GreptimeDB-fit edge **if** Phase-2 zero-copy DataFusion
ingest ships with measured numbers. But today it is **experimental on all sides, transport-only
(network egress, not query/storage), needs a mandatory collector hop, and its egress benefit largely
evaporates for a co-located self-hosted backend** — so it is **not a near-term decision factor**, only
a trajectory signal to track. Do not weight it now.)*

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
