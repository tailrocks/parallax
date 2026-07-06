# Plan 032: Add an `evidenceGaps(traceId | runId)` detector and surface gaps in trace/run detail and the bundle

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat 8bc3f13..HEAD -- crates/parallax-core/src/bundle.rs crates/parallax-api/src/lib.rs ui/src/routes/traces.\$traceId.tsx`
> On excerpt mismatch, STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW
- **Depends on**: none (complements plan 029 story; independent)
- **Category**: direction
- **Planned at**: commit `8bc3f13`, 2026-07-07

## Why this matters

A core Parallax differentiator in the brief: missing evidence is itself an
answer. An orphan server span, a browser span with no backend child, a
producer link with no consumer, or a log with no trace id all mean
"instrumentation is incomplete" — competitors hide this; Parallax should show
it. The audit confirmed the data to detect the cheapest, highest-value gaps
already exists at read time. This plan adds a deterministic gap detector over
a trace's (or run's) spans+logs and surfaces the results in the UI and,
optionally, the bundle's `missing_evidence`.

## Current state

- `spans_by_trace` returns all spans with `parent_span_id`, `span_id`, `kind`
  (`adapter.rs:145`; `greptime.rs:340-353`). `logs_by_trace` returns logs with
  `trace_id`/`span_id` (`adapter.rs:151`).
- The worker already has the exact "empty/zero trace id" predicate for logs:
  `!trace_id.is_empty() && trace_id.chars().any(|c| c != '0')`
  (`crates/parallax-server/src/worker.rs:137`).
- `Bundle.missing_evidence: Vec<String>` already exists and is populated for
  budget drops (`bundle.rs:432, 456`).
- Typed span links may exist if plan 028 landed (for producer/consumer gap
  detection).
- Repo conventions: zero clippy warnings; cargo-nextest; DCO signoff. UI
  conventions per `ui/AGENTS.md`.

## Commands you will need

(Rust fmt/clippy/nextest at repo root; UI typecheck/lint/test/build from
`ui/`.)

## Scope

**In scope**:
- `crates/parallax-core/src/` (a pure `evidence_gaps` function, e.g. in a new
  `gaps.rs`)
- `crates/parallax-api/src/lib.rs` (`evidenceGaps` resolver + object)
- `ui/src/lib/api.ts` + `ui/src/routes/traces.$traceId.tsx` (gap list)
- optionally hook the detector into bundle assembly to enrich
  `missing_evidence` (small edit in `bundle.rs`)
- test files

**Out of scope**:
- Producer→consumer gap across traces via reverse link lookup (needs an index
  — storage audit). Detect only *within* the fetched trace/run set: a producer
  span whose link target is absent from the fetched set is a candidate, but do
  not attempt a global reverse scan.
- A `telemetryQuality` numeric score (the brief's separate feature) — this
  plan lists gaps, not a score. Defer scoring and note it.
- Sampling/propagation-rate metrics — separate.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one agent trailer. Push when
  done.

## Steps

### Step 1: Pure gap detector

Add a pure function in `parallax-core` (unit-testable, deterministic):

```rust
pub struct EvidenceGap {
    pub kind: String,     // "orphan_span" | "log_without_trace" | "producer_without_consumer" | "browser_without_backend"
    pub subject: String,  // span_id / log ref
    pub detail: String,   // human-readable, low-cardinality
}

