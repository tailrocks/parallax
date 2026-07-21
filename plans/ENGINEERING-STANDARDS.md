# Parallax engineering structure and strictness standard

- **Status:** Active target contract for plans 093-103, 119, and 126-153
- **Last revised:** 2026-07-12
- **Parallax baseline:** `e3e7997933801e0e78804d32f0973181036bb617`
- **Applies to:** Handwritten Rust and TypeScript/React product, test, build,
  policy, and generated-code ownership

This document is the self-contained end state for the restructuring program.
Executors must not need to inspect another repository to decide where code
belongs, how tests are laid out, which boundaries are public, or what strictness
is required. Every rule below is a Parallax implementation decision.

## Baseline And Intent

At the baseline commit Parallax has seven Rust workspace crates, only one crate
inherits workspace lints, 31 production files contain inline test bodies, 32
production files declare `#[cfg(test)]`, and one legacy `mod.rs` remains. The
largest handwritten Rust files are
3,540, 2,588, 1,370, 1,310, and 1,216 lines. The UI already has a strong
TypeScript compiler baseline and 41 test files under `__tests__`, but route
files reach 1,500, 990, 871, 841, and 767 lines. Route modules own data
fetching, caching, state, business rules, and rendering together. The ordinary
Vitest path can pass through a Node shebang; the forced-Bun 2026-07-12 probe
loads all 41 files but fails 17 suites at the Zod schema import boundary. A
passing default run is therefore not yet Bun-only baseline evidence.

The goal is not a crate or file count. The goal is compiler-visible ownership:

1. domain and contract crates point downward and never know infrastructure;
2. concrete adapters are composed only at runtime edges;
3. crate and feature roots expose reviewed facades, not filesystem layout;
4. external data remains untrusted until decoded at one named boundary;
5. tests are separate from production bodies and exercise the narrowest valid
   surface;
6. every exception is local, reasoned, expiring, and shrink-only; and
7. local and CI checks call the same repository-owned implementations.

## Rust Workspace Decision

### Final crate graph

The final product graph is intentionally more granular than the current seven
crates, but every crate corresponds to an invariant, dependency boundary, or
independent test surface:

```text
T0  parallax-proto       OTLP wire/service types only
T0  parallax-model       normalized domain rows, IDs, time/value types
T0  parallax-semconv     generated semantic-convention constants (plan 119)

T1  parallax-ingest      decode and normalize ingest-ready values
T1  parallax-analysis    error derivation, fingerprint, trace/span analysis
T1  parallax-storage     capability ports and query-neutral contracts only

T2  parallax-evidence    bundle/story/gap/redaction/bounding/ranking/hash projections
T2  parallax-greptime    GreptimeDB transport, native tables, query and ingest
T2  parallax-metadata    Turso migrations and mutable metadata implementation
T2  parallax-spool       durable raw-frame append, recovery, replay, and limits

T3  parallax-api         GraphQL schema, resolvers, batching, error projection
T4  parallax-server      mandatory engine composition, receivers, workers, serve
T5  parallax-cli         command parsing, API client, output, embedded serve edge

Aux parallax-test-support  builders, fakes, conformance, controlled seeding
Aux parallax-xtask         repository policy and developer orchestration
Aux parallax-mcp          local-stdio MCP product surface (no product deps on it)
```

`parallax-core` is a migration shell only. Its responsibilities move to
`model`, `ingest`, `analysis`, and `evidence`; it is deleted once no supported
consumer needs the compatibility facade. The current `parallax-storage` becomes
the narrow port crate. Arrow, HTTP, GreptimeDB, Turso, spool implementation, and
engine-specific models must not remain reachable through that port crate.

Production normal/build dependencies point strictly from a higher tier to a
lower tier. Same-tier product edges and cycles are forbidden. `parallax-api`
depends on ports/domain logic, never concrete stores. `parallax-server` is the
first crate allowed to compose GreptimeDB and Turso implementations. Product
crates may use `parallax-test-support` through acyclic dev-dependencies only;
normal/build reachability from release roots is forbidden. Workspace metadata
may still describe the package and dev edges; release binaries, SBOMs, and
normal/build feature graphs must not reach it.

### Crate extraction test

A new crate is justified only when at least two of these are true:

- it owns a stable domain invariant or capability contract;
- it removes a concrete external dependency from downstream compilation;
- it needs a distinct feature, target, fuzz, conformance, or release surface;
- it breaks a measured cycle or high-churn compilation boundary;
- it has a narrow facade with multiple real consumers; or
- it isolates a security, persistence, or hot-path ownership boundary.

