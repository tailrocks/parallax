# Plan 167: Verify every Parallax UI surface with agent-browser — functional + responsive

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving on. If
> anything in "STOP conditions" occurs, stop and report — do not improvise.
> When done, update this plan's row in `plans/README.md`.
>
> **Before anything**: run `agent-browser skills get core --full` and read it
> — it is the CLI's own agent guide (refs, snapshot workflow, patterns).
>
> **Drift check (run first)**:
> `git diff --stat f6208070..HEAD -- ui/src/routes/ ui/src/shared/navigation.ts docs/research/reference/feature-inventory-and-playground-verification.md`
> — if routes changed since planning, reconcile the route table below against
> `ui/src/routes/` before proceeding; a missing/renamed route is drift, not
> an error to skip.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: LOW (read-only against a local stack; the only mutations are the
  UI's own test actions — resolve/reopen, saves — against disposable data)
- **Depends on**: plans/163-playground-example-upgrades.md,
  plans/164-playground-feature-coverage.md (seeded data); runs alongside
  plans/165-user-lens-comparison.md; feeds plans/166-production-readiness-fix-loop.md
- **Category**: tests
- **Planned at**: parallax `f6208070`, playground `6e0a0d5`, 2026-08-13

## Why this matters

The scenario sweep (plan 164) asserts Parallax through CLI/GraphQL; nothing
verifies the **UI itself** beyond the narrow Playwright lanes (mobile lane
covers only shell + investigations; visual lane only shell + investigations
dark). Operator requirement (2026-08-13): drive every UI surface with an
agent-controlled browser and confirm (a) the feature works as a user would
exercise it and (b) the layout is responsive — no horizontal overflow, nav
usable — at phone/tablet/desktop widths in both themes. Findings feed the
same `DISCREPANCY:` pipeline plan 166 consumes.

## Current state

- Tool: `agent-browser` v0.34.0+ (Homebrew, `which agent-browser` →
  `/opt/homebrew/bin/agent-browser`) — "fast browser automation CLI for AI
  agents". Key commands (from `--help`): `open <url>`, `snapshot`
  (accessibility tree with `@ref`s), `click/fill/press/hover <sel|@ref>`,
  `get text|url|count|box`, `is visible|enabled`, `set viewport <w> <h>`,
  `set media [dark|light]`, `eval <js>`, `screenshot [path]`,
  `diff screenshot --baseline`, `wait <sel|ms>`, `close --all`.
- UI serves embedded from `parallax serve` at `http://localhost:4000/`
  (SPA fallback; `crates/parallax-server/src/serve.rs:306-321`).
- The full route inventory (from `ui/src/routes/` — the checklist for this
  plan; detail-route params come from seeded data):

| # | Route | Surface | Core functional check |
|---|---|---|---|
| 1 | `/` | Overview | stat cards populated (non-zero after sweep); click "Spans" card → lands on `/traces` with range preserved |
| 2 | `/issues` | Issues list | search box filters rows; status filter Open/Resolved switches sets |
| 3 | `/issues/<fingerprint>` | Issue detail | stacktrace block renders; Resolve → status flips → Reopen restores |
| 4 | `/tests` | Test explorer | table renders seeded variants; flaky-state filter changes rows |
| 5 | `/tests/<caseKey>` | Test case | attempt chain rows link out (invocation link navigates) |
| 6 | `/traces` | Traces list | service filter + errors-only toggle change rows; Live toggle streams during traffic |
| 7 | `/traces/<traceId>` | Trace detail | waterfall renders spans; flamegraph mode switch works; span click opens detail panel |
| 8 | `/ecosystem` | Service map | React Flow canvas renders nodes ≥ playground service count; focus + 1-hop hides others |
| 9 | `/logs` | Logs | severity floor changes rows; where-clause chip add/remove; Live tail delivers a row during traffic; histogram visible |
| 10 | `/metrics` | Metric catalog | search narrows list; kind filter works |
| 11 | `/metrics/<name>` | Metric workbench | chart renders; aggregation selector limited to kind-legal set; "Add to dashboard" opens dialog |
| 12 | `/services` | Services catalog | table rows for every playground service; heat cells render |
| 13 | `/services/<service>` | Service detail | RED charts render; quick links land pre-filtered |
| 14 | `/invocations` | CLI Apps | seeded invocation listed; status filter works |
| 15 | `/invocations/<id>` | Invocation hub | all 6 tabs switch and render |
| 16 | `/alerts` | Alerts | 3 tabs switch; New-rule dialog opens from template; enable Switch toggles |
| 17 | `/dashboards` | Dashboards | gallery lists c5-created dashboard; create dialog opens |
| 18 | `/dashboards/<id>` | Dashboard | widget grid renders series |
| 19 | `/investigations` | Investigations | list + create dialog |
| 20 | `/investigations/<id>` | Case file | pin list renders; note textarea persists after reload |
| 21 | `/sql` | SQL workbench | schema browser expands; ⌘⏎ runs `SELECT 1`; results render |

- Cross-cutting checks: ⌘K palette opens and jumps on a pasted trace id;
  theme switcher Light/Dark persists across reload; route error boundary
  shows on an invalid route (`/nonexistent` → not-found panel, not blank).
- Viewport matrix: `375x812` (phone), `768x1024` (tablet), `1440x900`
  (desktop) × media `light` and `dark` = 6 passes per route.
- Responsive pass criterion (machine): no horizontal document overflow —
  `agent-browser eval "document.documentElement.scrollWidth - document.documentElement.clientWidth"`
  → `0` — and primary nav reachable (sidebar or its collapsed trigger
  visible: `agent-browser is visible nav` or the snapshot shows the nav
  landmark).
- Existing browser-test conventions (do not duplicate, do not modify):
  `ui/tests/e2e/` Playwright lanes; this plan is a *live-stack agent pass*,
  not new Playwright code.
- Discrepancy pipeline: same `DISCREPANCY:` line token as plans 164/165, in
  the inventory doc's W5 list
  (`docs/research/reference/feature-inventory-and-playground-verification.md`).

## Commands you will need

| Purpose | Command | Expected on success |
|---|---|---|
| Tool present | `agent-browser --version` | `0.34.0` or newer |
| Agent guide | `agent-browser skills get core --full` | usage guide prints |
| Stack | plans 162/163 procedure (`parallax serve` + lab + playground compose) | `curl -s localhost:4000/health` → healthy |
| Seed data | `cd ../parallax-telemetry-playground/scenarios && ./run.sh a1 && ./run.sh a12 && ./run.sh c5` (+ `./run.sh a13` for releases; c-series from plan 164) | scripts exit 0 |
| Ambient traffic (for Live checks) | `docker compose -f deploy/docker-compose.yml --profile demo up -d loadgen` | k6 running |
| Open route | `agent-browser open "http://localhost:4000/<route>"` | page loads |
| Structure read | `agent-browser snapshot` | accessibility tree with @refs |
| Overflow check | `agent-browser eval "document.documentElement.scrollWidth - document.documentElement.clientWidth"` | `0` |
| Screenshot | `agent-browser screenshot artifacts/ui/<route-slug>-<w>-<theme>.png` | file written |
| Cleanup | `agent-browser close --all` | sessions closed |

## Scope

**In scope**: a new runbook script
`../parallax-telemetry-playground/scenarios/c11-ui-agent-verify.sh` (the
deterministic core: per route × viewport × theme → open, wait, overflow
eval, key-element visibility, screenshot; registered in `run.sh` + README
per plan-164 convention), the agent-led functional checklist results, the
inventory doc (results table + `DISCREPANCY:` rows), playground
`VERIFICATION.md` (c11 section), `plans/README.md` row.

**Out of scope**:
- Any change under `ui/` or `crates/` (defects → `DISCREPANCY:` rows for
  plan 166; do not fix here).
- New Playwright tests (`ui/tests/e2e/` untouched — promotion of agent
  findings into Playwright lanes is a plan-166 maintenance decision).
- Visual pixel-diff baselines (agent-browser `diff screenshot` exists, but
  baseline curation is deferred — screenshots are evidence, not gates).

## Git workflow

PR-only `main`, one branch + one PR per repo, `git commit -s`, Conventional
Commits, agent trailer per `COMMITS.md`.

## Steps

### Step 1: Preflight

`agent-browser --version` (≥ 0.34.0), read
`agent-browser skills get core --full`, stack up, seed data, ambient
traffic on. Collect real ids for detail routes (fingerprint, traceId,
caseKey, invocation id, dashboard id, investigation id, metric name,
service name) via the GraphQL queries plan 164 uses — record them in a
scratch env file.

**Verify**: `curl -s localhost:4000/health` healthy; every placeholder in
the route table has a concrete id.

### Step 2: Write and run the deterministic core (`c11-ui-agent-verify.sh`)

Script loops route table × 3 viewports × 2 themes: `set viewport`,
`set media`, `open`, `wait` for a route-specific readiness selector, the
overflow `eval` (must print `0`), an `is visible` check on the route's
primary landmark, `screenshot` to `artifacts/ui/`. Non-zero overflow or
missing landmark → record `FAIL <route> <viewport> <theme>` and continue;
exit 1 at the end if any FAIL. Register c11 in `run.sh` (catalog + case
arm) and `scenarios/README.md`.

**Verify**: `./run.sh c11` runs all 126 combinations (21 × 6); output ends
with a summary line `ui-verify: <pass>/<total> pass`; screenshots exist
(`ls artifacts/ui/*.png | wc -l` → 126).

### Step 3: Agent-led functional pass (desktop viewport)

For each route, perform the "Core functional check" column interactively
with agent-browser (`snapshot` → act on `@ref`s → re-`snapshot` to confirm
state change). Also the three cross-cutting checks (⌘K palette — use
`press Meta+k`; theme persistence — `set media` is emulation, so use the
in-app theme switcher then `reload` and confirm; invalid route boundary).
Log one line per check: `PASS|FAIL <route> <check> <evidence>`.

**Verify**: every route-table row + 3 cross-cutting checks has a PASS/FAIL
line; no check skipped.

### Step 4: Mobile functional spot-pass

At `375x812` repeat the interaction-critical subset: nav to every surface
via the sidebar/collapsed menu, one filter interaction on Issues/Logs/
Traces, tab switches on the invocation hub and alerts, span-detail open on
trace detail. Mobile-specific FAIL = control unreachable/covered/
non-functional at phone width.

**Verify**: PASS/FAIL line per subset item.

### Step 5: Record

- Inventory doc: add "UI verification (plan 167, <date>)" results table —
  21 rows, columns `Surface | Functional | Responsive | Evidence` — plus
  one `DISCREPANCY:` line per FAIL (same token/shape as plans 164/165:
  `DISCREPANCY: <feature> | c11/agent-pass | parallax-ui | <observed> |
  <expected> | parallax bug`).
- Playground `VERIFICATION.md`: c11 section with the summary line + FAIL
  list.
- Attach the sweep log + screenshot dir listing to the PR description.

**Verify**: `cargo xtask docs links` passes; FAIL-line count equals
`DISCREPANCY:` rows added.

## Test plan

The c11 script is the repeatable regression artifact (re-run at every pin
bump and after every plan-166 UI fix); the agent-led pass is the exploratory
layer — its checklist lives in this plan and its results in the inventory
doc. Reviewer spot-checks two screenshots per viewport against the live UI.

## Done criteria

- [ ] `./run.sh c11` exists, registered (README + run.sh), exits 0 on a
      healthy stack; 126 screenshots produced.
- [ ] Functional PASS/FAIL log covers all 21 routes + 3 cross-cutting
      checks (Step 3) and the mobile subset (Step 4) — zero skips.
- [ ] Every FAIL has a `DISCREPANCY:` row in the inventory doc; counts
      match.
- [ ] `cargo xtask docs links` passes.
- [ ] `plans/README.md` row updated.

## STOP conditions

1. Route-table drift: a route in `ui/src/routes/` is missing from the table
   or renamed — reconcile against the routes dir and report the delta
   before continuing.
2. `agent-browser` cannot drive the app at all (blank snapshot on `/` with
   a healthy `/health`) — report; do not switch to another automation stack
   unilaterally.
3. A "functional check" would require destructive server state (delete-all,
   prune --execute) — those belong to c9's isolated-HOME pattern, not the
   UI pass.
4. More than ~20% of combinations FAIL on overflow — likely a systemic
   layout/regression or a wrong readiness selector; report the pattern
   instead of logging 25 identical rows.

## Maintenance notes

- Route table is duplicated knowledge (source: `ui/src/routes/`); plan-166
  reviewers must update it when routes change — the Step-1 drift check
  exists for exactly that.
- Recurring agent passes should stay cheap: c11 first (machine), agent-led
  pass only on surfaces c11 or plan-166 flagged.
- Deferred: pixel-diff baselines via `agent-browser diff screenshot`
  (needs curated baselines + flake policy); promotion of stable agent
  checks into `ui/tests/e2e/` Playwright lanes.
