# Plan 038: One time-window behavior everywhere — shared URL-state helper, preserved on drilldowns, runs list scoped

> **Executor instructions**: Follow step by step; run every verification. On
> any STOP condition, stop and report. When done, update the status row in
> `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat 408be17..HEAD -- ui/src/routes ui/src/lib/range.ts`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: LOW
- **Depends on**: plans/035 (calendar wiring; this plan makes its output
  survive navigation). Can start after 035's Step 2.
- **Category**: bug
- **Planned at**: commit `408be17`, 2026-07-07

## Why this matters

The console currently has **three divergent time-range behaviors**, two of
them buggy: (a) on `/traces` and `/logs`, picking a *preset* writes concrete
`from`/`to` into the URL, and the resolver prefers `from&&to` — so "Last 24h"
silently freezes into a static window that never advances; (b) on
`/services`, `/services/$service`, `/issues`, `/dashboards/$id`, only the
preset key is written and `from`/`to` are dropped — so custom windows are
impossible there and an active custom window is destroyed by any range
interaction; (c) cross-route drilldown links (service → its traces) carry no
range at all, resetting investigations to the default 24h. Time-scoped
investigation is the product's core loop; it must behave identically on every
route and survive every pivot.

## Current state

All excerpts verified at commit `408be17`.

- `ui/src/lib/range.ts` — the shared model. `resolveRangeSearch` prefers
  explicit bounds over the preset key:

  ```ts
  // range.ts:49-58
  if (parsed.data.from && parsed.data.to) {
    try {
      if (BigInt(parsed.data.from) < BigInt(parsed.data.to)) {
        return customRange(parsed.data.from, parsed.data.to)
      }
    } catch { ... }
  }
  return resolvePreset(parsed.data.range, now)
  ```

- **The one correct pattern** (private to the overview route):

  ```ts
  // ui/src/routes/index.tsx:237-242
  function updateRangeSearch(range: ResolvedRange) {
    if (RANGE_PRESETS.some((preset) => preset.key === range.key)) {
      return { range: range.key, from: undefined, to: undefined }
    }
    return { range: "custom", from: range.fromNanos, to: range.toNanos }
  }
  ```

- **Bug (a) — presets frozen into custom windows**:

  ```ts
  // ui/src/routes/traces.index.tsx:411-420
  <RangePicker
    value={range}
    onChange={(next) =>
      update({
        range: next.key,
        from: next.fromNanos,
        to: next.toNanos,
      })
    }
  />
  ```

  Same shape in `ui/src/routes/logs.tsx:264-266` (`setRange`). Because
  `resolveRangeSearch` prefers `from&&to`, a preset pick pins static bounds
  (and the label falls back to a date-span, not "Last 24h").

- **Bug (b) — custom impossible / destroyed**:
  - `ui/src/routes/services.tsx:231` —
    `onSearch({ range: next.key, from: undefined, to: undefined })`
  - `ui/src/routes/issues.index.tsx:221` — identical
  - `ui/src/routes/services.$service.tsx:244-248` —
    `navigate({ search: { range: next.key } })`
  - `ui/src/routes/dashboards.$dashboardId.tsx:195` —
    `navigate({ search: { range: next.key } })`

- **Bug (c) — drilldowns drop the window**:

  ```tsx
  // ui/src/routes/services.tsx:339-343 (spans cell), :355-359 (errors cell)
  <Link
    to="/traces"
    search={{ service: row.name }}
    ...
  ```

  (the errors variant adds `errors: true`); the service-name link at `:331`
  region likewise carries at most the preset key. No `from`/`to`/`range`
  forwarding anywhere in these links.

- **Runs list has no time scoping at all**:

  ```ts
  // ui/src/routes/runs.index.tsx:153-179
  loader: async () => {
    const { runs, observedRuns } = await graphql<...>(`
      { runs { runId command status ... } observedRuns { runId service firstNanos lastNanos ... } }
    `)
    return { rows: mergeRuns(runs, observedRuns) } ...
  ```

  No `RangePicker` in its `PageHeader` (`:228` region), unbounded fetch, and
  its search schema (`:140-152` region) validates only `q`/`status`. The
  `runs`/`observedRuns` GraphQL fields take no time args (checked against the
  schema in `crates/parallax-api/src/lib.rs` — do NOT add API args in this
  plan; scope the rows client-side).

