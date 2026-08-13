# Plan 170: Playwright coverage for every critical user flow

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving on. If
> anything in "STOP conditions" occurs, stop and report — do not improvise.
> When done, update this plan's row in `plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat f6208070..HEAD -- ui/tests/ ui/playwright.config.ts ui/test-matrix.json ui/src/routes/tests/ ui/src/workers/ ui/src/features/ecosystem/workers/ crates/parallax-test-support/src/browser/`
> — on mismatch with the excerpts below, STOP.
>
> **Matrix gate (applies to EVERY step adding a test file)**: the
> `ui.tests` policy enforces EXACT equality on `ui/test-matrix.json`
> ratchets (`test_files`, `test_cases`) AND errors on any discovered test
> file with no matrix entry
> (`crates/parallax-xtask/src/policy/ui_tests.rs:166-190`). Every new spec
> or Vitest file needs a matrix entry + updated ratchet counts in the same
> commit; verify with `cargo xtask policy --only ui.tests`.

## Status

- **Priority**: P1
- **Effort**: L (staged; dataset seam first, then per-surface S/M specs)
- **Risk**: LOW-MED (new datasets are additive; diagnostics-auto will fail
  currently-green specs by design — staged rollout below)
- **Depends on**: none (complements plan 167's live agent pass; this is the
  in-repo CI lane)
- **Category**: tests
- **Planned at**: parallax `f6208070`, 2026-08-13

## Why this matters

The UI has 42 Playwright tests but real interaction coverage for only 2 of
21 surfaces (shell, investigations). 11 of the SDL's 14 mutations have
zero browser exercise; `/traces/$traceId` — the deepest surface and
highest-churn route file — is never opened by any spec; `/alerts` and
`/metrics` appear in no spec at all; several full-stack assertions are
unfalsifiable (pass on a bare `<h1>`); and the diagnostics gate that would
catch crashed sub-trees is opt-in and used by 2 tests. The blocker for all
of it is the dataset seam: exactly two seed datasets exist, so the fast
contracts lane can't reach any other surface. QA priority: seed seam →
mutation CRUD specs → deepest read surfaces → assertion honesty → gates.

## Current state (verified excerpts)

- Dataset seam: `ui/tests/e2e/datasets/catalog.ts:3` —
  `export type ProductDatasetId = "shell-empty" | "investigations-pilot"`;
  Rust facade mirror `crates/parallax-test-support/src/browser/datasets.rs`
  (`ShellEmpty`/`InvestigationsPilot` → `as_str`). Reset/snapshot control
  plane exists (`ui/tests/e2e/fixtures/product-fixture.ts:77-91`).
  `ui/test-matrix.json` holds 11 `status: "reserved"` playwright entries;
  9 point at spec files that don't exist yet (the shell + investigations
  reserved entries already have living specs — reconcile those two rows
  to their real status while in the file).
- Diagnostics fixture: `ui/tests/e2e/fixtures/test.ts:78-90` — declared
  WITHOUT `{ auto: true }` (unlike `fixedTime`/`seeded`); catches
  console-error/pageerror/external-network/dialog/download; only
  `contracts/shell.spec.ts` and `smoke/foundation.spec.ts` opt in.
- Mutation ledger — the SDL (`ui/graphql/schema.graphql` `type Mutation`)
  has 14 mutations. Browser-covered today: 3 —
  investigationSave/Delete (`contracts/investigations.spec.ts:47,68`),
  issueSetStatus (`full-stack/storage-composition.spec.ts:25,46`).
  NOT covered, browser-relevant (9): dashboardSave, dashboardDelete,
  savedViewSave, savedViewDelete (used TWICE client-side: logs saved
  views at `ui/src/features/logs/components/logs-page.tsx:294-318`
  (string-interpolated, no escaping test) AND the SQL snippet documents
  `ui/src/features/sql/api/sql-snippet-save.graphql` /
  `sql-snippet-delete.graphql`, which are CLIENT document names over
  savedViewSave/Delete — both call paths need exercise), alertRuleSave,
  alertRuleDelete, alertRuleSetEnabled, alertDestinationSave,
  alertDestinationDelete (`ui/src/features/alerts/api/alerts-gql.ts:120,155`
  — string-built over `gqlString`).
  Not browser-relevant (2): invocationStart, invocationFinish (CLI
  wrapper surface — covered by the playground c2 scenario, plan 164);
  record this split in the PR.
- Never-navigated routes: `/traces/$traceId` (trace-detail-page.tsx 49.7K,
  highest churn), `/services/$service`, `/tests/$caseKey`, `/metrics`,
  `/metrics/$metricName`, `/alerts`.
