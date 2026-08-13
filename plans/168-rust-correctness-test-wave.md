# Plan 168: Close the correctness-critical Rust test gaps (wave 1)

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving on. If
> anything in "STOP conditions" occurs, stop and report — do not improvise.
> When done, update this plan's row in `plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat f6208070..HEAD -- crates/parallax-xtask/src/command.rs crates/parallax-redaction/ crates/parallax-greptime/src/arrow_sql.rs crates/parallax-evidence/src/bundle/ crates/parallax-spool/ crates/parallax-ingest/src/metrics.rs crates/parallax-server/src/otlp_validation.rs crates/parallax-server/src/live.rs crates/parallax-server/src/sentry_http.rs crates/parallax-server/src/alerting/delivery.rs crates/parallax-api/src/query_limits.rs crates/parallax-cli/`
> — on any change, re-verify the excerpts below against live code before
> proceeding; on mismatch, STOP.

## Status

- **Priority**: P1
- **Effort**: L (12 independent steps, each S–M; land as several small PRs)
- **Risk**: LOW (almost entirely test-only; two tiny production refactors called out)
- **Depends on**: none
- **Category**: tests
- **Planned at**: parallax `f6208070`, 2026-08-13

## Why this matters

A deep coverage audit found the workspace strong in analysis/worker/prune
testing but with holes exactly where silent corruption lives: the command
named `integration` runs only doctests, 13 of 20 secret detectors have zero
positive tests, the Arrow decode path is tested for 3 of ~12 column types,
evidence bounding/hashing swallow failures untested, spool framing has no
corruption tests, and the CLI's destructive prune guard is unproven. These
are the "wrong answer / data loss / secret leak" class — the ones QA must
kill first. Every step here is a test (or a ≤10-line enabling refactor) that
makes a real defect class fail loudly.

## Current state (verified excerpts)

- `crates/parallax-xtask/src/command.rs:241-243`:
  ```rust
  fn integration(root: &Path) -> Result<()> {
      run(root, "cargo", &["test", "--workspace", "--doc", "--locked"])
  }
  ```
  Only one executable doctest exists in the workspace. `.config/nextest.toml`
  defines a `real-engine` profile that no xtask arm invokes; the 9
  `#[ignore]`d live-GreptimeDB tests in `crates/parallax-server/tests/` run
  only in `.github/workflows/storage-integration.yml`.
- `crates/parallax-redaction/src/lib.rs:44-154` — 20 ordered detectors;
  order is load-bearing (`anthropic_api_key` must precede `openai_api_key`).
  Positive tests exist for only 7; no benign-text false-positive corpus.
- `crates/parallax-greptime/src/arrow_sql.rs` — `array_value_to_json`
  branches over Timestamp/Dictionary/Decimal128/Binary/unsigned ints; the
  fixture in `arrow_sql/tests.rs:8-45` uses only Int64/Utf8/Float64. The
  test `row_count_parity_json_shape` (`tests.rs:91-101`) decodes the same
  bytes twice and compares — a tautology.
- `crates/parallax-evidence/src/bundle/hash.rs:5-8`:
  ```rust
  pub(super) fn canonical_hash(bundle: &Bundle) -> String {
      let mut value = serde_json::to_value(bundle).unwrap_or_default();
  ```
  Serialization failure → hashes `null`. Same `unwrap_or_default` pattern in
  `bundle/bounding.rs` token estimation (failure → 0 tokens → no bounding).
  `decimate_points` / `bound_metric_windows` (`bounding.rs:61-121`) have
  zero tests.
- `crates/parallax-spool/src/spool/append.rs:73`:
  ```rust
  let len = u32::try_from(payload.len()).unwrap_or(u32::MAX);
  ```
  Oversized payload writes a wrong length prefix → desyncs the segment.
  `count_pspl_frames` returns `Ok(0)` on bad magic (corrupt = "empty"), and
  is REIMPLEMENTED at `crates/parallax-cli/src/doctor.rs:104`.
- `crates/parallax-ingest/src/metrics.rs:84-87` — exponential histograms /
  summaries hit `_ => {}` (silent drop; the promised doctor counter does not
  exist); `number_point` `None => 0.0` fallback fabricates zero points.
- `crates/parallax-server/src/otlp_validation.rs` — three validators, one
  test (traces only).
