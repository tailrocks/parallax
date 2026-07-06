# Plan 033: Capture OTLP metric exemplars end-to-end and jump from a metric spike to the exact trace (design + thin slice)

> **Executor instructions**: This is a **design-anchored** plan. Step 1 is a
> spike that produces a short design note and confirms feasibility; only then
> do the implementation steps. If Step 1 finds the blocking assumption false,
> STOP and report with the design note instead of implementing. Update the
> status row in `plans/README.md` when done.
>
> **Drift check (run first)**: `git diff --stat 8bc3f13..HEAD -- crates/parallax-core/src/normalize.rs crates/parallax-storage/src/greptime.rs crates/parallax-storage/src/model.rs crates/parallax-storage/src/adapter.rs crates/parallax-api/src/lib.rs`
> On excerpt mismatch, STOP.

## Status

- **Priority**: P3
- **Effort**: L
- **Risk**: MED
- **Depends on**: none
- **Category**: direction
- **Planned at**: commit `8bc3f13`, 2026-07-07

## Why this matters

Exemplars attach `trace_id`/`span_id` to a metric measurement, turning a
dashboard from a dead chart into an investigation entry point: click the p99
latency bucket, jump to the exact slow trace. The brief calls this critical
for the Grafana-replacement claim. The audit found Parallax **drops exemplars
entirely** — the OTLP normalization path never reads `dp.exemplars` for
number or histogram points, so they are unavailable from both the normalized
path and the native tables. Restoring them is a full vertical (ingest →
storage → API → UI) and needs a design pass because the storage question
("new column vs new table") and the producer question ("Rust SDK doesn't emit
exemplars yet") are both open.

## Current state (the facts the spike starts from)

- `crates/parallax-core/src/normalize.rs:276-297` — `number_point` reads only
  `dp.value`, `dp.time_unix_nano`, `dp.attributes`; `dp.exemplars` is never
  touched. Same for the histogram arm (`normalize.rs:252-265`).
- `crates/parallax-storage/src/model.rs` — `MetricPointRow`/`HistogramRow`
  carry no trace/span id fields.
- Raw OTLP is also force-forwarded to GreptimeDB's native metrics endpoint
  (`greptime.rs:604-614`); the native tables store value+tags, no exemplar
  column.
- Only run-scoped points are persisted to the Parallax `run_metric_points`
  extension table (`greptime.rs:618-639`); most points live only in native
  metric-engine tables.
- Playground reality (sibling repo): the Rust telemetry lib documents
  "Metric exemplars intentionally absent — Rust SDK issue #3369"; the JVM tier
  **does** emit exemplars (`management.tracing.exemplars.include: all`). So a
  demo works only against JVM metrics today.
- OTLP proto types are in `crates/parallax-proto` (`parallax_proto::metrics`).
- Repo conventions: zero-copy ingest is a design rule (decode once, move
  ownership, never clone telemetry on the hot path) — the exemplar read must
  not clone the whole batch.

## Scope

**In scope** (after the spike approves it):
- `crates/parallax-core/src/normalize.rs` (read exemplars)
- `crates/parallax-storage/src/model.rs` (carry exemplar refs)
- `crates/parallax-storage/src/greptime.rs` (+ `memory.rs`) — persist + query
- `crates/parallax-api/src/lib.rs` (`metricExemplars` resolver)
- UI: exemplar markers on a histogram/heatmap panel (one panel as the slice)
- test files
- a design note under `docs/research/architecture/` (Step 1 output)

**Out of scope**:
- Changing the playground's Rust SDK to emit exemplars (upstream-blocked) —
  the demo/tests use JVM-style exemplar data or synthetic OTLP fixtures.
- Exponential-histogram exemplars.
- Backfilling exemplars for already-ingested metrics.

## Steps

### Step 1 (SPIKE — do this first, output a design note)

Answer, in a short note at
`docs/research/architecture/metric-exemplars-design.md`:
1. **Storage shape.** Can exemplars live in the existing `run_metric_points`
   extension table (add `trace_id`/`span_id` columns) for the run-scoped
   subset, and/or does a new `metric_exemplars` extension table
   (name/ts/value/trace_id/span_id/resource attrs) following the
   `run_metric_points` bootstrap+insert pattern (`greptime.rs:81-112,
   618-639`) cover the non-run case? The native metric-engine tables cannot
   hold exemplars, so the Parallax-side extension table is the likely home —
   confirm.
2. **Ingest.** Confirm `parallax_proto::metrics::NumberDataPoint` and
   `HistogramDataPoint` expose an `exemplars` field with `trace_id`/`span_id`
   and value; sketch reading it without cloning the batch (zero-copy rule).
3. **Producer coverage.** State plainly that Rust playground metrics lack
   exemplars today (SDK issue) and that tests/demo use JVM/synthetic data —
   so the feature is correct even where the producer doesn't populate it (the
   UI must show a transparent "no exemplar attached" fallback, per the brief).
