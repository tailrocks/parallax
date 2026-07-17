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

## Architecture (Plan 100 control plane)

25. **One package, one strict project.** Keep `ui/` as a single Bun package.
    Do not add internal npm packages or TypeScript project references without a
    separate measured plan.

26. **Canonical tree** (create a directory only with its first real file):

    ```text
    ui/src/
      app/                 router/provider composition (plan 143)
      layout/              shell/nav/theme/fallbacks (plan 143)
      routes/              thin TanStack adapters only → export Route
      features/<feature>/  api/ model/ components/ hooks/ tests/ index.ts
      domain/<concept>/    framework-neutral product concepts
      platform/<adapter>/  GraphQL, SSE, browser adapters
      shared/              product-neutral components/hooks/lib
      test/                Vitest builders only (no test bodies)
      components/ui/       shadcn CLI island
      lib/utils.ts         shadcn `cn` island
    ui/tests/harness/      plan 129 harness self-tests
    ui/tests/e2e/          Playwright (plans 132+)
    ```

27. **Closed dependency graph** (Oxc-enforced via
    `cargo xtask policy --only ui.architecture`):

    | From | Allowed | Forbidden |
    |------|---------|-----------|
    | app | all layers for composition | imported by lower layers |
    | routes | feature facades, domain, shared; root-only layout | feature internals, platform, other routes, app |
    | layout | feature facades, domain, shared | routes, feature internals, platform, app |
    | features | own internals, domain, platform, shared, approved feature facades | other feature internals, routes, layout, app |
    | platform | platform, domain, shared | features, routes, layout, app |
    | domain | domain, product-neutral shared | React/TanStack/browser/transport, features |
    | shared | shadcn + third-party only | Parallax feature/domain concepts, upper layers |

    Generated `routeTree.gen.ts` is the only reverse composition exception.
    Cross-feature imports need an exact `[[ui.feature_edges]]` row in
    `ratchet.toml`. Feature `index.ts` is the only public facade (named
    exports only; no handwritten `export *`).

28. **Feature catalog / migration owners**

    | Owner | Surfaces | Plan |
    |-------|----------|------|
    | investigations | `/investigations` | 134 |
    | sql | `/sql` | 135 |
    | ecosystem | `/ecosystem` | 136 |
    | dashboards | dashboards | 137 |
    | services | services | 138 |
    | issues | issues | 139 |
    | runs | invocations | 140 |
    | logs | logs | 141 |
    | traces | traces | 142 |
    | alerts | `/alerts` | dedicated alerts migration (ledger `alerts`) |
    | runtime-metrics, story, time-range, page-header | route-less | 149 |
    | overview | `/` | 150 |
    | app, layout, app-status, quick-navigation | shell/root | 143 |
    | platform/graphql | client + transport + codegen | 152 foundation |
    | platform/sse, platform/visibility, platform/storage, platform/url, platform/external-values | live/visibility/storage/search/decode | 153 foundation |

29. **Placement decision** for a new file:
    1. shadcn primitive? → `components/ui` via CLI only.
    2. Technical adapter (GraphQL/SSE/storage/clock/visibility)? → `platform/`.
    3. Framework-neutral Parallax concept used by ≥2 features? → `domain/`.
    4. Product-neutral UI with ≥2 independent consumers? → `shared/`.
    5. Product feature behavior? → `features/<feature>/…` behind `index.ts`.
    6. Route URL/search/loader only? → `routes/` exporting `Route` only.
    7. Shell/nav/theme? → `layout/` (plan 143).
    8. Always add/update `[[ui.ownership]]` in `ratchet.toml` same change.

30. **Module rules:** kebab-case files; named exports; `use*` hooks; `*Schema`
    runtime schemas; external data enters as `unknown` (decode once — plan 152/
    153); Result-shaped expected failures; discriminated async/UI state;
    no new catch-all `utils.ts`/`helpers.ts`/`types.ts`/`common.ts`.

31. **Tests:** bodies never share production files; final directory name is
    `tests/` under the owner; `src/test/` is builders only; every test has a
    `ui/test-matrix.json` owner. Compatibility reexports are
    `kind = "compatibility-reexport"` with a removal plan.

32. **Verify placement/architecture:**

    ```bash
    cargo xtask policy --only ui.architecture
    cargo xtask policy --only ui.ratchets
    cargo xtask policy --only ui.tests
    cd ui && bun run check && bun run lint && bun run typecheck && bun run --bun test:ci && bun run build
    ```

33. **Playwright product contracts (plan 144):** fixture-backed Chromium contracts
    are a required CI gate. Extend them as follows — never invent a second
    runner, Node process, happy-path `page.route()` stub, or product memory mode.

    | Kind | Path |
    |------|------|
    | Dataset IDs / manifests | `ui/tests/e2e/datasets/` (feature plans) + Rust `parallax_test_support::browser` |
    | Screens (shared locators only) | `ui/tests/e2e/screens/<surface>-screen.ts` |
    | Contract specs | `ui/tests/e2e/contracts/<surface>.spec.ts` |
    | Product fixture | `ui/tests/e2e/fixtures/product-fixture.ts` + `productTest` in `fixtures/test.ts` |
    | Matrix rows | `ui/test-matrix.json` with `lane_owner: playwright/contracts`, stable `id`, `scenario_owner`, temporary `delivery_plan`, `dataset_id`, `state_class` |
    | Template | `ui/tests/e2e/contracts/_template.spec.ts` (policy-checked, not counted) |

    Rules:
    - Seed/reset through the control plane (`resetDataset`) before navigation;
      assert mutations via UI **and** `snapshot()` postconditions.
    - Semantic locators only (role/name/label/placeholder/text). No CSS/XPath,
      `waitForTimeout`, `test.only`/`skip`/`fix`, or happy-path interception.
    - Diagnostics (console errors/warnings, page errors, external network,
      dialogs, downloads) fail the owning test automatically.
    - Commands every feature plan runs:

    ```bash
    cargo xtask policy --only ui.browser-contracts
    cd ui && bun ci
    cd ui && bunx --bun --no-install playwright install --with-deps chromium
    cd ui && bun run build
    cd ui && bun run test:browser:list
    cd ui && bun run test:browser
    ```

## Final placement (plan 151 live tree, 2026-07-17)

| Path | Owner |
|------|-------|
| `src/app/` | Router composition entry (`create-router`) |
| `src/layout/` | Product shell, palette, theme, route boundaries |
| `src/routes/` | Thin file-route adapters (`Route` only) |
| `src/features/<name>/` | Product features with explicit `index.ts` facades |
| `src/domain/` | Framework-neutral product concepts |
| `src/platform/` | GraphQL/SSE/browser technical adapters |
| `src/shared/` | Product-neutral UI kit (`console/`, `hooks/`, `navigation`, page-header) |
| `src/components/ui/` | shadcn generator island only |
| `src/lib/utils.ts` | shadcn `cn` island |
| `src/shared/{format,colors,color-by,where-clause}.ts` | Product-neutral helpers (plan 151) |
| 
| `src/routeTree.gen.ts` | TanStack generated tree |
| `src/styles.css` | Global tokens |

**Import directions:** app → layout/features/shared; layout → features/domain/shared; features → features(facade)/domain/platform/shared; platform → domain/shared; domain → shared; shared → shared only. Routes import feature facades + shared/domain only (root may import layout).

**Verify:** `cargo xtask policy --only ui.architecture`, `ui.tests`, `ui.ratchets`; `cd ui && bun run check && bun run lint && bun run typecheck && bun run --bun test:ci`.

