# Deep Internals Comparison — GreptimeDB vs ClickHouse

I want one focused, never-ending research loop that does exactly one thing:
compare **GreptimeDB** and **ClickHouse** at the level of their actual technical
implementation — how each one works under the hood, and which design decisions
make each fast or slow for Parallax's workload (metrics, logs, traces, and
cross-signal evidence-bundle correlation).

This is not a market comparison and not a feature checklist. Marketing claims and
vendor blog numbers are inputs to be verified, never the verdict. The deliverable
is a mechanism-level explanation: *what the code actually does on the write path,
the read path, indexing, compaction, compression, query execution, the
distributed model, and the object-storage path — and why that makes it fast or
slow for a specific signal type.*

## How this differs from the research we already have

Three documents already exist and must not be duplicated. This loop sits one
layer deeper than all of them:

- [`docs/research/storage/evaluation.md`](../docs/research/storage/evaluation.md)
  is a strategy/fit evaluation (open source, maturity, metrics-native posture,
  risks). It reasons *about* the systems, not *inside* them.
- [`docs/research/storage/benchmark-plan.md`](../docs/research/storage/benchmark-plan.md)
  defines *what to measure and why*.
- [`docs/research/storage/benchmark-plan.md`](../docs/research/storage/benchmark-plan.md)
  is the runnable, black-box harness that produces numbers and holds **veto
  power** over the default storage choice.

This loop is the **white-box** counterpart to that black-box benchmark. The
benchmark measures *that* one system is faster; this loop must explain *why* it
is faster, by reading the architecture docs and the source code. The two are
designed to confirm each other: when the benchmark shows a result, the internals
analysis must be able to predict and explain it from the data structures and code
paths. A benchmark number that the internals cannot explain is a signal that
either the benchmark or the analysis is wrong — flag that explicitly.

---

# Prompt Maintenance Rule

This prompt is durable operator intent for repeated research runs, not a
disposable input. When the operator clarifies the comparison target, names a new
subsystem to
dissect, changes the evaluation criteria, pins a different version, or confirms a
finding, update this file in the same change if a future run would otherwise use
stale instructions. Do not keep important direction changes only in chat or only
in the generated notes.

---

# Run Mode

This brief is meant to run indefinitely. It never self-completes — it runs deepening
and re-verification passes until the operator stops or replaces it. Treat it as
continuous: do not converge on a single deliverable and stop.

Run passes back to back (nothing external is being watched, so there is no reason
to idle between them). Each pass picks the single highest-value unanswered
internals question, researches it against primary sources and source code, writes
or revises one focused note under `docs/research/storage/greptimedb-vs-clickhouse/`, then
commits and pushes per [`AGENTS.md`](../AGENTS.md). Each pass must surface the
pass target, versions checked, evidence produced, files changed, commit pushed,
remaining uncertainty, and next gap so the run controller can see that the
research is still making progress. Do not declare the comparison "done" just
because the output files exist; keep deepening until told to stop.

---

# The Operator Hypothesis To Verify (Do Not Cheerlead)

The operator's working belief is:

> GreptimeDB will be the fastest, then ClickHouse. If that is not true, explain
> exactly why — at the level of the design decision that causes it.

Treat this as a hypothesis to **test honestly**, not a conclusion to defend. This
project's north star is to verify the belief, not cheerlead it. If ClickHouse's
columnar engine, sparse index, and merge model make it faster for high-volume log
or trace analytics, say so plainly and trace it to the specific mechanism
(vectorized execution, granule skipping, codec choice, merge strategy). If
GreptimeDB wins for metrics or for object-storage retention, prove it from the
storage engine and metric-engine design, not from a vendor claim. A result that
contradicts the operator's belief, fully explained at the mechanism level, is the
single most valuable thing this loop can produce.

Every "X is faster" claim in the output must carry a *because* tied to a concrete
data structure or code path, and a *scenario* (which signal, which query shape,
what cardinality, hot vs cold cache, single-node vs scaled-out).

## Practical fit, not raw speed — and the Parallax-proxy lens (operator, 2026-05-25)

Judge the systems on **practical fit for Parallax's vision**, not on which is abstractly faster. The
decisive architectural fact: **Parallax is the first layer — a proxy that owns OTLP ingestion, routing,
and conversion, and writes to whatever backend it chooses.** Consequences this loop must apply:

- **Native-protocol / ingest-ergonomics advantages are largely NEUTRALIZED.** "GreptimeDB speaks OTLP/
  PromQL/Jaeger natively and needs no collector/pipeline" stops being a differentiator, because Parallax
  *is* that pipeline by design and translates to any backend API. If ClickHouse lacks a format out of
  the box, Parallax supplies it. Weight ingest-nativeness near zero in the verdict; do **not** let the
  Run 150–152 native-trio findings drive the recommendation.
