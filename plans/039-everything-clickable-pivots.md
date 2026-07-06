# Plan 039: Everything clickable — overview KPIs, issues-list doorways, detail-page cross-signal pivots, runs columns

> **Executor instructions**: Follow step by step; run every verification. On
> any STOP condition, stop and report. When done, update the status row in
> `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat 408be17..HEAD -- ui/src/routes ui/src/components`
> On mismatch with the excerpts below, STOP.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: LOW
- **Depends on**: plans/038 (use its `rangeLinkSearch` helper on every link
  added here)
- **Category**: dx (UX)
- **Planned at**: commit `408be17`, 2026-07-07

## Why this matters

The product's design principle (from the observability research brief,
`docs/research/architecture/full-observability-ui-and-playground-research.md`:
"Everything clickable. Chart → time window → traces/errors/logs → span →
bundle. Dead ends are UI bugs.") is violated on the highest-traffic surfaces:
the overview's four KPI cards and both trend charts go nowhere; the issues
list fetches `service` and `lastTraceId` per row but renders neither; service
detail has no path to that service's logs or issues; trace detail shows
service names as plain badges; issue detail computes the run id and then
never links it; the runs list hides `traceCount` and renders the errors count
as dead text. Each fix is a link to a surface that already exists — pure
navigation wiring, no new API.

## Current state

All excerpts verified at commit `408be17`.

- **Overview dead-ends** — `ui/src/routes/index.tsx:312-375`: four
  `StatCard`s (Spans `:313`, Logs `:330`, Error rate `:347`, p95 `:360`) with
  no link/onClick; `SignalTrendCard` (`:378`) and the latency trend card
  below it have no drill behavior. `StatCard` lives in
  `ui/src/components/console/` (check its props — it accepts `chart`, `icon`,
  etc.; it has no built-in link prop, so wrap or extend).
  Range is in scope as `range` (`index.tsx:246`).

- **Issues list hides fetched doorways** — the loader fetches per row
  (`ui/src/routes/issues.index.tsx:138-139` region: field list includes
  `service` and `lastTraceId`), but the table columns
  (`issues.index.tsx:310-335`) are Issue | Trend | Events | Age | Last seen |
  Tags | Status — no Service column; `lastTraceId` unused (grep: single
  match at `:139`). Row body `:338-433`.

- **Issue detail** — `ui/src/routes/issues.$fingerprint.tsx`:
  - loader resolves the last trace's run id (`:110-130`):

    ```ts
    traceRunId = correlated.trace?.spans.find((s) => s.runId)?.runId ?? null
    ```

    but the component only passes it to `MetricStrip` (`runId={traceRunId}`
    around `:314`) — there is no "Run" link anywhere.
  - the service renders as a dead badge (`:253-257`):

    ```tsx
    <Badge variant="outline">{issue.service}</Badge>
    ```

- **Trace detail** — `ui/src/routes/traces.$traceId.tsx:195-199`: service
  badges are plain text:

  ```tsx
  {services.map((service) => (
    <Badge key={service} variant="outline">
      {service}
    </Badge>
  ))}
  ```

  The span inspector also shows the selected span's service as text (search
  for the inspector's "service" row near `:484`).

