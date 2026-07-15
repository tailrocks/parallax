# Plan 152: Establish the generated GraphQL contract foundation

> **Executor instructions**: Build one schema-to-operation pipeline and one
> decoded GraphQL transport after the compiler, architecture, and test
> foundations are green. Export the schema from `parallax-api`, generate static
> operation types/documents/Zod schemas with the exact locked GraphQL Code
> Generator toolchain under Bun, and check every artifact into the repository.
> Do not migrate product features or create every product operation here. The
> only product-specific exception in this plan is the bounded dynamic dashboard
> widget-series contract, because it cannot be represented by a static generated
> operation.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- crates/parallax-api crates/parallax-xtask ui/package.json ui/bun.lock ui/bunfig.toml ui/tsconfig.json ui/codegen.ts ui/graphql ui/src/lib/api.ts ui/src/platform/graphql 'ui/src/routes/dashboards.$dashboardId.tsx' ui/src/features/dashboards ratchet.toml`
> Reconcile Plan 100's final platform path and cache adapter before editing. Do
> not create a second transport beside a completed Plan 100 owner.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 095, 100, 101, 128, 129, 130
- **Category**: TypeScript / GraphQL / generated runtime contracts
- **Planned at**: `e3e7997`, 2026-07-12
- **Status**: BLOCKED — Plans 100, 128, and 129 are not complete

## Why This Matters

The Rust API already constructs one Juniper schema, but the UI has no exported
SDL contract. Product routes embed anonymous GraphQL strings, interpolate
values, declare handwritten generic result types, and trust `response.json()`
through `as T`. Schema, operation, TypeScript type, and runtime validation can
therefore drift independently.

Most operations are static and should be generated from named `.graphql`
documents. Dashboard widget series is the sole real dynamic shape: the number
of aliased `metricSeries` fields follows the saved dashboard layout. It needs a
small reviewed AST builder and an exact alias-aware decoder, not a raw string or
an exemption from runtime validation.

This plan establishes the reusable pipeline and freezes current inline debt for
the feature migration plans. It does not move route components, feature models,
or all static operations.

## Current Evidence

- `parallax-api::build_schema().as_sdl()` exists and current tests inspect only
  sentinel strings; no deterministic checked-in SDL is exported for the UI.
- `ui/package.json` has no GraphQL Code Generator, TypedDocumentNode, GraphQL
  AST, schema-validation, or codegen-drift command.
- `ui/src/lib/api.ts` accepts a raw string and generic `T`, sends only
  `{ query }`, and asserts the JSON envelope to `{ data?: T; errors?: unknown[] }`.
- Product routes interpolate identifiers, ranges, filters, SQL, and mutations
  into GraphQL text through `gqlString`.
- Dashboard detail builds up to 24 aliases per document as a template string
  and trusts an open `Record<string, Series[]>` response.

## Fixed Decisions

### Authoritative schema

`parallax-api::build_schema().as_sdl()` is the only GraphQL schema authority.
Expose one library function that returns that SDL and make
`cargo xtask ui graphql export` write `ui/graphql/schema.graphql`. Normalize only
line endings and exactly one final newline. Do not reorder definitions with
regex, maintain a handwritten SDL, fetch introspection from a running server,
or make the TypeScript toolchain authoritative.

Export twice and require byte-identical output. `cargo xtask ui graphql check`
regenerates into a temporary directory and fails a byte diff against the
checked-in schema.

### One exact Bun code-generation toolchain

Use one strict `ui/codegen.ts` loaded by Bun. Add the latest mutually compatible
stable releases at execution time through Bun with exact versions, including:

- `@graphql-codegen/cli`;
- `@graphql-codegen/typescript`;
- `@graphql-codegen/typescript-operations`;
- `@graphql-codegen/typed-document-node`;
- `@graphql-codegen/typescript-validation-schema`;
- the official near-operation-file preset required for sibling output;
- `graphql` for typed `DocumentNode` AST construction/printing; and
- the TypedDocumentNode core type package if the generated output imports it
  directly.

`zod` remains the single runtime schema library. Every direct tool/runtime
package and transitive compatibility unit is lock-exact under Plan 101. Invoke
the generator only as:

```text
bunx --bun --no-install graphql-codegen --config codegen.ts
```

No Node process, package auto-install, JavaScript plugin, alternate generator,
post-generation text rewrite, or remote schema is allowed. Oxfmt is the sole
formatter for generated TypeScript after Plan 130; its exact output manifest is
part of deterministic generation.

### Static operation output

Generate one base file:

```text
ui/src/platform/graphql/generated/schema-types.generated.ts
```

For each production static operation, use the following final template:

```text
ui/src/features/<feature>/api/<operation>.graphql
ui/src/features/<feature>/api/<operation>.generated.ts
ui/src/features/<feature>/api/<feature>-api.ts
ui/src/features/<feature>/tests/api/<operation>-contract.test.ts
```

Use lower kebab-case file names and exactly one named operation per `.graphql`
file. Operation names are globally unique PascalCase names beginning with the
feature name, such as `InvestigationsList` or `DashboardsSave`. Feature adapters
may import their sibling generated file. Routes, components, and feature
facades expose decoded domain values, never wire DTOs or generated modules.

The codegen configuration uses `typescript` for base schema types and the
near-operation output uses `typescript-operations`, `typed-document-node`, and
`typescript-validation-schema`. The validation plugin is configured with:

```text
schema = "zodv4"
withOperationType = true
withObjectType = false
zodOptionalType = "nullable"
strictScalars = true
avoidOptionals.field = true
avoidOptionals.object = true
avoidOptionals.inputValue = false
avoidOptionals.defaultValue = false
maybeValue = "T | null"
inputMaybeValue = "T | null | undefined"
documentMode = "documentNode"
immutableTypes = true
useTypeImports = true
```

Output fields selected by an operation are required keys. GraphQL nullable
output is `T | null` and Zod nullable, not optional or nullish. Input-variable
optionality follows GraphQL variable definitions independently. Map every
built-in scalar consistently in TypeScript and Zod; a new custom scalar fails
generation until both mappings and positive/negative fixtures exist.

Generated files are checked in, carry one machine-owned header, are included in
TypeScript compilation, and are never manually edited or re-exported from a
feature facade. Generation runs twice byte-identically and the check command
fails schema, document, config, package, output-manifest, formatter, or generated
artifact drift.

### Static request and decode contract

Every static field argument uses a GraphQL variable. Anonymous operations,
multiple operations per file, template interpolation, raw string transport,
literal argument values, and manual escaping are forbidden. Fragments are
allowed only when named, feature-owned, statically reachable from one named
operation, and validated by the same schema/codegen command.

Adopt Plan 100's platform transport in place and expose only these public calls:

```ts
executeGraphqlOperation(document, resultSchema, variables, options)
executeCachedGraphqlOperation(document, resultSchema, variables, options)
```

The document is a generated `TypedDocumentNode<Result, Variables>` and the
schema is its generated Zod operation-result schema. The implementation derives
and validates the single operation name from the AST, prints the document
deterministically, and sends exactly:

```text
{ operationName, query, variables }
```

The cached form preserves Plan 100's client-only TTL, capacity, in-flight
deduplication, abort, and per-request SSR isolation until Plan 133 replaces that
cache. Its key includes the printed document plus a deterministic canonical
GraphQL-variable encoding; object keys sort, array order is preserved, and
undefined, non-finite, cyclic, or non-JSON values fail before fetch.

Read response JSON as `unknown`. Decode the GraphQL envelope first, preserving
the current behavior of rejecting any non-empty `errors` array even when
partial `data` exists. Require `data`, then parse it once with the generated
operation schema. Throw one typed `Error` subclass with a stable code for HTTP,
malformed JSON, invalid envelope, GraphQL errors, invalid operation data, abort,
and transport failure. Diagnostics include operation name, status, bounded
schema issue paths/count, and error code only; never include variables, query
text, response bodies, headers, tokens, or storage contents.

### Sole dynamic dashboard exception

Static generation is mandatory everywhere except dashboard widget-series
batching. Implement that exception as a typed `DocumentNode` object assembled
from GraphQL `Kind` nodes; do not construct source text and parse it afterward.
The builder contract is exact:

1. Preserve input widget order and split it into documents of at most 24
   top-level `metricSeries` fields.
2. Name the operation `DashboardWidgetSeries` and derive aliases from the
   global zero-based ordinal as `series_<ordinal>`.
3. Give each alias its own exact variable names for metric name, from/to nanos,
   aggregation, and nullable group-by. Every argument value is a `Variable`
   node; all user/data values exist only in the variables object.
4. Use the fixed selection `groupValue` and `points { tsNanos value }`. No
   caller can inject a field, alias, directive, fragment, type, or selection.
5. Before parsing values, require the returned data object's sorted key set to
   equal the sorted expected alias set exactly. Missing, extra, duplicate, or
   malformed aliases fail the whole chunk.
6. Parse each alias independently with one strict Zod v4 series schema matching
   the SDL nullability, then restore original widget order. An empty widget list
   makes no request.

The narrow dashboard document builder, variables builder, alias validator,
series schema, and adapter may land under
`ui/src/features/dashboards/api/widget-series-*` because this is the only
non-generated product contract in scope. The current route may delegate only
its existing widget-series loading seam to that adapter. Do not move dashboard
components, layout/model code, static dashboard operations, cache policy, or
route ownership; Plan 137 owns that later feature migration.

## Target Foundation

```text
crates/parallax-api/src/
  schema.rs                              authoritative SDL function