Do not create crate-per-file wrappers, `common`, `utils`, or `helpers` crates.
All internal crates set `publish = false`; Parallax distributes release
binaries, not an accidental crates.io library family. New members inherit
workspace version, Rust version, edition, license, repository, and lints in the
same commit.

### Crate and module shape

- `lib.rs` and `main.rs` are orientation/composition roots, not business-logic
  containers. They contain crate docs, private `mod` declarations, deliberate
  `pub use` exports, wiring, and small entry points.
- Implementation modules are private by default. Use `pub(crate)` for a proven
  intra-crate consumer and external `pub` only through the reviewed crate
  facade. `unreachable_pub` is required.
- Use Rust 2024 self-named module files: `foo.rs`, children under `foo/`, and no
  `mod.rs`. `clippy::mod_module_files` and a structural ratchet enforce this.
- Names describe capabilities or domain concepts. Generic dumping grounds such
  as `misc`, `common`, `shared`, `utils`, and `helpers` are forbidden unless a
  narrow documented concept makes the name accurate.
- Split by responsibility and change reason, not merely line count. Data types,
  parsing/validation, orchestration, persistence mapping, rendering, and tests
  should not live in one file when they change independently.
- Cross-crate types live at the lowest valid owner. Wire DTOs stay in `proto`,
  domain values in `model`, query-neutral contracts in `storage`, engine rows in
  adapter crates, and transport/output types at API/CLI edges.

### Structural budgets

The ratchet counts logical handwritten lines and AST items, not generated code
or comments. Initial target ceilings are:

| Surface | New or fully restructured target | Existing over-target rule |
|---------|----------------------------------|---------------------------|
| Rust `lib.rs` / `main.rs` | 200 lines | exact baseline, shrink only |
| Other Rust production file | 400 lines | exact baseline, shrink only |
| Rust test scenario file | 600 lines | exact baseline, shrink only |
| Rust function | 100 lines | measured exception, shrink only |
| Clippy cognitive complexity | 25 | measured exception, shrink only |
| Nesting / function arguments | 4 / 6 | reasoned boundary exception |

Generated files require a generator, reproducible drift check, and an explicit
manifest entry. They are excluded from manual size budgets. Repository-owned
generators emit format-clean output; third-party generated output may be
excluded from the manual formatter/linter only through a narrow owned fixture
that proves generator determinism. Compile, dependency, secret, license, and
drift checks still apply. A budget increase is a separate policy change with
owner, evidence, and expiry; implementation changes cannot refresh their own
baseline.

## Rust Test Architecture

Production files may declare only the test module boundary:

```rust
#[cfg(test)]
mod tests;
```

The bodies live in `foo/tests.rs`. When a module has several concerns,
`foo/tests.rs` is a small index and scenario modules live under
`foo/tests/<concern>.rs`. This preserves private-item access without placing
test bodies in production files. Multiple focused scenario files are preferred
to one giant `tests.rs`.

Use the following ownership rule:

| Test kind | Location | Allowed surface |
|-----------|----------|-----------------|
| Private algorithm/unit | external child module beside `src/foo.rs` | private module API |
| Public crate contract | `<crate>/tests/` | reviewed public facade only |
| Shared adapter conformance | `parallax-test-support` or downstream harness | capability facade and controlled seed API |
| Documentation example | rustdoc | public API and real types |
| Property/fuzz/performance | dedicated target owned by plan 103 | named invariant/hot path |
| Real engine | serialized nextest profile/workflow | production adapter facade |

Group integration scenarios under a small number of integration test crates to
avoid recompiling the library for every top-level file. Integration tests may
not force private implementation modules public. Shared fixtures use typed
builders; sleeps, ambient environment mutation, global ports, wall-clock time,
and unordered assertions are forbidden unless the harness owns and proves the
boundary. Non-doctest Rust tests run through nextest; doctests remain a separate
required nextest-discoverable compile-UI partition.

## Rust Strictness Target

### Toolchain and inheritance

- Pin the exact latest stable Rust release, `rustfmt`, `clippy`, and supported
  release targets. Align `rust-toolchain.toml`, mise, CI, workspace
  `rust-version`, and `rust-version.workspace = true` in every package.
- Use edition and style edition 2024. Formatting is repository-wide and
  warning-free.
- One root `[workspace.lints]` table is inherited by every member with
  `[lints] workspace = true`. Cargo does not support adding package-manifest
  lints on top of inherited lints; stricter leaf rules use crate-root inner
  attributes and remain ratcheted.

### Required lint policy

The exact names must be validated against the pinned compiler before landing,
but the target behavior is fixed:

- Rust groups `rust_2024_compatibility`, `future_incompatible`,
  `rust_2018_idioms`, `nonstandard_style`, and `unused` are enabled with explicit
  group priorities. `unsafe_code` is forbidden after ambient test mutation is
  removed. `unreachable_pub`, `unused_must_use`, `unfulfilled_lint_expectations`,
  unsafe-operation, dead/unreachable-code, and missing-debug coverage are
  required.
- Rustdoc denies broken intra-doc links, bare URLs, invalid HTML/code blocks,
  and invalid/private links. Public facades document errors, panics, safety, and
  important examples where applicable.
- Clippy `all` is required and `pedantic` is enabled under CI `-D warnings`.
  `cargo` is selective because duplicate-version findings need dependency
  ownership. Never enable `restriction` or `nursery` wholesale.
- Explicit high-signal restrictions include `dbg_macro`, `todo`,
  `unimplemented`, production `panic`/`unwrap`/`expect`, ignored must-use/future
  results, `await_holding_lock`, `await_holding_refcell_ref`, undocumented or
  multiple unsafe operations, `mem_forget`, wildcard dependencies, stale lint
  expectations, and stdout/stderr macros outside owned CLI/protocol writers.
- Numeric truncation/sign/precision and indexing lints are promoted only after
  semantic review, then cannot regress. Protocol conversions use checked
  conversions or a reasoned, bounded invariant.
- Runtime async code may not perform unowned blocking process/filesystem work.
  Use Tokio APIs or `spawn_blocking`; startup, xtask, and dedicated blocking
  threads require narrow reasoned expectations. `clippy.toml` disallowed-method
  entries and policy fixtures encode the boundary.
- Test valves may allow intentional panic/unwrap/expect/indexing inside test
  code, but never `dbg!`, discarded futures/results, ambient races, or unsafe
  without justification.

The executor starts from this explicit matrix, not from an unspecified
"strict Clippy" preset:

| Layer | Required names/behavior |
|-------|-------------------------|
| Rust deny groups | `rust_2024_compatibility`, `future_incompatible`, `rust_2018_idioms`, `nonstandard_style`, `unused` with priority `-1` |
| Rust explicit | `unused_imports`, `unused_variables`, `unused_must_use`, `dead_code`, `unreachable_pub`, `unsafe_op_in_unsafe_fn`, `unsafe_attr_outside_unsafe`, `missing_unsafe_on_extern`, `never_type_fallback_flowing_into_unsafe`, `unused_qualifications`, unused/redundant/single-use lifetimes, trivial casts, `unnameable_types`, `unit_bindings`, `macro_use_extern_crate`, `meta_variable_misuse`, `let_underscore_drop`, `missing_debug_implementations` |
| Rust forbid | `unsafe_code` after the characterized environment-test boundary is removed |
| Rustdoc | deny `broken_intra_doc_links`; promote private links, redundant explicit links, unescaped backticks, bare URLs, and invalid HTML/code blocks after the initial census |
| Clippy base | `all = deny`; `pedantic` and selected `cargo` at warning level with CI `-D warnings`; explicit group priorities |
| Correctness/async hard denies | `dbg_macro`, `let_underscore_future`, `let_underscore_must_use`, `unused_result_ok`, `assertions_on_result_states`, `await_holding_lock`, `await_holding_refcell_ref`, `todo`, `unimplemented`, `mem_forget`, `disallowed_methods`, `manual_assert` |
| Panic/safety hard denies | production `unwrap_used`, `expect_used`, `panic`; `undocumented_unsafe_blocks`, `multiple_unsafe_ops_per_block`; `exit` outside the binary edge |
| Layout hard deny | `mod_module_files` |
| Numeric hard denies | `float_cmp`, `float_cmp_const`, `lossy_float_literal`, `cast_sign_loss`, `invalid_upcast_comparisons`; promote truncation/wrap/precision after conversion review |
| API/ownership hard denies | `expl_impl_clone_on_copy`, `iter_not_returning_iterator`, `infallible_try_from`, `rc_mutex`; promote `clone_on_ref_ptr`, `rc_buffer`, `result_large_err`, and `large_enum_variant` after the census |
| Maintainability zero-warning set | `manual_let_else`, `match_bool`, `trivially_copy_pass_by_ref`, `str_to_string`, `return_self_not_must_use`, excessive boolean parameters/fields, `too_many_lines`, `cognitive_complexity`, `excessive_nesting` |

`clippy.toml` starts with the 100/25/4/6 thresholds above and these adapted
blocking entries: `std::process::Command::output`, `std::thread::sleep`,
`std::fs::File::open`, and `std::fs::OpenOptions::open`. The restriction means
"not on async runtime/render paths," not "never anywhere": synchronous xtask,
startup, test setup, or a dedicated blocking thread uses an item-local reasoned
expectation until a context-aware policy rule can prove the owner. New runtime
code receives no exception.

