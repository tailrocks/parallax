# Plan 165: Run the full playground sweep and record a user-lens comparison across all backends

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving on. If
> anything in "STOP conditions" occurs, stop and report — do not improvise.
> When done, update this plan's row in `plans/README.md` (parallax repo).
>
> **Drift check (run first)**:
> `git diff --stat f6208070..HEAD -- docs/research/market/competitors/ docs/research/reference/feature-inventory-and-playground-verification.md`
> and `git -C ../parallax-telemetry-playground diff --stat 6e0a0d5..HEAD -- VERIFICATION.md docs/corner-case-matrix.md scenarios/README.md`.
> On mismatch with "Current state", STOP.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: LOW (read/record work; the risk is bias, handled by the no-bias rule below)
- **Depends on**: plans/162-fanout-lab-backend-pins.md, plans/163-playground-example-upgrades.md, plans/164-playground-feature-coverage.md
- **Category**: docs
- **Planned at**: parallax `f6208070`, playground `6e0a0d5`, 2026-08-13

## Why this matters

The program's fourth workstream (inventory doc, Workstream 4): with identical
telemetry fanned out to every backend, record — feature by feature, as a
practicing user of each product — what each backend can and cannot do, at
what fidelity, with what workflow quality. This produces the evidence rows
that (a) drive plan 166's fix list for Parallax and (b) refresh the
competitor matrices that are currently part-stale (`PROGRESS.md` tracks
🔴 stale cells). Binding rule from `docs/research/market/competitors/README.md`:
**"a comparison that always favors Parallax is a failure state."**

## Current state

- Scenario sweep available after plan 164: a-series (~30), b-series chaos,
  c-series Parallax journeys (`../parallax-telemetry-playground/scenarios/`).
  Each script prints per-backend "Check in <backend> UI" lines.
- Backends (plan-162 pins): Parallax (host), OpenObserve, Maple, SigNoz,
  Sentry self-hosted. Sentry receives traces+logs only (no OTLP metrics —
  `bench/otlp-fanout/rotel.env.example:51`).
- Recording surfaces:
  - Playground `VERIFICATION.md` — per-scenario machine results (the shape
    used historically: per-backend PASS/FAIL/PRODUCT-LIMITED rows with dated
    evidence, e.g. the "Multi-backend fan-out residual" table).
  - Playground `docs/corner-case-matrix.md` — UI-rendering risk → scenario id.
  - Parallax `docs/research/market/competitors/` — canonical no-bias matrix:
    `README.md` (axes), `comparison-set.md` (roster, last reviewed
    2026-07-17), `PROGRESS.md` (per-cell verification status:
    ✅ verified / 🟡 inherited / 🔴 stale / ⚪ benchmark-dependent), deep-dives
    `parallax-vs-{maple,openobserve,signoz,sentry,…}.md`.
  - Parallax inventory doc
    `docs/research/reference/feature-inventory-and-playground-verification.md`
    — Workstream 4/5 checklists.
