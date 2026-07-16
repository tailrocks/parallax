# Plan 157: Build the CLI-invocation observability surface — invocation hub, sessions/screens/actions, cycles/jobs, ecosystem node kinds

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. Read `ui/AGENTS.md` fully before touching `ui/`. If anything in
> the "STOP conditions" section occurs, stop and report — do not improvise.
> When done, update the status row for this plan in `plans/README.md`.
>
> **Drift check (run first)**:
> `git diff --stat 39f172c..HEAD -- ui/src ui/tests`
> Plan 156 intentionally regenerates `ui/src/shared/semconv.ts` and changes the
> GraphQL SDL before this plan starts — that drift is expected. Any OTHER
> drift against the excerpts below is a STOP condition.

## Status

- **Priority**: P1
- **Effort**: XL
- **Risk**: MED (product-behavior change by design; replaces the Runs surface)
- **Depends on**: plans/156-unified-cli-observability-contract.md
- **Category**: direction / product / UI
- **Planned at**: commit `39f172c`, 2026-07-17
- **Operator directive (2026-07-17)**: one platform where a connected CLI
  application (jackin❯ or any OTel-instrumented CLI) is observable end to end:
  what is running now, every trace/span/log/error for one invocation, its
  interactive sessions/screens/actions, its background cycles and jobs, its
  agent conversations, plus service/CLI/browser topology — with a
  user-controlled real-time toggle on every live view. Lands as direct
  commits to `main` alongside plans 156 and 160 (159 provides the closing
  evidence); no branches, no pull requests (operator delivery model).

### Landed by Grok (preliminary) — full UI still peer-owned

Wire-contract unblock so UI talks to plan-156 GraphQL/SSE (not product UX):
- `ui/src/routes/sql.tsx` + `-sql.test.tsx`: SQL presets / `targetForCell` use
  `CLI_INVOCATION_ID` / `invocation_id`.
- Runs list/detail + shared API selections: GraphQL
  `invocations`/`observedInvocations`/`invocation(invocationId)` /
  `tracesByInvocation`/`logsByInvocation`/`story(invocationId)` / etc.; log
  `LOG_FIELDS` and span live types use `invocationId`; SSE query
  `invocation_id=`.
- **Routes still `/runs` and `/runs/$runId`** (param key `runId`) — full
  rename + hub/journey/live toggles remain this plan’s core work.
- Regenerated `ui/src/shared/semconv.ts` ships with plan 156.
- `bun run typecheck` green; vitest `-runs`/`-sql` green.

**Peer: implement full invocation hub, session journey, live toggles, route
rename `/runs`→`/invocations`, browser checks with `agent-browser`.** Do not
mark DONE from this note.

## Why this matters

The current "Runs" surface assumes Parallax's vendor attribute and a
registered-wrapper world. jackin❯'s cutover makes the generic contract
(`cli.invocation.id`, `session.id`, `app.mode`, `ui.*` events,
`background.cycle`, jobs, `gen_ai.*`) the only thing arriving on the wire.
This plan rebuilds the surface as **Invocations**: a live list of CLI
processes, and a per-invocation hub that answers "what is this CLI doing right
now and what did it do" from every angle — without any Parallax-specific
attribute. It follows the CURRENT UI conventions (flat routes +
`components/console/`, `lib/api.ts` single data path) because the
feature-architecture chain (plans 128→151) is blocked on an external
TypeScript-7 issue; the later migration plan (140, retitled) moves this
surface behind a feature facade unchanged.

## Current state

(verified at `39f172c`; all paths under `ui/`)

- **Routes** (file-based TanStack Router; generated `src/routeTree.gen.ts` is
  committed and never hand-edited): `src/routes/runs.index.tsx` (list, ~460
  lines) and `src/routes/runs.$runId.tsx` (detail) are the surface being
  replaced. Their tests: `src/routes/__tests__/-runs.test.tsx`.
- **List loader** (`runs.index.tsx:185-209`) queries `runs { runId command
  status exitCode startedAtNanos endedAtNanos errorCount traceCount }` and
  `observedRuns { runId service firstNanos lastNanos spanCount logCount }`;
  `mergeRuns` (`:99-142`) unions them with CLI-over-external precedence;
  columns Run/Command/Status/Traces/Errors/Duration/Last seen; filters =
  free-text + status FilterSelect + `RangePicker`; `RunStatusBadge`
  (`:446-459`) renders running (pulsing dot) / finished / `exit N` / external.
