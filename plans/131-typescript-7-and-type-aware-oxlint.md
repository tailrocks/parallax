# Plan 131: Adopt TypeScript 7 and complete the Oxlint cutover

> **Executor instructions**: Treat TypeScript, native Oxlint, and
> `oxlint-tsgolint` as one compatibility wave. Characterize TypeScript 6 versus
> 7 diagnostics and prepare the ESLint migration while TypeScript 6 is still
> live. Then remove the sole known TypeScript-6-API consumer and install
> TypeScript 7/Oxlint in one manifest, lock, script, and config cutover. Never
> run ESLint after replacing its compiler API. Finish with one compiler and one
> linter, while keeping `tsc --noEmit` independent from experimental Oxc
> type-checking.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 095, 101
- **Category**: TypeScript / Oxc / compiler and lint tooling
- **Planned at**: `a1d8bf82`, revised 2026-07-12
- **Status**: TODO

## Why

TypeScript 7.0 became generally available on 2026-07-08 and the TypeScript team
calls it production-ready. Version 7.0.2 is the current registry release at this
plan's refresh. It has no stable programmatic compiler API until the planned 7.1
API, but Parallax is an application and does not need that API in its final
toolchain:

- Vite 8 transpiles TypeScript with Oxc and deliberately does not type-check;
- the current TanStack Start/Router, Vite, Vitest, React, and React type packages
  declare no TypeScript compiler peer;
- the TanStack route generator uses its own Babel/Oxc-adjacent tooling rather
  than the TypeScript compiler API; and
- the only current hard peer conflict is typescript-eslint through
  `@tanstack/eslint-config`, which caps TypeScript below 6.1 and is deleted by
  this Oxc migration.

A read-only 2026-07-12 probe ran the TypeScript 7.0.2 native compiler against
the full current `ui/tsconfig.json` at `a1d8bf82`; `--noEmit` passed with zero
diagnostics. The `skipLibCheck=false` probe exposed the same existing third-party
declaration classes as TypeScript 6, so plan 128 still owns that independent debt.
The current config already uses `ES2022`, ESM, bundler resolution, explicit
types, and no deprecated `baseUrl`/legacy module options.

Oxlint type-aware analysis is now aligned with this move: it is powered by
TypeScript Go/7, currently covers 59 of 61 typescript-eslint typed rules, and can
replace the incompatible ESLint path after exact parity. The 2026-07-12 registry
snapshot is `oxlint@1.73.0` plus `oxlint-tsgolint@0.24.0`. Type-aware behavior
is officially Alpha/outside normal semver. The operator's 2026-07-12 Oxc-only
direction authorizes this exact narrow pre-stable component through plan 101's
durable policy; Oxc `--type-check` remains experimental, so exact pins, negative
fixtures, and independent `tsc` remain mandatory. At this
refresh, tsgolint 0.24.0 embeds a 2026-06-25 pre-GA TypeScript Go snapshot; that
package version alone does not prove final TypeScript 7 project-model parity.

