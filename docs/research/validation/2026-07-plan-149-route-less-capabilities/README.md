# Plan 149 — Route-less capability foundation

**Recorded:** 2026-07-17
**Scope:** Permanent owners + facades for runtime metrics, story, time-range
presentation, and product-neutral page header. Import-only consumer switches.
No Query/cache, live-algorithm, bundle, or product-feature restructuring.

## Delivered

### Domain

| Owner | Path |
|---|---|
| Story beat | `ui/src/domain/story/story-beat.ts` |
| Runtime metric | `ui/src/domain/runtime-metrics/runtime-metric.ts` |
| Time range (Plan 100) | `ui/src/domain/time-range/range.ts` (unchanged) |

`@/lib/api` re-exports `StoryBeat` / `RuntimeMetric` / `MetricPoint` for legacy
route consumers until feature migrations.

### Facades

| Capability | Public import | Exports |
|---|---|---|
| Runtime metrics | `@/features/runtime-metrics` | `MetricStrip`, `RuntimeSnapshotCard`, domain types |
| Story | `@/features/story` | `StoryTimeline`, `StoryBeat` |
| Time range | `@/features/time-range` | `RangePicker`, `ResolvedRange` |
| Page header | `@/shared/components/page-header` | `PageHeader`, `PageHeaderBack` |

### Runtime metrics internals

- `api/runtime-metrics.graphql` + generated sibling via Plan-152 codegen
- `load-runtime-metrics.ts` → `executeGraphqlOperation` (raw, non-cached)
- Scope precedence: `invocationId` wins over `service`
- `use-runtime-metrics` → Plan-153 `usePageVisible`, 5s live interval, abort
- Mapper: CPU ×100, memory bytes, tasks identity

### Consumers (import-only)

All routes that used the five legacy component paths now import facades.
Legacy files deleted:

- `ui/src/components/metric-strip.tsx`
- `ui/src/components/runtime-snapshot.tsx`
- `ui/src/components/page-header.tsx`
- `ui/src/components/console/story-timeline.tsx`
- `ui/src/components/console/range-picker.tsx`

## Verification (twice green)

```bash
cargo xtask policy --only ui.architecture
cargo xtask policy --only ui.tests
cargo xtask policy --only ui.ratchets
cd ui && bun run check && bun run lint && bun run typecheck
cd ui && bunx --bun --no-install vitest run --maxWorkers=2 \
  src/features/runtime-metrics src/features/story src/features/time-range \
  src/domain/runtime-metrics src/domain/story src/domain/time-range \
  src/shared/tests/components/page-header.test.tsx
```

Focused capability suite: **10 files / 21 tests pass**.

### Known out-of-scope reds (pre-existing on main, not Plan 149)

1. **SQL route unit test** `renders SQL keyboard hint and examples menu` —
   empty render body; fails with or without Plan 149 tree.
2. **UI production build** — TanStack import-protection denies
   `platform/sse/event-source.client` through loader→live-stream path
   (Plan 153 surface; not touched here).

## Handoff

Plans 134-142 and 150 consume only the facades above. Do not deep-import
feature internals. Plan 151 owns final zero-compatibility proof after product
moves.