Do not copy a residual allow merely because another repository has it. Lints
such as `needless_pass_by_value`, `large_futures`, `implicit_hasher`,
`assigning_clones`, `missing_errors_doc`, `missing_panics_doc`, and numeric casts
are measured against Parallax. New/restructured code must be clean; any legacy
row is exact and shrink-only with a named remediation owner.

Every `allow` or `expect` is item-local where possible and includes a reason.
Counts are syntax-aware and shrink-only by crate and lint. `-D warnings` applies
to default and every supported feature contract, including separately prepared
`embed-ui` and `conformance` lanes.

## TypeScript Strictness Target

### Oxc-first formatting contract

Oxfmt is the sole final JavaScript/TypeScript and supported frontend-text
formatter; rustfmt retains Rust ownership. Plan 094 keeps the current gate
required and freezes its behavior; actionable plan 130 migrates the existing
80-column, no-semicolon, double-quote, LF and ES5 trailing-comma contract with
Tailwind sorting for `src/styles.css`, `cn`, and `cva`. It uses a checked-in
JSON/JSONC config and Bun-owned locked command; executable TypeScript config is
forbidden because it would introduce a Node loading path. Import and
`package.json` sorting begin disabled and are enabled only as isolated,
fixture-backed mechanical changes.

Oxfmt is compiler-version independent and supports TypeScript 7 source syntax;
only type-aware Oxlint is coupled to the TypeScript Go/7 project model. Oxfmt is
still officially Beta, but the operator's 2026-07-12 Oxc-only decision authorizes
its exact narrow use through plan 101's durable pre-stable allowlist.

The migration runs Oxfmt's official Prettier converter, compares exact file
selection and two clean runs, and establishes generated route/shadcn/fixture
ownership before it deletes direct Prettier and
`prettier-plugin-tailwindcss`. Oxfmt's current Beta status conflicts with the
general latest-stable-only policy, so the exception stays exact-pinned,
manually reviewed, fixture-gated, and expires at Oxfmt stable. Cross-platform
idempotence remains mandatory and there is no dual-formatter completion state.

### Compiler contract

The application keeps `strict` and verifies that its effective configuration
enables `noImplicitAny`, `noImplicitThis`, `strictBindCallApply`,
`strictBuiltinIteratorReturn`, `strictFunctionTypes`, `strictNullChecks`,
`strictPropertyInitialization`, and `useUnknownInCatchVariables`. It also
explicitly enables:

```text
noUncheckedIndexedAccess
exactOptionalPropertyTypes
noPropertyAccessFromIndexSignature
noImplicitOverride
noImplicitReturns
noUnusedLocals
noUnusedParameters
noFallthroughCasesInSwitch
noUncheckedSideEffectImports
allowUnreachableCode = false
allowUnusedLabels = false
forceConsistentCasingInFileNames
isolatedModules
moduleDetection = "force"
verbatimModuleSyntax
moduleResolution = "bundler"
```

The repository uses exactly one latest-stable TypeScript 7 compiler CLI/project
line after plan 131; no TypeScript 6 alias, preview, nightly, or second compiler
is retained. The npm package is not represented as shipping `tsserver` in 7.0.
Editor-specific TypeScript 7 LSP integration is optional, follows the current
editor vendor path, and is not a repository-owned or required CI gate.
`tsc --noEmit` remains the canonical independent type-check gate even after
type-aware Oxlint is required.
`tsc --showConfig` is captured and fixture-checked so inherited defaults cannot
silently weaken. `erasableSyntaxOnly` is enabled only after the current TanStack
and generated-code surface passes a compatibility spike. `isolatedDeclarations`
is not enabled for an application that emits no declarations. `skipLibCheck`
may remain temporarily only while a report-only full declaration lane names
specific third-party incompatibilities; plan 128 owns eliminating those using
plan 101's mutually compatible dependency policy, not application casts.

### Rust-style boundary discipline

- `any` is forbidden in handwritten code. External values enter as `unknown`
  and are parsed once with Zod or generated runtime schemas before becoming
  domain values.
- `fetch().json()`, GraphQL envelopes, SSE frames, URL/search parameters,
  storage values, environment values, and cross-window messages are runtime
  boundaries. A generic `as T` is not decoding.
- Static GraphQL operations use named documents, generated variables/results,
  and runtime response schemas. The sole dynamic widget-series exception uses
  Plan 152's bounded structured `DocumentNode`, variables-only values, exact
  alias set, and strict per-series decoder; raw query construction is forbidden.
