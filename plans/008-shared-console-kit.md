# Plan 008: Build the shared console kit (stat cards, table toolkit, heat cells, copy, time, range, charts)

> **Executor instructions**: Step by step; verify each step; STOP conditions
> binding; update `plans/README.md` when done.
>
> **Reference project**: operator-designated local reference console — name
> must NEVER appear in this repo. `REF_ROOT="$(cat plans/.reference-root)"`
> (STOP if missing). Pinned at its commit `9f028d7`. Leak check
> (plans/README.md §Reference) before every commit.
>
> **Drift check (run first)**: `git diff --stat ad9115d..HEAD -- ui/src/components ui/src/lib`
> Plans 005-007 must be DONE. New files below must not already exist with
> different content; if they do, reconcile or STOP.

## Status

- **Priority**: P1
- **Effort**: L
- **Risk**: MED
- **Depends on**: plans/005, plans/006, plans/007
- **Category**: tech-debt (design system)
- **Planned at**: commit `ad9115d`, 2026-07-03

## Why this matters

Every redesigned screen (plans 011-018) is composed from the same small kit the reference
console uses: KPI stat cards with delta badges and bottom-bleed sparklines, a URL-state table
toolkit (search, filter selects, toggle chips, sortable heads, pagination windows), heat-
tinted numeric cells, self-ticking relative timestamps, copy buttons, delayed-loading
skeleton gates, a global time-range picker, and clickable chart legends with tick-thinning
helpers. Building the kit once — before any screen — is what makes the screen plans small and
consistent. It also replaces Parallax's current one-off composites (`kpi-card`, decorative
sparkline bars, per-page ad-hoc filters).

## Current state

- Parallax shared components today: `ui/src/components/kpi-card.tsx` (brand-toned tiles with
  fake decorative bars — will be superseded), `metric-strip.tsx` (self-fetching 3-chart strip;
  all series one color; duplicated by `RunMetrics` in `runs.$runId.tsx:84-210`),
  `logs-table.tsx`, `live-stream-panel.tsx`, `page-heading.tsx` (adapter since plan 007),
  `route-fallbacks.tsx`. No copy button, no relative-time component, no table toolkit, no
  range picker (each page hand-rolls a `rangeMinutes` select or hardcodes 1h).
- Data layer: `ui/src/lib/api.ts` exposes `graphql<T>(query)`, `gqlString(v)` (escaper),
  `relativeTime(nanos)`; nanosecond timestamps are **strings** end-to-end (`ui/AGENTS.md`
  rule 16).
- Router: TanStack Router with typed search params; `ui/AGENTS.md` rule 6 mandates
  `zod` `validateSearch` for URL state (zod is already a dependency).
