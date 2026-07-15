# Plan 100: Establish the TypeScript project architecture foundation

> **Executor instructions**: Establish one enforced source/test architecture
> before any route feature moves. Preserve all product behavior, URLs, requests,
> cache semantics, and rendering. This plan owns layer definitions, the complete
> source-to-destination ledger, lower-layer extraction, public-facade templates,
> structural policy, and durable agent instructions. Plans 152 and 153 harden
> the provisional GraphQL and non-GraphQL runtime boundaries, Plan 149 then
> establishes route-less capability facades, plans 134-143 and 150 move product
> features independently, and plan 151 verifies final closure.
>
> **Drift check (run first)**:
> `git diff --stat e3e7997..HEAD -- ui/src ui/AGENTS.md PROJECT_STRUCTURE.md ratchet.toml`
> Reconcile changed files with the ownership ledger before editing. Do not create
> a second architecture beside a partial migration.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: HIGH
- **Depends on**: 095, 101, 128, 129
- **Category**: TypeScript / project structure / architecture policy
- **Planned at**: `e3e7997`, revised 2026-07-12
- **Status**: BLOCKED — Plans 128 and 129 are not complete

## Why This Matters

The UI is a 28,000-line TypeScript application organized mainly by technical
file kind. Route modules own URL state, GraphQL strings, DTOs, cache calls,
domain transforms, React state, and presentation. The largest route files are
1,500, 990, 871, 841, and 767 lines. `ui/src/lib/api.ts` mixes runtime selection,
transport, cache, escaping, and many product contracts. Tests import route
implementation exports because those responsibilities have no other owner.

Before parallel feature refactors can be safe, every file needs one destination,
every layer needs a closed dependency direction, and every new file needs a
machine-checkable placement decision. This plan creates that control plane and
the reusable lower layers. It does not move product features or alter behavior.

## Fixed Architecture Decisions

### One package and one strict project

Keep `ui/` as one Bun package and one canonical strict TypeScript project. Do
not add internal npm packages, a frontend monorepo, or TypeScript project
references merely to imitate Rust crates. Source-layer enforcement provides the
needed boundary today. Project references require measured typecheck/editor
evidence and a separate plan defining declaration/build ownership.

### Canonical tree

```text
ui/src/
  app/                         router/provider/composition root only
    tests/                     public composition/router contracts
  layout/                      shell, navigation, global boundaries
    tests/                     shell/navigation/theme/boundary contracts
  routes/                      TanStack route adapters only
    tests/                     URL/search/loader/boundary contracts
  features/<feature>/
    api/                       documents, schemas, decoded adapters
    model/                     domain state and pure transforms
    queries/                   created by plan 133 on first real use
    components/                feature-owned presentation
    hooks/                     feature orchestration only
    tests/                     separated feature tests
      api/
      model/
      components/
      integration/
    index.ts                   reviewed explicit public facade
  domain/<concept>/            framework-neutral cross-feature concepts
    tests/                     pure domain contracts
  platform/<adapter>/          GraphQL, SSE, storage/clock/runtime adapters
    tests/                     technical boundary contracts
  shared/
    components/                product-neutral composed components
    hooks/                     product-neutral hooks
    lib/                       cohesive named pure utilities only
    tests/                     product-neutral component/hook/lib contracts
  test/                        Vitest setup/builders; no test bodies
  components/ui/               shadcn CLI-owned primitive island
  lib/utils.ts                 shadcn CLI-owned `cn` island
  routeTree.gen.ts             TanStack-generated composition
  styles.css                   global tokens and Tailwind entry
ui/tests/harness/              plan 129 test-infrastructure self-tests
ui/tests/e2e/                  plans 132 and 144-146 browser tests
ui/test-matrix.json            plan 129 durable risk/evidence manifest
```

Create directories only with their first real file. Empty architecture theater
is forbidden.

### Closed dependency graph

