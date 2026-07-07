# Plan 059: GraphQL operation explorer in trace detail — field/resolver tree, latency, partial errors (consumes plan 047's traces)

> **Executor instructions**: Follow this plan step by step. Run every
> verification command. On any STOP condition, stop and report. When done,
> update the status row in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat ed5b10f..HEAD -- ui/src/routes/traces.\$traceId.tsx ui/src/components/console crates/parallax-api/src/lib.rs`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW (additive, kind-specific rendering)
- **Depends on**: plan 058 (traceEvents backbone — used for flag/exception
  beats; the tree itself builds from spans). Pairs with playground plan 047
  (GraphQL field spans, N+1 contrast, partial errors) — without 047 the
  section renders for no trace, which is correct behavior.
- **Category**: direction
- **Planned at**: commit `ed5b10f`, 2026-07-07

## Why this matters

The research brief's GraphQL story — "which field/resolver caused the
latency?" — has no UI surface: zero references to `graphql.*` attributes
exist in `ui/src` (only the API client's own `graphql()` helper), so GraphQL
resolver spans render as opaque attribute bags. Playground plan 047 turns on
Java GraphQL data-fetcher spans (`otel.instrumentation.graphql.data-fetcher.enabled=true`)
with `graphql.field.name`/`graphql.field.path` attributes and N+1-vs-batched
contrast scenarios; this plan gives those traces a differentiated reading: an
operation header, a field tree with per-resolver latency, and partial-error
badges. The pattern to follow already exists — `db.query.text` gets a special
inspector block today; this generalizes that idea to a span-domain section.

## Current state

Verified at commit `ed5b10f`.

- No GraphQL-aware rendering: `rtk grep -rn '"graphql\.' ui/src` → no hits.
- The special-case precedent — `ui/src/routes/traces.$traceId.tsx:444`:

  ```tsx
  const dbQuery = valueFor(attributes, "db.query.text")
  ...
  {dbQuery ? (
    <InspectorCode title="db.query.text" value={dbQuery} copy />
  ) : null}
  ```

- Trace page composition: waterfall card at `traces.$traceId.tsx:232-243`,
  inspector card from `:456`, span attributes parsed by `parseKeyValues`
  (`:439`). Spans arrive from the `trace(traceId)` query (fields include
  `name kind attributes` — see the loader query near `:72-77`).
- Attribute names the tree keys on (from OTel GraphQL semconv + plan 047's
  scope): `graphql.operation.type`, `graphql.operation.name`,
  `graphql.field.name`, `graphql.field.path`, opt-in `graphql.document`.
  Java data-fetcher spans are children of the operation span (span kind
  INTERNAL, names like `graphql.fetch`/field name — plan 047 records the
  exact shape; a STOP condition covers drift).
- Reusable primitives: `HeatCell` (quintile color for durations,
  `ui/src/components/console/heat-cell.tsx:11-40`), `KeyValueList`,
  `InspectorSection`/`InspectorCode` (in `traces.$traceId.tsx`),
  `SpanKindChip` (`span-kind.tsx`).
- Conventions: strict TS, shadcn-on-Base-UI components, no new heavy deps.

## Commands you will need

| Purpose | Command (from `ui/`) | Expected |
|---------|----------------------|----------|
| Typecheck/lint/test/build | `bun run typecheck && bun run lint && bun run test && bun run build` | all exit 0 |

## Scope

**In scope**:
- `ui/src/lib/graphql-trace.ts` (new — pure tree-builder over the page's
  span array)
- `ui/src/components/console/graphql-operation.tsx` (new — the section
  component)
- `ui/src/routes/traces.$traceId.tsx` (mount the section; one conditional)
- Tests for the tree-builder + component

**Out of scope** (do NOT touch):
- Backend/resolver changes — the spans already carry what's needed
  (plan 047 side). No new GraphQL API fields.
- The waterfall component (`trace-waterfall.tsx`) — plan 061 owns waterfall
  modes; this is a separate card.
- Aggregation across traces (per-operation p95 over time) — future; needs
  `aggregateTrace` (deferred in plan 051).
- gRPC/messaging sections — plan 060.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one
  `Co-authored-by: Claude <noreply@anthropic.com>` trailer. Push when done.

## Steps

### Step 1: Pure tree-builder — `graphql-trace.ts`

```ts
export interface GraphqlFieldNode {
  path: string            // "products.reviews" from graphql.field.path
  fieldName: string       // graphql.field.name
  spanId: string
  durationNs: bigint
  selfDurationNs: bigint  // duration minus children (clamped ≥ 0)
  hasError: boolean       // span status ERROR
  callCount: number       // merged siblings with the same path (N+1!)
  children: GraphqlFieldNode[]
}
export interface GraphqlOperation {
  operationSpanId: string
  operationType: string   // graphql.operation.type
  operationName: string | null
  document: string | null // graphql.document (opt-in)
  durationNs: bigint
  fieldErrors: number
  roots: GraphqlFieldNode[]
}
export function buildGraphqlOperations(spans: TraceSpan[]): GraphqlOperation[]
```

Rules: an operation span = any span whose attributes contain
`graphql.operation.type`. Field spans = descendants carrying
`graphql.field.name` (walk `parentSpanId` up to the nearest operation span).
Tree by `graphql.field.path` segments when present, else by span
parent/child. **Merge repeated siblings with the same path into one node
with `callCount` and summed duration** — this is what makes N+1 legible
(047's contrast scenario shows `callCount: 8` vs `1`). Reuse the span type
the route already defines (import it or lift it to the lib file — check how
`trace-tree.ts` imports its span shape and match that convention).

**Verify**: `bun run test` — unit tests: single op with nested fields;
N+1 merge (8 same-path siblings → one node, callCount 8); partial error
(field span with ERROR status under an OK operation → `fieldErrors` counted,
`hasError` on the node); spans without graphql attrs → `[]`.

### Step 2: Section component — `graphql-operation.tsx`

Render per operation (usually one):
- Header row: `operationType` + `operationName ?? "(anonymous)"`, total
  duration, `fieldErrors > 0` → rose badge "N field errors" (partial-error
  case: HTTP 200 + field errors is exactly what it flags).
- Field tree: indented rows — field name, `callCount > 1` → amber badge
  `×N` (the N+1 marker), self/total duration with `HeatCell` coloring
  against the sibling set, error badge per row. Row click → `onSelect(spanId)`
  (the page's existing span-select callback) so the inspector shows the raw
  span.
- `document` present → collapsed `InspectorCode`-style block (move
  `InspectorCode` out of the route file into the component's module or a
  shared location ONLY if the route exports it cleanly; otherwise replicate
  the 10-line pattern locally and note it).
- Empty operations array → render nothing (the page mounts it
  unconditionally-but-empty-safe).

**Verify**: `bun run test` — component renders the N+1 badge and error
badge from a fixture; nothing renders for a non-GraphQL fixture.

### Step 3: Mount on the trace page

In `traces.$traceId.tsx`, below the Waterfall card (`:232-243`) and above
Trace logs: `const ops = useMemo(() => buildGraphqlOperations(spans), [spans])`,
render `<GraphqlOperationCard operations={ops} onSelect={setSelectedId} />`
inside a `Card` titled "GraphQL" only when `ops.length > 0`.

**Verify**: `bun run typecheck && bun run build` → exit 0. Live (if 047's
telemetry is available): a playground GraphQL trace shows the tree; a plain
checkout trace shows no GraphQL card. Record which check ran.

## Test plan

- `graphql-trace.test.ts`: the four Step 1 cases + deterministic ordering.
- `graphql-operation.test.tsx`: badges + select callback + empty-state.
- Pattern: follow the existing co-located UI test files (grep
  `*.test.ts` under `ui/src` and match the harness/imports of the nearest
  one, e.g. the trace-tree or logs-table tests).

## Done criteria

- [ ] `bun run typecheck && bun run lint && bun run test && bun run build` all exit 0
- [ ] `rtk grep -n "graphql.field.name" ui/src/lib/graphql-trace.ts` → present
- [ ] N+1 merge covered by a passing test (`callCount`)
- [ ] Trace page renders the card only for GraphQL traces (test or recorded
      live check)
- [ ] `plans/README.md` status row updated

## STOP conditions

- Plan 047 landed with a different span shape than
  `graphql.field.name`/`graphql.field.path` descendants of an
  operation-typed span (read one real trace first when available) — adjust
  keys ONLY if the real shape is documented in 047's commit; otherwise
  report.
- The route's span type isn't importable without a circular dependency —
  report the structure rather than duplicating the type in a third place.
- Merging N+1 siblings breaks selection (merged node maps to many spanIds)
  — select the slowest constituent and note it; if that feels wrong in
  review, STOP and propose alternatives.

## Maintenance notes

- Plan 060 adds sibling domain sections (gRPC/messaging) — keep this
  component's "domain section" shape (build from spans, mount when
  non-empty, onSelect into the inspector) as the template.
- Cross-trace field aggregation (Apollo-style field p95) is the natural
  follow-up once `aggregateTrace` exists (deferred in plan 051).
- Reviewer: `selfDurationNs` clamping and the sibling-merge are where subtle
  math bugs live; check the tests actually pin them.
