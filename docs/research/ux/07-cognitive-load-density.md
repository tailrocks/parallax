# 07 — Cognitive load & density

## Load audit per surface (GOAL §7 lens: irrelevant info, dead-ends, reconstruction cost)

- **Overview** (`features/overview/.../overview-page.tsx`, 120+ lines of chart
  plumbing): stat cards (Volume→Health→Performance→Cost, correct order) +
  movers + issues + traces. Load OK. Gap: cards don't link onward uniformly
  (contract says every chart/list links onward) — C check.
- **Issues list**: 8 columns incl. Tags (`issues-page.tsx:292`) — tags are
  low-signal at list density and each row carries 4+ links. Recommend: drop
  Tags column to detail-only, keep service/trend/events/seen. C call.
- **Issue detail**: worst load on the site (see 03.1). Fix path shipped via
  `DetailSummary` + tabs contract.
- **Logs**: 7-control toolbar wraps to 2–3 rows (`logs-page.tsx:425-500`).
  Highest chrome-to-data ratio. `QueryBar` contract collapses to 2 ordered
  rows max; C adopts.
- **Traces list**: toolbar + attribute-compare + facets + field explorer +
  table compete for attention. Compare/explorer are power tools — default
  collapsed or tabbed (C). Facets: good.
- **Trace detail**: earns its complexity; modes/tabs chunk well. Keep.
- **Metrics index**: under-loaded — bare names, no sparkline/unit/signal.
  Discovery cost high ("which of 500 metrics matters?"). C: add sparkline +
  last-value columns (data: catalog has `pointCount`/`lastDatapointNanos`,
  `routes/metrics.index.tsx:15-22`).
- **Services/Ecosystem overlap**: two surfaces for service topology; unclear
  when to use which. Product question, flagged.

## Density bar

Tables meet it (`text-sm`, compact rows, `table-fixed`). Prose/chrome don't
always: `PageHeader` descriptions repeat nav labels ("Grouped errors by
service…" under "Issues") — low value; contract now says descriptions must
carry non-obvious hint or be omitted (C trims).

## Progressive disclosure rule (contract `DESIGN.md` §8.2)

Detail pages: summary strip → tabs → sections. Power panels (attribute
compare, field explorer, metric strips) default to their tab, never crowd
the primary answer. List pages: max 2 toolbar rows at 1280px.
