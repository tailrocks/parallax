# Plan 021: Anchored log/span queries keep the newest rows (DESC + LIMIT), plus a severity upper bound for range filtering

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat f7f2c17..HEAD -- crates/parallax-storage/src/greptime.rs crates/parallax-storage/src/memory.rs crates/parallax-api/src/lib.rs ui/src/routes/runs.\$runId.tsx`
> Plans 014/017 (logs/runs redesigns) may have moved UI code — the storage/API
> fixes here stand alone; STOP only if the storage queries changed.

## Status

- **Priority**: P2
- **Effort**: S
- **Risk**: LOW
- **Depends on**: none (backend); UI toggle belongs to plan 017 — coordination note below
- **Category**: bug
- **Planned at**: commit `f7f2c17`, 2026-07-03

## Why this matters

Run- and trace-anchored reads order **ASC with LIMIT**, then the run page re-sorts newest-first. For any run with more rows than the limit (a `--debug` jackin run easily exceeds 200), the query keeps the **oldest** N and the newest lines — the ones an operator triaging a just-failed run needs — never reach the client. The run page's "newest-on-top" contract is silently inverted into "oldest N, reversed". Related range gap: the logs API filters severity as min-threshold only (`severity_number >= min`); a true range (e.g. "DEBUG only", "everything below WARN") needs an upper bound — cheap to add while in these clauses, and it unblocks the jackin-side workflow of auditing what noise ships at each tier.

## Current state

(From recon of `f7f2c17`; re-verify each site before editing.)

- `crates/parallax-storage/src/greptime.rs:674-681` — `logs_by_run`: `ORDER BY timestamp ASC LIMIT n`.
- `crates/parallax-storage/src/greptime.rs:662-672` — `spans_by_run`: same pattern.
- General `logs` resolver: `crates/parallax-api/src/lib.rs:936-995` — takes `severityMin: Option<i32>`, storage clause at `greptime.rs:498-517` (`"severity_number" >= {min}` inside `log_filter_clauses`), plus in-memory re-filter after a capped anchored fetch (`lib.rs:956-995`).
- UI consumer: `ui/src/routes/runs.$runId.tsx:242-256` merges loaded + live logs and sorts newest-first (comment at `:256`: "every run-page surface reads newest-on-top"); loader limit 200.
- Severity mapping already numeric end-to-end: `ui/src/routes/logs.tsx:48-54` maps All/Debug+5/Info+9/Warn+13/Error+17 to `severityMin`.
- The in-memory adapter `crates/parallax-storage/src/memory.rs` mirrors every `TelemetryStore` method — **any signature/semantic change must land in both** (repo hard rule).
- SQL-injection posture note (do not regress): storage SQL is `format!`-assembled with `escape()` for literals; numeric params must be formatted as integers (`{}` on `i32`), never string-interpolated user text.
- Conventions: main-branch, `-s`, one Claude co-author trailer; Rust nextest + clippy `-D warnings`; UI (only if touched): bun-only, strict TS, but this plan intends **no UI changes** — plan 017 owns the run-page INFO+ default toggle and should consume this fix.

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| fmt/clippy | `rtk cargo fmt --all -- --check` ; `rtk cargo clippy --workspace --all-targets --locked -- -D warnings` | exit 0 |
| Tests | `rtk cargo nextest run --workspace --all-targets` | pass |

## Scope

**In scope**: `crates/parallax-storage/src/greptime.rs` (anchored queries + severity clause), `crates/parallax-storage/src/adapter.rs` (only if the trait signature grows `severity_max`), `crates/parallax-storage/src/memory.rs` (parity), `crates/parallax-api/src/lib.rs` (`severityMax` GraphQL arg on `logs` + `logCountSeries`), tests.

**Out of scope**: any `ui/` change (plan 017/014 own the surfaces); pagination/cursoring (plan 010 of the active program owns trace paging); changing default limits.

## Git workflow

- `main`; commits: `fix(storage): return newest rows from run/trace-anchored queries` and `feat(api): severityMax bound on log queries`; `-s` + one Claude co-author trailer each.

## Steps

### Step 1: Newest-first anchored reads

In `greptime.rs`: `logs_by_run` and `spans_by_run` (and any sibling `*_by_trace` anchored reads — grep `rg -n "ORDER BY timestamp ASC LIMIT" crates/parallax-storage/src/greptime.rs` and audit each hit's consumers) switch to `ORDER BY timestamp DESC LIMIT {n}`, then **reverse the fetched rows in Rust before returning** so the function's return order (ascending) is unchanged for existing consumers — the semantic fix is *which* rows survive the LIMIT, not the return order. Mirror the same change in `memory.rs`'s implementations (sort DESC, truncate, reverse).

**Verify**: unit/integration test against the in-memory adapter: insert limit+50 rows, fetch with limit → returned set contains the NEWEST limit rows in ascending order (assert first returned row is row #51, last is the newest). If an m-series GreptimeDB-backed test covers `logs_by_run` (grep `rg -rn "logs_by_run\|logsByRun" crates/parallax-server/tests/`), extend it with an over-limit case.

### Step 2: `severityMax`

- Storage: `log_filter_clauses` (`greptime.rs:498`) gains `severity_max: Option<i32>` → `"severity_number" <= {max}` (integer-formatted). Thread through the `TelemetryStore` trait method(s) that accept `severity_min` today (find via `rg -n "severity_min" crates/parallax-storage/src`); update `memory.rs` in the same commit.
- API: `logs(...)` and `logCountSeries(...)` resolvers (`lib.rs:936`, and the count-series resolver — locate via `rg -n "logCountSeries\|severityMin" crates/parallax-api/src/lib.rs`) gain `severityMax: Option<i32>`, passed straight through; the post-fetch in-memory re-filter (`lib.rs:956-995`) applies the same bound.

**Verify**: in-memory adapter test: rows at severities 5/9/13/17; `min=5,max=8` → only the DEBUG row. GraphQL schema smoke: existing m2_api test pattern (grep `crates/parallax-server/tests/m2_api*`) — add a query exercising `severityMax` if the harness allows; otherwise unit-test at the storage layer and note it.

### Step 3: Coordination notes for the UI program

Append to `plans/README.md` dependency notes: plan 017 (runs redesign) should (a) rely on `logsByRun` now returning the newest rows and (b) default the run page to INFO+ (`severityMin: 9`) with a DEBUG/TRACE toggle — no backend blocker remains; plan 014 (logs redesign) may expose the severity range using `severityMax`.

**Verify**: `rtk cargo nextest run --workspace --all-targets`, fmt, clippy → green.

## Test plan

Step 1 over-limit newest-rows test (in-memory + optional Greptime-backed), Step 2 range test, plus a no-regression run of the full suite. Pattern: existing storage tests in `memory.rs` inline module.

## Done criteria

- [ ] `rg -n "ORDER BY timestamp ASC LIMIT" crates/parallax-storage/src/greptime.rs` → no hits in run/trace-anchored reads (time-window scans may legitimately remain — audit each)
- [ ] Over-limit test proves newest rows returned (both adapters)
- [ ] `severityMax` filters in both adapters + exposed on `logs`/`logCountSeries`
- [ ] memory.rs parity for every touched method
- [ ] fmt/clippy/nextest green; `plans/README.md` updated (status row + coordination notes)

## STOP conditions

- A consumer depends on the anchored reads returning the OLDEST rows (e.g. a bundle builder reading "first N lines" — grep `rg -n "logs_by_run\|spans_by_run" crates/` and check `bundle.rs`): if any, the fix needs a `direction` parameter instead of a blanket flip — report first.
- The GraphQL schema change breaks a committed UI query (grep `ui/src/lib/api.ts` for `severityMin` usage — additive optional args should be safe; STOP if the client pins the full signature).
- GreptimeDB rejects `DESC` + `LIMIT` on these tables' index shape with a performance cliff (visible in test runtime) — report numbers.

## Maintenance notes

- Plan 017's run-page severity toggle is the user-visible payoff; without it the fix is invisible except on over-limit runs.
- When plan 010 (trace paging) lands cursor-based reads, fold this newest-first semantic into its cursor design rather than keeping two shapes.
