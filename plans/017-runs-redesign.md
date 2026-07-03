# Plan 017: Runs — scannable list (errors/duration) + composed run detail

> **Executor instructions**: Step by step; verify each; STOP conditions
> binding; update `plans/README.md` when done.
>
> **Reference project**: operator-designated local reference console — name
> NEVER in this repo. `REF_ROOT="$(cat plans/.reference-root)"` (STOP if
> missing), pinned at its commit `9f028d7`. Leak check before commits.
>
> **Drift check (run first)**: `git diff --stat ad9115d..HEAD -- ui/src/routes/runs.index.tsx ui/src/routes/runs.\$runId.tsx`
> Plans 005-008 must be DONE (008 already swapped RunMetrics for the shared
> MetricStrip in the detail route).

## Status

- **Priority**: P2
- **Effort**: M
- **Risk**: LOW
- **Depends on**: plans/005-008 (012 recommended first for visual consistency of trace links)
- **Category**: tech-debt (UX redesign)
- **Planned at**: commit `ad9115d`, 2026-07-03

## Why this matters

Runs is Parallax's agent-loop anchor (`parallax run` wraps a command; everything correlates
by run id). The spec (`docs/research/architecture/simple-ui-v2.md:86-88`) wants the list to
answer "which run failed / took longest" — today's columns are Run/Source/Command/Status/
Exit/Last seen with **no error count and no duration**, and the detail is the only page that
skips the shared header (hand-rolled `<h1 className="font-mono …">` at `runs.$runId.tsx:328`),
plus the evidence bundle is preview-only (spec says export). Live SSE follow (dual stream +
poll) is a strength — keep it, restyled.

## Current state

- `ui/src/routes/runs.index.tsx`: loader merges `runs` + `observedRuns` into one Map
  (`:57-82`); table columns Run/Source/Command/Status/Exit/Last seen (`:151-159` area);
  4 KpiCards; `.parallax-panel`; only the runId cell links.
  Available fields per merged row: registered `Run {runId, command, startedAtNanos,
  endedAtNanos, exitCode, status, errorCount, traceCount, issues}` (errorCount/traceCount
  are derived resolvers — `crates/parallax-api/src/lib.rs:460-470`) and/or `ObservedRun
  {runId, service, firstNanos, lastNanos, spanCount, logCount}`.
- `ui/src/routes/runs.$runId.tsx`: loader `run{…issues} + tracesByRun + logsByRun(200) +
  bundle{markdown}`; go-live toggle → dual SSE + 10s run poll + 5s metric repoll; hand-rolled
  header (`:326-370`); cards: LiveStreamPanel, MetricStrip (shared since plan 008), issues,
  traces (rows `border-b`), logs (shared LogsTable), bundle `<pre>` (`:518-534`), empty bare
  `<p>` (`:374`).
- Kit: PageHeader, StatCard, table toolkit, RelativeTime, CopyButton, EmptyState, skeletons,
  formatters (`formatDurationNs`), Badge variants.

## Commands you will need

From `ui/`: `rtk bun run typecheck` / `lint` / `test` / `build` → 0; dev + serve +
`parallax run <cmd>` or playground run traffic. Leak check: plan 005 table.

## Scope

