# 02 — Query workflows

## Current state: 3 query idioms

| Page | Text query | Structured filter | Apply gesture |
| --- | --- | --- | --- |
| Logs (`features/logs/.../logs-page.tsx:461-486`) | body `q` form | `WhereClauseEditor` | Enter / ⌘Enter |
| Traces (`features/traces/.../traces-page.tsx`) | `q` + lookup | where + duration + facets | mixed |
| Issues (`features/issues/.../issues-page.tsx:157-212`) | instant `SearchInput` | service/status selects | instant |
| Metrics (`routes/metrics.index.tsx:88-109`) | local-state text | kind select | instant, local-only |

## Findings

1. **No shared query bar.** Each page hand-stacks 4–7 controls; order,
   density, and apply gestures differ. Users relearn each signal.
   Fix shipped: `shared/console/query-bar.tsx` — one composition
   (`QueryBar`, `QueryBarPrimary`, `QueryBarSecondary`) enforcing order:
   search → structured filter → faceted selects → view actions → range.
   Contract in `DESIGN.md` §8.1. Adoption is C's (features-owned).
2. **Two overlapping text inputs on Logs.** Body-`q` (`logs-page.tsx:468`)
   and where-clause (`:474`) sit side by side; nothing explains which to use
   when. `QueryBar` contract: exactly one free-text input per page; where
   editor owns structured predicates. C to merge or label.
3. **`f` focuses filter via copy-pasted handlers** (`logs-page.tsx:226-242`
   ≡ `traces-page.tsx:335-351`). Enabling condition: no shared shortcut
   primitive. Fix shipped: `shared/keyboard.ts`
   (`useFilterFocusShortcut`, `useShortcut`, `SHORTCUTS` registry). C to
   replace both handlers (drop-in).
4. **Tab hijack in where editor** (`shared/console/where-clause-editor.tsx:133`).
   Tab accepts a suggestion instead of moving focus — keyboard trap smell,
   breaks form navigation. Enter already applies; Tab must move focus.
   Recommend C change: Tab accepts only when listbox open AND user arrowed
   (track `touchedByArrow`), else default. Spec'd in `DESIGN.md` §8.4; the
   file is D-owned but behavior change needs verifier pass with C — left
   deliberate, flagged here.
5. **Where-syntax undiscoverable.** Only a placeholder shows grammar; parse
   errors read `"(at 14)"` (`where-clause-editor.tsx:172-176`). Ship `?`
   shortcuts dialog (done, `shortcuts-dialog.tsx`); where grammar cheatsheet
   still wanted — C, small popover.
6. **Saved views only on Logs** (`logs-page.tsx:499+` menu); traces/issues/
   metrics lack them. Query history nowhere. Both are P1 differentiators
   (Grafana/Sentry have both). Backend exists (`savedViews` GQL); C to reuse
   the Logs pattern via a shared hook later.
7. **No copy-link.** State lives in URL (good) but no affordance. Cheap C add:
   `CopyButton` with `window.location.href` in `PageHeader actions`.

## Time-to-first-answer (GOAL §7)

"Which logs belong to this span?" — today: copy span id → Logs → paste `q`
→ submit. With entity links (shipped, `entity-links.tsx`) it becomes one
click once C wires span→logs links. Same for "which trace explains this
metric spike" (exemplar links exist on service RED charts,
`service-red-charts.tsx:181`, but not on metric detail — C).
