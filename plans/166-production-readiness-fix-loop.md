# Plan 166: Drive every verified discrepancy to zero — the production-readiness fix loop

> **Executor instructions**: This plan is a *loop contract*, not a linear
> script: iterate Steps 2–5 until the exit criteria hold. Run every
> verification command and confirm the expected result. If anything in "STOP
> conditions" occurs, stop and report — do not improvise. Update this plan's
> row in `plans/README.md` each session (it stays IN PROGRESS across
> sessions until the done criteria hold).
>
> **Dependency gate (run first)**:
> `grep -c "^DISCREPANCY:\|^CLOSED:\|^PROMOTED" docs/research/reference/feature-inventory-and-playground-verification.md`
> → must be ≥ 1 (plan 165 wrote the W5 list with these tokens). Zero
> matches = dependency not met → STOP.
>
> **Drift check**: `git diff --stat f6208070..HEAD -- docs/research/reference/feature-inventory-and-playground-verification.md plans/ docs/guide/`
> plus, before EACH fix cluster (Step 2), diff the exact files that cluster
> will touch against their "Current state" reproduction — this plan spans
> sessions, so per-cluster drift checks replace a single global one.

## Status

- **Priority**: P1
- **Effort**: L (multi-session)
- **Risk**: HIGH (real product fixes across ingest/API/UI; each mitigated by the per-fix gates below)
- **Depends on**: plans/165-user-lens-comparison.md,
  plans/167-agent-browser-ui-verification.md (UI discrepancies enter the
  same W5 list)
- **Category**: bug
- **Planned at**: parallax `f6208070`, playground `6e0a0d5`, 2026-08-13

## Why this matters

The program's goal (inventory doc, Workstream 5): every Parallax feature
production-ready — verified by scripted scenario, compared against the
roster, **zero known bugs**. Plans 162–165 produce the evidence; this plan
consumes the discrepancy list and drives it to empty. "Production-ready"
here is machine-defined: every c-series scenario green, every W5 discrepancy
closed (fixed + re-verified) or explicitly promoted to a numbered blocked
plan with a named blocker.

## Current state

- Input: the Workstream 5 discrepancy list in
  `docs/research/reference/feature-inventory-and-playground-verification.md`
  (created by plan 165), each row:
  `feature | scenario | backend(s) | observed | expected | suspected owner`.
- Fix targets live in the parallax repo: `crates/` (17-crate workspace, tiers
  enforced by `cargo xtask arch`) and `ui/` (TanStack Start, ownership in
  `ui/AGENTS.md` + `ratchet.toml`). Regression net: c-series scenarios
  (playground repo) + the parallax test/gate stack.
- Repository engineering rules that bind every fix (parallax `AGENTS.md`):
  - Root-cause first: "first find why architecture allowed bug class; prefer
    structural fix removing enabling condition" (operator rule). Symptom
    patch only if root fix infeasible — name the deferred root cause in the
    commit and in the discrepancy row.
  - GreptimeDB+Turso only; native raw-signal tables; native TLS never
    rustls; Bun only; zero-copy ingest hot path (decode once, move
    ownership, never clone telemetry on the hot path).
  - cargo-nextest runner; fmt + clippy strict zero warnings.
  - Contract changes go to
    `docs/research/architecture/v1-implementation-spec.md` first, then code;
    GraphQL SDL drift-gated (`cargo xtask ui graphql check`).
