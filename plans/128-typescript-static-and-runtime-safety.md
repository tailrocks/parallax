# Plan 128: Enforce TypeScript compile-time and runtime boundary safety

> **Executor instructions**: Preserve the already-strong compiler baseline and
> add one strict control at a time with effective-config evidence. Never replace
> an error with `as`, `!`, `any`, or an unvalidated generic. Generated and
> shadcn exclusions must be narrow and tested. Plan 131 has already established
> TypeScript 7 plus native/type-aware Oxlint; do not reopen compiler or linter
> selection here.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MEDIUM
- **Depends on**: 095, 101, 131
- **Category**: TypeScript / static safety / runtime contracts
- **Planned at**: `a1d8bf82`, 2026-07-12
- **Status**: TODO

## Why

Parallax already enables `strict`, unchecked-index protection, exact optional
properties, unused/return/fallthrough checks, and side-effect import checking.
The remaining gap is not a switch called "strictest": index signatures still
allow property syntax, current lint lacks key promise/unsafe/React signal, and
GraphQL/SSE/JSON data becomes trusted through casts instead of decoding.
Compiler strictness cannot prove network data. Plan 131 supplies the Oxc-first
TypeScript 7/native/type-aware lint foundation; this plan uses it to eliminate
unchecked application boundaries and finish the strict compiler contract.

## Current Evidence

- `tsconfig.json` omits `noPropertyAccessFromIndexSignature`,
  `isolatedModules`, and forced module detection while retaining
  `skipLibCheck`; a read-only full declaration probe exposes third-party
  incompatibilities, so flipping both application and library checks together
  would obscure ownership.
- The pre-plan-131 effective ESLint config lacks floating/misused-promise,
  unsafe-value, non-null, React Hooks, and compiler-diagnostic enforcement. A
  read-only check-only typed-rule probe found 22 hidden findings, 13 in
  production; plan 131 converts that inventory into the final Oxc rule baseline.
- Handwritten UI code has no explicit `any`; the visible `as any` instances are
  generated route-tree code. Preserve that strength while eliminating generic
  JSON and SSE assertions.
- `ui/src/lib/api.ts` trusts `response.json()` as generic `T`; dynamic dashboard
  aliases and logs/traces/run SSE feeds are not fully runtime decoded.
- Official Oxc documentation currently describes type-aware linting as outside
  normal semver guarantees, coupled to TypeScript Go/TS7, and implementing
  59 of 61 typescript-eslint typed rules. Oxc type-check is experimental and
  JavaScript plugins are alpha, so neither can silently replace a proven gate.

## Scope

- Complete compiler options and effective-config drift evidence.
- Boundary-specific Oxlint/xtask rules and effective-config drift evidence on
  plan 131's final native/type-aware configuration.
- `unknown`-first runtime decoding for GraphQL, SSE, search, and storage edges.
- Assertion/non-null/directive/suppression ratchets and exhaustive domain state.
- Generated/shadcn ownership and declaration-compatibility evidence.

Out of scope:

- Feature/route/cache movement, owned by plan 100.
- Test harness/topology, owned by plan 129.
- General dependency policy/upgrades, owned by plan 101. This plan may make the
  narrow compatible upgrade needed to clear a known TypeScript declaration
  failure, using plan 101's already-landed policy.
- Cosmetic type rewrites or application-wide branded-type migration.

## Steps

### Step 0: Verify the compiler/linter prerequisite

Verify plan 131 left exactly one latest-stable TypeScript 7 compiler, native and
type-aware Oxlint as the only invoked linters, independent `tsc --noEmit`, and no
ESLint/typescript-eslint or TypeScript 6 alias. Reproduce its effective config,
selected-file, Bun-process, and dependency/API-consumer gates before proceeding.

### Step 1: Freeze the compiler, lint, and boundary baseline

Capture `tsc --showConfig`, Oxlint's printed effective config for handwritten,
test, generated-route, config, and shadcn files, current diagnostics, assertion/
non-null/directive counts, and runtime boundary inventory. Classify every
violation by production, test, generated, or third-party declaration ownership.
Freeze exact selected-file manifests and representative positive/negative files
so the migration cannot appear green by omitting a class.

### Step 2: Complete the compiler contract

Add `noPropertyAccessFromIndexSignature`, `isolatedModules`, and
`moduleDetection: "force"`; preserve every strict option named in
`ENGINEERING-STANDARDS.md`. Repair application findings using precise access
and narrowing. Spike `erasableSyntaxOnly` and enable it only if handwritten and
generated application code is compatible.

Start from plan 101's report-only `skipLibCheck=false` inventory. Resolve each
third-party failure by selecting the latest mutually compatible stable
TypeScript 7/TanStack/declaration set under that dependency policy, then make
the lane required and disable `skipLibCheck`. This plan owns only upgrades necessary
for declaration compatibility. Never paper over dependency declarations with
local `any` or an ambient declaration patch.

### Step 3: Extend the final static boundary contract

Consume plan 131's exact native/type-aware Oxlint inventory. Add or confirm the
boundary-specific rules for unsafe values, promises, exhaustive variants,
assertions, throw/catch, non-null, type imports, unused directives, and test-only
expectations. Each added rule has a negative fixture and exact file-class scope.

Use plan 095's Oxc-backed xtask provider for Parallax-specific escape-hatch,
Result-union, generated-owner, and boundary-decode ratchets that generic lint
cannot express. Do not add JS plugins, ESLint, a second import graph, or
experimental Oxc type-check authority.

