# Plan 027: Make the bundle canonical hash stable across versions/budgets and bound trace+metric sections to the token budget

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat 8bc3f13..HEAD -- crates/parallax-core/src/bundle.rs`
> On excerpt mismatch, STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW
- **Depends on**: none (touches different regions of bundle.rs than 023/025;
  land after them to avoid rebase churn — see Maintenance notes)
- **Category**: bug
- **Planned at**: commit `8bc3f13`, 2026-07-07

## Why this matters

The bundle's `canonical_hash` is meant to be a stable evidence-identity/audit
key ("same evidence → same hash"), but it hashes the *entire* serialized
bundle including `generator = "parallax/<CARGO_PKG_VERSION>"` and the whole
`bounded` report (`estimated_tokens`, `dropped_log_lines`, `max_tokens`). So
the same underlying evidence hashes differently after any version bump or when
requested at a different `maxTokens`. Two agents pulling the same issue at
different budgets get different hashes — the hash cannot serve its purpose.
Separately, the token budget only trims log lines and the stacktrace tail;
`trace.spans` (up to 500), `metric_windows` (3 × 60 points), the issue
summary, and hypotheses are never bounded, so a large failing trace or wide
metric windows blow past `max_tokens` exactly when context is largest.

## Current state

- `crates/parallax-core/src/bundle.rs:421-439` — the bundle is built with
  `generator: concat!("parallax/", env!("CARGO_PKG_VERSION"))` and a
  `bounded: BoundReport { max_tokens, .. }`.
- `crates/parallax-core/src/bundle.rs:441-471` — the budget loop trims only
  logs (oldest-first) then the stacktrace (to 3 frames). `trace.spans`,
  `metric_windows`, issue, hypotheses are untouched.
- `crates/parallax-core/src/bundle.rs:473-476`:

  ```rust
  let serialized = serde_json::to_string(&bundle).unwrap_or_default();
  bundle.bounded.estimated_tokens = estimate_tokens(&serialized);
  bundle.canonical_hash = Some(canonical_hash(&bundle));
  ```

- `crates/parallax-core/src/bundle.rs:561-600` — `canonical_hash` strips only
  the `canonical_hash` field, then sorts keys recursively and SHA-256s the
  whole object (so `generator` and `bounded` are included).
- The trace section is populated from `inputs.trace_spans` (up to `MAX_ROWS`
  = 500, set by the API at `crates/parallax-api/src/lib.rs:1469`).
- Existing bundle test: `crates/parallax-server/tests/m2_bundle.rs` uses a
  tiny trace, so it never triggers over-budget.
- Repo conventions: zero clippy warnings; cargo-nextest; DCO signoff.

## Commands you will need

| Purpose | Command (repo root)                                                  | Expected |
|---------|----------------------------------------------------------------------|----------|
| Format  | `rtk cargo fmt --all`                                                | exit 0   |
| Lint    | `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0   |
| Tests   | `rtk cargo nextest run --workspace`                                  | all pass |

## Scope

**In scope**:
- `crates/parallax-core/src/bundle.rs`
- `crates/parallax-server/tests/m2_bundle.rs` (add cases)

**Out of scope**:
- The redaction fields/rules (plans 023/025).
- The GraphQL `bundle` resolver's fetch limits
  (`crates/parallax-api/src/lib.rs`) — the API still passes up to 500 spans;
  this plan bounds them *inside* assembly so the contract is unchanged.
- Changing `SCHEMA_VERSION` semantics.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one agent trailer. Push when
  done.

## Steps

### Step 1: Hash only the evidence, not environment/budget metadata

Change `canonical_hash` (bundle.rs:561) so it excludes both `generator` and
the entire `bounded` object (in addition to the existing `canonical_hash`
exclusion). Since it already deserializes the bundle to a
`serde_json::Value` and strips a field, remove the `generator` and `bounded`
keys from the object before sorting+hashing. Add a doc comment stating exactly
what the hash covers: anchor, issue, latest_event, run, trace, metric_windows,
logs, hypotheses, missing_evidence, redaction, schema_version — but **not**
generator/bounded/canonical_hash.

