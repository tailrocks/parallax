# Parallax vs Better Stack — 2026-09-13

Roster gap filled this pass. Better Stack is a **closed SaaS** ClickHouse-backed
platform that unifies logs, traces, metrics, Sentry-compatible errors, RUM
(with replay), uptime checks, status pages, and incidents. Not self-hostable as
a product. Checked 2026-09-13 against first-party docs (no live tenant).

Primary sources:

- [Distributed tracing](https://betterstack.com/docs/logs/tracing/)
- [Log management](https://betterstack.com/log-management)
- [RUM](https://betterstack.com/real-user-monitoring)
- [Community: 10 best obs tools 2026](https://betterstack.com/community/comparisons/best-observability-tools/) (vendor-authored; treat as marketing, confirm docs)
- GitHub org `BetterStackHQ` last push 2026-09-12 (collector, helm, clients)

## Why it is in the roster

Uptime + incident + status-page loop is the best *adjacent* implementation of
“something broke → someone is paged → public comms.” GOAL §5 requires a
classification, not a clone.

## What is better (workflow, not checkbox)

| Workflow | Why Better Stack is better | Parallax response |
| --- | --- | --- |
| Uptime / synthetics | Multi-region checks + status pages; entry-level obs many teams start with | **Reject status pages.** Uptime checks = P1 (not freeze). Synthetics = Reject/P2 per freeze |
| Incident + on-call | Rotations, escalations, status comms in the same product as logs | Parallax incidents exist (bundle hash). On-call rotations = P2 |
| RUM + replay | Session replay tied to errors/traces; Core Web Vitals per URL | Parallax `/rum` is a projection over traces/metrics (HEAD). Replay = Reject unless freeze promotes |
| eBPF collector | Zero-code cluster ingest | **Reject building eBPF.** Integrate Odigos/Coroot/Better Stack collector later (P2/watch) |
| Sentry-SDK errors | Claims 100+ SDK compatibility at lower cost | Parallax envelope ingest shipped; multi-SDK public ledger still unproven (freeze item 10) |

## What Parallax should not copy

- Status pages (comms product).
- Hosted on-call as the identity of the product.
- Session replay store (heavy; freeze Reject).
- Their ClickHouse SaaS economics.

## Keep

Rust single-binary, OTLP+Sentry, CLI runs, evidence bundles, read-only MCP.

## Classification used in the gap matrix

Uptime P1 · synthetics Reject (freeze) · status pages Reject · incidents keep
Parallax case-file shape · RUM P0 as vitals+FE→BE (not replay).
