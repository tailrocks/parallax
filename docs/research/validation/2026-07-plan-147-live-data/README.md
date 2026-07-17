# Plan 147 — live data boundedness (IN PROGRESS, 2026-07-17)

## Landed this slice

| Item | Result |
| --- | --- |
| `createBoundedFrameBuffer` | max 2000 default; drop-oldest overflow; diagnostics |
| SSE controller | uses bounded buffer (no unbounded `T[]` push) |
| `mergeLiveLogs` | pure, non-mutating, identity dedupe, capacity |
| `mergeLiveSpans` | spanId identity, pure order |
| Call sites | logs page, traces list live, invocation hub logs+spans |
| Unit tests | merge + buffer green |

## Residual (keeps plan active)

- Feature-owned `@live` full-stack specs / performance harness (`perf:live`)
- RuntimeDecoder cutover (legacy `parse` still present)
- Status union with reconnect/degraded variants
- Heap/timing ratchets and 10k+1k merge p95 gate
- Delete any remaining legacy parse/merge paths after all callers migrate
