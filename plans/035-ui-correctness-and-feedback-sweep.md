# Plan 035: Fix UI correctness bugs — hooks crash, dead calendar, swallowed failures, unguarded delete, fetch races

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat 408be17..HEAD -- ui/src/routes ui/src/components ui/src/lib`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: LOW
- **Depends on**: none
- **Category**: bug
- **Planned at**: commit `408be17`, 2026-07-07

## Why this matters

Five verified correctness defects undermine trust in the console: (1) trace
detail violates the Rules of Hooks and can crash when navigating between a
populated and a missing trace; (2) the custom-range calendar renders on every
page but is wired to nothing, so users cannot pick an arbitrary time window
anywhere; (3) most mutations and side-effect loads swallow failures — a failed
issue-resolve or dashboard-delete looks identical to success; (4) dashboard
delete is a one-click destructive action with no confirmation; (5) component
fetches cannot be cancelled, so polls and route changes race and stale
responses win. These are all small, independent fixes; landing them first
gives every later UI plan a stable foundation.

## Current state

All excerpts verified against commit `408be17`.

- `ui/src/routes/traces.$traceId.tsx` — trace detail page. Hooks violation:

  ```tsx
  // traces.$traceId.tsx:137-147
  const [selectedId, setSelectedId] = useState<string | null>(WHOLE_TRACE_ID)

  if (!trace || trace.spans.length === 0) {
    return (
      <EmptyState ... />
    )
  }
  ```

  then, after that early return:

  ```tsx
  // traces.$traceId.tsx:164-169
  const orderedLogs = useMemo(
    () =>
      [...logsByTrace].sort((a, b) =>
        BigInt(a.tsNanos) < BigInt(b.tsNanos) ? 1 : -1
      ),
    [logsByTrace]
  ```

  Hook count differs between the not-found render (1 hook) and the normal
  render (2+ hooks). TanStack Router reuses the component instance when only
  `$traceId` changes, so navigating populated → missing trace (or the
  reverse) throws "Rendered fewer hooks than expected".

- `ui/src/components/console/range-picker.tsx` — the global time control.
  The preset buttons call `onChange` (lines 29-45), but the calendar is dead:

  ```tsx
  // range-picker.tsx:47
  <Calendar mode="range" numberOfMonths={2} disabled={{ after: new Date() }} />
  ```

  No `selected`, no `onSelect`. `ui/src/lib/range.ts:39-41` already exports
  `customRange(fromNanos, toNanos)` returning `{ key: "custom", ... }`, and
  `resolveRangeSearch` (`range.ts:49-53`) already honors `from`/`to` search
  params, and `formatRangeLabel` (`range.ts:61-67`) already renders a
  date-span label for non-preset keys. Only the calendar wiring is missing.
  Note: `RangePicker.onChange` consumers currently write the result into URL
  search params — e.g. `ui/src/routes/logs.tsx:264-266`:

  ```tsx
  const setRange = (next: ResolvedRange) => {
    update({ range: next.key, from: next.fromNanos, to: next.toNanos })
  }
  ```

- Swallowed failures (each site lacks a `catch` that surfaces the error to
  the user; a rejected promise is silent):
  - `ui/src/routes/dashboards.index.tsx:210-213` — `remove(id)` has no
    try/catch at all.
  - `ui/src/routes/dashboards.$dashboardId.tsx:150-163` — `save` and
    `removeDashboard`; `removeDashboard` is invoked bare as
    `onClick={removeDashboard}` at `:229` (an async function as event
    handler → unhandled rejection).
  - `ui/src/routes/issues.$fingerprint.tsx:191-201` — `setStatus` uses
    `try { ... } finally { setMutating(false) }` with **no catch**.
  - `ui/src/routes/issues.$fingerprint.tsx:203-225` — `filterBucket` has no
    try at all.
  - `ui/src/routes/logs.tsx:268-291` — `loadOlder` try/finally, no catch.
  - `ui/src/routes/runs.$runId.tsx:203-217` — live poll `.then(...)` with no
    `.catch(...)`.
  - `ui/src/routes/sql.tsx:125-152` — schema-load effect (verify exact lines
    on read; the loader promise has no rejection surface).

  The repo's own good pattern to copy is `create()` in
  `ui/src/routes/dashboards.index.tsx:188-208`: `setError(null)` → try →
  catch sets `setError(err instanceof Error ? err.message : String(err))` →
  inline `<p className="text-sm text-destructive">{error}</p>` at `:263-265`.

- Destructive delete without confirm:
  - `ui/src/routes/dashboards.index.tsx:303-310` — trash button
    `onClick={() => void remove(dashboard.id)}`.
  - `ui/src/routes/dashboards.$dashboardId.tsx:226-233` — Delete button
    `onClick={removeDashboard}`.
  The repo already uses the shadcn `Dialog` family
  (`ui/src/components/ui/dialog.tsx`, used in `dashboards.index.tsx:223-278`);
  there is also an `alert-dialog.tsx` under `ui/src/components/ui/` — check
  and prefer it if present.

- Fetch cancellation / races:
  - `ui/src/lib/api.ts:8-25` — `graphql<T>(query)` takes no `AbortSignal`:

    ```ts
    export async function graphql<T>(query: string): Promise<T> {
      const response = await fetch(`${BASE}/graphql`, { ... })
    ```

  - `ui/src/components/metric-strip.tsx:53-106` — effect fetches, then
    `setPanels` in `.then` with no cancellation/ignore flag; `live` mode adds
    `setInterval(fetchPanels, 5000)` (`:104`) so overlapping in-flight
    responses can resolve out of order and a stale anchor's data can render
    after props change.
  - `ui/src/components/parallax-shell.tsx:163-177` — dashboards nav effect
    creates `new AbortController()` and returns `() => controller.abort()`
    but never passes the signal into `graphql(...)` — dead code. Contrast
    `StatusPill` (`parallax-shell.tsx:123-139`) which wires
    `signal: controller.signal` into a raw `fetch` correctly.

- Conventions: strict TypeScript (no `any`, no non-null assertions where
  avoidable); Bun-only tooling; components use shadcn/ui primitives from
  `ui/src/components/ui/`; GraphQL only through `ui/src/lib/api.ts`.

## Commands you will need

| Purpose | Command (run from `ui/`) | Expected on success |
|---------|--------------------------|---------------------|
| Install | `bun install` | exit 0 |
| Typecheck | `bun run typecheck` | exit 0 |
| Lint | `bun run lint` | exit 0 |
| Tests | `bun run test` | all pass |
| Build | `bun run build` | exit 0 |

## Scope

**In scope** (the only files you should modify):
- `ui/src/routes/traces.$traceId.tsx`
- `ui/src/components/console/range-picker.tsx`
- `ui/src/lib/api.ts` (add optional `AbortSignal` parameter only)
- `ui/src/components/metric-strip.tsx`
- `ui/src/components/parallax-shell.tsx`
- `ui/src/routes/dashboards.index.tsx`
- `ui/src/routes/dashboards.$dashboardId.tsx`
- `ui/src/routes/issues.$fingerprint.tsx`
- `ui/src/routes/logs.tsx` (the `loadOlder` catch only)
- `ui/src/routes/runs.$runId.tsx` (the poll `.catch` only)
- `ui/src/routes/sql.tsx` (the schema-effect catch only)
- test files under `ui/src/components/__tests__/`,
  `ui/src/components/console/__tests__/`, `ui/src/routes/__tests__/`

**Out of scope** (do NOT touch, even though they look related):
- Range/URL-state propagation across routes (plan 038 owns forwarding
  `from`/`to` on drilldowns — here you only wire the calendar control
  itself).
- Any GraphQL variable/codegen migration (recorded as rejected; keep the
  string-building style).
- Virtualization or render-performance work (plan 040).
- The `relativeTime` duplication in `api.ts` (plan 053).

## Git workflow

- Work directly on `main` (repo rule — see `BRANCHING.md`).
- Conventional Commits, DCO signoff, `git commit -s -m "fix(ui): ..."` +
- Push after finishing (repo rule in `AGENTS.md`).

## Steps

### Step 1: Fix the hooks violation in trace detail

In `ui/src/routes/traces.$traceId.tsx`, move the `orderedLogs` `useMemo`
(currently at `:164`) — and any other hook call that sits below the early
return at `:139` (search the component body for `use` calls; also check the
`useMemo`/`useEffect` count below line 147) — to directly after the
`useState` at `:137`, above the `if (!trace ...)` early return. `logsByTrace`
comes from the loader, so the memo is safe to compute even when `trace` is
null (`logsByTrace` defaults to an array — verify in the loader; if it can be
undefined when the trace is missing, guard with `?? []` inside the memo).

**Verify**: `bun run typecheck` → exit 0; `bun run lint` → exit 0. Then
`rtk grep -n "useMemo\|useState\|useEffect" ui/src/routes/traces.\$traceId.tsx`
→ every hook line number is smaller than the line of `if (!trace`.

### Step 2: Wire the custom-range calendar

In `ui/src/components/console/range-picker.tsx`:
1. Add local state for the in-progress selection:
   `const [draft, setDraft] = useState<DateRange | undefined>(undefined)`
   (import `DateRange` from `react-day-picker` — the type the shadcn
   `Calendar` re-exports; check `ui/src/components/ui/calendar.tsx` for the
   exact import the repo uses).
2. Pass `selected={draft}` and `onSelect={setDraft}` to the `Calendar`.
3. When `draft.from && draft.to`, call `onChange(customRange(...))` with
   `from` at 00:00:00.000 local of `draft.from` and `to` at 23:59:59.999
   local of `draft.to`, converted to nanos strings
   (`(BigInt(ms) * 1_000_000n).toString()`), then reset the draft. Import
   `customRange` from `@/lib/range`.
4. Keep the preset buttons exactly as they are.

The popover label already handles custom keys via `formatRangeLabel`.

**Verify**: `bun run typecheck && bun run lint` → exit 0. Manual check (if a
dev server is feasible): `bun run dev`, open `/logs`, pick two dates → URL
gains `from`/`to` and the label shows the date span. If no server/data is
available, add the component test in the Test plan instead — do not skip
both.

### Step 3: Add abortability to `graphql` and fix the racing consumers

1. `ui/src/lib/api.ts`: change the signature to
   `graphql<T>(query: string, init?: { signal?: AbortSignal })` and pass
   `signal: init?.signal` into `fetch`. All existing call sites stay valid.
2. `ui/src/components/metric-strip.tsx`: inside the effect, create one
   `AbortController` per fetch cycle; pass its signal into `graphql`; in the
   cleanup, abort the in-flight controller and clear the interval. Ignore
   `AbortError` rejections (`.catch((err) => { if (err?.name !== "AbortError") setPanels([]) })`).
   Additionally guard with a local `let cancelled = false` set in cleanup so
   a resolved-but-stale response never calls `setPanels`.
3. `ui/src/components/parallax-shell.tsx:163-177`: pass
   `{ signal: controller.signal }` into the `graphql` call so the existing
   abort stops being dead code.

**Verify**: `bun run typecheck && bun run test` → exit 0, existing tests
pass (shell test exists at `ui/src/components/__tests__/shell.test.tsx`).

### Step 4: Surface mutation/side-effect failures

Apply the `create()` pattern (setError → try/catch → inline destructive
text) to each site listed in Current state. Concretely:
- `dashboards.index.tsx` `remove`: wrap in try/catch; on error set the
  existing `error` state (it renders inside the dialog — for the list-card
  delete add a small `error` state rendered above the grid).
- `dashboards.$dashboardId.tsx` `save`/`removeDashboard`: add an `error`
  state + inline `<p className="text-sm text-destructive">` near the action
  buttons; change `onClick={removeDashboard}` to
  `onClick={() => void removeDashboard()}`.
- `issues.$fingerprint.tsx` `setStatus`/`filterBucket`: add catch branches
  setting an `actionError` state rendered near the status buttons.
- `logs.tsx` `loadOlder`: add catch setting an inline error near the "Load
  older" button; keep `finally { setOlderLoading(false) }`.
- `runs.$runId.tsx` poll: append `.catch(() => {})` **with a comment** that
  the poll intentionally tolerates transient failures and retries in 10s
  (this one is a deliberate swallow — the run header keeps the last data).
- `sql.tsx` schema effect: `.catch` → set the existing error state if one
  exists, else a local `schemaError` rendered where the schema tree shows.

Keep messages short: `err instanceof Error ? err.message : String(err)`.

**Verify**: `bun run typecheck && bun run lint` → exit 0. And
`rtk grep -n "onClick={removeDashboard}" ui/src/routes/dashboards.\$dashboardId.tsx`
→ no matches.

### Step 5: Confirm-gate the dashboard deletes

Wrap both delete buttons (`dashboards.index.tsx:303-310`,
`dashboards.$dashboardId.tsx:226-233`) in a confirmation: use
`ui/src/components/ui/alert-dialog.tsx` if it exists, otherwise the existing
`Dialog` with title "Delete dashboard?", the dashboard name in the
description, Cancel + Delete (destructive variant) actions. Delete proceeds
only from the confirm action.

**Verify**: `bun run typecheck && bun run lint && bun run build` → all exit 0.

## Test plan

Add tests (model after the existing structure in
`ui/src/components/console/__tests__/kit.test.tsx` and
`ui/src/routes/__tests__/-logs.test.tsx` — Bun test + Testing Library):

- `ui/src/components/console/__tests__/range-picker.test.tsx` (create):
  selecting a from/to pair on the calendar fires `onChange` with
  `key: "custom"` and `fromNanos < toNanos`; clicking a preset still fires
  the preset key.
- `ui/src/components/__tests__/metric-strip.test.tsx` (create): mock
  `graphql` (module mock); mount with anchor A, change props to anchor B
  before A's promise resolves; resolve A late → assert panels reflect B (or
  none), not A. Assert unmount aborts (mock receives an aborted signal).
- Extend a route-level test only if one already mounts these routes; do not
  build new route harnesses for Step 4 — the typecheck/lint gates plus the
  two component tests are the bar.

**Verification**: `bun run test` → all pass including the 2 new files.

## Done criteria

Machine-checkable. ALL must hold (run from `ui/`):

- [ ] `bun run typecheck` exits 0
- [ ] `bun run lint` exits 0
- [ ] `bun run test` exits 0; `range-picker.test.tsx` and
      `metric-strip.test.tsx` exist and pass
- [ ] `bun run build` exits 0
- [ ] `rtk grep -n "onSelect" ui/src/components/console/range-picker.tsx` →
      at least one match
- [ ] Every hook call in `traces.$traceId.tsx` precedes the `if (!trace`
      early return (grep check from Step 1)
- [ ] `git status` shows no modified files outside the in-scope list
- [ ] `plans/README.md` status row for 035 updated

## STOP conditions

Stop and report back (do not improvise) if:

- The excerpts above don't match the live code (drift).
- The shadcn `Calendar` in this repo does not accept `selected`/`onSelect`
  for `mode="range"` (Base UI variant may differ) — report the actual API
  instead of forcing it.
- Making `graphql` abortable breaks more than 3 existing tests — that
  signals a hidden coupling; report instead of rewriting tests wholesale.
- You find additional hook-order violations in other routes — fix ONLY
  `traces.$traceId.tsx` here; list the others in your report.

## Maintenance notes

- Plan 038 (range continuity) builds directly on the calendar wiring —
  after both land, custom windows survive cross-route drilldowns.
- Plan 040 (performance) will touch `metric-strip.tsx` again (memoizing the
  chart-data transform); keep its data flow simple here.
- Reviewer: scrutinize the local-midnight → nanos conversion in Step 2
  (off-by-one-day is the classic bug) and that no `catch` silently
  swallows non-Abort errors in Step 3.
- Deferred: toast system (repo has none; inline errors only — introducing a
  toaster is a design decision for plan 053's design-system pass).