- Every list route already validates search with `rangeSearchSchema.parse`
  (or a superset) and resolves via `resolveRangeSearch` — e.g.
  `ui/src/routes/index.tsx:110-116`. `stepSecondsForRange` lives in
  `index.tsx:129-133` and is imported by other routes (check importers before
  moving anything).

- Conventions: strict TS; TanStack Router file routes; search params are the
  URL state; Bun tooling.

## Commands you will need

| Purpose | Command (from `ui/`) | Expected |
|---------|----------------------|----------|
| Typecheck | `bun run typecheck` | exit 0 |
| Lint | `bun run lint` | exit 0 |
| Tests | `bun run test` | all pass |
| Build | `bun run build` | exit 0 |

## Scope

**In scope**:
- `ui/src/lib/range.ts` (add the shared helper + a `rangeLinkSearch` helper)
- `ui/src/routes/index.tsx` (swap to shared helper; keep behavior)
- `ui/src/routes/traces.index.tsx`, `ui/src/routes/logs.tsx` (fix bug a)
- `ui/src/routes/services.tsx`, `ui/src/routes/issues.index.tsx`,
  `ui/src/routes/services.$service.tsx`,
  `ui/src/routes/dashboards.$dashboardId.tsx` (fix bug b)
- Cross-route `<Link search={...}>` sites on `services.tsx`,
  `services.$service.tsx`, `issues.index.tsx` (fix bug c — grep
  `to="/traces"`, `to="/logs"`, `to="/issues"` in routes and forward range)
- `ui/src/routes/runs.index.tsx` (RangePicker + client-side scoping)
- `ui/src/lib/__tests__/` or colocated test for the helper

**Out of scope**:
- Adding time args to the `runs`/`observedRuns` GraphQL fields (defer; note
  in Maintenance).
- New pivot links that don't exist yet (plan 039 adds those — it must use
  this plan's helper).
- The calendar control itself (plan 035).
- `dashboards.index.tsx` (no range picker there today; leave).

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one

## Steps

### Step 1: Move the canonical helper into `lib/range.ts`

Add to `ui/src/lib/range.ts` (exported):

```ts
/** URL search-param patch for a range change: presets clear explicit bounds
 * (so the window stays relative), custom pins them. */
export function updateRangeSearch(range: ResolvedRange): {
  range: string
  from: string | undefined
  to: string | undefined
} {
  if (RANGE_PRESETS.some((preset) => preset.key === range.key)) {
    return { range: range.key, from: undefined, to: undefined }
  }
  return { range: "custom", from: range.fromNanos, to: range.toNanos }
}

/** Range portion of a cross-route link's search params — preserves the
 * active window (preset key stays relative; custom carries bounds). */
export function rangeLinkSearch(range: ResolvedRange): {
  range?: string
  from?: string
  to?: string
} {
  return updateRangeSearch(range)
}
```

Delete the private copy in `ui/src/routes/index.tsx:237-242` and import from
`@/lib/range`.

**Verify**: `bun run typecheck` → exit 0.

### Step 2: Fix bug (a) on /traces and /logs

Replace the `onChange`/`setRange` bodies with
`update(updateRangeSearch(next))` (traces) and
`update(updateRangeSearch(next))` (logs). Do NOT touch the logs histogram
drag-to-window path — it constructs a genuine custom window and must keep
pinning `from`/`to` (it can call `updateRangeSearch(customRange(...))`).

**Verify**: `bun run typecheck` → exit 0. Behavior check (dev server or the
route test): navigating to `/traces` then clicking "Last hour" leaves URL
with `range=1h` and **no** `from`/`to` params.

### Step 3: Fix bug (b) on the four preset-only routes

Swap each site listed in Current state to `updateRangeSearch(next)`:
- `services.tsx:231`, `issues.index.tsx:221` (their `onSearch` patch)
- `services.$service.tsx:244-248`, `dashboards.$dashboardId.tsx:195`
  (`navigate({ search: (current) => ({ ...current, ...updateRangeSearch(next) }) })`
  — preserve other params like `q`/`status`; check each route's existing
  navigate pattern and keep it).

Confirm each of these routes' `validateSearch` already accepts `from`/`to`
(they all use `rangeSearchSchema` or a superset — verify per route; if one
validates a narrower schema, extend it with the two optional fields).

