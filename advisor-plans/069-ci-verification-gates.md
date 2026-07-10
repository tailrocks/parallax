# Plan 069: Close the CI verification gaps — UI tests/lint, embed-ui compile, real-engine nightly, timeouts

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat dbaba3c..HEAD -- .github/workflows/ci.yml ui/package.json ui/vite.config.ts`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: LOW
- **Depends on**: none
- **Category**: tests / dx
- **Planned at**: commit `dbaba3c`, 2026-07-10

## Why this matters

The repo has a real UI test suite (38 files, ~152 tests under `ui/src/**/__tests__/`)
and an ESLint config, but CI runs neither — the `ui` job only typechecks and
builds. A change that breaks 150 tests merges green. Separately, the
`embed-ui` cargo feature (the release artifact path — rust-embed of `ui/dist`)
is never compiled in PR CI; it is first compiled post-merge in `preview.yml`,
so a break in the embed path is discovered only after it has landed on `main`.
Finally, the only tests that exercise the real GreptimeDB engine are
`#[ignore]`d and never run anywhere automatically. This plan turns all of that
into CI signal.

## Current state

- `.github/workflows/ci.yml` — the CI workflow. The `ui` job (lines 299–321)
  ends with:

  ```yaml
      - run: bun install --frozen-lockfile --ignore-scripts
        working-directory: ui
      - run: bun run typecheck
        working-directory: ui
      - run: bun run build
        working-directory: ui
  ```

  No `bun run test`, no `bun run lint`. The `ui` job is the only job with
  `timeout-minutes` (15, line 303); `fmt`, `check`, `clippy`, `test`,
  `actionlint`, `ci-required` have none (6-hour GitHub default).

- `ui/package.json` scripts:

  ```json
  "test": "vitest run --passWithNoTests",
  "lint": "eslint",
  "typecheck": "tsc --noEmit"
  ```

  `--passWithNoTests` means a broken glob silently passes with zero tests.

- `ui/vite.config.ts` — has NO `test` block at all (plugins + server proxy
  only, 35 lines). Vitest currently matches the `-`-prefixed route test files
  (`ui/src/routes/__tests__/-logs.test.tsx` etc.) only via its default
  include glob.

- `grep -c "embed-ui" .github/workflows/ci.yml` → 0. The feature is compiled
  only in `.github/workflows/preview.yml:245`
  (`cargo zigbuild --release --locked -p parallax-cli --features embed-ui ...`)
  and in `release.yml`. `scripts/release.sh:28` guards that
  `ui/dist/client/_shell.html` exists before embedding — CI has no equivalent
  guard.

- Ignored real-engine tests (all four download/boot a real GreptimeDB):
  - `crates/parallax-server/tests/m1_greptime.rs:15`
  - `crates/parallax-server/tests/m1_table_inventory_greptime.rs:30`
  - `crates/parallax-server/tests/m2_metrics_greptime.rs:34`
  - `crates/parallax-server/tests/m5_gates.rs:19`
  All carry `#[ignore = "downloads and runs a real GreptimeDB; run with --ignored"]`.
  `cargo nextest run --workspace --all-targets` (ci.yml:294) never runs them;
  no `.config/nextest.toml` exists; no scheduled workflow exists (only `ci`,
  `preview`, `release`).

- Repo CI conventions to match (visible throughout `ci.yml`): every third-party
  action is SHA-pinned with a `# vX.Y.Z` comment; tools install through
  `jdx/mise-action` with `install_args`; jobs are gated on the `changes`
  path-filter job; the `ci-required` aggregate lists every job in `needs`.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| UI tests (from `ui/`) | `rtk bun run test` | exit 0, lists test files run |
| UI lint (from `ui/`) | `rtk bun run lint` | exit 0 |
| UI typecheck (from `ui/`) | `rtk bun run typecheck` | exit 0 |
| Workflow lint (repo root) | `rtk mise exec -- actionlint` (or `actionlint` if installed) | exit 0, no findings |
| Rust tests (repo root) | `rtk cargo nextest run --workspace --all-targets` | all pass |

## Scope

**In scope** (the only files you should modify):
- `.github/workflows/ci.yml`
- `.github/workflows/storage-integration.yml` (create)
- `ui/package.json` (scripts only)
- `ui/vite.config.ts` (add `test` block only)
- `advisor-plans/README.md` (status row)

