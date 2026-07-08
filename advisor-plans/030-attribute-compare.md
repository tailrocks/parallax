# Plan 030: Add `attributeCompare` — selected-vs-baseline span-attribute overrepresentation, rendered as a BubbleUp panel

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat 8bc3f13..HEAD -- crates/parallax-api/src/lib.rs crates/parallax-storage/src/adapter.rs crates/parallax-storage/src/greptime.rs crates/parallax-storage/src/memory.rs`
> On excerpt mismatch, STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: MED
- **Depends on**: none
- **Category**: direction
- **Planned at**: commit `8bc3f13`, 2026-07-07

## Why this matters

The Honeycomb-BubbleUp idea — "select an anomaly, compare it against baseline
across all dimensions, and rank the fields that stand out" — is the brief's
answer to "why did this spike?". The storage audit found this is
**live-feasible without materialization for span attributes**, because
GreptimeDB's native pipeline flattens every span attribute key to its own
real, columnar `span_attributes.<key>` column. A time-bounded `GROUP BY` over
those columns is cheap and needs no JSON parsing. This plan adds a storage
method + resolver that, given a selected window/filter and a baseline window,
returns the attribute values overrepresented in the selection — the
deterministic, inspectable table the brief specifies.

## Current state

- Span attributes are flattened columns: `greptime.rs:452-473`
  (`reassemble_attrs` folds `span_attributes.<k>` / `resource_attributes.<k>`
  columns back into JSON on read). So the raw columns exist to `GROUP BY`.
- `traces_search` (`greptime.rs:1238-1272`) shows the existing time-bounded
  span-scan pattern (`scan_where` / participation) to copy for the windowed
  aggregate.
- `information_schema` discovery of columns is already used for metric tables
  (`discover_metric_names`, `greptime.rs:1498-1532`) — the same approach lists
  the `span_attributes.*` columns available to compare.
- No existing resolver aggregates attribute value counts (the API audit's gap
  matrix: `attributeCompare` = "No").
- Storage adapter trait: `crates/parallax-storage/src/adapter.rs:123-275`;
  new methods must be implemented in both `greptime.rs` and `memory.rs`.
- Guardrails the brief mandates: never group by `trace_id`, `run_id`,
  `user_id`, `session_id`, raw user text, secrets, or stacktrace bodies;
  prefer low-cardinality semantic fields; label exact identifiers as
  identifiers.
- Repo conventions: zero clippy warnings; cargo-nextest; DCO signoff.

## Commands you will need

(Rust fmt/clippy/nextest at repo root, as in prior plans. UI is optional in
this plan — see Step 5.)

## Scope

**In scope**:
- `crates/parallax-storage/src/adapter.rs` (new trait method + result type)
- `crates/parallax-storage/src/greptime.rs` (impl over flattened columns)
- `crates/parallax-storage/src/memory.rs` (impl over in-memory attrs)
- `crates/parallax-api/src/lib.rs` (`attributeCompare` resolver + objects)
- test files
- **Optional** UI (Step 5): a `AttributeCompare` panel + wiring on trace list
  / issue detail, only if time allows; otherwise ship API + tests and defer UI
  to a follow-up (note it in README).

**Out of scope**:
- Log-attribute compare (needs per-row JSON extraction — costlier; defer and
  note).
- Materialized `span_attribute_rollups` — live only in this plan.
- Metric exemplar compare (plan 033).
- Comparing high-risk fields (enforce the denylist).

## Git workflow

- `main`, Conventional Commits, `git commit -s`. Push when done.

## Steps

### Step 1: Result type + trait method

In `adapter.rs` add:

```rust
pub struct AttributeCompareRow {
    pub key: String,
    pub value: String,
    pub selected_count: u64,
    pub selected_total: u64,
    pub baseline_count: u64,
    pub baseline_total: u64,
    pub score: f64,        // bounded overrepresentation score
}

// on TelemetryStore:
async fn attribute_compare(
    &self,
    selected: RangeInclusive<u128>,
    baseline: RangeInclusive<u128>,
    service: Option<&str>,
    error_only: bool,
    keys: &[String],       // candidate low-cardinality keys; empty = discover
    top_n: usize,
) -> anyhow::Result<Vec<AttributeCompareRow>>;
```

### Step 2: GreptimeDB implementation over flattened columns

Implement `attribute_compare` in `greptime.rs`:
- If `keys` is empty, discover candidate `span_attributes.*` columns via
  `information_schema` (reuse the `discover_metric_names` approach), then
  apply the **denylist** (drop any key whose leaf name is `trace_id`,
  `span_id`, `run_id`, `user.id`, `session.id`, `enduser.*`, or matches an
  id/high-cardinality heuristic). Cap the number of keys scanned.
- For each candidate key, run two time-bounded `GROUP BY "span_attributes.<k>"`
  count queries (selected window + baseline window), each with the optional
  service filter and `error_only` (`span_status_code = 'STATUS_CODE_ERROR'`).
  Escape the key as an **identifier** (`escape_ident` from plan 022 if landed,
  else double the `"`).
