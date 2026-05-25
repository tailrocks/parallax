# Dedup and Update Semantics — Latest-State Queries

<!-- markdownlint-disable MD013 -->

Status: pass 39, re-verified pass 96 (Run 59 — reproduces; partial-upsert loss proven) + **Run 163
(live exec re-verify, no drift): CH `ReplacingMergeTree(ts) ORDER BY k` → plain SELECT shows **2 dup
rows** until `FINAL` (→1, latest); GreptimeDB exact-`(PK,ts)`-dup → plain SELECT **1 row at READ** (no
FINAL), and latest-state-per-key via `last_value(v ORDER BY ts)`. **Clarified the dedup-MODEL distinction
(not just timing):** ClickHouse `ReplacingMergeTree` is an **upsert table** — one row per `ORDER BY`
key, latest version, deduped *eventually* at merge/`FINAL`; GreptimeDB is a **time series** — keeps
`(PK, ts)` points, deduping only *exact* `(PK,ts)` duplicates at read, with latest-state read via
`last_value`. So "latest state" (issue status / metric last-value / deploy marker) is correct-by-default
at read on GreptimeDB but needs `FINAL`/`argMax` on ClickHouse — reinforcing the metadata-store split
(mutable workflow state → relational store, `platform-fit-and-alternatives.md`)).
White-box teardown of how each engine handles **duplicate keys and
updates** — when a row with an existing key is overwritten, and what a query sees.
Decision-relevant because several Parallax signals are *upsert-shaped*: the **current
status of an issue/fingerprint** (Q2 issue-history wants the latest), **deploy
markers** (one row per release, updated), and **metric last-value** semantics. The
sharp question: is dedup applied **at read** (always correct) or **only at merge**
(duplicates visible until then)? Source-confirmed and measured live (Run 19).

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`),
re-confirmed latest stable 2026-05-25.

## GreptimeDB — dedup is read-time (always correct)

Dedup lives in the **read path**: `src/mito2/src/read/dedup.rs` defines
`DedupReader<R, S>`, a reader that wraps the scan's merge-sorted stream and collapses
rows with the same `(primary key, time index)` on the fly. Because it is part of the
read iterator stack, **every query sees the deduped result** — no special keyword, no
dependence on compaction having run. It piggybacks on the merge-sort the LSM read
already does to combine memtables + SSTs by key+time, so dedup is near-free on top of
that ordering.

Two strategies (`DedupStrategy`), selected by the table's `merge_mode`:

- **`LastRow`** (default, `dedup.rs:142`): the row with the highest sequence for a
  `(key, ts)` wins — last write replaces.
- **`LastNonNull`** (`dedup.rs:487`, `merge_last_non_null` at :420): per-field, the
  last *non-null* value wins — so partial upserts merge. This is the upsert-merge
  metrics/state want (different writes set different fields).
- **`filter_deleted`** (`dedup.rs:147,236`): delete markers are honored during the same
  read pass — a deleted `(key, ts)` is filtered out at read.

**Opt-out: `append_mode='true'`** disables dedup entirely (no `DedupReader` work) for
append-only signals (logs/spans/traces) — those have no duplicate keys, so paying for
dedup would be waste. Parallax's seed DDL already sets `append_mode` on the
high-volume append tables and leaves dedup on for the upsert tables.

**Measured (Run 19):** `(k='A', ts=1000)` inserted twice (v=1 then v=2) → plain
`SELECT` returns **one row, v=2** (no compaction forced). With
`merge_mode='last_non_null'`, two partial writes (`v1=1`, then `v2=2` at same key/ts)
→ **one row, `v1=1 AND v2=2`** merged. Correct immediately, no keyword.

## ClickHouse — ReplacingMergeTree dedups at merge (eventual)

`ReplacingMergeTree(version)` keeps, per `ORDER BY` key, the row with the max
`version`. But the collapsing is done by a **merge algorithm**
(`src/Processors/Merges/Algorithms/ReplacingSortedAlgorithm.cpp`) that runs **only
during background merges or when `FINAL` is applied** — never automatically at plain
read time. Consequences:

- A plain `SELECT` (no `FINAL`) **returns duplicate keys** until a background merge
  happens to collapse them — and merges only combine *some* parts, so there is **no
  guarantee** all duplicates are gone at any given moment.
- **`SELECT … FINAL`** forces the ReplacingSorted merge at read time → correct now, but
  it must read and merge all parts covering each key (cost grows with the number of
  un-merged covering parts × rows).
- Soft deletes: `ReplacingMergeTree(version, is_deleted)` + `cleanup` drop a row marked
  deleted (`ReplacingSortedAlgorithm.cpp:58` honors `is_deleted_column`), but only at
  merge/FINAL — same eventual semantics.

**Measured (Run 19, re-verified Run 59):** two inserts of `(k='a', ts)` (v=10 then
v=20) = two parts. Plain `SELECT` returned **both** rows (10 and 20); `SELECT … FINAL`
returned **one** (v=20 wins); after `OPTIMIZE TABLE … FINAL` the plain `SELECT`
collapsed to one — **reproduces unchanged**.

**Partial-upsert loss proven (Run 59) — the sharp capability gap.** Two partial writes
to the same key, `('x', a=10, b=NULL)` then `('x', a=NULL, b='hello')`:

- **GreptimeDB `last_non_null`** plain `SELECT` → **`a=10, b='hello'`** (per-field merge).
- **ClickHouse `ReplacingMergeTree` `… FINAL`** → **`a=NULL, b='hello'`** — the **`a=10`
  is LOST.** RMT keeps the *last whole row*, not a per-field merge, so a field set only in
  the earlier insert is discarded. To get GreptimeDB's per-field result, ClickHouse needs
  `AggregatingMergeTree` with an `argMax(col, ts)`-per-column schema (explicit `-State`
  columns + a materialized view) — real ceremony, not a table option.

This matters for Parallax signals updated by *multiple partial events* (an issue row whose
status, assignee, and last-seen arrive from different writes; a span enriched by
late-arriving attributes): GreptimeDB merges them with one `merge_mode='last_non_null'`
option; ClickHouse RMT silently drops the un-latest fields.

**Fairness note:** modern ClickHouse has made `FINAL` much cheaper (parallel final,
skipping already-merged parts, `do_not_merge_across_partitions_select_final`), and the
idiomatic "latest state" pattern is often **`argMax()` / `GROUP BY`** or
`AggregatingMergeTree`, not ReplacingMergeTree+FINAL. So this is not "FINAL is
catastrophic" — it is that **correctness-now is opt-in and carries a cost/skill
burden**, where GreptimeDB makes it the default.

## Side by side

| | GreptimeDB | ClickHouse (ReplacingMergeTree) |
| --- | --- | --- |
| When dedup applies | **Every read** (DedupReader in scan stack) | **Merge time / `FINAL` only** |
| Plain query correctness | **Always deduped** | **May show duplicates** until merged |
| Force-correct-now | nothing needed | `SELECT … FINAL` (read-time merge cost) |
| Upsert-merge of partial rows | **`last_non_null`** (per-field) | not native — needs `AggregatingMergeTree` / `argMax` |
| Delete handling | `filter_deleted` at read | `is_deleted` + `cleanup` at merge/FINAL |
| Turn dedup off (append-only) | `append_mode='true'` | use plain `MergeTree` |
| Cost shape | dedup piggybacks LSM read-merge (near-free) | eventual merges cheap; `FINAL` cost ∝ covering parts |

## Parallax implication and axis consequence

- **Upsert/latest-state reads** (current issue status, deploy marker, metric
  last-value): GreptimeDB returns the correct latest row on a **plain query**, for
  free, via read-time dedup — and `last_non_null` natively merges partial updates.
  ClickHouse must use `FINAL` (cost) or restructure to `argMax()/GROUP BY` /
  `AggregatingMergeTree`. **Ergonomics + correctness edge: GreptimeDB**, for Parallax's
  upsert-shaped signals.
- **Append-only signals** (logs/spans/traces): no duplicate keys, so dedup is moot —
  GreptimeDB `append_mode` skips it, ClickHouse uses plain `MergeTree`. **Tie.**
- **Axis:** primarily **speed/correctness of latest-state queries** (axis #1) plus an
  operational-fit factor. It does **not** change the raw-scan or log-search verdicts;
  it sharpens the *evidence-bundle* picture: the issue/fingerprint-history sub-query
  (Q2) is upsert-shaped, and GreptimeDB answers "latest status" correctly without
  `FINAL` while ClickHouse needs the extra construct. Reinforces, not flips, the
  standing verdict — but it is a concrete GreptimeDB ergonomics win on a real Parallax
  query.

## Honest caveats

- Smoke scale. The `FINAL`-cost-vs-read-dedup-cost crossover at millions of rows with
  many un-merged parts is **not measured** — that is where ClickHouse `FINAL` could
  bite and where GreptimeDB's per-scan dedup also costs CPU; owed to the harness.
- GreptimeDB read-time dedup is not free on huge scans of high-duplicate data (it is a
  merge pass); `append_mode` exists precisely to avoid it where unneeded.
- ClickHouse's faster modern `FINAL` and the `argMax`/`AggregatingMergeTree` idioms
  mean a skilled ClickHouse user gets correct latest-state acceptably — the gap is
  *default ergonomics*, not capability.

## Source / evidence

- GreptimeDB: `src/mito2/src/read/dedup.rs` — `DedupReader` (read-path), `LastRow:142`,
  `LastNonNull:487` + `merge_last_non_null:420`, `filter_deleted:147`;
  `append_mode` opt-out (`store-api` region options); `merge_mode_test.rs`.
- ClickHouse: `src/Processors/Merges/Algorithms/ReplacingSortedAlgorithm.cpp`
  (merge-time dedup; `is_deleted`/`version`/`cleanup` at :37–68); `FINAL` applies the
  same algorithm at read.
- Live: `local-benchmark-results.md` Run 19, Run 59 (re-verified; partial-upsert loss proven — RMT `FINAL` returns `a=NULL` vs GreptimeDB `last_non_null` `a=10`).
- Cross-refs: `write-path-and-ingestion.md` (append_mode, schema-on-write
  `merge_mode='last_non_null'`), `compaction-and-merge.md`, `per-signal-verdict.md`
  (Q2 issue-history).
