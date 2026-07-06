# Plan 040: UI performance at observability scale — stable keys, O(N) heat cells, shared ticker, virtualized logs, windowed waterfall

> **Executor instructions**: Follow step by step; run every verification. On
> any STOP condition, stop and report. When done, update the status row in
> `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat 408be17..HEAD -- ui/src/components ui/src/routes ui/package.json`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P2
- **Effort**: L
- **Risk**: MED (virtualization interacts with selection/sticky UI)
- **Depends on**: plans/035 (its MetricStrip cancellation lands first —
  same file regions); best after plans/039 (plain-JSX link sweep) to avoid
  rebase churn
- **Category**: perf
- **Planned at**: commit `408be17`, 2026-07-07

## Why this matters

The console's hottest views degrade quadratically or churn the DOM: heat
cells re-sort an N-length column array per cell (O(N²·log N) per table
render, ~1000 sorts for a 500-row table on every poll); the live log tail
prepends batches 4×/sec while row keys embed the array index, so React
remounts all 500 rows on every flush; every visible row mounts its own 15s
`setInterval` for relative time; log tables and trace waterfalls render
unbounded row counts with no virtualization. These are exactly the surfaces
an observability tool must keep smooth under volume.

## Current state

All excerpts verified at commit `408be17`.

- **HeatCell** — `ui/src/components/console/heat-cell.tsx:12-15`:

  ```ts
  const sorted = values.filter(Number.isFinite).sort((a, b) => a - b)
  if (!sorted.length) return 0
  const rank = sorted.findIndex((candidate) => value <= candidate)
  const pct = (rank < 0 ? sorted.length - 1 : rank) / Math.max(1, sorted.length - 1)
  ```

  `percentileBucket` runs per rendered cell. Callers pass the full column
  array per row: `ui/src/routes/services.tsx:364,372` (two cells per row),
  `traces.index.tsx:666`, `index.tsx:790`, `issues.$fingerprint.tsx:664`,
  `runs.$runId.tsx:490`, `services.$service.tsx` (import at `:20`; find the
  call site). Note `findIndex` is also O(N) — the whole thing is
  sort-per-cell + linear scan.

- **LogsTable keys** — `ui/src/components/logs-table.tsx:189`:

  ```tsx
  key={`${log.tsNanos}-${index}`}
  ```

  while the live feed prepends: `ui/src/routes/logs.tsx:245-252`:

  ```ts
  const flush = setInterval(() => {
    if (buffer.length === 0) return
    const incoming = buffer
    buffer = []
    setLogs((current) =>
      [...incoming.reverse(), ...current].slice(0, PAGE_SIZE)
    )
  }, 250)
  ```

  Every prepend shifts all indices → all keys change → full-table remount
  4×/sec. `PAGE_SIZE = 500` (`logs.tsx:87`), and `loadOlder` appends more
  pages unbounded (`logs.tsx:286`).

- **RelativeTime** — `ui/src/components/console/relative-time.tsx:5-10`:
  one `setInterval(..., 15_000)` per instance; rendered per row in
  `issues.index.tsx:398,401`, `services.tsx:378`, `traces.index.tsx:673`,
  `index.tsx:744`, and others.

- **TraceWaterfall** — `ui/src/components/console/trace-waterfall.tsx`:
  `rows`/`window`/`ids` are memoized (`:31-36`), but the service-badge set is
  recomputed inline on every render (`:106`):

  ```tsx
  {Array.from(new Set(spans.map((span) => span.service))).map(...)}
  ```

  and `rows.map(...)` renders every span as its own button row (`:124`) with
  no windowing. Live span feeds cap at 100 (`traces.index.tsx:335`), but a
  fetched trace detail has no cap.