- Unfalsifiable asserts: `full-stack/{ecosystem,dashboards,sql,overview}.spec.ts`
  heading-only; `full-stack/logs.spec.ts:12-15` `.or()` across two
  outcomes; `full-stack/traces.spec.ts:11` asserts fixture length;
  `traces-live-performance.spec.ts:29-30` asserts `count >= 1` under a
  "dedupe" docstring (correct pattern exists at `:51-53`).
- Flake-risk: conditional `if (await goLive.isVisible().catch(() => false))`
  in `runs-live-performance.spec.ts:22-32,50-61`; in-body cleanup in
  `storage-composition.spec.ts:46-48` (workers:1 lane inherits dirty
  state); ~25 magic 20s/45s timeouts; `retries: 0`
  (`ui/playwright.config.ts:49`).
- Near-zero-value route tests: 7 files asserting
  `expect(typeof loadX).toBe("function")`
  (`ui/src/routes/tests/-logs-routes.test.tsx:7-8` et al); good pattern to
  keep = search-param coercion in `-traces-routes.test.tsx:12-27`.
- Untested pure models (Vitest gaps, S each):
  `ecosystem/model/service-map-layout-engine.ts` (145L, worker-executed,
  zero tests, duplicated worker entry at `ui/src/workers/` +
  `ui/src/features/ecosystem/workers/`), `services/model/service-detail.ts`
  (211L RED math), `tests/model/test-summary.ts` (97L),
  `services/model/services-search.ts` (87L),
  `overview/model/overview-chart-helpers.ts` (69L).
- Clean: zero `.skip/.only/.fixme` repo-wide; `forbidOnly` on CI.
- Policy: NO `page.route` GraphQL stubs (`fixtures/test.ts:60` records it);
  extend datasets instead. Lanes run via
  `cargo xtask browser-{foundation,contracts,full-stack}-serve`.

## Commands you will need

| Purpose | Command | Expected |
|---|---|---|
| UI unit | `cd ui && bun run test` (see package.json scripts) | pass |
| Typecheck | `cd ui && bun run typecheck` | exit 0 |
| Contracts lane | `cd ui && bun run test:browser` | Playwright green (`test:browser` = contracts-chromium project; the `cargo xtask browser-*-serve` commands are the lanes' blocking webServer processes, NOT test runners — never invoke them to run tests) |
| Full-stack lane | `cd ui && bun run test:browser:full` | green |
| A11y / cross+mobile / visual | `cd ui && bun run test:browser:a11y` / `test:browser:cross` / `test:browser:visual` | green |
| One spec | `cd ui && bun run test:browser -- -g "<test name>"` | targeted spec runs |
| Visual golden update | `cd ui && bun run test:browser:visual:update` | goldens regenerated |
| Policy gates | `cargo xtask policy --only ui.tests && cargo xtask policy --only ui.browser-contracts` | pass |
| Rust seed build | `cargo xtask ci --fast` | pass (`ci` REQUIRES `--fast` or `--full`) |

## Scope

**In scope**: `crates/parallax-test-support/src/browser/datasets.rs` (+ its
seed builders), `ui/tests/e2e/**` (datasets catalog, fixtures, new +
strengthened specs), `ui/test-matrix.json`, `ui/playwright.config.ts`
(retries, shared timeout constants), `ui/src/routes/tests/-*.test.tsx`
(fold/delete typeof tests), new Vitest files under
`ui/src/features/*/tests/model/`, dedupe of the ecosystem worker entry
(pick ONE canonical path, update imports).

**Out of scope**: feature/product code changes (a failing new spec that
reveals a product bug → `DISCREPANCY:` row for plan 166, spec lands
`fixme`-free but may stay red only if the lane supports expected-fail —
otherwise record + skip-with-linked-discrepancy comment); visual-snapshot
expansion (explicitly rejected: pixel goldens on dense data tables = churn
> value); `page.route` stubbing (policy).

## Git workflow

PR-only `main`; stage as ~4 PRs (P0 seam+CRUD, P1 read-depth, P2 honesty+
gates, P3 vitest models); `git commit -s`; Conventional Commits; agent
trailer per `COMMITS.md`.

## Steps

### Step 1 (P0): Dataset seam — one new dataset end-to-end, then the rest

Extend `datasets.rs` with seed builders following `InvestigationsPilot`'s
shape, and mirror ids in `catalog.ts` + `product-fixture.ts:6`:
`logs-pilot` (N log rows across 2 services, 3 severities, one known body
string), `traces-pilot` (1 seeded trace with named root+children, one
error span), `dashboards-pilot` (1 dashboard, 1 widget), `sql-pilot`
(minimal telemetry for `SELECT count(*)`), `alerts-pilot` (1 rule, 1
destination, 1 resolved incident), `metrics-pilot` (2 metrics: gauge +
histogram with known series). Ship `logs-pilot` end-to-end FIRST (builder →
catalog → one passing spec) to prove the seam, then batch the rest.

**Verify**: `cargo xtask ci --fast` (Rust builders compile+unit-test);
`cd ui && bun run test:browser` green with the first new spec (matrix
entry added — see Matrix gate).

### Step 2 (P0): Mutation CRUD specs (contracts lane)

Model each on `contracts/investigations.spec.ts`:
- `contracts/dashboards.spec.ts`: create → add widget → save → cross-route
  nav → reload → persisted → delete (AlertDialog confirm) → snapshot
  postcondition.
- `contracts/alerts.spec.ts`: destination create (webhook URL) → rule
  create from template dialog → appears in Rules tab → enable toggle off/on
  → rule delete → destination delete. (This also end-to-end-proves the
  string-interpolated `alertRuleSaveMutation` escaping — include a rule
  name with quotes/backslash.)
- `contracts/sql.spec.ts`: run known-good query → result rows; invalid
  query → error surface (NOT empty state); snippet save → reload → present
  → delete.
- `contracts/logs-views.spec.ts`: where-clause chip add via editor → URL
  reflects filter → rows narrow → save view (name with a quote char) →
  reload → select view → filters restored → delete view.

**Verify**: `cd ui && bun run test:browser` green; `ui/test-matrix.json`:
reserved entries flipped to implemented where one exists (sql,
dashboards, logs); alerts and metrics have NO reserved entry — ADD new
entries for their specs; ratchet counts updated;
`cargo xtask policy --only ui.tests` green.

### Step 3 (P1): Deep-read surfaces (full-stack lane)

- `full-stack/trace-detail.spec.ts`: deep-link
  `/traces/${manifest.trace_id}?view=tree` → root span name visible; switch
  flame view; toggle critical path; open span detail panel. Assert
  roles/text/counts, never pixel geometry.
- `full-stack/service-detail.spec.ts`: `/services/<seeded>` → RED charts
  containers present with non-empty series, quick-links land pre-filtered.
- `full-stack/metrics.spec.ts`: catalog lists seeded metrics; workbench
  chart renders; aggregation selector constrained by kind; "Add to
  dashboard" opens dialog (graduation into save covered by Step 2
  dashboards spec).