- Compute a bounded overrepresentation score per value (e.g.
  `selected_share - baseline_share`, clamped to [0,1]; or JS-divergence — keep
  it deterministic and documented). Return the top-N rows across all keys.

Keep the total query count bounded (cap candidate keys) so one request cannot
fan out unboundedly — coordinate with plan 024's complexity limits.

**Verify**: `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` → exit 0.

### Step 3: Memory implementation

Implement `attribute_compare` in `memory.rs` by iterating the in-memory spans,
bucketing into selected/baseline by timestamp, counting values per key from
each span's attributes map, applying the same denylist and scoring. This makes
the feature unit-testable without a live engine.

**Verify**: `rtk cargo nextest run --workspace` → all pass.

### Step 4: Resolver

In `lib.rs`, add an `AttributeCompareRow` GraphQL object and an
`attributeCompare` resolver (args: `selectedFromNanos`, `selectedToNanos`,
`baselineFromNanos`, `baselineToNanos`, `service?`, `errorOnly?`, `keys?`,
`topN?`). Parse ranges with the existing `parse_range`. Enforce a `topN` cap
(`clamp_limit`-style). Return `[AttributeCompareRow!]!`.

**Verify**: `rtk cargo nextest run --workspace` → all pass.

### Step 5 (optional): UI panel

If shipping UI: add an `AttributeCompare` component in `ui/src/components/
console/` rendering the brief's table (Rank | Field | Selected % | Baseline %
| note), with each row's paired mini-bars (reuse `PillMeter`/`HeatCell`). Wire
one entry point — the trace list "errors only" selection or the issue-detail
spike — calling `attributeCompare` over the current window vs a baseline
window. Label exact-identifier fields as identifiers per the brief. If not
shipping UI now, stop after Step 4 and record the UI as a follow-up.

**Verify (from `ui/` if UI shipped)**: `rtk bun run typecheck`/`lint`/`build`
→ exit 0.

### Step 6: Tests

- Rust (memory store): seed spans across two windows where
  `service.version = 2.0.0` is 90% of the selected set and 5% of baseline;
  assert `attribute_compare` ranks `service.version=2.0.0` at the top with a
  high score. Assert denylisted keys (`trace_id`) never appear in output.
- Determinism: same seed → identical ranked output.

**Verify**: `rtk cargo nextest run --workspace` → all pass.

## Test plan

- Rust memory-store tests: overrepresentation ranking, denylist enforcement,
  determinism (Step 6).
- UI (if shipped): a render test of the compare table.
- Pattern: existing memory-store metric tests in
  `crates/parallax-server/tests/m2_metrics_dashboards.rs`.

## Done criteria

- [ ] Rust: `fmt` no diff, `clippy -D warnings` exit 0, `nextest` exit 0 with
      new tests
- [ ] `attributeCompare(...)` resolver returns ranked
      `[AttributeCompareRow!]!` (grep)
- [ ] A seeded test proves the overrepresented value ranks first
- [ ] A test proves `trace_id`/`run_id`/session/user keys never appear in
      output (denylist)
- [ ] Candidate-key count is bounded (no unbounded fan-out) — code inspection
- [ ] No out-of-scope files modified (`git status`)
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report if:

- Excerpts don't match live code (drift).
- `information_schema` does not expose `span_attributes.*` as separate columns
  in the pinned GreptimeDB (the flattening assumption is wrong) — STOP; the
  whole approach depends on it, and the fallback (JSON extraction) is a
  different, costlier design.
- The denylist can't be applied because attribute keys arrive in a form you
  can't classify — report before shipping something that could group by an
  identifier.

## Maintenance notes

- **Deferred:** log-attribute compare (JSON extraction cost), and a
  `span_attribute_rollups` materialization if live group-bys get slow at
  server scale. Track in README.
- Reviewer: scrutinize the candidate-key cap and the denylist — an
  unbounded group-by set or a leaked identifier are the two real risks.
- The score formula is a deterministic approximation; if a later plan needs
  statistical rigor, swap in JS-divergence behind the same interface.
