# 01 — Information architecture & navigation

## Current IA (`shared/navigation.ts:36-144`, `layout/app-shell.tsx:190-208`)

Primary (unlabeled): Overview Issues Tests Traces Ecosystem Logs Metrics Services.
Workspace: CLI Apps Alerts Dashboards Investigations SQL.

## Findings

1. **Signal flow interrupted.** Issues→Traces→Logs→Metrics→Services is the
   investigation order (GOAL §9 chain), but Tests sits between Issues and
   Traces and Ecosystem splits Traces/Logs. Tests is verification, not a
   signal; Ecosystem is a map view over Services.
   Fix shipped: Tests moved to Workspace front (`shared/navigation.ts`).
   Safe: `app-shell.tsx:193-200` renders both arrays generically; shell test
   asserts presence, not order (`layout/tests/application-shell.test.tsx:35`).
2. **Flat 13 items, one label.** Primary group has no label; "Workspace"
   mixes execution (CLI Apps), verification (Tests), authoring (Dashboards,
   SQL), response (Alerts, Investigations). 13 items exceed sidebar scan
   (~7±2 per group). Deeper regroup needs `app-shell.tsx` (not D-owned);
   proposed: Observe / Map / Work. Left for C with this doc as spec.
3. **Palette dead-ends (bug, C-owned).** `layout/command-palette.tsx:70-86`
   `pageRoute` allowlist omits `/tests`, `/metrics`, `/alerts`, yet
   `nav.map` (`:281`) lists all 13. Selecting those three silently no-ops.
   Violates "every link goes somewhere useful". One-line fix in C.
4. **Detail back-links good.** `PageHeader back` (`shared/components/page-header.tsx:39`)
   + `navItem` used by all detail pages. Keep.
5. **Dashboards submenu** (`app-shell.tsx:88-113`) capped at 7 + "All" — right
   call; only loads on `/dashboards*` so no global cost. Keep.
6. **No breadcrumbs beyond one level.** Issue→trace→span depth loses trail.
   Cheap fix later: `DetailSummary` + browser back suffice for now; revisit if
   verifier complains.

## Hierarchy/density notes

- `ShellMain` (`app-shell.tsx:147`) `max-w-380 p-10 2xl:p-16`: padding fights
  the dense bar on list pages. Recommend `p-6 2xl:p-10` (C-owned file).
- Page titles correct per contract (`text-base`, `page-header.tsx:54`).

## Adoption (C)

- Add 3 routes to palette allowlist.
- Consider shell padding cut + 3-group nav (spec above).