- User-lens feature axes = the inventory doc's feature sections: ingest
  fidelity, trace UX (waterfall/critical path/compare), log UX (filters/
  patterns/live tail), metrics UX (catalog/query/exemplars), errors/issues
  (grouping/lifecycle), evidence/agent surface (bundles/MCP — Parallax-only
  by design, record competitors' nearest equivalent), service map, alerting,
  dashboards, saved state, SQL/raw query, sessions/CLI-invocation evidence,
  test reporting.

## Commands you will need

| Purpose | Command | Expected on success |
|---|---|---|
| Stack + lab up | plan-162/163 procedures | all backends healthy |
| Full sweep | `cd ../parallax-telemetry-playground/scenarios && for s in a*.sh b*.sh c*.sh; do ./run.sh ${s%.sh} || echo RED ${s%.sh}; done 2>&1 | tee /tmp/sweep.log` | log of every scenario; RED lines only where VERIFICATION.md already records a DISCREPANCY |
| Ambient load (during UI review) | `docker compose -f deploy/docker-compose.yml --profile demo up loadgen` | k6 profile running |
| Docs links | `cargo xtask docs links` (parallax) | passes |

## Scope

**In scope** (parallax): `docs/research/market/competitors/PROGRESS.md`,
`comparison-set.md` (review date + roster state), the five deep-dive files
`parallax-vs-{maple,openobserve,signoz,sentry,hyperdx}.md` — hyperdx only if
its cells change from inherited evidence — plus
`docs/research/reference/feature-inventory-and-playground-verification.md`
(Workstream 4 results + Workstream 5 discrepancy list).
**In scope** (playground): `VERIFICATION.md` (dated sweep record),
`docs/corner-case-matrix.md` (new rows).

**Out of scope**:
- Any code change in either repo (pure run-and-record).
- Benchmark/performance numbers — ⚪ cells stay benchmark-dependent
  (separate benchmark protocol owns those; see `AGENTS.md` four-build rule).
- Re-scoring competitors not fed by the lab (Grafana, Datadog, …): their
  cells keep their existing evidence state.

## Git workflow

PR-only `main`, one branch + one PR per repo, `git commit -s`, Conventional
Commits, agent trailer per `COMMITS.md`.

## Steps

### Step 1: Fresh full-stack run

Bring up the pinned lab + playground (plans 162/163), verify every backend
healthy (smoke), start ambient load, run the full sweep command, keep
`/tmp/sweep.log`.

**Verify**: sweep completes; RED list matches the DISCREPANCY list already in
`VERIFICATION.md` (new REDs are new discrepancies — record them, they do not
block this plan).

### Step 2: Per-feature user-lens review

For each feature axis (list in "Current state"), in each backend's actual UI,
perform the user task the scenario sets up (find the failing trace, read the
N+1, tail the logs, build the alert, …). Record per cell:
`capability (full/partial/absent) | fidelity notes | workflow friction |
evidence (scenario id + screenshot-worthy observation)`. Sentry metrics cells
= N/A-by-transport, not absent. Parallax-only surfaces (bundle, MCP,
invocation evidence): record the competitor's nearest-equivalent workflow
honestly (e.g. Sentry issue context, SigNoz trace detail export).

**Verify**: a filled grid exists (features × 5 backends) — attach it as the
Workstream 4 results table in the inventory doc.

### Step 3: Write the records

- Playground `VERIFICATION.md`: dated section "Full sweep <date> at pinned
  versions" — per-scenario per-backend PASS/FAIL/PRODUCT-LIMITED, matching
  the existing table idiom.
- `docs/corner-case-matrix.md`: add rows for rendering risks found.
- Parallax `PROGRESS.md`: flip cells verified this run to ✅ with date +
  scenario id; leave unverified cells untouched.
- Deep-dives for the four lab competitors: update only sections contradicted
  or newly evidenced by the run; keep the multi-angle rule (capability +
  price/TCO + license + ops).
- Inventory doc: Workstream 4 grid + Workstream 5 input list — every
  discrepancy as `feature | scenario | backend(s) | observed | expected |
  suspected owner (parallax bug / playground bug / product gap)`.

**Verify**: `cargo xtask docs links` passes; `grep -c "DISCREPANCY"` in the
inventory doc's W5 list equals the count in `VERIFICATION.md`'s new section.

### Step 4: Roster review note

Update `comparison-set.md` review date. Record the standing deferral: lab
roster stays at 5 backends; Grafana LGTM / HyperDX / Uptrace addition is a
separate decision (pointer to `plans/README.md` note).

**Verify**: `comparison-set.md` shows the new review date.

## Test plan

Not a code plan. The sweep log + filled grid are the artifacts. Reviewer
spot-checks two cells per backend against the live UI.

## Done criteria

- [ ] Full sweep executed at pinned versions; log preserved (PR description
      or a dated section in `VERIFICATION.md`).
- [ ] Features × backends grid complete in the inventory doc (no empty cells;
      N/A only with a stated transport/product reason).
- [ ] `PROGRESS.md` cells updated with date + scenario evidence.
- [ ] Workstream 5 discrepancy list complete with suspected-owner triage.
- [ ] `cargo xtask docs links` passes.
- [ ] `plans/README.md` row updated.

## STOP conditions

1. Drift check fails.
2. A backend is down/unhealthy at the pinned version and one restart doesn't
   recover it — report; don't swap the pin mid-run.
3. The grid would exceed a day of manual UI work because scenario "Check in
   UI" lines are missing/wrong — fix the *lines* (plan-164 scope) first via a
   small follow-up, don't improvise the grid from memory.
4. You catch yourself scoring a Parallax weakness as a strength — re-read
   the no-bias rule; if genuinely ambiguous, record the ambiguity verbatim.

## Maintenance notes

- This grid is a snapshot; it goes stale at the next version bump. The
  refresh procedure is: re-run this plan's steps at new pins (plan 162
  maintenance note).
- Reviewer: scrutinize Parallax-favoring cells hardest (the failure state is
  bias, per the competitors README).
- The W5 discrepancy list is plan 166's direct input — its quality bounds the
  fix loop's completeness.
