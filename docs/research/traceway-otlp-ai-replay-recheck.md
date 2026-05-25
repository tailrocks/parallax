# Traceway OTLP AI Replay Recheck

<!-- markdownlint-disable MD013 -->

Research date: 2026-05-25

## Pass Target

Re-check Traceway because it pressures the side of Parallax that is easy to
understate when the research focuses on Sentry-compatible error trackers:

```text
direct OTLP ingest
+ logs/traces/metrics correlation
+ exceptions/issues
+ session replay/RUM
+ AI trace capture
+ low-friction self-hosted and embedded modes
```

This pass tests whether Traceway has closed the Parallax wedge or whether it
raises the minimum bar for OTLP-native context while leaving Sentry migration,
evidence bundles, and agent action audit open.

## Short Verdict

Traceway is a stronger OTLP/context competitor than the previous watch row
proved. The relevant evidence is not only marketing copy: current source shows
OTLP/HTTP routes for traces, metrics, and logs; code converts spans into
endpoints, tasks, exceptions, generic spans, and AI traces; docs describe direct
SDK export without a required Collector; and the SQLite deployment path is a
single container with local blob storage or optional S3.

Traceway does **not** close the Parallax wedge in the checked sources:

- no Sentry-compatible envelope ingest or DSN migration path was found in the
  checked tree/docs;
- no MCP server or read-only evidence-bundle agent surface was found;
- no canonical, portable, redacted evidence-bundle schema was found;
- no coding-agent command/file/patch/test/action audit or fix-outcome loop was
  found;
- it is Go/Svelte, not Rust-first.

The product implication is narrower and sharper:

```text
Parallax cannot sell "OTLP-native, self-hosted, no Collector required" as a
complete differentiator. Traceway already pressures that shape. Parallax's
defensible gap is Sentry-compatible migration plus OTLP context plus
agent-safe, citable evidence bundles and action/outcome audit.
```

## Current Source Snapshot

