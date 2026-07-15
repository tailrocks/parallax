# Plan 153: Establish non-GraphQL runtime boundary foundations

> **Executor instructions**: Run this foundation after Plans 095, 100, 101,
> 128, 129, and 130 are complete. Harden the provisional Plan-100 browser adapters
> without moving product features or changing observable behavior. Build one
> unknown-first mechanism for JSON text, SSE/EventSource, URL search, and browser
> storage, plus fail-closed first-consumer policy for runtime environment and
> cross-window messages. Product schemas,
> storage keys/versions, search defaults, and live-record models remain with
> their feature plans. Stop on a listed STOP condition instead of adding a
> second decoder, a broad browser-global exception, or payload-bearing logs.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src/hooks/use-live-stream.ts ui/src/hooks/__tests__/use-live-stream.test.tsx ui/src/lib/use-visible.ts ui/src/lib/__tests__/use-visible.test.tsx ui/src/platform ui/src/routes ui/src/features crates/parallax-xtask ratchet.toml ui/AGENTS.md PROJECT_STRUCTURE.md`
> Resolve Plan-100 provisional destination paths and current migration
> handoffs before editing. A changed product contract is a STOP condition.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 095, 100, 101, 128, 129, 130
- **Category**: TypeScript / runtime safety / platform boundaries
- **Planned at**: `e3e7997`, 2026-07-12
- **Status**: BLOCKED — Plans 100, 128, and 129 are not complete

## Why This Matters

The strict compiler cannot validate values received after compilation. At the
baseline, the shared live hook casts `MessageEvent.data`, feature routes parse
SSE JSON independently, SQL reaches `localStorage` directly, search decoding is
implemented separately in routes, and malformed live frames are silently
dropped. Those edges have no uniform cancellation, failure, diagnostic, or
policy contract.

Plan 100 moves generic adapters to their final layer without changing behavior;
this plan hardens those adapters. Plan 152 separately owns every GraphQL SDL,
document, generated operation schema, envelope, transport, and cache concern.
Plans 134-143, 149, and 150 instantiate product-specific contracts on top of the
two foundations.

## Current State

- `ui/src/hooks/use-live-stream.ts` directly constructs `EventSource`, casts
  `event.data` to `string`, catches every decoder failure without a diagnostic,
  buffers on a timer, and closes/reopens through `usePageVisible`.
- `ui/src/lib/use-visible.ts` directly owns `document.hidden` and the
  `visibilitychange` subscription. Its server fallback is visible.
- Logs, traces, and run detail each parse live-frame JSON and assert product
  arrays independently. Their schemas and mapping remain feature work.
- `ui/src/routes/sql.tsx` directly reads/writes `localStorage` and owns the
  history key, JSON shape, dedupe/order/cap, and current read/write failure
  behavior. Plan 135 keeps that product policy.
- Route `validateSearch` callbacks receive `Record<string, unknown>` and mix
  validation with feature defaults and normalization. Each feature retains its
  search schema and semantics.
- There is no current product cross-window message or runtime-environment
  consumer. The foundation must provide guarded mechanisms and negative policy
  fixtures without inventing a product protocol.
- Plan 095 provides the sole Rust Oxc parser/AST/semantic/resolver authority and
  the typed `ratchet.toml`; this plan extends it rather than adding ESLint, an
  Oxlint plugin, regex source checks, or a second import graph.

## Fixed Boundary Contract

### Unknown-first decode and mapping

Use one structural runtime-decoder protocol compatible with generated Plan-152
schemas and feature-owned Zod schemas:

```ts
export interface RuntimeDecoder<T> {
  safeParse(input: unknown):
    | { readonly success: true; readonly data: T }
    | { readonly success: false; readonly error: unknown }
}

