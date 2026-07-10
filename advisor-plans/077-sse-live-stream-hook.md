# Plan 077: One SSE lifecycle — extract `useLiveStream`, wire real connection health into the Live badges

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat dbaba3c..HEAD -- ui/src/routes/logs.tsx ui/src/routes/traces.index.tsx "ui/src/routes/runs.\$runId.tsx" ui/src/components/live-stream-panel.tsx`
> Plan 071 legitimately touches `logs.tsx` first. If the SSE effects below
> have been restructured beyond 071's token guard, STOP and re-verify.

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW (additive hook, migrate one route at a time)
- **Depends on**: 071 (lands first in `logs.tsx`)
- **Category**: tech-debt / bug
- **Planned at**: commit `dbaba3c`, 2026-07-10

## Why this matters

Four EventSource lifecycles are hand-rolled across three routes — same
buffer-in-closure + 250ms flush interval + cleanup pattern, four copies. None
of them registers `onerror`/`onopen`, so a dead stream is silent, and the
"connected"/"Live" badges reflect the URL's `live` search param, not stream
health: `LiveStreamPanel` renders "connected" from a hardcoded `active` prop.
A user can watch a dead tail believing it's live. One shared hook fixes the
duplication and gives every consumer a real connection state.

## Current state

- `ui/src/routes/logs.tsx` (~`:290-319`) — pattern instance 1:

  ```ts
  useEffect(() => {
    if (!live) return
    const params = new URLSearchParams()
    // ...filters from search params...
    const source = new EventSource(`/v1/logs/stream?${params}`)
    let buffer: LogDoc[] = []
    source.onmessage = (event) => {
      try {
        const batch: unknown = JSON.parse(event.data as string)
        if (Array.isArray(batch)) buffer.push(...assignLogKeys(batch as LogDoc[]))
      } catch { /* skip malformed frames */ }
    }
    const flush = setInterval(() => {
      if (buffer.length === 0) return
      const incoming = buffer
      buffer = []
      setLogs((current) => [...incoming.reverse(), ...current].slice(0, PAGE_SIZE))
    }, 250)
    return () => { source.close(); clearInterval(flush) }
  }, [live, search.service, search.sev, search.q])
  ```

- `ui/src/routes/traces.index.tsx` (~`:365-395`) — instance 2, same shape
  (`/v1/traces/stream`, `SpanDoc[]`, `setSpans`, cap 100). No `onerror`.

- `ui/src/routes/runs.$runId.tsx` (~`:211-250`) — instances 3+4 in ONE
  effect: two EventSources (`/v1/logs/stream?run_id=...` and
  `/v1/traces/stream?run_id=...`), two buffers, one shared flush interval
  (caps 300/…). No `onerror` on either.

- `ui/src/components/live-stream-panel.tsx:36-41` — presentational only:

  ```tsx
  <Badge variant={active ? "emerald" : "secondary"}>
    {active ? (<span className="size-1.5 animate-pulse rounded-full bg-current" />) : null}
    {active ? "connected" : "idle"}
  </Badge>
  ```

  and `runs.$runId.tsx` (~`:392-398`) passes `active` (hardcoded true while
  the `live` param is set).

- The logs/traces routes key their badge off `live` (the search param), e.g.
  `logs.tsx` around `:604-611`.

- Conventions: hooks live in `ui/src/hooks/` (directory exists) or
  `ui/src/lib/` for pure logic — `ui/src/hooks` is the right home (check its
  existing naming, e.g. `use-mobile.ts` pattern under components or hooks).
  Strictest TS. Tests: vitest; lib-level tests in `ui/src/lib/__tests__/`,
  hook tests are fine under `ui/src/hooks/__tests__/` with
  `@vitest-environment jsdom` per-file pragma (follow any existing test file's
  header). All list caps and flush cadence must remain configurable per
  consumer (logs 250ms/PAGE_SIZE, traces 250ms/100, runs 250ms/300).

## Commands you will need

All from `ui/`:

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Typecheck | `rtk bun run typecheck` | exit 0 |
| Tests | `rtk bun run test` | all pass |
| Lint | `rtk bun run lint` | exit 0 |
| Build | `rtk bun run build` | exit 0 |

## Scope

**In scope** (the only files you should modify):
- `ui/src/hooks/use-live-stream.ts` (create)
- `ui/src/hooks/__tests__/use-live-stream.test.ts` (create)
- `ui/src/routes/logs.tsx`
- `ui/src/routes/traces.index.tsx`
- `ui/src/routes/runs.$runId.tsx`
- `ui/src/components/live-stream-panel.tsx` (accept a real `active`)
- `advisor-plans/README.md` (status row)

**Out of scope** (do NOT touch, even though they look related):
- The server-side SSE endpoints (`crates/parallax-server/src/live.rs`).
- Reconnection/backoff logic — EventSource auto-reconnects natively; do not
  add a custom retry layer. The hook only OBSERVES state.
- Any other polling in the routes (metric-strip etc.).

## Git workflow

- Work directly on `main` (repo rule — `BRANCHING.md`).
- Conventional Commits, DCO signoff (`git commit -s`), trailer
  `Co-authored-by: Claude <noreply@anthropic.com>`. One commit for the hook +
  one per migrated route is a clean split.

## Steps

### Step 1: Write the hook

`ui/src/hooks/use-live-stream.ts`:

```ts
import { useEffect, useRef, useState } from "react"