- **Detail** (`runs.$runId.tsx`): `loadRunDetail` (`:120-167`) two-stage —
  run metadata, then `tracesByRun`/`logsByRun`/`story`/`runtimeSnapshot`/
  `agentSession`/`bundle`; live section (`:232-284`): "Follow live" toggle
  opens SSE `/v1/logs/stream?run_id=` + `/v1/traces/stream?run_id=` and polls
  the run record every 10 s while `live && pageVisible`; live buffers capped
  at 300.
- **Live plumbing**: `src/hooks/use-live-stream.ts` (EventSource, 250 ms
  batched flush, visibility-gated via `src/lib/use-visible.ts`, status
  `idle|connecting|open|error`). Traces list (`traces.index.tsx:353-381`) and
  Logs (`logs.tsx:292-312`) already have Live toggles on the same hook.
- **Data layer**: `src/lib/api.ts` — `graphql<T>` POST `/graphql`,
  `graphqlCached<T>` client-only 15 s TTL cache; hand-written interfaces
  (`Run`/`ObservedRun` `:293-308`, `LiveSpan = Span & {runId}` `:170`,
  `LOG_FIELDS` includes `runId` `:158`). No codegen, no TanStack Query yet
  (`ui/AGENTS.md` rule 7).
- **Cross-links keyed on run id today**: `components/logs-table.tsx:44,115,
  447-459,490-498` (`run_id` chip → `/runs/$runId`);
  `components/metric-strip.tsx:104,132-133,199` (run- vs service-scoped
  query); `components/console/command-palette.tsx:48-55,190-196,361-384`
  (recent runs + id jump); `lib/quick-jump.ts:9,27` (6-hex-style RUN_ID
  regex); `routes/traces.$traceId.tsx:96,194,331,426-436,581` (trace → run
  link + run-scoped MetricStrip); `routes/issues.$fingerprint.tsx:96-140,
  312-317,386` (issue → run badge); `routes/services.$service.tsx:107,197,200`
  (exemplar runId, unrendered); `routes/sql.tsx:33,106,110,180`
  (`targetForCell`: `run_id`/`parallax.run.id` column → `/runs/$runId`).
- **Navigation**: `src/components/nav.ts:82-90` — workspaceNav first entry
  `{ href: "/runs", label: "Runs", icon: IconTerminal2, … violet … }`;
  rendered by `components/parallax-shell.tsx:201-266`.
- **Ecosystem**: `routes/ecosystem.tsx` loads `serviceMap { nodes {name
  lastSeenNanos spanCount errorCount p95Ms} edges {source target callCount
  errorCount p50Ms p95Ms} }`; `components/console/ecosystem-graph.tsx`
  hand-rolled SVG (BFS layers, bezier edges, width `log2(callCount)`); nodes
  link to `/services/$service`, edges to `/traces?service=…`.
- **Existing building blocks to reuse, not rebuild**: `components/console/
  story-timeline.tsx`, `agent-session.tsx` (`AgentSessionData`, step kinds
  INVOKE_AGENT/EXECUTE_TOOL/SHELL/OTHER), `trace-waterfall.tsx`,
  `runtime-snapshot.tsx`, `metric-strip.tsx`, `logs-table.tsx`,
  `data-table.tsx` (Toolbar/SearchInput/FilterSelect/SortableHead),
  `range-picker.tsx`, `relative-time.tsx`, `stat-card.tsx`,
  `live-stream-panel.tsx`, `page-header.tsx`.
- **Conventions that bind this plan** (`ui/AGENTS.md`): strictest TS
  (`bun run typecheck` gates commits); loaders isomorphic, derive via
  `Route.useLoaderData()`; zod `validateSearch` for typed search params;
  shadcn Base UI (`base-vega`) added only via
  `bunx --bun --no-install shadcn add`; charts = Recharts in
  `ChartContainer`; tables = TanStack Table; nanosecond timestamps are
  strings end-to-end; **every chart/list links onward — no dead ends**; one
  data path = `lib/api.ts`; tests colocated `__tests__/-name.test.tsx`
  (Vitest 4 + Testing Library + jsdom). Keep new functions/components ≤60
  logical lines and new modules ≤300 where feasible (ENGINEERING-STANDARDS
  budgets) — extract into `components/console/invocations/` rather than
  growing one giant route file (the 1,216-line runs routes are the
  anti-pattern this replaces).