- **What still counts (Parallax can't paper over it):** retrieval speed, storage cost/compression,
  object-store economics at scale, high-cardinality handling, horizontal-scale/topology change, and —
  central — **the "build on top" ecosystem surface.** ClickHouse leads retrieval + ecosystem; GreptimeDB
  leads object-store economics + metrics cardinality + auto-rebalance. Score on these, re-weighted.
- **Alternatives are in scope.** Each pass may sanity-check whether a third system (within the
  language filter) beats both as an *embeddable backend* — OpenObserve (a competitor *platform*, not a
  DB), Quickwit (logs/traces only), InfluxDB 3, VictoriaMetrics/Logs (split products), StarRocks/Doris
  (JVM-FE filter risk). Current finding: none clearly beats CH/GT as a backend.
- **The data model is a 2–3 store split, not one engine.** metrics/logs/traces/raw-error-events →
  columnar store (ClickHouse or GreptimeDB); **Sentry-style grouped errors + metadata (mutable,
  relational, OLTP) → Postgres**; cold tier → object storage. Do not force mutable issue state into the
  columnar engine (Sentry's ClickHouse "replacements consumer" is the warning). See
  [`platform-fit-and-alternatives.md`](../docs/research/storage/greptimedb-vs-clickhouse/platform-fit-and-alternatives.md).

Net standing lean under the proxy: **ClickHouse is the pragmatic default**; GreptimeDB is the choice
only for the metrics-cardinality/PromQL · self-hosted-1×-S3-economics · mandatory-auto-rebalance bet.
Keep testing this honestly — flip it if the mechanism evidence says so.

## Count Experimental As Stable — Judge On Mechanism And Trajectory

Operator rule (durable): when either system gates an observability capability
behind an *experimental* flag, evaluate it **as if it were stable**. Do not lower
the score, hedge the verdict, or treat the feature as absent because of the
experimental label. ClickHouse's `TimeSeries` engine, its PromQL table functions
(`prometheusQuery` / `prometheusQueryRange`), the `timeSeries*ToGrid` family,
lightweight `UPDATE` / `DELETE`, the `JSON` type, async inserts — and GreptimeDB's
equivalently young subsystems — all count as real, shipping capabilities. Both
teams are best-in-class and both ship fast; the question is which *design* serves
Parallax best and which has the **bigger future advantage**, not which printed a GA
stamp first.

What this rule does and does not change:

- **Capability is judged on the mechanism, present-tense.** If the feature works
  when enabled — verify it live, do not assume — it counts. "Experimental,
  therefore it loses" is banned reasoning.
- **Maturity and ergonomics may still be reported — but only as a *shipping-today*
  operational note, never inflated into a capability or speed verdict.** "Off by
  default / needs a flag / setup-heavy / fewer integrations today" is a fair cost to
  state plainly; "experimental, so it does not count" is not. Do not let a maturity
  gap masquerade as a mechanism gap.
- **Trajectory is a first-class axis.** For every gap, ask which design's *future*
  is stronger: is the laggard's gap an engineering item already in flight (cite the
  changelog / RFC / PR), or a true architectural limit? A capability that ships
  experimentally *this* release and closes a gap is evidence the *direction* favors
  that system — weigh it explicitly when judging the bigger future advantage.
- **Symmetry.** Apply the identical standard to GreptimeDB. Neither system is
  penalized for the experimental label; both are judged on what the code does now
  and where the design is heading.

---

# Output Location And File Plan

All output goes under a dedicated subfolder:

```text
docs/research/storage/greptimedb-vs-clickhouse/
```

The subfolder `README.md` is the index and method log — keep it current. Grow the
following focused notes over successive passes (split or rename if the material
demands it, and update the README and `PROJECT_STRUCTURE.md` when you do):

- `README.md` — index, method, version pins, source-commit references, status.
- `greptimedb-internals.md` — GreptimeDB architecture and code-path teardown.
- `clickhouse-internals.md` — ClickHouse architecture and code-path teardown.
- `write-path-and-ingestion.md` — both systems' ingest path to queryable, side by
  side, with the freshness consequence.
- `read-path-indexing-and-execution.md` — query planning, indexing, execution
  model, and what gets skipped vs scanned.
- `compression-and-cost.md` — on-disk/object layout, codecs, compression by
  signal, and the retention-cost consequence.
- `distributed-and-scaling.md` — single-node ceiling and the horizontal-scale
  design of each.
- `greptimedb-implementation.md` — the concrete Parallax-on-GreptimeDB storage
  design: full schema, ingest path, and exact retrieval queries (see below).
- `clickhouse-implementation.md` — the concrete Parallax-on-ClickHouse storage
  design: full schema, ingest path, and exact retrieval queries (see below).
- `per-signal-verdict.md` — the scenario matrix: metrics vs logs vs traces vs
  evidence-bundle correlation, who wins each and the mechanism why.
- `benchmarking-the-differences.md` — for each mechanism-level difference found,
  the targeted benchmark to measure it for Parallax usage and what we must have to
  run it (see below); routes runnable cases into `storage-benchmark-prototype.md`.
- `local-benchmark-results.md` — the empirical log of local Docker runs: env,
  pinned image tags, dataset, queries, real measured numbers, and which published
  claim each run confirms or refutes (see below).
- `verdict-which-to-choose.md` — the final synthesized decision (see below).
- `greptimedb-parity-roadmap.md` — for the recommended system, the per-capability
  gap-closing analysis: what GreptimeDB would implement (against its real internals) to
  match each ClickHouse advantage, tiered by effort (see "Closing The Gap" below).

Keep each note source-linked and concise. Prefer comparison tables plus short
mechanism analysis, per the repo research conventions.

---

# Method: Read The Source, Not The Marketing

1. **Pin versions first, every pass — always the latest.** This is a hard rule:
   always compare the newest stable release available on both sides at run time.
   At the start of every pass, check for a newer stable release of each system and
   bump to it before comparing; if one side has shipped a new version, upgrade the
   comparison rather than reusing an old number. Record the exact version and the
   source commit SHA you read in every note. Current pins (re-verified 2026-05-25,
   still latest — re-check and bump at run time): GreptimeDB `v1.0.2` (GA 2026-05-14;
   `v1.1.0` is nightly-only, not GA), ClickHouse `v26.5.1.882-stable` (the newest stable
   *feature* line — pin the exact patch; note newer-dated `26.x-lts`/`26.2/26.3` tags are
   backport/LTS patches of older lines, not higher than 26.5). *(Brief was authored at
   ClickHouse `25.x`; bumped through the loop to 26.5.)* Never analyze an old major
   against a current one unless the point is explicitly historical, and never carry
   forward a stale benchmark or claim as current — re-verify it against the latest version.

2. **Clone the source and read it.** The repos are open:
   - GreptimeDB (Rust): <https://github.com/GreptimeTeam/greptimedb>
   - ClickHouse (C++): <https://github.com/ClickHouse/ClickHouse>

   Clone to a scratch location **outside this repository's working tree** (e.g.
   `~/src/` or `/tmp/`). Never commit upstream source into this repo. When a claim
   rests on the implementation, cite the file path and, where useful, the
   function or line, plus the commit SHA you read. "The code does X (path/to/file,
   commit abc123)" beats "the docs say X."

3. **Use design docs as the map, the code as the ground truth.** Read each
   project's architecture and design documents to orient, then confirm the
   load-bearing claims against the actual code. When a doc and the code disagree,
   trust the code and note the discrepancy.

4. **Gather the public performance claims, then verify each against the code.**
   Search the internet for what each project says about its own speed and what
   others have measured: vendor benchmark posts, "X is faster than Y" claims,
   engineering blogs, conference talks, design RFCs and proposals, release notes
   and changelogs (especially performance work — new index types, codec changes,
   execution-engine rewrites, compaction or merge improvements), GitHub issues and
   PRs that landed an optimization, and independent third-party benchmarks. For
   every claim that matters, do not record it as fact — trace it to the mechanism
   in the source that would produce it, and rate it:
   *confirmed by code*, *plausible but unverified*, *contradicted by code*, or
   *workload-specific (true only under stated conditions)*. A claim that cannot be
   located in the implementation is a marketing assertion, not evidence. Note the
   claim's date and the version it referred to (claims go stale as both projects
   ship improvements — re-check against the current pinned version).

5. **Predict, then check against the benchmark.** Where the runnable benchmark
   (`storage-benchmark-prototype.md`) has produced numbers, the internals analysis
   must explain them. Where it has not, state the internals-based prediction so a
   later benchmark run can confirm or falsify it. When a public claim, the code
   reasoning, and the benchmark disagree, that three-way conflict is a top-priority
   finding — resolve it or flag it loudly.

---

