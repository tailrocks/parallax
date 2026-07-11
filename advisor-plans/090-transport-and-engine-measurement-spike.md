# Plan 090: SPIKE — measure the read transport and engine defaults: Arrow vs JSON results, wire-protocol prepared statements, RANGE queries, 16-partition trace table

> **Executor instructions**: This is a MEASUREMENT SPIKE — the deliverable is
> a research note plus a go/no-go recommendation, NOT production code. Any
> code written lives under `poc/` or a scratch branch of throwaway harness
> code inside the note's instructions. Follow the steps; if a STOP condition
> occurs, stop and report. When done, update the status row in
> `advisor-plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat df81d86..HEAD -- crates/parallax-storage/src/greptime.rs`
> This spike only READS the repo; drift matters only for the query inventory
> in Step 1.

## Status

- **Status**: **DONE** (2026-07-11)
- **Priority**: P3
- **Effort**: M
- **Risk**: LOW (read-only spike; the risk it manages is adopting a transport
  migration without evidence)
- **Depends on**: 084 (its corrections change the query mix — measure after),
  085 recommended (measure the post-rewrite queries, not doomed ones)
- **Category**: perf / direction
- **Planned at**: commit `df81d86`, 2026-07-10
- **Deliverables**:
  - [`docs/research/storage/read-transport-and-engine-defaults.md`](../docs/research/storage/read-transport-and-engine-defaults.md)
  - [`poc/read-transport-bench/`](../poc/read-transport-bench/)
  - GO follow-up: [091](091-http-arrow-zstd-read-path.md) (HTTP arrow+zstd)
- **Verdicts**: arrow+zstd **GO**; MySQL prepared **NO-GO**; RANGE **NO-GO**;
  partition hint **REVISIT-AT-SCALE**

## Why this matters

All Parallax reads go over GreptimeDB's HTTP `/v1/sql` returning JSON. The
2026-07-10 audit verified against docs + the v1.1.2 source that (a) the same
endpoint serves `format=arrow` (Arrow IPC, optional `compression=zstd|lz4`),
(b) the MySQL (:24002) and PostgreSQL (:24003) wires support prepared
statements with SESSION-CACHED LOGICAL PLANS while HTTP re-parses and re-plans
every statement, (c) GreptimeDB's docs recommend "mature SQL drivers" for
programmatic reads, and (d) the auto-created `opentelemetry_traces` table is
partitioned 16-ways by `trace_id` — a distributed-scale default running on
one laptop (`trace_table_partitions` hint can shrink it). Each of these is a
POSSIBLE win of unknown size. The wrong move is a big transport migration on
faith; this spike buys the numbers.

## Current state

- `crates/parallax-storage/src/greptime.rs:408-440` — `sql()`: form-encoded
  POST, `serde_json::Value` parse, rows cloned out of the response tree.
- Heaviest read shapes (post-074/075/085 inventory to confirm in Step 1):
  `select_spans` (`SELECT *` by trace_id, wide auto-widened schema),
  `logs_search` (500-row pages with two JSON columns), `traces_search`
  (window + ROW_NUMBER + join), metric series (`date_bin` GROUP BY).
- Engine facts (all verified 2026-07-10, engine 1.1.0/1.1.2):
  - `format=arrow` + `compression=zstd` on `/v1/sql` (docs.greptime.com/user-guide/protocols/http).
    Server buffers the full result either way (no streaming; upstream
    `arrow_result.rs` collects to `Vec<u8>`).
  - MySQL wire: `on_prepare`/`on_execute`, per-session
    `HashMap<String, SqlPlan>` — plan built once at prepare
    (upstream `src/servers/src/mysql/handler.rs`). Postgres wire: pgwire
    extended query protocol, same plan caching (`postgres/handler.rs`).
  - RANGE queries (`agg RANGE '5m' … ALIGN '5m'`) are the engine-native
    time-bucket syntax; `date_bin`+GROUP BY works and stays supported.
  - `trace_table_partitions` hint (0/1 disables partitioning) exists at
    ≥1.1 (docs hints list); default 16 (`src/sql/src/partition.rs`,
    `DEFAULT_PARTITION_NUM_FOR_TRACES`).
- Repo TLS rule: never enable rustls in any dependency — a MySQL/PG client
  crate must use native-tls or no TLS features (plaintext localhost).
- PoC rule (AGENTS.md): concept-proving code under `poc/`, small, runnable,
  test-covered, supports no product claims.
- Bench data source: the sibling playground repo generates realistic traffic;
  `bench/` holds the four-way benchmark discipline (NOT required here — this
  is a same-engine A/B, not a cross-engine claim; still record engine version
  + dataset size with every number).

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Serve + data | `parallax serve` + playground traffic | UI shows data |
| Row count | `curl -s -XPOST 'http://127.0.0.1:24000/v1/sql?db=public' -d 'sql=SELECT COUNT(*) FROM opentelemetry_traces'` | count ≥ 100k for meaningful numbers |
| Arrow probe | same endpoint + `&format=arrow&compression=zstd` (write to file) | arrow IPC bytes |
| PoC tests | `rtk cargo test` (inside `poc/<name>/`) | pass |

## Scope

**In scope**:
- `poc/read-transport-bench/` (new, self-contained cargo project; excluded
  from the workspace like `poc/evidence-loop`)