- Known pre-triaged items that belong in this loop's first sessions:
  1. Guides drift: `docs/guide/cli.md`, `agent-howto.md`, `conventions.md`,
     `jackin.md`, AND `quickstart.md` (`docs/guide/quickstart.md:106,110`)
     still document `parallax run …` / `parallax.run.id`; live CLI + spec
     use `invocation` / `cli.invocation.id` (CLI rejects the retired `run`
     alias). Fix every guide the done-criteria grep finds — the five named
     files are the known set, the grep is the authority. Docs-only fix.
  2. Exponential-histogram drop: `crates/parallax-ingest` `normalize_metrics`
     silently drops exponential histograms/summaries (`_ => {}` arm,
     `crates/parallax-ingest/src/metrics.rs`; CODE-CONFIRMED in playground
     `VERIFICATION.md` 2026-07-17). The playground JVM tier emits base2
     exponential histograms by default (`deploy/docker-compose.yml:143`).
     (Note: the playground JVM tier emits exponential histograms via an
     explicit "W5 probe" env var
     `OTEL_EXPORTER_OTLP_METRICS_DEFAULT_HISTOGRAM_AGGREGATION` at
     `deploy/docker-compose.yml:141-143` — not an SDK default; keep the
     probe in place, it is the reproduction.)
     Decide per spec-first rule: either model exponential histograms (spec
     update → ingest + query + UI) or make the drop *observable* (ingest
     counter + doctor/UI signal) — silent data loss is the bug even if
     support stays deferred.
  3. Sentry multi-SDK compatibility ledger (unproven per README Working
     Direction #2) — c8 scenario results decide: fix envelope handling per
     SDK or record the proven ledger.
- Existing blocked plans 089 (extension-table gRPC writes) and 114 (legacy
  spool reader) stay independent — do not merge their scope into this loop.

## Commands you will need

Parallax repo gates (every fix session, before PR):

| Purpose | Command | Expected on success |
|---|---|---|
| Fast partition | `cargo xtask ci` | exit 0 |
| Lint | `cargo xtask lint` | zero warnings |
| Workspace tiers | `cargo xtask arch` | exit 0 (required for any `crates/` change) |
| Tests | `cargo xtask test` | all pass |
| Single new test | `cargo nextest run -p <crate> -E 'test(<test_name>)'` | 1 test listed; fails pre-fix, passes post-fix |
| Integration | `cargo xtask integration` | all pass |
| Policy families | `cargo xtask policy` | all pass |
| GraphQL drift | `cargo xtask ui graphql check` | no drift |
| UI gates | `cargo xtask ui` | exit 0 |
| Browser lanes (UI-touching fixes) | `cargo xtask browser-contracts-serve` (+ `browser-full-stack-serve` for storage-visible changes) | Playwright green |
| Docs links | `cargo xtask docs links` | passes |
| Scenario re-verify | `cd ../parallax-telemetry-playground/scenarios && ./run.sh <id>` | exit 0 |

## Scope

**In scope**: any parallax `crates/`/`ui/`/`docs/guide/` file a triaged
discrepancy traces to; playground scenario scripts only for *script bugs*
(wrong assert), never to mask a product bug; the inventory doc's W5 list
(status column); `docs/research/architecture/v1-implementation-spec.md` when
a fix changes a contract; `plans/` for promotions.

**Out of scope**:
- New features beyond discrepancy closure (no scope creep: profiles signal,
  SLO/burn-rate, GraphQL subscriptions, alert email stay inventory "gaps"
  unless the operator promotes them — they are absences, not bugs).
- Plans 089/114 scope.
- Weakening a scenario assert to turn it green.

## Git workflow

PR-only `main`. **One discrepancy cluster per branch+PR**, smallest
reviewable unit; never a second parallel PR in the same repo. `git commit -s`,
Conventional Commits (`fix(scope): …`), agent trailer per `COMMITS.md`.
Red CI rule: if CI is red, first rebase on latest `origin/main` before
treating it as a defect (`AGENTS.md` operator rule 2026-08-13).

## Steps

### Step 1: Triage the list (once per loop entry)

Order the W5 list: (a) data-loss/correctness in ingest or evidence, (b) wrong
answers in API/UI, (c) workflow breakage in CLI, (d) docs drift, (e) cosmetic.
Within a class, playground-owned script bugs first (cheapest, unblock
re-verification). Confirm each row's suspected owner by reproducing it once
locally.

**Verify**: every row has owner ∈ {parallax-code, parallax-docs,
playground-script, product-gap} and a priority class; product-gap rows moved
out of the fix queue into the inventory "gaps" section.

### Step 2: Fix one cluster (repeat)

For the top cluster: write the failing regression test FIRST (unit/
integration in the owning crate, or UI test per `ui/` conventions — the
c-series scenario alone is not the regression net for a code fix), then the
root-cause fix per the rules above. Spec-first if a contract moves.

**Verify**: `cargo nextest run -p <crate> -E 'test(<test_name>)'` fails
before the fix and passes after (record both runs in the PR); full gate
table above green (`arch` included when `crates/` changed).

### Step 3: Re-verify through the playground (repeat)

Re-run the discrepancy's scenario(s) against the rebuilt Parallax
(`cargo run -p parallax-cli -- serve` or reinstalled preview) on the pinned
lab. Flip the W5 row to CLOSED with date + evidence line; update
`VERIFICATION.md` (playground) if it carried the DISCREPANCY marker.

**Verify**: `./run.sh <id>` exit 0; W5 row updated in the same PR as the fix
(parallax side) or a paired playground PR.

### Step 4: Promote what cannot be fixed now (as needed)

A discrepancy blocked on an upstream/external dependency becomes a new
numbered plan (`plans/NNN-…`, next free number, following the plan-089 shape:
residual, blocker, recheck stamp, done criteria, STOP) + index row; the W5
row gets `PROMOTED → plans/NNN`.

**Verify**: `plans/README.md` row exists; no W5 row left OPEN without either
a fix PR or a promotion.

### Step 5: Loop exit audit

When the W5 list has no OPEN rows: run the full c-sweep + smoke one final
time at pinned versions; confirm zero DISCREPANCY markers left in
`VERIFICATION.md`'s current section; update the inventory doc header with
the completion date; set this plan DONE and delete it + its row per the
`plans/` lifecycle (terminal work leaves `plans/`; durable evidence stays in
the inventory doc + VERIFICATION.md).

**Verify**: done criteria checklist below all green in one session.

## Test plan

Per fix: mandatory failing-first regression test in the owning crate/UI
layer + scenario re-run. Per loop exit: full parallax gate stack + full
c-sweep green. No fix merges on scenario-green alone.

## Done criteria

- [x] W5 discrepancy list: zero OPEN rows (all CLOSED with evidence or
      PROMOTED to a numbered blocked plan). #418 CLOSED 2026-08-14;
      165+167 still blocked so this plan stays BLOCKED.
- [ ] Full c-series sweep green at pinned versions (`run.sh c1..c11` exit 0
      — c11 is the agent-browser UI pass from plan 167; re-run after every
      UI-touching fix).
- [ ] `cargo xtask ci && cargo xtask lint && cargo xtask test && cargo xtask
      integration && cargo xtask policy && cargo xtask ui graphql check` all
      exit 0 on `main`.
- [ ] Guides drift item closed:
      `grep -rln "parallax run \|parallax\.run\.id" docs/guide/` → no output
      (covers quickstart.md and any file the named five missed).
- [ ] Exponential-histogram decision executed spec-first (either modeled or
      observable-drop), proven by a named regression test:
      `cargo nextest run -p parallax-ingest -E 'test(/exponential/)'` →
      ≥ 1 test listed, all pass.
- [ ] Inventory doc stamped with completion date; this plan file + row
      removed per lifecycle.

## STOP conditions

1. Plan-165's W5 list doesn't exist or has no owner triage — dependency gap.
2. A fix requires violating a mandatory constraint (fallback engine, rustls,
   hand-rolled raw-signal table, telemetry clone on hot path) — report; the
   constraint wins.
3. A root fix demands a contract change the spec doesn't sanction and the
   operator hasn't approved — write the spec proposal, PR it, wait.
4. The same scenario flips a fix red↔green across runs (flaky evidence) —
   report the nondeterminism before continuing the loop.
5. Two consecutive sessions close zero rows — the loop is stuck; report why.

## Maintenance notes

- After this plan completes, the standing regression net is: c-series sweep
  at every backend pin bump (plan-162 note) + the parallax gate stack in CI.
- Reviewer per fix PR: check the regression test actually encodes the
  discrepancy (not just any test), and that no scenario assert was weakened.
- Deferred product gaps (profiles, SLO, subscriptions, alert email, browser
  sessions) remain recorded in the inventory doc's gap section for a future
  roadmap decision — explicitly not smuggled into this loop.