| Source | Checked signal | Parallax implication |
| --- | --- | --- |
| [Traceway repository](https://github.com/tracewayapp/traceway) and [latest backend release](https://github.com/tracewayapp/traceway/releases/tag/backend/v1.7.27) | GitHub metadata checked 2026-05-25 shows MIT license, 817 stars, 23 forks, 20 open issues/PRs combined by the repo API, latest push on 2026-05-25, latest `main` commit `38b8d385`, and latest release `backend/v1.7.27` published 2026-05-22. The release notes include distributed-trace handling from OTel, a Helm chart, and pre-built Docker image docs. | Treat Traceway as active and strategically relevant, not as a stale README-only signal. |
| [Route source](https://github.com/tracewayapp/traceway/blob/main/backend/app/controllers/routes.go) | `POST /api/report` is the native Traceway client protocol. `POST /api/otel/v1/traces`, `/v1/metrics`, and `/v1/logs` are registered with bearer-token client auth. Dashboard/admin endpoints also expose projects, widgets, metrics query/discovery, endpoint/task/session detail, AI traces, distributed traces, logs, exceptions, auth/OAuth, org/member/invite management, source-map upload, notification channels/rules/history, and archive/unarchive operations. | Traceway has a broad dashboard API and real OTLP ingress. Its HTTP API is not a Parallax-style least-privilege evidence-bundle surface. |
| [OTLP codec source](https://github.com/tracewayapp/traceway/blob/main/backend/app/controllers/otelcontrollers/codec.go) and [OTel docs](https://docs.tracewayapp.com/client/otel) | OTLP/HTTP accepts protobuf or JSON, supports gzip, has a 10 MB body limit, uses `/api/otel` as the base endpoint, and documents direct SDK export without a required Collector. | Parallax's OTLP receiver must meet this low-friction direct-export bar while adding stronger failure semantics, redaction, and bundle projection. |
| [OTLP controller source](https://github.com/tracewayapp/traceway/blob/main/backend/app/controllers/otelcontrollers/otel.controller.go) | Trace ingest converts batches and inserts endpoints, tasks, spans, exceptions, and AI traces; metrics ingest inserts metric points and auto-registers metric metadata; log ingest inserts log records; all three record ingest monitoring and enforce report rate limits. | This is real ingest logic, not only a docs promise. Parallax should compete above the conversion layer with evidence semantics. |
| [Trace converter source](https://github.com/tracewayapp/traceway/blob/main/backend/app/controllers/otelcontrollers/trace_converter.go) | SERVER/INTERNAL HTTP spans become endpoints, CONSUMER spans become tasks, `exception` span events become issue stack traces, child spans are linked to owning entities, and any span with `gen_ai.*` attributes becomes an AI trace. AI trace conversation content is written to object storage under `ai-traces/<project>/<trace>.json`. | Traceway already turns OTel into product concepts and AI traces. Parallax must preserve broader causality, source policy, redaction, and action/outcome rows rather than only "store spans." |
| [Logs converter source](https://github.com/tracewayapp/traceway/blob/main/backend/app/controllers/otelcontrollers/logs_converter.go) and [logs docs](https://docs.tracewayapp.com/client/otel/logs) | OTLP logs preserve trace/span IDs, severity, service/resource/scope/log attributes, and docs say logs emitted inside active spans link to the originating trace/span and appear in trace detail views. | Trace-linked logs are a baseline feature for the Parallax bundle, not a differentiator. |
| [Metrics converter source](https://github.com/tracewayapp/traceway/blob/main/backend/app/controllers/otelcontrollers/metric_converter.go) | Gauge, sum, and histogram data points are converted; histogram support stores average/count points; selected process resource attributes are allowlisted into metric tags. | Metrics ingest exists, but this pass did not benchmark storage or query performance. Keep metric-cost and scale claims unmeasured. |
| [AI tracing docs](https://docs.tracewayapp.com/learn/ai-tracing), [OpenRouter guide](https://docs.tracewayapp.com/client/openrouter), and [OpenRouter golden fixture](https://github.com/tracewayapp/traceway/blob/main/backend/app/controllers/otelcontrollers/testdata/openrouter_ai_trace.json.golden.json) | Docs state spans with `gen_ai.*` semantic attributes are promoted to AI traces whether root or child spans; conversation content is stored in S3 or local filesystem; OpenRouter can export OTLP traces directly; the checked golden fixture records model, provider, operation, token counts, costs, finish reason, and conversation count. | AI-call observability is not enough for Parallax's agent-session wedge. Parallax needs command/tool/file/approval/test/patch/outcome context around agents, not only LLM call cost and prompt/completion traces. |
| [Protocol spec](https://docs.tracewayapp.com/protocol) | The native `/api/report` protocol accepts traces, exceptions, metrics, sessions, and session recordings. Frontend/mobile SDKs use this protocol where OTel does not yet cover session replay. | Traceway's session replay/RUM path pressures Parallax's frontend roadmap, but it is not Sentry-envelope compatibility. |
| [Embedded mode docs](https://docs.tracewayapp.com/learn/embedded-mode) and [embedded Go example](https://github.com/tracewayapp/traceway/blob/main/examples/embedded-backend-otel/main.go) | A Go app can call `tracewaybackend.Run`, seed a default user/project, and export OTel traces to the embedded server at `/api/otel/v1/traces`. Docs call embedded mode development-only and use SQLite by default, in memory unless `WithSQLitePath` is set. | Parallax local/dev UX must be honest against this bar. Embedded dev mode is strong, but it does not prove production retention, backup, or agent-bundle parity. |
| [Self-host docs](https://docs.tracewayapp.com/server), [SQLite docs](https://docs.tracewayapp.com/server/sqlite), [Docker Compose](https://github.com/tracewayapp/traceway/blob/main/docker-compose.yml), [SQLite Compose](https://github.com/tracewayapp/traceway/blob/main/docker-compose.sqlite.yml), and [Docker signatures](https://github.com/tracewayapp/traceway/blob/main/DOCKER_SIGNATURES.md) | Deployment options include all-in-one, Docker Compose, minimal external-db, SQLite, local setup, and Haloy. Root Compose declares `traceway`, `clickhouse`, and `postgres`. SQLite mode is a single Alpine container with two SQLite files plus local blobs under `/data`, optional S3 for blobs, and retention knobs. Image-size and signed-image claims are documented, not independently measured here. | The simplicity baseline must count both visible services and hidden bundled subsystems. SQLite mode is the relevant low-friction comparison; image size and performance claims remain unmeasured unless benchmark artifacts exist. |
| [Integration skills tree](https://github.com/tracewayapp/traceway/tree/main/skills) and [add Traceway skill](https://github.com/tracewayapp/traceway/blob/main/skills/add-traceway.md) | The tree has integration-instruction files for adding Traceway to apps. The generic skill tells coding assistants how to ensure `http.route`, status codes, exception events, and CONSUMER task spans. It also says Go/frontend SDKs use `/api/report` while generic OTel uses `/api/otel`. Tree-path checks found no `mcp`, `sentry`, `claude`, `cursor`, or `codex` product/tool folders besides `skills/`; content hits for `sentry`, `dsn`, and `envelope` were comparison/design/test/framework references, not a Sentry-compatible ingest surface. | Traceway uses agent-readable integration guidance, but that is not an MCP/data-access/evidence-bundle surface. |
| [Creator note](https://github.com/tracewayapp/traceway/blob/main/HN.md) | Maintainer states goals around simple/cheap hosting, sub-15-developer teams, no paid add-ons, ClickHouse base with SQLite for self-hosting, sessions in S3, no AI SRE upsell, and frontend/mobile custom protocol because current frontend/mobile OTel does not cover session replay. | Useful product-intent evidence, but lower weight than shipped source. It reinforces the small-team simplicity threat. |
| [Traceway OTel Agent docs](https://docs.tracewayapp.com/learn/otel-agent) and [agent repository](https://github.com/tracewayapp/traceway-otel-agent) | Docs describe a small preconfigured OTel Collector service for host metrics every 60 seconds, optional logs and process metrics, checksum-verified installers, and self-hosted endpoint override. GitHub metadata checked 2026-05-25 shows 3 stars, no license in the repo API, and latest push on 2026-04-28. | Host telemetry agent is relevant but much less mature than the core Traceway repo; do not treat it as a mature agent-observability or action-audit surface. |

## What Changed Or Was Narrowed

1. **Traceway is no longer only a README-level watch item.** Source and docs
   show direct OTLP ingest, conversion logic, trace-linked logs, metrics, AI
   traces, native `/api/report`, session replay, and multiple self-hosted modes.
2. **Traceway's strongest pressure is not Sentry migration.** It pressures the
   OTLP-native unified-context, AI-trace, frontend/session, and embedded/local
   developer experience parts of Parallax.
3. **The "no Collector required" bar is higher.** Traceway documents direct SDK
   export and optional Collector use. Parallax's tiny tier should do the same,
   but with explicit redaction, missing-evidence, and projection-equivalence
   guarantees.
4. **The agent gap remains open.** Integration skills are instructions for
   coding assistants to add instrumentation. They are not an MCP server, CLI
   context surface, or read-only evidence-bundle schema.
5. **Deployment claims need mode separation.** Compose, all-in-one, minimal,
   SQLite, and embedded mode have different hidden dependencies and persistence
   semantics. Benchmark comparisons must not collapse them into "one container."

## Parallax Impact

Traceway raises the Phase 1 bar in four places:

- OTLP/HTTP traces, metrics, and logs should be accepted directly from SDKs
  without requiring a Collector.
- OTel exception events and trace-linked logs should appear in the first useful
  issue context if available.
- SQLite or similarly low-friction local/dev modes need clear persistence,
  backup, and retention semantics.
- AI traces should be treated as ordinary runtime evidence, but not as a
  substitute for agent-session audit.

The wedge is still not closed because Traceway does not prove:

- Sentry SDK/envelope/DSN migration for existing Sentry users;
- canonical evidence bundles with hashes, raw refs, source policy, redaction
  reports, and missing-evidence rows;
- projection-equivalent CLI/HTTP/MCP/Markdown/JSON output;
- read-only agent context tools;
- coding-agent side-effect audit across commands, files, tests, patches,
  approvals, PRs, deploys, and reversions;
- accepted/rejected/reverted fixer outcome rows.

## Falsification Triggers

Reopen the Parallax verdict if Traceway:

- adds Sentry envelope or DSN-compatible ingest for current Sentry SDKs;
- publishes a portable evidence-bundle schema with source/raw-ref/redaction
  policy and projection hashes;
- adds a read-only CLI/MCP/HTTP evidence surface that coding agents can consume
  safely;
- adds coding-agent command/file/patch/test/PR/deploy audit and fix-outcome
  writeback;
- makes session replay plus backend traces plus AI traces export as a citable
  incident dossier rather than only dashboard views;
- publishes independently reproducible setup/resource/throughput benchmarks
  that cover the same first-use evidence surface Parallax targets.

## Bottom Line

Traceway is now the live OTLP-native/self-hosted/context-product comparison row.
It does not make Parallax unnecessary, but it makes weak wording fail:

```text
Not: "self-hosted OTLP observability with AI traces"
Yes: "Sentry-compatible runtime evidence engine that accepts OTLP, correlates
errors/logs/traces/metrics/frontend/agent execution, and returns redacted,
citable bundles with action/outcome history for coding agents"
```