export type BoundaryResult<T> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly error: BoundaryError }
```

`decodeJsonText` accepts `unknown`, first proves it is a string, parses JSON to
`unknown`, applies exactly one supplied decoder, and returns a discriminated
result. A feature maps that result once to its own exhaustive error and readonly
domain model. The platform never owns a product DTO, schema, fallback, or UI
message. Do not throw for an expected malformed external value.

### Secret-safe diagnostics

`BoundaryError` contains only a stable boundary ID, finite error code, observed
primitive kind, and optional bounded numeric metadata. The diagnostic sink:

- never includes raw JSON, SSE data, storage content/key, URL query, message
  data, origin, environment value, caught error text, stack, or serialized input;
- caps every operator-facing field and the complete rendered diagnostic;
- uses static codes such as `invalid-type`, `invalid-json`, `schema-rejected`,
  `unavailable`, `read-failed`, `write-failed`, `origin-rejected`, and
  `cancelled`; and
- is injectable and silent by default, so this structural plan does not add
  console noise or telemetry behavior.

Feature-visible errors are separate and preserve their existing text and fatal,
inline, fallback, or skip behavior.

### SSE, cancellation, reconnect, and visibility

- The browser-specific `EventSource` constructor lives only in
  `platform/sse/event-source.client.ts`. A small interface/factory makes tests
  deterministic without replacing production browser behavior.
- The controller accepts an exact URL, decoder, batch callback, flush interval,
  diagnostic sink, visibility source, and optional `AbortSignal`. It owns one
  source, one timer, one generation, and one buffer.
- Null URL, hidden document, abort, URL change, unmount, or disposal closes the
  source, clears the timer and buffer, invalidates the generation, and prevents
  late delivery. Visibility restoration reconnects only the current URL.
- Preserve native EventSource retry behavior. Do not add custom retry/backoff,
  credentials, headers, heartbeat, or deduplication policy.
- `open` after `error` restores the current status. A malformed feature frame is
  skipped and diagnosed without closing the stream or changing status.
- Ordering, reversal, deduplication, caps, filter-generation reset, and product
  frame schemas remain in Plans 140-142; Plan 147 later owns live performance.

### URL and search values

- `decodeSearchValue(input, decoder)` accepts `unknown` and returns a boundary
  result. The feature owns its schema, defaults, coercion, range rules, and
  typed search value.
- A route's `validateSearch` delegates to an explicit feature/domain facade
  decoder. The route does not cast or manually inspect unknown properties, and
  it never imports `platform` directly.
- Query-string construction uses `URL`/`URLSearchParams` with product keys and
  omission rules owned by the feature. Do not concatenate an untrusted query.
- Preserve every existing accepted value, fallback, omission, replace/push
  behavior, and round-trip. Hardening that changes a URL contract requires its
  feature plan to stop and report.

### Browser storage

- `browser-storage.client.ts` is the only handwritten application owner of
  `localStorage` and `sessionStorage`. It handles absent SSR globals and browser
  security/quota exceptions as typed results.
- `versioned-storage-codec.ts` composes storage access, JSON-text decoding, and a
  supplied feature codec. The feature owns the key, version literal, wire shape,
  migration/fallback/delete policy, order/cap, and user-visible failure behavior.
- Reads never silently delete or rewrite corrupt/unsupported data. Writes never
  claim success after an exception. No automatic cross-tab synchronization is
  added by this plan.
- Tests use injected `Storage` doubles. Production code does not replace browser
  storage with an in-memory fallback.

### Environment and cross-window values

There is no current product consumer, so this plan creates no environment or
window-message production adapter. Oxc negative fixtures forbid trusting a
declared environment value, wildcard/suffix/payload-selected origins,
payload-first `MessageEvent` handling, unvalidated `postMessage` data, and
application-level event casts. The first real consumer requires its own plan,
schema/key/protocol owner, exact origin/source contract, abort/unsubscribe tests,
and a `.client.ts` adapter built from the shared decoder/result/diagnostic
primitives. Do not add dead compatibility exports merely to exercise policy.

### Runtime and architecture enforcement

Extend Plan 095's Rust Oxc provider with `ui.runtime-boundaries`. It resolves
aliases/reexports and fails closed on:

- direct `EventSource`, `localStorage`, `sessionStorage`, external
  `MessageEvent.data`, or browser environment access outside the exact platform
  owner;
- `JSON.parse(...) as T`, generic JSON trust, or a decoder result widened back
  to `any`/`unknown` before product mapping;
- inline route casts/manual unknown-property trust instead of a named search
  decoder;
- platform imports from routes, domain, shared, app, or layout;
- a raw external value/caught exception interpolated into diagnostics; and
- a second decoder, browser-global allowlist, JS lint plugin, or import graph.

The shadcn generator island is not rewritten. Any necessary exception is exact,
generated-owner scoped, fixture-tested, and cannot authorize product code.
Existing product violations receive shrink-only rows naming Plans 134-143, 149,
or 150 and their exact removal step. No row can permit a new caller.

## Target Ownership

```text
ui/src/platform/
  external-values/
    boundary-error.ts
    boundary-diagnostic.ts
    runtime-decoder.ts
    decode-json-text.ts
    tests/
      boundary-diagnostic.test.ts
      decode-json-text.test.ts
  sse/
    event-source.client.ts
    live-stream-controller.ts
    use-live-stream.ts
    tests/
      live-stream-controller.test.ts
      use-live-stream.test.tsx
  visibility/
    page-visibility.client.ts
    use-page-visible.ts
    tests/use-page-visible.test.tsx
  url/
    decode-search-value.ts
    tests/decode-search-value.test.ts
  storage/
    browser-storage.client.ts
    versioned-storage-codec.ts
    tests/
      browser-storage-client.test.ts
      versioned-storage-codec.test.ts