pub fn detect_gaps(spans: &[SpanRow], logs: &[LogRow]) -> Vec<EvidenceGap>;
```

Detections (only the ones computable from the fetched set):
1. **orphan_span**: a span whose `parent_span_id` is non-empty and not present
   among the fetched spans' `span_id`s. Caveat in `detail`: may be a
   legitimate cross-service root (say so, don't over-claim).
2. **log_without_trace**: a log whose `trace_id` is empty or all-zero (reuse
   the worker's predicate). 
3. **producer_without_consumer**: a `SPAN_KIND_PRODUCER` span whose link
   target span/trace is not in the fetched set (best-effort within-set only).
4. **browser_without_backend**: a browser/client span (resource
   `telemetry.sdk.language = webjs` or `span.kind = CLIENT` with an
   `http`/`url` attr) that has no child server span in the set.

Keep it deterministic (sort output by span/log order; no clock/rng).

**Verify**: `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` → exit 0.

### Step 2: `evidenceGaps` resolver

In `lib.rs`, add an `EvidenceGap` object and an `evidenceGaps(traceId?,
runId?)` resolver (exactly one anchor, mirror `bundle`). Fetch spans+logs for
the anchor, call `detect_gaps`, return `[EvidenceGap!]!`.

**Verify**: `rtk cargo nextest run --workspace` → all pass.

### Step 3: Enrich the bundle (optional, small)

In `bundle.rs::assemble`, call `detect_gaps(&inputs.trace_spans,
&inputs.trace_logs)` and push a short line per gap into `missing_evidence`
(dedup with existing budget lines). This makes the bundle honest about what
the evidence lacks — a brief goal. Keep it deterministic so plan 027's hash
stays reproducible (gaps are derived from the same inputs, so they are stable).

**Verify**: `rtk cargo nextest run --workspace` → all pass (update the bundle
test expectation if it asserts `missing_evidence` exactly).

### Step 4: UI gap list

In `traces.$traceId.tsx`, add an "Evidence gaps" section (only shown when
non-empty) listing each gap with an icon (warning), the kind, and the detail.
Fetch `evidenceGaps` in the loader. Reuse existing section/chip styling. Keep
it calm — a gap is informative, not an error state.

**Verify (from `ui/`)**: `rtk bun run typecheck`/`lint`/`build` → exit 0.

### Step 5: Tests

- Rust: unit tests on `detect_gaps` — a trace with a dangling
  `parent_span_id` yields one `orphan_span`; a log with all-zero trace id
  yields `log_without_trace`; a producer with no consumer in-set yields the
  producer gap; a clean trace yields none. Determinism assertion.
- UI: render test showing the gap list appears for a fixture with gaps and is
  absent for a clean fixture. Model on `waterfall.test.tsx`.

**Verify**: `rtk cargo nextest run --workspace` → all pass;
`rtk bun run test` (from `ui/`) → all pass.

## Test plan

- Rust: four detection cases + clean case + determinism (Step 5).
- UI: gap list present/absent (Step 5).
- Pattern: `parallax-core` unit tests; `waterfall.test.tsx`.

## Done criteria

- [ ] Rust: `fmt` no diff, `clippy -D warnings` exit 0, `nextest` exit 0 with
      new tests
- [ ] UI: `typecheck`/`lint`/`build`/`test` all exit 0 (from `ui/`)
- [ ] `evidenceGaps(...)` returns `[EvidenceGap!]!` (exactly-one anchor)
- [ ] `detect_gaps` is deterministic (same input → equal output, asserted)
- [ ] Orphan-span detection carries the cross-service-root caveat in `detail`
- [ ] Gap list renders in trace detail only when non-empty (asserted)
- [ ] Reference leak check prints nothing
- [ ] No out-of-scope files modified (`git status`)
- [ ] `plans/README.md` status row updated

## STOP conditions

Stop and report if:

- Excerpts don't match live code (drift).
- Orphan-span detection produces overwhelming false positives on the
  playground's traces (because batches split a trace across fetches) — the
  detector runs over the *fully fetched* trace via `spans_by_trace`, so this
  should be safe; if it isn't, STOP and report rather than suppressing gaps.
- Enriching the bundle breaks plan 027's hash-stability test — reconcile
  ordering (gaps must be deterministic) or drop Step 3 and ship the resolver
  only.

## Maintenance notes

- **Deferred (named):** cross-trace producer→consumer gap detection needs a
  reverse link index (storage audit); and `telemetryQuality` scoring is a
  separate feature that would aggregate these gaps into a grade. Track both in
  README.
- Reviewer: confirm the orphan-span caveat text is present (avoid claiming a
  legitimate cross-service root is a bug) and that detection is within-set
  only (no global scans).
- When plan 028's typed links land, the producer/consumer detection can use
  them directly instead of re-parsing JSON.
