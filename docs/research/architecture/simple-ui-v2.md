# Parallax UI Concept (V1)

<!-- markdownlint-disable MD013 -->

Decision date: 2026-06-03; **revised 2026-06-12 (operator statement #7): the UI is V1 scope.**
The earlier staging ("V2 adds a simple local UI after CLI/API V1") is superseded — the operator
ruled the UI very important and part of V1, alongside CLI and API, with **no authentication in
V1** (local, single-user; auth arrives with the server profiles). The filename keeps its
historical name; this is the V1 UI specification.

> **Decision.** V1 ships a local web UI over the same Parallax API the CLI and agents use. It is
> an investigation console, not a dashboard suite: Sentry-grade grouped issues, standard service
> dashboards plus **user-defined dashboards built from whatever metrics the apps send**, trace
> lookup and waterfalls, and fully interactive cross-navigation (chart → time window → errors →
> event → trace → logs). Stack: **TanStack Start + shadcn/ui on Base UI, default theme as-is,
> shadcn charts and blocks reused wholesale.**

## Required Tech Stack (verified 2026-06-12)

| Choice | Rule | Verified state |
| --- | --- | --- |
| Framework | **TanStack Start** | API-stable RC (officially "feature-complete, preparing 1.0"; `@tanstack/react-start` 1.168.x, lockstep with Router 1.x). Server functions/route loaders fetch from the local GraphQL API server-side; TanStack Query hydration built in. |
| Components | **shadcn/ui on Base UI** (operator instruction) | Official since Dec 2025 (`npx shadcn create`, Base UI variant; full per-component Base UI docs since Jan 2026; Base UI 1.0 shipped 2025-12-11, `@base-ui/react`, by MUI's team incl. ex-Radix founder). Same import surface as Radix variant. |
| Theme | **Default theme as-is** — no customization | Coherent instruction: Vega style + Neutral base, OKLCH CSS variables, built-in light/dark, dedicated `--sidebar-*` and `--chart-1..5` token sets — the default already themes exactly the sidebar+charts surfaces a dashboard needs. |
| Charts | **shadcn chart components only** | Built on **Recharts v3**; Area/Bar/Line/Pie/Radar/Radial + tooltip variants (~10 copy-paste variants each at ui.shadcn.com/charts). Sparklines = axis-stripped line/area. Colors via `--chart-N` tokens. |
| Blocks/pages | **Reuse shadcn blocks** | `dashboard-01` (sidebar + header + SectionCards + interactive area chart + DataTable) is the app shell starting point; data tables on **TanStack Table** per the official data-table guide (sorting, filtering, pagination, row actions — the issue list). |
| Install | `shadcn init` against the TanStack Start template | First-class: `pnpm dlx shadcn@latest init -t start` (Tailwind + `@/*` alias preconfigured). |

The industry-standard look is the point: reuse, don't redesign.

## V1 Screens

### Issues (the Sentry-page requirement, statement #7)

List view — one row per grouped error, columns matching what the operator uses daily in Sentry:

| Column | Content |
| --- | --- |
| Issue | error type + normalized message (culprit frame beneath) |
| Trend | sparkline of recent occurrence counts (rollups) — clicking opens the issue with the trend expanded |
| Events | total count, clickable |
| Age / First seen | relative + absolute |
| Last seen | relative, sorted-by default |
| Tags | top tags chips (service, environment, release) |

Filters: project, service, environment, time range, status (open/resolved), tag values, free-text
over message. Sort: last seen, first seen, events, trend.

Detail view: full **stack trace** (frames, file:line), the **message**, the **occurrence trend
chart** (clickable time brush → the events in that window), first/last seen, counts, **tags
table** (value distribution per tag), **context sections** — runtime/OS/process from resource
attributes, **SDK and dependency info** where the SDK reports it (`telemetry.sdk.*`, build
attrs), recent breadcrumb-style log window — and the jump links: → trace waterfall of any
occurrence, → logs around it, → `parallax issue context` CLI snippet to hand the agent.

### Dashboards

1. **Service overview (predefined):** per service — historical **CPU and memory** charts
   (process metrics feed), **HTTP/gRPC request rate and duration** (p50/p95/p99 from the
   middleware histograms), error rate from rollups. The "what Grafana would show" page, zero
   setup.
2. **Custom dashboards (operator requirement, statement #7b):** the user composes their own
   pages from **any metric their apps send** — pick a metric by name (autocomplete from stored
   metric names), choose chart type (shadcn chart variants), label it, place it on a grid; saved
   to the metadata store, listed in the sidebar. V1 keeps it deliberately simple: metric +
   aggregation + group-by-attribute + chart type. No alerting, no sharing, no templating —
   that is the entire builder.

### Traces

- **Lookup** by pasted `trace_id` (the lifecycle-4 entry) and **by `run_id`** — both first-class
  (statement #7b), mirrored by `parallax trace inspect <trace_id>` and
  `parallax run inspect <run_id>` in the CLI.
- Waterfall: span tree across services (cross-service via shared trace), durations, status;
  span detail pane shows attributes — **including `db.query.text`/`db.operation.name` spans
  from the Postgres/ClickHouse wrappers and Juniper resolver/DataLoader spans**
  ([rust-stack-instrumentation.md](../capture/rust-stack-instrumentation.md)) — plus the span's
  logs and errors.

### Logs

Kibana-style object inspection: fields, filters, search, selected columns; scoped by trace, run,
service, or time window.

### Runs

The local `run_id` list (status, error count, duration) → run detail (timeline of errors, logs,
spans, metric windows; bundle preview/export).

## Interactivity rule (the correlation requirement)

Everything is a link; every chart is a filter. The flows that must work in V1:

```text
error-rate chart → brush a time window → events in that window → one event
  → its trace waterfall → a span → that span's logs/attributes
issue trend → click a spike → occurrences in the spike → trace
dashboard chart (any metric) → brush window → "errors in this window" cross-link
trace span (db.query.text) → copy query; trace → "open in CLI" snippet for the agent
```

The UI's job in the agent era (statement #7): the human *sees* the anomaly visually, then hands
the agent the reference — trace ID, run ID, issue ID — and the agent pulls the same data through
the CLI/API. The UI and the agent read one source of truth.

## Architecture

TanStack Start app in `ui/` (per the [build plan](v1-build-plan.md)); route loaders/server
functions call the local GraphQL API on `:4000`; no direct storage access (the API boundary is
absolute); no auth in V1 (loopback assumption); served by `parallax serve` (embedded static
build) so V1 stays one binary + one URL.

## Non-goals (V1)

Alerting UI; multi-user/orgs/auth; dashboard sharing/templating; full Grafana parity; session
replay; the fix-review screen (waits for outcome data to exist — deferred with the fixer rails);
uptime/crons. The **2026-06-11 trust-surface addendum** (trace lookup, fix review) is folded in
above: trace lookup ships in V1; fix review stays deferred.

## UX References

Sentry (issue grouping/detail — the explicit model), Kibana (log object view), Grafana
(cross-signal dashboards), Tempo/Jaeger (waterfall), Prometheus (metric windows).
