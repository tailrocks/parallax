# Plan 128: Enforce strict TypeScript static safety

> **Executor instructions**: Finish the compile-time contract on top of Plan
> 131's single TypeScript 7 and Oxlint toolchain. Add one compiler option or
> static invariant at a time, keep selected-file and effective-config evidence,
> and repair findings by narrowing or modeling. Never replace an error with
> `any`, `as`, `!`, a suppression, or a local declaration patch. This plan owns
> static safety only. Plan 152 owns GraphQL runtime contracts; Plan 153 owns all
> other external-value runtime contracts.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/tsconfig.json ui/package.json ui/bun.lock ui/.oxlintrc.jsonc ui/src ui/tests crates/parallax-xtask ratchet.toml plans/ENGINEERING-STANDARDS.md`
> Reconcile compiler, declaration, file-selection, and escape-hatch changes
> before editing. Do not absorb runtime decoding from Plans 152 or 153.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MEDIUM
- **Depends on**: 095, 101, 131
- **Category**: TypeScript / compiler / static safety
- **Planned at**: `e3e7997`, revised 2026-07-12
- **Status**: TODO

## Why This Matters

Parallax already enables most strict compiler options and Plan 131 establishes
TypeScript 7 plus native and type-aware Oxlint. The remaining static contract is
incomplete: index signatures still permit property access, module isolation is
implicit, third-party declarations are skipped, and handwritten code has no
single enforced policy for assertions, directives, impossible async states, or
stringly expected failures.

Runtime validation is a separate ownership problem. Mixing GraphQL, SSE,
storage, search, environment, and message decoding into this plan makes the
compiler migration too large and prevents the boundary foundations from running
independently. This plan therefore establishes the static rules that those
runtime plans must satisfy, but does not implement a decoder or move a product
feature.

## Current Evidence

- `ui/tsconfig.json` omits `noPropertyAccessFromIndexSignature`,
  `isolatedModules`, and forced module detection.
- `skipLibCheck` remains enabled. A full declaration probe exposes third-party
  incompatibilities that must be solved as one mutually compatible dependency
  set rather than hidden with ambient declarations.
- Handwritten UI code has no explicit `any`; the visible `as any` cases are in
  generated route-tree output. File-class ownership must preserve that
  distinction.
- Existing production code still contains broad assertions, non-null
  assertions, coordinated boolean states, and exception-text classification
  that are not covered by one required static policy.
- Plan 131 inventories and enables the final native/type-aware Oxlint rules.
  This plan consumes that inventory and closes only the remaining application
  policy gaps; it does not introduce another linter or compiler.

## Fixed Decisions

1. `tsc --noEmit` remains the authoritative compiler gate. Experimental Oxc
   type-checking cannot replace it.
2. Native and type-aware Oxlint remain the only linters. Parallax-specific
   syntax and graph invariants use Plan 095's Rust/Oxc xtask provider.
3. Effective configuration and selected files are checked artifacts. A green
   command with a weakened config or incomplete file set fails.
4. `any`, production `@ts-ignore`, unreasoned `@ts-expect-error`, non-null
   assertions, and broad assertions are forbidden in handwritten production
   code except an exact no-growth runtime-boundary handoff to Plan 152 or 153.
   Generated and shadcn files have explicit, narrow owners.
5. Async/UI state uses discriminated unions when independent booleans can
   represent an impossible combination. Expected domain failures use exhaustive
   Result-shaped unions with typed codes.
6. `skipLibCheck` is disabled only by fixing the compatible dependency graph.
   Ambient patches, `any`, and application casts are not declaration fixes.
7. External values remain `unknown` until their owning runtime plan parses
   them. GraphQL belongs to Plan 152; SSE, route search, storage, environment,
   cross-window messages, and other external JSON belong to Plan 153. Plans 152
   and 153 depend on this plan; this plan does not wait for either one.

## Scope

In scope:

- The complete TypeScript 7 compiler-option contract and effective-config
  drift check.
- Full third-party declaration compatibility with `skipLibCheck = false`.
- Static native/type-aware Oxlint rules and Oxc-backed xtask policies not
  completed by Plan 131.
- Assertion, non-null, directive, suppression, generated-owner, and file-class
  ratchets.
- Exhaustive discriminated state and Result-shaped expected-failure policy.
- Required local and CI diagnostics for this static contract.

Out of scope:

- GraphQL SDL, documents, code generation, transport, envelope parsing, and
  dynamic dashboard response validation, owned by Plan 152.
- SSE, search, storage, environment, cross-window, and other non-GraphQL runtime
  boundary implementations, owned by Plan 153.
- Feature/route movement, owned by Plans 100, 134-143, 149, and 150; cache
  behavior, owned by Plan 133.
- Test topology and browser automation, owned by Plans 129 and 132/144-146.
- General dependency upgrades except the narrow mutually compatible changes
  required to make third-party TypeScript declarations clean under Plan 101.
- Application-wide branded types or cosmetic type rewrites.

## Required Commands

| Purpose | Command | Expected result |
|---------|---------|-----------------|
| Effective compiler config | `cd ui && bunx --bun --no-install tsc --showConfig` | exact strict options and complete file classes |
| Compiler | `cd ui && bun run typecheck` | TypeScript 7 exits zero with full declaration checking |
| Native lint | `cd ui && bun run lint:native` | warnings, stale directives, parse errors, and zero selection fail |
| Type-aware lint | `cd ui && bun run lint:type-aware` | typed rule fixtures and full project pass |
| Static policy | `cargo xtask policy --only ui.typescript-static-safety` | ratchets, ownership, state, and Result rules pass |
| UI regression | `cd ui && bun run test:ci && bun run build` | existing behavior remains green |
| Aggregate | `cargo xtask ci --fast` | required static diagnostics are represented once |

Use the final script names established by Plan 131 if they differ. Update this
table and the stable xtask diagnostic IDs in the same implementation change;
never retain aliases that invoke a second toolchain.

## Steps

### Step 0: Verify the compiler and linter prerequisite

Confirm Plan 131 left exactly one latest-stable TypeScript 7 compiler, one
native Oxlint lane, one type-aware Oxlint lane, independent `tsc --noEmit`, and
no invoked ESLint/typescript-eslint or TypeScript 6 alias. Reproduce its
effective-config, selected-file, Bun-process, dependency, and API-consumer
negative gates before changing strictness.

**Verify:** compiler/linter versions match the lock, every command is forced
through Bun with installation disabled, process ancestry contains no Node, and
all Plan 131 compatibility fixtures pass.

### Step 1: Freeze the static baseline by file class

Capture the effective compiler config, compiler file list, native/type-aware
Oxlint effective rules and file lists, full declaration diagnostics, and the
current assertion/non-null/directive/state/Result inventories. Classify every
finding as handwritten production, test, config, generated route tree, shadcn,
or third-party declaration ownership.

Store exact positive and negative file manifests in the quality-control-plane
fixtures. The baseline can shrink but cannot grow, omit a class, or silently
move a handwritten file into an excluded class.

**Verify:** missing, duplicate, stale, zero-file, parse-failure, and deliberately
misclassified fixtures fail with the same rule IDs in human and JSON output.

### Step 2: Complete the compiler contract

Add `noPropertyAccessFromIndexSignature`, `isolatedModules`, and
`moduleDetection: "force"`. Preserve `strict` and explicitly verify all strict
sub-options plus unchecked-index, exact-optional, override, return, unused,
fallthrough, side-effect-import, casing, unreachable-code, unused-label,
verbatim-module, and bundler-resolution settings required by
`ENGINEERING-STANDARDS.md`.

Repair application diagnostics with precise property access, narrowing,
readonly values, and exact optional-property construction. Spike
`erasableSyntaxOnly` over handwritten, generated, test, and config classes;
enable it only when the complete application surface passes. Do not enable
`isolatedDeclarations` for this no-emit application.

**Verify:** `tsc --showConfig` matches the checked strict contract, one fixture
fails for every newly required option, the full compiler file manifest is
non-empty and exact, and `tsc --noEmit` passes without casts or exclusions.

### Step 3: Make the full declaration graph clean

Start from Plan 101's report-only `skipLibCheck = false` inventory. For each
failure, record package owner, declaration path, diagnostic, current compatible
version set, and upstream fixed version if one exists. Select the latest
mutually compatible stable TypeScript 7, TanStack, React, Vite, Vitest, and
declaration set under Plan 101's dependency policy.

Upgrade only packages necessary for declaration compatibility, run their owned
behavior suites, then set `skipLibCheck` to `false` and make the full lane
required. Do not add local ambient modules, declaration merging that lies about
the package, `patch-package`, `any`, or application casts.

**Verify:** a clean checkout checks every library declaration; a fixture that
re-enables `skipLibCheck`, excludes the failing declaration, or adds an ambient
patch fails policy. `bun why`, the exact lock, UI tests, build, and TanStack route
generation remain green.

### Step 4: Close static Oxlint and xtask gaps

Consume Plan 131's implemented/mapped/missing inventory. Require every
available stable native/type-aware rule for unsafe values, promises,
exhaustiveness, unnecessary conditions/assertions/type arguments, non-null,
throw/catch, strict booleans, confusing void, and consistent type imports.
Never duplicate a rule already owned by Plan 131 under a second name.

Use Plan 095's Rust/Oxc provider only for invariants the pinned Oxlint pair
cannot express: handwritten/generated owner separation, forbidden escape-hatch
forms, reason-bearing type-test directives, Result-union shape, stringly error
classification, and coordinated-boolean state candidates. Parse syntax and
semantic imports; do not enforce these with regex, alpha JavaScript plugins,
ESLint, or experimental Oxc type-check authority.

**Verify:** each rule or custom invariant has one positive and one minimal
negative fixture in every applicable file class. Missing rules are mapped to an
exact compiler/xtask oracle; warnings, unused disables, parse/resolution errors,
scope broadening, and zero selection fail.

### Step 5: Remove escape hatches and impossible states

Replace handwritten production non-null assertions and broad assertions with
narrowing, `satisfies`, literal-preserving `as const`, or explicit impossible-
state handling. Do not implement a runtime decoder here. If an assertion exists
only because an external GraphQL value is still unparsed, record its exact
path/symbol/test IDs and expiry under Plan 152; assign every other external-
value assertion to Plan 153. Those handoffs are exact and no-growth. Forbid
`@ts-ignore`. Permit `@ts-expect-error` only in a type test with a precise reason
and an adjacent assertion proving the intended diagnostic; stale directives
must fail compilation.

Inventory components/hooks that coordinate two or more booleans for one async
or UI lifecycle. Convert only genuine state-machine groups to readonly
discriminated unions with exhaustive switches and a `never` oracle. Do not
combine unrelated orthogonal booleans merely to satisfy a metric.

Inventory pure/domain functions whose expected failures are classified by
thrown text or untyped strings. Replace them with a discriminated Result-shaped
union carrying a stable typed code. Framework and transport edges may throw an
`Error`, but map it once and never branch on its message.

**Verify:** type fixtures reject impossible state combinations, unhandled
variants, stringly error codes, message-based control flow, broad assertions,
production non-null, and unreasoned directives. Existing behavior tests remain
unchanged and green.

### Step 6: Lock generated and shadcn ownership

Keep TanStack's route tree and shadcn primitives in explicit generated/tool-owned
classes. Generated code is compiler-checked but is never hand-edited to satisfy
an application policy. Exceptions name generator, path glob, allowed rules,
reason, owner, refresh command, and stale/removal condition. A handwritten file
cannot match either class.

Generated files introduced later by Plan 152 follow the same contract, but
their generation and runtime semantics remain Plan 152's responsibility.

**Verify:** handwritten-in-generated-path, generated-outside-owner,
scope-broadening, stale exception, and direct generated edit fixtures fail.
Regeneration produces the same compiler/lint classification.

### Step 7: Integrate required static gates

Expose separate stable diagnostics for effective compiler config, compiler file
selection, full declarations, native lint, type-aware lint, escape-hatch/state/
Result policy, and generated ownership. Add them once under the aggregate
required check with exact tool/config/project-graph/platform cache inputs.

Keep failures attributable: a declaration failure cannot appear as lint, and a
runtime boundary failure from Plans 152/153 cannot be hidden in this static
gate. A skipped, cached-with-wrong-inputs, zero-file, Node-spawned, or implicit-
install invocation fails closed.

**Verify:** local and CI command manifests agree; intentional failures route to
the exact diagnostic; two clean runs are deterministic; typecheck, lint, tests,
build, and `cargo xtask ci --fast` pass.

## Test Plan

- Effective TypeScript config and selected-file goldens, including weakening,
  omission, duplicate class, parse failure, and zero-file fixtures.
- One compiler-error fixture for each newly required compiler option.
- `skipLibCheck = false` declaration graph plus skip, exclusion, ambient-patch,
  and incompatible-dependency negative fixtures.
- One positive/negative fixture for every selected native/type-aware Oxlint rule
  and every Oxc-backed custom static invariant.
- Assertion, non-null, directive, suppression, generated-owner, and stale-
  exception ratchet cases.
- Exhaustive discriminated-state and Result-shaped failure type fixtures,
  including impossible combinations and exception-message classification.
- Bun-only typecheck/lint/test/build and process-ancestry checks from a clean
  checkout on supported macOS and Linux CI environments.

## Done Criteria

- [ ] Effective TypeScript 7 config contains every compatible strict option and
  cannot drift weaker or omit a file class.
- [ ] `skipLibCheck` is false and the complete third-party declaration graph is
  clean without ambient patches, exclusions, or application casts.
- [ ] Native/type-aware Oxlint and Oxc-backed xtask rules cover every selected
  static invariant with exact positive and negative fixtures.
- [ ] Handwritten production code contains no `any`, `@ts-ignore`, unreasoned
  `@ts-expect-error`, non-null assertion, or broad assertion outside exact
  no-growth Plan-152/153 runtime-boundary handoffs.
- [ ] Genuine async/UI state machines and expected domain failures are
  exhaustive discriminated unions; exception text is not control flow.
- [ ] Generated and shadcn classes are narrow, owned, refreshable, and cannot
  capture handwritten files.
- [ ] Static gates are deterministic, Bun-only, non-empty, separately
  attributable, and required under the aggregate check.
- [ ] GraphQL and other runtime parsing remain assigned only to Plans 152 and
  153; this plan has not created a boundary implementation.

## STOP Conditions

Stop and report if:

- a compiler or rule change requires broad disablement, generated-code edits,
  a handwritten exclusion, `any`, a broad assertion, or a suppression;
- no latest mutually compatible stable dependency set has a clean declaration
  graph;
- a proposed state union combines unrelated state or changes behavior rather
  than removing an impossible combination;
- native/type-aware Oxlint skips a file class, needs Node, or has no exact
  compiler/xtask substitute for a required missing rule;
- generated/shadcn ownership cannot distinguish tool output from handwritten
  code; or
- a remaining assertion cannot be assigned exactly to static repair, Plan 152,
  or Plan 153 without changing runtime behavior; or
- implementation starts decoding GraphQL or any Plan 153 external boundary.

## Remove When

Delete this plan and its index row when the complete strict TypeScript 7
compiler contract, full declaration graph, static Oxlint/xtask rules,
escape-hatch/state/Result policy, generated ownership, and required Bun-only
gates are green. Runtime boundary plans do not block this plan's retirement.