# What To Dissect — The Subsystem Checklist (Both Systems)

For each system, build a mechanism-level account of every item below, then put
the two side by side. The point of each is always: *how does this make the system
fast or slow, and for which signal?*

1. **On-disk / on-object data layout.** Row vs column orientation, file format,
   block/granule/part structure, how a single signal's data is physically grouped
   and sorted.
2. **Write path (ingest → durable → queryable).** Buffering, memtable/insert
   block, WAL/durability, the flush/part-creation step, and exactly when written
   data becomes visible to a query (the freshness mechanism).
3. **Indexing.** Primary/sort key, time index, secondary/skip indexes (min-max,
   bloom/token/ngram, inverted, full-text), and what each index lets the engine
   *skip* at read time.
4. **Read path and query execution.** Query planning, predicate pushdown, the
   scan/skip decision, vectorized/columnar execution, parallelism, and the join
   strategy (critical for cross-signal correlation).
5. **Compaction / merge.** The background process that reorganizes data, its
   strategy (time-window vs level/merge), write amplification, and its effect on
   read speed and freshness.
6. **Compression.** Per-column/per-signal codecs, dictionary/low-cardinality
   handling, and the size and CPU consequence for logs vs traces vs metrics.
7. **Caching.** Page/block cache, mark/index cache, vector/write cache, and how
   cold-cache vs warm-cache performance diverges.
8. **Distributed model.** Component roles, sharding/partitioning, replication and
   consensus, how a query fans out, and whether scale-out was designed in from the
   start or added later.
9. **Object-storage path.** How (and whether) data lives on S3-style storage, the
   request/latency pattern, local cache in front of it, and the cost/freshness
   tradeoff versus local disk.
10. **Schema / dynamic columns.** Handling of changing OTLP attributes,
    high-cardinality tags, JSON/Map columns, and the metric-series model.
11. **Native observability data model (out-of-the-box).** What table/column
    structure each system creates for OTLP metrics, logs, and traces with no schema
    work — the default ingest schema, the built-in log/trace models, the metric
    engine — and whether that native structure is good enough to adopt for Parallax
    or must be customized (feeds the adopt-native-vs-custom decision in the
    implementation documents).

---

# System-Specific Leads To Chase (Verify Each Against Source)

These are starting leads, not facts to trust. Confirm names, behavior, and
current relevance against the pinned source — components get renamed, replaced, or
removed.

## GreptimeDB (Rust)

- The storage engine (LSM-tree family) and region engine: memtable structure, SST
  file format (Parquet-based?), the time-index column, tag/primary-key columns.
- The indexing stack: inverted index, full-text index for logs, skipping index —
  what each accelerates.
- Compaction strategy (time-window oriented?) and the file-purge path.
- WAL options: local engine vs remote log (Kafka), and the durability/freshness
  tradeoff of each.
- The query engine: its relationship to Apache DataFusion / Arrow, and the PromQL
  planning path.
- The metric engine: logical metric tables mapped onto a physical wide table, and
  the partitioning consequence for high-cardinality metrics.
- Object storage via the storage abstraction (OpenDAL?), the write/read cache in
  front of object storage.
- The distributed split: Frontend (stateless), Datanode (storage/compute),
  Metasrv (metadata/scheduling), and region migration/rebalancing.
- Append mode for logs and its effect on the write path.

## ClickHouse (C++)

- The MergeTree engine family (incl. Replacing/Aggregating/Summing) and the part →
  granule → mark structure.
- The sparse primary index over `ORDER BY` (not a B-tree) and how
  `index_granularity` governs the scan/skip decision.
- Data-skipping indexes (min-max, set, bloom_filter, tokenbf, ngrambf) — which
  ones matter for log search and trace lookups.
- Compression codecs (LZ4, ZSTD, Delta, DoubleDelta, Gorilla, T64) and which suit
  metrics vs logs vs trace columns; `LowCardinality` and `Map`.
- The vectorized execution pipeline and how parallelism is structured.
- Background merges and mutations, and their effect on read speed and on freshness
  (incl. async inserts).
- Rollup/correlation tooling: materialized views, projections, AggregatingMergeTree.
- The distributed story: Distributed engine, sharding, ReplicatedMergeTree +
  Keeper (Raft), and the S3 disk + storage policies + zero-copy replication for
  object storage and tiering.

---

# The Scenario Matrix (The Core Question)

The whole loop converges on this: **for each signal and query shape, which system
is faster, and because of which mechanism?** Build and continually refine a matrix
covering at least:

- **Metrics** — high-cardinality series ingest and PromQL-style range/aggregation
  queries. (GreptimeDB metric-native model vs ClickHouse codecs +
  AggregatingMergeTree.)
- **Logs** — high-volume append, full-text/substring search, severity/service
  filters over time windows. (ClickHouse token/ngram skip indexes + string codecs
  + fast columnar scan vs GreptimeDB full-text index + append mode.)
- **Traces** — wide spans, `trace_id` point lookups, span trees, status/duration
  filters. (Sort-key/primary-key locality in each.)
- **Evidence-bundle correlation** — the Parallax-specific pattern: cross-signal
  joins by `trace_id` / `fingerprint` / time window to assemble one bundle
  (mirror the exact queries Q1–Q6 in `storage-benchmark-prototype.md`). This is
  where the join strategy and index design of each engine decide the winner, and
  it is the query that matters most for Parallax.

For each cell: name the winner, the mechanism, the scenario qualifiers
(cardinality, window, hot/cold cache, single-node/scaled), and the confidence
(architecture-reasoned vs benchmark-confirmed).

---

# Concrete Implementation Design Per System

The comparison is not complete until each candidate has a concrete, buildable
Parallax storage design written down — so the choice is "we know exactly how we
would build it on either", not an abstraction. Produce one design document per
system (`greptimedb-implementation.md` and `clickhouse-implementation.md`), each
answering: *if Parallax stored its data in this system, what is the structure and
how do we get the data back out?*

Each implementation document must specify, for the full Parallax signal set
(error events, spans, logs, metrics, deploy markers, CLI invocations, agent
actions, frontend events):

