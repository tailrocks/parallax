# Plan 148: Enforce route-owned production chunks and deterministic bundle budgets

> **Executor instructions**: Begin only after plan 151 closes the move-only UI,
> plan 133 installs the sole final cache, and plan 147 lands final live-data
> modules. Measure that settled graph before changing build behavior.
> Optimize route/chunk reachability, supported Vite/Rolldown minification, and
> source-map delivery only. Do not change Query/cache/live algorithms, product
> behavior, or install direct Oxc transformer/minifier packages. Every claimed
> improvement must be visible in a production manifest and a real browser
> resource trace from two clean deterministic builds.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src ui/package.json ui/bun.lock ui/vite.config.ts ui/tsconfig.json ui/test-matrix.json ui/tests/e2e crates/parallax-cli crates/parallax-xtask .github/workflows/ci.yml ratchet.toml`
> Resolve route and feature paths through the plan 151 ownership ledger. If the
> embed or release artifact owner moved, preserve its live public facade and
> re-run the source-map leakage baseline before editing.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 095, 100, 101, 105, 132, 133, 144, 146, 147, 151
- **Category**: performance / build / delivery
- **Planned at**: `e3e7997`, revised 2026-07-12
- **Status**: IN PROGRESS — production build contract + analyze script landed; residual budgets/ratchets/@bundle

## Why This Matters

The baseline `ui/vite.config.ts:7-36` configures TanStack Start, React,
Tailwind, tests, and the dev proxy, but has no explicit production chunk,
minification, source-map, manifest, or budget contract. The UI's largest route
modules are 1,500, 990, 871, 841, and 767 lines and import heavy charts,
virtualizers, trace visualizations, and feature components. Source refactoring
alone does not prove those features are absent from the initial browser graph.

This plan makes production loading a compiler/build artifact contract: shell and
route code have named ownership, client/server boundaries cannot leak, clean
builds are comparable, and CI fails regressions before release embedding hides
them inside the Rust binary.

## Fixed Decisions

1. TanStack Start's supported Vite/Rolldown pipeline owns route generation,
   transforms, chunking, source maps, and production minification. Use the exact
   locked stable integration; do not install direct Oxc transformer/minifier
   JavaScript bindings or run a second transform/minification pass.
2. Generated `routeTree.gen.ts` remains generator-owned. Enable or configure the
   exact stable plugin's supported automatic route code splitting only after a
   small manifest/browser spike proves its semantics; never hand-edit the tree.
3. A route module remains a thin adapter. Feature facades must not eagerly
   re-export every component, and `app`/`layout` cannot import feature
   implementations merely to make navigation metadata convenient.
4. Initial shell loading may contain React/TanStack/layout/shared primitives but
   no route-only chart, virtualizer, trace/log/run/dashboard/SQL implementation,
   server module, filesystem/process API, secret, or full feature registry.
5. Route navigation loads that route's feature chunk and approved shared chunks.
   It does not eagerly load unrelated feature chunks. Cross-feature imports must
   match plan 100's explicit facade edge ledger.
6. Prefer framework-supported route lazy boundaries. Add component-level lazy
   loading only for a measured heavy optional surface, with a stable accessible
   pending/error state and no request waterfall. Manual chunk rules require
   before/after graph evidence and cannot create a giant vendor bucket.
7. Production minification remains enabled through supported Vite/Rolldown.
   Development/test output remains debuggable; minified production is never used
   as a substitute for type/lint tests.
8. Hidden client source maps, when produced, are private CI/release diagnostic
   artifacts keyed to the exact build. They are excluded from the embedded/public
   UI tree and have no `sourceMappingURL` exposure. Do not silently disable all
   maps or serve them publicly.
9. Size, reachability, module duplication, map leakage, and two-clean-build
   determinism are required gates. Runtime navigation/resource/interaction
   traces prove that a smaller manifest did not introduce lazy waterfalls.
10. Every `@bundle` row uses its final route/feature architecture ID as
    `scenario_owner`, `performance/bundle` as `lane_owner`, and 148 only as
    temporary `delivery_plan`. Bundle analysis never becomes product ownership.

## Target Artifacts

```text
target/ui-bundle/
  baseline.json
  current.json
  module-graph.json
  route-reachability.json
  duplicate-modules.json
  clean-build-a.json
  clean-build-b.json
  source-map-manifest.json
  browser-resource-trace.json
