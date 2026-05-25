# Verdict — Which To Choose, And Why

<!-- markdownlint-disable MD013 -->

Status: standing decision, continually sharpened (current through pass 40).
Synthesizes the internals teardowns (all 10 subsystems + rollup + retention,
schema-evolution, dedup), the per-signal matrix, Docker Runs 1–19, and public-claims
triangulation. The runnable `storage-benchmark-prototype.md` holds final veto; this
verdict states the mechanism-grounded recommendation and the triggers that would flip
it. Pins re-verified current through pass 40 (no newer stable on either side:
GreptimeDB v1.1.0 is nightly-only; ClickHouse 26.5.x is the highest line).

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`).

## Headline

**Recommended: GreptimeDB** — but **not because it is the fastest engine. It is
not.** ClickHouse is faster for high-volume log/trace analytics, by clear,
code-confirmed mechanisms. GreptimeDB is the recommendation because its *design
aligns with Parallax's dominant axes*: metrics/PromQL-native, fresh-on-write with
small-write ingest ergonomics, horizontal scale-out designed-in, object-storage
native, and Rust (tiebreak). This is a **fit decision, not a speed decision** —
and the honest correction to the operator hypothesis below makes that explicit.

**The "fit not speed" thesis is now anchored on the query that matters most.** Pass
35 measured the full anchored evidence-bundle composite (Q6 = Q1+Q2+Q3, Run 16): CH
~10 ms vs GreptimeDB ~33 ms, **both far under the 300 ms gate** — so for Parallax's
dominant retrieval, **engine choice is not latency-bound**. The decision therefore
rests on the *fit* pillars below (metrics-native, ingest/upsert ergonomics, retention
cost, scaling), exactly where GreptimeDB leads — not on the analytical-scan latency
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
  query speed**: at 40k series / 8M rows ClickHouse's SQL aggregation was **~10×
  faster** (Run 11; Run-3's near-tie was a 1,200-series small-scale artifact).
  GreptimeDB ties only on **freshness** (both visible-on-write, Run 5). So even on
  metrics, "GreptimeDB fastest" is false for aggregation *latency at volume* — it
  wins on PromQL **maturity/ergonomics** (GA, default-on) vs ClickHouse's
  **experimental** 26.x PromQL (`TimeSeries` engine, off by default) — a real lead,
  but narrower than the old "ClickHouse has no PromQL" framing (corrected pass 44,
  `promql-and-metrics-query.md`).

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
| Freshness | Visible-on-write (tie with ClickHouse, not a win). | smoke |

## Decision question 2 — where is ClickHouse genuinely better, and why?

| Area | Mechanism | Confidence |
| --- | --- | --- |
| **Log/trace selective scan + full-text search** | 8,192 granule + PREWHERE + inverted text index + LowCardinality + C++ SIMD vectorized pipeline. | arch+smoke (Runs 1–2) |
| **Generic wide scan / aggregate throughput** | Decade-tuned vectorized engine — the OLAP-scan bar. Mechanism (pass 42): 65k-row blocks (8× DataFusion's batch) + LLVM-JIT expressions/aggregation + bespoke SIMD kernels + specialized hash aggregation vs GreptimeDB's DataFusion-over-Arrow; explains Runs 11–12. | arch+live (`query-execution-engine.md`) |
| **Vertical single-node ceiling** | Saturates many cores + NVMe on one big box. | arch |
| **Per-column codec tuning** | Hand-picked `DoubleDelta`/`Gorilla`/etc. (counter 7.3×, gauge 78×, Run 4). | smoke (Run 4) |
| **Dynamic-attribute path queries** | `JSON` type stores each path as a **typed columnar subcolumn** (`attributes.k` reads only that subcolumn); GreptimeDB `Json` is a binary blob + `json_get_*` per-row parse. Faster for querying arbitrary OTLP attributes by path at volume. | source+measured (Run 18) |
| Query latency at smoke scale | Won every non-metric query (2–4 ms vs 9–54 ms) — but cache-resident, fixed-overhead-dominated. | smoke |

## Decision question 3 — can ClickHouse replace GreptimeDB for Parallax?

**Yes, technically** — it stored every Parallax signal and returned identical
evidence bundles (Runs 1–4 parity PASS). But three design decisions impose real
cost:

1. **PromQL/Prom/OTLP are experimental or external, not GA-native** → as of 26.x
   ClickHouse *does* have PromQL (`prometheusQuery[Range]`) and Prom remote-write via
   the **experimental, off-by-default `TimeSeries` engine** (pass 44 correction — not
   "absent" anymore), and OTLP still needs a collector. So Parallax would depend on an
   *experimental* metrics path or an external pipeline, vs GreptimeDB's GA-native one.
   A maturity/ergonomics cost now, not a hard capability blocker.
2. **Horizontal scale-out is manual** (shard count + sharding key up front; no OSS
   auto-resharding; `SharedMergeTree` is Cloud-only). Outgrowing the initial layout
   is a disruptive data-move — friction against the startups→big-companies path.
3. **Part-explosion** on streaming small writes → a batching/async-insert layer is
   required to stay healthy.

→ ClickHouse can replace GreptimeDB **at the cost of** a PromQL+OTLP compatibility
layer, a sharding/ops burden, and an ingest-batching layer.

## Decision question 4 — can GreptimeDB replace ClickHouse for Parallax?

**Yes** — it stored every signal and ran Q1–Q6 with identical results. The cost:

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

**Update (Run 12, measured at 5M logs, both indexed):** condition (b) is now
**partly confirmed** — ClickHouse full-text log search is **~18×** faster (7 ms vs
130 ms; mature `text` posting-list index vs GreptimeDB `FULLTEXT`+DataFusion), and
full scans ~4×. So if Parallax's mix is **log-search-dominated**, the flip is real
and large. But the **selective keyed filter was a tie** (4 vs 5 ms), and Parallax's
designed pattern is *anchored* bundle assembly (keyed lookups), not ad-hoc log
search — so the verdict holds **conditional on that workload assumption**. Validate
the assumption (what fraction of real Parallax queries are ad-hoc log search vs
anchored retrieval) — it is now the load-bearing question, not the engine speed.

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
   beyond the cache-resident smoke floor? (Could flip Q5.) Run 12 measured warm 5M
   (CH ~18× on full-text search); the GB–TB cold *latency* number is still owed.
2. **Object-store cost on equal footing** (MinIO) — **largely answered:** retained
   bytes ~tie (compression wash); object count GreptimeDB 4 vs CH 74; and cold GET
   count is **query-shape-dependent** (anchored: CH 5 < GT 22, Run 14; full scan:
   GT 26 < CH 57, Run 15). Remaining: cold-read **egress $** + GET counts under a
   realistic mixed bundle workload.
3. **Concurrent ingest+query freshness p95** — **penalty answered (Run 13):** both
   pass the ≤2× gate (CH 1.55×, GT 1.38×). The precise mixed-load *freshness p95*
   (stamp-emit→poll) still owed.
4. **Multi-node scale-out hold** — does p95 hold as nodes are added; GreptimeDB
   region rebalance vs ClickHouse resharding effort. **Untested (needs multi-node
   harness).**
5. **Realistic-cardinality compression** — **answered (Run 10):** realistic
   99%-unique log text → tie at matched codecs (GreptimeDB 25 vs CH 24.24 MiB),
   GreptimeDB-favored out-of-the-box.

## Supporting notes

- Mechanisms: `greptimedb-internals.md`, `clickhouse-internals.md`,
  `read-path-indexing-and-execution.md`, `write-path-and-ingestion.md`,
  `compression-and-cost.md`, `distributed-and-scaling.md`,
  `compaction-and-merge.md`, `caching-and-cold-warm.md`,
  `rollup-and-continuous-aggregation.md`, `retention-and-ttl.md`,
  `schema-evolution-and-dynamic-columns.md`, `dedup-and-update-semantics.md`.
- Matrix: `per-signal-verdict.md`. Empirical: `local-benchmark-results.md`
  (Runs 1–19). Public claims: `public-performance-claims.md`. Targeted cases:
  `benchmarking-the-differences.md` (B1–B12).
- Build designs: `greptimedb-implementation.md`, `clickhouse-implementation.md`.
- Reproducible object-store stack: `bench/s3/`.