- **Native out-of-the-box structure first — then an explicit adopt-native vs
  design-custom decision.** Before designing any schema, establish what each system
  gives you with **zero schema work**: what table(s) and column layout it
  auto-creates when a standard client just sends OTLP metrics / logs / traces (and
  Prometheus remote-write) at it. Verify it **live**, do not assume — e.g.
  GreptimeDB's OTLP / pipeline auto-created tables, its log-table model, its trace
  model (and Jaeger query API), and the metric engine; ClickHouse's default
  observability schema (the OpenTelemetry Collector ClickHouse exporter tables /
  "ClickStack" defaults, since OTLP ingest is collector-mediated — confirm whether a
  native receiver exists). Then decide, **per signal**: if the native / default
  structure already serves Parallax's retrieval — anchored evidence-bundle
  correlation by `trace_id` / `fingerprint` / time window — **prefer adopting it**:
  it is less to build, less to maintain, and stays aligned with the ecosystem and
  each project's own tuning. Only **design a custom schema where the native one
  demonstrably falls short** (wrong sort/primary key for the anchored lookup, no
  home for `fingerprint`, dynamic-attribute handling that hurts the hot path, etc.) —
  and when you deviate, **name the exact native shortfall that forces it**. Do not
  hand-roll a schema without first proving the out-of-the-box one inadequate; do not
  adopt the native one without proving it actually fits. This native-vs-custom
  finding is a required part of each implementation document.
- **Schema / structure.** The complete table design in that system's real DDL:
  table engine / table type, the time/sort/primary key and why it is ordered that
  way, tag vs field columns, per-column types and compression codecs, JSON/Map vs
  flat columns for dynamic OTLP attributes, partitioning, TTL/retention, and any
  rollup/materialized structures (materialized views / projections /
  AggregatingMergeTree for ClickHouse; metric engine, append mode, logical→physical
  tables for GreptimeDB). Justify each choice from the engine's internals — the
  schema must exploit how that engine actually stores and skips data.
- **Ingest path.** How each signal lands: OTLP and Prometheus remote write for
  GreptimeDB; the insert/`Map`/materialized-view path (and async inserts) for
  ClickHouse — and how schema choices affect ingest-to-queryable freshness.
- **Retrieval.** The exact queries Parallax runs to get data back, in that
  system's real dialect: the evidence-bundle/correlation set (mirror Q1–Q6 in
  `storage-benchmark-prototype.md` — trace-context fetch, issue/fingerprint
  history, release-regression diff, cross-tier frontend↔backend join,
  high-cardinality filter, and the composite bundle), plus how each query uses the
  schema's keys and indexes, and where it must fall back to a scan.
- **Object-storage and retention layout.** The S3-mode configuration and the
  hot/cold tiering for cheap long retention with re-readable history.
- **Operational shape.** What must run for a Tier-1 single-node deployment and how
  the same schema survives scale-out.

Build on, do not duplicate, the seed DDL and Q1–Q6 already in
`storage-benchmark-prototype.md`; expand them into a complete storage design and
keep the two consistent (the benchmark runs what these documents specify). Put the
two designs side by side so the structural differences — and what each makes easy
or hard for Parallax retrieval — are obvious.

---

# Benchmarking The Differences (What To Measure And What We Need)

A mechanism-level difference is only a real advantage if it shows up under
Parallax's usage. So whenever a pass finds a difference that plausibly moves
speed, cost, or scaling, it must also design the targeted benchmark that would
prove or disprove the advantage — not just assert it. Capture these in
`benchmarking-the-differences.md`.

For each difference worth measuring, specify:

- **The hypothesis and the mechanism.** "GreptimeDB's metric engine should beat
  ClickHouse on high-cardinality PromQL range queries because …", tied to the data
  structure from the internals notes.
- **The targeted workload.** The specific signal, query shape, cardinality, time
  window, cache state (cold/warm), and concurrency (write-only vs ingest+query)
  that isolates this one difference — a micro-benchmark, not a general scan.
- **What to record.** The metric tied to an axis: ingest-to-queryable freshness,
  per-class query latency (p50/p95/p99), retained size and compression by signal,
  object-store request/egress count, CPU/RSS per phase, or the single-node
  breaking point — using the measurement protocol already defined in
  `storage-benchmark-prototype.md`.
- **The pass/fail or comparison criterion.** What result confirms the advantage,
  what falsifies it, and what counts as "close enough to not matter for Parallax".
- **What we must have to run it.** The concrete prerequisites: pinned versions, a
  dataset shape and the generator knobs that produce it (cardinality, linkage,
  signal mix), object store (MinIO/S3), hardware/resource profile, the schema/DDL
  under test, the exact queries, and any instrumentation the harness still needs.
  Call out anything the current harness cannot yet measure and what to add.

Keep this consistent with, and routed into, the runnable harness: this document
proposes and refines the cases; `storage-benchmark-prototype.md` is where they
become runnable and holds veto power over the storage choice. New cases discovered
here should be folded back into that prototype (and its generator/queries
extended) rather than forked into a parallel benchmark. Distinguish what is
**already runnable there** from what is a **proposed new case**.

---

# Cost Is A First-Class Axis — Measure $ Alongside Speed (operator, 2026-05-25)

We have mostly measured **performance**. The operator wants the **cost** to run each system measured
too — so the final decision weighs **time-to-answer against cost-to-run**. Every speed number should
carry its **$ context**: *"X ms at $Y/GB-month on Z compute,"* not a bare latency.