## Product design (fixed)

**Navigation**: workspaceNav entry becomes
`{ href: "/invocations", label: "CLI Apps", icon: IconTerminal2 }` (keep the
violet treatment). `/runs*` routes are deleted — pre-release, no redirects.

**`/invocations` — the list ("what is running in my CLI")**
- Columns: Invocation (short-id `<code>` + CopyButton + source badge
  cli/external), Command (`cli.command.name` or registered command), Mode
  (`app.mode` badge: one_shot/interactive/daemon/capsule), Service
  (`service.name@version`), Status (running pulse / finished / `exit N` /
  failed / stale), Outcome (`outcome` chip when present), Traces, Errors
  (rose left-border when >0), Sessions, Duration, Last seen.
- Filters: free text (id+command+service), Mode FilterSelect, Status
  FilterSelect, Outcome FilterSelect, `RangePicker`. Search params via zod
  `validateSearch`.
- **Real-time toggle**: `live` search param; when on, poll the merged
  invocation list every 5 s while `pageVisible` (list is aggregate — polling,
  not SSE, matches `metric-strip.tsx:180-182` precedent) and show the pulsing
  live indicator; when off, manual `IconRefresh` → `router.invalidate()` like
  `traces.index.tsx`.
- Data: single `invocations` GraphQL field (plan 156 does the merge
  server-side — the UI `mergeRuns` duplicate dies).