- `crates/parallax-server/src/live.rs:84-97,166+` — StreamFilter /
  SpanStreamFilter predicates: zero tests; lag drop is silent by design but
  unpinned.
- `crates/parallax-server/src/sentry_http.rs` — only `parse_sentry_auth` is
  unit-tested; RejectReason→HTTP-status mapping untested (a 2xx on a
  rejected envelope means the SDK drops the event forever).
- `crates/parallax-server/src/alerting/delivery.rs:174-193` —
  `claim_is_available` + lease consts live INSIDE `#[cfg(test)]`; the test
  exercises the shadow, not the CAS SQL in
  `crates/parallax-metadata/src/turso/alerts.rs:790-830`. No concurrent
  double-claim test (the occurrence analog exists at
  `crates/parallax-metadata/src/turso/tests.rs:41`).
- `crates/parallax-api/src/query_limits.rs:7-11` — `clamp_limit` guards ~30
  resolver call sites, zero direct tests; `Some(0)` yields 0 rows (intended?
  undocumented).
- `crates/parallax-cli/` — no `tests/` dir at all; prune `execute/yes`
  gating in `src/doctor.rs:389-397` untested.

Conventions: cargo-nextest, table-driven tests common (`state_machine.rs`
tests are the exemplar), proptest used in 9 files (`trace_analysis/tests.rs`
order-invariance is the good pattern), fmt+clippy zero warnings.

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| Fast gates | `cargo xtask ci && cargo xtask lint` | exit 0, zero warnings |
| Full unit suite | `cargo xtask test` | all pass |
| One crate | `cargo nextest run -p <crate>` | all pass |
| One test | `cargo nextest run -p <crate> -E 'test(<name>)'` | listed + pass |
| Real-engine suite (after Step 1) | `cargo xtask integration` | runs `--run-ignored` real-engine profile, exit 0 (needs Docker-free managed engine download; CI mirrors storage-integration.yml) |
| Arch tiers | `cargo xtask arch` | exit 0 |

## Scope

**In scope**: test modules in the crates named above; `crates/parallax-xtask/src/command.rs`
(integration arm only); `crates/parallax-spool/src/spool/append.rs` (export
parser + oversized-payload error); `crates/parallax-cli/src/doctor.rs`
(replace duplicated parser with the spool export); `crates/parallax-cli/Cargo.toml`
+ new `crates/parallax-cli/tests/cli.rs`; `crates/parallax-server/src/alerting/delivery.rs`
(move lease predicate out of cfg(test) or delete the shadow).

**Out of scope**: any behavior change beyond the two named refactors —
especially NO fix for the exponential-histogram drop (plan 166 owns that
decision; here we only characterize), NO fake/engine parity work (plan 169),
NO resolver suites (plan 169), NO migration machinery (plan 169).

## Git workflow

PR-only `main`; land as 3–4 small PRs grouped by crate cluster (never
parallel PRs); `git commit -s`, Conventional Commits (`test(scope): …`),
agent trailer per `COMMITS.md`.

## Steps

### Step 1: Real integration gate

In `crates/parallax-xtask/src/command.rs`: rename the current `integration`
body to a new `doctests` arm (keep it reachable: `cargo xtask doctests`);
make `integration` run
`cargo nextest run --workspace --run-ignored all --profile real-engine --locked`
(profile exists in `.config/nextest.toml`). Update the xtask CLI enum/help
in `crates/parallax-xtask/src/cli.rs` accordingly and any CI workflow that
called `cargo xtask integration` expecting doctests
(`grep -rn "xtask integration" .github/ docs/ CONTRIBUTING.md`).

**Verify**: `cargo xtask doctests` → runs doc tests, exit 0.
`cargo xtask integration` → executes the 9 previously-ignored tests (count
visible in nextest summary). CI references updated.

### Step 2: Redaction detector table

New tests in `crates/parallax-redaction/src/lib.rs` tests module: one
table of `(detector_name, positive_canary, benign_negative)` covering ALL
20 detectors — synthetic canaries shaped like each pattern (follow the
existing `sk_live_XXXX…`/`ghp_0123…` placeholder style; NEVER real-looking
live values). Assert per row: positive input → `redacted_counts` contains
exactly that detector key; benign input → zero redactions. Plus an ordering
test: an `sk-ant-…` canary counts as `anthropic_api_key`, never
`openai_api_key`; a bearer-token canary NOT preceded by a GitHub token
fires `bearer_token`.

