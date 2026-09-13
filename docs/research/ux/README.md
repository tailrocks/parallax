# UX research: workstream D

Independent product/interaction audit of Parallax UI vs GOAL.md §2D/§7/§9
(dense, fast, keyboard-friendly, navigable, understandable, consistent,
accessible, light+dark). Code is truth; paths relative to `parallax/ui/src/`.

## Verdicts

| Area | Verdict | Doc |
| --- | --- | --- |
| IA / navigation | signals interrupted by Tests; 13 flat items; palette dead-ends on 3 pages | [01](01-ia-navigation.md) |
| Query workflows | 3 query idioms; Tab hijack; `f` shortcut copy-pasted; no history | [02](02-query-workflows.md) |
| Detail / drill-down | 3 detail idioms; issue page is 7-card scroll; no shared summary/correlation strip | [03](03-detail-drilldown-correlation.md) |
| States | empty good; loading generic; section errors are bare text, no retry | [04](04-states.md) |
| Visual encoding / themes | ramp sound; 2 axis violations in own code; nav chroma fights principle | [05](05-visual-encoding-themes.md) |
| Keyboard / a11y | ⌘K only global; rows mouse-only; no skip link; dim text fails AA | [06](06-keyboard-a11y.md) |
| Cognitive load / density | toolbar sprawl; where-syntax undiscoverable; shell padding fights density | [07](07-cognitive-load-density.md) |

## Structural fixes shipped (workstream D owns `ui/src/shared/` + `ui/DESIGN.md` only)

- `shared/keyboard.ts` — canonical shortcut registry; kills per-page `f` handlers.
- `shared/console/shortcuts-dialog.tsx` — `?` help surface over the registry.
- `shared/console/query-bar.tsx` — one query-toolbar composition order.
- `shared/console/detail-layout.tsx` — `DetailSummary` + `SectionCard`; one detail idiom.
- `shared/console/entity-links.tsx` — range-carrying cross-signal links; kills hand-built correlation links.
- `shared/console/error-state.tsx` — `ErrorState`/`SectionError`; errors get retry + `role=alert`.
- `shared/console/empty-state.tsx` — `action` slot.
- `shared/colors.ts` — `seriesColor("ok")` no longer steals severity ramp; new `ISSUE_STATUS` / `INCIDENT_STATUS` (`open|resolved`) records.
- `shared/navigation.ts` — Tests moved out of signal flow into Workspace.
- `ui/DESIGN.md` — §8 interaction contract; §5/§6 updated; changelog.

## Handed to workstream C (outside D file ownership)

1. `layout/command-palette.tsx:70-86` — `pageRoute` allowlist drops `/tests`, `/metrics`, `/alerts`; palette lists them but `goPage` silently no-ops. Dead-end; add the three routes.
2. Adopt `QueryBar`, `DetailSummary`/`SectionCard`, `EntityLinks`, `ErrorState` in `features/` + `routes/` (per-doc adoption lists).
3. `features/traces/components/trace-waterfall.tsx`, `features/issues/.../issues-page.tsx:313` — row keyboard activation (`j/k` + Enter), needs feature-side work on top of `shared/keyboard.ts`.
4. `routes/alerts.index.tsx:92`, `features/issues/.../issues-page.tsx:423` — move badges onto `ISSUE_STATUS` / `INCIDENT_STATUS`.
5. Nav chroma reduction (13 hue chips vs "telemetry owns the chroma") — decision + mock in [05](05-visual-encoding-themes.md); needs visual sign-off before touching `navigation.ts` chips.
