# Parallax UI design guide

Canonical visual language for `ui/`. Binding agent rules stay in
[`../AGENTS.md`](../AGENTS.md); this file is the vocabulary those rules point at.
Tokens come from [`src/styles.css`](src/styles.css). Domain color records
live in [`src/shared/colors.ts`](src/shared/colors.ts) — there is no
`signal-colors.ts`.

Foglamp (github.com/foglamp-labs/foglamp, Apache-2.0) informed the
elevation, loading, and one-record-per-concept patterns. No foglamp source
was copied; do not add NOTICE attribution for ideas.

## 1. Principles

- **Telemetry owns the chroma.** The canvas is neutral oklch gray. Color
  is reserved for severity, service identity, RED/percentile series, and
  the domain records in `colors.ts`.
- **Density with hierarchy.** Linear-class: tight rows, `text-sm` body,
  page titles at `text-base font-medium tracking-tight`.
- **Dark is re-lit, not inverted.** Dark tokens raise lightness slightly
  and rebuild elevation as inset bevel + white-alpha ring + stacked drops.
- **Every number is `tabular-nums`** (or `.tabular-stack`) so columns
  align.
- **Every chart/list links onward.** Issue → trace → logs; never a dead
  end (`AGENTS.md` rule 17).
- **Motion is functional.** ≤150ms, opacity/transform only, and it
  respects `prefers-reduced-motion`. Content replacing a skeleton uses
  `.content-enter` (150ms). Hard load uses `.page-fade` (75ms, once).
  Never animate row height. No spinner for a sub-second load.
- **Structure is shared.** Query bars, detail layouts, correlation links,
  error states, and shortcuts come from `shared/` (§8). Pages own state
  and data, never structure.

## 2. Tokens

Values must grep-match `src/styles.css`. Theme only through these
variables (`AGENTS.md` rule 11).

### Canvas (light / dark)

| Token | Light | Dark |
| --- | --- | --- |
| `--background` | `oklch(0.995 0 0)` | `oklch(0.17 0 0)` |
| `--foreground` | `oklch(0.145 0 0)` | `oklch(0.985 0 0)` |
| `--card` | `oklch(1 0 0)` | `oklch(0.205 0 0)` |
| `--border` | `oklch(0.922 0 0)` | `oklch(1 0 0 / 10%)` |
| `--muted` | `oklch(0.97 0 0)` | `oklch(0.269 0 0)` |
| `--muted-foreground` | `oklch(0.556 0 0)` | `oklch(0.708 0 0)` |
| `--destructive` | `oklch(0.577 0.245 27.325)` | `oklch(0.704 0.191 22.216)` |
| `--success` | `oklch(0.55 0.15 155)` | `oklch(0.72 0.14 155)` |
| `--warning` | `oklch(0.75 0.15 80)` | `oklch(0.8 0.15 80)` |
| `--info` | `oklch(0.62 0.15 245)` | `oklch(0.7 0.15 245)` |
| `--chart-1` | `oklch(0.87 0 0)` | `oklch(0.87 0 0)` |
| `--radius` | `0.625rem` | `0.625rem` |
| `--border-hairline` | `1px` (`0.5px` at ≥192dpi) | same |

### Severity ramp (axis 1)

`--severity-trace|debug|info|warn|error|fatal`. Light starts
`oklch(0.65 0.01 260)` … `oklch(0.45 0.19 20)`. Dark is slightly brighter.
**Log severity and error/incident state only.** Always pair color with
the literal severity word.

### Percentile / RED (axis 3)

`--chart-p50` `oklch(0.62 0.15 245)`, `--chart-p95` `oklch(0.75 0.15 80)`,
`--chart-p99` `oklch(0.62 0.2 40)`, `--chart-error` `oklch(0.6 0.21 25)`,
`--chart-throughput` `oklch(0.7 0.12 200)` (dark variants raise
lightness). Latency and RED series only. Generic series use
`--chart-1..5` or `seriesColor(name, index)`. Many-series telemetry keeps
the dataviz / golden-angle palette — do not flatten charts onto the
grayscale `--chart-*` tokens.

