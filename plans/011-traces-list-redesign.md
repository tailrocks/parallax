# Plan 011: Traces list — reference table grammar, URL state, live tail kept

> **Executor instructions**: Step by step; verify each; STOP conditions
> binding; update `plans/README.md` when done.
>
> **Reference project**: operator-designated local reference console — name
> NEVER in this repo. `REF_ROOT="$(cat plans/.reference-root)"` (STOP if
> missing), pinned at its commit `9f028d7`. Leak check before commits
> (plans/README.md §Reference).
>
> **Drift check (run first)**: `git diff --stat ad9115d..HEAD -- ui/src/routes/traces.index.tsx`
> Plans 005-008 and 010 must be DONE. Check `plans/README.md` for the plan-010
> Step-6 naming decision (`traces` returning `TraceList` vs new `tracesPage`).

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED
- **Depends on**: plans/005-008, plans/010
- **Category**: tech-debt (UX redesign)
- **Planned at**: commit `ad9115d`, 2026-07-03

## Why this matters

Traces is the entry point of the investigation loop. Today the page opens with a 4-tile KPI
strip of filter echoes, keeps all filter state in `useState` (not shareable), paginates by a
manual "Load older" button, only the root-name cell is clickable, and durations/times are
plain text. The reference list grammar: toolbar (search + filter selects + errors-only chip +
count + range picker) → dense interactive table (whole-row click, error left-accent bar,
heat-tinted duration, self-ticking "when") → numbered pagination. Live tail (SSE) is a
Parallax capability the reference lacks — keep it, restyled.

## Current state

`ui/src/routes/traces.index.tsx` (verified at `ad9115d`):
- All state client-side `useState` (`:100-107`): traces, spans (live), lookup, service,
  errorsOnly, minDuration, query, rangeMinutes, refreshSeconds; `live = refreshSeconds ===
  -1` (`:107`).
- `load()` builds the GraphQL call inline (`:109-146`): `services + traces(...)` with
  `limit: 100`; `loadOlder()` cursors with `toNanos = oldest.startNanos - 1` (`:150-181`);
  poll timer (`:188-192`); SSE live tail `/v1/traces/stream` with per-row filter params
  (`:197-220`).
- Rendering below: 4 `KpiCard`s, filter toolbar in `.parallax-panel`, `<Table>` where only
  `rootName` links to the detail, "Load older" button, `"Loading…"` string states.
- After plan 010 the API provides sort (`START_DESC | DURATION_DESC | DURATION_ASC |
  SPAN_COUNT_DESC`), `offset`, `maxDurationMs`, and a `total` (String) — via `traces` or
  `tracesPage` (read the recorded decision).
- Kit available from plan 008 (`ui/src/components/console/*`): Toolbar, SearchInput,
  FilterSelect, ToggleChip, ClearFiltersButton, SortableHead, pageWindow, HeatCell,
  RelativeTime, EmptyState, TableSkeleton, useDelayedLoading, RangePicker +
  `resolveRangeSearch`, formatters (`formatDurationNs`, `formatTimeInRange`, `formatCount`).
- Reference to mirror: `$REF_ROOT/apps/web/src/app/(app)/traces/traces-client.tsx` —
  toolbar `:234-275`; columns `:287-335` (name flexible + numeric right-aligned fixed-width
  `w-28/w-32` sortable); row error accent `shadow-[inset_1px_0_0_0_var(--color-rose-500)]`
  (`:347-354`); pagination footer `:503-557`; skeleton/empty split "no data ever" vs "no
  matches" (`:226-231, 277-282`); PAGE_SIZE 25 (`:74`).

## Commands you will need

From `ui/`: `rtk bun run typecheck` / `lint` / `test` / `build` → exit 0. Manual: `rtk bun
run dev` + `parallax serve` running with some telemetry (use
`parallax-telemetry-playground` or any OTLP source) for live verification. Leak check: plan
005 table.

## Scope

**In scope**: `ui/src/routes/traces.index.tsx` (rewrite).
**Out of scope**: trace detail (plan 012); the SSE server; `lib/api.ts` transport; kit files
(extend only via plan-008 conventions if something small is missing — note it).

## Git workflow

`main`; `feat(ui): redesign traces list` style; `git commit -s`; trailer
`Co-authored-by: Claude <noreply@anthropic.com>`; leak check first.

## Steps

### Step 1: URL state via `validateSearch`

Replace the `useState` filter set with zod-validated search params (`ui/AGENTS.md` rule 6):
`{q, service, errors, minMs, maxMs, sort, page, range, from, to, live}` — model the schema
on `issues.index.tsx:47-60` (already validateSearch-based) and use
`resolveRangeSearch` for the time window. `loaderDeps` + route `loader` fetch page data
server-side (TanStack loader), EXCEPT live mode which stays client-side. Filters/sort/page
write via `useNavigate({replace: true})` through the kit's URL-patch helper; any filter
change resets `page`.