**In scope**: `ui/src/routes/runs.index.tsx`, `ui/src/routes/runs.$runId.tsx` (rewrites).
**Out of scope**: API; bundle format; SSE server; `live-stream-panel.tsx` internals (restyle
via wrapper classes only if needed — full restyle is plan 018's cleanup sweep).

## Git workflow

`main`; `feat(ui): redesign runs pages`; `git commit -s`; trailer
`Co-authored-by: Claude <noreply@anthropic.com>`; leak check first.

## Steps

### Step 1: Runs list

`PageHeader` (Runs icon; description; no range — runs are few) → `Toolbar` (SearchInput on
runId/command client-side, status FilterSelect running/finished/external, count) → `Table`
(interactive rows → `/runs/$runId`; delete KPI strip):

| Column | Treatment |
|---|---|
| Run | runId `font-mono text-xs` truncated + CopyButton (stopPropagation); source Badge (`cli`/`external` secondary) |
| Command | `truncate font-mono text-xs text-muted-foreground` (observed-only rows: service name italic) |
| Status | Badge: running → sky w/ pulsing dot, finished+exit 0 → emerald, finished+exit≠0 → rose `exit N`, external → secondary |
| **Errors** | right `w-24`; `errorCount` (registered) — rose when >0, muted-40 when 0; observed-only rows "—" |
| **Duration** | right `w-28`; `endedAtNanos-startedAtNanos` via `formatDurationNs`; running → live-ticking "…"; observed-only: `lastNanos-firstNanos` |
| Last seen | right `w-32` `RelativeTime` |

Rows with errors get the rose left-accent bar. Empty → EmptyState with `parallax run
<your command>` snippet + CopyButton.

**Verify**: failed run shows rose exit badge + error count; durations humanized; whole-row
click navigates.

### Step 2: Run detail — shared header + stat row

Replace the hand-rolled `<h1>` with `PageHeader back={navItem("/runs")}` title=runId
(`titleLeading` mono styling ok), `titleTrailing`=CopyButton(runId), description=command
(mono), actions = Follow-live toggle Button (existing behavior, restyled: outline; active =
secondary with pulsing emerald dot) + "Download bundle" (Step 4).
Stat row (4 StatCard sm): Status (badge as value), Errors (rose when >0), Traces
(`traceCount`), Duration (ticking while running).

**Verify**: `grep -n "font-mono text-lg" ui/src/routes/runs.\$runId.tsx` → none; header
matches every other detail page.

### Step 3: Body composition

Order: (live) LiveStreamPanel when following → MetricStrip (shared, live flag) → Issues Card
(rows: errorType+title → `/issues/$fingerprint`, count, RelativeTime) → Traces Card
(compact table: root name, spans, duration `formatDurationNs`, error accent, whole-row →
`/traces/$traceId`) → Logs Card (shared LogsTable) → Bundle Card. All Cards plan-006 style;
kill remaining `border-b` hand dividers (table rows carry their own). Empty run →
EmptyState (kills bare `<p>`).

### Step 4: Bundle export

Bundle Card: markdown preview in a `ScrollFade` mono block + two actions: CopyButton (whole
markdown) and **Download** — `Blob` + `URL.createObjectURL` + anchor download
`parallax-bundle-<runId>.md` (client-side; no API change). Spec's "bundle export" satisfied.

**Verify**: clicking Download saves the .md file with the bundle content.

### Step 5: Gate

typecheck/lint/test/build; both themes; live-follow manual check (SSE rows stream, poll
updates status); leak check.

## Test plan

Unit: merged-row shaping (registered+observed union — duration/errors fallbacks per source),
status→badge mapping totality. Component: list fixture renders error/duration columns; detail
fixture renders stat row + download button presence (jsdom: assert anchor download attr).

## Done criteria

- [ ] typecheck / lint / test / build exit 0; tests pass
- [ ] List has Errors + Duration columns (spec gap closed); rows fully clickable
- [ ] Detail uses PageHeader; bundle downloadable; every ID copyable
- [ ] Live follow still works (manual, noted in report)
- [ ] No KpiCard/.parallax-panel/lucide/bare `<p>` empties in scope files
- [ ] Leak check → no output; `plans/README.md` row updated

## STOP conditions

- `Run.errorCount/traceCount` resolvers absent (schema drift vs `lib.rs:460-470`).
- SSE follow breaks under the loader-based rewrite (same trap as plan 011 Step-4 note) — if
  loaders refetch-kill the stream, keep detail client-fetched as today and note it.
- Bundle markdown exceeds ~5MB and stalls the preview — render download-only + report.

## Maintenance notes

- A future "run timeline" (single time-ordered track of errors/logs/spans) is the natural
  next step (spec mentions it); the composed cards here are the stepping stone — don't
  half-build a timeline inside this plan.
- If plan 012's waterfall component gets virtualization, consider embedding a mini-waterfall
  per trace row here.