**Verify**: `cargo nextest run -p parallax-redaction` → all pass; if any
detector regex fails its canary, that is a REAL finding — report it in the
PR, fix the regex only if the fix is one-line-obvious, else STOP condition 4.

### Step 3: Arrow decode type matrix

Extend `crates/parallax-greptime/src/arrow_sql/tests.rs`: parameterize the
fixture over one column per supported `DataType` (Timestamp ns/ms,
Dictionary<Int32,Utf8>, Decimal128, Binary, LargeBinary, LargeUtf8, u8–u64,
i8–i32, Float32, Boolean, Null) asserting the exact JSON per type. Add a
truncated-frame case for `validate_ipc_frame_lengths` (expect Err) and a
two-`RecordBatch` stream case for `append_batch_rows`. DELETE
`row_count_parity_json_shape` (tautology) — real parity moves to the
real-engine suite (plan 169).

**Verify**: `cargo nextest run -p parallax-greptime` → all pass.

### Step 4: Evidence bounding + hash

`crates/parallax-evidence/src/bundle/`: unit tests for `decimate_points`
(len 1/2/3; keep=1; keep≥len; keep=2 — assert output length ∈ [1,keep] and
both endpoints preserved) and `bound_metric_windows` (post-condition:
token estimate ≤ max OR a bounded-note present). For `canonical_hash`: add
a test proving `serde_json::to_value(bundle)` cannot fail for a
maximally-populated Bundle (fuzz-ish: build via the existing builders in
`bundle/tests.rs`), then replace `unwrap_or_default()` with
`expect("Bundle serialization is infallible")` in `hash.rs` AND the token
estimator in `bounding.rs` so a future non-serializable field panics in
tests instead of corrupting identity silently. Add a proptest: decimation
output is a subsequence of input.

**Verify**: `cargo nextest run -p parallax-evidence` → all pass.

### Step 5: Spool framing corruption suite

`crates/parallax-spool`: byte-level tests — truncated final frame, corrupt
magic (expect an error or a distinct "damaged" signal, NOT Ok(0): change
`count_pspl_frames` to return `Err` on bad magic — callers:
`grep -rn "count_pspl_frames" crates/`), zero-length frame, restart-then-append
(reopen path, no second MAGIC mid-file). Change `append.rs:73` oversized
clamp to return an error instead of writing `u32::MAX`. Proptest:
`Vec<Vec<u8>>` append → count round-trip. Export the parser
(`pub fn count_pspl_frames`) and delete the duplicate at
`crates/parallax-cli/src/doctor.rs:104`, pointing the CLI at the spool
crate's export.

**Verify**: `cargo nextest run -p parallax-spool -p parallax-cli` → pass;
`grep -rn "fn count_pspl_frames" crates/ | wc -l` → 1.

### Step 6: Ingest characterization

`crates/parallax-ingest/src/tests.rs`: exponential-histogram request →
empty normalized output (characterize current drop); summary → empty;
`data: None` → empty; exemplar skip arms (missing value, empty
trace_id/span_id); `number_point` absent-value arm — characterize the
current `None => 0.0` and mark with a comment pointing at plan 166's
decision. Property: gauge+histogram normalization conserves point count.

**Verify**: `cargo nextest run -p parallax-ingest` → all pass.

### Step 7: OTLP validation completeness

Mirror the existing traces test for `log_trace_ids` and
`metric_trace_ids` + exemplar validation in
`crates/parallax-server/src/otlp_validation/tests.rs`.

**Verify**: `cargo nextest run -p parallax-server -E 'test(/otlp_validation/)'` → ≥3 tests pass.

### Step 8: Live filter predicates

Table-driven tests for `StreamFilter::matches` and
`SpanStreamFilter::matches` in `crates/parallax-server/src/live/tests.rs`
(each predicate individually + combined; severity boundary equals-floor
case; `min_duration_ms` float conversion boundary). Pin the lag behavior:
a lagged receiver continues without error (documented-intentional).

**Verify**: `cargo nextest run -p parallax-server -E 'test(/live/)'` → pass.

### Step 9: Sentry HTTP status mapping