- Use discriminated unions for async/UI state and exhaustive switches for
  variants. Impossible states should be unrepresentable instead of coordinated
  booleans.
- Pure/domain operations represent expected failure with a discriminated
  Result-shaped union and typed error code, while transport/framework edges may
  throw `Error` and map it once. Do not use exception text as a retry, UI, or
  control-flow classifier.
- Non-null assertions and broad type assertions are forbidden in production
  except a tiny adapter after a validated invariant, with a reason and ratchet.
  Use `satisfies` and `as const` for literal configuration without erasing
  inference.
- Catch values and callback errors are `unknown`; only `Error` objects are
  thrown. Promise-returning calls are awaited, returned, or explicitly handled.
- `@ts-ignore` is forbidden. A necessary `@ts-expect-error` includes a precise
  reason and is exercised as a type test; stale directives fail compilation.

### Oxc-native lint contract

Oxlint is the sole final JavaScript/TypeScript linter. Use a checked-in JSON or
JSONC configuration, never experimental `oxlint.config.ts`, and invoke the
locked binary through Bun. Native and type-aware lanes fail warnings, unused
disable directives, parse/resolution errors, and zero/incomplete handwritten
file selections. Presets are only inventory seeds: the effective rules and
overrides are printed, versioned, and fixture-tested by file class.

The final direct UI tooling set is TypeScript 7, `oxlint`,
`oxlint-tsgolint`, and `oxfmt`. Oxlint's non-nursery native ESLint, TypeScript,
React/Hooks, import, promise, JSX accessibility, Unicorn/Oxc, and Vitest plugins
are configured by built-in names, not installed as `eslint-plugin-*` or
`@typescript-eslint/*` packages. `@oxlint/migrate` is isolated migration input,
not a final dependency. No Parallax command/config invokes ESLint or Prettier.

High-signal typed behavior includes no unsafe assignment/argument/call/member
access/return, no explicit `any`, no floating or misused promises, no
unnecessary condition or assertion, switch exhaustiveness, only-throw-Error,
unknown catch callbacks, consistent type imports, and no non-null assertions.
Use Oxc's stable native rule IDs. When a typed rule is not yet implemented by
Oxlint, encode the invariant in `tsc`, the Oxc-backed xtask provider, or a
runtime/test oracle; never retain ESLint as an unbounded second linter.

The minimum handwritten-code rule inventory is:

```text
typescript/no-explicit-any
typescript/no-unsafe-argument
typescript/no-unsafe-assignment
typescript/no-unsafe-call
typescript/no-unsafe-member-access
typescript/no-unsafe-return
typescript/no-floating-promises
typescript/no-misused-promises
typescript/await-thenable
typescript/return-await
typescript/switch-exhaustiveness-check
typescript/no-unnecessary-condition
typescript/no-unnecessary-type-assertion
typescript/no-unnecessary-type-arguments
typescript/no-unnecessary-type-parameters
typescript/no-non-null-assertion
typescript/only-throw-error
typescript/use-unknown-in-catch-callback-variable
typescript/strict-boolean-expressions
typescript/no-confusing-void-expression
typescript/consistent-type-imports
```

At execution, validate every name against the exact pinned
`oxlint`/`oxlint-tsgolint` pair and record implemented, mapped, or separately
enforced status. Type-aware linting requires a compatible TypeScript Go/TS7
project configuration and is upgraded with Oxlint as one unit because upstream
labels it Alpha and excludes it from normal semver guarantees. Plan 101's second
narrow Oxc exception permits that exact pair with revision, memory, diagnostic,
and negative-fixture proof; it does not permit alpha JS plugins. Experimental
`--type-check` remains report-only; it cannot replace `tsc --noEmit` without a
later stability and diagnostic-parity decision. Baseline repairs may be staged,
but new/restructured handwritten files pass the complete enforceable set and the
report-only count cannot grow.

Plan 131 owns the atomic latest-stable TypeScript 7, native/type-aware Oxlint,
and ESLint deletion wave. The missing 7.0 programmatic compiler API is not an
application prerequisite: current consumers are inventoried and the only hard
conflict, typescript-eslint through TanStack's ESLint config, is removed in that
wave. Plan 128 follows on the final stack and owns declaration cleanup, maximum
compiler strictness, and static boundary invariants. Plan 152 owns the generated
GraphQL/decoded-transport foundation; Plan 153 owns SSE/search/storage/
environment/cross-window runtime foundations. Product feature plans instantiate
those mechanisms for their own contracts.

