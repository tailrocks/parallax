# Plan 142 — Traces feature migration

**Recorded:** 2026-07-17

Move trace list/detail routes, pure models (`trace-tree`, GraphQL/RPC
reconstruction), and console waterfall/inspector components behind
`@/features/traces`. Routes export only `Route`.

## Layout

```text
ui/src/features/traces/
  index.ts                         # explicit facade
  components/
    traces-page.tsx                # list + loadTraces
    trace-detail-page.tsx          # detail + loadTraceDetail
    trace-waterfall.tsx
    trace-flamegraph.tsx
    trace-field-explorer.tsx
    trace-attribute-compare.tsx
    trace-evidence-gaps.tsx
    trace-graphql-operations.tsx
    trace-rpc-streams.tsx
    trace-span-kind.tsx
  model/
    trace-tree.ts
    graphql-operations.ts
    rpc-streams.ts
  tests/
    model/**
    components/**
ui/src/routes/traces.index.tsx     # thin Route only
ui/src/routes/traces.$traceId.tsx  # thin Route only
ui/src/routes/tests/-traces-routes.test.tsx
```

## Verification

```bash
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.tests
cargo xtask policy --only ui.ratchets
cd ui && bun run check && bun run lint && bun run typecheck
cd ui && bunx --bun --no-install vitest run --pool=forks --maxWorkers=2 \
  src/features/traces src/routes/tests/-traces-routes.test.tsx
```

All commands green (2026-07-17). 78 focused tests pass.

Browser full-stack/breadth gates close with plans 145/146. Plan-152 typed ops
for list/detail/critical/compare can tighten in a follow-up (same posture as
logs/overview).
