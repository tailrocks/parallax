# 04 — Empty / loading / error / live states

## Empty: good, one gap

Voice follows contract ("No sessions yet — …", `DESIGN.md` §6) and zero-data
pages use `SnippetTabs` (`issues-page.tsx:220`, overview). Gap: `EmptyState`
(`shared/console/empty-state.tsx`) renders an always-empty `EmptyContent` —
no action slot, so pages can't offer "Clear filters" / "Open docs" inline.
Fix shipped: optional `action` prop.

## Loading: generic skeletons

- `TableSkeleton` (`shared/console/skeletons.tsx`) is a fixed 4-column shape;
  callers with 8 columns (issues) show a shape that doesn't mirror data,
  despite `DESIGN.md` §4 claiming mirroring. The `columns` prop exists —
  callers just don't pass it. Contract fix (`DESIGN.md` §8.3): list pages
  MUST pass mirror widths; `table-fixed` already set on issues
  (`issues-page.tsx:443`).
- `useDelayedLoading` 700ms gate (`shared/console/hooks.ts`) is right; keep.
- `RoutePendingPanel` (`layout/route-boundaries.tsx:123`) shows header + 3
  cards + spinner — fine.
- No skeleton for detail summaries; `DetailSummary` accepts `loading` and
  renders muted placeholders (shipped).

## Error: structurally weakest state

- Route-level: `RouteErrorPanel` good (401 token flow + retry,
  `route-boundaries.tsx:88-121`).
- Section-level: nothing shared. Logs renders bare `<p class=text-destructive>`
  for saved-view and older-load failures (`logs-page.tsx:519,671-672`) — no
  retry, no `role=alert`, no icon. Traces/issues/services have no visible
  section-error path at all (loader throw → whole route dies).
- Fix shipped: `shared/console/error-state.tsx` — `ErrorState` (icon +
  message + Retry, `role=alert`) and `SectionError` (compact inline row
  with retry). C adopts at both `logs-page` sites + trace/issue loaders.
- Contract (`DESIGN.md` §8.3): every user-triggered fetch (retry, load-older,
  live reconnect) renders `SectionError` on failure; never bare text.

## Live: two idioms, one dead component

- `LiveStreamPanel`/`LiveEventStack` (`shared/console/live-stream-panel.tsx`)
  have zero callers — dead shared code. Meanwhile logs/traces each hand-roll
  a Query/Live toggle (`logs-page.tsx:412-421`). Converge or delete: keep the
  panel as the specified live surface (contract §8.3), C wires logs/traces
  live mode to it; if C refuses, delete the file (no-legacy rule).
- Live status must be announced: `role=status` on connect/reconnect badge.
  Specified in contract; implementation in C's live wiring.
- `animate-pulse` dot (`live-stream-panel.tsx:37`) needs
  `motion-safe:` guard — D-owned file, but class-only; folded into C's live
  adoption to avoid churn. Flagged, not forgotten.