### Step 4: Decode every runtime boundary

Make `build_schema().as_sdl()` from `parallax-api` the authoritative GraphQL SDL
source. A repository-owned command exports a deterministic checked-in schema and
fails drift. Static named `.graphql` documents are validated against that schema
and one Bun-compatible generator emits variables/results plus runtime schemas;
generated outputs are checked in or reproduced deterministically under Bun.
Transport always sends `{ operationName, query, variables }`; remove value
interpolation/manual escaping. The generic client accepts `unknown`, decodes the
GraphQL envelope and operation payload once, and returns the generated domain
contract. Inventory dynamic dashboard aliases and choose a bounded typed query
representation or an explicit decoder that validates alias keys and values.

Land these contracts behind the current narrow API/SSE boundary without moving
route ownership in the same change. Plan 100 relocates the already-tested
documents/schemas into `shared/api` and feature `api/` modules mechanically.

Define schemas for each logs/traces/run SSE event and validate before state
mutation. Reject malformed frames deterministically, record bounded diagnostics
without payload secrets, and test reconnect/order behavior. Apply the same rule
to route search, local/session storage, environment, and cross-window values.

### Step 5: Model states and remove escape hatches

Convert touched multi-boolean async states to discriminated unions and exhaust
their variants. Replace production non-null assertions and broad assertions
with narrowing, schema parsing, or a tiny proven adapter. Forbid `@ts-ignore`;
reason-bearing `@ts-expect-error` exists only in type tests. Ratchet all escape
hatches by handwritten/generated/test ownership.

Pure/domain operations represent expected failures as a discriminated
Result-shaped union with a typed code. Transport/framework boundaries may throw
an `Error`, but map it once; exception text cannot classify retry, UI, or control
flow. Inventory touched domain functions and add Oxc/xtask/type fixtures that
reject stringly failure variants and unhandled result cases.

### Step 6: Integrate required gates

Make the current required formatter, native Oxlint, type-aware Oxlint,
effective-config drift, `tsc --noEmit`, schema/codegen drift, and boundary
negative fixtures separate xtask/CI diagnostics under the stable aggregate.
Experimental Oxc type-check cannot satisfy `tsc`. Lint cache keys include exact
tool/TypeScript/platform/config/project-graph inputs; skipped or zero/incomplete
file selection fails.

Consume plan 094's global bunfig contract: native/type-aware Oxlint, `tsc`, and
the selected GraphQL generator must resolve exact lock-local
executables under Bun with auto-install disabled. A Node descendant or mutable
`@latest` command fails the gate.

## Test Plan

- Compiler/effective-Oxlint config and selected-file goldens plus weakening,
  zero-file, parse-failure, and resolution-failure fixtures.
- One failing fixture for every selected boundary-specific native/type-aware
  rule and custom Oxc-backed ratchet.
- Exact SDL export/drift, named-document validation, generated variable/result/
  runtime-schema drift, operation-name, variables, and no-interpolation tests.
- Valid, missing, extra, wrong-type, null, malformed JSON, and partial GraphQL
  envelopes for static and dynamic operations.
- Valid/malformed/oversized/out-of-order SSE frames with secret-safe errors.
- Search/storage schema round trips and exhaustive-union type tests.
- Result-shaped domain failure and exception-text-classifier rejection tests.
- Assertion/non-null/directive ratchet growth/stale/generated cases.
- Bun-only lint/typecheck/build on a clean checkout.

## Done Criteria

- [ ] Effective TypeScript config contains every required compatible option and
  cannot drift weaker.
- [ ] TypeScript 7, native/type-aware Oxlint, and stable React Hooks rules catch
  every intentional failure, select every intended file, and use no Node process.
- [ ] No handwritten boundary trusts JSON, GraphQL, SSE, search, or storage via
  a generic cast.
- [ ] Handwritten production code contains no `any`, `@ts-ignore`, or unreasoned
  non-null/broad assertion.
- [ ] Authoritative SDL, named documents, generated variables/results, runtime
  schemas, and `{ operationName, query, variables }` transport drift together;
  dynamic GraphQL and every SSE event have explicit runtime contracts.
- [ ] Expected domain failures use exhaustive typed Result-shaped unions rather
  than exception-text control flow.
- [ ] Generated/shadcn exceptions are narrow, owned, and tested.
- [ ] `skipLibCheck` is disabled and the full third-party declaration graph is
  clean without ambient patches or application casts.
- [ ] Required Bun gates pass with no zero-file selection.

## STOP Conditions

- A compiler/rule change requires broad disablement or generated-code edits.
- Runtime validation changes a public payload/search contract without a spec
  migration.
- A fix introduces `any`, a broad assertion, duplicate domain types, or a
  second decoder for the same boundary.
- Native or type-aware Oxlint cannot identify the intended project,
  skips a file class, needs Node, or a critical invariant has no native,
  compiler, xtask, runtime, or test oracle.
- No latest mutually compatible stable dependency set has a clean declaration
  graph; record exact upstream failures and block rather than patch types
  locally or re-enable `skipLibCheck`.

## Remove When

Delete this plan and index row when strict TypeScript 7/`tsc`, native/type-aware
Oxlint, authoritative GraphQL/runtime decoding, Result-shaped failures,
escape-hatch ratchets, and generated ownership are required and green.