| From | Allowed product imports | Forbidden imports |
|------|-------------------------|-------------------|
| `app` | all layers for composition plus generated route tree | app internals imported by lower layers |
| `routes` | feature facades, `domain`, `shared`; root-only reviewed layout entry | feature internals, `platform`, other route implementations, `app` |
| `layout` | feature facades, `domain`, `shared` | routes, feature internals, platform, app |
| `features/<a>` | own internals, domain, platform, shared, explicitly approved facade of `<b>` | another feature's internals, routes, layout, app |
| `platform` | domain and shared | features, routes, layout, app |
| `domain` | product-neutral pure shared utilities only | React, TanStack, browser/transport APIs, features |
| `shared` | shadcn primitives and third-party libraries | Parallax feature/domain concepts and every upper layer |

Generated `routeTree.gen.ts` is the only reverse composition exception.
Cross-feature imports require an exact `feature A -> feature B facade` row with
reason and owner. Prefer passing a typed value/callback from composition over
adding an edge.

### Complete feature catalog

| Owner | Routes/surfaces | Separate migration plan |
|-------|-----------------|-------------------------|
| `investigations` | `/investigations`, detail, pins/notes | 134 |
| `sql` | `/sql`, history/snippets/results | 135 |
| `ecosystem` | `/ecosystem`, topology graph | 136 |
| `dashboards` | dashboard list/detail/widget CRUD | 137 |
| `services` | service list/detail/RED/exemplars | 138 |
| `issues` | issue list/detail/status/context | 139 |
| `runs` | run list/detail/live/bundle | 140 |
| `logs` | log search/context/saved/live | 141 |
| `traces` | trace list/detail/compare/links | 142 |
| `runtime-metrics`, `story`, `time-range`, shared page header | Route-less cross-feature capability facades | 149 |
| `overview` | `/`, trends, movers, onboarding and recent entities | 150 |
| `app`, `layout`, `app-status`, `quick-navigation` | Root composition, shell/nav/theme/fallbacks, shell data | 143 |
| Whole UI graph | Final ownership/ratchet/handoff/documentation verification only | 151 |

Before this plan completes, classify every handwritten file currently under
`routes`, `components`, `hooks`, and `lib` as one feature above or exact
`app`, `layout`, `domain`, `platform`, `shared`, test-support, or generator
ownership. The architecture gate fails unclassified files.

### Shared and lower-layer promotion rules

- `domain` contains only cross-feature Parallax concepts with no framework,
  browser, network, persistence, or React dependency. Feature-specific concepts
  stay in the feature model.
- `platform` contains technical adapters such as GraphQL transport/envelope,
  SSE/EventSource, browser storage, visibility, clock, downloads, and runtime
  environment selection. It contains no feature presentation. This plan's
  extraction is behavior-preserving and provisional: Plan 152 hardens GraphQL
  generation/transport, and Plan 153 hardens every non-GraphQL external value.
- `shared` is product-neutral. Promotion requires at least two independent
  consumers, a cohesive responsibility, no Parallax domain name, and no upper
  import. It is not a dumping ground for unresolved ownership.
- `components/ui` and `lib/utils.ts` remain where `components.json` expects them.
  shadcn primitives are generator-owned; product composition is not.

## TypeScript Module Rules

1. Use kebab-case filenames, named exports, PascalCase components/types,
   `use*` hook names, and `*Schema` runtime schemas.
2. Feature `index.ts` files contain explicit value/type exports only. Handwritten
   `export *` and deep feature imports are forbidden.
3. External data enters as `unknown`, is parsed once, then mapped once to a
   domain value. Plan 152 supplies generated GraphQL operation contracts; Plan
   153 supplies SSE/search/storage/environment/cross-window mechanisms. Plan
   128 supplies static compiler and lint safety only. Do not duplicate wire/
   domain shapes or trust generic JSON with `as T`.
4. Expected failure uses a discriminated Result-shaped union. Transport and
   framework edges may throw `Error`, then map once. Error text is not control
   flow.
