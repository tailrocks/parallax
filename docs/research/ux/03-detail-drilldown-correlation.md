# 03 — Detail pages, drill-down, cross-signal correlation

## Current state: 3 detail idioms, no shared layout

| Detail | Structure | Verdict |
| --- | --- | --- |
| Issue (`features/issues/.../issue-detail-page.tsx`, 745 lines) | single column, 7+ full-width cards, occurrences last (`:320-327`) | structurally inferior |
| Trace (`features/traces/.../trace-detail-page.tsx`, 1493 lines) | Waterfall/Story tabs (`:558-559`) + tree/errors/lanes/flame modes (`:130`) | best; keep, converge others toward it |
| Invocation (`.../invocation-hub-page.tsx`) | 6 tabs (`:375-380`) | good |
| Service (`.../service-detail-page.tsx`, 194 lines) | flat cards, no tabs | thin; fine for now |

## Findings

1. **Issue detail buries the answer.** Order today: header → attributes →
   stacktrace → metric strip → tags → correlation → agent handoff →
   occurrences. "How many, since when, which release, getting worse?" needs
   full-page scroll; occurrences (the event list) sit at the bottom.
   Fix: `DetailSummary` (shipped, `shared/console/detail-layout.tsx`) —
   first/last seen, event count, trend delta, service, release, status —
   above the fold; tabs (Occurrences / Stacktrace / Correlation / Metrics /
   Tags) per trace/invocation idiom. Contract `DESIGN.md` §8.2. C adopts.
2. **No shared summary/correlation strip.** Every detail hand-builds header
   stats and correlation links. Enabling condition, same class as (1).
   Shipped: `DetailSummary` + `SectionCard` (deep-linkable sections) +
   `shared/console/entity-links.tsx` (`TraceLink`, `IssueLink`,
   `ServiceLink`, `LogsLink`, `InvocationLink`, `MetricLink`) — all carry
   range via `rangeLinkSearch`, killing per-page hand-built links
   (e.g. `issues-page.tsx:323-346`, 6 links × `stopPropagation`).
3. **Correlation good where present, uneven.** `CorrelationCard`
   (`issue-detail-page.tsx:301`) and RED exemplar links
   (`service-red-charts.tsx:181`) are right. Gaps for C: log row → trace
   exists (`logs-table.tsx:379`) but span → surrounding-logs missing;
   metric detail → exemplars/traces missing; issue tags not clickable
   (`issues-page.tsx:407-418` render dead badges — dead-end, make them
   filter links).
4. **Row click + nested links pattern** (`issues-page.tsx:313-319`): whole row
   `onClick`, inner `Link`s `stopPropagation`. Mouse-fine, keyboard-poor
   (see 06). Contract: rows keep inner-link reachability AND get Enter-to-open
   via row action (C + `shared/keyboard.ts`).
5. **Trace detail is the model.** View modes + color-by (`trace-detail-page.tsx:762-810`)
   + evidence gaps + compare panels answer "where/why slow or failed" without
   raw JSON. Only gap: 1493-line file = comprehension cost for maintainers,
   not users. No D action.
6. **Service detail thin.** No RED-at-a-glance ordering guarantee; release
   strip exists (`service-release-strip.tsx`). Fine once `DetailSummary`
   adopted.

## GOAL §7 spot checks (clicks to answer)

- "What happened before this exception?" — Logs anchor context exists
  (`logs-page.tsx:272-283`) but issue→logs-around-event link missing. C: one
  `LogsLink` with anchor window.
- "Which release introduced this error?" — needs summary + release strip on
  issue detail (C, after `DetailSummary`).
- "Did browser, backend, or network fail?" — no frontend RUM surface yet;
  outside UI-design scope, flagged for product.