Native stable `react/rules-of-hooks` and `react/exhaustive-deps` are required.
Experimental `react/react-compiler` and compiler-derived purity, immutability,
refs, static-component, and state-update diagnostics are disabled in the live
graph. Isolated non-gating evaluation does not permit promotion: each rule also
requires upstream stable/non-nursery status, a separate operator-approved plan
and durable policy change, and deterministic fixtures. Alpha JavaScript plugins
are never installed, configured, or executed in the live/final graph. Generated
route code and shadcn primitives have
narrow path overrides; every override names generator ownership and does not
suppress security, boundary, promise, or import-direction rules unnecessarily.

## UI And TanStack Architecture

### Final layout and imports

```text
ui/src/
  app/                    composition root: router, QueryClient, providers
    tests/                public composition/router contracts
  layout/                 shell, navigation, application boundaries
    tests/                shell/navigation/theme/boundary contracts
  features/<feature>/
    api/                  documents, schemas, decoded adapters
    model/                domain types, state machines, pure transforms
    queries/              queryOptions, mutations, cache keys
    components/           feature UI
    hooks/                feature orchestration only
    tests/                separated feature-owned tests
    index.ts              reviewed public facade
  domain/<concept>/       framework-neutral cross-feature product concepts
    tests/                pure domain contracts
  platform/<adapter>/     GraphQL, SSE, storage/clock/runtime adapters
    tests/                technical boundary contracts
  shared/
    components/           product-neutral components
    hooks/                product-neutral hooks
    lib/                  cohesive named utilities only
    tests/                product-neutral component/hook/lib contracts
  routes/                 route declaration, search, loader, boundary, compose
    tests/                route contracts only
  test/                   deterministic Vitest setup/builders; no test bodies
  components/ui/          shadcn CLI-owned primitive island
  lib/utils.ts            shadcn CLI-owned `cn` island
ui/tests/harness/         test-infrastructure self-tests
ui/tests/e2e/             Playwright black-box fixtures/screens/specs
ui/test-matrix.json       durable risk-to-evidence manifest
```

The closed graph is: `app` composes all layers; routes import feature facades,
domain, and shared, with root-only access to the reviewed layout entry; layout
imports feature facades, domain, and shared; features import their own internals,
domain, platform, shared, and only explicitly approved other-feature facades;
platform imports domain/shared; domain imports only product-neutral pure shared
utilities; shared imports no Parallax upper layer. No route, layout, feature,
domain, platform, or shared module imports `app`. Shared is not a default bucket:
promotion requires multiple independent consumers and product-neutral naming.
Generated `routeTree.gen.ts` is a composition exception, not a precedent.
Aliases, type-only imports, dynamic imports, barrels, source test directories,
and browser test imports are included in cycle/direction analysis.

Feature `index.ts` files use explicit named type/value exports; handwritten
`export *` barrels are forbidden because they hide public-surface growth and can
pull server/heavy modules into client graphs. A public entry exposes only stable
feature contracts/components used outside the feature.

Route files export only the TanStack `Route` contract required by file routing.
Testable components, loaders, and transforms live in feature modules. Do not
export route implementation properties that prevent automatic code splitting.
Use `getRouteApi` in deep feature code rather than importing route definitions
and creating cycles. Route loaders return `void` or minimal identifiers when
the query cache owns data, keeping inferred route types and bundles bounded.

Keep `ui/` as one Bun package and one canonical strict TypeScript project.
Internal npm packages or TypeScript project references require measured
typecheck/editor evidence and a separate migration contract; file-count analogy
to Rust crates is not sufficient. Prefer pure named modules and readonly values.
A class is justified only by a real lifecycle or invariant-bearing mutable
identity, not by class-per-file ceremony. New catch-all `utils.ts`, `types.ts`,
`helpers.ts`, and `common.ts` modules are forbidden.

### Data, cache, and execution decision

TanStack Query becomes the sole server-state cache. Each router/server request
receives a fresh QueryClient; the browser keeps one stable client for that
router lifetime. Module singletons are forbidden, and two-router isolation plus
hydrate/dehydrate tests prove no request cache leaks. Each feature owns stable
query keys and `queryOptions`; loaders use `ensureQueryData`, loader-backed
components may use `useSuspenseQuery`, and on-demand/optional work uses
`useQuery` or `fetchQuery`. Mutations invalidate or update exact keys, and live
SSE updates reconcile through QueryClient. Router preload stale time is `0` so
the Query cache owns freshness. The current ad hoc GraphQL TTL cache is deleted
after behavior parity; dual caches are forbidden. SPA mode remains authoritative:
only the root shell is prerendered unless a separate deployment-contract change
explicitly enables route SSR.