**Out of scope** (do NOT touch, even though they look related):
- `.github/workflows/preview.yml` and `release.yml` — release automation is
  owned separately; do not restructure it.
- The four `*_greptime.rs` / `m5_gates.rs` test files — do not remove
  `#[ignore]`; the gating stays, only a scheduled runner is added.
- Any UI test file — if tests fail when first enabled in CI, report the
  failures; do not "fix" tests to make CI green.
- `scripts/release.sh`.

## Git workflow

- Work directly on `main` (repo rule — see `BRANCHING.md`; no PRs for routine work).
- Conventional Commits with DCO signoff and agent trailer, e.g.:
  `ci: run ui tests and lint in ci` — commit with
  `git commit -s` and include `Co-authored-by: Claude <noreply@anthropic.com>`.

## Steps

### Step 1: Run the UI suite locally to establish the baseline

From `ui/`: `rtk bun install --frozen-lockfile` then `rtk bun run test` and
`rtk bun run lint`.

**Verify**: both exit 0. If either fails, STOP and report the failures — the
suite must be green before it can gate CI (fixing test failures is out of
scope for this plan).

### Step 2: Make an empty test run fail loudly

In `ui/vite.config.ts`, add a `test` block to the config object (vitest reads
vite config):

```ts
const config = defineConfig({
  resolve: { tsconfigPaths: true },
  test: {
    include: ["src/**/*.test.{ts,tsx}"],
  },
  // ...existing plugins/server...
})
```

Note: the route tests are named like `src/routes/__tests__/-logs.test.tsx` —
the pattern above matches them (leading `-` is part of the filename, not a
glob operator). If TypeScript complains that `test` is not a known key of the
vite config type, change the import to `import { defineConfig } from "vitest/config"`
(vitest's `defineConfig` includes the `test` key and is a superset for this
use).

In `ui/package.json`, add a CI-strict script and keep the lenient local one:

```json
"test": "vitest run --passWithNoTests",
"test:ci": "vitest run",
```

**Verify**: from `ui/`: `rtk bun run test:ci` → exit 0 and the run reports the
same number of test files as Step 1 (38 at planning time). Then temporarily
check the failure mode: `rtk bunx vitest run --config vite.config.ts src/does-not-exist.test.ts`
→ non-zero exit ("No test files found"). Do not commit that check.

### Step 3: Add test + lint steps to the `ui` job in ci.yml

After the `bun run typecheck` step (ci.yml:318-319) and before `bun run build`,
insert:

```yaml
      - run: bun run lint
        working-directory: ui
      - run: bun run test:ci
        working-directory: ui
```

**Verify**: `rtk mise exec -- actionlint` → exit 0.

### Step 4: Compile the embed-ui feature in PR CI

Add a new job `embed` to `ci.yml`, modeled on the existing `check` job's cache
steps (rustup cache, cargo registry cache, sccache, per-job target cache with
`-embed-` in the key, mise-action with `install_args: "rust"`), plus the `ui`
job's Bun setup. Gate it like the others:

```yaml
  embed:
    needs: changes
    if: needs.changes.outputs.rust == 'true' || needs.changes.outputs.ui == 'true'
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      # checkout, mise-action (rust + bun), bun/module caches, cargo caches, sccache
      # — copy the exact SHA-pinned action steps from the existing check and ui jobs
      - run: bun install --frozen-lockfile --ignore-scripts
        working-directory: ui
      - run: bun run build
        working-directory: ui
      - name: Assert embedded shell exists
        run: test -f ui/dist/client/_shell.html
      - run: cargo check --locked -p parallax-cli --features embed-ui
```

The `test -f ui/dist/client/_shell.html` line mirrors the guard in
`scripts/release.sh:28` — if the TanStack build output path ever moves, CI now
fails before release does.

Add `embed` to the `ci-required` aggregate: `needs: [changes, actionlint, fmt, check, clippy, test, ui, embed]`
(ci.yml:324).

**Verify**: `rtk mise exec -- actionlint` → exit 0. Locally approximate the job:
from `ui/` run `rtk bun run build`, then from the repo root
`test -f ui/dist/client/_shell.html && echo ok` → `ok`, then
`rtk cargo check --locked -p parallax-cli --features embed-ui` → exit 0.

### Step 5: Add timeout-minutes to the Rust jobs

