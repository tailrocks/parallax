# Plan 026: Fix issue miscounting — capture exceptions on non-error spans, and dedup the same failure across span+log signals

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat 8bc3f13..HEAD -- crates/parallax-core/src/derive.rs crates/parallax-server/src/worker.rs`
> On excerpt mismatch, STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED
- **Depends on**: none (independent of plan 019 fingerprint work; coordinate
  ordering only — see Maintenance notes)
- **Category**: bug
- **Planned at**: commit `8bc3f13`, 2026-07-07

## Why this matters

Issues exist to count recurrences, and two derivation defects corrupt that
count in opposite directions:

1. **Undercount:** exception events on spans whose status is not ERROR are
   silently dropped — the `if !is_error { continue; }` gate runs *before* the
   exception event is inspected. Many SDKs call `recordException` without
   also setting the span to ERROR, so recorded exceptions vanish.
2. **Overcount:** the standard OTel logging-bridge setup emits the *same*
   failure as both a span exception event and an ERROR/exception log. Both
   become separate `ErrorEventRow`s and each bumps `event_count`, roughly
   doubling occurrence totals for any service that both records and logs
   exceptions — the common configuration.

Together they make "N occurrences" and severity ranking unreliable, which is
the core value of the issues surface.

## Current state

- `crates/parallax-core/src/derive.rs:29-59` — the span gate drops exceptions
  on non-error spans:

  ```rust
  for span in &ss.spans {
      let is_error = span.status.as_ref().map(|s| s.code == 2).unwrap_or(false);
      if !is_error {
          continue;                    // <-- exception event never inspected
      }
      let exception = span.events.iter().find(|e| e.name == "exception");
      let (source, error_type, message, stacktrace, ts) = match exception {
          Some(event) => ( ErrorSource::SpanException, … ),
          None => ( ErrorSource::SpanStatus, "span_error".into(), … ),
      };
      let fp = fingerprint(&error_type, &message, stacktrace.as_deref());
      events.push(ErrorEventRow { … });
  }
  ```

- `crates/parallax-core/src/derive.rs:81-131` — `derive_from_logs` builds
  `LogException` (from `exception.type`/`exception.message` attrs) and
  `LogRecord` (ERROR/FATAL body) events, each with its own fingerprint.
- `crates/parallax-server/src/worker.rs:66-93` — traces and logs are derived
  independently and both funnel into `record_errors`
  (`worker.rs:130-146` → `upsert_issue_occurrence`, which increments
  `event_count`). Nothing dedups a failure seen through both channels.
- `ErrorEventRow` (search `struct ErrorEventRow` in
  `crates/parallax-storage/src/model.rs`) carries `trace_id`, `span_id`,
  `fingerprint`, `ts_nanos`, `source`.
- Repo conventions: zero clippy warnings; cargo-nextest; DCO signoff.
- Note plan 019 (fingerprint v2) is a separate TODO and changes the
  fingerprint *formula*; this plan changes *which events are produced* and
  *whether duplicates are collapsed*. They do not overlap in code lines but
  both touch derivation semantics — see Maintenance notes for ordering.

## Commands you will need

| Purpose | Command (repo root)                                                  | Expected |
|---------|----------------------------------------------------------------------|----------|
| Format  | `rtk cargo fmt --all`                                                | exit 0   |
| Lint    | `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0   |
| Tests   | `rtk cargo nextest run --workspace`                                  | all pass |

## Scope

**In scope**:
- `crates/parallax-core/src/derive.rs`
- `crates/parallax-server/src/worker.rs`
- their test modules / `crates/parallax-server/tests/` (add cases)

**Out of scope**:
- The fingerprint formula (`fingerprint.rs`, `normalize.rs`) — plan 019.
- Cross-*source* error_type normalization (`span_error`/`log_error` vs the
  structured exception type). That is a real issue (a failure seen via
  span-status vs span-exception fingerprints differently) but it is
  entangled with plan 019's structured-field preference; defer it and note
  it in the README "considered" section so it isn't lost.
- The digit-normalizer over-merge (status codes/ports) — also 019-adjacent;
  deferred and noted.

## Git workflow

- `main`, Conventional Commits, `git commit -s`. Push when done.

## Steps

### Step 1: Derive exceptions independently of span status

In `derive.rs`, restructure the span loop so an `exception` event produces a
`SpanException` error **whether or not** the span status is ERROR, while a
non-exception ERROR span still produces a `SpanStatus` error:

- Look up `let exception = span.events.iter().find(|e| e.name == "exception");`
  first.
- Emit a `SpanException` row when `exception.is_some()`.
- Emit a `SpanStatus` row when `is_error && exception.is_none()`.
- Emit nothing when `!is_error && exception.is_none()`.

