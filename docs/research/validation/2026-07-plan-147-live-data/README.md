# Plan 147 — live data boundedness (CLOSED 2026-07-17)

## Evidence (host macOS arm64, attach full-stack QA)

| Gate | Result |
| --- | --- |
| Platform SSE | bounded buffer, stream-state, controller, hook unit tests green |
| Feature merges | `mergeLiveLogs` / `mergeLiveSpans` pure, capacity, identity, no mutation |
| p95 10k+1k | `bun run perf:live` — **22 passed** (logs + spans + platform SSE) under 16 ms |
| Schemas | unknown-first `logStreamBatchDecoder` / `spanStreamBatchDecoder` |
| `@live` full-stack | **7 passed** via `PARALLAX_FULL_STACK_MODE=attach bun run test:browser:full -- --grep @live` |
| Matrix | `performance/live` rows for logs, traces, invocations |
| Control plane | `seed-live-log`, `seed-live-log-burst`, `seed-live-log-duplicate-pair`, `seed-live-span`, `seed-live-span-duplicate-pair` |

## Commands

```bash
cd ui
bun run perf:live
# → 22 passed

PARALLAX_FULL_STACK_MODE=attach bun run test:browser:full -- --grep @live
# → 7 passed (~38s, attach Greptime+Turso QA)
```

## `@live` cases

| Spec | IDs |
| --- | --- |
| `logs-live-performance.spec.ts` | `@pw-live-logs-burst`, `@pw-live-logs-dedup`, `@pw-live-logs-filter-reset` |
| `traces-live-performance.spec.ts` | `@pw-live-traces-identity`, `@pw-live-traces-dedup` |
| `runs-live-performance.spec.ts` | `@pw-live-runs-log`, `@pw-live-runs-span` |

`@storage` one-event transport remains in `live-transport.spec.ts` (plan 145); not duplicated.

## Landed shape

```text
ui/src/platform/sse/          # lifecycle, buffer, state, hook
ui/src/features/logs/         # schema + mergeLiveLogs
ui/src/features/traces/       # schema + mergeLiveSpans
ui/src/features/invocations/  # hub dual-stream consumers
ui/tests/e2e/full-stack/*-live-performance.spec.ts
ui/tests/e2e/support/live-performance.ts
```

## Closure

Plan 147 done criteria for typed decode, identity/capacity merges, zero hidden timers, p95 ratchets, and distinct `@live` real-stack cases are satisfied. Optional xtask `ui.live-data` ownership policy deferred to ongoing architecture policy rather than a residual plan file.