5. Async/UI states use discriminated unions and exhaustive `never` checks, not
   independent booleans that represent impossible combinations.
6. Prefer readonly values and pure functions. Use a class only for a real
   lifecycle or invariant-bearing mutable identity, never class-per-file
   ceremony for stateless logic.
7. More than three related parameters become a readonly named input object.
   Behavior-changing booleans become a discriminated option.
8. Use `import type`, `satisfies`, and `as const` where inference benefits. Do
   not solve moves with `any`, broad assertions, non-null assertions, or
   suppressions.
9. New catch-all `utils.ts`, `helpers.ts`, `types.ts`, and `common.ts` names are
   forbidden. Name modules by responsibility.
10. `.server.ts` and `.client.ts` mark runtime-specific boundaries when needed;
    server-only transitive code can never enter a client chunk.

## Test Placement Rules

- Test bodies never share a production file.
- `tests/` is the only final source-owned test directory name; do not retain
  both `tests/` and `__tests__/`.
- Feature/layer tests live below their owner. Route contracts live only in
  `routes/tests/` and cannot import private route components.
- `src/test/` contains setup/builders only. `ui/tests/harness/` tests that
  infrastructure; `ui/tests/e2e/` contains black-box Playwright code only.
- Every test has a stable `ui/test-matrix.json` owner from plan 129.

## Required Templates

### Feature facade

```ts
export { InvestigationsPage } from "./components/investigations-page"
export { loadInvestigations } from "./api/load-investigations"
export type { Investigation } from "./model/investigation"
```

### Boundary-to-domain adapter

```ts
import { executeGraphqlOperation } from "@/platform/graphql/execute-graphql-operation"
import {
  investigationListDocument,
  investigationListSchema,
} from "./investigation-list.generated"
import { toInvestigation } from "../model/investigation"

export async function loadInvestigations() {
  const value = await executeGraphqlOperation({
    document: investigationListDocument,
    resultSchema: investigationListSchema,
    variables: {},
  })
  return value.investigations.map(toInvestigation)
}
```

The generated document/schema and transport come from Plan 152. Non-GraphQL
adapters use Plan 153's equivalent unknown-first result contract. This adapter
does not build a raw query, cast JSON, or duplicate cache behavior.

### Exhaustive state

```ts
type LoadState<T> =
  | { readonly kind: "idle" }
  | { readonly kind: "loading" }
  | { readonly kind: "ready"; readonly value: T }
  | { readonly kind: "failed"; readonly error: InvestigationError }

export function stateLabel(state: LoadState<unknown>): string {
  switch (state.kind) {
    case "idle":
      return "Idle"
    case "loading":
      return "Loading"
    case "ready":
      return "Ready"
    case "failed":
      return state.error.message
    default: {
      const unreachable: never = state
      return unreachable
    }
  }
}
```

### Thin route adapter

```tsx
import { createFileRoute } from "@tanstack/react-router"
import {
  InvestigationsPage,
  loadInvestigations,
} from "@/features/investigations"

export const Route = createFileRoute("/investigations/")({
  loader: loadInvestigations,
  component: InvestigationsRoute,
})

function InvestigationsRoute() {
  return <InvestigationsPage investigations={Route.useLoaderData()} />
}
```

Routes may retain route/search/loader/boundary/composition code and export only
`Route`. Feature plans instantiate this template with current contracts.

## Commands

| Purpose | Command | Expected result |
|---------|---------|-----------------|
| Architecture | `cargo xtask arch` | no unknown edge, cycle, deep import, or unclassified file |
| UI policy | `cargo xtask policy --only ui.architecture` | layer/facade/runtime/test topology passes |
| Ratchets | `cargo xtask policy --only ui.ratchets` | no new size/export/test exception |
| Format | `cd ui && bun run check` | exit 0 |
| Lint | `cd ui && bun run lint` | zero warnings |
| Typecheck | `cd ui && bun run typecheck` | exit 0 |
| Tests | `cd ui && bun run --bun test:ci` | all tests pass, no unexpected diagnostic |
| Build | `cd ui && bun run build` | exit 0, generated tree current |
| Fast aggregate | `cargo xtask ci --fast` | exit 0 |

