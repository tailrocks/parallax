# Reference UI Transfer For Parallax

> **Status (2026-07-17): implemented direction.** The current TanStack Start UI
> uses the custom token theme, shadcn/Base UI, React Flow + ELK for ecosystem
> graphs, Recharts for charts, and the shared feature/platform boundaries. The
> follow-up bar below guides refinement rather than describing an unbuilt UI.

Research date: 2026-07-03

## Brief

The operator wants Parallax to keep its current product behavior, framework, Tailwind setup, and shadcn/Base UI component stack, but move the web UI/UX much closer to the selected visual reference. The target is not feature cloning or product semantics. It is design transfer: dark product shell, compact observability cards, colored signal accents, dense timelines, and live streaming telemetry surfaces.

## Sources

- Visual reference homepage and docs, fetched 2026-07-03.
- Visual reference dashboard overview, trace view, and live HUD docs, fetched 2026-07-03.
- Operator design brief: `/Users/donbeave/.codex/attachments/5bd281b7-e93b-41e9-84a9-c8dcfe32d9cf/pasted-text-1.txt`

## Design Observations

The visual reference presents observability as a dense dark product console, not as generic admin CRUD. The homepage emphasizes a black shell, translucent top navigation, overlapping colored logo dots, compact pill actions, raised dark panels, and bright blue/orange/pink/green signal accents.

The dashboard overview docs define the home surface around KPI cards, deltas against a previous window, trend charts, and breakdown tables by model, agent, workflow, and customer. The important UX pattern is "answer what is happening right now, then let the user drill down."

The trace docs describe the main debugging surface as a waterfall with span rows, proportional bars, color-coded span types, status tinting, and an inspector beside the waterfall. That maps directly to Parallax traces, runs, logs, and issue evidence.

The Live HUD docs describe local SSE streaming of execution steps/tool calls/tokens/cost into a floating dev overlay. For Parallax, the matching concept is live run/log/trace tails from the local SSE endpoints, represented as connected pills plus a compact newest-first event stack.

## Parallax Mapping

| Reference concept | Parallax equivalent | UI rule |
| --- | --- | --- |
| Cost, latency, quality KPI cards | issues, traces, logs, runs, services, metrics | Put four compact KPI cards near every major page heading. |
| Agent/workflow breakdowns | services, runs, dashboards, issue groups | Use dense panels and tables with colored badges and small metadata chips. |
| Trace waterfall + inspector | trace detail, run detail, issue latest event | Use proportional bars, click-through links, and side/detail panels without changing data semantics. |
| Live HUD SSE broker | `/v1/logs/stream`, `/v1/traces/stream` | Live mode must look native: connected pill, endpoint chip, count chip, event stack. |
| Dark console shell | Parallax app shell | Use a persistent dark sidebar/header and raised content panels. |

## Implemented Direction

- Global theme now defaults to a dark reference-style product surface with Parallax-owned brand variables.
- The shell now uses colored icon tiles, active nav pills, a dark sidebar, and a translucent top header.
- Shared presentational primitives were added:
  - `PageHeading` for page title/description/action composition.
  - `KpiCard` for compact metric/status cards.
  - `LiveStreamPanel` and `LiveEventStack` for live run/log/trace streams.
  - branded route fallback surfaces for loading/error/not-found states.
- Issues, runs, logs, traces, services, dashboards, SQL, and detail pages now use dark raised panels, compact status chips, and shared heading/KPI vocabulary while keeping existing loaders, routes, GraphQL calls, and mutations.

## Constraints Preserved

- No backend/API files changed.
- No route paths changed.
- No package installs.
- UI remains TanStack Start + Tailwind v4 + shadcn/Base UI.
- Product semantics stay Parallax-specific: grouped issues, traces, logs, services, runs, dashboards, SQL, metric windows, and evidence bundles.

## Follow-Up Design Bar

UI refinements compare against these reference-derived patterns:

1. Every page starts with a concise heading and dense KPI summary.
2. Data is framed in dark raised panels, not loose text or raw tables.
3. Important status is color-coded with semantic badges/dots.
4. Live telemetry is represented as connected streams with newest-first event stacks.
5. Drilldowns preserve object context: issue -> trace -> logs; run -> traces/logs/bundle; dashboard -> metric widget.