**Verify**: `bun run typecheck && bun run lint` → exit 0;
`rtk grep -rn "range: next.key" ui/src/routes/` → **no matches**.

### Step 4: Fix bug (c) — forward the window on drilldown links

On `services.tsx` (both `/traces` links + the service-name link),
`services.$service.tsx` (any list links), `issues.index.tsx` (trace/detail
links if present): spread the active range into the link search:

```tsx
<Link to="/traces" search={{ service: row.name, ...rangeLinkSearch(range) }}>
```

Sweep with `rtk grep -n 'to="/traces"\|to="/logs"\|to="/issues"' ui/src/routes/*.tsx`
and update every list/detail drilldown that has an active `range` in
component scope. Links from pages **without** a range picker (e.g. trace
detail back-links) stay untouched.

**Verify**: `bun run typecheck` → exit 0; manual or test: on `/services` with
`?range=1h`, the spans-count link now navigates to
`/traces?service=X&range=1h`.

### Step 5: Runs list gets the picker + client-side scoping

In `ui/src/routes/runs.index.tsx`:
1. Extend `validateSearch` with `rangeSearchSchema` fields (merge into the
   existing `q`/`status` result object).
2. Resolve `range` in the component via `resolveRangeSearch(search)`; add
   `<RangePicker value={range} onChange={(next) => navigate({ search: (c) => ({ ...c, ...updateRangeSearch(next) }) })} />`
   to the `PageHeader` `actions` (match `services.tsx`'s header for
   placement).
3. Scope rows client-side: keep a run when its activity overlaps the window —
   `startedAtNanos`/`endedAtNanos` (RunRecord) or `firstNanos`/`lastNanos`
   (ObservedRun) intersect `[fromNanos, toNanos]`. Implement inside
   `mergeRuns`'s caller or as a `filterRunsByRange(rows, range)` helper next
   to `mergeRuns`; handle null `endedAtNanos` (still-running → treat as
   now/open-ended).
4. Show the total: reuse the existing count text if present, rendering
   "N of M runs in window" when filtered.

**Verify**: `bun run typecheck && bun run test` → exit 0 (mergeRuns has
existing tests? check `ui/src/routes/__tests__/`; add the filter test per
Test plan regardless).

## Test plan

- `ui/src/lib/__tests__/range.test.ts` (create or extend if exists): 
  `updateRangeSearch` — preset key clears from/to; custom pins both;
  `rangeLinkSearch` mirrors it.
- Runs filter: unit test for `filterRunsByRange` — run fully inside window
  kept; run ended before window dropped; still-running (null end) kept when
  started before window end.
- Pattern: existing pure-function tests in
  `ui/src/components/console/__tests__/kit.test.tsx`.
- `bun run test` → all pass with the new files.

## Done criteria

ALL must hold (from `ui/`):

- [ ] `bun run typecheck`, `bun run lint`, `bun run test`, `bun run build`
      all exit 0
- [ ] `rtk grep -rn "range: next.key" ui/src/routes/` → no matches
- [ ] `rtk grep -rn "function updateRangeSearch" ui/src` → exactly one match,
      in `lib/range.ts`
- [ ] `/traces` preset pick leaves no `from`/`to` in URL (test or recorded
      manual check)
- [ ] Services→traces drilldown links carry range params (grep shows
      `rangeLinkSearch` used in `services.tsx`)
- [ ] Runs list renders a `RangePicker` and filters by window
- [ ] `plans/README.md` status row updated

## STOP conditions

- Any route's `validateSearch` rejects `from`/`to` in a way that needs schema
  surgery beyond adding two optional strings — report it.
- The logs histogram drag path regresses (its custom window must still pin
  bounds) — if Step 2 breaks it, stop and report rather than special-casing
  deeper.
- Runs data lacks usable timestamps for filtering (nulls dominate) — report;
  the API-side range args become the prerequisite.

## Maintenance notes

- Plan 039 (pivot sweep) must use `rangeLinkSearch` on every link it adds.
- Deferred: server-side time args on `runs`/`observedRuns` (client filtering
  is correct but still fetches everything; when run volume grows, add
  `fromNanos`/`toNanos` args to those resolvers and move the filter).
- Reviewer: check no route lost its non-range search params (the spread
  pattern in Step 3), and that "previous window" comparisons on the overview
  (`previousRange`, `index.tsx:118-127`) still work with custom windows.