- Reference sources to port from (read each; all under `$REF_ROOT/apps/web/src/`):
  - `components/app/page-parts.tsx` — `StatCard` (:125-191: Card size=sm; header row =
    icon `size-[13px]` + `CardDescription` label + `DeltaBadge`; value `CardTitle
    tracking-tight tabular-nums` + right `hint` `text-xs text-muted-foreground/70`; `chart`
    slot pinned `mt-auto -mb-6`), `DeltaBadge` (:193-218: flat ⇒ `~0%` muted; else pct +
    `IconCircleArrowUp/DownFilled`, emerald good / rose bad, `inverted` flips for
    cost/errors/latency), `CardSparkline` (:229-304, full-bleed area, color from `text-*`
    class, `dashedLast` for the filling bucket), `PillMeter` (:365-397, segmented capsule),
    `EmptyState` (:547-574, dashed card, icon `opacity-40`), `TableSkeleton` (:576-586),
    `CardsSkeleton` (:588-596), `ScrollFade` (:406-464).
  - `components/app/data-table.tsx` — URL filter state (:51-100), tri-state sort
    desc→asc→off + `parseSortParam`/`cycleSortParam` (:104-149), `sortRows` nulls-last
    (:155-173), `SortableHead` (:177-233, whole-head click target, arrow brightens),
    `Toolbar` (:250-258, `flex flex-wrap items-center gap-2`), `SearchInput` (:261-293,
    `w-56`, leading `IconSearch`, `Input h-8 rounded-full px-8 dark:bg-input/20`, clear
    button), `ToggleChip` (:298-322, active ⇒ `bg-rose-500/10 text-rose-600
    dark:bg-rose-500/15 dark:text-rose-400`), `FilterSelect` (:326-415, `rounded-full`
    compact select with "All" reset), `ClearFiltersButton` (:420-450).
  - `components/app/heat-cell.tsx` — 5-quintile traffic-light tint (`text-green-600
    dark:text-green-400` … `text-red-600 dark:text-red-400`), `percentileBucket`, tooltip
    naming the bucket, right-aligned `tabular-nums`, muted for null.
  - `components/app/relative-time.tsx` — self-refresh every 15s; `lib/format.ts` formatters
    (`formatRelative` `Ns/Nm/Nh/Nd ago` clamped ≥0; `formatDateTime` 24h).
  - `components/app/copy-button.tsx` + `copy-icon.tsx` + `use-copied.ts` — icon button
    `text-muted-foreground/60 hover:text-foreground`, `stopPropagation`, animated check.
  - `components/app/hooks.ts` — `useDelayedLoading(loading, 700)` (skeletons only after
    700ms), `useDebouncedValue(value, 300)`.
  - `components/app/range-context.tsx` + `range-picker.tsx` + `lib/range.ts` — presets
    `1h/24h/7d/30d/90d/today/month/lastMonth`, default 24h; picker = Popover with `w-40`
    preset rail + two-month range `Calendar`, trigger = outline button `rounded-2xl
    corner-squircle` with `IconCalendarEventFilled` + label + chevron.
  - `components/app/trend-charts.tsx` — clickable `ChartLegend` (swatch `h-2 w-2 rounded-2xl
    corner-squircle`, dims unselected to `opacity-30`), `thinTicks` (≈8 ticks, endpoints
    kept), `makeEdgeTick`, `makeBucketLabel`, `dedupeTicks`, `pageWindow` (pagination
    ellipsis).
  - `components/app/view-toggle.tsx` — cards/table segmented pill persisted per page
    (localStorage), spring thumb.
  - `components/app/span-type.tsx` — the palette pattern (variant/icon/bar/chip maps) to
    imitate for OTel span kinds.

## Commands you will need

From `ui/`: `rtk bun run typecheck` / `lint` / `test` / `build` → exit 0. New deps: none
(charts use existing recharts; calendar for the range picker: `bunx --bun shadcn@latest add
calendar` per `ui/AGENTS.md` rule 9 — if the CLI pulls `react-day-picker`, that is expected
and allowed here). Leak check: see plan 005.

## Scope

**In scope** (all NEW files unless noted):
- `ui/src/components/console/stat-card.tsx` (StatCard + DeltaBadge + CardSparkline + PillMeter)
- `ui/src/components/console/empty-state.tsx`, `skeletons.tsx` (TableSkeleton, CardsSkeleton)
- `ui/src/components/console/scroll-fade.tsx`
- `ui/src/components/console/data-table.tsx` (Toolbar, SearchInput, FilterSelect, ToggleChip,
  ClearFiltersButton, SortableHead, sortRows, pageWindow)
- `ui/src/components/console/heat-cell.tsx`
- `ui/src/components/console/relative-time.tsx`
- `ui/src/components/console/copy-button.tsx` (+ `use-copied.ts`)
- `ui/src/components/console/hooks.ts` (useDelayedLoading, useDebouncedValue)
- `ui/src/components/console/range-picker.tsx` + `ui/src/lib/range.ts`
- `ui/src/components/console/trend.tsx` (ChartLegend, thinTicks, makeEdgeTick, makeBucketLabel)
- `ui/src/components/console/view-toggle.tsx`
- `ui/src/components/console/span-kind.tsx` (OTel span-kind palette, see Step 6)
- `ui/src/lib/format.ts` (duration/count/time formatters, see Step 5)
- `ui/src/components/ui/calendar.tsx` (via shadcn CLI, restyled to reference recipe if the
  reference has one — it does: `$REF_ROOT/packages/ui/src/components/calendar.tsx`)
- `ui/src/components/metric-strip.tsx` (MODIFY: de-duplicate with RunMetrics — accept a
  `live` flag + series colors `--chart-1/2/3`; keep API self-fetch behavior)

**Out of scope**:
- All route files (`ui/src/routes/*`) — screens adopt the kit in plans 011-018. EXCEPTION:
  none. Do not wire the kit into any route here.
- `kpi-card.tsx`, `page-heading.tsx` — left in place (deprecated) until plan 018 deletes
  them.
- Live SSE plumbing (`live-stream-panel.tsx`) — untouched until the screen plans.

## Git workflow

`main`; Conventional Commits (`feat(ui): shared console kit`); `git commit -s`; trailer
`Co-authored-by: Claude <noreply@anthropic.com>`; leak check before commits.