This preserves current behavior for error spans and stops dropping recorded
exceptions on OK/UNSET spans.

**Verify**: `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` → exit 0.

### Step 2: Dedup within (trace_id, span_id, fingerprint) at record time

In `worker.rs`, before `record_errors` upserts occurrences, collapse events
that describe the same failure observed through multiple signals. The dedup
key is `(trace_id, span_id, fingerprint)` — the same exception logged and
recorded shares all three when trace context is present. Prefer the
`SpanException` source over the `LogRecord`/`LogException` echo when both
exist for one key.

Where to apply it: the trace-derived and log-derived events are produced in
separate `process` arms (`worker.rs:66` and `:81`). They can arrive in
different ingest items, so a purely in-batch dedup will not catch a log that
arrives in a later batch. Two acceptable scopes — pick based on
`upsert_issue_occurrence`'s shape (read
`crates/parallax-storage/src/metadata.rs` around
`upsert_issue_occurrence` / `issue_buckets` first):

- **(a) In-batch dedup (simpler, partial):** dedup the `Vec<ErrorEventRow>`
  within each `record_errors` call by the key above. This collapses the
  common case where the SDK emits both in the same export, which is typical.
- **(b) Persisted-idempotency (complete):** make `upsert_issue_occurrence`
  idempotent on `(trace_id, span_id, fingerprint)` by recording seen keys
  (e.g. a small dedup table or a uniqueness check) so a late log echo does
  not double-count.

Implement (a) in this plan (bounded, no schema change). If the metadata layer
already has a natural idempotency hook, note it and propose (b) as a
follow-up in the README rather than expanding scope here.

When events with the same key are collapsed, keep one occurrence but do not
lose a genuinely distinct second failure (different span_id or fingerprint
must survive).

**Verify**: `rtk cargo nextest run --workspace` → all pass.

### Step 3: Tests

Add unit tests in `derive.rs` and an integration case in
`crates/parallax-server/tests/` (model on the existing derivation/ingest
tests — search for `derive_from_traces` / `derive_from_logs` usage in
`crates/parallax-server/tests/*.rs`):

- **Exception on OK span** produces one `SpanException` error (regression for
  Step 1). Before the fix this yields zero.
- **Error span without exception** still produces one `SpanStatus` error
  (no regression).
- **Same failure as span exception + ERROR log** with the same trace/span id
  yields **one** occurrence after dedup, not two (Step 2), and the surviving
  source is `SpanException`.
- **Two distinct failures** (different span_id) both survive dedup.

**Verify**: `rtk cargo nextest run --workspace` → all pass with new cases.

## Test plan

Covered in Step 3. New cases: OK-span exception capture, error-span no
regression, cross-signal dedup, distinct-failure survival. Pattern: existing
`derive.rs` tests / server ingest tests.

## Done criteria

- [ ] `rtk cargo fmt --all` no diff; clippy exits 0 with `-D warnings`
- [ ] `rtk cargo nextest run --workspace` exits 0 with new cases present
- [ ] `grep -n "if !is_error" crates/parallax-core/src/derive.rs` — the gate
      no longer precedes exception lookup (manual check the restructure)
- [ ] A test proves an exception on a non-error span is captured
- [ ] A test proves span+log of one failure yields one occurrence
- [ ] No out-of-scope files modified (`git status`)
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report if:

- Excerpts don't match live code (drift).
- Capturing non-error-span exceptions makes an existing test that asserts a
  specific issue count fail — that test encodes the old (buggy) behavior;
  report it so the expected count can be updated deliberately rather than
  silently.
- `upsert_issue_occurrence` cannot express in-batch dedup cleanly because
  events are streamed one-by-one with no batch handle — report and propose
  the persisted-idempotency path (b) instead.

## Maintenance notes

- **Ordering vs plan 019:** 019 changes the fingerprint formula. If 019 lands
  first, the dedup key's `fingerprint` component reflects the new formula —
  fine, no conflict. If this plan lands first, 019's change is still
  orthogonal. No hard dependency either direction; just re-run both test
  suites after the second one lands.
- **Deferred (named):** cross-*source* `error_type` divergence
  (`span_error`/`log_error` synthetic labels vs the structured exception
  type) still splits one failure into up to four issues by channel. Fixing it
  means normalizing `error_type` to the structured type across all four
  sources before hashing — entangled with 019's structured-field work, so
  tracked in the README "considered" section, not here.
- Reviewer should confirm dedup does not collapse legitimately distinct
  repeats (same fingerprint, different span) — the key includes `span_id`
  precisely to avoid that.
