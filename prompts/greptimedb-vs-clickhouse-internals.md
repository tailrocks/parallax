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

This prompt is durable operator intent for `/loop` runs, not a disposable input.
When the operator clarifies the comparison target, names a new subsystem to
dissect, changes the evaluation criteria, pins a different version, or confirms a
finding, update this file in the same change if a future run would otherwise use
stale instructions. Do not keep important direction changes only in chat or only
in the generated notes.

---

# How To Run

This is a `/loop` brief. It never self-completes — it runs deepening passes until
the operator stops it by hand.

```text
/loop prompts/greptimedb-vs-clickhouse-internals.md
```

Self-paced (no interval): the agent starts the next pass as soon as one finishes,
because nothing external is being watched. Each pass picks the single
highest-value unanswered internals question, researches it against primary
sources and source code, writes or revises one focused note under
`docs/research/greptimedb-vs-clickhouse/`, then commits and pushes per
[`AGENTS.md`](../AGENTS.md). Do not declare the comparison "done" just because the
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
- `per-signal-verdict.md` — the scenario matrix: metrics vs logs vs traces vs
  evidence-bundle correlation, who wins each and the mechanism why.
- `verdict-which-to-choose.md` — the final synthesized decision (see below).

Keep each note source-linked and concise. Prefer comparison tables plus short
mechanism analysis, per the repo research conventions.

---

# Method: Read The Source, Not The Marketing

1. **Pin versions first, every pass.** Compare the latest reasonably available
   stable release of each system as of the run date. Record the exact version and
   the source commit SHA you read in every note. Starting pins (re-check and bump
   at run time): GreptimeDB `v1.0.2` (GA 2026-05-14), ClickHouse latest stable
   (`25.x` — pin the exact patch). Do not analyze an old major against a current
   one unless the point is explicitly historical.

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

4. **Predict, then check against the benchmark.** Where the runnable benchmark
   (`storage-benchmark-prototype.md`) has produced numbers, the internals analysis
   must explain them. Where it has not, state the internals-based prediction so a
   later benchmark run can confirm or falsify it.

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