- **Service detail** — `ui/src/routes/services.$service.tsx`: `PageHeader`
  has `back` + `RangePicker` only (`:276-280` in the no-data branch; the
  populated branch's header is at `:290+` — same shape). No links to
  `/logs?service=…` or `/issues?service=…` anywhere in the file (grep
  `to="/logs"` and `to="/issues"` → no matches). NOTE: check that `/logs` and
  `/issues` accept a `service` search param (logs: `search.service` exists —
  `ui/src/routes/logs.tsx:257` deps include it; issues: the list has a
  service filter dropdown — find its search key, likely `service`).

- **Runs list** — `ui/src/routes/runs.index.tsx`: loader fetches
  `traceCount` (`:167`) but the columns (`:283-289` region) omit it; the
  errors cell (`:323-332` region) renders a plain count; the service/command
  cell is plain text. Run detail route exists at `/runs/$runId`.

- Conventions: links use TanStack `<Link to search params>`; row-level links
  call `event.stopPropagation()` when the row itself is clickable (exemplar:
  `issues.index.tsx:354-361`); tables are shadcn `Table*` primitives.

## Commands you will need

| Purpose | Command (from `ui/`) | Expected |
|---------|----------------------|----------|
| Typecheck | `bun run typecheck` | exit 0 |
| Lint | `bun run lint` | exit 0 |
| Tests | `bun run test` | all pass |
| Build | `bun run build` | exit 0 |

## Scope

**In scope**:
- `ui/src/routes/index.tsx`
- `ui/src/routes/issues.index.tsx`
- `ui/src/routes/issues.$fingerprint.tsx`
- `ui/src/routes/traces.$traceId.tsx`
- `ui/src/routes/services.$service.tsx`
- `ui/src/routes/runs.index.tsx`
- `ui/src/components/console/stat-card.tsx` (or wherever `StatCard` lives —
  only if adding an optional link/`onClick` prop is needed)
- test files

**Out of scope**:
- New API fields (everything here uses already-fetched data).
- The ecosystem map / story / compare surfaces (advisor-plans 028-032).
- Chart brush-to-window on trend charts (the logs histogram pattern) — the
  KPI/trend links here navigate with the **current** window; drag-brush on
  overview charts is deferred (note in Maintenance).
- Runs range picker (plan 038 Step 5).

## Git workflow

- `main`, Conventional Commits, `git commit -s`, one
  `Co-authored-by: Claude <noreply@anthropic.com>` trailer. Push when done.

## Steps

### Step 1: Overview KPI + trend doorways

In `ui/src/routes/index.tsx`:
1. Make each StatCard navigate (wrap in `<Link>` or add an optional
   `href`-ish prop to `StatCard` — prefer wrapping with a block `Link` +
   `className` hover state to avoid touching the component API; keep focus
   ring for keyboard users):
   - Spans → `/traces` with `rangeLinkSearch(range)`
   - Logs → `/logs` with `rangeLinkSearch(range)`
   - Error rate → `/issues` with `{ status: "open", ...rangeLinkSearch(range) }`
     (verify the issues route's status search key by reading its
     `validateSearch`)
   - p95 → `/traces` with `{ sort: "DURATION", ...rangeLinkSearch(range) }`
     (verify the traces route's sort search key/value vocabulary in
     `traces.index.tsx` `validateSearch` before hardcoding; if no
     duration-sort param exists, link without it)
2. Add a "View traces"/"View issues" affordance to `SignalTrendCard` and the
   latency card headers (small ghost Button-Link, top-right of the card),
   same destinations.

**Verify**: `bun run typecheck && bun run lint` → exit 0.

### Step 2: Issues list — Service column + last-trace link

In `ui/src/routes/issues.index.tsx`:
1. Add a `Service` column (between Issue and Trend; width ~`w-32`):
   `<Link to="/services/$service" params={{ service: issue.service }}>` with
   `stopPropagation` (copy the row-link pattern at `:354-361`), rendered as
   the existing outline-badge style.
2. In the Issue cell's metadata line (under the title), add a small
   `trace` chip when `issue.lastTraceId` is non-null:
   `<Link to="/traces/$traceId" params={{ traceId: issue.lastTraceId }}>`.
3. Verify the service filter still works (the dropdown reads distinct
   services — unchanged).

**Verify**: `bun run typecheck` → exit 0;
`rtk grep -n "lastTraceId" ui/src/routes/issues.index.tsx` → ≥2 matches
(fetch + render).

### Step 3: Issue detail — link service, link run

In `ui/src/routes/issues.$fingerprint.tsx`:
1. Replace the dead service badge (`:254`) with a linked badge to
   `/services/$service`.
2. Next to it, when `traceRunId` is non-null render
   `<Badge variant="secondary">` wrapping a `Link` to `/runs/$runId`
   (`params: { runId: traceRunId }`) labeled `run <short-id>` (first 8 chars
   + ellipsis, matching how run ids render elsewhere — check
   `runs.index.tsx` for the id-shortening convention and reuse it).
3. Also link the existing last-trace affordance if present (the detail page
   has trace links in the events table — leave those; this is the header).

**Verify**: `bun run typecheck` → exit 0.

### Step 4: Trace detail — service badges → service links

In `ui/src/routes/traces.$traceId.tsx`:
1. Header badges (`:195-199`): wrap each in a `Link to="/services/$service"`.
2. Span inspector service row (~`:484`): same link treatment.

**Verify**: `bun run typecheck` → exit 0.

### Step 5: Service detail — Logs / Issues / Traces pivots

In `ui/src/routes/services.$service.tsx`, add to the populated-branch
`PageHeader` `actions` (before the RangePicker) three small outline
button-links, each carrying the window:
- `Traces` → `/traces` `search={{ service, ...rangeLinkSearch(range) }}`
- `Logs` → `/logs` `search={{ service, ...rangeLinkSearch(range) }}`
- `Issues` → `/issues` `search={{ service, ...rangeLinkSearch(range) }}`
  (verify the issues service-filter search key first; if the issues route
  filters by a different key name, use that key)

**Verify**: `bun run typecheck && bun run lint` → exit 0;
`rtk grep -n 'to="/logs"' ui/src/routes/services.\$service.tsx` → 1 match.

### Step 6: Runs list — show traceCount, link the counts

In `ui/src/routes/runs.index.tsx`:
1. Add a `Traces` column rendering `traceCount` (tabular-nums, right-aligned
   like Errors).
2. Make the errors count a `Link` to `/runs/$runId` (the run detail's issues
   section — plain detail link; anchor params only, no hash unless the
   detail page has stable section anchors — check; if it has none, link to
   the detail page and note the anchor as deferred).
3. Make the service/command cell's run id (or the row's primary cell) a
   proper `Link` to the detail (if the row already navigates via onClick,
   keep it and add stopPropagation on inner links — follow the
   issues.index.tsx row pattern).