## Scope

In scope:

- Lower-layer modules under `domain`, `platform`, `shared`, and test support,
  plus enforceable placement contracts for the later `app`/`layout` owners.
- Complete current/destination ownership ledger and feature dependency matrix.
- Oxc-backed import/facade/runtime/test topology and ratchet configuration.
- Behavior-preserving provisional extraction of generic GraphQL/SSE/browser/
  runtime adapters and truly cross-feature pure concepts. Decoded hardening is
  exclusively Plans 152 and 153.
- `ui/AGENTS.md` and `PROJECT_STRUCTURE.md` placement/navigation rules.

Out of scope:

- Product feature/route moves and route-less capability migration (plans
  134-143, 149, and 150).
- Playwright implementation (plans 132 and 144-146).
- Query/cache/live/bundle behavior (plans 133, 147, and 148).
- Toolchain/compiler/lint/format selection (plans 128, 130, and 131).
- Visual redesign, product/backend contract changes, Node, foreign package
  managers, internal npm packages, or manual generated/shadcn primitive edits.

## Git Workflow

- Stay on the single active branch; do not create a branch or PR.
- Land the ledger/policy, lower-layer extraction, and durable docs as separate
  green changes.
- Use Conventional Commits, DCO, and exactly one agent-product trailer.
- Push every durable green update.

## Steps

### Step 0: Prove prerequisites

Run the command table. Confirm Plan 128's static compiler/lint contracts, Plan
129's green forced-Bun harness/matrix, and Plan 095's Oxc parser/resolver.
Capture current route exports, imports, file/function budgets, provisional
external-boundary owners, and generated/client boundaries.

**Verify**: every prerequisite command passes at the same commit.

### Step 1: Build the complete ownership ledger

Inventory every handwritten `ui/src` file with current path, current consumers,
target layer/feature, public facade requirement, test owner, generated/runtime
classification, and plan 134-143, 149, or 150 migration owner. Resolve duplicate
types and generic bucket ambiguity in the ledger before code moves.

Store machine ownership in plan 095's single typed `ratchet.toml` policy source.
Document the human feature catalog and placement decision tree in
`ui/AGENTS.md`/`PROJECT_STRUCTURE.md`.

**Verify**: removing or adding an unclassified file fails with exact file/line,
rule ID, destination guidance, and rerun command.

### Step 2: Enforce the graph before migration

Implement fixtures for every dependency-table row, feature facade/deep import,
type-only/dynamic/reexport path, generated reverse edge, source-test/harness/E2E
topology, and `.server`/`.client` reachability. Handwritten wildcard facades,
route-to-route implementation imports, production-to-test imports, and cycles
fail closed.

Baseline only exact current violations with owner and removal plan 134-143,
149, or 150.
Rows are shrink-only and cannot authorize new callers.

**Verify**: every intentional negative fixture fails and current source produces
only the exact migration ledger.

### Step 3: Extract platform, domain, shared, and test foundations

Move only lower-layer responsibilities used by several future feature plans:

- current GraphQL transport/envelope and cache-preserving adapter, provisionally
  and without claiming generated/runtime safety;
- SSE/EventSource, browser storage, visibility, clock, download, and runtime
  environment adapters, provisionally and without product decoders;
- cross-feature telemetry range/time/identity concepts with no framework;
- product-neutral shared components/hooks/pure utilities; and
- deterministic test builders from plan 129.

Preserve signatures and behavior. Record the exact GraphQL handoff to Plan 152
and SSE/search/storage/environment/cross-window handoffs to Plan 153. Temporary
old-path reexports are allowed only with one removal plan, no new callers, and
an exact expiry. Keep shadcn-owned paths unchanged.