4. **Query cost.** How `metricExemplars(name, from, to, filters)` reads the
   extension table efficiently (time-bounded, indexed on ts).

If (1) or (2) is infeasible against the pinned engine/proto, **STOP** and
report the note — do not implement.

### Step 2: Ingest — read exemplars into a new row type

Following the approved design, extend `number_point`/histogram normalization
to collect exemplars into a `MetricExemplarRow { ts_nanos, service, name,
value, trace_id, span_id, attributes }` (or the columns the design chose),
without cloning telemetry on the hot path (borrow, move). Thread the new rows
through `NormalizedMetrics`.

**Verify**: `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` → exit 0.

### Step 3: Storage — persist and query

Add the extension table (bootstrap DDL + insert) in `greptime.rs` per the
design, and a `metric_exemplars(name, range, filters, limit)` adapter method
implemented in `greptime.rs` and `memory.rs`.

**Verify**: `rtk cargo nextest run --workspace` → all pass.

### Step 4: API resolver

Add a `MetricExemplar` object and `metricExemplars(name, fromNanos, toNanos,
service?, limit?)` resolver returning `[MetricExemplar!]!`.

**Verify**: `rtk cargo nextest run --workspace` → all pass.

### Step 5: UI — markers on one panel + honest fallback

Add exemplar dot markers to a single histogram/latency panel (e.g. service
detail latency, or the metric-strip): overlay markers where exemplars exist;
clicking one opens a popover with trace/span id + "open trace". When no
exemplars exist for the panel's metric, show the brief's transparent fallback
text ("No trace exemplar attached; showing traces near this timestamp"). Reuse
Recharts within `ChartContainer`; do not add a chart library.

**Verify (from `ui/`)**: `rtk bun run typecheck`/`lint`/`build` → exit 0.

### Step 6: Tests

- Rust: normalize a synthetic OTLP metrics request carrying an exemplar →
  assert a `MetricExemplarRow` with the trace/span id is produced;
  `metric_exemplars` returns it from the memory store.
- UI: render the panel with exemplar fixtures (markers appear, popover links)
  and without (fallback text appears).

**Verify**: `rtk cargo nextest run --workspace` → all pass;
`rtk bun run test` (from `ui/`) → all pass.

## Done criteria

- [ ] Design note committed under `docs/research/architecture/`
- [ ] Rust: `fmt`/`clippy -D warnings`/`nextest` all clean, new tests present
- [ ] UI: `typecheck`/`lint`/`build`/`test` all exit 0
- [ ] A synthetic OTLP exemplar round-trips ingest→storage→`metricExemplars`
      (asserted)
- [ ] The exemplar panel shows the honest "no exemplar" fallback when none
      exist (asserted)
- [ ] Ingest does not clone the telemetry batch to read exemplars (code
      inspection against the zero-copy rule)
- [ ] Reference leak check prints nothing
- [ ] `plans/README.md` status row updated

## STOP conditions

- Spike finds the proto has no accessible `exemplars` field, or the engine
  cannot host the extension table → STOP with the note.
- Reading exemplars forces a clone of the batch (zero-copy violation) with no
  borrow-based alternative → STOP and report.
- Excerpts don't match live code (drift).

## Maintenance notes

- **Producer gap:** Rust playground exemplars are upstream-blocked; when the
  SDK ships them, the demo gains Rust coverage with no Parallax change.
- **Deferred:** exponential-histogram exemplars; backfill; a
  `metric_exemplars` minute-rollup if query cost grows.
- Reviewer: the honest fallback is mandatory (brief) — a panel that silently
  shows nothing when exemplars are absent is a bug, not a no-op.