Search state uses typed `validateSearch` schemas and `loaderDeps` contains every
search value that changes loaded data. Links/navigation use typed route APIs.
Server-only code lives behind TanStack Start server functions or `.server`
modules with input validation. Client-only globals remain in `.client` or
effect-owned code. CI inspects production chunks so secrets, filesystem/process
APIs, and server modules cannot enter client bundles. Same-origin GraphQL and
the two typed SSE feeds are the only browser data paths unless the product
contract is changed first.

### UI structural budgets

| Surface | New or fully restructured target | Existing over-target rule |
|---------|----------------------------------|---------------------------|
| Route module | 150 logical lines | exact baseline, shrink only |
| Handwritten TS/TSX module | 300 logical lines | exact baseline, shrink only |
| UI test scenario file | 500 logical lines | exact baseline, shrink only |
| Function/component/hook | 60 logical lines | AST baseline, shrink only |
| Cyclomatic/cognitive complexity | 12 / 15 | measured exception, shrink only |

Moving an oversized component unchanged from a route to a feature does not
satisfy the ratchet. Functions/components and import surfaces are measured in
addition to files.

## Frontend Test Architecture

Vitest remains the test runner and is invoked through Bun. Do not mix `bun:test`
and Vitest APIs. Testing Library tests observable user behavior with semantic
role/name queries and `userEvent.setup()`; `fireEvent` is reserved for events a
user-event abstraction does not model, such as low-level SSE, pointer, resize,
or virtualization mechanics.

Tests stay outside production bodies and mirror ownership:

- `model` tests cover pure transforms, state machines, exhaustive variants, and
  schema round trips;
- `api` tests cover valid/malformed GraphQL and SSE envelopes, cancellation,
  retry classification, and error projection;
- route tests cover search validation, loader dependencies, SSR/client
  navigation, and error/pending boundaries without importing private route
  implementation;
- component tests cover user-visible behavior and accessibility;
- Playwright browser tests cover every shipped screen plus critical cross-route
  flows in distinct deterministic contract and real-stack projects;
  and
- type tests prove public facade and generated contract expectations.

`ui/src/test/` owns deterministic render/router/QueryClient builders, endpoint
fixtures, fixed clock/timezone, matchMedia/ResizeObserver/scroll polyfills,
theme and reduced-motion defaults, and cleanup. Unexpected console errors,
unhandled rejections, network calls, and no-test selections fail. `tests/` is
the one source-owned test directory convention; `src/test/` contains no test
bodies; `ui/tests/harness/` owns tests of that infrastructure itself.
`ui/test-matrix.json` maps stable risk IDs to unit/component/route/browser
evidence and is machine validated.

The required browser entry is exact lock-local `@playwright/test` invoked by
`bun run test:browser`, with Bun forced and installation disabled. Playwright
is the sole browser test framework; Rust xtask may start/seed/stop Parallax and
the engines but never implements a second browser driver, locator, assertion,
or reporter stack. Direct `playwright`, direct `playwright-core`, Node runtime,
Playwright component testing, ESLint Playwright plugins, and alpha Oxlint
JavaScript plugins are forbidden.

Playwright adoption is conditional on the complete exact-version macOS/Linux
Bun matrix in plan 132. A hang, hidden compatibility flag, Node child,
unsupported browser, or leaked process blocks adoption; Node and custom CDP are
not fallbacks.

Browser projects separate deterministic test-support contracts from a managed
GreptimeDB + isolated Turso full-stack lane. Chromium/Firefox/WebKit and real
Playwright mobile device descriptors are explicit. Semantic role/name/label
locators and web-first assertions are required; CSS/XPath, fixed sleeps,
order-dependent tests, and response interception for happy paths are forbidden.
Runtime axe plus keyboard/focus tests supplement static JSX accessibility.
Canonical visual comparison runs in one digest-pinned Linux/browser/font
environment with explicit update and shrink-only threshold policy.

Coverage is risk based: critical boundary/state modules and touched hotspots
receive branch evidence. A global percentage is not a quality proxy. If line
coverage is adopted under Bun, use a compatible provider verified by a spike;
do not assume Vitest V8 coverage runs under Bun. Large DOM snapshots, snapshot-
only behavior tests, real sleeps, shared mutable fixtures, and blanket test
retries are forbidden.

## Dependency, CI, And Exception Contract

- Bun is the only JS runtime/package manager. `bun ci` consumes the only
  lockfile. `ui/bunfig.toml` sets `[run] bun = true` so Node shebangs/`node`
  recurse through Bun and `[install] auto = "disable"` so scripts cannot fetch
  missing packages. Every package script invokes an exact lock-local CLI; Bunx
  uses `--bun --no-install`, never `@latest`. Required Vite, Vitest, TypeScript,
  lint, format, codegen, and shadcn process-ancestry fixtures reject Node or an
  undeclared installer. `trustedDependencies` is explicitly empty first so
  Bun's built-in top-package trust list is not active; a package is added only
  after exact locked-version, integrity, script, reason, owner, and expiry review.
