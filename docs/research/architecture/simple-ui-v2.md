# Simple UI V2 Concept

<!-- markdownlint-disable MD013 -->

Decision date: 2026-06-03

> **Decision — V2 adds a simple local UI after CLI/API V1.** V1 stays CLI/API-first for agent
> evidence. V2 adds a lightweight web UI so a developer can see the same grouped errors, stack traces,
> logs, traces, spans, and metrics locally without installing Sentry, Grafana, Kibana, Tempo, Loki, or
> Prometheus. The required frontend stack is **TanStack Start + shadcn/ui**.

## Why UI Exists

CLI/API is best for agents. Humans still need visual inspection:

- what errors grouped together;
- full stack trace and frames;
- error frequency over time;
- affected runs/releases/services;
- trace waterfall;
- logs around a trace/span/error;
- metric windows around a failure;
- raw evidence refs and bundle preview.

So V2 UI is not a dashboard suite. It is a local investigation console over the same evidence bundle
API.

## Required Tech Stack

Use:

- **TanStack Start** for the web app framework, routing, server functions, and type-safe data loading.
- **shadcn/ui** for accessible, owned source components built with Tailwind/Radix patterns.

Reasons:

- good AI/developer ergonomics;
- modern React stack;
- source-owned UI components, not opaque vendor widgets;
- easy to keep UI compact and operational, not marketing-heavy;
- works with local Parallax API and future server deployments.

## V2 Screens

Minimum UI:

| Screen | Purpose |
| --- | --- |
| Runs | list local `run_id`s, service count, error count, log/span/metric counts, status, duration. |
| Run detail | timeline of errors, logs, spans, metrics; bundle/export buttons. |
| Issues | grouped errors like Sentry: title, fingerprint, count, first/last seen, affected runs/services. |
| Issue detail | full stack trace, occurrences, breadcrumbs/log windows, linked spans, metric windows. |
| Trace detail | waterfall view, span list, errors/logs attached to spans. |
| Logs | Kibana-inspired object inspection: fields, filters, search, selected columns. |
| Metrics | small Prometheus-like metric windows around selected run/issue/trace. |
| Bundle preview | exact JSON/Markdown evidence sent to agent. |

## UX References

Borrow what works:

- **Sentry:** grouped issue workflow, frequency, stack trace, release/run context.
- **Kibana:** log object inspection, field filtering, selected-column view.
- **Grafana:** cross-signal linking and compact charts.
- **Tempo:** trace waterfall.
- **Prometheus:** metric windows and PromQL-shaped thinking.

Do not copy full scope:

- no dashboard-builder;
- no alert-rule UI in V2;
- no user/team admin unless needed for local auth;
- no production incident suite;
- no autonomous fixer UI.

## Architecture

```text
TanStack Start UI
  -> Parallax local API
     -> bundle/query services
        -> Turso metadata
        -> GreptimeDB evidence
```

The UI must not query GreptimeDB directly. It talks to Parallax API only, so local, server, and future
backend profiles stay compatible.

## Source Anchors

- [TanStack Start docs](https://tanstack.com/start/latest/docs/framework/react/overview) — full-stack
  React framework powered by TanStack Router, server functions, routing/data loading.
- [shadcn/ui docs](https://ui.shadcn.com/) — accessible source-owned components, Tailwind/Radix,
  installable through CLI into the app.
