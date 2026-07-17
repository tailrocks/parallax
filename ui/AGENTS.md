# AGENTS.md — `ui/`

Rules for working on the Parallax UI (TanStack Start + shadcn/ui on **Base UI**, Vega preset,
default theme). Researched against tanstack.com and ui.shadcn.com docs, 2026-06-12; re-verify on
major upgrades. The root [`AGENTS.md`](../AGENTS.md) and the
[UI spec](../docs/research/architecture/simple-ui-v2.md) govern scope; this file governs *how*.

## TypeScript

1. **Strictest mode always** (operator rule): `strict` plus `noUncheckedIndexedAccess`,
   `exactOptionalPropertyTypes`, `noImplicitOverride`, `noImplicitReturns`,
   `forceConsistentCasingInFileNames`, no unused labels/unreachable code. `bun run typecheck`
   must pass before every commit — vite build does NOT type-check.
2. Rely on router inference: derive data with `Route.useLoaderData()`/`useParams()`; never
   hand-annotate loader return types (they go stale and fight inference).
3. Keep the template's `verbatimModuleSyntax` as shipped; there are upstream reports of it
   leaking server code into clients in Start apps — if a server/client boundary bug appears,
   check this first (TanStack/router#5659).

## TanStack Start / Router

4. **Loaders are isomorphic** — they run on server AND client. No secrets in loaders; absolute
   API base on the server side (`src/lib/api.ts` handles this). Server-only work goes in
   `createServerFn`.
5. File conventions: `index.tsx` exact match, `$param` dynamic, `dot.notation` nesting,
   `_prefix` pathless layouts, `__root.tsx` shell. The route tree is generated —
   **`routeTree.gen.ts` stays committed** (CI typecheck needs it), excluded from lint/format,
   and is drift-checked after the build regenerates it. Generated shadcn components are formatted
   after generation and remain inside the normal formatter selection.
6. Navigation only via typed APIs: `<Link to/params>`, `Route.useSearch()`; validate search
   params with zod `validateSearch` when a page grows URL state.
7. When TanStack Query lands here: `queryClient.ensureQueryData` in loaders +
   `useSuspenseQuery` in components; router `defaultPreload: 'intent'` +
   `defaultPreloadStaleTime: 0`. Until then, plain loader fetches are the pattern.
8. Throw `notFound()` from loaders, not components; root provides the 404.

## shadcn/ui (Base UI variant)

9. **Add components only via the lock-local CLI**
   (`bunx --bun --no-install shadcn add <name|block>`), never
   hand-copy — the CLI resolves deps and applies the Base UI transforms. Compose existing
   blocks before writing custom UI.
10. **Base UI composition uses `render={<El …/>}`, not `asChild`** (and `nativeButton={false}`
    when a Button renders a non-button). This is the #1 migration trap.
11. Theme only through the CSS variables in `src/styles.css` (`--background/--foreground`,
    `--chart-1..5`, `--sidebar-*`); never inline colors. The variable set is the custom
    token system adopted 2026-07-03, and visual separation uses `--custom-shadow*` tokens,
    not borders. `components.json` is locked config.
12. Charts: native Recharts composed inside `ChartContainer` with a `ChartConfig`; series
    colors via `var(--color-<key>)`; the container needs a height.
13. Tables that grow features use TanStack Table + shadcn `<Table>` split as
    `columns.tsx`/`data-table.tsx`/route — there is deliberately no monolithic DataTable.
14. `cn()` from `@/lib/utils` for all conditional classes.

## Parallax-specific

15. **One data path:** everything through `src/lib/api.ts` → `/graphql` (same-origin; dev
    proxies to :4000, the embedded prod build is same-origin by construction). No direct
    storage access, no other endpoints, no auth headers in V1.
16. Nanosecond timestamps are strings end-to-end (JSON precision); format with `relativeTime`.
17. Every chart/list links onward (the interactivity rule from the UI spec): issue → trace →
    logs; never a dead end.
18. Unused demo code from blocks gets deleted, not kept: strict mode + dead demo files fail
    the bar. Re-add blocks via CLI when their page is actually built.

## Observability design language (plan 162)

19. **Three semantic color axes — never repurpose:**
    - **Severity ramp** (`--severity-trace|debug|info|warn|error|fatal` via
      `@/lib/colors` `severityToken`/`severityColor`): log severity and error/
      incident state ONLY. Always pair color with the **literal severity word**
      (dot + WORD); color is never the sole signal.
    - **Service identity** (`serviceColor(name)` + `ServiceDot`): service
      **identity** ONLY — never sentiment, health, or a metric. Same service
      name → same color on every page. Always render the service name text
      next to the decorative (`aria-hidden`) squircle.
    - **Percentile / RED charts** (`--chart-p50|p95|p99|error|throughput`):
      latency and RED series ONLY. Generic series use `--chart-1..5` or
      `seriesColor(name, index)` (semantic names map; unknown → golden-angle).
20. **Tabular numerals** on every stacked duration/count cell
    (`tabular-nums` / `.tabular-stack`) so columns align vertically.
21. **Motion doctrine:** content replacing a skeleton uses opacity-only
    `.content-enter` (~150ms); never move data the user is about to read
    (no height animations on rows; no spinners for sub-second loads). All
    motion respects `prefers-reduced-motion`.
22. **Empty-state voice:** say what is missing and what would produce it
    (e.g. "No sessions yet — this invocation emitted no session.start event"),
    never marketing copy.
23. **Six-item browser verification checklist** (every UI change against
    playground data before the next step):
    1. **Data correctness** — values match GraphQL/source for the visible
       window.
    2. **Links** — every row/chip navigates to a real detail surface.
    3. **States** — empty, loading (skeleton), error, and live/polling each
       render sanely.
    4. **Layout** — no overflow/clip at default density; table numerals
       align.
    5. **Theme** — light and dark both readable (severity words + contrast).
    6. **Motion** — no layout shift on refresh; reduced-motion does not
       break usability.

24. **Every graph visualization renders with React Flow** (operator,
    2026-07-17): wherever the UI draws a graph schema or displays anything
    as a node/edge graph — the ecosystem service map, dependency diagrams,
    topology views, any future graph surface — use `@xyflow/react`
    (React Flow). Never hand-roll an SVG/canvas graph renderer. Layout
    stays ours (ELK worker + deterministic fallback); React Flow consumes
    the computed coordinates. Node/edge visuals keep the plan-162 language
    (ServiceDot identity, kind glyphs, severity-worded error chips).
    Timeline/chart surfaces (waterfall, flamegraph, Recharts) are not
    graphs in this sense and keep their dedicated renderers.