- Required dependency policy covers both graphs: Rust advisories/licenses/
  sources/features/unused dependencies and Bun advisories/licenses/integrity/
  lifecycle trust/unused direct dependencies. Outdated reports never create
  branches.
- CI path routing includes policy, generated inputs, manifests/locks, shared
  actions, server/client boundaries, and supported features. One skipped-aware
  aggregate remains the required check.
- Preserve the established layered cache model: rustup/toolchain, Cargo registry
  and git data, sccache, per-job/per-target build output with main-branch restore
  fallback, mise-installed Cargo tools, and Bun's package cache. Keys include
  toolchain, lockfile, target, profile, feature partition, and relevant config;
  cache misses never skip a gate. Cold/warm timing and sccache statistics must
  justify consolidation or key changes.
- Browser installation is explicit after an ignore-scripts Bun install.
  Playwright's CI guidance says browser binary cache restore is often comparable
  to download and system dependencies are not cacheable, so no browser cache is
  added without cold-versus-restore measurement. Any adopted cache includes the
  exact Playwright/browser version, OS, architecture, and manifest in its key.
- Rust default, supported feature, doctest, nextest, real-engine, and release
  lanes are distinct. UI formatting, typed lint, typecheck, forced-Bun Vitest,
  production build, generated drift, deterministic Playwright contract/visual,
  cross-browser/mobile, and real-stack browser lanes are distinct but
  aggregated according to their required cadence.
- Nextest retry-pass is failure in CI. Quarantined tests keep running in a
  visible selection and require owner, reason, expiry, and shrink-only state.
- Exceptions use a common schema: rule, exact scope, evidence, owner, created
  date, expiry, removal condition, and replacement. Missing, expired, broadened,
  or stale exceptions fail closed. Generated exclusions use the same rigor.

Release delivery keeps one repository-owned Rust helper/xtask byte-producing
path for local
rehearsal, preview, and stable archives. Version/tag/binary/archive identity is
validated before build; Zig cross-build inputs are digest verified; native TLS
uses the approved vendored OpenSSL path only on cross targets; line tables or
symbol companions preserve source attribution; and checksums, SBOMs, cosign
signatures, provenance attestations, and the rolling preview formula all refer
to the same finalized bytes. Actions remain full-SHA pinned and release jobs use
least privilege plus protected operator approval.

## Plan Ownership

| Decision area | Owning plan |
|---------------|-------------|
| Contract and behavior/risk baselines | 093 |
| CI routing, interim formatter enforcement, required aggregate | 094 |
| Xtask, Oxc AST/resolver graph, ratchets, diagnostics | 095 |
| Rust toolchain, format, lint, async blocking rules | 096 |
| Rust test file topology | 127 |
| Model, ports, test support, initial inversion removal | 097 |
| Final Rust crate decomposition | 126 |
| Facades and remaining module splits | 098 |
| TypeScript 7, stable native plus operator-excepted type-aware Oxlint, ESLint removal | 131 |
| TypeScript declaration strictness and static safety | 128 |
| Operator-excepted Oxfmt cutover and Prettier removal | 130 |
| Forced-Bun Vitest topology, harness, characterization matrix | 094, 129 |
| Playwright Bun compatibility, config, fixtures, and foundation smoke | 132 |
| Fixture-backed Playwright product contracts and required CI | 144 |
| Managed GreptimeDB + Turso Playwright integration | 145 |
| Cross-browser, mobile, accessibility, and visual Playwright gates | 146 |
| TypeScript layers, lower-level adapters, facades, and placement policy | 100 |
| Generated GraphQL SDL/operations/runtime transport | 152 |
| SSE, search, storage, environment, and cross-window runtime foundations | 153 |
| Route-less cross-feature capability facades | 149 |
| Independent TypeScript feature moves, including overview | 134-142, 150 |
| App/root/layout/shell migration | 143 |
| Final UI ownership/ratchet/handoff closure | 151 |
| TanStack Query ownership and legacy TTL-cache removal | 133 |
| Typed bounded live-data behavior and performance | 147 |
| Route chunks, bundle budgets, minification, and source maps | 148 |
| Cargo and Bun dependency/test evidence | 101 |
| Release profiles, symbols, deterministic artifacts | 102 |
| Property, fuzz, and performance evidence | 103 |

This standards file is active plan material. Plan 107 deletes it after all
currently executable rules, including plans 130 and 131, are durable.
