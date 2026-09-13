# 2026-09-13 — service-map render parity

Status: **implementation-complete**; deterministic UI, live Playground, and
full-stack browser-mount gates cover the investigation slice. A dedicated
high-volume browser screenshot remains follow-up evidence.

## Workflow solved

A developer can read the service map without opening raw spans: dependency
kind/system, traffic volume, failure, and p50/p95 latency are visible on typed
nodes and edges.

## Implementation evidence

- Backend `ServiceNode` already returns
  `service | cli | browser | database | queue | external` plus `system`:
  `ui/graphql/schema.graphql`.
- The UI now fetches and preserves `system`; unknown backend kinds are not
  downgraded to `service`: `ui/src/features/ecosystem/api/service-map.graphql`,
  `ui/src/features/ecosystem/model/service-map.ts`.
- Edges use the existing `edgeWidthFromCalls`, low/medium/high dash bands,
  motion-safe flow animation, visible error stroke/marker, and
  `calls · error rate · p50 · p95` labels:
  `ui/src/features/ecosystem/components/ecosystem-graph.tsx`.
- The accessible investigation legend covers all node kinds, healthy/error
  edges, and traffic width/animation.
- Node cards retain metrics under long names with truncation and a taller
  deterministic layout box: `service-map-layout-engine.ts`.

## Verification

Focused UI gate:

```sh
rtk mise exec -- bun run test -- src/features/ecosystem/tests/model/service-map.test.ts src/features/ecosystem/tests/components/ecosystem-graph.test.tsx
```

Playground contract/workload:

```sh
rtk mise exec -- cargo test -p playground-cli shapes::tests::eco
rtk mise exec -- cargo test -p playground-cli scenario_runner
```

The new `ecosystem:service_map` scenario reuses live commerce and browser
traffic, emits deterministic PostgreSQL/RabbitMQ/HTTP dependency spans, and
asserts the six node kinds plus high/medium/healthy/error edges through
Parallax GraphQL.

Live run result: `six node kinds; database=100 queue=10 healthy/error external
edges`; the embedded Playwright commerce/browser journey passed.

Parallax full-stack browser gate:

```sh
rtk mise exec -- bun run test:browser:full -- --grep ecosystem
```

## Remaining gaps

- Edge clicks filter by source only; a target/dependency-aware trace filter and
  issue/log/metric pane remain open.
- HyperDX-style live map interaction polish remains competitive, not P0.
- A dedicated high-volume ecosystem screenshot should accompany the next
  validation sweep.
