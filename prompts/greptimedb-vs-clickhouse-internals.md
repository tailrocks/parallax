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

- [`docs/research/greptimedb-storage-evaluation.md`](../docs/research/greptimedb-storage-evaluation.md)
  is a strategy/fit evaluation (open source, maturity, metrics-native posture,
  risks). It reasons *about* the systems, not *inside* them.
- [`docs/research/observability-storage-benchmark-plan.md`](../docs/research/observability-storage-benchmark-plan.md)
  defines *what to measure and why*.
- [`docs/research/storage-benchmark-prototype.md`](../docs/research/storage-benchmark-prototype.md)
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

This brief is meant to run indefinitely. It never self-completes — it runs
deepening passes until the operator stops it by hand. Treat it as continuous: do
not converge on a single deliverable and stop.

Run passes back to back (nothing external is being watched, so there is no reason
to idle between them). Each pass picks the single highest-value unanswered
internals question, researches it against primary sources and source code, writes
or revises one focused note under `docs/research/greptimedb-vs-clickhouse/`, then
commits and pushes per [`AGENTS.md`](../AGENTS.md). Do not declare the comparison
"done" just because the
output files exist; keep deepening until told to stop.

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

---

# Output Location And File Plan

All output goes under a dedicated subfolder:

```text
docs/research/greptimedb-vs-clickhouse/
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

# Verify Claims Locally With Docker (Measure, Do Not Just Reason)

A local Docker environment is available, so do not stop at reasoning from the code
— actually stand each system up and measure. Reading the source tells you *why* a
result should happen; a local run tells you *whether* it does. Run real
comparisons, record real numbers, and use them to confirm or refute the published
performance claims for each system. Capture every run in
`local-benchmark-results.md`.

How to run it:

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

The decision must rest on the design decisions behind each system, so the choice
is the right one to build on the first time.

---

# Evaluation Axes (Priority Order)

Judge every mechanism by its consequence on these axes, in this order:

1. **Speed — time to see real data.** Ingest-to-queryable freshness, and query
   latency for the evidence-bundle/correlation patterns under concurrent
   ingest+query — not generic scans.
2. **Cost — storage size and money.** Retained size and compression by signal,
   object-vs-local economics, compute per ingested GB and per query class.
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
   `docs/research/greptimedb-vs-clickhouse/`.
2. Pick the single highest-value unanswered (or weakest, or most stale) internals
   question.
3. Confirm/refresh the pinned versions; read the relevant design docs and the
   source code for that subsystem in both systems.
4. Write or revise one focused note with mechanism-level findings, file/commit
   citations, and the scenario/axis consequence.
5. Update the subfolder `README.md` (index + status), and update this prompt,
   `prompts/README.md`, and `PROJECT_STRUCTURE.md` when durable direction or
   repository shape changes.
6. Commit and push the durable change.
7. Move to the next highest-value gap. Do not declare the comparison complete;
   keep deepening until the operator stops the loop.

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