### Elevation vs border

| Use | Token | Surfaces |
| --- | --- | --- |
| Floating card / panel | `--elevation-1` | `Card`, stat cards |
| Raised overlay | `--elevation-2` | popovers, command palette |
| Accent ring on a floating control | `--elevation-destructive\|success\|warning\|info` | destructive/success buttons |
| Inline structure | `--border` / `--border-hairline` | tables, inputs, dashed empty-state frames |
| Existing control shadows | `--custom-shadow*` | shadcn buttons already wired |

Do **not** ban borders repo-wide. Tables stay border-based so dense
numeric grids keep readable column edges.

Radius scale (`@theme`): `--radius-sm` = `calc(var(--radius) - 4px)`
through `--radius-4xl` = `calc(var(--radius) + 16px)`.

## 3. Typography

| Role | Classes | Exemplar |
| --- | --- | --- |
| Page title | `text-base font-medium tracking-tight` | `shared/components/page-header.tsx` |
| Body | `text-sm` | page description, cards |
| Meta | `text-xs text-muted-foreground` | hints, relative time |
| Micro | `font-mono text-xs tabular-nums` | IDs, SQL, snippet code |

Fonts: Inter Variable (`--font-sans`), Geist Mono Variable
(`--font-mono`). Heading face is the sans.

## 4. Loading and entrance

- Skeletons appear only after **700ms** (`useDelayedLoading` in
  `shared/console/hooks.ts`).
- One **75ms** opacity-only `.page-fade` per hard load
  (`shared/page-fade.tsx`, module-scope gate). Skipped under
  `prefers-reduced-motion`.
- Content replacing a skeleton uses `.content-enter` (150ms, opacity
  only).
- Table skeletons render `table-fixed` rows that mirror column
  alignment (`shared/console/skeletons.tsx`). Logs / traces / issues
  tables set `table-fixed` so widths do not jump on skeleton → data.

## 5. Color maps

Normative module: `src/shared/colors.ts`.

| Record | Keys | Use |
| --- | --- | --- |
| Severity ramp helpers | `trace..fatal` | logs / incidents only |
| `serviceColor(name)` | 120-slot hash | service **identity** only |
| `SPAN_STATUS` | `ok` `error` `unset` | span status |
| `INVOCATION_STATUS` | `running` `finished` `failed` `stale` | CLI invocation |
| `INVOCATION_OUTCOME` | `success` `skip` `cancellation` `error` | invocation outcome |
| `TEST_ROLLUP` | `PASSED` `FLAKY_PASS` `FAILED` `BROKEN` `SKIPPED` `UNKNOWN` | test explorer |
| `TEST_RESULT` | `PASSED` `FAILED` `BROKEN` `SKIPPED` `UNKNOWN` | attempt status |
| `TEST_FLAKY` | `HEALTHY` `FLAKY` `FIXED` `BROKEN` | flaky state |
| `ISSUE_STATUS` | `open` `resolved` | issue state |
| `INCIDENT_STATUS` | `open` `resolved` | alert incident state |

Each domain record is a `DomainTone`: `color` (CSS var) + `badge` /
`bar` / `chip` / `icon` (Tailwind classes). One record drives every
surface for that concept. Error/incident may use the severity ramp;
success / running / skip use `--success` / `--info` / `--muted`, never
`--severity-info`.

Waterfall color-by (`shared/color-by.ts`) reads `SPAN_STATUS.*.color`.

## 6. Component rules