## Steps

### Step 1: Stat cards, empty state, skeletons, scroll fade

Port `StatCard`/`DeltaBadge`/`CardSparkline`/`PillMeter`/`EmptyState`/`TableSkeleton`/
`CardsSkeleton`/`ScrollFade` from `page-parts.tsx` (imports → `@/components/ui/*`,
`@tabler/icons-react`). Delta type: `{dir: "up"|"down"|"flat", pct: number}` + a
`formatDelta(current, previous)` helper in `ui/src/lib/format.ts`. Keep the StatCard ordering
doctrine comment (Volume → Health → Performance → Cost) — screen plans cite it.

**Verify**: `rtk bun run typecheck` → exit 0.

### Step 2: Table toolkit

Port the data-table toolkit with one structural adaptation: **URL state goes through TanStack
Router**, not a Next router. Implement `useUrlFilters` on top of `useNavigate()` +
`Route.useSearch()` patterns: the hook takes and returns a flat `Record<string, string |
undefined>` patch and navigates with `replace: true`, dropping default-valued keys, resetting
`page` on any non-page change (mirror the reference semantics exactly; its file documents
them at :51-100). Screens declare their search schema with zod `validateSearch` (rule 6) and
pass parsed values in. Port `SortableHead` (tri-state desc→asc→off), `sortRows` (stable,
nulls-last), `Toolbar`, `SearchInput` (wire `useDebouncedValue`), `FilterSelect`,
`ToggleChip`, `ClearFiltersButton`, `pageWindow`.

**Verify**: typecheck; plus unit tests (Test plan) for `sortRows`, `cycleSortParam`,
`pageWindow`.

### Step 3: HeatCell, RelativeTime, CopyButton, hooks

Port each (imports adjusted). `RelativeTime` accepts **nanosecond strings** (Parallax's wire
format): convert with `Number(BigInt(ns) / 1_000_000n)` to ms internally; re-render every
15s. `CopyButton` = `navigator.clipboard.writeText` + animated check + `stopPropagation`.
`useDelayedLoading` default 700ms.

**Verify**: typecheck; RelativeTime unit test with a fixed `Date.now` mock.

### Step 4: Global time range (URL-first) + RangePicker

Adaptation decision (recorded): the reference keeps range in context+localStorage; Parallax
puts it **in the URL** (`?from=<ns>&to=<ns>&range=<key>`) because the product's core flow is
handing links/IDs to agents (`docs/research/architecture/simple-ui-v2.md` interactivity
rule), and `ui/AGENTS.md` rule 6 mandates zod-validated search params. Implement:
- `ui/src/lib/range.ts`: `RANGE_PRESETS` (`1h/24h/7d/30d/90d/today/month/lastMonth`, default
  `24h`), `resolvePreset`, `customRange`, `formatRangeLabel` (reference `lib/range.ts`
  semantics; use `date-fns`? — NO: date-fns is not a Parallax dep; implement with plain
  `Date`/`Intl.DateTimeFormat` and keep the same output shapes, or add date-fns via
  `rtk bun add date-fns` and note it. Either is acceptable; prefer adding date-fns for parity
  with the reference).
- `rangeSearchSchema` (zod): `{range?: string, from?: string, to?: string}` → resolved
  `{fromNanos, toNanos, key}` helper `resolveRangeSearch(search, now)` — presets re-resolve
  to "now" on every load; custom from/to win when both present.
- `RangePicker` component: Popover + preset rail (`w-40`, active = secondary button) +
  two-month `Calendar mode="range"` with the two-click draft logic and
  `disabled={{after: new Date()}}`; trigger = outline button `rounded-2xl corner-squircle`
  with `IconCalendarEventFilled` + label + chevron. It reads current value from props and
  emits `onChange(next)` — the route owns the URL write (keeps the component router-free).

**Verify**: typecheck; unit test `resolveRangeSearch` (preset resolution + custom range
passthrough + garbage tolerance).

### Step 5: Formatters (`ui/src/lib/format.ts`)

`formatDurationNs(ns: string|number)` → `950µs / 12.3ms / 1.24s / 2m 14s`;
`formatCount(n)` → `12 340 → 12.3k / 1.2M`; `formatRelative(ns)` (used by RelativeTime);
`formatTimeInRange(ns, {fromNanos,toNanos})` → time-of-day when the window ≤ 1 day, else
`MMM d, HH:mm:ss` (fixes the "7-day range shows only HH:MM:SS" defect);
`formatPercent(x)`; `formatDelta(cur, prev)`. All must accept the ns-string wire format
(BigInt-safe, `ui/AGENTS.md` rule 16).