Primary status sources are the
[TypeScript 7.0 announcement](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/),
[Oxc type-aware guide](https://oxc.rs/docs/guide/usage/linter/type-aware.html),
[Oxlint stable core](https://oxc.rs/blog/2025-06-10-oxlint-stable),
[type-aware linting Alpha](https://oxc.rs/blog/2025-12-08-type-aware-alpha),
[official ESLint migration guidance](https://oxc.rs/docs/guide/usage/linter/migrate-from-eslint),
[Oxc versioning policy](https://oxc.rs/docs/guide/usage/linter/versioning.html),
[the tsgolint 0.24.0 embedded TypeScript Go revision](https://github.com/oxc-project/tsgolint/blob/5a37e8902f65440900be1436b814919fcdb4e3d4/go.mod),
and [Vite TypeScript behavior](https://vite.dev/guide/features.html#typescript).

## Scope

- One latest-stable TypeScript 7 compiler CLI/project line; no TypeScript 6
  alias. Editor integration is documented separately and is not a repository-
  owned language-server package.
- TS6-to-TS7 config and diagnostic characterization.
- Stable, non-nursery native plugins plus exact-pinned, operator-excepted
  type-aware Oxlint configuration and fixtures.
- Complete direct ESLint/TanStack config removal and transitive typescript-eslint
  lock-path elimination.
- Bun-only native compiler/linter execution, dependency/API-consumer policy, and
  stable CI diagnostics.

Out of scope:

- Runtime GraphQL/SSE/search/storage decoding and `skipLibCheck=false`, plan 128.
- Feature/cache/layout refactoring, plan 100.
- Oxfmt/Prettier migration, actionable plan 130 and independent of TypeScript.
- Oxc `--type-check` as the required compiler.
- Shipping an editor extension, embedding a TypeScript language server, or
  treating editor selection as a required repository gate.
- Alpha JavaScript plugins, experimental React compiler promotion, direct Oxc
  transforms/minification, or a TypeScript 6 compatibility alias.

## Steps

### Step 1: Refresh the compatibility proof

Resolve the latest stable TypeScript 7/Oxlint and exact policy-allowed
`oxlint-tsgolint` versions at execution time under plan 101's supply-chain
policy. In isolated temporary
directories, run current TypeScript 6 and candidate TypeScript 7 over the exact
same source/tsconfig and store structured diagnostics, effective config, file
inventory, duration, and peak memory. The candidate must pass `--noEmit` and
must not add an unexplained diagnostic.

Inspect the exact TypeScript Go revision embedded by `oxlint-tsgolint`. Promote
type-aware lint only when it uses a GA-or-newer TypeScript 7 snapshot or when a
checked-in corpus proves exact project selection, resolution, configuration,
and required-rule diagnostic parity against the GA compiler. A pre-GA revision
with only a matching package peer/version string is insufficient evidence.

Inventory every direct/transitive TypeScript peer and every source/config/script
that imports the compiler API. Current expected result: only the ESLint/
typescript-eslint branch rejects 7 and is removable in this plan. Fail the
inventory if TanStack, Vite, Vitest, React, shadcn, codegen, or a repository
script has gained a TypeScript 6 API dependency. The future GraphQL generator in
plan 128 must be Bun/TS7-compatible and cannot reopen that dependency.

### Step 2: Freeze ESLint and prepare Oxlint while TypeScript 6 is live

Before changing the live compiler dependency, capture the current ESLint
effective config, selected files, structured diagnostics, and exact rule/plugin
inventory for production, test, config, generated route, and shadcn classes.
This is the last point at which repository ESLint may execute.

In an isolated temporary package graph that retains TypeScript 6, install the
exact matched-version `@oxlint/migrate`/Oxlint candidates with Bun, installation
disabled at execution, and JS plugins disabled. Run the migrator with
`--details --type-aware --js-plugins=false` and treat its output as an inventory,
not policy. Prepare the checked-in JSON/JSONC candidate, native rule mappings,
overrides, selected-file goldens, and negative fixtures.
Do not add a second TypeScript compiler, run type-aware Oxlint against 6, or
commit an interim dual-linter/toolchain state.

### Step 3: Cut over the compiler and linter atomically

In one reviewed manifest/lock/config/script slice, use Bun to remove the direct
`eslint` and `@tanstack/eslint-config` packages plus the TypeScript 6 package,
then add the exact latest stable TypeScript 7/Oxlint and operator-excepted
`oxlint-tsgolint` set selected by plan 101. The isolated migration tool must not
enter the final graph.
Delete ESLint config, scripts, cache keys, and shadow jobs in that same slice. Do
not run or commit any intermediate graph whose TypeScript 7 package is visible
to typescript-eslint's `<6.1` peer/API path. Use `bun why` plus lock reachability
to prove every typescript-eslint, `@typescript-eslint/*`, ESLint parser/plugin/
config, and TanStack ESLint node disappeared. A specifically named unrelated
transitive ESLint owner may remain only when no Parallax command/config loads it.

Create the final `.oxlintrc.jsonc`; never use Node-required experimental
TypeScript config. Apply only TypeScript config changes required by the official
6-to-7 migration. Preserve the explicit `target`, `module`, `moduleResolution`,
`types`, strictness, no-emit, and generated/config file inclusion. Reject
`ignoreDeprecations`, removed legacy options, implicit new defaults, and
application casts used to hide compiler findings. Capture and fixture the stored
TypeScript 6 and final TypeScript 7 `tsc --showConfig` results.

The TypeScript npm wrapper has a Node shebang, so the package script runs under
plan 094's `bunfig.toml` (`[run] bun = true`, auto-install disabled) and an exact
lock-local path. Process ancestry proves Bun launches the reviewed platform-
native compiler without Node or runtime download.

Enable applicable stable, non-nursery native core, TypeScript, React, import, JSX
accessibility, promise, Unicorn/Oxc, and Vitest rules. Require native
`react/rules-of-hooks` and `react/exhaustive-deps`. Configure supplemental
`import/no-cycle` with type-only edges included, unsafe dynamic cycles disabled,
and unlimited depth; it must agree with plan 095's authoritative Oxc Rust graph.
Every selected native rule has a negative fixture. These plugins are built into
Oxlint; do not install their `eslint-plugin-*` or `@typescript-eslint/*`
counterparts.

### Step 4: Prove type-aware parity

Use the already-pinned `oxlint` and `oxlint-tsgolint` compatibility unit,
including the embedded TypeScript Go revision proven in Step 1. Compare it with
the stored pre-cutover ESLint inventory; do not reload ESLint. Map every existing
or desired unsafe, promise,
exhaustiveness, assertion, condition, throw/catch, non-null, and type-import
invariant to an exact `typescript/*` rule and positive/negative fixture. Record
implemented/missing status rather than claiming preset parity.

For either of the currently missing typed rules, use an equivalent `tsc`,
Oxc-backed xtask, runtime, or test oracle before accepting the cutover. Alpha JS
plugins are forbidden. Type-aware lint fails unmatched/zero files, warnings,
unused disable directives, parse/resolution errors, and non-deterministic
diagnostic deltas. Measure memory and duration on the full project; unexplained
non-semver changes block grouped upgrades.

Keep `tsc --noEmit` and `oxlint --type-aware` as separate required jobs. Oxc
`--type-check` remains report-only/shrink-only until upstream marks it stable and
a later plan proves complete diagnostic/config/editor parity.

### Step 5: Prove the old compiler-API/lint path is absent

After native and typed parity passes, fail closed on direct `eslint` and
`@tanstack/eslint-config`, every unowned transitive typescript-eslint/
`@typescript-eslint/*`/ESLint plugin-parser-config lock path, the migration
package, ESLint config/script/cache/shadow-job, TypeScript 6 alias/lock, and every
stale instruction.
This verifies Step 3; it does not postpone deletion until after the incompatible
compiler has already landed.

The final scripts invoke only native/type-aware Oxlint for lint and TypeScript 7
for compiler checking. Oxc's alpha JS-plugin bridge and `eslint-plugin-oxlint`
are not completion paths.

### Step 6: Make the cutover a required compatibility gate

Add separate TypeScript config/typecheck, native Oxlint, type-aware Oxlint,
selected-file/config drift, dependency peer/API-consumer, and Bun process-tree
diagnostics under the stable aggregate. Cache keys include exact TypeScript,
Oxlint/tsgolint, config, source graph, platform, and architecture. A skipped,
zero-file, wrong-platform, Node-spawned, or implicit-install result fails.

Run current Vitest, Vite production build, route generation/drift, and TanStack
SPA smoke after the compiler/linter cutover. Vite remains the Oxc-transform owner
and `tsc` remains no-emit; do not introduce a second build pipeline.

## Test Plan

- TS6 versus TS7 structured diagnostic/effective-config/file-list comparison.
- Candidate TypeScript 7 `--noEmit` on all source, tests, generated route, and
  Vite config; intentional compiler-error and removed-option fixtures.
- Dependency peer/API-import fixtures, including the current typescript-eslint
  conflict, direct-versus-transitive `bun why`/lock ownership, and a synthetic
  hidden TS6 API consumer.
- Transaction fixture proving no committed/tested manifest or lock combines
  TypeScript 7 with the old ESLint/typescript-eslint API path, and no ESLint
  command runs after compiler replacement.
- Exact TypeScript native-platform package, integrity, Bun process ancestry,
  missing-binding, auto-install, and dual-compiler failure cases.
- Exact two-entry pre-stable allowlist fixture proving `oxlint-tsgolint` is
  authorized while Alpha, expires at stable, and does not authorize JS plugins,
  type-check authority, React compiler rules, transformer, or minifier packages.
- Native/type-aware Oxlint `--rules`, effective-config, selected-file, parse/
  resolve, zero-file, memory, and non-semver delta fixtures.
- Embedded TypeScript Go revision plus GA project/config/resolution parity
  evidence when the pinned `oxlint-tsgolint` does not contain a GA-or-newer
  snapshot.
- One positive/negative case per required native/typed rule and every mapped
  former ESLint diagnostic.
- Proof that experimental Oxc type-check cannot satisfy `tsc --noEmit`.
- Stale ESLint/typescript-eslint/TanStack-config/TS6 alias/script/cache searches.
- Vitest, route generation, Vite production build, and SPA smoke under Bun.

## Done Criteria

- [ ] One latest-stable TypeScript 7 compiler owns repository CLI/project policy;
  TypeScript 6, preview/nightly, and aliases are absent. Optional editor setup
  points to current vendor integration and is not represented as a bundled
  `tsserver` or required repository gate.
- [ ] Candidate TS7 diagnostics are clean and every TS6-to-7 config/default
  change is explicit and fixture-tested.
- [ ] No required dependency or repository tool embeds the missing TS7
  programmatic API; every peer range accepts the final graph.
- [ ] Native and type-aware Oxlint select every intended file and fail every
  intentional native/typed defect with bounded resource use.
- [ ] Plan 101's exact `oxlint-tsgolint` exception is present, grouped with
  stable Oxlint, and no broader pre-stable Oxc surface is authorized.
- [ ] The exact `oxlint-tsgolint` TypeScript Go revision is GA-or-newer, or its
  project model and required diagnostics have exact checked-in GA parity proof.
- [ ] `tsc --noEmit` remains an independent required compiler gate and Oxc
  type-check remains non-authoritative.
- [ ] Direct `eslint` and `@tanstack/eslint-config` are absent; `bun why` and lock
  reachability prove every typescript-eslint/`@typescript-eslint/*` and ESLint
  plugin/parser/config node is absent or has one named unrelated non-invoked
  transitive owner. Migration/shadow jobs and TS6 compatibility packages are
  absent.
- [ ] All compiler/lint commands are exact, lock-local, Bun-only, platform-
  complete, and incapable of implicit install/runtime download.
- [ ] Full UI tests, route generation, Vite build, and SPA smoke pass.

## STOP Conditions

- Candidate TypeScript 7 adds an unexplained application or declaration
  diagnostic relative to the characterized TypeScript 6 baseline.
- Any transition would execute or commit the old ESLint/typescript-eslint path
  after TypeScript 7 replaces the compiler package.
- Plan 101's `oxlint-tsgolint` exception is missing, broader than intended, or
  cannot expire cleanly when type-aware linting becomes stable.
- A required non-ESLint dependency/tool needs the missing TypeScript 7 API or
  has an incompatible peer with no current stable release.
- A strict lint invariant has no Oxlint or separately enforceable oracle.
- Type-aware Oxlint skips files, exceeds an evidence-backed resource ceiling,
  or changes non-deterministically at the exact grouped pin.
- Compiler/linter execution needs Node, a foreign manager, implicit download,
  preview/nightly tooling, or a TypeScript 6 side-by-side alias.
- Vite/TanStack/Vitest/route-generation behavior or SPA output changes.

## Remove When

Delete this plan and index row when TypeScript 7 is the sole compiler,
native/type-aware Oxlint are the sole linters beside independent `tsc`, the old
ESLint/TS6 API path is gone, and every compatibility/runtime/UI gate is green.
