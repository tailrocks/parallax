# Projections and Multiple Access Paths

<!-- markdownlint-disable MD013 -->

Status: pass 50. ClickHouse **projections** (a ClickHouse system-lead, not yet
dissected — pass 27 covered MVs + AggregatingMergeTree, not projections) and the
question they answer: *how do you serve more than one access path from a single
table?* This recurs throughout the loop — ClickHouse `ORDER BY (trace_id, ts)` wins
trace lookups but a `(service, ts)` scan wants a different order; GreptimeDB's seed PK
`(service, name)` leaves `trace_id` un-keyed (Run 1/6). Source + live (Run 28).

Pins: GreptimeDB `v1.0.2` (`0ef5451`), ClickHouse `v26.5.1.882-stable` (`5b96a8d8`),
re-confirmed latest stable 2026-05-25.

## ClickHouse — projections: a second physical ordering inside the part

A **projection** stores the table's data **again, in a different `ORDER BY`** (a
*normal* projection) or **pre-aggregated** (an *aggregate* projection), as a hidden
sub-part **inside every part**, maintained automatically on insert/merge. The query
optimizer **transparently** reads the projection when it serves the query better — no
separate table, no app-side routing.

**Live (Run 28):** `CREATE TABLE proj_test … ORDER BY (trace_id, ts), PROJECTION
p_service (SELECT * ORDER BY service)`, 500k rows. `EXPLAIN indexes=1` for `WHERE
service='svc5'` →

```
ReadFromMergeTree (p_service)
```

— the optimizer chose the **projection**, not the base `(trace_id, ts)` data. So one
`proj_test` serves **both** access paths: `trace_id`-anchored lookups (base order) and
`service`-filtered scans (projection), auto-selected.

**Cost — it ~doubles storage.** `system.parts` total = **4.07 MiB**;
`system.projection_parts` = **2.07 MiB** → the projection is a near-full second copy
(base data ~2 MiB + projection ~2 MiB). Each *normal* projection adds ~1× the data
size and is rewritten on every merge. Aggregate projections are smaller (store only the
rollup), behaving like a per-part materialized view.

## GreptimeDB — no projections; secondary indexes instead

GreptimeDB has **no projection feature** — the SQL parser rejects `PROJECTION`
outright (Run 28: *"Cannot use keyword 'PROJECTION' as column name"*). Its answer to
"serve another access path" is the **secondary index stack** (`indexing-internals.md`):
an **inverted index** on `trace_id` (FST + roaring) gives the anchored lookup without
reordering the data; skipping/fulltext indexes accelerate other predicates. The data
stays in one physical order (PK + time); indexes provide **row positions**, not a
second physical layout.

## Side by side

| | GreptimeDB | ClickHouse |
| --- | --- | --- |
| Mechanism for a 2nd access path | **secondary index** (inverted/skipping/fulltext) | **projection** (full alternate `ORDER BY`, or aggregate) |
| Physical layout | one (PK + time); index = row positions | base **+ a second physical copy** per normal projection |
| Storage cost | ~index size (small vs data) | **~+100% per normal projection** (measured 2→4 MiB) |
| Best for | **point / filter** on an alternate column (anchored) | **sequential scan** on an alternate ordering |
| Selected by | planner uses the index when predicate matches | optimizer transparently picks the projection |
| Maintained | index built at flush/compaction | projection rewritten on insert/merge |

## The decision-relevant tradeoff

The two mechanisms optimize different alternate-access shapes:

- **Anchored point/filter (Parallax's dominant pattern):** look up by
  `trace_id`/`fingerprint`. GreptimeDB's **inverted index** does this at **index size**
  (no data copy); a ClickHouse **projection** also does it but pays a **full second
  copy**. So for the *anchored* shape Parallax actually runs, GreptimeDB's index is the
  **more storage-efficient** mechanism, and ClickHouse's cheaper option is just making
  `trace_id` the base `ORDER BY` prefix (no projection needed).
- **Scan by an alternate ordering** (e.g. "stream all spans ordered by `service, ts`"):
  here a ClickHouse **projection gives true sequential locality** that an index cannot
  (an index yields scattered row positions, not a contiguous scan). Parallax does this
  *less* (its reads are anchored), but where it matters, projections are a real
  ClickHouse capability with no GreptimeDB equivalent — at the ~2× storage cost.

So projections are a genuine ClickHouse strength for **multi-ordering scan workloads**,
bought with storage; GreptimeDB's index model is leaner for **anchored multi-access**,
which is Parallax's shape. Neither flips the verdict; it sharpens *why* the engines
feel different when you need a second access path.

## Axis consequence

- **Speed (axis #1):** projections let ClickHouse serve scan-heavy alternate orderings
  fast from one table (optimizer-transparent); GreptimeDB matches the *anchored*
  alternate access via inverted index but cannot give a second *physical scan order*.
  For Parallax's anchored reads this is a wash (both fast); for hypothetical
  scan-by-service workloads ClickHouse projections win.
- **Cost (axis #2):** each normal projection **~doubles** the table's storage
  (measured) — material for Parallax's retention-heavy store; GreptimeDB's inverted
  index is far cheaper than a full projection copy. So "use projections for every
  access path" is a real cost on the cost axis, not free.
- **Net:** a ClickHouse capability advantage for multi-ordering *scans*, offset by a
  storage cost; GreptimeDB's leaner index model fits Parallax's anchored pattern. Adds
  nuance to the read-path verdict, doesn't move it.

## Honest caveats

- Smoke scale (500k rows); the projection storage-doubling is measured but
  scan-latency-with-projection vs index-lookup at GB scale is not (a
  `benchmarking-the-differences.md` candidate if Parallax adds scan-by-alternate-order
  queries).
- Aggregate projections (pre-agg) overlap with the MV/AggregatingMergeTree rollup story
  (`rollup-and-continuous-aggregation.md`) — same "precompute inside the engine" idea,
  per-part; not separately measured here.
- Projections add insert/merge write-amplification (each merge rewrites them) — another
  cost beyond storage, consistent with the merge write-amp theme.

## Source / evidence

- ClickHouse: `PROJECTION` DDL; `system.parts` (4.07 MiB total) vs
  `system.projection_parts` (2.07 MiB) — Run 28; `EXPLAIN indexes=1` →
  `ReadFromMergeTree (p_service)` (optimizer picks the projection).
- GreptimeDB: no `PROJECTION` (parser rejects, Run 28); secondary-index model in
  `indexing-internals.md`.
- Cross-refs: `read-path-indexing-and-execution.md` (ORDER BY / skip),
  `indexing-internals.md` (GT inverted index vs CH sort-key/projection),
  `rollup-and-continuous-aggregation.md` (aggregate projections vs MV),
  `compression-and-cost.md` (the storage cost of a second copy).
