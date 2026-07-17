# GreptimeDB vs ClickHouse — Deep Internals Comparison

<!-- markdownlint-disable MD013 -->

White-box, source-level comparison of **GreptimeDB** and **ClickHouse** for the Parallax storage
layer: how each works internally, which design decisions make each fast or slow, and — for
Parallax's signals (metrics, logs, traces, anchored evidence-bundle correlation)
— what risks and upstream opportunities the comparison exposes.

> **Current authority:** GreptimeDB + Turso are mandatory; ClickHouse is a
> comparator only, never a fallback. The historical study selected GreptimeDB on
> fit + cost + Rust while measuring ClickHouse as the faster analytical engine.
> Read the one-page
> [`verdict-which-to-choose.md`](verdict-which-to-choose.md); the product decision is in
> [`../../decisions/storage-engine.md`](../../decisions/storage-engine.md); the full run-by-run
> history and detailed synthesis are in [`run-log.md`](run-log.md).

This sub-study is driven by the loop brief
[`prompts/greptimedb-vs-clickhouse-internals.md`](../../../../prompts/greptimedb-vs-clickhouse-internals.md).

## How this fits with the rest of storage research

This is the **white-box** layer — the *why* behind the *what* the other notes establish:

- [`../evaluation.md`](../evaluation.md) — strategy/fit evaluation (reasons *about* the systems).
- [`../benchmark-plan.md`](../benchmark-plan.md) — benchmark plan + runnable black-box harness (qualifies claims and risks; it cannot change the stack).
- [`../size-and-object-cost.md`](../size-and-object-cost.md) and [`../freshness-and-latency.md`](../freshness-and-latency.md) — the cost and latency proof gates.

A benchmark number the internals cannot explain is a flag that one of them is wrong.

## Version pins (re-check and bump every pass)

| System | Pinned version | Source commit | Notes |
| --- | --- | --- | --- |
| GreptimeDB stable | **`v1.1.3`** (GA 2026-07-17) | `63ef18a74a640135b983db6332226f90f9ae2b24` | **Run 173 bump** from `v1.0.2`. Do **not** pin `v1.1.0` alone — critical JSON upgrade bug; use ≥`v1.1.1`. |
| GreptimeDB nightly | **`v1.2.0-nightly-20260713`** | `c12f40cec232dda23429a0995d70bb4a230a562c` (reports `1.2.0`) | Rolls; re-bump dated tag each pass. |
| ClickHouse stable | **`v26.6.1.1193-stable`** | `840482cdca4e574927c1853900043b81d0687d00` | **Run 173 bump** from `v26.5.1.882`. Latest **feature** line (not LTS). `v26.5.5.8-stable` is a newer *patch* of the older 26.5 line. |
| ClickHouse nightly | **`clickhouse/clickhouse-server:head`** | reports **`26.7.1.1097`** | Rolls daily. |

*Prior pins preserved in run history:* GT `v1.0.2` / CH `v26.5.1.882` through Run 172.

## Recent loop status (Runs 173–200, 2026-07-17)

Re-pinned to **v1.1.3 / 26.6** and re-verified live (not “done” — server-tier + workload mix +
managed quotes remain):