**Verify**: `bun run typecheck && bun run test` → exit 0.

## Test plan

- Extend the existing route test setup only if route tests already mount
  these pages (`ui/src/routes/__tests__/` — currently only `-logs.test.tsx`
  exists): do NOT build new route harnesses. Bar for this plan:
  - `bun run typecheck` / `lint` / `build` gates
  - one snapshot-free DOM test for the issues-list Service column if the
    logs route test pattern is reusable cheaply; otherwise document manual
    checks (each new link navigates with expected search params) in the
    commit message.

## Done criteria

ALL must hold (from `ui/`):

- [ ] `bun run typecheck`, `bun run lint`, `bun run test`, `bun run build`
      all exit 0
- [ ] Overview: all four StatCards and both trend cards navigate (grep
      `<Link` count in `index.tsx` increased by ≥6)
- [ ] Issues list renders a Service column and a trace chip
      (`lastTraceId` grep ≥2)
- [ ] Issue detail header links service + run (`to="/runs/$runId"` present)
- [ ] Trace detail service badges are links (`to="/services/$service"` in
      `traces.$traceId.tsx` ≥2)
- [ ] Service detail header has Traces/Logs/Issues pivots carrying range
      params
- [ ] Runs list shows `traceCount` and links the errors count
- [ ] Every link added uses `rangeLinkSearch` where a range is in scope
- [ ] `plans/README.md` status row updated

## STOP conditions

- Plan 038's `rangeLinkSearch` doesn't exist yet (038 not landed) — STOP and
  either land 038 first or report; do not fork a second helper.
- The issues route has no `service`/`status` search params to target (its
  filter works differently) — report the actual search schema before
  inventing params.
- `StatCard` wrapping in a Link produces nested-interactive a11y errors
  (lint) — switch to the prop-based approach and report the API change.

## Maintenance notes

- Plan 040 virtualizes tables these links live in — coordinate merge order
  (this plan is plain JSX; land first, it's cheaper to rebase).
- Deferred: drag-brush-to-window on overview trend charts (the logs
  histogram already proves the pattern — port it in a later polish plan);
  stable section anchors on run detail for the errors deep-link.
- Reviewer: every inner link inside a clickable row needs
  `stopPropagation` (pattern at `issues.index.tsx:358`); check keyboard
  focusability of the wrapped StatCards.
