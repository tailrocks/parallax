# 06 — Keyboard workflows & accessibility

## Keyboard: one global, the rest ad-hoc

| Shortcut | Scope | Status |
| --- | --- | --- |
| ⌘K/Ctrl+K palette | global (`layout/command-palette.tsx:66-68,133-141`) | good; keep |
| `f` focus filter | logs (`logs-page.tsx:226-242`), traces (`traces-page.tsx:335-351`) | identical copy-paste; no other pages; undiscoverable |
| ⌘Enter apply | where editor (`where-clause-editor.tsx:110`) | fine, hinted (`:144`) |
| `?` help, `/` search, `j/k` rows, `Esc` clear | — | missing everywhere |
| Palette toggle-off | — | `onOpenChange(!open)` (`:137`) reopens races; minor |

Fix shipped: `shared/keyboard.ts` — `SHORTCUTS` registry
(`palette`, `focus-filter`, `focus-search`, `show-shortcuts`, `clear-close`),
`matchesShortcut`, `useShortcut` (ignores typing targets, skips with
modifiers unless specified), `useFilterFocusShortcut` (drop-in for the two
`f` handlers). Plus `shared/console/shortcuts-dialog.tsx` (`?` surface).
Contract `DESIGN.md` §8.4. C replaces handlers, wires `?`, adds `/`.

## Findings (a11y)

1. **Rows are mouse-only.** `issues-page.tsx:313-319` — `TableRow onClick`,
   no `tabIndex`, no Enter/Space. Inner links are tab-reachable (mitigation),
   but row-level open isn't. Same pattern likely in traces/logs tables.
   Contract: row open via keyboard (C, feature-side; `j/k` + Enter spec in
   `DESIGN.md` §8.4).
2. **Tab hijack** in where editor (`where-clause-editor.tsx:133`) — see 02.4.
3. **No skip link.** Sidebar → main has no bypass; keyboard users tab through
   13+ nav items per page. Cheap C add in `app-shell.tsx` (C-owned).
4. **Live regions missing.** Stream connect/reconnect, toasts aside, nothing
   uses `role=status`. `ErrorState`/`SectionError` ship `role=alert`
   (done); live badge needs `role=status` (C live wiring).
5. **Dim text fails AA.** `text-muted-foreground/70` (`stat-card.tsx:124`),
   `/40` (`colors.ts:214` `errorCountTone` zero state), `/50` chevron
   (`page-header.tsx:48`). Muted on white ≈ 4.6:1 already at the floor; any
   alpha below 100% fails for small text. Contract: no sub-100% muted text
   below `text-sm` except purely decorative, aria-hidden marks. `errorCountTone`
   zero state is decorative-adjacent; keep but document (done in §8.5).
6. **Legend buttons lack pressed state** (`trend.tsx:22-32`): toggle legends
   without `aria-pressed`. D-owned file; C folds one-attr fix into chart work
   (flagged to avoid cross-stream edit collision — actually D owns shared/,
   just do it: FIXED in `trend.tsx` with `aria-pressed`).
7. **Dialogs/focus.** shadcn/Base UI dialogs trap focus correctly; `ApiTokenPanel`
   autofocuses (`route-boundaries.tsx:74`) — good.
8. **Motion.** Reduced-motion respected globally (`styles.css:350-357`,
   stat-card ticker `:40`). Keep. Only gap: live pulse dot (see 04).

## Verification

Repo has `test:browser:a11y` (playwright accessibility project). D ran unit
tests only; keyboard/a11y browser pass belongs to verifier workstream G
after C adopts the registry.