**`/invocations/$invocationId` — the hub**
Header: command + mode badge + service@version + status + outcome + exit code
+ started/duration + invocation id copy + **master Live switch** (search param
`live`, drives every tab's streaming/polling).
Tabs (search param `tab`, zod-validated):
1. **Overview** — stat cards (traces/spans/logs/errors/sessions), story
   timeline (`story(invocationId)` — plan 156 extends beats with session
   start/end, screen transitions, cycle and job beats), `MetricStrip` scoped
   to the invocation, evidence-gaps card (existing component).
2. **Traces** — table of `tracesByInvocation` (root name, service, spans,
   errors, duration, start) + live prepend from
   `/v1/traces/stream?invocation_id=` when Live; row → `/traces/$traceId`
   (waterfall/span inspector already exist there — do not duplicate).
3. **Logs** — `LogsTable` fed by `logsByInvocation` + live tail from
   `/v1/logs/stream?invocation_id=` when Live; severity + text filters
   (reuse the logs-table capabilities, not the /logs route).
4. **Errors** — error.type breakdown (count per stable `error.type`, bar +
   table) + correlated issues list (fingerprint, title, events, last seen →
   `/issues/$fingerprint`) filtered to this invocation.
5. **Sessions & UI** — sessions list (`sessions(invocationId)`: id, start,
   end/open, previous-id chain link); per selected session: screen-visit
   lane (Gantt-style rows from `screenVisits` — screen id, dwell, navigation
   sequence), `uiActions` table (action name, screen, duration, outcome,
   trace link), conversations (`conversations(invocationId)`: agent name,
   provider, span count, first/last, token totals; selecting one renders the
   existing `agent-session` step timeline via `agentSession`), and the
   **Journey view** (below).
6. **Jobs & Cycles** — `backgroundCycles` summary table (name, runs, error
   count, p50/p95, last trace link) + `jobs` table (job id short, type,
   producer time, attempts with outcome chips, trace links).
Empty states: every tab has an explicit "nothing yet — this invocation has
not emitted X" empty card; the hub renders for **observed-only** invocations
(no registration row) without error.

**Journey view (operator requirement, 2026-07-17)** — per session, one
chronological narrative answering "what happened to this user": interleaved,
time-ordered entries built purely from generic signals — `session.start` →
screen entered/exited (with dwell) → `ui.action` (name, outcome, widget
context from `app.widget.*` when present) → **errors and exceptions placed on
the screen where they happened** (attribution: error/exception event or
error-status span whose timestamp falls inside a screen visit of the same
session/invocation; unattributable errors render in an "outside any screen"
bucket, never dropped) → `session.end`. Every entry links onward (action →
trace, error → issue/trace, screen → filtered logs). The journey must answer,
from the UI alone: which screen the user was on, where they moved, and on
which screen/widget an error hit. Same component works for any emitter that
sends the generic events — no jackin-specific logic (generic-attributes-only
invariant: application-specific keys appear only inside generic
attribute-list views).

**Ecosystem upgrade** — `serviceMap` nodes gain `kind: cli | browser |
service` (plan 156 derives: cli = service emitted spans carrying
`cli.invocation.id`; browser = `telemetry.sdk.language=webjs`; else service).
Graph renders kind glyphs (terminal icon for cli, globe for browser); cli
nodes link to `/invocations?service=<name>` instead of `/services/$service`.
Legend row added. Everything else (layout, edges) unchanged.

**Cross-link re-keying** — every run link listed in Current state points at
`/invocations/$invocationId`; `logs-table` chip reads the renamed log field;
`quick-jump.ts` recognizes UUIDs (36-char hyphenated) in addition to the old
hex shape; command palette lists recent invocations; `sql.tsx targetForCell`
matches `invocation_id`/`cli.invocation.id` column names (drop
`parallax.run.id`).

## Browser verification protocol (binding on every step)

Operator requirement (2026-07-17): each implemented feature is verified in a
real browser against live playground data before the next step starts — not
only at the end. Concretely, after each step below: run `parallax serve` +
the playground corpus (plan 161 scenarios once available, plan 158 sims
otherwise), open the affected pages with the browser-automation tooling, and
check every item of this list, capturing one screenshot per page state into
`docs/research/validation/2026-07-unified-cli-observability/ui/steps/`:

1. Data correctness — every value on screen traceable to the seeded corpus;
   nothing silently missing that the corpus emitted.
2. Links — every row/chip/badge navigates somewhere sensible (no dead ends).
3. States — loading, empty, and error states each seen at least once.
4. Layout — no clipped/overlapping/overflowing elements at 1440px and 375px
   widths; long values (commands, ids, attribute values) truncate with
   tooltips, never break layout.
5. Live behavior — toggles start/stop streams visibly; no duplicate rows; no
   scroll jumping while streaming.
6. Console — zero errors/warnings in the browser console during the walk.

A failed item is fixed (or routed to plan 156/160 when out of this plan's
scope) before proceeding. This same checklist is the usability bar plans 159
and 160 assert.

## Commands you will need

| Purpose | Command | Expected on success |
|---|---|---|
| Install | `cd ui && bun install` | exit 0, `bun.lock` unchanged |
| Typecheck | `cd ui && bun run typecheck` | exit 0 |
| Unit tests | `cd ui && bun run --bun test:ci` | all pass |
| Focused | `cd ui && bun run --bun test:ci -- src/routes/__tests__/-invocations.test.tsx` | pass |
| Lint/format | `cd ui && bun run lint && bun run check` | exit 0 |
| Build | `cd ui && bun run build` | exit 0, route tree regenerated |
| Backend up (manual QA) | `cargo run -p parallax-cli -- serve` (read the real serve command from `crates/parallax-cli` first) | ready banner with UI/GraphQL/OTLP surfaces |

## Scope

**In scope (ui/ only):**
- `src/routes/invocations.index.tsx`, `src/routes/invocations.$invocationId.tsx`
  (new; thin — composition only), delete `src/routes/runs.index.tsx`,
  `src/routes/runs.$runId.tsx`, `src/routes/__tests__/-runs.test.tsx`.
- `src/components/console/invocations/` (new): `invocations-table.tsx`,
  `invocation-status-badge.tsx`, `invocation-header.tsx`,
  `invocation-overview-tab.tsx`, `invocation-traces-tab.tsx`,
  `invocation-logs-tab.tsx`, `invocation-errors-tab.tsx`,
  `sessions-tab.tsx`, `screen-visit-lane.tsx`, `session-journey.tsx`,
  `ui-actions-table.tsx`, `conversations-panel.tsx`, `jobs-cycles-tab.tsx`
  + colocated `__tests__/`.
- `src/lib/api.ts` (types: `Invocation`, `Session`, `ScreenVisit`,
  `UiAction`, `BackgroundCycleSummary`, `Job`, `Conversation`; `LiveSpan`
  and `LOG_FIELDS` field renames), `src/lib/quick-jump.ts`,
  `src/lib/invocation.ts` (new pure helpers: status derivation, duration,
  mode labels — unit-tested).
- Re-keyed consumers: `src/components/nav.ts`, `components/logs-table.tsx`,
  `components/metric-strip.tsx`, `components/console/command-palette.tsx`,
  `routes/traces.$traceId.tsx`, `routes/issues.$fingerprint.tsx`,
  `routes/services.$service.tsx`, `routes/sql.tsx`,
  `routes/ecosystem.tsx` + `components/console/ecosystem-graph.tsx`.
- `src/components/console/agent-session.tsx` — argument re-anchor only
  (invocationId), no redesign.
- `src/routeTree.gen.ts` via the generator (never by hand).

**Out of scope (do NOT touch):**
- Any `crates/**` change — plan 156 owns the backend; if a field you need is
  missing, STOP (see below).
- `features/` directories, TanStack Query, facade architecture — plans
  100/133/140/149/152/153 own that later.
- `/logs`, `/traces` list pages' own behavior (only their run-link fields).
- Dashboards, investigations, tests surface (plan 155), overview page.
- Visual theme, shadcn primitive edits, `src/styles.css` beyond nothing.

## Git workflow

- Work directly on `main` (operator delivery model: no branches, no PRs).
  Conventional Commits, DCO `-s`, one agent trailer,
  push after every durable green commit. Suggested subjects:
  `feat(ui): invocations list and hub replace runs surface`,
  `feat(ui): ecosystem node kinds for cli and browser emitters`.

## Steps

### Step 1: Types and pure model

Add the new interfaces + `lib/invocation.ts` helpers (status: `running` if no
end and last-seen < 5 min; `stale` if no end and older; `failed` if exit ≠ 0
or outcome ∈ {failure,error,timeout}; else `finished`). Rename `LiveSpan`
field and `LOG_FIELDS` entry to the plan-156 wire names. Update
`quick-jump.ts` UUID recognition.

**Verify**: `cd ui && bun run typecheck` → fails ONLY in files this plan will
rewrite next (runs routes + consumers) — record the list; new lib tests pass:
`bun run --bun test:ci -- src/lib/__tests__/-invocation.test.ts`.

### Step 2: List route

Create `invocations.index.tsx` (loader → `invocations` query; zod search
`{q, mode, status, outcome, range, live}`), `invocations-table.tsx` +
`invocation-status-badge.tsx` using the `data-table.tsx` toolbar pattern and
the column set from Product design. Live = 5 s poll via `setInterval` gated
on `live && pageVisible` (copy the pattern from `runs.$runId.tsx:267-284`
before deleting it). Delete nothing yet.

**Verify**: `bun run --bun test:ci -- src/routes/__tests__/-invocations.test.tsx`
→ list renders rows/filters/status/live-toggle from fixture data (model the
test on the old `-runs.test.tsx` fixtures, updated to the new shape).

### Step 3: Hub route

Create `invocations.$invocationId.tsx` with two-stage loading like the old
detail (metadata first, then tab data), `tab`+`live` search params, and the
six tab components. Reuse `use-live-stream` for Traces/Logs tabs with
`?invocation_id=`; master Live switch feeds each tab. Sessions & UI tab and
Jobs & Cycles tab render from the plan-156 fields with explicit empty states.
`session-journey.tsx` builds the journey narrative as a pure function over
(session events, screen visits, actions, errors) — unit-testable without the
DOM; error→screen attribution is a pure interval lookup. Conversations panel
wraps the existing `agent-session` timeline for the selected conversation.
Every table row links onward (trace → `/traces/$id`, issue → `/issues/$fp`,
session → filters logs tab).

**Verify**: component tests per tab (fixture-driven; assert empty states,
links, live-buffer caps at 300, no session/screen data for a one_shot
fixture); `bun run typecheck` → exit 0 except legacy runs files.

### Step 4: Re-key consumers, swap nav, delete runs

Update nav.ts label/href; re-point every cross-link listed in Current state;
ecosystem node kinds + legend; sql `targetForCell`; command palette recent
invocations + UUID jump. Delete both runs routes + their test; regenerate the
route tree via the build.

**Verify**: `grep -rn "runs\.\$runId\|/runs\|runId" ui/src --include='*.ts*' |
grep -v invocation | grep -v routeTree.gen` → no product matches (generated
file may lag until build); `bun run build` → exit 0; `bun run typecheck` →
exit 0; full `bun run --bun test:ci` → pass.

### Step 5: Manual live QA on the real stack

Start GreptimeDB+Turso-backed `parallax serve`, seed via plan 158's playground
(or, if 158 has not yet landed on the playground's main, via the repo's OTLP test fixtures in
`crates/parallax-server/tests/support/harness.rs` replayed with a small
script). Walk: list shows a running daemon invocation with pulsing status →
open hub → Live on → logs/traces tabs stream → sessions tab shows an
interactive session with screen visits → jobs tab shows producer/consumer
attempts → ecosystem shows cli/browser/service kinds. Capture screenshots to
`docs/research/validation/2026-07-unified-cli-observability/ui/` (plan 159
formalizes the full acceptance — this step is the developer smoke).

**Verify**: screenshots exist; no console errors in the browser during the
walk (check devtools console).

## Test plan

- `src/lib/__tests__/-invocation.test.ts` — status derivation matrix
  (running/stale/failed/finished × exit/outcome/last-seen), duration, mode
  labels.
- `src/routes/__tests__/-invocations.test.tsx` — list: merge-free rendering,
  filters, live poll start/stop on toggle+visibility, empty state.
- `src/components/console/invocations/__tests__/` — one file per tab:
  overview stats, traces live prepend cap, logs tab severity filter, errors
  breakdown, sessions Gantt pairing (open session renders "active"), actions
  table links, conversations token totals, jobs attempt outcomes, all empty
  states.
- `-session-journey.test.tsx` — chronological interleave; error attributed to
  the screen whose visit interval contains it; error outside any visit lands
  in the unattributed bucket (never dropped); widget context rendered when
  present; every entry's link target.
- Ecosystem: extend the existing ecosystem test with node kinds + cli-node
  link target.
- Pattern exemplar: the old `-runs.test.tsx` fixture style; timers via
  `src/test/timers.ts`; network via `src/test/network.ts`.

## Done criteria

- [ ] `cd ui && bun run typecheck && bun run lint && bun run check` all exit 0.
- [ ] `cd ui && bun run --bun test:ci` passes; new tests cover every tab and
  the status matrix.
- [ ] `cd ui && bun run build` exits 0 with regenerated route tree containing
  `/invocations` and not `/runs`.
- [ ] Step-4 grep shows zero `runId`/`/runs` product references.
- [ ] `parallax.run.id` appears nowhere in `ui/src` except (possibly) the
  generated semconv legacy constant (`grep -rn "parallax.run.id" ui/src`).
- [ ] Screenshots recorded under
  `docs/research/validation/2026-07-unified-cli-observability/ui/` including
  the per-step protocol captures (`ui/steps/`), all six checklist items
  passing for every tab.
- [ ] Journey view answers screen→screen→error attribution from seeded data
  (browser-verified capture included).
- [ ] `plans/README.md` status row updated.

## STOP conditions

Stop and report back (do not improvise) if:
- A hub tab needs a GraphQL field plan 156 did not deliver (missing
  argument, missing aggregate) — the fix belongs in 156 on the same branch,
  not in an ad-hoc UI workaround (no client-side joins over `sql`).
- The SSE endpoints reject `invocation_id` (156 step 6 incomplete).
- The route-tree generator produces diffs outside the expected new/deleted
  routes.
- Reusing `agent-session.tsx` requires changing its rendering contract (its
  tests must keep passing unchanged apart from the id argument).
- Any change would touch shadcn primitives under `src/components/ui/` or
  `src/styles.css`.

## Maintenance notes

- Plan 140 (retitled to the invocations feature migration) later moves these
  files behind `features/invocations` unchanged — keep components pure and
  route files thin so that move is mechanical.
- Plan 147 will re-own the live merge/buffer behavior; keep this plan's live
  code inside the existing `use-live-stream` pattern so 147's swap is local.
- Reviewer focus: no dead-end views (every row links onward), empty states
  for observed-only invocations, live toggles never leak timers/EventSources
  on unmount or tab-hide (assert cleanup in tests).