**The storage-architecture thesis (verified — `storage-cost-and-tiering.md`, Run 161):**
- **ClickHouse is performance-first / local-storage-centric.** Speed comes from attached SSD/NVMe
  locality + sparse index + vectorized C++. On object storage there is a **real cold-read penalty**
  (cold S3 ~10–500 ms vs warm cache ~250 µs ≈ ~2000×; latency-bound), so S3 in OSS ClickHouse is a
  **cold tier** (hot-cold tiering is ClickHouse's own recommended S3 pattern), and OSS HA still keeps
  **N× copies**. (ClickHouse Cloud's distributed cache largely closes the S3 gap at a $ premium.)
- **GreptimeDB is S3-native / cost-first.** SSTs flush directly to object storage at **1× shared copy**
  with a local hot cache and near-stateless, elastic compute — built to store a lot cheaply and read
  back as fast as cache + the (younger) engine allow.
- *Correction to avoid overstating:* ClickHouse is **not "broken" on S3** — it is tier-vs-native; the
  real OSS issues are cold-read latency on uncached data + N× replication, not inability.

**Measure these cost components (per benchmark, alongside latency):**
1. **$/GB retained** = on-disk size (per signal — densities differ, metrics/logs favour GreptimeDB,
   traces favour ClickHouse) × storage-class price (S3 ~$0.023/GB ≈ 3.5× cheaper than EBS gp3 ~$0.08/GB)
   × **replication factor (1× GreptimeDB vs N× OSS ClickHouse)**.
2. **Compute footprint for the SLA** — instance count/size to hold the latency target, and whether it
   must be **always-on** (ClickHouse hot tier) or can be **elastic/scale-to-low** (GreptimeDB near-
   stateless). The cost of the SLA, not just the SLA.
3. **Cold-read cost** — latency **and** S3 GET/egress for re-reading historical data (Parallax re-reads
   history for AI context; egress can dominate — R2 zero-egress matters, `retention-cost-model.md`).
4. **Operational cost** — what must run (ClickHouse Keeper + manual resharding; GreptimeDB metasrv +
   optional Kafka), or the managed-cloud premium.

**Which system for what (state this in the verdict):** **ClickHouse optimizes time-to-answer on hot
data** (pay more compute + premium hot storage + N× copies → fastest queries); **GreptimeDB optimizes
$/GB on deep data** (S3-native 1×, ~3.5× cheaper medium, denser on metrics+logs, fewer/elastic servers →
store everything cheaply, read back via cache/engine).

**The hybrid question (evaluate, don't assume):** Parallax could run **ClickHouse for live/hot data +
GreptimeDB for cold/historical** (roll CH→GT by age; the proxy routes queries by time range). It's the
cross-engine version of ClickHouse's own hot-cold pattern, getting CH's best hot speed + GT's best cold
cost — but it **doubles operational surface** (two engines + a roll-over pipeline + cross-boundary query
federation) against the anti-complexity goal. Benchmark it as `CH(hot window) + GT(historical) +
federation` **vs** single-engine internal tiering, at a realistic hot:cold ratio. Treat the hybrid as a
**Phase-2 cost optimization** the proxy enables without a rewrite — not a Day-1 default. Decide on the
sized $ numbers.

---

# How To Benchmark — Production Realism Over Scoreboard Optics

Benchmarks here exist to predict the **real Parallax user's experience**, not to
produce a flattering number. The operator does not care how the benchmark *looks*;
the operator cares whether, on the system Parallax actually ships, the everyday
workflow is fast and everything stays queryable under load. Hold every benchmark to
this standard:

1. **Model the real usage, not a microbenchmark trophy.** Parallax's user is one
   developer / SRE / AI agent (and later a team) hitting the store with the queries
   they run *often*: anchored evidence-bundle assembly (Q1–Q6), a request-id /
   trace-id grep during an incident, a metric-panel refresh, a recent-logs tail.
   Benchmark *those*, at a realistic shape and load, against a realistically
   populated store — the queries that decide whether the product feels fast,
   weighted by how often they actually run. A system that is heavily and repeatedly
   used by one user is the case to optimize for.
2. **No benchmark tricks — ever.** Vendors game benchmarks; we do the opposite.
   Forbidden: cherry-picking the query form that flatters one engine, tuning one
   side's schema / flags / codecs while leaving the other on defaults, reporting
   only warm when cold is the real user moment (or vice-versa), hiding the ingest
   cost by querying a frozen table, or picking a cardinality / window that dodges a
   known weak spot. If a result depends on a trick to look good, it is worthless
   here — discard it.
3. **Fair footing on both sides.** Give *each* engine its best honest
   configuration: if you tune codecs / indexes / flags on one, apply equivalent
   effort on the other, and state what you did for each. Enable each engine's
   relevant capabilities (experimental counts as stable, per the rule above). The
   comparison is design-vs-design at equal effort, never tuned-vs-default.
4. **Measure what the user feels.** Lead with user-perceptible outcomes:
   ingest-to-queryable freshness, end-to-end latency of the *often-run* queries
   (p50 / p95 / p99, warm **and** cold stated separately), and whether latency holds
   under concurrent ingest+query (the real production state) — not an isolated
   read-only scan. Always attach the scenario qualifiers: signal, query shape,
   cardinality, time window, cache state, single-node vs scaled, single-user vs
   concurrent.
5. **A laptop smoke run is indicative, never the verdict.** State the scale and
   hardware; label small-scale numbers as directional; route the load-bearing
   questions to the sized / cold / multi-node harness that holds veto.

# Reproducibility Contract — Every Run Is Re-Runnable By Hand

The operator must be able to re-run any benchmark himself and land on the same
number. **A result that cannot be reproduced from the written record does not
exist.** So for *every* benchmark logged in `local-benchmark-results.md`, capture
enough that a reader reproduces it byte-for-byte without asking the agent — **and**
record *how the benchmark was constructed and how the two systems were compared*,
not merely the number:

- **Exact environment.** Image tags **and** the source commit SHA under test, host
  hardware / OS, Docker / compose versions, and how the stack was brought up (the
  `bench/` compose or script invocation).
- **Exact dataset.** The generator command, the seed, row counts per table, the
  shape (cardinality, linkage, signal mix), and which file / table loaded where — so
  the same bytes regenerate.
- **Exact schema.** The full DDL under test for *both* systems (copy-pasteable),
  every index / codec / flag included, with the per-side tuning stated.
- **Exact queries.** The literal queries run on each system, copy-pasteable in that
  system's dialect, in the form actually executed. (Note: the dev sandbox blocks
  host→container ports, so prefer the `docker exec …` form the agent actually ran;
  record it verbatim.)
- **Exact method.** Warm vs cold and how cache state was set, repetition count and
  which figure is reported (min / median), the timing source (ClickHouse `--time`,
  GreptimeDB `execution_time_ms`, or wall-clock — and note their non-comparability),
  and any overhead stripped.
- **The comparison logic and the result.** State *what mechanism the case isolates*
  and *why this query / shape is the fair way to compare the two here*, then the
  measured numbers for both sides, then the verdict against the claim being checked —
  **confirmed / refuted / workload-specific / inconclusive-at-this-scale** — tied to
  the internals mechanism that predicts it.

Rule of thumb: a future pass, or the operator, should be able to open the entry,
paste the commands, land within noise of the recorded number, **and understand why
that comparison is fair**. If they cannot, the entry is incomplete — finish it
before moving on.

# Verify Claims Locally With Docker (Measure, Do Not Just Reason)

A local Docker environment is available, so do not stop at reasoning from the code
— actually stand each system up and measure. Reading the source tells you *why* a
result should happen; a local run tells you *whether* it does. Run real
comparisons, record real numbers, and use them to confirm or refute the published
performance claims for each system. Capture every run in
`local-benchmark-results.md`.

## STANDING RULE — every benchmark runs on ALL FOUR builds + updates the comparison table

Operator directive (durable): **every performance benchmark must be measured on all four builds**,
never stable-only, never a 2-way:

1. **GreptimeDB — latest stable** (currently `v1.0.2`, production-OK).
2. **GreptimeDB — latest nightly** (currently `v1.1.0-nightly-…`, everything not-yet-released).
3. **ClickHouse — latest stable feature line, NOT LTS** (currently `v26.5.1.882-stable`).
4. **ClickHouse — latest nightly** (`clickhouse/clickhouse-server:head`, currently `v26.6.x`).

**Two tiers — local SMALL (preliminary), server LARGE (detailed, on request only).** On the laptop,
run a **small but meaningful** tier — default `N=100,000` (min 50,000). **Do NOT run millions-scale
locally** — four DB containers + millions of rows **freezes the operator's MacBook**. The proper
large-scale test (`N=5,000,000`+) runs on a **server**, and **only when the operator explicitly asks**
— do not launch it locally on your own. **Don't keep all four containers standing with big data on the
laptop:** `docker start` the nightlies → `gen.sh` (small) → `bench.sh` → `docker stop` the nightlies.
Build identical data on all four via `range()` (GT) / `numbers()` (CH). On a new nightly tag re-pull.
**Every benchmark must also update [`four-way-version-comparison.md`](../docs/research/storage/greptimedb-vs-clickhouse/four-way-version-comparison.md)** — the single consolidated matrix (every query × 4 builds,
a *Faster* column, per-query *Details* links to the mechanism note + run). Always re-pin the latest
stable + nightly of both at the start of a benchmarking pass. Do **not** record a stable-only number
as the result; the four-build row is the result.

How to run the benchmarks:

- **Stand up the candidates in Docker.** Bring up GreptimeDB and ClickHouse (and
  MinIO for the object-storage path) from official images, pinned to the exact
  version tags under analysis (the same versions pinned in the Method section).
  Prefer a small `docker compose` definition under the benchmark harness's `bench/`
  area so it stays consistent with `storage-benchmark-prototype.md`; reuse it
  rather than forking a second setup.
- **Start small and correct.** Begin at the laptop-scale smoke tier — enough data
  to be representative, small enough to iterate. Prove query results match across
  candidates before trusting any latency number.
- **Measure the targeted differences.** Run the micro-benchmarks designed in
  `benchmarking-the-differences.md` and the evidence-bundle/correlation queries
  (Q1–Q6), using the measurement protocol in `storage-benchmark-prototype.md`
  (freshness, per-class latency p50/p95/p99, retained size/compression,
  object-store requests, CPU/RSS). Record warm and cold cache separately.
- **Tie each run to a claim.** For every published claim being checked, log the
  local result and mark it *confirmed*, *refuted*, *workload-specific*, or
  *inconclusive at this scale*, with the exact env so it is reproducible: image
  tags, host hardware, dataset config/seed, and the queries run.
- **Promote solid cases into the harness.** Quick ad-hoc local probes are fine for
  fast claim-checking, but once a case is worth keeping, fold it into the runnable
  `parallax-bench` harness so it is reproducible and counts toward the prototype's
  veto on the storage choice. The full reproducible harness stays owned by
  `storage-benchmark-prototype.md`; this section is about getting empirical numbers
  on the table fast.

Hygiene and honesty:

- Keep containers ephemeral and clean them up; do **not** commit Docker images,
  generated datasets, or large result blobs into the repo — gitignore any data
  directories and commit only the Markdown results log (and the small compose/
  scripts).
- A laptop run is not a production verdict. State the scale and hardware, and label
  any single-node laptop result as indicative, not final — the larger tiers and the
  scaled-out runs in the prototype settle the real numbers. Never present a local
  smoke number as a proven general result.

---

# The Decision Questions To Answer

The loop must drive toward an explicit, defensible answer to all of these, in
`verdict-which-to-choose.md`:

1. **Where is GreptimeDB genuinely faster, and why (mechanism)?** Hold this to the
   same evidentiary bar as the reverse.
2. **Where is ClickHouse genuinely faster, and why (mechanism)?**
3. **Can ClickHouse replace GreptimeDB for Parallax?** If not, what specific
   design decision blocks it (e.g. metrics/PromQL nativeness, object-storage
   economics, operational shape)? If yes, at what cost?
4. **Can GreptimeDB replace ClickHouse for Parallax?** If not, what specific
   design decision blocks it (e.g. log/trace analytical maturity, join
   performance, ecosystem)?
5. **Which to choose** — GreptimeDB, ClickHouse, or, only if a mechanism reason
   justifies it, a third system. The language/runtime filter still applies
   (Rust/Go/Zig/C++/C only; no JVM/interpreted), and Rust breaks ties (see
   `AGENTS.md` and the storage evaluation). A third system may only enter the
   conversation if its *design* solves a problem neither of these two can — name
   the mechanism, do not reopen the field on popularity.
6. **Which is the better long-term *investment*** — a distinct question from "faster
   today." Is ClickHouse's raw-speed lead a permanent moat or a depreciating asset?
   Decide on: (a) **closability** — are the gaps engineering/time or architectural
   physics (per the closability test above)? (b) **contributability** — the operator
   invests in Rust, not C++, and AI-assisted contribution favors Rust, so a
   GreptimeDB/DataFusion gap is one *this operator can actually close* whereas a
   ClickHouse advantage is one he can only wait on; (c) **design trajectory + growth
   potential** (Postgres-overtook-MySQL: better-architected-for-the-domain can pass a
   more-mature incumbent); (d) **cost + scalability**, now and projected. Answer it
   unbiased — name the honest risk that the bet depends on sustained contribution.
7. **Does the Parallax-proxy lens change the answer?** Parallax owns ingestion (OTLP/routing/
   conversion), so native-protocol/ingest-ergonomics advantages are neutralized. Re-score on what
   remains (retrieval speed + build-on-top ecosystem + cost/scaling/cardinality). Current standing
   answer: the proxy tilts the default to **ClickHouse**; GreptimeDB stays for the metrics-cardinality /
   self-hosted-1×-S3 / mandatory-auto-rebalance bet. Keep this honest and flip on contrary evidence.
   (See [`platform-fit-and-alternatives.md`](../docs/research/storage/greptimedb-vs-clickhouse/platform-fit-and-alternatives.md).)
8. **Where do grouped errors + metadata live?** Sentry-style grouped errors (fingerprint → first/last
   seen, count, status, assignee) are **mutable, relational, low-volume OLTP** — neither ClickHouse nor
   GreptimeDB handles that well (Sentry uses Postgres + a ClickHouse "replacements consumer" to fake
   mutations). Standing answer: put issue/workflow/metadata state in the **relational metadata store
   already chosen — Turso (default) / Postgres (scale-out fallback)** per `deep-research-parallax.md`
   "Metadata Store" — NOT in the columnar engine; keep the raw firehose + computed aggregates in the
   columnar store; cold tier on object storage. Confirm or refute this split as evidence accrues.

The decision must rest on the design decisions behind each system, so the choice
is the right one to build on the first time.

---

# Closing The Gap — What The Winner Must Implement For Full Parity

The recommended system (currently **GreptimeDB**, on fit) wins most cases but **lacks
some capabilities the other has** that may matter for Parallax. The comparison is not
complete at "X wins on balance" — it must also answer: *to make the winner a clear
winner in **all** cases, what would have to be built into it, and is that an
engineering gap or an architectural one?* Maintain this as a standing deliverable in
`greptimedb-parity-roadmap.md` (and keep it consistent with `verdict-which-to-choose.md`).

For each capability where the rejected system (ClickHouse) is genuinely ahead, specify:

- **The gap, mechanism-level.** What ClickHouse does and *why it is faster/abler*, tied
  to the concrete data structure or code path (e.g. PREWHERE late materialization, 65k
  blocks + LLVM JIT, the `text` posting-list index, typed-subcolumn JSON, projections).
- **What GreptimeDB would implement to close it — against its *actual* structure.** Map
  the fix to GreptimeDB's real internals (mito2 region engine, Parquet SST + Puffin index
  sidecar, DataFusion `=52.x` execution, OpenDAL object store). Name the file/subsystem
  that would change. Be concrete: "bump the DataFusion `RecordBatch` size in
  `SessionConfig`," "add late materialization to the mito2 Parquet reader," "shred JSON
  paths into Parquet subcolumns," not "make it faster."
- **Classify the effort** into one of three tiers, because the operator's decision turns
  on this:
  1. **Tier A — solvable in Parallax today** (schema/app, no engine change): indexing
     `trace_id`/`fingerprint`, Flow pre-aggregation, choosing SQL over PromQL for hot
     aggregations. These close the gaps that matter for Parallax's *anchored* workload now.
  2. **Tier B — upstream engine work (contributable, since GreptimeDB + DataFusion are
     open-source Rust)**: batch size, expression/aggregation JIT, SIMD kernels, PREWHERE,
     JSON shredding, projection-equivalent alternate ordering, index↔scan fusion. These are
     what "clear winner for *all* cases (incl. heavy ad-hoc log/scan analytics)" requires.
     State whether each is already on the DataFusion roadmap vs a GreptimeDB-specific build.
  3. **Tier C — accept or wait**: distributed/analytical maturity that only time and
     battle-testing close.
- **Whether it is a *design* gap or an *integration* gap.** The load-bearing finding so
  far: GreptimeDB's index *toolkit is richer* (FST+roaring inverted, tantivy full-text),
  so its losses are **execution-integration**, not architecture — closable by engineering,
  not redesign. Keep testing whether each gap is integration-level (good news) or a true
  architectural limit (which would weaken the recommendation).

## The closability test — is any gap a physics wall, or only time-on-task?

The operator's framing: some gaps are *fundamental* (like Singapore↔US network latency —
longer than Singapore↔China by pure geography, unimprovable no matter the engineering),
and some are *merely a matter of effort* (a decade of hand-tuning the other side has
already paid for). The decision to bet on GreptimeDB turns on which kind each gap is.
For **every** gap in the roadmap, assign a closability verdict — do not leave it implicit:

- **Engineering-closable** — same architectural model (vectorized columnar over Arrow),
  the rejected system is just further along the *same* curve. Examples found so far: scan/
  agg throughput (batch size + JIT + SIMD), PREWHERE late materialization, join-pushdown,
  JSON shredding, broad-term scan-index fusion. These are "someone has to write the Rust,"
  not "it cannot be done." State *who* would write it (Tier A Parallax / Tier B upstream)
  and whether it is **already on the DataFusion roadmap** (shared-ecosystem leverage — the
  contribution benefits every Arrow engine, not a private fork).
- **Architecturally fundamental** — a real wall that the current design cannot cross
  without a redesign (the "physics" bucket). The honest near-candidate is the
  PK=sort-key=series-key conflation behind cold *selective* egress; record whether a
  mitigation (e.g. partition-by-`trace_id`, Run 88 cut it to ~10×) defuses it to
  "engineering" or whether a residue is truly structural. If you ever find a gap that is
  genuinely fundamental, say so plainly — that is the single finding that would flip the
  recommendation to ClickHouse.
- **Time-only (Tier C)** — not engineering, not physics: maturity, battle-testing,
  ecosystem depth. Closes on a calendar, not a PR. Name it as such; do not pretend code
  closes it.

The standing conclusion to keep testing (and to overturn if a run disproves it):
**the gaps are engineering or time, not physics.** That is what makes GreptimeDB an
investable long-term substrate rather than a permanent runner-up.