crates/parallax-xtask/
  src/policy/ui_runtime_boundaries.rs
  tests/fixtures/ui-runtime-boundaries/
    valid/
    invalid/
```

If Plan 095 or 100 landed the same responsibility under a different exact path,
move it to this tree in one mechanical green change. Do not retain two canonical
names. Platform consumers import exact modules; do not create a catch-all
platform barrel.

## Commands

| Purpose | Command | Expected result |
|---|---|---|
| Architecture | `cargo xtask arch` | platform direction and client reachability pass |
| Runtime policy | `cargo xtask policy --only ui.runtime-boundaries` | all positive/negative fixtures and exact handoffs pass |
| Ratchets | `cargo xtask policy --only ui.ratchets` | external-boundary exceptions are shrink-only and current |
| Format | `cd ui && bun run check` | exit 0 |
| Lint | `cd ui && bun run lint` | zero warnings |
| Typecheck | `cd ui && bun run typecheck` | exit 0 |
| Focused tests | `cd ui && bun run --bun test:ci -- src/platform` | non-empty platform suite passes under Bun |
| All UI tests | `cd ui && bun run --bun test:ci` | complete suite passes under Bun |
| Build | `cd ui && bun run build` | exit 0; client/server boundaries remain valid |
| Aggregate | `cargo xtask ci --fast` | exit 0 with runtime-boundary partition non-empty |

Use exact lock-local tools through Bun. Do not install a package, invoke Node,
add ESLint/typescript-eslint, or introduce an alpha Oxc JavaScript plugin.

## Scope

In scope:

- The exact platform tree above and behavior-preserving replacement of Plan
  100's provisional generic live-stream/visibility owners.
- Runtime-boundary Oxc rules, fixtures, typed ratchets, exact product handoffs,
  and durable placement rules in `ui/AGENTS.md`/`PROJECT_STRUCTURE.md`.
- Generic decode/result/diagnostic, SSE lifecycle, search, and storage
  mechanisms plus environment/cross-window first-consumer policy fixtures.

Out of scope:

- GraphQL SDL/documents/codegen/envelopes/transport/cache or dynamic dashboard
  aliases; Plan 152 owns them completely.
- Product log/trace/run frame schemas, ordering, caps, deduplication, models, or
  UI; Plans 140-142 instantiate the SSE mechanism.
- Product search schemas/defaults, storage keys/versions/migrations, environment
  keys, or window-message protocols; a current feature instantiates only search/
  storage, while the first environment/message consumer requires a future plan.
- Query/cache migration, live optimization, browser E2E infrastructure, backend
  changes, product behavior, new runtime/package, or generated/shadcn edits.

## Git Workflow

- Stay on the active branch; never create a branch or PR.
- Land decode/diagnostic, SSE/visibility, storage/search, environment/message
  policy-fixture, and Oxc-policy slices separately and green.
- Serialize `ratchet.toml`, shared policy fixtures, and durable agent-doc edits
  with all active feature plans. Re-read before editing and change only this
  plan's rows/rules.
- Use Conventional Commits, DCO, and exactly one agent-product trailer; push
  each durable green update under repository policy.

## Steps

### Step 0: Prove prerequisites and freeze every boundary

Confirm Plans 095, 100, 101, 128, 129, and 130 are complete. Run the prerequisite
commands that already exist, then inventory every handwritten `EventSource`,
SSE data read, visibility subscription, `JSON.parse` assertion, route search
decoder, local/session storage access, environment read, and cross-window
message read. Record exact current behavior and assign each product instance to
Plans 134-143, 149, or 150.

Plan 152 may execute concurrently. Confirm the live ledger assigns all GraphQL
values exclusively to it, serialize any shared ratchet/policy write, and keep
this plan's files disjoint. No GraphQL envelope or cache work appears here.
Freeze generic live status transitions, native retry,
250 ms default flush, buffer cleanup, visibility behavior, malformed-frame skip,
and SQL storage failure behavior.

**Verify:** prerequisite gates are green; the inventory assigns every instance
exactly once; no target-focused command is run before its files exist.

### Step 1: Add the decode result and diagnostic primitives

Create the exact `external-values` modules. Implement structural schema
compatibility, `unknown` JSON parsing, exhaustive boundary codes, and a silent
injectable diagnostic sink. Test every primitive input kind, valid JSON value,
invalid JSON, rejected schema, throwing decoder defense, and error projection.

Use compile-time fixtures to prove success narrowing and exhaustive error
handling. Prove diagnostics remain within the configured static cap and contain
none of the supplied JSON, URL, storage, origin, environment, caught-error, or
token-like sentinel values.

**Verify:** focused external-value tests, lint, and typecheck pass; snapshots or
failure output contain no supplied sentinel.

### Step 2: Harden generic SSE and visibility lifecycle

Move/harden Plan-100 provisional live-stream and visibility code into the exact
target paths. Separate browser construction, lifecycle controller, React hook,
and visibility source. Preserve status and batching while adding abort and stale
generation safety. Keep native EventSource reconnect behavior.

Move the existing generic tests, then add URL change, null URL, hidden-at-mount,
hide/show, error/open, malformed unknown/string frame, abort before/after open,
unmount with buffered values, old-source late event, timer disposal, and
diagnostic secrecy cases. Temporary old-path reexports are allowed only for
exact Plans 140-142 consumers with shrink-only expiry rows.

**Verify:** platform SSE/visibility tests pass with fake timers and injected
factory; source/timer/listener counts return to zero after every case.

### Step 3: Establish search and versioned storage mechanisms

Implement the generic search decoder and browser-storage adapter/codec. Keep
feature schemas and policies injected. Test SSR absence, local/session choice,
missing key, valid/corrupt/unsupported-version values, security/quota/read/write
exceptions, no implicit deletion/rewrite, exact one-decode/one-encode behavior,
and secret-safe errors.

Do not edit SQL or another product implementation. Add exact shrink-only Oxc
handoffs so Plan 135 removes direct SQL storage and every feature plan replaces
inline unknown search trust with its named decoder.

**Verify:** focused URL/storage tests pass; Oxc rejects a new direct storage or
inline cast fixture while allowing only the exact recorded legacy call sites.

### Step 4: Ratchet absent environment and window-message boundaries

Add only Oxc positive/negative fixtures and the durable first-consumer placement
contract. Cover declared-but-unvalidated environment values, exact versus
lookalike/suffix/wildcard origins, source-before-payload ordering, payload casts,
missing abort/unsubscribe ownership, late delivery, and secret-bearing
diagnostics. The positive fixture is test-only and uses the shared decoder; it is
not copied into production.

**Verify:** architecture reports zero live product consumer, creates no
environment/window-message production module, and every invalid fixture fails
with exact rule ID, file, reason, remediation, and rerun command.

### Step 5: Require Oxc runtime-boundary policy

Extend only Plan 095's Rust Oxc provider. Add syntax/semantic/resolver fixtures
for aliases, reexports, optional chaining, destructuring, renamed globals,
type-only imports, valid exact adapters, every forbidden direct boundary,
unsafe parse/cast, diagnostic leak, route-to-platform import, generated/shadcn
scope, stale handoff, growth, missing owner, and zero-file selection.

Write exact migration rows for current product violations. Each row contains
path/symbol, owner plan, reason, removal step, and maximum count; rows cannot
grow, broaden, or survive after the call site disappears.

**Verify:** `cargo xtask policy --only ui.runtime-boundaries` passes on the live
tree and every intentional negative fixture fails for the expected rule.

### Step 6: Publish feature instantiation contracts

Record in the live ownership ledger:

- Plan 134: investigation state/search JSON;
- Plan 135: SQL search, storage, and nested row JSON;
- Plans 136-139 and 150: feature search plus embedded non-GraphQL JSON;
- Plans 140-142: feature search, live-frame schemas, and SSE orchestration;
- Plan 143: current non-GraphQL shell browser values only; environment/message
  consumer count remains zero, and discovering one is a STOP/new-plan trigger;
  and
- Plan 149: visibility/cancellation for runtime metrics and time-range URL values.

The feature owns the schema, one mapper, one exhaustive feature error, and
behavior tests. It consumes exact platform modules and removes its handoff in
the same change. Plan 152 remains the only GraphQL owner.

**Verify:** every active handoff has exactly one owner/removal step, every
feature plan names Plans 152/153 correctly, and no handoff overlaps GraphQL.

### Step 7: Run the final gate twice

Run the complete command table twice from a clean state. The second run must not
change generated files, ratchets, diagnostics, test manifests, or tracked source.

**Verify:** all commands exit 0 twice and `git diff --check` is clean.

## Test Plan

- Decode/result tests for all primitive/container values, JSON failures, schema
  failures, throwing decoders, exhaustive narrowing, and map-once behavior.
- Diagnostic tests with payload/query/key/origin/environment/error sentinels,
  cap boundaries, sink failure isolation, and stable codes.
- SSE controller/hook tests for status, native reopen, visibility, URL changes,
  abort, stale events, buffering/order, malformed skip, timers, and teardown.
- Search tests for unknown objects/arrays/primitives and caller-owned fallback/
  normalization without route-to-platform imports.
- Storage tests for local/session/SSR, versions, corrupt data, unavailable and
  throwing access, quota, no implicit rewrite/delete, and exact calls.
- Environment/message policy fixtures for schema rejection, origin/source
  order, lookalikes, payload casts, cancellation ownership, late delivery, and
  no secret-bearing errors, with no production module while consumer count is zero.
- Rust Oxc fixtures for every allowed/forbidden access, alias/reexport path,
  client reachability, exact exception lifecycle, and zero selection.

## Done Criteria

- [ ] One platform mechanism owns unknown-first non-GraphQL decoding and every
  expected failure is exhaustive, payload-free, and bounded.
- [ ] EventSource/visibility has one browser constructor, deterministic
  cancellation/generation cleanup, preserved native reconnect/status/batching,
  and no product schema or ordering policy.
- [ ] Search and storage mechanisms accept supplied schemas while product keys,
  defaults, and versions remain feature-owned; environment/cross-window policy
  is fixture-complete and creates no unused production abstraction.
- [ ] Plan 152 exclusively owns GraphQL; no GraphQL transport/envelope/cache/
  codegen concern or product operation is duplicated here.
- [ ] Oxc policy rejects new direct browser trust, parse-casts, diagnostic leaks,
  route-to-platform imports, and broad/stale exceptions using one Rust provider.
- [ ] Every remaining product violation has one shrink-only feature-plan handoff
  and cannot gain callers.
- [ ] No Node, ESLint/typescript-eslint, JS lint plugin, second import graph,
  package, product behavior, or generated/shadcn edit was introduced.
- [ ] Architecture, runtime policy, ratchets, Bun format/lint/typecheck/tests/
  build, and aggregate pass twice with no tracked drift.

## STOP Conditions

Stop and report if:

- any prerequisite is incomplete/red or Plan 100 has two generic owners;
- Plan 152 does not exclusively own a GraphQL boundary found in the inventory;
- preserving current live/search/storage behavior requires a product decision,
  schema/default/version change, custom retry, or feature edit;
- a product-specific log/trace/run/search/storage/environment/message schema is
  needed in platform;
- runtime safety requires logging raw input, caught error text, URL/query,
  storage key/content, origin, environment value, or stack;
- an Oxc rule needs regex parsing, a second graph, JS plugin, broad global/path
  allowlist, generated edit, or exception that can authorize a new caller;
- origin/source validation cannot occur before window-message payload decode;
- a client-only module reaches a server/domain/shared chunk;
- a target path already has a materially different permanent owner; or
- a required gate fails twice after one reasonable correction.

## Maintenance And Removal

New external-value kinds extend the structural decoder/result and Oxc fixtures;
they do not create a parallel helper. New product boundaries add their schema,
mapper, typed error, tests, and handoff removal in the owning feature. Reviewers
must scrutinize diagnostic fields, client reachability, exact exception scope,
and stale-generation teardown.

Delete this plan and its README row only after all generic modules/policy/tests
are required and green, Plan 152 ownership is disjoint, and every remaining
product instance is assigned to exactly one active feature plan.
