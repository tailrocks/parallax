# Plan 163 evidence — trace timeline interactions

Date: 2026-07-17. Live corpus traces against `parallax serve`.

- Drag-zoom (marquee): pointer drag across 20–40% of the t-wide gesture
  surface produced viewport 2.038–4.076 ms, the axis relabeled to the
  zoomed window, and the URL gained `?vs=2.038&ve=4.076`
  (`t-wide-zoomed.png`).
- URL round-trip: a hard reload of that URL restored the zoomed axis
  (+2.0ms first tick); "Reset zoom (0)" returned to fit and cleared the
  params.
- Minimap controller: viewport rectangle rendered with dimmed
  outside-viewport regions; interior drag pans, edge drag resizes, click
  recenters (`trace-minimap-viewport` testid).
- Clamp/skip: t-deep at `vs=5&ve=15` renders all 14 rows with bars clamped
  to the [-50%, 150%/200% width] envelope and no gigapixel elements
  (`t-deep-zoomed-clamped.png`); t-skew at `vs=0&ve=50` skips the
  out-of-viewport parent bar entirely while the backdated child renders at
  0% — no negative geometry.
- Color-by: picker (service default / span kind / status / span
  attributes present in the trace) is URL-encoded (`?color=kind` renders
  INTERNAL bars violet); attribute values hash through the same identity
  palette as services.
- Self-time: `computeSelfTimes` (children clipped to parent, overlaps
  merged) surfaces as the duration-cell tooltip ("self 2.0ms" on t-deep)
  and as an inspector row.
- Flamegraph: Flame tab (`?view=flame`) icicle-packs 521 t-wide spans into
  greedy per-depth lanes (`t-wide-flame.png`); Shift+click focuses a
  subtree (521 → subtree), "Show whole trace" restores.
- In-bar labels gate on rendered width (56px name / 140px +duration).
- Plan-160 waterfall regression suite passes unchanged on the new engine;
  full UI gates green; browser console clean after adding the picker's
  form id (the only message seen during the walk).

Foundation pure layer (reducer, gestures, lane packing, color-by lib) was
landed preliminarily by the parallel agent and verified here; the waterfall
rework, minimap controller, URL schema, picker, self-time display,
click-through gesture surface, and evidence are this closing slice.