| Theme | Run | Status |
| --- | --- | --- |
| JSON2 closes most dynamic-attr gap | 173/176 | harness + note |
| Backup/DR | 174 | new note |
| Managed-cloud framework | 175 | new note; $ quotes owed |
| Last-value ~tie at 100k | 177 | scale-shaped GT win |
| Concurrent ingest ≤1.5× | 178 | gate pass |
| Quotas / fsync settings | 179–180 | CH Code 201 live |
| Native OTLP/PromQL/Jaeger | 181 | protobuf-only OTLP |
| Schema-on-write identity | 182 | no drift |
| PromQL tax ~1.5–2× @100k | 183 | not fixed 5.6× |
| Full-text / matrix refresh | 184 | selective ~tie |
| CH projections | 185 | 2/12 granules |
| append_mode forbids DELETE | 186 | blueprint |
| TTL expire | 187 | no drift |
| Flow + CH MV | 188 | capability parity |
| Storage density | 189 | shape-dependent |
| Verdict pin refresh | 190 | one-pager |
| Freshness 20/20 | 191 | tie |
| Cardinality-insensitive ingest | 192 | flat |
| Join pushdown | 193 | CH prunes; both interactive |
| MinIO S3 object layout re-verify (pins bumped) | 220 | GT 3 / CH 22 objs @100k |
| Managed cloud primary $ rates | 221 | CH transparent; GT $290 floor opaque |
| Product RPO/RTO runbook (D1–D3) | 222 | `product-rpo-runbook.md` |
| Workload-mix decision input (DQ5) | 223 | `workload-mix-decision-input.md` |
| Four-way N=50k + native OTEL; pins hold | 224 | no direction drift |
| Live D1 COPY/BACKUP restore drill | 225 | 50k full match both engines |
| Concurrent ingest + anchored query | 226 | ~1.5× GT / ~1.2× CH; no drift |
| PARTITION ON(trace_id) file_ranges | 227 | 1/2 regions; 1 vs 2 ranges |
| Flow + CH MV continuous agg | 228 | parity; Flow async lag |
| Freshness + append_mode DELETE | 229 | no drift |
| TWCS multi-window SST prune | 230 | files 2→1 on time filter |
| CH PREWHERE plan shape | 231 | 1/6 granules; ms ~tie @50k |
| CH projection p_svc non-PK filter | 232 | ReadFromMergeTree(p_svc) 1/7 |
| Gap ranking after 220–232 | 233 | server/quotes/mix still top |
| GT OpenDAL /metrics for S3 reads | 234 | opendal_operation_bytes |
| Live S3 cold GetObject deltas | 235 | GT +5 vs CH +3 @20k |
| CH head TimeSeries SELECT | 236 | outer Code 48 (facade) |
| CH TimeSeries INSERT+prometheusQuery real | **403** | drift: query path works 26.6+26.7 |
| CH PromQL rate/sum OK; increase missing | **404** | matches GT on rate; Code 48 increase |
| Managed $ list hold + D2 CLI limit | **405** | rates no drift; standalone meta snap fail |
| adopt-native OTEL re-verify | **406** | protobuf OTLP; identity OK; no drift |
| native tables on-ingest only | **407** | no pre-DDL opentelemetry_* |
| gap ranking after 403–407 | **408** | not done; top 5 still open |
| D3 dump/restore pattern | **409** | SQLite stand-in ROW_MATCH |
| re-pin + PREWHERE 1/6 | **410** | pins hold; service PK prune |
| increase flag no help | **411** | still Code 48 on head 26.7 |
| last_value vs argMax m2m | **412** | both live; GT +20 dirty rows |
| freshness 1→2 | **413** | memtable/part visible; no drift |
| append DELETE ban | **414** | GT Code 1004; CH lw DELETE OK |
| Flow/MV still live | **415** | r228 flows; CH SummingMV |
| ranking 403–415 | **416** | not done; top 5 open |
| TWCS + partition hold | **417** | twcs option + 2 files live |
| CH impl PromQL stale fix | **418** | TimeSeries facade; no product path |
| impl pin headers | **419** | v1.1.3 / 26.6 current |
| concurrent query ms | **420** | under insert still interactive |
| still not done | **421** | top 5 gaps open |
| bloom trace_id 2/7 | **422** | idx_trace prunes; data dirty |
| PromQL fn matrix expand | **423** | CH partial; *_over_time mostly missing |
| join prune N=2k fresh | **424** | CH both sides; GT PK ok |
| four-way last_value warm | **425** | GT 10–16ms / CH 4–11ms |
| JSON path micro N=200 | **426** | JSON ok; JSONB type absent; JSON2 INSERT caveats |
| re-pin still current | **427** | not done; top 5 open |
| FT selective vs broad | **428** | CH tokenbf 0/7 rare; 7/7 broad |
| metric-agg four-way warm | **429** | GT 7–9ms / CH 2–7ms |
| TTL expire re-verify | **430** | GT 0 after compact; CH 2→1 |
| SQL agg tax spot | **431** | GT 8–11ms / CH 3–8ms warm |
| CH projection p_svc both | **432** | 26.6+head ReadFromMergeTree(p_svc) |
| ranking after 423–432 | **433** | not done; top 5 open |
| adopt-native smoke | **434** | Jaeger/identity/OTLP no drift |
| append DELETE + fresh | **435** | Code 1004; CH 2→1; 1→2 |
| Flow still async lag | **436** | r228 flows; sink 3 vs src 4 |
| still not done | **437** | top 5 open |
| TWCS + PREWHERE hold | **438** | twcs option; PREWHERE 2/7 |
| PromQL gaps still open | **439** | increase/min_over_time Code 48 |
| still not done | **440** | top 5 open |
| CH density snapshot | **441** | logs 23B/row; m2m 4.5 |
| health + spans match | **442** | 4167 both engines |
| still not done | **443** | top 5 open |
| anchored N=2k prune | **444** | GT 3–5ms / CH 2–4ms |
| still not done | **445** | top 5 open |
| CH ACCESS surface | **446** | quotas/profiles still present |
| re-pin hold | **447** | no newer nightly |
| count-distinct spot | **448** | exact 50k; approx_distinct OK |
| still not done | **449** | top 5 open |
| row_number top-k | **450** | GT 21ms / CH 5ms warm |
| still not done | **451** | top 5 open |
| p50/p99 panel | **452** | GT 13ms / CH 3ms warm |
| still not done | **453** | top 5 open |
| health pins adopt | **454** | 2h up; Prom/Jaeger 200 |
| still not done | **455** | top 5 open |
| export-v2 schema-only | **456** | public.sql 508 lines |
| still not done | **457** | top 5 open |
| join prune recheck | **458** | PK+bloom both sides |
| still not done | **459** | top 5 open |
| TimeSeries rate hold | **460** | sum(rate)=1.5; increase no |
| still not done | **461** | top 5 open |
| time-range aged data | **462** | 0 rows; ~3ms both |
| still not done | **463** | top 5 open |
| freshness 1→2 | **464** | both engines no drift |
| still not done | **465** | top 5 open |
| append DELETE + health | **466** | Code 1004; 2h healthy |
| still not done | **467** | top 5 open |
| last_value @2h uptime | **468** | GT 10–12ms / CH 4–5ms |
| still not done | **469** | top 5 open |
| adopt-native + pins | **470** | identity OK; pins hold |
| still not done | **471** | top 5 open |
| projection hold @2h | **472** | p_svc both builds |
| still not done | **473** | top 5 open |
| Flow/MV + pin hold | **474** | r228 flows; v1.1.3 |
| still not done | **475** | top 5 open |
| metric-agg warm spot | **476** | GT 8–9ms / CH 3ms |
| still not done | **477** | top 5 open |
| FT prune shape hold | **478** | 0/7 rare; 7/7 broad |
| still not done | **479** | top 5 open |
| increase still missing | **480** | Code 48; pins hold |
| still not done | **481** | top 5 open |
| anchored + healthy | **482** | PK 1/1; 4 containers |
| still not done | **483** | top 5 open |
| TTL empty hold | **484** | 0 rows both; ttl=1s |
| still not done | **485** | top 5 open |
| health+pins+adopt | **486** | all healthy; Prom/Jaeger 200 |
| still not done | **487** | top 5 open |
| TimeSeries topk/last | **488** | still OK both builds |
| still not done | **489** | top 5 open |
| count-distinct recheck | **490** | GT 10ms / CH ~9–35ms |
| still not done | **491** | top 5 open |
| row_number recheck | **492** | GT 29–38ms / CH 6–16ms |
| still not done | **493** | top 5 open |
| p99 panel recheck | **494** | GT 14–18ms / CH 4–5ms |
| still not done | **495** | top 5 open |
| adopt+export+pins | **496** | Jaeger/Prom/export OK |
| greptime_identity schema-on-write | 237 | auto columns; no drift |
| Cold S3 measure recipe in cache note | 238 | cache wipe + OpenDAL |
| last_value vs argMax @50k | 239 | GT ~5ms / CH ~3ms |
| Server-tier runbook (1M/5M/GB) | 240 | `server-tier-runbook.md` |
| PromQL vs SQL tax small | 241 | ~1.5×; path OK |
| JSON2 vs Jsonb @50k | 243 | 3 vs 18 vs CH 2 ms |
| re-pin still current | 244 | no bump |
| Gap ranking 220–245 | 246 | server/GB/mix/quotes top |
| Session milestone 220–249 | 250 | not done; server/GB open |