crates/parallax-xtask/src/
  ui/graphql.rs                          export/check/policy orchestration
ui/
  codegen.ts                             strict checked-in codegen config
  graphql/
    schema.graphql                       deterministic checked-in SDL
  src/platform/graphql/
    client.ts                            unknown-first envelope + operation decode
    error.ts                             typed bounded error contract
    variables.ts                         canonical GraphQL variable encoding
    generated/
      schema-types.generated.ts          checked-in generated base types
    tests/
      client.test.ts
      variables.test.ts
      generation-contract.test.ts
      fixtures/
        static-probe.graphql              one named test-only pipeline probe
        static-probe.generated.ts         checked-in deterministic probe output
  src/features/dashboards/api/
    widget-series-operation.ts            bounded DocumentNode builder
    widget-series-schema.ts               per-series Zod v4 decoder
    widget-series-api.ts                  alias-aware decoded adapter
  src/features/dashboards/tests/api/
    widget-series-contract.test.ts
```

If Plan 100 established different final filenames inside
`ui/src/platform/graphql/`, reconcile by responsibility and update this tree
before implementation. Extend the existing owner; never retain parallel client,
cache, error, or variables modules with the same responsibility.

## Scope

In scope:

- Deterministic `parallax-api` SDL export and checked schema drift.
- Exact Bun-only GraphQL Code Generator, official TypeScript operation/
  TypedDocumentNode plugins, Zod v4 operation-result schemas, and checked-in
  deterministic artifacts.
- Static operation naming/path/nullability/scalar/variables-only policy and a
  test-only generated probe.
- Unknown-first GraphQL envelope and generated result decoding.
- The sole bounded dynamic dashboard AST/alias/per-series runtime contract.
- No-growth handoffs for current inline static operations and legacy escaping.
- Required schema/config/generated/runtime/policy diagnostics.

Out of scope:

- Moving or creating all product static operations. Plans 134-143, 149, and 150
  own their named feature operations and adapters using this template.
- Dashboard components, model/layout migration, static dashboard documents, or
  facade completion, owned by Plan 137.
- TanStack Query adoption and final cache deletion, owned by Plan 133.
- SSE, search, storage, environment, cross-window, or non-GraphQL JSON decoding,
  owned by Plan 153.
- GraphQL server field/semantic changes, persisted queries, subscriptions,
  fragments shared across features, schema federation, or a second API client.
- Runtime code generation, remote introspection, generated files ignored by
  Git, or TypeScript types treated as runtime validation.

## Required Commands

| Purpose | Command | Expected result |
|---------|---------|-----------------|
| Export SDL | `cargo xtask ui graphql export` | writes only deterministic `ui/graphql/schema.graphql` |
| Generate | `cd ui && bun run graphql:generate` | Bun-forced exact local codegen plus Oxfmt succeeds |
| Drift | `cargo xtask ui graphql check` | schema, config, documents, manifest, and outputs reproduce byte-for-byte |
| Contract tests | `cd ui && bun run test:ci -- src/platform/graphql src/features/dashboards/tests/api` | static, envelope, variables, and dynamic cases pass |
| Type/lint/format | `cd ui && bun run typecheck && bun run lint && bun run check` | handwritten and generated classes pass their contracts |
| Policy | `cargo xtask policy --only ui.graphql-contract` | naming, variables-only, legacy ratchet, ownership, and drift fixtures pass |
| Aggregate | `cargo xtask ci --fast` | GraphQL diagnostics appear once and are required |

## Legacy Static-Operation Handoffs

Plan 152 inventories every raw static GraphQL string, generic result argument,
`gqlString` call, and direct GraphQL transport import at execution time. Store
exact AST-backed rows with path, operation/field identity, cache mode, and one
removal owner:

| Owner plan | Static operation area |
|------------|-----------------------|
| 134 | investigations |
| 135 | SQL workspace and saved views |
| 136 | ecosystem |
| 137 | dashboard list/detail/metadata/mutations, excluding the dynamic series exception completed here |
| 138 | services |
| 139 | issues |
| 140 | runs |
| 141 | logs |
| 142 | traces and trace field exploration |
| 143 | app status and quick navigation |
| 149 | runtime metrics and story capability data |
| 150 | overview |

The exact baseline may shrink only. A new raw string, interpolation,
`gqlString`, anonymous operation, generic result assertion, or direct transport
consumer fails immediately. Plan 151 removes the legacy client/escaping surface
after all owners retire their rows. This handoff does not make Plan 152 wait for
the product migrations.

## Steps

### Step 0: Verify prerequisites and reconcile ownership

Confirm Plans 095, 100, 101, 128, 129, and 130 are complete. Reproduce strict
TypeScript/Oxlint, sole-Oxfmt formatting, Oxc-backed architecture policy,
dependency integrity, Vitest diagnostic, and Plan 100 platform GraphQL/cache
tests. Resolve the live platform owner and inventory all raw GraphQL consumers
before adding files.

Run the drift command and classify every changed GraphQL responsibility as
schema export, generated foundation, platform transport, dynamic dashboard
exception, or one legacy handoff row. Stop on an unknown or duplicate owner.

**Verify:** prerequisite commands are green; the inventory covers every call,
query/mutation string, interpolation helper, generic result argument, cache
mode, and current test; no product operation is silently assigned to this plan.

### Step 1: Export one deterministic SDL

Add the small `parallax-api` library function around
`build_schema().as_sdl()`. Add xtask export/check commands that normalize line
endings and one final newline, use a temporary comparison for check mode, write
atomically in export mode, and narrate their work without printing schema
contents.

Check in `ui/graphql/schema.graphql`. Replace the current sentinel-only schema
test with exact export determinism plus sentinel coverage so an empty or wrong
schema cannot be accepted merely because the file matches itself.

**Verify:** two exports are byte-identical; schema field/type/nullability changes
fail drift; missing/empty/truncated/wrong-path/write-failure fixtures fail; check
mode never modifies the worktree.

### Step 2: Lock the Bun codegen compatibility unit

Resolve the latest mutually compatible stable codegen packages, GraphQL AST
runtime, TypedDocumentNode type package when imported, and Zod v4 under Plan
101. Add each direct package through Bun with an exact version. Prove the CLI,
TypeScript config, plugins, preset, schema load, and output run through Bun on
supported macOS/Linux with Node absent and install disabled.

Add `ui/codegen.ts`, the base output, and exact document/output globs. Include
only feature `api/*.graphql` documents and the named platform test probe; reject
build output, snapshots, arbitrary fixtures, and legacy inline strings. Put the
config and generated outputs in TypeScript/Oxlint/Oxfmt ownership manifests.

**Verify:** missing package, version skew, wrong plugin, remote schema, omitted
glob, extra output, Node ancestry, implicit install, and unsupported scalar
fixtures fail. Generate-format-generate twice with no byte diff.

### Step 3: Prove the static operation template

Add one test-only named probe operation and its checked-in sibling output. It
must exercise non-null and nullable output, required and optional variables,
list/object nesting, and every current scalar category. Assert the exact
TypedDocumentNode result/variables types and Zod operation-result schema.

Add Oxc-backed policies for one named operation per file, globally unique
feature-prefixed names, variables-only field arguments, named reachable
fragments, correct path/output pairing, generated header/owner, and no direct
generated facade export. Validate documents against the checked SDL before
generation.

**Verify:** anonymous, duplicate-name, two-operation, literal-argument,
interpolation, invalid-field, wrong-variable-type, orphan fragment, stale
output, nullable-as-optional, missing selected field, and manual generated-edit
fixtures fail.

### Step 4: Replace the platform GraphQL trust boundary

Refine Plan 100's platform adapter in place to accept only generated
TypedDocumentNode plus generated result schema and variables. Derive the one
operation name, print a deterministic query, build the exact request envelope,
and preserve abort/HTTP behavior. Decode JSON as `unknown`, validate the GraphQL
envelope, reject non-empty errors, require data, and parse the operation result
once.

Implement the canonical variable encoder and preserve Plan 100's exact client-
only cache/in-flight/SSR semantics in the cached call. Add typed bounded errors
and secret-safe diagnostics. Do not switch legacy product operations in this
step; their exact no-growth inventory remains callable only through the frozen
compatibility path until each owner migrates.

**Verify:** request body tests assert operation name, printed query, variables,
headers, abort, and cache identity. Response tests cover valid, nullable,
malformed JSON, non-object envelope, errors-only, data-plus-errors, missing/null
data, missing/wrong-type selected fields, HTTP failure, abort, and secret-safe
diagnostics. No test mocks away envelope/result decoding.

### Step 5: Implement the bounded dynamic dashboard contract

Create the typed AST/variables builder, strict per-series schema, exact alias-set
validator, and decoded adapter under the narrow dashboard API path. Use typed
GraphQL AST nodes directly and prove every field/variable/selection invariant.
Delegate only the existing widget-series request seam from the current route;
preserve chunking, request order/count, cached/raw choice, result order, empty
behavior, abort behavior, and visible output.

Delete the old template-string alias builder and its dynamic generic
`Record<string, Series[]>` assertion in the same change. Do not touch static
dashboard strings or any other dashboard responsibility.

**Verify:** 0, 1, 24, 25, and multi-chunk widget cases preserve request and
result order. Negative cases reject 25 fields in one document, literal AST
values, alias/variable mismatch, extra/missing/duplicate/unexpected alias,
wrong/null series values, malformed points, injected field/directive/fragment,
and secret-bearing diagnostics. Existing dashboard behavior tests remain green.

### Step 6: Freeze legacy debt and require drift gates

Store the exact legacy handoff table in the typed ratchet. Add separate stable
diagnostics for SDL drift, codegen config/package/document validity, generated
artifact drift, platform envelope/result contracts, dynamic dashboard contract,
and legacy no-growth. Cache keys include Rust schema source, exporter, SDL,
codegen/plugin/GraphQL/Zod versions, config, document manifest, Oxfmt config,
generated outputs, TypeScript config, and platform.

Wire the diagnostics once under the stable aggregate. Commands fail on skipped,
zero-document, zero-generated-output, wrong-platform, cached-with-wrong-inputs,
Node-spawned, or implicit-install execution.

**Verify:** each intentional drift routes to one exact diagnostic; human/JSON
findings agree; two clean local runs are deterministic; typecheck, lint, format,
unit tests, build, GraphQL check, policy, and aggregate pass.

## Test Plan

- Exact SDL export determinism, non-empty sentinels, atomic write, check-only,
  and schema/type/field/nullability drift fixtures.
- Bun/no-Node exact codegen package/plugin/preset/config/document/output matrix
  on supported macOS and Linux CI environments.
- Static operation naming, one-operation, variables-only, fragment reachability,
  schema validation, scalar mapping, nullable-key, and generated-owner fixtures.
- Checked-in base/probe output reproducibility and manual/stale/missing/extra
  artifact failures.
- Unknown-first JSON/envelope/result tests for valid, null, partial-error,
  malformed, missing, wrong-type, HTTP, abort, and bounded diagnostic cases.
- Canonical variable encoding for key order, array order, null, undefined,
  non-finite values, cycles, and cache/in-flight/SSR identity.
- Dynamic dashboard AST snapshots and structural assertions for operation,
  aliases, variables, field limit, selection, chunking, order, exact alias set,
  per-series schemas, and injection attempts.
- Legacy raw-string/interpolation/generic-result/direct-import no-growth and stale
  owner fixtures.
- Clean-checkout typecheck, lint, format, Vitest, build, GraphQL drift, policy,
  and aggregate commands.

## Done Criteria

- [ ] `build_schema().as_sdl()` exports one deterministic checked-in SDL and
  any Rust schema drift fails before UI generation.
- [ ] Exact stable GraphQL Code Generator packages run only through Bun with
  installation disabled and produce deterministic checked-in base/operation
  TypeScript plus Zod v4 operation-result schemas.
- [ ] Static operation paths, names, variables-only arguments, nullable output,
  scalar mappings, and generated ownership are machine-enforced.
- [ ] The platform client accepts only TypedDocumentNode, generated result
  schema, and variables; JSON/envelope/data are decoded from `unknown` once.
- [ ] Requests send `{ operationName, query, variables }`; no new raw string,
  interpolation, manual escaping, or generic result assertion is possible.
- [ ] Dynamic dashboard series uses a typed AST, at most 24 fields per document,
  deterministic aliases, variables-only values, exact alias-set validation, and
  one strict per-series schema while preserving behavior.
- [ ] Every legacy static operation has one exact no-growth removal owner; this
  plan has not migrated product routes/features beyond the dynamic exception.
- [ ] Schema, config, generated, runtime, dynamic, ratchet, Bun ancestry, and
  aggregate gates are deterministic and required.

## STOP Conditions

Stop and report if:

- `build_schema().as_sdl()` is not byte-deterministic after line-ending/final-
  newline normalization, or exporting requires a running server;
- the exact codegen compatibility unit needs Node, implicit install, a remote
  schema, a second generator, a JavaScript plugin, or post-generation text
  rewriting;
- Zod v4 operation-result output cannot represent selected nullable fields as
  required nullable keys or cannot compile with TypeScript 7;
- generated output cannot be reproduced on supported macOS/Linux or requires a
  broad generated/lint/format exclusion;
- the platform adapter needs a cast, trusts generic JSON, changes public error/
  cache/abort semantics, or logs query/variables/payload secrets;
- the dashboard exception needs raw source construction, parsing a generated
  string, more than 24 fields, uncontrolled selections, or an open response
  record; or
- implementation begins moving static product operations, route components,
  feature models, TanStack Query, or a Plan 153 boundary.

## Remove When

Delete this plan and its index row when the deterministic SDL, exact Bun codegen
pipeline, static operation template/probe, decoded platform client, bounded
dynamic dashboard exception, legacy no-growth handoffs, and required drift/
runtime/policy gates are green. The handoff rows remain in `ratchet.toml` until
their owning feature plans remove them; those migrations do not block this
foundation plan's retirement.