export type LiveStreamStatus = "idle" | "connecting" | "open" | "error"

export interface UseLiveStreamOptions<T> {
  /** Full stream URL incl. query params; null/undefined disables the stream. */
  url: string | null
  /** Parse one SSE frame's payload into items; return [] to skip. */
  parse: (data: string) => T[]
  /** Called with each flushed batch (newest-first NOT applied — caller owns ordering). */
  onBatch: (items: T[]) => void
  flushMs?: number
}

export function useLiveStream<T>({ url, parse, onBatch, flushMs = 250 }: UseLiveStreamOptions<T>): LiveStreamStatus {
  const [status, setStatus] = useState<LiveStreamStatus>("idle")
  const onBatchRef = useRef(onBatch)
  onBatchRef.current = onBatch
  const parseRef = useRef(parse)
  parseRef.current = parse

  useEffect(() => {
    if (!url) { setStatus("idle"); return }
    setStatus("connecting")
    const source = new EventSource(url)
    let buffer: T[] = []
    source.onopen = () => setStatus("open")
    source.onerror = () => setStatus("error") // EventSource retries natively; state flips back on reopen
    source.onmessage = (event) => {
      try { buffer.push(...parseRef.current(event.data as string)) }
      catch { /* skip malformed frames */ }
    }
    const flush = setInterval(() => {
      if (buffer.length === 0) return
      const incoming = buffer
      buffer = []
      onBatchRef.current(incoming)
    }, flushMs)
    return () => { source.close(); clearInterval(flush); setStatus("idle") }
  }, [url, flushMs])

  return status
}
```

Notes: `onBatch`/`parse` go through refs so consumers can pass inline
closures without retriggering the effect; the effect keys ONLY on `url` —
consumers encode all filters into the URL string (they already build
`URLSearchParams`), which preserves today's reconnect-on-filter-change
behavior.

**Verify**: `rtk bun run typecheck` → exit 0.

### Step 2: Hook tests

`ui/src/hooks/__tests__/use-live-stream.test.ts` (jsdom pragma at top; mock
`EventSource` globally — jsdom lacks it; a minimal class with
`onopen/onerror/onmessage/close` and a static registry so the test can drive
events; use vitest fake timers for the flush interval). Cases:
1. frames buffered and flushed after `flushMs` via `onBatch`
2. malformed frame skipped (parse throws) without killing the stream
3. `onerror` → status `"error"`; subsequent `onopen` → `"open"`
4. cleanup on unmount closes the source and clears the interval
5. `url: null` → status `"idle"`, no EventSource constructed

**Verify**: `rtk bunx vitest run src/hooks/__tests__/use-live-stream.test.ts`
→ 5 tests pass.

### Step 3: Migrate the three routes

One route per commit, in this order (simplest first):

1. `traces.index.tsx`: replace the effect with
   `const streamStatus = useLiveStream<SpanDoc>({ url: live ? \`/v1/traces/stream?${params}\` : null, parse, onBatch })`
   where `onBatch` does the existing
   `setSpans((current) => [...incoming.reverse(), ...current].slice(0, 100))`
   and the `setSpans([])` reset moves to a small effect keyed on the same url
   (preserve current behavior: the reset happens when filters change).
2. `logs.tsx`: same shape; `parse` wraps `assignLogKeys`; keep 071's
   generation token untouched.
3. `runs.$runId.tsx`: two `useLiveStream` calls (logs + spans); derive
   `active` for the panel as `logStatus === "open" || spanStatus === "open"`.

Then thread status into the badges:
- `LiveStreamPanel` (`live-stream-panel.tsx`): keep the `active: boolean`
  prop but pass the real derived value from runs (`:392-398` currently
  hardcodes `active`). Optionally add a third visual state for `"error"`
  (badge text "reconnecting…") — do it only if the Badge variants make it a
  ≤5-line change.
- `logs.tsx` / `traces.index.tsx` Live badges: swap the `live`-param
  condition for `streamStatus === "open"` on the "connected/Live" indicator
  text, keeping the param as the on/off intent (badge shows intent + health:
  e.g. live param on but status error → "reconnecting…" or non-emerald
  variant; match the existing Badge variants in the file).

**Verify** after EACH route: `rtk bun run typecheck` → 0; `rtk bun run test`
→ route suites (`-logs.test.tsx`, `-traces-search.test.tsx`, and any runs
tests) pass unchanged. After all three:
`grep -rn "new EventSource" ui/src/routes` → 0 matches (all go through the
hook).

### Step 4: Full gates

**Verify**: from `ui/`: `rtk bun run typecheck`, `rtk bun run lint`,
`rtk bun run test`, `rtk bun run build` → all exit 0.

## Test plan

- 5 hook unit tests (Step 2) — the EventSource mock is the only new test
  infrastructure; keep it inside the test file (do not create a shared mock
  module until a second consumer needs it).
- Existing route suites act as characterization tests for the migration; they
  must pass UNCHANGED. If a route test asserts implementation details of the
  old effect (unlikely — they render + assert DOM), report rather than
  rewriting assertions wholesale.

## Done criteria

Machine-checkable. ALL must hold:

- [ ] `ui/src/hooks/use-live-stream.ts` exists; `grep -n "onerror" ui/src/hooks/use-live-stream.ts` → ≥1
- [ ] `grep -rn "new EventSource" ui/src/routes ui/src/components` → 0 matches
- [ ] `grep -n "active$" -n "ui/src/routes/runs.\$runId.tsx"` — the hardcoded `active` prop is gone (passes a derived status)
- [ ] From `ui/`: typecheck, lint, test, build all exit 0; 5 new hook tests pass
- [ ] `git status` shows no modified files outside the in-scope list
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- A route's stream behavior differs from the hook's model in a way the
  options can't express (e.g. runs' shared flush interval matters
  behaviorally — two hooks give two intervals; that is acceptable drift, but
  if a test proves ordering coupling between the two buffers, report).
- jsdom EventSource mocking fights vitest fake timers (flaky hook tests
  after 2 attempts) — report with the failing setup.
- The Badge component lacks a sane variant for an error state and the change
  balloons past ~5 lines of presentational code.

## Maintenance notes

- Future stream consumers (ecosystem live view, metric tails) must use this
  hook; a `new EventSource` in a route is now a review-blocking smell.
- If the server later emits SSE heartbeat/retry hints, extend the hook (one
  place) rather than consumers.
- The paging generation-token pattern (Plan 071) and this hook could merge
  into a small "live data" toolkit later; deferred until a third consumer
  exists.