Add `timeout-minutes: 30` to `fmt`, `check`, `clippy`, `test`, and
`timeout-minutes: 10` to `actionlint` and `ci-required` in `ci.yml` (the `ui`
job already has 15).

**Verify**: `rtk mise exec -- actionlint` → exit 0;
`grep -c "timeout-minutes" .github/workflows/ci.yml` → 8.

### Step 6: Add a scheduled real-engine integration workflow

Create `.github/workflows/storage-integration.yml`: nightly cron plus manual
trigger, running ONLY the ignored real-GreptimeDB tests. Non-blocking at first
(not part of `ci-required`). Reuse the same SHA-pinned checkout/mise/cache
steps as the `test` job in `ci.yml` (copy them verbatim — same action SHAs).

```yaml
name: Storage integration
on:
  schedule:
    - cron: "17 3 * * *"
  workflow_dispatch:
permissions:
  contents: read
jobs:
  greptime:
    runs-on: ubuntu-latest
    timeout-minutes: 45
    steps:
      # checkout, rustup cache, cargo registry cache, sccache, target cache
      # (key suffix `-greptime-`), mise-action rust + cargo:cargo-nextest —
      # copied from the test job in ci.yml
      - run: cargo nextest run --workspace --run-ignored only
```

`--run-ignored only` runs exactly the `#[ignore]` tests (the four listed in
"Current state"). These tests download a GreptimeDB binary at runtime; that is
expected and why this is a scheduled job, not a PR gate.

**Verify**: `rtk mise exec -- actionlint` → exit 0. Also run the suite once
locally so the workflow's command is known-good:
`rtk cargo nextest run --workspace --run-ignored only` → all four ignored
tests pass (this downloads a real engine; allow several minutes). If they fail
locally, STOP and report — promoting broken tests to a schedule helps nobody.

## Test plan

This plan adds CI plumbing, not product tests. The verification is:
- Step 1/2: the existing 38-file suite passes under the strict `test:ci` script.
- Step 6: the four ignored integration tests pass locally once.
- After commit, observe one CI run on `main`: `ui` job shows lint+test steps,
  `embed` job green, `ci-required` includes `embed`.

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `grep -n "bun run test:ci" .github/workflows/ci.yml` → 1 match in the `ui` job
- [ ] `grep -n "bun run lint" .github/workflows/ci.yml` → 1 match in the `ui` job
- [ ] `grep -n "embed-ui" .github/workflows/ci.yml` → at least 1 match
- [ ] `grep -n "_shell.html" .github/workflows/ci.yml` → 1 match
- [ ] `.github/workflows/storage-integration.yml` exists and `actionlint` exits 0
- [ ] `grep -c "timeout-minutes" .github/workflows/ci.yml` → 8
- [ ] `ui/vite.config.ts` contains `include: ["src/**/*.test.{ts,tsx}"]`
- [ ] From `ui/`: `rtk bun run test:ci` exits 0
- [ ] `git status` shows no modified files outside the in-scope list
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- Step 1 fails: any UI test or lint error exists today — report the exact
  failures instead of fixing tests/code.
- The vitest `test` block cannot be added without breaking `bun run build`
  (vite config type conflict that the `vitest/config` import doesn't solve).
- `cargo check --locked -p parallax-cli --features embed-ui` fails for a reason
  other than a missing `ui/dist` (an actual embed-path compile error) — that is
  a product bug to report, not to fix here.
- The ignored greptime tests fail locally in Step 6.
- `ui/dist/client/_shell.html` is not produced by `bun run build` — the
  TanStack output contract has changed; report it (release.sh has the same
  assumption).

## Maintenance notes

- When the storage-integration workflow has run green for ~2 weeks, consider
  promoting it into `ci-required` on a path-filtered basis (it is deliberately
  non-blocking at first).
- Plan 074 (greptime SQL golden tests + adapter conformance) adds fast,
  non-ignored tests for the same layer; once those exist, the nightly job is
  the deep gate and the golden tests are the PR gate.
- If a future TanStack Start upgrade changes the build output path, the CI
  `_shell.html` assert and `scripts/release.sh:28` must be updated together.
- Reviewer: check the `embed` job's cache keys don't collide with the existing
  `-check-`/`-clippy-`/`-nextest-` keys (use a distinct `-embed-` segment).
