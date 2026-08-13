# Plan 170: Playwright coverage for every critical user flow

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving on. If
> anything in "STOP conditions" occurs, stop and report — do not improvise.
> When done, update this plan's row in `plans/README.md`.
>
> **Drift check (run first)**: `git diff --stat f6208070..HEAD -- ui/tests/ ui/playwright.config.ts ui/test-matrix.json crates/parallax-test-support/src/browser/`
> — on mismatch with the excerpts below, STOP.

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
21 surfaces (shell, investigations). 8 of 11 GraphQL mutations have zero
browser exercise; `/traces/$traceId` — the deepest surface and
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
  `ui/test-matrix.json` holds 11 `status: "reserved"` playwright entries
  whose spec files don't exist.
- Diagnostics fixture: `ui/tests/e2e/fixtures/test.ts:78-90` — declared
  WITHOUT `{ auto: true }` (unlike `fixedTime`/`seeded`); catches
  console-error/pageerror/external-network/dialog/download; only
  `contracts/shell.spec.ts` and `smoke/foundation.spec.ts` opt in.
- Mutation ledger (browser-covered: 3 of 11): covered =
  investigationSave/Delete (`contracts/investigations.spec.ts:47,68`),
  issueSetStatus (`full-stack/storage-composition.spec.ts:25,46`). NOT
  covered: dashboardSave/Delete, sqlSnippetSave/Delete,
  savedViewSave/Delete (`ui/src/features/logs/components/logs-page.tsx:294-318`,
  string-interpolated, no escaping test), alertRuleSave,
  alertDestinationSave (`ui/src/features/alerts/api/alerts-gql.ts:120,155`
  — string-built over `gqlString`).
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
  (`ui/playwright.config.ts:69`).
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
| Contracts lane | `cargo xtask browser-contracts-serve` | Playwright green |
| Full-stack lane | `cargo xtask browser-full-stack-serve` | green |
| One spec | append Playwright args per xtask lane help (check `cargo xtask browser-contracts-serve --help`); fallback: run lane then filter via `-g` if supported | targeted spec runs |
| Policy gates | `cargo xtask policy --only ui.tests && cargo xtask policy --only ui.browser-contracts` | pass |
| Rust seed build | `cargo xtask ci` | pass |

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

**Verify**: `cargo xtask ci` (Rust builders compile+unit-test);
`cargo xtask browser-contracts-serve` green with the first new spec.

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

**Verify**: contracts lane green; `ui/test-matrix.json` reserved entries
flipped to implemented for these surfaces.

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
  (`fixtures/test.ts:78`); triage fallout (each failure = real console/page
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
  tests per the `-traces-routes.test.tsx:12-27` pattern; delete the typeof
  asserts; adjust the `ui.ratchets` counts if the policy gate complains
  (`cargo xtask policy --only ui.ratchets` tells you).

**Verify**: `cd ui && bun run test && bun run typecheck` green;
`grep -rn 'toBe("function")' ui/src/routes/tests/` → no matches;
`cargo xtask policy --only ui.tests` green.

## Test plan

This plan IS tests. Expected net-new: ~6 datasets, ~12 new spec files,
~5 model unit files. Every new spec must pass with diagnostics auto-on.

## Done criteria

- [ ] 8 dataset ids in `catalog.ts` (2 existing + 6 new), each with a Rust
      builder and ≥1 spec consuming it.
- [ ] All 11 mutations have a browser CRUD exercise
      (`grep -l "dashboardSave\|alertRuleSave\|sqlSnippetSave\|savedViewSave" ui/tests/e2e/contracts/` non-empty per family).
- [ ] `/traces/$traceId`, `/services/$service`, `/tests/$caseKey`,
      `/metrics`, `/metrics/$metricName`, `/alerts` each appear in ≥1
      `page.goto` (`grep -rn "goto(" ui/tests/e2e/ | grep -c "alerts\|metrics\|traces/\|services/\|tests/"` ≥ 6).
- [ ] `diagnostics` fixture is `{ auto: true }` on both fixtures.
- [ ] No heading-only full-stack specs remain; no `.or()` disjunction
      asserts; no conditional-isVisible guards.
- [ ] Both browser lanes green twice consecutively; `retries: isCi ? 1 : 0`.
- [ ] 5 model unit files added; typeof route tests gone; worker entry
      deduplicated.
- [ ] `ui/test-matrix.json` reflects reality (no reserved entry whose spec
      exists; no implemented entry that is heading-only).
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