| Kind | Rule | Exemplar |
| --- | --- | --- |
| Buttons | shadcn `Button`; primary actions already `active:scale-[0.97]`; Base UI `render` not `asChild` | `components/ui/button.tsx` |
| Badges | status/type from the domain record, never a one-off `text-rose-600` | `TEST_ROLLUP[rollup].badge` |
| Tables | TanStack Table + shadcn `<Table>` split; `table-fixed` on logs/traces/issues; borders stay | `components/ui/table.tsx` |
| Empty states | dashed frame + 40% icon + what is missing and what would produce it; zero-data overview/issues use `SnippetTabs`; inline next step via `action` | `shared/console/empty-state.tsx`, `snippet-tabs.tsx` |
| Error states | `ErrorState` (section) / `SectionError` (inline row); `role=alert`; retry when the fetch is user-repeatable; never bare destructive text | `shared/console/error-state.tsx` |
| Query bars | `QueryBar` → `QueryBarRow` × ≤2 → `QueryBarCount`; order in §8.1 | `shared/console/query-bar.tsx` |
| Detail pages | `PageHeader back` → `DetailSummary` → tabs → `SectionCard`s; structure in §8.2 | `shared/console/detail-layout.tsx` |
| Correlation links | range-carrying `TraceLink` `IssueLink` `ServiceLink` `LogsLink` `InvocationLink` `MetricLink`; never hand-build `Link` + `rangeLinkSearch` | `shared/console/entity-links.tsx` |
| Shortcuts | registry in `shared/keyboard.ts`; `?` dialog renders from it; rules in §8.4 | `shared/keyboard.ts`, `shared/console/shortcuts-dialog.tsx` |
| Detail panels | `Card` at `--elevation-1`; charts in `ChartContainer` with `--chart-*` | `shared/console/stat-card.tsx` |
| Stat cards | Volume → Health → Performance → Cost; ticker respects reduced-motion; sparkline dashes the incomplete last bucket; delta badges invert for error-rate/latency | `stat-card.tsx` |
| Nav | per-section hue chip + outline→filled 100ms crossfade (`NavIcon`); AA contrast held by the existing chip tints | `shared/navigation.ts`, `layout/nav-icon.tsx` |
| Graphs | React Flow only (`AGENTS.md` rule 24) | ecosystem map |

Empty-state voice: "No sessions yet — this invocation emitted no
session.start event". Never marketing copy.

## 7. Modern-practice appendix (2026-08)

Living section. Re-verify on the next major redesign.