## Remaining execute work (Run 403+)

Laptop smoke mostly saturated; **Run 403** closed a false "TimeSeries unusable" reading.
Highest remaining:

1. Workload mix A1–A7 fill (`workload-mix-decision-input.md`)
2. Server 1M/5M (`server-tier-runbook.md`)
3. Vendor trial quotes (`managed-cloud-vs-self-host.md`)
4. GB cold S3 (`caching-and-cold-warm.md` recipe)
5. RPO D2/D3 (`product-rpo-runbook.md`)
6. Optional: CH TimeSeries PromQL **completeness at volume** (comparator watch only)

## Method

- Compare the latest stable release of each system; record exact versions and the source commit SHA in every note.
- Orient on architecture docs, then confirm load-bearing claims against the cloned source (GreptimeDB Rust, ClickHouse C++); cite file:line. When docs and code disagree, trust the code.
- Every "X is faster" claim carries a *because* (mechanism) and a *scenario* (signal, query shape, cardinality, cache state, single-node vs scaled).
- Benchmarks run on all four builds (GT stable+nightly, CH stable+nightly) and update [`four-way-version-comparison.md`](four-way-version-comparison.md).

## Evaluation axes (priority order)

1. **Speed** — ingest-to-queryable freshness and evidence-bundle/correlation latency under concurrent ingest+query.
2. **Cost** — retained size/compression by signal, object-vs-local economics, compute per GB and per query class.
3. **Scaling** — single-node ceiling and horizontal scale-out (horizontal first; vertical-only is a flagged limitation).

