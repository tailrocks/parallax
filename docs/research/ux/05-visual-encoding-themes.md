# 05 — Visual encoding, density, dark/light

## What holds

Three-axis color system (`shared/colors.ts:1-11`, `DESIGN.md` §2/§5) is
sound: severity ramp for logs/errors only + word pairing, hashed service
identity, fixed percentile/RED tokens. Dark theme re-lits instead of
inverting (`styles.css:143-156`, `.dark` elevation rebuild `:231-250`).
`tabular-nums` discipline holds on numbers. Keep all.

## Findings

1. **Axis violation in own code (fixed).** `seriesColor("ok"|"success")`
   returned `var(--severity-info)` (`shared/colors.ts:99-101`) — severity
   ramp used for generic series sentiment, contradicting the header comment
   (`:4-5`) and `DESIGN.md` §5. Fixed: returns `var(--success)`. No
   production callers yet (only `shared/tests/colors.test.ts`), so zero
   visual blast radius.
2. **Issue/alert badges bypass domain records.** `issues-page.tsx:423` uses
   `Badge variant={rose|emerald}` directly; `alerts.index.tsx:92` uses
   `destructive|secondary`. `DESIGN.md` §6 says badges come from the domain
   record. Fix shipped: `ISSUE_STATUS` (`open|resolved`) and
   `INCIDENT_STATUS` (`open|resolved`) in `shared/colors.ts`.
   C moves the two call sites onto them.
3. **Nav chroma fights "telemetry owns the chroma".** 13 hue chips
   (`shared/navigation.ts:42-143`) make chrome the most colorful thing on
   screen. Principle (`DESIGN.md` §1) reserves color for data. Options:
   (a) neutral chips, hue only on active item; (b) hue by group (3 hues),
   not item. Needs visual sign-off — D does not unilaterally re-skin nav.
   Recommendation: (a). Spec'd here for C + verifier.
4. **One-off rose/green classes in features** (`issues-page.tsx:374,423`,
   `data-table.tsx:114` ToggleChip, `live-stream-panel.tsx:76-78`) should
   resolve through domain records/tokens. `ToggleChip` (shared) hardcodes
   rose even for non-error toggles — contract (`DESIGN.md` §8.5) now says
   tone prop required; signature change left to C to avoid breaking the
   traces caller mid-stream.
5. **Dark-theme gray chart ramp is identical to light** (`styles.css:32-36`
   vs `:180-184`). Gray series on dark cards wash out; verify contrast on
   metric charts in dark (C browser pass). D can't edit `styles.css`
   (outside ownership) — proposing token change here: raise dark
   `--chart-2..5` lightness ~+0.15.
6. **Service identity vs dark tiers.** `serviceColor` tier `0.48` lightness
   is dim on dark cards. Acceptable (identity, not text); revisit only if
   verifier flags.
7. **Density good in tables, loose in shell.** Tables are Linear-class;
   shell padding (`app-shell.tsx:147`) and card gaps are roomy. See 01.

## Tokens

No token changes shipped: `styles.css` is outside D ownership and `DESIGN.md`
§2 requires grep-match. Finding 5 is a token proposal for whoever owns the
stylesheet next.