| Practice | Why we adopted it | Source |
| --- | --- | --- |
| oklch + CSS-first theming | perceptual uniformity; one variable set for light/dark; no JS theme math | [CSS Color Module Level 4](https://www.w3.org/TR/css-color-4/#ok-lab); shadcn Vega/oklch default |
| Shadow elevation | floating surfaces separate without competing with table hairlines | foglamp elevation tokens (Apache-2.0 ideas); Material elevation as the older reference |
| Reduced-motion compliance | vestibular safety; our motion is opacity/transform only | [WCAG 2.2 2.3.3](https://www.w3.org/WAI/WCAG22/Understanding/animation-from-interactions.html) |
| Density-first dashboards | operators scan numbers, not marketing cards | Linear / internal-tool convention; our `text-sm` + compact tables |
| Command-palette-centric nav | keyboard-first (`⌘K`) for a local console | existing `layout/command-palette.tsx` |
| APCA-aware contrast | WCAG AA as the floor; pair color with a word so chroma is never the sole signal | [APCA](https://www.myndex.com/APCA/); plan-162 severity-word rule |

### Changelog

- 2026-08-14 — plan 172: first published guide; elevation tokens;
  snippet tabs; domain records on `colors.ts`.
- 2026-09-13 — workstream D: §8 interaction contract (`QueryBar`,
  `DetailSummary`/`SectionCard`, entity links, `ErrorState`,
  keyboard registry); `ISSUE_STATUS`/`INCIDENT_STATUS` records;
  `seriesColor("ok"|"success")` fixed to `--success`; Tests moved to
  Workspace nav; `EmptyState action`; legend `aria-pressed`.
- 2026-09-13 — `ISSUE_STATUS.regressed` (warning tone); skip-link on
  `app-shell` `#main-content`; `LogsLink` carries `trace`.
- 2026-09-13 — R1 source maps: issue-detail stacktrace renders
  server-resolved `mappedFrames` with an `N/M frames mapped` badge
  (generated position kept as fallback/title); `SourceMapCard`
  lists artifacts + uploads `.map` files for the event's
  `(service, version)` when frames are unmapped.
- 2026-09-14 — R5: release strip renders crash-free session % per
  version from `releaseHealth`; suspect releases get destructive tone +
  `N suspect` badge (word, not color alone).
- 2026-09-13 — R2 `/pipeline` route (workspace nav): sampling-policy,
  drop-reason, and queue-watermark cards; uncached loader (live counters);
  exact-count `bigint` mappers (`formatCount`, `totalDropped`).


## 8. Interaction contract

### 8.1 Query bars

Every list page composes `QueryBar` with at most two `QueryBarRow`s at
1280px. Order, fixed:

1. Primary row: one free-text search, then the structured filter
   (`WhereClauseEditor` where the signal has one).
2. Secondary row: faceted selects → toggles → view actions (columns,
   patterns, saved views) → `QueryBarCount` pinned right.

Exactly one free-text input per page. Range stays in `PageHeader actions`.

### 8.2 Detail pages and progressive disclosure

Order, fixed: `PageHeader` (with `back`) → `DetailSummary` (the answer:
counts, first/last seen, status, release — above the fold) → tabs →
`SectionCard`s with stable `id` anchors. Power panels (attribute compare,
field explorer, metric strips) live on their tab, never crowd the primary
answer. `PageHeader` descriptions must carry a non-obvious hint or be
omitted — never restate the nav label.

### 8.3 Loading, error, and live states

- Table skeletons MUST pass mirror `columns` widths for the real table;
  the 4-column default is a placeholder, not a mirror.
- Every user-triggered fetch (retry, load-older, saved-view IO, live
  reconnect) renders `SectionError` on failure; section loads render
  `ErrorState`. Never bare destructive text.
- Live surfaces converge on `LiveStreamPanel`; the connect/reconnect badge
  carries `role=status`. Pulse dots use `motion-safe:`.

### 8.4 Keyboard

- All shortcuts live in the `shared/keyboard.ts` registry; no per-page
  `keydown` handlers. Registry additions update `SHORTCUT_LIST` (the `?`
  dialog renders from it) and this section.
- Canonical set: `⌘K` palette, `/` search, `F` structured filter, `?`
  help, `Esc` clear/close. Plain-letter shortcuts never fire while
  typing; mod/Esc fire everywhere.
- Suggestion lists must not hijack `Tab`: `Tab` moves focus. `Enter`
  applies/accepts; arrow keys move the highlight.
- List rows open via keyboard (`Enter` on the focused row; `j/k` move
  between rows where virtualized lists support it). Shell provides a skip
  link to main content.

### 8.5 Encoding and text accessibility

- Toggle legends set `aria-pressed` (`trend.tsx` does this when `selected`
  is provided).
- No sub-100%-alpha `muted-foreground` text below `text-sm`, except
  decorative marks that are `aria-hidden` or redundant with adjacent text.
- `ToggleChip` tone must come from the domain record for the toggled
  concept; the hardcoded rose is error-only.
- Nav hue chips are under review (see `docs/research/ux/05`): chrome must
  not out-color telemetry. Decision pending visual sign-off.

## Browser verification checklist

Every UI change against playground data before the next step:

1. **Data correctness** — values match GraphQL/source for the visible window.
2. **Links** — every row/chip navigates to a real detail surface; range survives.
3. **States** — empty, loading (skeleton), error (with retry), and live/polling each render.
4. **Layout** — no overflow/clip at default density; table numerals align.
5. **Theme** — light and dark both readable (severity words + contrast).
6. **Motion** — no layout shift on refresh; reduced-motion stays usable.
7. **Keyboard** — `?` lists working shortcuts; rows open via keyboard; no `Tab` traps.