## Note index (the evidence layer)

**Verdict and history**
- [`verdict-which-to-choose.md`](verdict-which-to-choose.md) — one-page current verdict (DQ1–DQ6 + flip rule).
- [`run-log.md`](run-log.md) — run-by-run status timeline, per-note status, and detailed verdict synthesis.
- [`open-questions-and-gaps.md`](open-questions-and-gaps.md) — gap ledger: what is NOT yet addressed, prioritized.

**Mechanism teardowns**
- [`greptimedb-internals.md`](greptimedb-internals.md) / [`clickhouse-internals.md`](clickhouse-internals.md) — architecture + code-path teardown of each engine.
- [`write-path-and-ingestion.md`](write-path-and-ingestion.md) — ingest → durable → queryable, and the freshness consequence.
- [`read-path-indexing-and-execution.md`](read-path-indexing-and-execution.md) — query planning, indexing, execution, scan-vs-skip, joins.
- [`query-execution-engine.md`](query-execution-engine.md) — CH C++ vectorized pipeline vs GT DataFusion-over-Arrow (the throughput gap).
- [`indexing-internals.md`](indexing-internals.md) — index file formats (GT Puffin sidecar vs CH per-part `.idx`).
- [`compaction-and-merge.md`](compaction-and-merge.md) — TWCS vs size-tiered merge; write amplification.
- [`caching-and-cold-warm.md`](caching-and-cold-warm.md) — cache hierarchies and the cold-vs-warm divergence.
- [`wal-and-durability.md`](wal-and-durability.md) — GT WAL (raft-engine/Kafka) vs CH no-WAL part-commit.
- [`dedup-and-update-semantics.md`](dedup-and-update-semantics.md) — read-time dedup vs `ReplacingMergeTree`.
- [`deletes-and-mutations.md`](deletes-and-mutations.md) — corrections / GDPR-erase / updates.
- [`schema-evolution-and-dynamic-columns.md`](schema-evolution-and-dynamic-columns.md) — OTLP attribute drift, ALTER cost, JSON storage.
- [`retention-and-ttl.md`](retention-and-ttl.md) — whole-file drop vs row rewrite.
- [`projections-and-access-paths.md`](projections-and-access-paths.md) — CH projections vs GT secondary indexes.
- [`metric-cardinality.md`](metric-cardinality.md) — high-cardinality metric storage and ingest.
- [`promql-and-metrics-query.md`](promql-and-metrics-query.md) — PromQL planning paths; the "no PromQL" drift correction.
- [`trace-span-tree.md`](trace-span-tree.md) — span-tree reconstruction (flat fetch vs recursive CTE).
- [`rollup-and-continuous-aggregation.md`](rollup-and-continuous-aggregation.md) — GT Flow vs CH MV + AggregatingMergeTree.
- [`compression-and-cost.md`](compression-and-cost.md) — layout, codecs, compression by signal, index cost.
- [`distributed-and-scaling.md`](distributed-and-scaling.md) — single-node ceiling and horizontal-scale design.
- [`storage-cost-and-tiering.md`](storage-cost-and-tiering.md) — CH performance/local-first vs GT S3-native/cost-first; hot/cold hybrid.
- [`multi-tenancy-and-isolation.md`](multi-tenancy-and-isolation.md) — tenant isolation, RBAC, row policies, quotas, and proxy-owned auth.
- [`backup-and-disaster-recovery.md`](backup-and-disaster-recovery.md) — engine backup/export/restore surfaces (GT COPY+cli meta/data vs CH BACKUP/RESTORE).
- [`product-rpo-runbook.md`](product-rpo-runbook.md) — product RPO/RTO: D1 telemetry / D2 meta / D3 Turso, cadence, restore order (Run 222).
- [`workload-mix-decision-input.md`](workload-mix-decision-input.md) — DQ5 flip-rule mix model: A1–A7 classes, rubric, how to measure (Run 223).
- [`managed-cloud-vs-self-host.md`](managed-cloud-vs-self-host.md) — managed Cloud vs self-host cost/ops calculus (SharedMergeTree + GT managed).