ratchet.toml                    # exact accepted size/reachability ceilings
```

Generated reports live outside source and are uploaded by CI. Only reviewed
ratchets and any explicitly approved hidden-map delivery manifest are checked
in. The production client directory embedded by `parallax-cli` contains no map
file or analysis artifact.

## Budget Contract

Step 0 records raw, gzip, and Brotli sizes for the initial entry, CSS, every
route-owned chunk, approved shared chunks, and total client graph. It also
records module-to-chunk ownership, duplicate bytes, dynamic-import edges, and
the browser resources loaded for direct entry and navigation.

Final required budgets are written as exact byte ceilings in `ratchet.toml` and
must satisfy all of these rules:

- the initial shell compressed bytes do not exceed the pre-plan production
  baseline and exclude every route-only module by reachability;
- no route navigation loads an unrelated feature implementation;
- no single route chunk exceeds its pre-plan equivalent; any chunk over 150 KiB
  gzip has a measured split decision or an owner/reason/expiry exception;
- total duplicate compressed JavaScript is at most 2% of total compressed JS
  and no application module appears in more than one emitted chunk unless the
  bundler manifest proves it is a shared dependency edge;
- total client compressed bytes do not increase by more than 2% without a
  separately approved product capability and compensating budget;
- two clean builds from identical inputs have identical client file names,
  hashes, sizes, module ownership, and hidden-map identity after normalizing only
  explicitly documented absolute build paths; and
- direct-entry plus navigation introduces no new serial request waterfall and
  keeps the selected route's interaction-ready p95 at or below the accepted
  pre-plan canonical-browser baseline.

After optimization, ratchets use the better measured value plus at most 1%
deterministic byte tolerance. Ratchets shrink automatically when output becomes
smaller; increases require owner, reason, evidence, expiry/removal condition,
and an explicit reviewed update.

## Commands

| Purpose | Command | Expected result |
|---------|---------|-----------------|
| Exact install | `cd ui && bun ci` | frozen lock, no lifecycle scripts |
| Production analysis | `cargo xtask ui-bundle analyze` | production manifest/module/size/map reports produced |
| Clean determinism | `cargo xtask ui-bundle build-twice` | normalized outputs identical |
| Budget policy | `cargo xtask policy --only ui.bundles` | size, reachability, duplication, map, and dependency rules pass |
| Browser resources | `cd ui && bun run test:browser -- --grep @bundle` | direct-entry/navigation resource assertions pass |
| Cross engine | `cd ui && bun run test:browser:cross -- --grep @bundle` | selected Firefox/WebKit loading and interaction pass |
| UI checks | `cd ui && bun run check && bun run lint && bun run typecheck && bun run --bun test:ci && bun run build` | all exit 0 |
| Embed check | `cargo check --locked -p parallax-cli --features embed-ui` | exact production client embeds, no maps/reports |
| Fast aggregate | `cargo xtask ci --fast` | bundle gate selected and green |

If plan 102 has already created the release artifact helper, extend its public
source-map manifest contract instead of adding another byte-producing path. Do
not make plan 148 responsible for release signing or archive publication.

## Scope

In scope:

- Production manifest/module graph/size/duplicate/reachability/source-map
  analysis through Rust xtask and supported Vite output.
- TanStack-supported route code splitting, facade import corrections, and
  measured optional-component lazy boundaries.
- Exact bundle ratchets, deterministic clean-build comparison, browser resource
  traces, source-map exclusion from embedded/public output, and CI integration.
- Stable `@bundle` matrix IDs for each direct-entry/navigation resource trace,
  with one declared route/project/spec owner and non-empty selection proof.
- Small facade/export adjustments required to prevent eager feature reachability,
  while preserving plan 151 ownership and behavior.

Out of scope:

- Feature restructuring, public URL/API changes, Query/cache behavior (plan
  133), live update algorithms (plan 147), visual redesign, dependency upgrades
  unrelated to the proven graph, or release signing/packaging (plan 102).
- Direct Oxc transform/minifier packages, a second bundler, hand-edited generated
  route files, giant manual vendor chunks, arbitrary dynamic imports, public
  source maps, CDN changes, or Node-based analyzers.

## Git Workflow

- Stay on the one active branch; do not create a branch or PR.
- Land baseline/analyzer, route/import corrections, optional lazy boundaries,
  and final ratchets/CI as separate green commits.
- Use Conventional Commits, DCO, and exactly one agent-product trailer.
- Push every durable green update.

## Steps

### Step 0: Capture the production build and browser baseline

Verify plans 105, 133, and 147 have retired from the active index and their
metric-summary/polling, sole-cache/client-lifetime, and bounded-stream/feature-
merge gates pass at the same commit. STOP if any known graph-changing wave
remains active or if an old metric/cache/live compatibility path is reachable.

Build from a frozen Bun lock in a clean temporary output directory twice. Parse
the supported Vite manifest and emitted source maps/module metadata with xtask;
do not scrape terminal text. Record every entry/dynamic/shared asset, module
owner, raw/gzip/Brotli size, duplicate module, server/client marker, and map.

Use Playwright to capture resource timing for direct entry to each route and a
representative shell-to-route navigation. Record request order, initiator,
transferred/decoded bytes where available, loaded chunk identities, and
interaction-ready mark. Use deterministic fixture data, canonical browser
environment, and no external network.

Register every resource trace as an exact `@bundle` row in
`ui/test-matrix.json`, including route, direct/navigation state, owning spec,
project, expected initial/feature chunks, and budget report key. Reuse plans
144/146 fixtures and projects without changing their infrastructure.

**Verify**: both baseline builds are inventory-complete; every emitted JS/CSS/
map belongs to one manifest row; every shipped route has direct/navigation
browser traces; unexplained nondeterminism is recorded before optimization.

### Step 1: Add parser-backed bundle policy and negative fixtures

Extend xtask/Oxc policy to combine source imports, TanStack route ownership, the
Vite module graph, emitted manifest, and browser trace. Fail server-only modules
or APIs in client chunks, secrets/config values in emitted text, deep feature
imports, eager feature implementations from app/layout/routes, unrelated route
reachability, duplicate application modules, public maps, missing manifest
rows, and stale/unowned ratchets.

Create synthetic build fixtures for each failure. Do not use regex alone for
TypeScript import ownership or infer a dynamic edge solely from file names.

**Verify**: every positive fixture passes and each negative fixture fails only
its named stable rule with file/module/chunk/route evidence.

### Step 2: Establish supported route-level code splitting

Inspect the exact locked TanStack Start/router plugin and generated production
manifest. If automatic route splitting is already active, preserve it and fix
eager imports that collapse boundaries. If not, enable only its supported stable
configuration and regenerate the route tree through the existing Bun script.

Make route adapters import the smallest feature facade entry. Split facade entry
points by stable responsibility only when a single root facade itself causes
eager reachability; keep explicit exports and forbid deep imports. Navigation
metadata must not import page implementations.

**Verify**: every route has an owned dynamic entry or documented shell/root
reason, direct entry renders, navigation/prefetch behavior remains correct, and
unrelated feature code is absent from resource traces.

### Step 3: Split only measured heavy optional surfaces

Rank route chunks and module contributors. For any route over the threshold,
first remove accidental imports/duplicates. Then consider a component-level
lazy boundary only for a genuinely optional heavy surface such as a trace
visualizer, chart editor, SQL result renderer, or export tool.

Provide accessible stable pending/error behavior, preserve typed props through
the feature facade, preload on the router/user intent when supported, and avoid
a fetch-then-import or import-then-fetch serial waterfall. Do not split small
components merely to reduce a file-size number.

**Verify**: each split has before/after bytes and browser waterfall/interaction
evidence; route behavior, keyboard/accessibility, visual, and cross-browser
cases pass; removing the split in a negative fixture violates its measured
budget.

### Step 4: Fix minification and hidden source-map delivery

Record the resolved Vite/Rolldown transform/minifier implementation and version
in the build report. Keep production minification through the supported pipeline
with no direct competing package. Generate hidden maps only when the exact map
can be retained as a private CI/release diagnostic artifact; strip/exclude map
files and mapping comments from the client directory before embedding.

Map manifests bind source commit, lock/tool versions, build identity, emitted
asset hash, map hash, and private artifact location. Prove a production error
frame can be symbolicated from the retained exact map in a local verification
fixture, while a request to the served/embedded `.map` path returns no source.

**Verify**: minified production, readable development, private symbolication,
no public/embedded map, no second minifier, and wrong-map/hash negative fixtures
all behave as declared.

### Step 5: Commit shrink-only budgets and required CI

Write exact accepted byte/reachability/duplication/determinism/interaction
ratchets from the final two clean builds. Add a path-aware bundle job for UI,
route/build/config/lock, embed, xtask/policy, and workflow changes. Upload
machine reports and browser traces; keep private maps in access-controlled,
bounded-retention artifacts rather than the public bundle report.

The stable aggregate distinguishes irrelevant skip from selected missing/
zero-manifest failure. CI never updates ratchets or baselines automatically.

**Verify**: actionlint/path fixtures pass; intentional byte growth, eager edge,
duplicate module, server leak, nondeterminism, waterfall, and map exposure each
fail the bundle job and `ci-required`; unchanged clean output passes.

### Step 6: Close temporary analysis and document ownership

Remove temporary logging, analyzer-only source changes, obsolete manual chunk
experiments, and any compatibility facade made unnecessary by final boundaries.
Update `ui/AGENTS.md` and `PROJECT_STRUCTURE.md` with route/facade/lazy/source-map
placement and the exact budget-update review protocol.

**Verify**: no analysis code enters production chunks, no unowned exception
remains, and every command in this plan passes twice from clean state.

## Test Plan

- Manifest/module graph/size/compression/duplicate/source-map parser tests.
- Oxc/source-to-chunk negative fixtures for deep/eager/server/secret/dynamic
  imports and incorrect feature ownership.
- Two-clean-build hash/size/ownership determinism and normalization tests.
- Browser direct-entry/navigation resource, preload, pending/error, interaction,
  cross-engine, and no-external-network cases.
- Minifier ownership, private symbolication, wrong map/hash, public map request,
  and embedded artifact tests.
- CI selection, zero-manifest, regression, artifact, skip, and aggregate tests.

## Done Criteria

- [ ] Plans 105, 133, and 147 are retired and their metric, cache, and live
  graph changes are present in the measured commit with no compatibility path.
- [ ] The initial shell and every route have explicit source-to-emitted-chunk
  ownership with no unrelated eager feature or server-only module.
- [ ] Supported TanStack/Vite/Rolldown splitting and minification are the only
  build path; no direct transformer/minifier or second bundler exists.
- [ ] Exact shrink-only byte, reachability, duplication, determinism, and browser
  interaction ratchets pass from two clean builds.
- [ ] Every shipped route has non-empty `@bundle` rows with one final feature
  `scenario_owner` and `lane_owner: performance/bundle` for direct entry and representative
  navigation, with no collapsed, orphan, or duplicate row.
- [ ] Measured heavy optional surfaces split without new request waterfalls or
  pending/error/accessibility regressions.
- [ ] Hidden maps symbolicate the exact build privately and are absent from the
  served/embedded client output.
- [ ] Path-aware bundle CI is part of the stable aggregate and cannot update its
  own ratchets or baselines.
- [ ] Every command in this plan passes twice from clean state.

## STOP Conditions

Stop and report if:

- plan 105, 133, or 147 remains active, has a red gate, or leaves a reachable
  compatibility path that can change the measured graph;
- the exact TanStack/Vite version cannot provide a supported route split without
  hand-editing generated files or installing a second bundler;
- a claimed split changes URL/search/loader/cache behavior or adds a serial
  fetch/import waterfall;
- meeting size budgets requires removing approved product behavior, weakening
  runtime validation, or changing live/cache semantics;
- source maps cannot be retained privately and excluded from public/embed output;
- clean builds remain nondeterministic after known absolute paths are normalized;
- direct Oxc transformer/minifier, Node tooling, or a giant unmeasured vendor
  chunk appears necessary; or
- a required gate fails twice after a reasonable correction.

## Maintenance And Removal

Every new feature/dependency updates manifest ownership, route/browser resource
evidence, and exact shrink-only budgets in the same change. Reviewers inspect
initial-entry reachability and interaction waterfalls, not only total byte size.

Delete this plan and its README row only after final route/import/lazy ownership,
private source-map contract, deterministic builds, budget policy, browser traces,
and required CI are durable and green.
