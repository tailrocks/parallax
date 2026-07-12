# Oxc implementation contract and official lookups

- **Status:** Active implementation companion for plans 094, 095, 100, 101,
  107, and 128-153
- **Last verified:** 2026-07-12
- **Parallax baseline:** `e3e7997933801e0e78804d32f0973181036bb617`
- **Upstream:** [Oxc](https://oxc.rs/) and its official documentation

This file fixes what Parallax will use from the Oxc toolchain, what it will not
use, and the gates required to retire the older direct ESLint/Prettier
toolchain. It is an execution-time companion to the numbered implementation
plans. The normative end state also appears in
[`ENGINEERING-STANDARDS.md`](ENGINEERING-STANDARDS.md), and executable ownership
is split across the numbered plans named above.

## Decision

Parallax adopts an **Oxc-only lint and format toolchain**:

1. Stable Oxlint core/native plugins plus exact-pinned type-aware
   `oxlint-tsgolint` become the only required JavaScript/TypeScript linter after
   parity proof. Direct `eslint` and `@tanstack/eslint-config` ownership is
   removed before plan 131 completes; lock reachability proves their transitive
   typescript-eslint/plugin/parser/config path disappeared or names an unrelated
   non-invoked owner. The old typed lane runs only during pre-cutover TypeScript
   6 characterization, never after TypeScript 7 lands.
2. Exact-pinned Oxfmt becomes the only JavaScript/TypeScript and supported
   frontend-text formatter after differential proof; rustfmt retains Rust.
   Direct Prettier and `prettier-plugin-tailwindcss` ownership is removed in plan
   130's atomic cutover.
3. `parallax-xtask` uses the compatible Rust `oxc_parser`, `oxc_ast`,
   `oxc_semantic`, and `oxc_resolver` crates as the single TypeScript syntax and
   module-graph engine. Regex and a second ESLint-derived architecture graph are
   forbidden.
4. Vite 8/TanStack Start retain ownership of the build pipeline. Their Rolldown
   path already uses Oxc components; Parallax verifies that path but does not
   add a competing direct transformer or minifier.
5. `tsc --noEmit` remains the canonical independent compiler gate. Oxlint's
   type-aware rules are required after validation, but experimental Oxc
   type-checking cannot become the sole type checker until upstream marks it
   stable and a separate parity decision proves equivalent diagnostics.

The direction is explicit, but it is not a license to turn alpha features into
hollow required checks. An Oxc component is promoted only when the exact pinned
version, file selection, diagnostics, Bun process tree, and negative fixtures
are reproducible on every supported development/CI platform.

The operator's 2026-07-12 direction authorizes exactly two pre-stable exceptions
needed for this end state: Oxfmt while officially Beta and `oxlint-tsgolint`
while type-aware linting is Alpha/outside semver. Plan 101 records that two-entry
allowlist durably and mechanically. It expires each entry at stable and does not
authorize alpha JS plugins, experimental type-check authority or React compiler
rules, or direct transformer/minifier adoption.

## Current Parallax Baseline

At the baseline:

- `ui/package.json` directly runs ESLint 9 through `@tanstack/eslint-config`
  and formats with Prettier 3 plus `prettier-plugin-tailwindcss`;
- the ESLint config disables cycle and import-order rules, and does not supply
  the strict typed/promise/unsafe/React rule inventory required by this program;
- Prettier owns only JS/TS-family files, excludes `routeTree.gen.ts`, uses an
  80-column, no-semicolon, double-quote contract, and Tailwind-sorts classes
  found through `src/styles.css`, `cn`, and `cva`;
- TypeScript 6 is the pre-plan-131 baseline and is strict, but `skipLibCheck`
  remains enabled;
- the 2026-07-12 registry snapshot is `typescript@7.0.2`, `oxlint@1.73.0`,
  `@oxlint/migrate@1.73.0`, `oxlint-tsgolint@0.24.0`, and Beta
  `oxfmt@0.58.0`; these are dated evidence, not freezes, and execution
  re-resolves the latest policy-allowed compatible set;
- Vite 8 resolves to Rolldown 1.0.3, and the lockfile already contains Oxc
  parser bindings transitively through TanStack development tooling; and
- no repository-owned TypeScript AST/resolution implementation exists for the
  architecture ratchet.

Transitive Oxc, ESLint, or Prettier packages do not prove adoption or violate
the end state by themselves. A framework or route generator may retain an
internal dependency. Parallax controls direct dependencies, commands,
configuration, lifecycle trust, and which executable is accepted as a gate.

## Upstream Capability And Maturity Matrix

| Component | Upstream capability | Parallax decision |
|-----------|---------------------|-------------------|
| Oxlint native lint | Stable v1 core plus non-nursery ESLint/TypeScript/Unicorn/Oxc defaults and native React, import, JSX accessibility, promise, and Vitest plugins; multi-file rules and several output formats | Adopt as the only final native linter; install no separate ESLint plugin packages |
| Oxlint type-aware lint | Officially Alpha/outside semver; `oxlint-tsgolint`, TypeScript Go/TS7 configuration, and 59 of 61 targeted typescript-eslint typed rules | Adopt under the exact two-entry operator exception after version/revision/rule/config/memory parity proof; never claim universal typescript-eslint parity |
| Oxlint type-check | CLI type checking integrated with the linter | Keep report-only while upstream labels it experimental; never replace `tsc` yet |
| Oxlint JavaScript plugins | Alpha compatibility layer for JavaScript ESLint plugins | Do not install, configure, or execute JS plugins in the live/final graph; the two-entry exception does not authorize them |
| Oxfmt | Officially Beta but recommended by Oxc; broad JS/TS/JSON/CSS/Markdown-family formatting, 100% Prettier JS/TS conformance, native Tailwind/import/package sorting | Adopt now under the exact two-entry operator exception after differential parity; retain the Beta label and risk gates |
| Oxc parser/AST/semantic | Rust-native JS/TS/JSX/TSX parsing and semantic analysis | Adopt in xtask policy providers |
| Oxc resolver | Configurable Node/TypeScript-style module resolution in Rust | Adopt as the sole architecture-graph resolver |
| Oxc transformer | Direct surface remains Alpha | Consume through supported Vite/Rolldown only; no duplicate direct build path or exception |
| Oxc minifier | Explicitly Alpha; used internally by Rolldown | Consume through supported Vite/Rolldown only; no direct package or exception |
| Oxlint language server | Editor diagnostics from the Oxlint configuration | Optional editor convenience; CLI remains authoritative; this is distinct from TypeScript 7 editor/LSP integration |

Status is time-sensitive. Plans 094, 101, 128, 130, and 131 capture the upstream
status page, versions, checksums/integrity, release notes, and compatibility at
execution time. Plan 101 must land the exact two-entry exception before plans
130/131 change dependencies. If either component becomes stable, its exception
is removed; if its required behavior regresses, the owning plan stops. There is
no permanent dual-linter or dual-formatter end state.

The final direct UI tooling graph contains TypeScript 7, `oxlint`,
`oxlint-tsgolint`, and `oxfmt`. `@oxlint/migrate` exists only in an isolated
matched-version migration graph and is removed. No direct `eslint`,
`@tanstack/eslint-config`, Prettier, Prettier plugin/config/command, or separate
`eslint-plugin-*` package remains. No unowned typescript-eslint,
`@typescript-eslint/*`, parser/plugin/config, or ESLint lock path is reachable;
`bun why` names and proves any unrelated non-invoked transitive owner.

## Oxlint Target

### Configuration and invocation

Use a checked-in JSON/JSONC configuration such as `.oxlintrc.json` or
`.oxlintrc.jsonc`. Do not use `oxlint.config.ts`: upstream marks TypeScript
configuration experimental and its loading path requires Node, which violates
the Bun-only runtime rule. Oxc npm wrappers use `#!/usr/bin/env node`, so plain
`bunx` or `bun run` does not prove runtime ownership. Required scripts force Bun
for the exact lock-local command and disable installation, conceptually:

```text
bun run --bun --no-install lint
bun run --bun --no-install lint:type-aware
bun run --bun --no-install lint:report
```

Plan 094 also sets global `ui/bunfig.toml` `[run] bun = true` and
`[install] auto = "disable"`, covering Oxc and every other Node-shebang CLI
(Vite, Vitest, `tsc`, interim ESLint/Prettier, codegen, and shadcn). Exact
per-script process fixtures remain defense in depth; no `bunx ...@latest` is an
accepted reproducible command.

The required lane uses warnings as failures, reports unused disable directives,
and fails when the expected handwritten file inventory is empty or incomplete.
CI output uses Oxc's GitHub or SARIF formatter; local/agent output may use the
human or `agent` formatter. JSON diagnostics are schema-fixtured before xtask
consumes them. `--debug=files` or an equivalent machine inventory proves which
files were selected, while `--print-config` fixtures prove overrides.

### Native rule inventory

The final rule table is explicit by exact IDs supported by the pinned Oxc
version, severity, file class, and negative fixture. Native stable rules and
non-semver type-aware rules are labeled separately. It does not trust a preset
name to remain unchanged.
Required families include:

- correctness and suspicious JavaScript/TypeScript behavior;
- `typescript/no-explicit-any` and the available type-aware unsafe, promise,
  exhaustiveness, assertion, throw/catch, and unnecessary-condition rules;
- native `react/rules-of-hooks` and `react/exhaustive-deps`;
- native import resolution, cycle, duplicate, and forbidden-edge signal;
- promise, JSX accessibility, Unicorn, Oxc, and Vitest correctness rules that
  apply to the actual file class; and
- unused-disable reporting plus a Parallax reason/owner/expiry ratchet.

Oxc's rule namespace is used directly. Do not preserve stale
`@typescript-eslint/*` names in the target config. Generated route code and
shadcn output receive exact path/rule overrides only where generator ownership
requires them; security, promise, runtime-boundary, and architecture defects may
not disappear through a blanket generated-file ignore.

Configure native `import/no-cycle` with `ignoreTypes:false`,
`allowUnsafeDynamicCyclicDependency:false`, and no finite depth limit so its
fast diagnostic sees the type-only and dynamic edges Parallax treats as
architecture. It remains supplemental to the xtask graph and must pass the same
alias/barrel/package/server-client cycle fixtures.

React compiler-derived diagnostics are not conflated with the stable Hooks
rules. Native Hooks correctness is required. Experimental
`react/react-compiler` and compiler-derived purity/immutability/static-component
diagnostics are disabled in the live graph. They may be evaluated only in an
isolated non-gating spike and promoted only after upstream marks the exact rule
stable/non-nursery, a separate operator-approved plan and durable policy change
authorize it, and deterministic Parallax fixtures pass.

### Type-aware contract

Oxlint's type-aware mode is officially Alpha, outside normal semantic-version
guarantees, and coupled to `oxlint-tsgolint` and the TypeScript Go/TS7 project
model. Plan 101's exact operator exception is therefore a prerequisite. TypeScript
7.0 became GA on 2026-07-08 and is the latest stable compiler line. A read-only
Parallax probe with 7.0.2 passes `ui/tsconfig.json`; disabling `skipLibCheck`
finds the same existing third-party declaration classes under 6 and 7. Version
7.0 has no stable programmatic compiler API, but the current application does
not consume that API after its incompatible typescript-eslint path is removed.
Therefore:

1. plan 101 records maturity, supply-chain, peer-range, API-consumer, and
   platform evidence; plan 131 pins latest stable TypeScript 7/Oxlint plus the
   exact policy-allowed `oxlint-tsgolint` release as one reviewed unit;
2. plan 131 runs the native/type-aware migration checks and records exactly which
   typed rules are implemented and selected, rather than claiming generic
   typescript-eslint parity;
3. a representative fixture corpus compares old diagnostics, native Oxlint,
   type-aware Oxlint, and `tsc --noEmit` during the migration;
4. every selected typed rule has a positive and negative case, and the file
   inventory includes production, test, config, generated-route, and shadcn
   classes deliberately; and
5. cache keys include the Oxc pair, TypeScript, tsconfig, Oxc config, source
   graph, platform, and architecture.

`oxlint --type-aware` becomes required only after those conditions pass on the
latest stable TypeScript 7 toolchain. Current `oxlint-tsgolint` 0.24.0 embeds a
2026-06-25 pre-GA TypeScript Go snapshot, so its version number alone is not
proof: plan 131 requires a GA-or-newer snapshot or an exact diagnostic/project-
model parity proof before promotion. The old typed ESLint lane remains only for
the stored pre-cutover TypeScript 6 baseline and is deleted in the atomic
compiler/linter replacement. Oxfmt is wholly
independent of this compiler decision and can format TypeScript 7 source. The
experimental `--type-check` surface remains report-only and shrink-only. It may
be reconsidered only after upstream stability plus a separate plan proves
diagnostic, project-reference, declaration, editor, and CI parity with `tsc`.

### ESLint migration and deletion

Add the official migration utility to an isolated exact temporary Bun manifest/
lock while TypeScript 6 is still live, and run it with Bun forced and automatic
installation disabled, conceptually

```text
bunx --bun --no-install @oxlint/migrate --details --type-aware --js-plugins=false
```

Never use plain `bunx` or `npx`. Match its version to Oxlint, fixture the process
tree, and treat generated output as a starting inventory, not accepted policy.
It never enters the final repository graph.
Migration order:

1. under TypeScript 6, freeze effective ESLint config, files, and diagnostics;
2. in the isolated graph, migrate rules/overrides/ignores and normalize them to
   native Oxc IDs;
3. prepare native/type-aware fixtures and classify unmatched high-value behavior
   against Oxc, `tsc`, xtask, runtime, or test oracles;
4. atomically remove TypeScript 6 plus direct `eslint` and
   `@tanstack/eslint-config`, then add TypeScript 7 plus the reviewed
   Oxlint/tsgolint pair;
5. run native/type-aware parity against stored baseline artifacts without
   loading ESLint again; and
6. use `bun why` and lock reachability to prove old config, scripts, direct
   dependencies, transitive typescript-eslint/plugin/parser/config paths, cache
   keys, aliases, and migration utility are absent or have one named unrelated
   non-invoked owner before plan 131 retires.

Do not retain residual ESLint for TanStack Query rules. Plan 133 translates the
important Query invariants (`stable-query-client`, exhaustive dependencies,
stable option ownership, no unstable dependency use, no void query function)
into typed facade APIs, Oxc-native signal where available, and Oxc-backed xtask
AST/architecture fixtures. Alpha JavaScript plugins are not installed,
configured, or executed as either a required or optional substitute.

## Oxfmt Target

### Configuration parity

Use `.oxfmtrc.jsonc`, not executable TypeScript configuration.
The initial migration preserves:

```text
endOfLine = lf
semi = false
singleQuote = false
tabWidth = 2
trailingComma = es5
printWidth = 80
sortTailwindcss.stylesheet = src/styles.css
sortTailwindcss.functions = [cn, cva]
```

The exact property spelling is taken from the pinned Oxfmt version's official
config reference and validated with a deliberately unsorted Tailwind fixture.
Oxfmt's defaults are not silently accepted: its different print-width and
package-sorting defaults would create unrelated churn.

Start `sortImports` and `sortPackageJson` disabled during parity. Either enable
one later in its own mechanical, fixture-backed step or record it disabled in
the final config. Do not combine import sorting, package-key sorting, generator
refresh, and product refactoring.

### Surface and generated ownership

Unlike the current JS/TS-only Prettier script, Oxfmt can cover JSON/JSONC,
styles, Markdown, and other supported text. Expansion is staged by file class:

- first migrate the current handwritten JS/TS/JSX/TSX surface byte-for-byte or
  with an isolated reviewed baseline;
- keep `routeTree.gen.ts` generator-owned and drift-checked, not hand-formatted;
- decide and fixture whether shadcn output is formatted after generation or
  retained verbatim;
- add eligible frontend JSON/JSONC/CSS/Markdown/GraphQL classes only after a
  clean differential and generated/documentation ownership review; record an explicit
  generator/semantic/stronger-formatter exclusion for every other supported
  repository class; and
- use explicit ignore patterns for binary, generated, vendored, fixtures whose
  whitespace is semantic, and build-output paths.

Required scripts expose write, check, and changed-file diagnostics using
`oxfmt`, `oxfmt --check`, and the pinned version's list-different support. A
no-file or accidentally ignored handwritten surface fails.

### Prettier migration and deletion

Run Oxfmt's official Prettier migration command through Bun, capture its
unsupported-option report, and compare two clean formatter runs. The migration
must prove:

- second-run idempotence;
- current handwritten formatting or an isolated approved mechanical delta;
- Tailwind class ordering for `src/styles.css`, `cn`, and `cva`;
- line-ending consistency on macOS and Linux;
- generated route/shadcn/fixture ownership; and
- identical local/CI file selection.

Then delete direct Prettier, `prettier-plugin-tailwindcss`, `.prettierrc`,
`.prettierignore`, Prettier scripts, and their cache/config references. A
transitive Prettier used internally by the TanStack route generator may remain,
but Parallax never invokes it as its formatter. There is no dual-formatter end
state.

## Rust-Native Oxc Architecture Engine

Plan 095 owns one parser/resolver implementation in `parallax-xtask`:

- `oxc_parser` and `oxc_ast` enumerate modules, imports, exports, functions,
  components, hooks, assertions, suppressions, and generated markers;
- `oxc_semantic` supplies scopes/references where a rule needs semantic rather
  than syntactic evidence;
- `oxc_resolver` resolves tsconfig paths, extension/index resolution, type-only
  and dynamic edges, barrels, generated route composition, and platform/client/
  server conditions; and
- a repository wrapper normalizes paths and emits the versioned Parallax
  diagnostic schema.

The parser/AST/semantic crates use one exact compatible core-family version;
the separately versioned resolver is pinned to a proven compatible release.
Fixtures cover
TS/TSX/JS/JSX syntax, aliases, type-only imports, `import()`, reexports, cycles,
missing files, package exports, `.server`/`.client`, generated route edges, and
parse/resolution failure. Parse or resolution failure is a finding, never an
absent edge. The architecture graph is authoritative; Oxlint
`import/no-cycle` is a fast supplemental developer diagnostic and must agree on
the shared parity corpus.

This engine enforces Parallax-specific concepts Oxc does not know: route to
feature to shared direction, feature public entries, server/client bundle
boundaries, facade size, generated ownership, and Query key/options ownership.

## Vite, Rolldown, And Build Boundary

Vite 8 and TanStack Start own transforms, route generation, chunking, source
maps, and production minification. Plan 148 verifies the resolved build graph
and records that the supported Vite/Rolldown path uses Oxc where upstream owns
it. It also measures entry/route chunk identities, compressed sizes,
source-map attribution, server-module reachability, and two clean builds.

Parallax does **not** directly install Oxc transformer/minifier JavaScript
bindings, replace TanStack route transforms, or add a second minification pass.
That would create divergent development/production semantics and duplicate
platform-native bindings. Vite+ or another Oxc-native build wrapper is not
adopted until TanStack Start officially supports it and a separate migration
plan proves all server/client, SPA, route-generation, and Bun contracts.

## Dependency, Security, And CI Policy

Plan 101 records exact versions, source, license, integrity, platform packages,
install scripts, and lifecycle trust for every direct Oxc component. Required
evidence includes:

- `bun ci` on clean macOS and Linux with an explicit `trustedDependencies`
  allowlist;
- no Node process, foreign lockfile, runtime download, or unreviewed lifecycle
  script;
- locked native binaries for every supported host architecture, with failure
  on unsupported/missing bindings rather than an opaque network fallback;
- advisory and source coverage for every lock entry, including packages a
  registry-specific audit would skip;
- exact compatible version coupling for Oxc Rust crates and for
  `oxlint`/`oxlint-tsgolint`; and
- cache keys that cannot mix Oxc binaries/config/TypeScript/platform versions.

CI keeps separate format, native lint, type-aware lint, TypeScript typecheck,
Vitest, build, architecture, and browser diagnostics under the stable aggregate
check. Oxc's `agent`, JSON, GitHub, SARIF, and JUnit outputs are useful only
after schema/empty-selection/failure-exit fixtures prove they cannot report
success while skipping work.

## Copy, Adapt, Reject

### Adopt

- Oxc-native lint and formatter executables.
- Oxc Rust parser/AST/semantic/resolver for repository policy.
- Native React Hooks, import, promise, accessibility, and Vitest lint signal.
- Type-aware lint under its exact operator exception after pinned compatibility,
  embedded-revision, memory, and fixture proof.
- Oxc's machine/agent-oriented diagnostics.
- Oxc components already owned by the supported Vite/Rolldown pipeline.

### Adapt

- Presets become an explicit Parallax rule inventory with negative fixtures.
- Formatter defaults become the existing 80-column/Tailwind contract first.
- Import/cycle lint supplements, but never replaces, the Parallax architecture
  graph.
- Generated/shadcn files get exact ownership overrides instead of broad ignores.
- Beta Oxfmt adoption gets its exact operator exception, pins, two-run
  differential proof, and a STOP condition.

### Reject Or Defer

- Node-required TypeScript Oxc config files.
- Any live/final alpha JavaScript plugin dependency, config, or execution.
- Experimental Oxc type-checking as the sole compiler.
- Experimental React compiler rules before upstream stable/non-nursery status
  plus a separate operator-approved plan and durable policy change.
- Direct transformer/minifier bindings beside Vite/Rolldown.
- Two lint or formatter stacks after migration.
- Claims that transitive Oxc packages alone constitute project adoption.

## Plan Ownership

| Plan | Oxc responsibility |
|------|--------------------|
| 094 | Keep current formatting Bun-only/required; freeze generated and parity inputs for plan 130 |
| 095 | Oxc Rust parser/AST/semantic/resolver providers and single architecture graph |
| 100 | Layer/facade/import/runtime/test ownership through the Oxc architecture graph |
| 101 | Oxc version/source/platform/lifecycle/advisory policy, exact two-entry pre-stable allowlist, and compatible upgrade units |
| 128 | TypeScript declaration strictness and static invariants on the final plan-131 toolchain |
| 129 | Vitest file-class lint fixtures and deterministic Bun-invoked frontend tests |
| 130 | Operator-authorized Oxfmt differential cutover and direct Prettier removal |
| 131 | TypeScript 7 adoption, native/type-aware Oxlint parity, direct ESLint removal, and transitive lock-path proof |
| 132 | Playwright config/spec/runtime ownership through stable configuration and Oxc-backed policy |
| 133 | TanStack Query key/options/client/invalidation and sole-cache invariants |
| 134-142 | Feature facade/import/runtime/test/browser ownership through the Oxc architecture and test policies |
| 143 | App/layout/shell facade and runtime ownership |
| 144-146 | Playwright contract/full-stack/breadth matrix invariants through Oxc-backed policy |
| 147 | Live-data facade, identity, and runtime-boundary invariants without a second parser/linter |
| 148 | Vite/Rolldown/Oxc build ownership, route reachability, chunks, maps, and deterministic bundle evidence |
| 149 | Route-less capability facade/import/test ownership |
| 150 | Overview feature/route/runtime/browser ownership |
| 151 | Final zero-exception architecture and test-topology proof |
| 152 | Generated GraphQL/config/output ownership and decoded transport policy without a second linter/parser |
| 153 | Non-GraphQL external-value facade/decode/test ownership through Oxc architecture policy |
| 107 | Independent proof that no obsolete lint/format path remains and only the exact expiring two-entry Oxc exception survives |

## Official Sources

- [What is Oxc?](https://oxc.rs/docs/guide/what-is-oxc)
- [Oxlint v1.0 stable announcement](https://oxc.rs/blog/2025-06-10-oxlint-stable)
- [Oxlint usage](https://oxc.rs/docs/guide/usage/linter)
- [Migrate from ESLint](https://oxc.rs/docs/guide/usage/linter/migrate-from-eslint)
- [`@oxlint/migrate` source and caveats](https://github.com/oxc-project/oxlint-migrate)
- [Type-aware linting](https://oxc.rs/docs/guide/usage/linter/type-aware.html)
- [Type-aware linting Alpha announcement](https://oxc.rs/blog/2025-12-08-type-aware-alpha)
- [Linter plugins](https://oxc.rs/docs/guide/usage/linter/plugins)
- [JavaScript plugin limitations](https://oxc.rs/docs/guide/usage/linter/js-plugins.html)
- [Linter versioning](https://oxc.rs/docs/guide/usage/linter/versioning.html)
- [Oxlint CLI](https://oxc.rs/docs/guide/usage/linter/cli)
- [`import/no-cycle`](https://oxc.rs/docs/guide/usage/linter/rules/import/no-cycle)
- [`react/rules-of-hooks`](https://oxc.rs/docs/guide/usage/linter/rules/react/rules-of-hooks)
- [`react/exhaustive-deps`](https://oxc.rs/docs/guide/usage/linter/rules/react/exhaustive-deps)
- [`react/react-compiler`](https://oxc.rs/docs/guide/usage/linter/rules/react/react-compiler)
- [Oxfmt usage](https://oxc.rs/docs/guide/usage/formatter)
- [Oxfmt Beta announcement](https://oxc.rs/blog/2026-02-24-oxfmt-beta)
- [Migrate from Prettier](https://oxc.rs/docs/guide/usage/formatter/migrate-from-prettier)
- [Oxfmt configuration](https://oxc.rs/docs/guide/usage/formatter/config-file-reference)
- [Oxfmt unsupported features](https://oxc.rs/docs/guide/usage/formatter/unsupported-features)
- [Oxc parser](https://oxc.rs/docs/guide/usage/parser)
- [Oxc resolver](https://oxc.rs/docs/guide/usage/resolver)
- [Oxc transformer Alpha announcement](https://oxc.rs/blog/2024-09-29-transformer-alpha)
- [Oxc minifier](https://oxc.rs/docs/guide/usage/minifier)
- [Projects using Oxc](https://oxc.rs/docs/guide/projects)
- [TypeScript 7.0 announcement and TypeScript 6 compatibility](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/)
- [TypeScript 7.0.2 registry metadata](https://registry.npmjs.org/typescript/7.0.2)
- [Oxlint 1.73.0 registry metadata](https://registry.npmjs.org/oxlint/1.73.0)
- [`@oxlint/migrate` 1.73.0 registry metadata](https://registry.npmjs.org/@oxlint%2fmigrate/1.73.0)
- [oxlint-tsgolint 0.24.0 registry metadata](https://registry.npmjs.org/oxlint-tsgolint/0.24.0)
- [Oxfmt 0.58.0 registry metadata](https://registry.npmjs.org/oxfmt/0.58.0)
- [tsgolint 0.24.0 embedded TypeScript Go revision](https://github.com/oxc-project/tsgolint/blob/5a37e8902f65440900be1436b814919fcdb4e3d4/go.mod)
- [Bun script execution and `--bun`](https://bun.com/docs/runtime/run)
- [Bunx execution](https://bun.com/docs/pm/bunx)

## Remove When

Plan 107 deletes this implementation companion when every currently executable Oxc
decision is encoded in live configuration/source/tests/policy/CI and actionable
plans 130/131 have retired. No ESLint/Prettier migration remains; durable policy,
not an active plan, owns the two exceptions until their stable-release expiry.