Keep `schema_version` in the hash (a schema change legitimately changes
evidence identity); exclude only `generator` (build version) and `bounded`
(per-request budget accounting).

**Verify**: `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` → exit 0.

### Step 2: After log/stacktrace trimming, bound the trace and metric sections

Extend the budget logic (bundle.rs:464 onward) so that if `used > max_tokens`
after the existing log+stacktrace trims:

1. Reduce `trace.spans` — keep error/ancestor spans preferentially, drop the
   least-relevant (e.g. keep spans with error status and the root; drop
   healthy leaf spans) until the estimate fits or a floor is reached. If the
   trace section carries a simple `Vec<span>`, a pragmatic first cut is:
   retain error spans + their ancestors, then fill remaining budget with the
   longest-duration spans. Record how many spans were dropped in
   `missing_evidence` (`"bounded: dropped N trace spans to fit budget"`).
2. Reduce `metric_windows` point counts — decimate each window's points
   (keep first/last + every k-th) until it fits; record it in
   `missing_evidence`.

Recompute `estimated_tokens` after trimming. If it *still* exceeds
`max_tokens` after both reductions reach their floor, leave it and let
`estimated_tokens` honestly report the overage (do not silently claim
in-budget). Keep the reductions deterministic (no randomness, no clock) so the
hash from Step 1 stays reproducible.

**Verify**: `rtk cargo nextest run --workspace` → all pass.

### Step 3: Tests

In `crates/parallax-server/tests/m2_bundle.rs` add:

- **Hash stability across budget:** assemble the same inputs at
  `max_tokens = 2000` and `max_tokens = 8000`; assert the two
  `canonical_hash` values are **equal** (only `bounded` differs). Before the
  fix they differ.
- **Hash stability across generator:** (unit test in `bundle.rs` is easier)
  build two bundles differing only in a synthetic `generator` value and
  assert equal hashes — or assert the hashed value string does not contain the
  version. Choose whichever is expressible.
- **Trace bounding:** assemble a bundle with a large synthetic trace (e.g.
  400 spans, a handful error) at a small `max_tokens`; assert
  `estimated_tokens <= max_tokens` OR (if the floor is hit) that
  `missing_evidence` records dropped spans, and that at least one error span
  survived.

**Verify**: `rtk cargo nextest run --workspace` → all pass with new cases.

## Test plan

Covered in Step 3. New cases: hash-equal-across-budget, hash-independent-of-
generator, trace-section bounding. Pattern: existing `m2_bundle.rs` assembly
test.

## Done criteria

- [ ] `rtk cargo fmt --all` no diff; clippy exits 0 with `-D warnings`
- [ ] `rtk cargo nextest run --workspace` exits 0 with new cases present
- [ ] Same inputs at two different `max_tokens` produce the **same**
      `canonical_hash` (asserted by test)
- [ ] A large-trace bundle either fits the budget or records dropped spans in
      `missing_evidence` while keeping error spans (asserted by test)
- [ ] No out-of-scope files modified (`git status`)
- [ ] `plans/README.md` status row updated

## STOP conditions

Stop and report if:

- Excerpts don't match live code (drift).
- The `trace` section's in-memory shape makes ancestor-preserving span
  dropping impractical without a parent map you'd have to rebuild — if so,
  fall back to "keep all error spans + first N by duration" and note the
  simplification, but still record drops.
- Excluding `bounded` from the hash breaks an existing test that asserted a
  specific hash literal — update that expectation deliberately (the hash
  *should* change once; document it in the commit).

## Maintenance notes

- **Ordering:** land after 023/025 (they edit nearby bundle regions) to avoid
  rebase churn; no logical dependency.
- **Deferred:** a token estimator based on real tokenization (vs
  `chars()/4`) would make the budget precise; out of scope — the current
  estimate is a documented approximation (bundle.rs:234).
- Reviewer should confirm the hash now covers exactly the evidence set listed
  in Step 1's doc comment, and that bounding is deterministic (no `Date`/rng)
  so hashes stay reproducible across runs.
- If a future plan adds a new evidence section to the bundle, it must be
  included in the hash and considered for budget bounding.