- `full-stack/tests-detail.spec.ts`: `/tests/<seeded caseKey>` attempt
  chain renders, invocation link navigates.
- `full-stack/invocation-hub.spec.ts`: all 6 tabs switch and render
  seeded content (extends the 2-tab live specs).
- Command palette (contracts): ⌘K opens, paste seeded trace id → lands on
  trace detail.

**Verify**: full-stack lane green.

### Step 4 (P2): Assertion honesty + diagnostics gate

- Strengthen heading-only asserts: each full-stack spec asserts one
  seeded-data fact (named service row, widget name, span name). Split
  `logs.spec.ts` `.or()` into two named assertions. Replace
  `traces-live-performance.spec.ts:29-30` with the `toHaveCount(1)` row
  pattern from `:51-53`. Replace fixture self-checks with product asserts.
- Make `diagnostics` `{ auto: true }` on `productTest` first
  (`fixtures/test.ts:78`; note THREE fixtures define diagnostics — base
  `test` at `:43`, `productTest` at `:78`, `fullStackTest` at `:116` —
  this plan flips `productTest` then `fullStackTest`; leave base `test`
  as-is for the smoke tier unless fallout is zero); triage fallout (each failure = real console/page
  error → fix the SPEC only if it's test-induced, else `DISCREPANCY:` row);
  then auto on `fullStackTest` (already pageerror-only).
- Flake fixes: conditional `isVisible` guards → unconditional
  `await expect(locator).toBeVisible()` + click
  (`runs-live-performance.spec.ts`); move `storage-composition.spec.ts`
  status restore into `test.afterEach`; hoist 20s/45s literals into
  `ui/tests/e2e/support/timeouts.ts`; set `retries: isCi ? 1 : 0` in
  `playwright.config.ts:69`.

**Verify**: both lanes green twice consecutively (flake check);
`grep -rn "isVisible().catch" ui/tests/e2e/` → no matches.

### Step 5 (P2): A11y + mobile breadth

Extend axe + horizontal-overflow checks (reuse
`fixtures/accessibility-fixture.ts` + the scrollWidth snippet from
`mobile/shell-mobile.spec.ts:45-52`) to `/logs`, `/traces`, `/issues`,
`/services`, `/dashboards` — 5 lines each, seeded datasets from Step 1.

**Verify**: a11y + mobile projects green; axe violations that are real →
`DISCREPANCY:` rows (do not add exceptions without recording).

### Step 6 (P3): Vitest model gaps + route-test cleanup

- New unit tests: `service-map-layout-engine.ts` (`runElkLayout` on a fixed
  graph → non-overlapping bounding boxes, deterministic), `service-detail.ts`
  (RED math incl. `stepSecondsForRange` boundaries), `test-summary.ts`,
  `services-search.ts`, `overview-chart-helpers.ts`. Resolve the duplicate
  ecosystem worker entry (one canonical module; other imports it).
- Fold the 7 `typeof === "function"` route tests into search-param coercion
  tests per the coercion section at `-traces-routes.test.tsx:12-27` (that
  FILE also contains typeof asserts at `:28-29` — delete those too; the
  pattern to keep is the coercion assertions, not the file wholesale);
  delete every typeof assert across all 7 files; update the affected
  `ui/test-matrix.json` entries + ratchet counts (`test_cases` changes),
  and check `cargo xtask policy --only ui.tests` and
  `--only ui.ratchets` both pass.

**Verify**: `cd ui && bun run test && bun run typecheck` green;
`grep -rn 'toBe("function")' ui/src/routes/tests/` → zero matches (all 7
files incl. -traces-routes);
`cargo xtask policy --only ui.tests && cargo xtask policy --only ui.ratchets` green.

## Test plan

This plan IS tests. Expected net-new: ~6 datasets, ~12 new spec files,
~5 model unit files. Every new spec must pass with diagnostics auto-on.

## Done criteria

- [ ] 8 dataset ids in `catalog.ts` (2 existing + 6 new), each with a Rust
      builder and ≥1 spec consuming it.
- [ ] All 12 browser-relevant mutations exercised: each of
      `contracts/dashboards.spec.ts`, `contracts/alerts.spec.ts`,
      `contracts/sql.spec.ts`, `contracts/logs-views.spec.ts` exists and
      is green, and together with the existing investigations + issues
      specs they invoke dashboardSave/Delete, alertRuleSave/Delete/
      SetEnabled, alertDestinationSave/Delete, savedViewSave/Delete (both
      client paths), investigationSave/Delete, issueSetStatus.
      invocationStart/Finish recorded as CLI-covered (playground c2).
- [ ] `/traces/$traceId`, `/services/$service`, `/tests/$caseKey`,
      `/metrics`, `/metrics/$metricName`, `/alerts` each appear in ≥1
      `page.goto`: `grep -rhoE 'goto\\(\"[^\"]+' ui/tests/e2e/ | sort -u`
      lists a URL for each of the six (use -h so file paths can't
      false-match).
- [ ] `diagnostics` fixture is `{ auto: true }` on `productTest` and
      `fullStackTest` (`ui/tests/e2e/fixtures/test.ts:78,116`).
- [ ] No heading-only full-stack specs remain; no `.or()` disjunction
      asserts; no conditional-isVisible guards.
- [ ] Both browser lanes green twice consecutively; `retries: isCi ? 1 : 0`.
- [ ] 5 model unit files added; typeof route tests gone; worker entry
      deduplicated.
- [ ] `ui/test-matrix.json` reflects reality (no reserved entry whose
      spec exists — incl. the 2 pre-existing stale rows; no implemented
      entry that is heading-only; every new file has an entry);
      `cargo xtask policy --only ui.tests` green.
- [ ] `plans/README.md` row updated.

## STOP conditions

1. Drift check fails on the seam files.
2. A new dataset needs a store capability the in-memory adapter lacks —
   report the missing trait method; do not stub via page.route (policy).
3. Diagnostics-auto reveals >10 distinct console errors across specs —
   systemic; report the pattern before triaging one-by-one.
4. A CRUD spec fails because the SERVER rejects the interpolated mutation
   (escaping bug) — that's a product bug: DISCREPANCY row + report; don't
   weaken the test input.
5. Lane runtime exceeds ~2× current CI budget — report; splitting the lane
   is an operator decision.

## Maintenance notes

- New feature rule going forward: a surface ships with (a) a dataset, (b) a
  contracts CRUD spec if it mutates, (c) a full-stack seeded-fact assert —
  reviewers should block on missing pieces.
- The string-interpolated GraphQL builders (alerts, saved views) are now
  end-to-end covered but remain an outlier vs the 7 codegen documents —
  candidate cleanup for a future plan (record only).
- Plan 167's agent pass and this lane overlap intentionally: 167 = live
  exploratory, 170 = deterministic CI; findings cross-feed via
  `DISCREPANCY:` rows.