- **No virtualization dependency**: `ui/package.json` dependencies include
  `@tanstack/react-table` but NOT `@tanstack/react-virtual` (verified). The
  research brief explicitly sanctions adding it:
  `docs/research/architecture/full-observability-ui-and-playground-research.md`
  ("Use `@tanstack/react-virtual` for high-volume trace/log/live-tail
  virtualization if the current table/list primitives are not enough").

- **Zero tests** on HeatCell/percentileBucket; LogsTable has a test
  (`ui/src/routes/__tests__/-logs.test.tsx`) and TraceWaterfall has
  `ui/src/components/console/__tests__/waterfall.test.tsx` — extend, don't
  replace.

- Conventions: strict TS; Bun; version policy = latest stable
  (`AGENTS.md`) — resolve the newest stable `@tanstack/react-virtual`.

## Commands you will need

| Purpose | Command (from `ui/`) | Expected |
|---------|----------------------|----------|
| Add dep | `bun add @tanstack/react-virtual` | exit 0, bun.lock updated |
| Typecheck | `bun run typecheck` | exit 0 |
| Lint | `bun run lint` | exit 0 |
| Tests | `bun run test` | all pass |
| Build | `bun run build` | exit 0 |

## Scope

**In scope**:
- `ui/src/components/console/heat-cell.tsx` + its 6 caller routes (call-shape
  change only)
- `ui/src/components/logs-table.tsx`
- `ui/src/components/console/relative-time.tsx`
- `ui/src/components/console/trace-waterfall.tsx`
- `ui/src/routes/logs.tsx` (only if the virtualized table needs container
  changes)
- `ui/package.json` + `bun.lock` (the one new dep)
- test files

**Out of scope**:
- MetricStrip fetch/race work (plan 035) — but DO memoize its per-render
  chart-data transform if 035 already landed (one `useMemo`; coordinate).
- Any table redesign/styling (plan 053).
- Server-side pagination changes.
- Virtualizing every table in the app — only LogsTable (the proven hot spot)
  and waterfall windowing here; note others in Maintenance.

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one
  `Co-authored-by: Claude <noreply@anthropic.com>` trailer. Push when done.

## Steps

### Step 1: Characterization tests first

1. `ui/src/components/console/__tests__/heat-cell.test.tsx` (create): pin
   `percentileBucket` behavior — empty array → 0; single value; ties; NaN
   filtered; value below min / above max; exact bucket boundaries for a
   known 10-element array. These tests must pass BEFORE and AFTER Step 2.
2. Extend the LogsTable test with a rerender assertion: render 3 logs,
   capture row DOM nodes, prepend 1 log, rerender, assert the previous rows'
   DOM nodes are reused (same element identity) — this fails with the index
   key and passes after Step 3.

**Verify**: `bun run test` → new heat-cell tests pass; the LogsTable
identity test FAILS (red) — that's expected; keep it failing into Step 3.

### Step 2: HeatCell — precompute thresholds once per column

Change the API so the sort happens once per table render:
1. In `heat-cell.tsx`, export `buildHeatScale(values: number[])` that
   filters/sorts once and returns the sorted array (or bucket thresholds);
   change `percentileBucket(value, sorted)` to binary-search the sorted
   array (O(log N)).
2. `HeatCell` accepts `scale` (the prebuilt sorted array) instead of raw
   `values` — or keep `values` for backward compat but ALSO accept `scale`
   and prefer it. Prefer the clean break: this is an internal component with
   6 call sites.
3. Update all callers: hoist `const scale = useMemo(() => buildHeatScale(rows.map(...)), [rows])`
   per column (services.tsx has two columns → two scales), pass `scale`.

**Verify**: `bun run test` → heat-cell characterization tests still pass
(identical bucket outputs); `bun run typecheck` → exit 0;
`rtk grep -rn "values={" ui/src/routes | rtk grep HeatCell` → no remaining
raw-array call sites (grep for `<HeatCell` and check props manually).

### Step 3: LogsTable — stable keys

Replace the key at `logs-table.tsx:189` with a stable identity:
`key={`${log.tsNanos}-${log.spanId ?? ""}-${log.traceId ?? ""}`}` — check
collision risk: two logs can share tsNanos+ids; if the row data has no
unique field, assign a monotonically-increasing client id when logs enter
state (in `logs.tsx` live path and loader path: wrap as
`{ ...log, _key: nextId() }`). Choose the client-id approach if any
collision is plausible (it is — batch inserts share timestamps); implement
it in the state layer, not render.

**Verify**: the Step 1 identity test now PASSES; `bun run test` all green.

### Step 4: Shared ticker for RelativeTime

In `relative-time.tsx`, replace per-instance intervals with a module-level
subscriber set + one `setInterval` started on first subscriber, stopped on
last unsubscribe (or a tiny context provider in the shell — module singleton
is simpler and SSR-safe if guarded by `typeof window !== "undefined"`).
Existing kit test (`console/__tests__/kit.test.tsx`) covers RelativeTime —
keep it passing; add a test: two mounted instances → only one interval
(mock `setInterval` and count calls).

**Verify**: `bun run test` → all pass including the new interval-count test.

### Step 5: Virtualize LogsTable

1. `bun add @tanstack/react-virtual`.
2. In `logs-table.tsx`, wrap the row list with `useVirtualizer` (parent =
   the existing scroll container — the table lives in an
   `overflow`-scrollable card; verify which element scrolls and give it a
   fixed height if none has one). Table semantics with virtualization:
   use the documented pattern — virtualize `<TableRow>`s inside `<TableBody>`
   with translateY spacers (padding rows) so `<Table>` markup stays valid.
3. Preserve: click-row → document Sheet, severity styling, the trace/run
   chips, sticky header.
4. Keep rendering ALL rows when `logs.length <= 100` (skip virtualizer
   overhead for small sets; also keeps the existing LogsTable tests
   meaningful without a scroll-container harness).

**Verify**: `bun run test` → existing LogsTable tests pass (they render <100
rows → non-virtualized path); `bun run build` → exit 0. Manual (dev server +
live tail): scrolling 500-row live tail stays smooth; row selection still
opens the doc viewer.

### Step 6: Waterfall — memoized services + windowing for very large traces

1. Memoize the service set: `const services = useMemo(() => Array.from(new Set(spans.map((s) => s.service))), [spans])`
   replacing the inline computation at `:106`.
2. Window the span rows with the same virtualizer when
   `rows.length > 300` (waterfall rows are fixed-height buttons — measure
   the row height from the existing py-1.5 layout). Keyboard navigation
   (`moveSelection`, `:43-47`) must still work across the whole `ids` list
   and scroll the selection into view (`virtualizer.scrollToIndex`).
3. Below 300 rows, render exactly as today (existing waterfall tests keep
   passing unmodified).

**Verify**: `bun run test` → `waterfall.test.tsx` passes unchanged;
`bun run typecheck && bun run build` → exit 0.

## Test plan

Summarized from steps: heat-cell characterization (Step 1, ≥6 cases);
LogsTable row-identity test (Step 1/3); RelativeTime single-interval test
(Step 4); existing logs/waterfall/kit/shell tests stay green throughout.
`bun run test` → all pass.

## Done criteria

ALL must hold (from `ui/`):

- [ ] `bun run typecheck`, `bun run lint`, `bun run test`, `bun run build`
      all exit 0
- [ ] `rtk grep -n "sort(" ui/src/components/console/heat-cell.tsx` shows
      the sort only in `buildHeatScale`, not in the per-cell path
- [ ] `rtk grep -n 'key={\`\${log.tsNanos}-\${index}\`}' ui/src/components/logs-table.tsx`
      → no matches
- [ ] `rtk grep -c "setInterval" ui/src/components/console/relative-time.tsx`
      → 1, inside the shared-ticker singleton
- [ ] `@tanstack/react-virtual` in `ui/package.json` dependencies;
      `bun.lock` updated; no other lockfile created
- [ ] LogsTable virtualizes above 100 rows; waterfall windows above 300
- [ ] `plans/README.md` status row updated

## STOP conditions

- The logs scroll container turns out to be the window/page (no bounded
  scroll parent) — introducing one changes layout; report the layout change
  needed before doing it.
- Virtualized `<tr>` spacer pattern fights the shadcn Table styles (sticky
  header breaks) — stop after two attempts and report; do not fork a
  non-table div layout silently.
- Heat-cell characterization outputs differ after Step 2 — your binary
  search has a boundary bug; fix until identical, else STOP.
- `@tanstack/react-virtual` latest stable requires React >19.2 or conflicts
  with `@tanstack/react-table` — report the version matrix.

## Maintenance notes

- Candidates deliberately not virtualized yet: traces list, issues list,
  services table (all capped ≤500 and colder); revisit when any gets an
  unbounded feed.
- The live span feed cap (100, `traces.index.tsx:335`) can be raised once
  the waterfall windows — note for whoever builds long-trace demos (A19
  scenario in the research brief).
- Reviewer: check the virtualizer's measured row heights against py-1.5
  rows on both themes; check `scrollToIndex` on keyboard nav; check the
  `_key` counter doesn't reset between live batches (duplicate keys).