**Verify**: typecheck; changing filters updates the URL; back/forward restores state;
pasting the URL reproduces the view.

### Step 2: Toolbar + table

Compose: `PageHeader` (icon = nav Traces item; description; actions = trace-id lookup form
(keep — spec-required trace_id entry) + refresh/live segmented control + `RangePicker`) →
`Toolbar` (SearchInput "Search root span…", service `FilterSelect` (from `services` query),
min/max-duration inputs (compact, ms), "Errors only" `ToggleChip`, `ClearFiltersButton`,
right: `formatCount(total)` traces + nothing else) → `Table`:

| Column | Treatment |
|---|---|
| Trace | flexible; root name `truncate font-medium`; sub-line `text-xs text-muted-foreground` service chip; error rows get the rose left-accent bar `shadow-[inset_1px_0_0_0_var(--color-rose-500)]` + rose Badge `errors` |
| Spans | right, `w-28`, sortable (SPAN_COUNT_DESC), `tabular-nums` |
| Duration | right, `w-32`, sortable (DURATION_DESC/ASC), `HeatCell metric="duration"` fed by all durations on the page |
| When | right, `w-32`, sortable (START_DESC default), `RelativeTime` + tooltip absolute via `formatTimeInRange` |

Whole row = `TableRow interactive` navigating to `/traces/$traceId`; inner links (service →
`/services?...` once plan 015 lands — until then omit) use `stopPropagation`. KPI strip:
**delete** (no stat cards on this page; the count lives in the toolbar).

**Verify**: typecheck; row click navigates; sort cycles desc→asc→off per column and hits the
server (`sort` search param changes; response order changes).

### Step 3: Pagination

Numbered pagination, PAGE_SIZE 25, `pageWindow` ellipsis, prev/next disabled while loading,
"Showing X–Y of N" line from `total`. Uses `offset` (plan 010). Delete `loadOlder` cursor
code.

**Verify**: page 2 URL `?page=2` returns different rows; total renders.

### Step 4: Live tail, restyled

Keep the SSE mechanics verbatim (`/v1/traces/stream`, buffer + flush interval, per-row filter
params — current `:197-220`). Present it as the reference would: when `live=1`, swap the
paginated table for a live table (same columns, no pagination) with a subtle "live" indicator
in the toolbar (pulsing emerald dot + `Live` label chip) — no `.parallax-glow-border`, no
`LiveStreamPanel` on this page. New rows prepend; cap 100.

**Verify**: with the playground emitting spans, toggling Live streams rows; switching back
restores the paginated query view.

### Step 5: States

`useDelayedLoading` + `TableSkeleton` while loading; first-load empty → `EmptyState` with an
OTLP snippet (`IconAffiliate`-family icon, "No traces yet", the OTLP endpoint from the old
empty panel); filtered-to-empty → "No matching traces" variant + Clear filters. Remove every
bare `"Loading…"` string.

**Verify**: `grep -n "Loading…" ui/src/routes/traces.index.tsx` → none. Empty DB shows the
snippet card inside the shell.

### Step 6: Gate

typecheck/lint/test/build all 0; both themes; leak check → no output.

## Test plan

Add `ui/src/routes/__tests__/traces-search.test.ts` (pure fn level): the zod search schema —
garbage tolerated, `page` resets on filter patch (test the patch helper usage), sort param
round-trip. Rendering smoke: table renders given a mocked 3-row payload (testing-library,
mock `graphql` module).

## Done criteria

- [ ] typecheck / lint / test / build exit 0
- [ ] No `KpiCard` import; no `.parallax-panel` string; no `"Loading…"`; no lucide imports
      in this route
- [ ] Filters/sort/page/range all in URL; reload reproduces view
- [ ] Whole-row click navigates; error rows show rose accent bar
- [ ] Live tail works (manual check with playground)
- [ ] Leak check → no output; `plans/README.md` row updated

## STOP conditions

- Plan 010's list query is absent or named differently than README records.
- SSE stream shape changed (rows don't match `SpanDoc` fields used).
- The loader-vs-live split fights TanStack Router (loader re-runs killing the stream) — if
  live mode can't coexist with route loaders, keep the whole page client-fetched like today
  and note the deviation.

## Maintenance notes

- Service chip should become a link to the service detail when plan 015 lands.
- HeatCell thresholds are page-local (quintiles of the visible page) by design — same as the
  reference; don't fetch global percentiles.
- If the API later adds a duration histogram for the toolbar, put it above the table as a
  brushable band (pattern in plan 014's histogram).
