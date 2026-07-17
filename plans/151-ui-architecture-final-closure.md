# Plan 151: Verify and close the final UI architecture

> **Executor instructions**: Run only after every product, route-less capability,
> overview, app, and layout migration in Plans 134-143, 149, and 150 plus the
> GraphQL/non-GraphQL foundations in Plans 152/153 are complete.
> This is a verification and mechanical closure plan, not a product migration.
> Rebuild the ownership and import graph from live source, prove every route,
> facade, test, matrix row, ratchet, and compatibility path satisfies the final
> contract, and rewrite durable navigation documentation to match reality. You
> may delete a stale compatibility path only when its already-completed owner and
> replacement are unambiguous and Oxc proves no caller. If closure exposes a
> missing move, disputed owner, behavior change, or new contract, stop and reopen
> or reassign the owning plan rather than absorbing it here. Preserve all cache,
> live, and bundle behavior for Plans 133, 147, and 148.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src ui/tests ui/test-matrix.json ui/AGENTS.md PROJECT_STRUCTURE.md ratchet.toml crates/parallax-xtask .github/workflows/ci.yml`
> Compare the live tree with the completed ownership ledger, not the historical
> baseline layout. Stop if an active product responsibility remains assigned to
> this verification plan or if a completed owner's evidence is missing.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 149, 150, 152, 153
- **Category**: TypeScript / architecture verification / final closure
- **Planned at**: `e3e7997`, 2026-07-12
- **Status**: IN PROGRESS — lib/* residual claimed (2026-07-17); browser 145/146 remain

## Why This Matters

Individual migrations can be green while the aggregate tree still contains a
stale compatibility reexport, duplicate owner, route implementation export,
deep feature import, orphaned test, unclassified file, expired exception, or
documentation that sends future work to the wrong place. Those residuals are
especially costly for parallel and AI-assisted maintenance because each creates
two plausible answers to where code belongs.

This plan provides one parser-backed closure transaction. It verifies the live
source rather than trusting completed checklists, removes only provably dead and
unambiguous compatibility artifacts, and records the final placement rules. It
does not become a catch-all implementation plan.

## Fixed Decisions

1. The live Oxc-resolved source/import/export graph is authoritative. Historical
   plan trees and prose are inputs, not exceptions to live machine evidence.
2. Every handwritten `ui/src` and `ui/tests` file has exactly one owner from the
   final catalog: app, layout, one feature, one domain concept, one platform
   adapter, shared, route, test support, or an explicit generator island.
3. Every file route exports only `Route`. Routes own only params/search/loader
   dependencies/boundaries/composition and import feature facades plus approved
   route/domain/shared contracts. Root-only layout access remains the sole
   reviewed layout exception.
4. Every feature has one explicit facade with named exports. Wildcard barrels,
   consumer deep imports, feature-to-route/app/layout imports, route-to-route
   implementation imports, undeclared cross-feature edges, and cycles are zero.
5. `shared` is product-neutral, `domain` is framework/browser/transport neutral,
   `platform` owns technical adapters only, and app/layout import directions
   match Plan 100's closed graph.
6. `tests/` is the only source-owned test directory name. Test bodies live below
   their real owner; `src/test` contains setup/builders only; `ui/tests/harness`
   tests harness infrastructure; `ui/tests/e2e` is the sole browser owner.
7. Plan-129 legacy handoffs, Plan-152 raw-GraphQL handoffs, Plan-153 external-
   value handoffs, migration exceptions, old root compatibility paths,
   private route test imports, test-only production exports, stale ledger rows,
   and orphan matrix IDs must all reach zero.
8. Generated `routeTree.gen.ts`, `components/ui/**`, `lib/utils.ts`, and other
   machine-owned islands remain in their required paths and are never edited
   manually. No other handwritten generic root bucket is grandfathered.
9. Final ratchets are exact and shrink-only: route module at most 150 logical
   lines, handwritten TS/TSX module at most 300, test scenario at most 500,
   function/component/hook at most 60, cyclomatic at most 12, cognitive at most
   15, plus exact facade/export/import/test ownership limits. Any exception must
   predate closure, name an owner/reason/expiry/removal condition, and cannot hide
   a missing migration.
10. This plan may delete only an unreferenced compatibility artifact whose final
    owner and replacement were already completed. It may update import paths only
    when the change is purely mechanical and behavior/type identity is unchanged.
    Otherwise it stops and reopens/reassigns the responsible plan.
11. Plan 133 remains the future Query/cache owner, Plan 147 the future live-data
    performance owner, and Plan 148 the future bundle owner. Closure records their
    current baselines without implementing them.
12. `ui/AGENTS.md` and `PROJECT_STRUCTURE.md` describe the live final tree,
    dependency table, feature catalog, module/runtime/error/schema/test placement,
    new-file decision process, and exact verification commands. They contain no
    completed migration instructions.

## Target Ownership

```text
ui/src/
  app/                         router/provider composition
  layout/                      root shell/navigation/global boundaries
  routes/                      thin TanStack file-route adapters
  features/<owner>/            feature API/model/components/hooks/tests/facade
  domain/<concept>/            framework-neutral product concepts and tests
  platform/<adapter>/          technical boundaries and tests
  shared/                      product-neutral components/hooks/lib/tests
  test/                        setup/builders only
  components/ui/               shadcn generator island
  lib/utils.ts                 shadcn `cn` island
  routeTree.gen.ts             TanStack generated tree
  styles.css                   global style/token entry
ui/tests/
  harness/                     harness self-tests
  e2e/                         single Playwright stack
ui/test-matrix.json            complete live risk/evidence ownership
ratchet.toml                   complete typed ownership/edge/budget policy
ui/AGENTS.md                   final UI placement and execution rules
PROJECT_STRUCTURE.md           repository-level live ownership map
```

The executor must generate a machine closure report under the existing ignored
xtask output directory containing at minimum:

- every handwritten file and owner;
- resolved static/type/dynamic/reexport edges;
- every route export and feature facade export;
- every source test, test ID, matrix row, and browser project owner;
- every compatibility path, exception, ratchet, and expiry;
- every client/server/runtime suffix and emitted reachability classification; and
- positive/negative policy fixture results.

The report is CI evidence, not a second checked-in source of truth.

## Commands

| Purpose | Command | Expected result |
|---|---|---|
| Ownership graph | `cargo xtask arch` | every handwritten file has one owner; no cycle/deep/route/unknown edge |
| UI architecture | `cargo xtask policy --only ui.architecture` | final layer, facade, runtime, route, generator, and bucket rules pass |
| Test ownership | `cargo xtask policy --only ui.tests` | final topology, IDs, imports, and zero legacy handoffs pass |
| Test matrix | `cargo xtask policy --only ui.test-matrix` | every shipped risk has one non-orphan live evidence owner |
| Ratchets | `cargo xtask policy --only ui.ratchets` | exact final budgets/exports/exceptions pass; no migration exception remains |
| Browser contracts | `cargo xtask policy --only ui.browser-contracts` | every dataset/screen/spec/ID/locator/runtime owner is valid |
| Full-stack policy | `cargo xtask policy --only ui.browser-full-stack` | every delegated row is implemented once and lifecycle rules pass |
| Browser breadth | `cargo xtask policy --only ui.browser-breadth` | engine/mobile/a11y/visual rows and goldens are complete |
| Format/lint/types | `cd ui && bun run check && bun run lint && bun run typecheck` | exit 0 with zero warnings/errors |
| All UI tests | `cd ui && bun run --bun test:ci` | all tests pass under Bun; no Node descendant or unexpected diagnostic |
| Browser contract run | `cd ui && bun run test:browser` | fixture-backed shipped surface contracts pass |
| Managed stack | `cd ui && bun run test:browser:full` | all delegated managed GreptimeDB + Turso rows pass |
| Cross/mobile | `cd ui && bun run test:browser:cross` | selected Firefox/WebKit/mobile rows pass |
| Accessibility | `cd ui && bun run test:browser:a11y` | axe plus keyboard/focus rows pass |
| Visual | `cd ui && bun run test:browser:visual` | canonical comparisons pass; no update mode |
| Production build | `cd ui && bun run build` | generated route tree current; no runtime boundary leak |
| Fast aggregate | `cargo xtask ci --fast` | exit 0 |
| Full aggregate | `cargo xtask ci --full` | exit 0 with all selected UI/browser gates |
| Whitespace | `git diff --check` | no whitespace errors |

All JavaScript, TypeScript, and browser commands use exact lock-local tools
through Bun with auto-install disabled. Oxc-backed xtask is the only source graph,
resolver, AST, and ratchet oracle. The single Playwright stack is the only browser
runner. Node, foreign package managers, ESLint, second parsers/runners, manual
generated edits, and implicit installs are forbidden.

## Scope

**In scope:**

- Rebuilding and validating the complete live UI ownership/import/export/runtime
  graph and every machine rule fixed above.
- Proving final route/facade/test/matrix/browser/ratchet ownership and zero stale
  migration debt.
- Deleting a provably dead compatibility file/reexport/row or making a purely
  mechanical final import switch when its completed owner is exact and unambiguous.
- Final `ui/AGENTS.md` and `PROJECT_STRUCTURE.md` updates based on the live graph.
- Negative fixtures for every closure invariant and the full clean-state gate
  run twice.

**Out of scope:**

- Moving or restructuring product, route-less capability, overview, app, layout,
  domain, platform, or shared behavior that a prerequisite plan failed to finish.
- Creating a missing schema, mapper, error model, facade, component, hook, route,
  source test, browser scenario, dataset, screen, or golden for an incomplete owner.
- Resolving a disputed owner, introducing a new cross-feature edge, redesigning an
  API/URL/search/render contract, or changing product behavior.
- Query/cache implementation (133), live/poll performance implementation (147),
  bundle/chunk/minifier/source-map implementation (148), dependency/tool upgrades,
  backend/Rust product work, or visual redesign.
- Broad deletions, automated baseline/golden updates, compatibility shims,
  catch-all modules, new internal packages/project references, or generated edits.

## Git Workflow

- Stay on the one active branch; never create another branch or PR.
- Land closure-report/policy corrections, unambiguous mechanical cleanup, ratchet/
  matrix closure, and documentation as separate green commits.
- Before deleting or switching any path, record its completed owner, replacement,
  zero-caller Oxc evidence, and targeted verification in the commit context.
- Serialize `ratchet.toml`, `ui/test-matrix.json`, `ui/AGENTS.md`, and
  `PROJECT_STRUCTURE.md`; re-read live content before every patch.
- Use Conventional Commits, DCO, exactly one agent-product trailer, and push each
  durable update under repository policy.

## Steps

### Step 0: Prove every prerequisite completed its owned move

Resolve durable completion evidence for Plans 134-143, 149, 150, 152, and 153. Run the full
command table without editing and capture the live ownership report. Verify every
prerequisite's required old-path deletions, source tests, matrix IDs, browser
rows, facades, and structural ratchets are present at the same commit.

Classify each failure as:

1. mechanical stale artifact with an exact completed owner and replacement;
2. missing or incorrect work belonging to a completed plan; or
3. a newly discovered product/architecture decision.

Only class 1 may continue here. Class 2 reopens/reassigns the owner; class 3
requires a separate decision/plan.

**Verify:** a machine report lists zero incomplete/unknown prerequisite owner.
Any class-2 or class-3 item triggers STOP before a source edit.

### Step 1: Rebuild the complete live ownership and dependency graph

Parse every handwritten TS/TSX/test/E2E file, static/type/dynamic import,
reexport, route export, facade export, runtime suffix, and generator marker.
Compare it with the typed Plan-100 ledger and live tree. Remove stale ledger rows
for paths already deleted and require one exact owner for every live path.

Test the graph with positive and negative fixtures for every dependency-table
edge, type-only import, alias, dynamic import, reexport, generated reverse edge,
server/client reachability, test import, and cycle.

**Verify:** adding an unclassified file, duplicate owner, unknown alias, hidden
dynamic/reexport edge, cycle, or forbidden runtime import fails with exact path,
line, stable rule ID, and rerun command.

### Step 2: Prove every route and facade is final

Inspect every route AST. Require exactly the framework's `Route` export and no
other public implementation export. Prove route code is within budget and owns
only params/search/loader/boundary/composition. Reject route-to-route imports,
platform imports, feature deep imports, test imports, and app access; permit only
the reviewed root-layout entry from the root route.

Inspect every feature facade. Require explicit named value/type exports, no
wildcard/reexport chain hiding internals, no document/schema/internal hook leak,
and only declared cross-feature consumers. Reject a missing, unused temporary,
or broadened facade export.

**Verify:** route and facade reports are complete and zero-valued for every
forbidden category. Negative fixtures fail their exact rule without false passes.

### Step 3: Close only unambiguous compatibility artifacts

Inventory every remaining handwritten root `components`, `components/console`,
`hooks`, and `lib` path; old feature path; compatibility reexport; private route
test import; migration exception; and legacy `__tests__` directory. For each,
require a completed owner, live replacement, zero caller, matching type/behavior
identity, and passing owner tests before deletion.

Delete one unambiguous artifact and its exact ledger/ratchet/matrix row at a time.
If a caller remains, make only an import-path switch when the final public facade
already exposes the identical contract. Do not move implementation or add an
export. If ownership or identity is unclear, STOP and reopen/reassign the owner.

Expected final generic islands are only generator-required `components/ui/**`
and `lib/utils.ts`; `routeTree.gen.ts` and `styles.css` retain their fixed roles.

**Verify:** after each deletion/switch, targeted owner tests, architecture, tests,
ratchets, typecheck, and build pass. Final searches and Oxc report show zero old
path, compatibility reexport, legacy test directory, or migration exception.

### Step 4: Close test and evidence ownership

Validate every source test body is below its feature/app/layout/domain/platform/
shared/route owner and no body lives in production or `src/test`. Require stable
test IDs, meaningful assertions, no private route imports, and no orphan file.

Validate `ui/test-matrix.json` against all shipped surfaces and risks. Every row
must reference a live test ID/file/project and one owner; all Plan-129 handoffs
and Plan-145/146 reservations must be resolved. Ensure fixture, managed-stack,
cross/mobile, accessibility, and visual rows remain distinct rather than counting
one scenario as several evidence classes.

**Verify:** deliberate missing/duplicate/orphan/wrong-owner/expired/reserved/zero-
selection fixtures fail. All real test/browser commands select non-zero evidence
and pass with no unexpected diagnostic or process/state/artifact leak.

### Step 5: Lock final structural ratchets

Recompute route/module/test/function/component/hook/complexity/export/import/
bucket/runtime/test-layout baselines from the live handwritten tree. Remove every
migration exception and stale row. Existing non-migration exceptions require
exact owner, measured value, reason, created date, expiry/removal condition, and
shrink-only behavior; otherwise STOP for owner action rather than normalizing it.

Add negative fixtures for growth, stale paths, renamed bypasses, split-file
evasion, wildcard facade growth, hidden reexports, private test imports, and
unowned exception updates. CI never rewrites ratchets.

**Verify:** final values satisfy declared limits or one pre-existing exact
non-migration exception; intentional growth/stale/bypass fixtures fail and two
unchanged runs produce identical policy output.

### Step 6: Rewrite durable UI and repository navigation docs

Generate the human-readable catalog from the live machine graph, then update
`ui/AGENTS.md` with:

- final tree and owner catalog;
- closed dependency matrix and approved cross-feature edges;
- feature facade, schema/mapper/error, runtime suffix, and test rules;
- route-only responsibilities and only-Route export rule;
- new-file placement decision table;
- generator islands and prohibited generic buckets;
- matrix/ratchet update process; and
- exact Bun/Oxc/Playwright verification commands.

Update `PROJECT_STRUCTURE.md` with the actual app/layout/routes/features/domain/
platform/shared/test ownership. Remove completed migration language and old paths;
do not embed active implementation backlogs or copy plan prose into durable docs.

**Verify:** every documented path exists, every live top-level owner is described,
link/path checks pass, and a generated documentation comparison reports no stale
or missing owner.

### Step 7: Run mechanical closure twice

Start from a clean state and run the complete command table in order. Remove
ignored generated reports between runs, then repeat from the same tracked inputs.
The second run must produce identical policy inventories and no tracked diff.

Inspect `git diff --check`, `git status --short`, generated route-tree status,
matrix/ratchet ordering, browser inventories, and process ancestry. Preserve
machine reports as bounded CI artifacts only.

**Verify:** all commands exit zero twice; no tracked file changes on the second
run; zero Node/foreign-manager process appears; no selected gate reports zero
tests; and the final closure report contains zero actionable row.

## Test Plan

- Ownership graph parser/resolver tests for aliases, static/type/dynamic imports,
  reexports, cycles, generated edges, and client/server reachability.
- Route/facade AST tests for only-Route, thin responsibilities, explicit exports,
  deep/wildcard/document/schema/internal leaks, and cross-feature edge ownership.
- Compatibility closure tests for zero callers, exact replacement, stale rows,
  ambiguous owner STOP, and deletion/import-only boundaries.
- Test topology/matrix tests for owner paths, private imports, body placement,
  stable IDs, missing/duplicate/orphan/reserved rows, and project distinctions.
- Ratchet tests for every budget, rename/split/reexport bypass, exact exceptions,
  shrink-only updates, and deterministic output.
- Documentation/path/catalog comparison tests generated from the live ledger.
- Complete Bun unit, fixture browser, real-stack, cross/mobile, accessibility,
  visual, build, and aggregate runs twice from clean state.

## Done Criteria

- [ ] Every handwritten UI/test file has exactly one live owner; unknown,
  duplicate, stale, and generic compatibility owners are zero.
- [ ] Every route exports only `Route`, meets its budget, and has no route-to-route,
  platform, app, deep-feature, test, or unauthorized layout import.
- [ ] Every feature exposes one explicit reviewed facade; wildcard exports, deep
  consumers, internal leaks, undeclared cross-feature edges, and cycles are zero.
- [ ] App/layout/domain/platform/shared directions and runtime suffix/reachability
  satisfy the closed graph with no broad exception.
- [ ] Plan-152 raw GraphQL/interpolation/generic-result handoffs and Plan-153
  direct external-value/parse-cast handoffs are zero.
- [ ] Source tests use one final `tests/` topology; private route imports,
  test-only production exports, legacy `__tests__`, and Plan-129 handoffs are zero.
- [ ] Every test-matrix row resolves to one live file/test/project, one scenario
  owner, and one lane owner; temporary delivery plans are cleared, all browser
  reservations are implemented, and no evidence class is falsely duplicated.
- [ ] Old root generic paths, compatibility reexports, stale ledger/ratchet rows,
  and migration exceptions are zero except the fixed generator islands.
- [ ] Structural ratchets satisfy final budgets or an exact pre-existing
  non-migration exception with owner/reason/expiry/removal condition.
- [ ] `ui/AGENTS.md` and `PROJECT_STRUCTURE.md` match the live graph and contain no
  completed migration paths or hidden implementation backlog.
- [ ] Query/cache, live-performance, bundle, product/API, and feature ownership
  behavior were not implemented or changed in this closure.
- [ ] Every command passes twice from clean state; the second run changes no
  tracked file and the machine closure report has zero actionable row.

## STOP Conditions

Stop and reopen/reassign the owning plan if:

- a prerequisite lacks its final source, facade, test, browser evidence, deletion,
  matrix row, or ratchet outcome;
- a file has no owner, more than one plausible owner, or behavior that must move;
- deleting a compatibility path requires implementation movement, new public
  exports, type conversion, or behavior change rather than a zero-caller deletion
  or identical import switch;
- a route cannot become only-Route/thin without moving product logic;
- a facade or cross-feature edge is disputed or requires a new architecture decision;
- a missing runtime schema/mapper/error/test/browser scenario must be created;
- an exception cannot meet exact owner/reason/expiry/removal and shrink-only rules;
- final docs cannot be derived from the live graph because ownership is unsettled;
- closure would require Query/cache work, live optimization, bundle/chunk work,
  backend/API/product redesign, another package/runner/parser, Node, foreign
  package managers, or manual generated/shadcn edits; or
- a required targeted/full gate fails twice after one reasonable correction.

## Maintenance And Removal

After closure, every UI change updates its owner, facade/edge, runtime contract,
source/browser tests, matrix rows, and ratchets in the same change. The final
documentation and machine graph must agree; reviewers reject generic buckets,
deep imports, route implementation exports, unowned exceptions, and hidden
compatibility paths.

Delete this plan and its README row only after the closure report is zero, final
docs and policy are durable, every done criterion is satisfied, and the complete
command table passes twice from clean state.