**Verify**: request/result/cache/SSE behavior parity, exact 152/153 handoffs,
focused tests, architecture, typecheck, full tests, and build pass after each
extraction.

### Step 4: Publish feature migration contracts

For plans 134-143, 149, and 150, freeze exact source paths, target tree, allowed
facade edges, test-matrix IDs, ratchet rows, and route terminal criteria. Plan
149 must land before its consumers; the feature plans may otherwise run in
parallel only when their write sets and facade edges are disjoint. Plan 151 has
no product-move write set and only verifies the completed graph.

Provide a machine policy that rejects a feature move lacking its declared plan
owner or touching another active feature's write set without coordination.

**Verify**: all plan-owned current paths are assigned exactly once and no
source/destination overlap is unexplained.

### Step 5: Make the foundation durable

Update `ui/AGENTS.md` with the tree, graph, module/test rules, templates, feature
catalog, placement decision table, and exact commands. Update
`PROJECT_STRUCTURE.md` with directory ownership. Re-run the ledger against the
live tree and remove completed foundation compatibility exports.

Record `tsc --extendedDiagnostics` as a baseline; do not introduce project
references without a separate measured plan.

**Verify**: full command table passes twice from clean state and
`git diff --check` is clean.

## Test Plan

- Oxc resolver fixtures for every layer/facade/alias/type-only/dynamic/generated/
  runtime/test edge and cycle.
- Ownership-ledger unknown/duplicate/stale/overlap/removal-plan fixtures.
- File/function/export/test topology and temporary compatibility ratchets.
- Provisional GraphQL/SSE/browser adapter behavior-parity tests and exact
  Plan-152/153 ownership-handoff fixtures.
- Server-only client bundle negative fixtures.
- Durable placement documentation and machine-policy consistency tests.

## Done Criteria

- [ ] Every current handwritten UI file has one current owner, target owner, and
  separate migration plan.
- [ ] The canonical tree and closed graph are machine-enforced with complete
  positive/negative fixtures.
- [ ] Platform/domain/shared/test foundations exist without product behavior
  changes or catch-all buckets; route-less capabilities, overview, and app/
  layout remain assigned to plans 149, 150, and 143 respectively.
- [ ] shadcn/generated ownership remains explicit and unchanged by hand.
- [ ] Feature facade, thin-route, strict type, and separated-test templates are
  durable and executable.
- [ ] `ui/AGENTS.md`, `PROJECT_STRUCTURE.md`, and policy give a new human/agent
  one unambiguous placement answer.
- [ ] No internal package/project-reference/Node/foreign-tool architecture was
  introduced.
- [ ] Architecture, ratchet, format, lint, typecheck, forced-Bun tests, build,
  and fast aggregate pass twice.

## STOP Conditions

Stop and report if:

- Plan 128's static-safety contracts or Plan 129's forced-Bun matrix are
  missing/red;
- a lower layer requires a forbidden upper import and composition cannot solve
  it;
- extraction changes a request, response, cache, SSE, URL, rendering, or user
  behavior;
- an ambiguity can be resolved only with a new generic bucket or broad graph
  exception;
- generated/shadcn ownership requires manual implementation edits;
- a client chunk reaches server-only code; or
- a required gate fails twice after a reasonable correction.

## Maintenance And Removal

New files must select an existing ledger owner or update the feature catalog,
graph, tests, and policy in the same change. Reviewers reject deep feature
imports, wildcard facades, generic buckets, hidden runtime edges, and parallel
plans with overlapping write sets.

Delete this plan and its README row only after the complete ledger, enforced
graph, provisional lower-layer foundations, durable templates/docs, exact
Plan-152/153 handoffs, and every command above are green. Plans 152 and 153 then
harden runtime boundaries, Plan 149 establishes shared capability facades,
plans 134-143 and 150 own product movement, and Plan 151 owns final proof only.