**Per-signal, benchmarks, and public claims**
- [`per-signal-verdict.md`](per-signal-verdict.md) — scenario matrix: metrics vs logs vs traces vs evidence-bundle correlation.
- [`benchmarking-the-differences.md`](benchmarking-the-differences.md) — per-difference targeted benchmark design (B1–B15).
- [`local-benchmark-results.md`](local-benchmark-results.md) — empirical log of local Docker runs (env, pins, numbers).
- [`four-way-version-comparison.md`](four-way-version-comparison.md) — consolidated matrix: every load-bearing query × 4 builds.
- [`public-performance-claims.md`](public-performance-claims.md) — public benchmark claims rated against code + local runs.
- [`vendor-claims-audit.md`](vendor-claims-audit.md) — audit of GreptimeDB's own marketing/comparison pages.
- [`otel-arrow-ingest-assessment.md`](otel-arrow-ingest-assessment.md) — OTel-Arrow (OTAP) ingest assessment.

**Historical implementation studies and improvement assessment**
- [`greptimedb-implementation.md`](greptimedb-implementation.md) / [`clickhouse-implementation.md`](clickhouse-implementation.md) — comparative schema, ingest, query, and retention studies; ClickHouse is not a product target.
- [`platform-fit-and-alternatives.md`](platform-fit-and-alternatives.md) — proxy lens, alternatives survey, the metadata/error-grouping split.
- [`greptimedb-parity-roadmap.md`](greptimedb-parity-roadmap.md) — research assessment of possible GreptimeDB/upstream improvements, not an active Parallax implementation roadmap.

## Source repositories (read, do not vendor into this repo)

- GreptimeDB (Rust): <https://github.com/GreptimeTeam/greptimedb>
- ClickHouse (C++): <https://github.com/ClickHouse/ClickHouse>