- `docs/research/storage/read-transport-and-engine-defaults.md` (the deliverable)
- `advisor-plans/README.md` (status row)

**Out of scope**:
- ANY change under `crates/` or `ui/` — this plan changes no product code.
- Cross-engine comparisons (four-way rule applies only if you compare engines
  — don't).
- Ingest transports (089 covers extension-table writes).

## Git workflow

Direct on `main`; Conventional Commits + `git commit -s` + Claude trailer.
Research note + poc in one or two commits.

## Steps

### Step 1: Freeze the query inventory

From the live greptime.rs (post-084/085 if landed), extract the 6 heaviest
real query texts: spans-by-trace (wide trace), logs page (500 rows), trace
search page, one metric series, one histogram bucket read, service summaries.
Record each verbatim in the research note with its calling surface.

**Verify**: note lists 6 queries with file:line provenance.

### Step 2: Seed a measurable dataset

Playground traffic until `opentelemetry_traces` ≥ ~500k spans and
`opentelemetry_logs` ≥ ~500k rows (or the laptop-tier bench cap — do NOT
build multi-million-row datasets on the dev machine; the repo bench rule's
small-tier floor of 50k is the minimum). Record exact counts + engine version.

**Verify**: counts recorded in the note.

### Step 3: A/B the HTTP formats

In `poc/read-transport-bench`: a small tokio+reqwest harness that runs each
inventory query N=50 times against (a) `format=greptimedb_v1` (current), (b)
`format=arrow`, (c) `format=arrow&compression=zstd`, measuring wall-clock
(client-observed) + response bytes + client-side decode time to comparable
in-memory rows (use the `arrow-ipc` crate for (b)/(c); count rows to prove
parity). Report p50/p95 per query per format.

**Verify**: `rtk cargo test` in the poc (a parity test: same row count across
formats for one query); numbers table in the note.

### Step 4: A/B prepared statements over a wire protocol

Same harness, one wire: pick MySQL (:24002) with a pure-Rust client whose TLS
features can be fully disabled (verify no rustls in `cargo tree` — TLS rule;
plaintext localhost). Prepare each inventory query once (parameterize the
obvious literals: trace_id, window bounds, limit), execute N=50 with fresh
parameters, measure. Compare against Step 3's HTTP numbers for the same
queries. Also record reconnect cost (session plans are per-connection — a
pooled client is the realistic shape).

**Verify**: parity test row counts match HTTP; numbers in the note.

### Step 5: RANGE-query spot check

Rewrite the metric-series inventory query in RANGE/ALIGN form; EXPLAIN ANALYZE
both forms live; record whether the plans/timings differ meaningfully. This is
a 30-minute check, not a benchmark.

**Verify**: both plans captured in the note.

### Step 6: Partition-count check

On a SCRATCH data dir: start serve with the traces forward carrying
`x-greptime-hints: …,trace_table_partitions=1` (hand-patch locally or use an
external curl forward against the OTLP endpoint on a scratch engine — do NOT
commit product-code changes), seed identical playground traffic, run the
trace-search + spans-by-trace inventory queries, compare against the 16-
partition default numbers from Step 3. Also record ingest-side visible
difference if any (region count, memory from the engine's own metrics).

**Verify**: numbers for 1 vs 16 partitions in the note.

### Step 7: Write the recommendation

The note ends with a decision table: for each candidate (arrow format, wire
prepared statements, RANGE syntax, partition hint) — measured delta on the
real query mix, adoption cost (from the audit: arrow = S per endpoint, wire
client = L, RANGE = cosmetic, partition hint = S but only fresh data dirs),
and a GO / NO-GO / REVISIT-AT-SCALE verdict. Follow the repo research-note
conventions (date, sources, comparison table). If any candidate is GO, file
it as a new advisor-plan entry in the index (do not implement here).

**Verify**: note committed; index row updated; recommendation explicit.

## Test plan

- PoC parity tests (row-count equality across transports) — the only tests
  this plan owns.
- No product-code tests change.

## Done criteria

- [x] `docs/research/storage/read-transport-and-engine-defaults.md` exists with: dataset size, engine version, per-query p50/p95 tables for ≥3 transports, partition A/B, decision table
- [x] `poc/read-transport-bench/` builds and its tests pass standalone
- [x] No diffs under `crates/` or `ui/` from this plan (`git status` for staged files)
- [x] `advisor-plans/README.md` status row updated (+ plan 091 for Arrow GO)

## STOP conditions

Stop and report back (do not improvise) if:

- No rustls-free MySQL/PG client crate exists at a current version — record
  the survey in the note and mark Step 4 UNMEASURED rather than bending the
  TLS rule.
- `format=arrow` output cannot be decoded by current `arrow-ipc` (version
  mismatch) — record versions, mark UNMEASURED.
- The laptop cannot hold the Step 2 dataset comfortably (memory pressure) —
  drop to the 50k floor and say so in every table.

## Maintenance notes

- Numbers age with engine versions: the note must pin `SELECT version()`
  output; re-run the harness on major engine bumps (the poc makes that cheap).
- If arrow wins big on `select_spans`/logs pages, the adoption plan should
  keep JSON for small results (schema probes, counts) — decode cost dominates
  only on wide/tall results.
- The partition-hint decision only affects FRESH data dirs — existing tables
  keep their partitioning; note this in any adoption plan.