## The long-term-investment decision (Rust-contributable vs C++-mature)

Beyond "which is faster today," answer the operator's actual question: **which engine is
the better thing to invest in for the next several years?** This is a distinct deliverable
from the fit verdict — keep it in `verdict-which-to-choose.md` and grounded in the
roadmap's closability verdicts. Weigh, explicitly and unbiased (do not cheerlead either):

- **Closability of the speed gap** (from the test above): if the gaps are engineering/time
  not physics, the faster-now incumbent's lead is a depreciating asset, not a moat.
- **Who can actually move the engine.** The operator will invest in **Rust** and will
  **not** invest in **C++**. GreptimeDB + DataFusion are Rust and open — the operator (and
  AI-assisted contribution, which is markedly stronger at Rust than at C++; cf. Bun's
  Zig→Rust move for performance + maintainability) can land PRs. ClickHouse's C++ engine is
  contributable in principle but not by *this* operator in practice. A gap you can close is
  categorically different from a gap you can only wait on.
- **Design trajectory / growth potential.** Judge the *direction*, not just the snapshot:
  observability-native (metrics+logs+traces one engine), object-store-native economics,
  horizontal-scale-designed-in, cardinality-insensitive ingest, Arrow/DataFusion
  extensibility. The Postgres-overtook-MySQL precedent: the better-architected-for-the-
  domain system can pass a more-mature incumbent once effort compounds.
- **Cost-effectiveness, now and projected.** Object-store tiering vs local-NVMe replicas;
  ingest cost under high cardinality; operational surface at small (single-node startup)
  and large (horizontal) scale — per [[scaling-trajectory]], small→large is a topology
  change, not a rewrite.
- **The honest risk of the bet.** ClickHouse is faster now, more mature, and has momentum;
  betting on GreptimeDB assumes sustained contribution (operator + community) actually
  closes the engineering gaps. If that investment does not materialize, the raw-speed gap
  persists (though never the observability-native *fit* gap). State this plainly.

## How to write `greptimedb-parity-roadmap.md` (the format this loop must keep)

This is a **dedicated, standalone file** — the one place that answers "what can GreptimeDB
improve, why, and how." It must be **detailed, specific, and code-oriented**, not a
one-line table. Keep a short summary table at the top for scanning, but **every
improvement gets its own full section** with this structure:

- **Borrowed concept (from which system).** Name the *concept* the improvement borrows —
  almost always something ClickHouse already does (PREWHERE late materialization, 65k-row
  blocks, LLVM-JIT expression/aggregation, the `text` posting-list scan integration,
  typed-subcolumn JSON, projections). State it as a portable idea, not "copy ClickHouse":
  *what is the mechanism that makes it work, and is it general (Parquet/Arrow/DataFusion)
  or ClickHouse-specific?* If a concept comes from a third system or a paper, say so.