Router-level tests via `tower::ServiceExt::oneshot` in
`crates/parallax-server/src/sentry_http.rs` tests: one per `RejectReason` →
assert non-2xx status per the mapping at `:216`; over-limit body → 413;
duplicate event id with DIFFERENT envelope bytes → the ack-ledger's
documented outcome; happy path → 200 + ack.

**Verify**: `cargo nextest run -p parallax-server -E 'test(/sentry/)'` → pass.

### Step 10: Outbox claim for real

Delete the `#[cfg(test)]` shadow (`claim_is_available` + consts) in
`crates/parallax-server/src/alerting/delivery.rs:174-193` OR promote it to
production code actually called by the claim path — whichever matches the
CAS SQL's semantics; then add a two-concurrent-claimers test against a temp
Turso store in `crates/parallax-metadata/src/turso/alerts.rs` tests,
modeled on `turso/tests.rs:41`: exactly one claimer wins, loser gets none,
lease expiry allows re-claim.

**Verify**: `cargo nextest run -p parallax-metadata -E 'test(/claim/)'` →
includes the new concurrency test, pass.

### Step 11: clamp_limit boundaries

Direct tests in `crates/parallax-api/src/query_limits.rs`: `None` →
default; negative → default; `Some(0)` → document the chosen semantics in
a doc comment (0 rows today — keep and pin, or map to default: STOP and
ask ONLY if changing; pinning current behavior needs no approval);
`i32::MAX` → MAX_ROWS; default > MAX_ROWS → MAX_ROWS. Plus
`check_query_limits` fragment-cycle and named-operation tests.

**Verify**: `cargo nextest run -p parallax-api -E 'test(/limits|clamp/)'` → pass.

### Step 12: CLI end-to-end skeleton + prune guard

Add `assert_cmd` + `predicates` as dev-deps of parallax-cli. New
`crates/parallax-cli/tests/cli.rs`: `--help` exits 0; unknown command exits
2; retired `run` verb rejected; `metrics --run` rejected; **prune dry-run
against a temp HOME** (`HOME=<tempdir>` with a seeded fake data dir):
prints a plan, deletes nothing (dir hash unchanged); `prune` without
`--yes` on a TTY-less stdin does not delete; `--execute --yes` deletes only
expected classes. JSON output shape asserted for `doctor --json` (if flag
exists — check `src/doctor.rs`; else `prune --json`).

**Verify**: `cargo nextest run -p parallax-cli` → new integration tests
pass; the dry-run test proves byte-identical data dir.

## Test plan

This plan IS tests. Net-new count expectation: ≥60 new test fns across 8
crates. Two production refactors (spool error + parser dedup, hash
expect) each carry their own regression tests above.

## Done criteria

- [ ] `cargo xtask integration` runs the real-engine nextest profile (not
      doctests); `cargo xtask doctests` exists.
- [ ] All 20 redaction detectors have positive+negative table rows; ordering
      test present.
- [ ] Arrow decode tests cover every `DataType` arm in
      `array_value_to_json`; tautology test deleted.
- [ ] `decimate_points`/`bound_metric_windows`/`canonical_hash` tested;
      `unwrap_or_default` removed from hash.rs + bounding token estimate.
- [ ] Spool corruption suite passes; `count_pspl_frames` single
      implementation; oversized payload errors.
- [ ] Ingest characterization tests for exp-histogram/summary/None arms.
- [ ] `cargo xtask ci && cargo xtask lint && cargo xtask test && cargo xtask arch` all green.
- [ ] `plans/README.md` row updated.

## STOP conditions

1. Drift check fails on any excerpt above.
2. Step 1 reveals CI depends on `integration`==doctests in a way that can't
   be updated in the same PR — report the coupling.
3. The real-engine suite fails on main before your changes — pre-existing
   red, report, don't chase.
4. A redaction canary exposes a detector defect needing more than a
   one-line regex fix — report as a plan-166 discrepancy instead of
   redesigning detectors here.
5. Changing `count_pspl_frames` to Err breaks a caller that relies on
   Ok(0)-for-corrupt — report the call site and its intent first.

## Maintenance notes

- The detector table is the template for future detectors: adding one
  without a table row should fail review.
- The characterization tests in Step 6 are intentionally behavior-pinning;
  plan 166's exponential-histogram decision will rewrite them — that's
  expected, they exist to make that rewrite deliberate.
- Reviewer: check no test asserts on nondeterministic ordering (use sorted
  comparisons like the existing suites do).
