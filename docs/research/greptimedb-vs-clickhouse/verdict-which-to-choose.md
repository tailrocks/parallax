# Verdict — Which To Choose, And Why

<!-- markdownlint-disable MD013 -->

Status: pass 11 — standing decision, continually sharpened. Synthesizes the
internals teardowns, the per-signal matrix, and Docker Runs 1–5. The runnable
`storage-benchmark-prototype.md` holds final veto; this verdict states the
mechanism-grounded recommendation and the triggers that would flip it.

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`).

## Headline

**Recommended: GreptimeDB** — but **not because it is the fastest engine. It is
not.** ClickHouse is faster for high-volume log/trace analytics, by clear,
code-confirmed mechanisms. GreptimeDB is the recommendation because its *design
aligns with Parallax's dominant axes*: metrics/PromQL-native, fresh-on-write with
small-write ingest ergonomics, horizontal scale-out designed-in, object-storage
native, and Rust (tiebreak). This is a **fit decision, not a speed decision** —
and the honest correction to the operator hypothesis below makes that explicit.

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
  wins on PromQL nativeness, which ClickHouse lacks entirely.

So the ordering "GreptimeDB fastest, then ClickHouse" does not hold for the
analytical query shapes; it inverts. The design decisions that cause it: ClickHouse
optimized the *read path for selective columnar scans* (sparse index + PREWHERE +
SIMD); GreptimeDB optimized for *metric-native ingest + time-series model + object
storage*, accepting a younger DataFusion scan engine.

## Decision question 1 — where is GreptimeDB genuinely better, and why?

| Area | Mechanism | Confidence |
| --- | --- | --- |
| **Metrics / PromQL** | Native PromQL planner + Prom remote-write + metric engine; ClickHouse has no PromQL (needs a translation layer). | plan+smoke (Run 3) |
| **Write ergonomics** | LSM memtable absorbs high-frequency small writes; no ClickHouse "too many parts". Native OTLP/Prom ingest, no collector. | arch+Run 5 |
| **Horizontal scaling** | Region model + Metasrv auto-rebalance + repartition + compute/storage separation (object store + remote WAL) → topology change, not rewrite. | arch (multi-node owed) |
| **Object-storage-native** | OpenDAL default + read cache; cheap re-readable retention first-class. Fewer *total* objects (4 vs 74, Runs 8–9) → wins full-scan cold reads. **But for a cold *anchored* lookup ClickHouse issued fewer S3 GETs (5 vs 22, Run 14)** — sort-key locality beats GreptimeDB's index indirection; request cost is query-shape-dependent. Read cache → warm re-reads local on both. | measured (layout + anchored cold-GETs); full-scan cold owed |
| Freshness | Visible-on-write (tie with ClickHouse, not a win). | smoke |

## Decision question 2 — where is ClickHouse genuinely better, and why?

| Area | Mechanism | Confidence |
| --- | --- | --- |
| **Log/trace selective scan + full-text search** | 8,192 granule + PREWHERE + inverted text index + LowCardinality + C++ SIMD vectorized pipeline. | arch+smoke (Runs 1–2) |
| **Generic wide scan / aggregate throughput** | Decade-tuned vectorized engine — the OLAP-scan bar. | arch |
| **Vertical single-node ceiling** | Saturates many cores + NVMe on one big box. | arch |
| **Per-column codec tuning** | Hand-picked `DoubleDelta`/`Gorilla`/etc. (counter 7.3×, gauge 78×, Run 4). | smoke (Run 4) |
| Query latency at smoke scale | Won every non-metric query (2–4 ms vs 9–54 ms) — but cache-resident, fixed-overhead-dominated. | smoke |

## Decision question 3 — can ClickHouse replace GreptimeDB for Parallax?

**Yes, technically** — it stored every Parallax signal and returned identical
evidence bundles (Runs 1–4 parity PASS). But three design decisions impose real
cost:

1. **No native PromQL / Prometheus remote-write / OTLP** → Parallax must build and
   maintain a PromQL→SQL layer and an OTLP→ClickHouse collector pipeline. Metrics
   are a first-class Parallax signal, so this is not optional.
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
   gap shrinks (Run 2) — so this blocker is less central than it looks.
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

0. **Reproduce the JSONBench cold-run result** (public-claims pass 22): GreptimeDB
   reportedly ranks #1 on ClickHouse's own JSONBench cold run at 1B docs. **This
   cold / object-store / wide-record regime is the one Parallax actually lives in**
   (evidence-bundle re-reads from cheap object storage), the *opposite* of the hot
   in-cache scans my B1/B5 measured (where ClickHouse won). If it reproduces, it
   **strengthens** the GreptimeDB verdict for Parallax's real access pattern.
   Highest-priority reproduction (`public-performance-claims.md`, B12).
1. **Cold-cache GB–TB log/trace scan gap** — how much slower is GreptimeDB really,
   beyond the cache-resident smoke floor? (Could flip Q5.) NB: my B1/B5 were
   warm/hot; the *cold* regime may behave oppositely (see #0).
2. **Object-store $ on equal footing** (MinIO): retained bytes, GET/PUT/LIST, cold-
   read egress — is GreptimeDB's object-store-native economics a real cost win?
   **Partly answered (Runs 8–9): yes on object count (4 vs 74, ~18× fewer requests
   per read); GET/PUT/LIST counts during cold query still owed for the $ figure.**
3. **Concurrent ingest+query freshness p95** — the real axis-1 number under load.
4. **Multi-node scale-out hold** — does p95 hold as nodes are added; GreptimeDB
   region rebalance vs ClickHouse resharding effort.
5. **Realistic-cardinality compression** — re-run with real log text (the smoke
   synthetic data distorted the logs result).

## Supporting notes

- Mechanisms: `greptimedb-internals.md`, `clickhouse-internals.md`,
  `read-path-indexing-and-execution.md`, `write-path-and-ingestion.md`,
  `compression-and-cost.md`, `distributed-and-scaling.md`.
- Matrix: `per-signal-verdict.md`. Empirical: `local-benchmark-results.md` (Runs 1–5).
- Build designs (next): `greptimedb-implementation.md`, `clickhouse-implementation.md`.
