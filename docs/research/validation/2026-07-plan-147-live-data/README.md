# Plan 147 — live data boundedness (IN PROGRESS residual @live e2e, 2026-07-17)

## Landed

| Item | Result |
| --- | --- |
| `createBoundedFrameBuffer` | max 2000 default; drop-oldest overflow; diagnostics |
| SSE controller | bounded buffer; generation invalidation; no unbounded `T[]` |
| `stream-state` reducer | `idle \| connecting \| open \| reconnecting \| error` |
| `mergeLiveLogs` / `mergeLiveSpans` | pure, non-mutating, identity dedupe, capacity |
| Feature stream schemas | `logStreamBatchDecoder`, `spanStreamBatchDecoder` (unknown-first) |
| Call sites | logs page, traces list live, invocation hub logs+spans — **decoder only** |
| Legacy `parse` path | **deleted** from `useLiveStream` |
| Unit/perf | merge + buffer + controller + hook + schema + 10k+1k p95 gate green |
| Script | `bun run perf:live` |

## Commands (verified this head)

```text
cd ui && bun run perf:live
cd ui && bun run typecheck
cd ui && bun run --bun test:ci -- src/features/logs/tests/api src/features/traces/tests/api src/platform/sse src/features/logs/tests/model src/features/traces/tests/model
```

## Residual (keeps plan active)

- Feature-owned `@live` full-stack Playwright specs for burst/capacity/filter-generation
  (`logs-live-performance.spec.ts`, `traces-live-performance.spec.ts`,
  `runs-live-performance.spec.ts` matrix rows)
- Optional: xtask `ui.live-data` ownership policy when graph rules are extended