- **What** — the concrete capability to add to GreptimeDB, in one sentence.
- **Why** — the mechanism gap and the measured/estimated impact, tied to an axis and a
  Parallax query shape. Quote the run number.
- **How — code-oriented and specific.** Map it onto GreptimeDB's *actual* structure: name
  the crate/file/function that changes (`src/mito2/...`, `src/query/...`), the existing
  primitive to wire in (e.g. arrow `RowFilter`, a DataFusion `SessionConfig` field, a new
  `cache/index` member), and the concrete steps. Cite the source you read (file + the
  pinned version). "Add `with_row_filter` to the mito2 reader" beats "make it faster."
- **Tier (A/B/C)** and **design-vs-integration**, as above.
- **User story & clear-winner test (first-class — lead the value with this).** The reason to
  add anything is **solving a real Parallax user's problem the smartest way** — never "parity
  for its own sake." So every improvement must answer, from the *user's* perspective:
  - **Who and when:** a concrete Parallax user story — *who* (the developer/SRE/AI agent
    debugging an incident, the dashboard viewer, the user running an ad-hoc query) and the
    *specific moment* the gap bites (e.g. "an SRE paged at 2am greps logs for a request-id
    across a service over the last hour"). Name the signal and query shape.
  - **Does it make GreptimeDB the clear winner here?** State it explicitly: "by adding this,
    GreptimeDB clearly wins the *<case>* case because …" — **and be honest when the answer is
    no** ("invisible to users / second-order / the anchored hot path is already fast, so this
    is a footnote, don't invest"). Most improvements only matter if real usage proves a
    log-search-/scan-heavy mix; say so.
  - **Huge now + future improvement:** where it delivers a large immediate win, and how it
    compounds (sets up later capability), if at all.

When brainstorming a new improvement, always start from "what does the other system do as a
*concept*, and how would that concept land in GreptimeDB's mito2/DataFusion/Puffin/OpenDAL
structure to provide value here" — borrow the idea, adapt it to the real internals, prove
it is integration not redesign (or flag it if it is redesign). Then **gate it on the user
story**: if no real Parallax user case is materially better for it, mark it a footnote, not a
roadmap priority. The deliverable ranks improvements by *user impact*, not by mechanism
elegance.

The point is decision-useful: tell the operator exactly what it costs to make GreptimeDB
the unambiguous choice for every Parallax query shape, and which of those costs they pay
in their own schema vs. by contributing upstream. (A hybrid GreptimeDB+ClickHouse split is
the alternative to closing the gap — evaluate it honestly, but note it splits Parallax's
cross-signal correlation hot path across two engines, its biggest cost.)

---

# Evaluation Axes (Priority Order)

These axes are proxies for one thing: **the real user's experience** on the system
Parallax ships — is it fast, does ingest keep up, does every signal stay queryable
under load (see "Benchmark Like Production Over Scoreboard Optics"). Judge every
mechanism by its consequence on these axes, in this order:

1. **Speed — time to see real data.** Ingest-to-queryable freshness, and query
   latency for the evidence-bundle/correlation patterns under concurrent
   ingest+query — not generic scans.
2. **Cost — money to run, co-equal with speed (operator, 2026-05-25).** Not just
   storage size — the **full run cost**, weighed *against* speed in the final
   decision ("X ms at $Y/GB-mo on Z compute"). Measure the four components: (a)
   **$/GB retained** = per-signal on-disk size × storage-class price (S3 ~$0.023/GB
   ≈ 3.5× cheaper than EBS gp3 ~$0.08/GB) × **replication (1× GreptimeDB vs N× OSS
   ClickHouse)**; (b) **compute footprint for the SLA** (always-on ClickHouse hot tier
   vs elastic near-stateless GreptimeDB); (c) **cold-read cost** (S3 GET/egress on
   history re-reads); (d) **operational** (Keeper/resharding vs metasrv/Kafka, or the
   managed-cloud premium). The storage-architecture thesis (`storage-cost-and-tiering.md`):
   **ClickHouse optimizes time-to-answer on hot data (performance-first, local/premium
   storage); GreptimeDB optimizes $/GB on deep data (S3-native, store-everything-cheaply).**
3. **Scaling — horizontal first.** Single-node ceiling, scale-out difficulty, and
   whether performance holds as parallel servers are added. Horizontal scaling is
   primary; vertical-only is a flagged limitation.

Tie every internals finding back to one of these axes; an interesting mechanism
that does not move speed, cost, or scaling for Parallax's workload is a footnote,
not a headline.

---

# Per-Pass Loop

Each pass:

1. Re-read this prompt and the current state of
   `docs/research/storage/greptimedb-vs-clickhouse/`.
2. **Re-verify before deepening — treat every existing comparison statement as a
   theory, not a settled fact.** Re-pin the versions (Method #1). Then re-check the
   most load-bearing, most-suspicious, or stalest existing claims against the *live*
   containers and the *current* source — especially any "X is faster / abler"
   headline and anything the verdict leans on. Rotate the slice each pass so that,
   over successive passes, the **entire record stays continuously re-verified**, not
   merely appended to. Any claim that fails to reproduce is corrected *immediately*,
   in the same pass, with the run that falsified it logged under the Reproducibility
   Contract. A number that no longer reproduces is a top-priority finding.
3. Pick the single highest-value unanswered (or weakest, or most stale) internals
   question.
4. Confirm/refresh the pinned versions; read the relevant design docs and the
   source code for that subsystem in both systems.
5. Where the question is empirical, run it on the live Docker stack to the
   production-realism + reproducibility standard above, and log the run so the
   operator can re-run it by hand.
6. Write or revise one focused note with mechanism-level findings, file/commit
   citations, and the scenario/axis consequence.
7. Update the subfolder `README.md` (index + status), and update this prompt,
   `prompts/README.md`, and `PROJECT_STRUCTURE.md` when durable direction or
   repository shape changes.
8. Commit and push the durable change.
9. Move to the next highest-value gap. Do not declare the comparison complete;
   keep deepening *and* re-verifying until the operator stops the loop.

---

# Required Final Deliverable

A standing, continually-sharpened `verdict-which-to-choose.md` that states:

- the per-signal scenario matrix with mechanism-level reasons;
- the honest test of the operator hypothesis (is GreptimeDB actually fastest,
  then ClickHouse — and where not, why, at the design-decision level);
- the two replaceability answers (each direction), grounded in design;
- a single recommended choice for Parallax with the tradeoffs and the rejected
  alternative made explicit;
- the open questions that only a benchmark run can settle, handed to
  `storage-benchmark-prototype.md`.

The bar: someone could read the verdict and the supporting notes and understand
not just *which* database to build Parallax on, but *why* — down to the data
structures and code paths that decide it.
