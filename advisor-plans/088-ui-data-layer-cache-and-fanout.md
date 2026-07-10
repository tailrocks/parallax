# Plan 088: Give the UI a data layer — query cache + preload reuse, collapse dashboard fan-out, bound the run-page scan, pause hidden pollers, cheap formatters, tame the issues table

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `advisor-plans/README.md` — unless a reviewer dispatched you and told you
> they maintain the index.
>
> **Drift check (run first)**: `git diff --stat df81d86..HEAD -- ui/src`
> Note: at planning time the working tree carried uncommitted edits to
> `ui/src/routes/dashboards.index.tsx`, `services.tsx`, `traces.index.tsx`,
> `ui/src/lib/trace-tree.ts` and several route tests — excerpts below are from
> that working tree. Mismatch = STOP.

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: MED (loader/cache wiring touches SSR hydration; everything else is additive)
- **Depends on**: 071 (UI correctness batch — same files), 077 (shared SSE hook — the visibility gating in Step 4 must build on its hook, not race it), 079 (query/type dedup — this plan's cache keys assume its consolidated query modules if it landed)
- **Category**: perf
- **Planned at**: commit `df81d86`, 2026-07-10

## Why this matters

Every GraphQL query the UI fires becomes 1..N GreptimeDB round-trips
server-side. The UI currently has NO data cache of any kind: the client is a
bare `fetch` wrapper, the router preloads on hover with
`defaultPreloadStaleTime: 0` — so hovering a link fetches the whole target
page and clicking fetches it AGAIN. Sibling pages re-fetch the same service
lists; a 10-widget dashboard fires 11 separate GraphQL requests; the run page
requests `runtimeSnapshot(fromNanos: "0")` — an epoch-to-now scan that
multiplies into the storage layer's per-metric fan-out; no poller or SSE
stream pauses when the tab is hidden; the issues page mounts up to 100
Recharts instances. This plan cuts store traffic at its source.

## Current state

Stack: TanStack Start + React 19, code-based routes in `ui/src/routes/`,
Tailwind v4, Bun only (`bun install/run`, never npm/pnpm). Tests: vitest
(`bun run test`), typecheck `bun run typecheck`, lint `bun run lint`, build
`bun run build`. All from `ui/`.

- `ui/src/lib/api.ts:8-32` — the entire data layer:

```ts
export async function graphql<T>(query: string, init?: { signal?: AbortSignal }): Promise<T> {
  const requestInit: RequestInit = { method: "POST", headers: {...}, body: JSON.stringify({ query }) }
  ...
  const response = await fetch(`${BASE}/graphql`, { ...requestInit })
```

  No cache, no in-flight dedup. No React Query anywhere (`useQuery`/`QueryClient`
  grep: zero hits) although `@tanstack/react-router-ssr-query` is already in
  `ui/package.json` dependencies.
- `ui/src/router.tsx:14-15` — `defaultPreload: "intent"`,
  `defaultPreloadStaleTime: 0`.
- `ui/src/routes/dashboards.$dashboardId.tsx:103-116` — per-widget fan-out:

```ts
    const data = await Promise.all(
      widgets.map(async (widget) => {
        const { metricSeries } = await graphql<{ metricSeries: Series[] }>(
          `{ metricSeries(name: "${gqlString(widget.metric)}",
               fromNanos: "${range.fromNanos}", toNanos: "${range.toNanos}", ...
```

- `ui/src/routes/runs.$runId.tsx`:
  - `:143` — `runtimeSnapshot(runId: …, fromNanos: "0", toNanos: …, stepSeconds: 5)`
    (epoch lower bound; server-side this loops every runtime metric).
  - `:153` — `bundle(runId: …) { markdown }` in the loader although the bundle
    renders behind a card/download (`:377,:466`).
  - `:237` 250 ms SSE flush interval; `:262` 10 s run-status poll; plus the
    mounted `MetricStrip` 5 s poll — three overlapping loops in live mode.
- Pollers/streams never pause: grep for `visibilitychange`/`document.hidden`
  in `ui/src` → zero hits. All `EventSource`s (`runs.$runId.tsx:213,:225`,
  `logs.tsx:296`, `traces.index.tsx:375`) run while backgrounded.
- `ui/src/lib/format.ts:72,94` — `new Intl.DateTimeFormat(...)` constructed
  per call; called per table cell / per chart point.
- `ui/src/routes/issues.index.tsx:131,345-474` — `limit: 100`, plain (non-
  virtualized) table, each row mounting `CardSparkline`
  (`ui/src/components/console/stat-card.tsx:97-125` — a full Recharts
  `AreaChart`).
- `ui/src/lib/trace-tree.ts:44-62,85-112,134-143` — `BigInt(span.tsNanos)`
  re-parsed per comparison/position; `computeWindow` copies `spans.slice(1)`
  and is called three times per trace render, once UNmemoized in
  `traces.$traceId.tsx:324` (recomputed per selection click).
- Virtualization exists and is good in `logs-table.tsx:380` and
  `trace-waterfall.tsx:110` — model on those.
- Existing route tests live in `ui/src/routes/__tests__/` (jsdom + Testing
  Library) — match their patterns.

## Commands you will need

All from `ui/`:

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Install | `rtk bun install` | exit 0, `bun.lock` only |
| Typecheck | `rtk bun run typecheck` | exit 0 |
| Tests | `rtk bun run test` | all pass |
| Lint | `rtk bun run lint` | exit 0 |
| Build | `rtk bun run build` | exit 0 |

## Scope

**In scope** (all under `ui/src/`):
- `lib/api.ts`, `router.tsx`, `lib/format.ts`, `lib/trace-tree.ts`
- `routes/dashboards.$dashboardId.tsx`, `routes/runs.$runId.tsx`,
  `routes/issues.index.tsx`, `routes/traces.$traceId.tsx`
- New: `lib/use-visible.ts` (or similar), test files for the above
- `advisor-plans/README.md` (status row)

**Out of scope**:
- The GraphQL SCHEMA (no server changes; if a batched field is missing, use
  aliases — they work today).
- `metric-strip.tsx` poll cadence (recorded deferred) — Step 4 only gates it
  on visibility via the shared hook, no cadence change.
- SSE hook internals if Plan 077 landed (build on it); if 077 not landed, do
  NOT implement its hook here — gate the existing EventSources minimally.
- Package removals (`date-fns`, `@dnd-kit/*` are unused — recorded as a Plan
  079 addendum, not here).
- `routeTree.gen.ts` (generated).

## Git workflow

Direct on `main`; Conventional Commits + `git commit -s` +
`Co-authored-by: Claude <noreply@anthropic.com>`. Commit per step.

## Steps

### Step 1: Minimal query cache + in-flight dedup + preload reuse

Smallest change that removes double-fetching (do NOT introduce React Query in
this plan — that is a larger migration; record it as follow-up):

In `lib/api.ts` add a keyed cache around `graphql()`:

```ts
const inflight = new Map<string, Promise<unknown>>()
const cache = new Map<string, { at: number; data: unknown }>()
const TTL_MS = 15_000

export async function graphqlCached<T>(query: string, init?): Promise<T> {
  const hit = cache.get(query)
  if (hit && Date.now() - hit.at < TTL_MS) return hit.data as T
  const pending = inflight.get(query)
  if (pending) return pending as Promise<T>
  const p = graphql<T>(query, init).then((data) => {
    cache.set(query, { at: Date.now(), data }); inflight.delete(query); return data
  }, (e) => { inflight.delete(query); throw e })
  inflight.set(query, p)
  return p
}
```

- Key = the query string itself (queries embed their variables — today that is
  the whole identity). Cap the cache (e.g. 100 entries, LRU-ish eviction by
  insertion order).
- Switch route LOADERS to `graphqlCached` (grep `graphql<` under
  `src/routes/`); leave imperative refreshes/pollers on raw `graphql` so
  "Refresh" always refetches — EXCEPT identical-query pollers can keep raw.
- `router.tsx`: set `defaultPreloadStaleTime: 15_000` so the hover-preload
  result is what the click uses.
- SSR note: the module-level Maps exist per-request on the server (fresh module
  instance per worker is NOT guaranteed — Bun/SSR shares modules). Guard:
  `if (typeof window === "undefined") return graphql(query, init)` at the top
  of `graphqlCached` — cache client-side only. This keeps hydration untouched.

**Verify**: `bun run typecheck && bun run test && bun run lint` → pass. Add a
unit test for `graphqlCached` (mock fetch): two awaited calls, one fetch;
expiry after TTL; server-side passthrough (`typeof window` mocked is hard in
jsdom — instead export the internal for a direct branch test or skip that
case with a comment).

### Step 2: One request per dashboard load

In `dashboards.$dashboardId.tsx:103-116`, replace the per-widget `Promise.all`
of separate `graphql()` calls with ONE aliased document:

```ts
const doc = `{ ${widgets.map((w, i) =>
  `w${i}: metricSeries(name: "${gqlString(w.metric)}", fromNanos: "…", toNanos: "…", …) { … }`
).join("\n")} }`
```

Parse `w${i}` back to widgets by index. Bound: if widgets.length > 24, chunk
into documents of 24 (GraphQL complexity limit server-side is 1000 —
`config.rs limits.graphql_max_complexity`; stay well under).

**Verify**: `bun run test` (existing dashboards tests in
`routes/__tests__/` updated to assert ONE fetch for N widgets — extend the
test that mocks `graphql`); typecheck/lint pass.

### Step 3: Bound the run page

In `runs.$runId.tsx` loader:
- `:143` — replace `fromNanos: "0"` with the run's window: the loader already
  fetches the run (`startedAtNanos`); use `startedAtNanos` (fallback: now-24h)
  as `fromNanos`. If the run query and snapshot are in one document today,
  split: fetch run first, then the rest with the bound (two round-trips
  total — still fewer store queries than the epoch scan).
- `:153` — move `bundle { markdown }` out of the loader into a lazy fetch
  triggered when the bundle card/download becomes visible (`useState` +
  `onClick`/on-mount-of-card fetch via `graphql`). Loading state: reuse the
  existing card skeleton patterns in the file.

**Verify**: `bun run test` — update the run-detail route test fixtures
(they mock the loader query; assert the new bounded `fromNanos` and that the
initial document no longer contains `bundle`). Typecheck/lint pass.

### Step 4: Visibility gating for every poller and stream

Add `lib/use-visible.ts`:

```ts
export function usePageVisible(): boolean {
  const [visible, setVisible] = useState(() => typeof document === "undefined" || !document.hidden)
  useEffect(() => {
    const onChange = () => setVisible(!document.hidden)
    document.addEventListener("visibilitychange", onChange)
    return () => document.removeEventListener("visibilitychange", onChange)
  }, [])
  return visible
}
```

Gate (pause interval + close/reopen EventSource on hidden→visible):
- `runs.$runId.tsx` 10 s poll (`:262`) + both EventSources + 250 ms flush,
- `logs.tsx` stream + flush,
- `traces.index.tsx` stream + flush,
- `metric-strip.tsx` 5 s poll (gate only — no cadence change),
- `relative-time.tsx` 15 s ticker.

If Plan 077's shared SSE hook exists, add the gating INSIDE that hook once
instead of per-route.

**Verify**: new unit test for `usePageVisible` (jsdom `document.hidden`
mock + visibilitychange dispatch). `bun run test` all pass. Manual: hide the
tab with devtools network open — polling stops; return — resumes and refreshes.

### Step 5: Cache `Intl.DateTimeFormat` instances

In `lib/format.ts`: module-level
`const formatters = new Map<string, Intl.DateTimeFormat>()` keyed by
`JSON.stringify(options) + timeZone`; `formatDateTime`/`formatTimeShort` fetch
or create. Behavior identical (same options object per call site).

**Verify**: existing format tests pass (`bun run test`); add one assertion
that repeated calls return consistent output (behavioral, not identity).

### Step 6: Issues table — virtualize + lightweight sparkline

- Virtualize the issues table body with `@tanstack/react-virtual`, modeled on
  `logs-table.tsx:380` (threshold: virtualize above ~30 rows).
- Replace the per-row `CardSparkline` (Recharts) with an inline memoized SVG
  polyline component (~20 lines: normalize points to a 0..1 box, `<svg><polyline
  points=…/></svg>`, stroke uses the existing chart CSS vars). Keep
  `CardSparkline` for the stat cards (few instances) — only the TABLE rows
  switch.

**Verify**: `bun run test` — issues route tests pass (update render
assertions if they counted DOM rows — virtualization changes off-screen DOM);
typecheck/lint/build pass.

### Step 7: trace-tree parse-once

In `lib/trace-tree.ts`: build one `Map<spanId, {start: bigint; end: bigint}>`
at entry of `buildTraceTree`/`orderSpans` and thread it through comparisons,
`computeWindow` (drop the `slice(1)` copy — index loop), and `positionPct`.
Memoize `computeWindow(spans)` at the `traces.$traceId.tsx:324` call site with
`useMemo` keyed on the spans array identity. Keep tie-breaking
(`spanId.localeCompare`) identical — the existing `trace-tree.test.ts` pins
ordering; do not weaken it.

**Verify**: `bun run test` — `lib/__tests__/trace-tree.test.ts` passes
UNCHANGED (that file pins behavior incl. the working-tree skew additions).

### Step 8: Full gates

**Verify**: from `ui/`: `rtk bun run typecheck && rtk bun run lint &&
rtk bun run test && rtk bun run build` → all exit 0.

## Test plan

- New: `graphqlCached` dedup/TTL test; `usePageVisible` test; dashboards
  single-fetch assertion; run-loader bounded-window assertion.
- Updated deliberately: dashboards/run/issues route tests where fetch counts
  or DOM shape change.
- Must pass unchanged: `trace-tree.test.ts` (behavior pin for Step 7).

## Done criteria

- [ ] `grep -n "defaultPreloadStaleTime" ui/src/router.tsx` → non-zero value
- [ ] `grep -rn "graphqlCached" ui/src/routes | wc -l` → ≥ 8 (loaders switched)
- [ ] `grep -n "fromNanos: \"0\"" ui/src/routes/runs.\$runId.tsx` → 0 matches
- [ ] `grep -rn "visibilitychange" ui/src` → ≥ 1 (the hook)
- [ ] `grep -n "new Intl.DateTimeFormat" ui/src/lib/format.ts` → inside the cached factory only
- [ ] dashboards detail: one `graphql` call for N widgets (test asserts)
- [ ] `bun run typecheck/lint/test/build` all exit 0
- [ ] `git status` clean outside in-scope list
- [ ] `advisor-plans/README.md` status row updated

## STOP conditions

Stop and report back (do not improvise) if:

- Plans 071/077/079 are mid-flight in the same files (check the index) —
  sequencing rule is 071 → 077 → 079 → this plan.
- The SSR guard in Step 1 is insufficient (evidence of cross-request cache
  hits server-side) — report; do not attempt a request-context cache here.
- Aliased dashboard queries trip the server complexity limit (GraphQL error
  mentions complexity) even after chunking to 24.
- `trace-tree.test.ts` needs ANY assertion change in Step 7.
- Closing/reopening EventSources on visibility conflicts with Plan 077's hook
  semantics (double-reconnect) — coordinate via the index.

## Maintenance notes

- The 15 s TTL is a deliberate staleness window on navigations — "Refresh"
  buttons and pollers bypass it. If users report stale panes, tune per-route
  via a TTL parameter rather than removing the cache.
- Follow-up recorded, not planned: full React Query migration
  (`@tanstack/react-router-ssr-query` is already a dependency) would subsume
  Step 1 — do it when the UI grows mutations that need invalidation.
- The SVG sparkline intentionally lacks tooltips (Recharts had them) — issues
  row click opens the detail; if product wants hover values back, that is a
  UI decision, not a regression.
- After this plan, remaining known UI perf items (recorded, deferred):
  metric-strip cadence/delta-fetch, live-tail insert-sorted buffers, Recharts
  lazy-loading out of the shell chunk.