**Verify**: unit tests per formatter (happy path + zero/negative/huge + ns-string precision).

### Step 6: Span-kind palette (`ui/src/components/console/span-kind.tsx`)

Imitate the reference `span-type.tsx` structure (VARIANT/ICON/BAR/CHIP maps + `SpanKindChip`
+ `SpanKindBadge`) for **OTel span kinds** with error overriding: kind `SERVER` → sky, kind
`CLIENT` → blue, `INTERNAL` → violet, `PRODUCER` → amber, `CONSUMER` → emerald, unknown →
slate/secondary; `statusCode === "STATUS_CODE_ERROR"` → rose everywhere (badge, bar color
`bg-rose-500`). Icons (Tabler, filled where available): SERVER `IconServerBolt`→fallback
`IconServer`, CLIENT `IconArrowUpRight`, INTERNAL `IconCpu`, PRODUCER `IconArrowBigUpLine`,
CONSUMER `IconArrowBigDownLine` — verify exports compile; on a missing export choose the
nearest existing Tabler glyph and note it. Bar map (`spanKindBar`) returns the `bg-*-500`
class used by the plan-012 waterfall.

**Verify**: typecheck; unit test: every kind returns a variant, icon, bar, chip class; error
forces rose.

### Step 7: Trend helpers + metric-strip de-dup

Port `ChartLegend`/`thinTicks`/`makeEdgeTick`/`makeBucketLabel`/`dedupeTicks` into
`console/trend.tsx`. Then refactor `metric-strip.tsx`: extract the fetch+render into one
component taking `{runId?, service?, live?: boolean}`; series colors `var(--chart-1/2/3)` for
CPU/memory/tasks (no more single-color); keep the 5s repoll when `live`. Update
`runs.$runId.tsx` ONLY by replacing its local `RunMetrics` with the shared component import —
no other changes to that route (its redesign is plan 017). Delete the now-dead local
`RunMetrics` code.

**Verify**: typecheck + build; `grep -n "RunMetrics" ui/src/routes/runs.\$runId.tsx` → no
local definition remains.

### Step 8: Gate

`rtk bun run typecheck && rtk bun run lint && rtk bun run test && rtk bun run build` → all 0.
Leak check → no output.

## Test plan

New file `ui/src/lib/__tests__/format.test.ts` — formatters incl. ns-string precision edge
(`"1719999999999999999"`), `formatTimeInRange` day-boundary behavior.
New file `ui/src/components/console/__tests__/kit.test.tsx` — `sortRows` (asc/desc/nulls-last
/stability), `cycleSortParam`, `pageWindow` (1, 7, 20 pages), `resolveRangeSearch`,
span-kind palette totality, `StatCard` renders label/value/delta, `RelativeTime` with mocked
clock. Pattern: plain vitest + testing-library; keep DOM assertions to classes/text.

## Done criteria

- [ ] typecheck / lint / test / build exit 0; all new unit tests pass
- [ ] `ls ui/src/components/console` shows the kit files listed in Scope
- [ ] `grep -rn "lucide-react" ui/src/components/console` → none
- [ ] `grep -n "chart-1" ui/src/components/metric-strip.tsx` → CPU/mem/tasks use three
      distinct chart tokens
- [ ] No route file except `runs.$runId.tsx` (RunMetrics swap only) modified: `git diff
      --name-only` confirms
- [ ] Leak check → no output
- [ ] `plans/README.md` row updated

## STOP conditions

- `plans/.reference-root` missing or reference files listed above absent.
- The shadcn CLI cannot add `calendar` for the Base UI variant (rule 9 forbids hand-copying
  CLI-managed components; the reference's calendar.tsx may be ported by hand ONLY if the CLI
  path fails — note which path you took).
- `useUrlFilters` can't express the semantics on TanStack Router without fighting types —
  report the mismatch; don't silently drop URL state.
- Any screen-plan file (011+) turns out to already exist and conflict with kit naming.

## Maintenance notes

- The kit is the single place screens get their table/stat/chart grammar; screen plans must
  not fork it. New needs → extend the kit file, keep the reference's recipe language.
- `formatTimeInRange` exists specifically so list timestamps stay readable on multi-day
  windows; any new list column showing a time must use it.
- Range lives in the URL by design (agent handoff); do not "simplify" it back into context.
