# Local-First V1 Concept

<!-- markdownlint-disable MD013 -->

Decision date: 2026-06-03

> **Decision — V1 starts as a local-first evidence server, not as a production observability
> cluster.** The first useful Parallax setup should run on a developer machine, manage a local
> GreptimeDB standalone process for observability evidence, use Turso/SQLite-like storage for local
> metadata and grouping state, expose CLI plus API access, and let a coding agent query a `run_id` for
> errors, logs, traces, spans, metrics, and grouped failures.

## Product Job

When a developer runs a local app, tests, or several local microservices, Parallax should capture
enough runtime state that the agent no longer needs a long human explanation.

Desired loop:

```text
developer runs app/test stack
  -> Parallax assigns run_id
  -> apps emit traces, spans, logs, metrics, errors
  -> Parallax groups errors and links signals
  -> developer says: "agent, inspect run_id X"
  -> agent queries Parallax
  -> agent gets bounded evidence, not scattered terminal text
```

This is the smallest product wedge: local debugging context for agent-assisted development.

## V1 Shape

V1 should feel like one self-contained command, even if it manages two local binaries:

```text
parallax serve
  -> managed local GreptimeDB standalone
  -> embedded Turso/SQLite-like metadata DB
  -> OTLP ingest
  -> grouping/correlation worker
  -> CLI commands
  -> local API server
```

Core commands:

```text
parallax run start
parallax run list
parallax run inspect <run_id>
parallax run bundle <run_id>
parallax issue list --run <run_id>
parallax issue context <issue_id>
```

V1 output should be useful to humans and agents:

- JSON bundle;
- Markdown bundle;
- compact terminal summary;
- raw refs for deeper local reads.

## API Surface

V1 should expose an API because agents and tools need stable access.

Preferred shape:

- **GraphQL first** for query/exploration over runs, issues, traces, logs, metric windows, and bundles.
- **OTLP endpoints** for telemetry ingest.
- **Minimal health/version endpoints** for ops.
- **Sentry-compatible ingest later**, after V1 proves the local OTLP loop.
- CLI calls the same local API rather than reimplementing query logic.

This keeps the surface small:

```text
CLI
  -> local API
     -> bundle service
     -> storage adapter
```

All clients must use this API boundary:

- CLI uses Parallax API;
- UI uses Parallax API;
- agents use Parallax API;
- future MCP adapter uses Parallax API;
- tests may use storage adapters directly only at adapter-test level.

No product client should query GreptimeDB, Turso, Postgres, ClickHouse, or any future backend directly.
This keeps redaction, grouping, auth, bundle projection, and backend portability in one place.

The API contract is specified in [Parallax API Concept](api-concept.md).

## Local Storage Default

V1 local default should be managed GreptimeDB plus embedded metadata storage:

| Need | Local V1 answer |
| --- | --- |
| install simplicity | one command starts Parallax plus managed GreptimeDB |
| observability evidence | local GreptimeDB standalone |
| grouping/state/config | local Turso/SQLite-like file |
| local run retention | short TTL / manual prune |
| query scope | one developer machine, one or few projects |
| data volume | enough for local tests and small microservice runs |
| durability | good enough for debugging, not production compliance |

GreptimeDB is suitable locally because it runs in standalone mode as a binary (`greptime standalone
start`) and can be installed through the Greptime Homebrew tap on macOS. Docker is optional, not
required. Parallax should support both:

```text
parallax serve --manage-greptime   # default local mode
parallax serve --greptime-url ...  # use existing GreptimeDB
parallax serve --no-greptime       # ultra-small fallback
```

Turso Database remains the local metadata candidate because current docs describe it as an in-process
SQL database written in Rust, compatible with SQLite, with local file and in-memory database examples.
It is still beta, so V1 must keep a fallback path if Turso behavior is not reliable enough. Plain
SQLite or another embedded store can substitute if needed.

The Turso-only fallback may store bounded telemetry for tiny demos/tests, but preferred V1 should not
force Parallax to rebuild observability storage in Turso. GreptimeDB stores the evidence; Turso stores
the local product state.

## Storage Growth Path

V1 local-first does not weaken the GreptimeDB decision. It clarifies tiers:

| Stage | Default storage | Why |
| --- | --- | --- |
| V1 local | managed GreptimeDB standalone + Turso/SQLite-like metadata | one-command local agent debugging with real observability storage. |
| V1 fallback | Turso/SQLite-like only | ultra-small demos/tests when no GreptimeDB sidecar is allowed. |
| V2 self-hosted server | GreptimeDB + metadata DB | higher telemetry volume, retained evidence, object-storage path. |
| Production durable | GreptimeDB + Postgres + workers | grouping/state durability, retained history, production workflows. |
| Scale-out | GreptimeDB distributed + Postgres + stream such as Apache Iggy | replay, backpressure, parallel processors. |

GreptimeDB becomes the evidence backend from local V1 onward, not only later production. Postgres
remains the safer production relational store for grouping, users, projects, policies, and audit state.
Apache Iggy remains optional until replay and parallel processing are real needs.

## What Makes This Different

This V1 is not another dashboard. UI is secondary.

Primary interface:

```text
run_id -> evidence bundle -> agent can reason
```

Existing tools usually start from dashboards, alerting, or production observability. Parallax starts
from local agent debugging context:

- capture one run;
- preserve runtime state;
- group failures;
- expose typed query surface;
- let agent inspect exact evidence.

That is why CLI/API matter before UI.

## Could This Be Wrong?

Yes. Current recheck shows the gap is narrower than the old story:

- **OpenObserve** now markets a Rust/open-source, single-binary or Helm observability platform for
  logs, metrics, traces, RUM, dashboards, alerts, AI SRE, and MCP. This is closest to "collapse the
  stack."
- **SigNoz** has open-source OpenTelemetry-native observability and now ships an MCP server for AI
  assistants to query logs, metrics, traces, alerts, and dashboards.
- **Rustrak** covers lightweight self-hosted Sentry-compatible error tracking and is moving toward AI
  assistant access.

These tools pressure Parallax. The remaining proposed gap is narrower:

> local-first run-id evidence for coding agents, with Sentry-style grouping, OpenTelemetry-native
> capture, managed local GreptimeDB, Turso/SQLite metadata, and a bundle contract that later scales to
> GreptimeDB/Postgres.

If OpenObserve, SigNoz, or another tool ships this exact local developer loop with a strong agent-ready
bundle, Parallax must narrow or pivot.

## V1 Non-Goals

- no full dashboard suite;
- no production HA;
- no full Sentry API parity;
- no full Grafana replacement;
- no long-retention telemetry lake in Turso-only fallback mode;
- no autonomous fixer inside Parallax core.

## Source Anchors

- [Turso Database repository](https://github.com/tursodatabase/turso) — in-process Rust SQL database,
  SQLite compatibility, local file and memory examples, beta caveat.
- [Greptime Homebrew tap](https://github.com/GreptimeTeam/homebrew-greptime) — macOS install path for
  `greptime` and standalone start command.
- [GreptimeDB standalone docs](https://docs.greptime.com/getting-started/installation/greptimedb-standalone)
  — binary standalone mode and local ports.
- [Tonic repository](https://github.com/hyperium/tonic) — Rust gRPC over HTTP/2 with generated
  client/server support.
- [OpenObserve homepage](https://openobserve.ai/) — current unified Rust observability / MCP / AI SRE
  competitor pressure.
- [SigNoz MCP changelog](https://signoz.io/changelog/2026-04-30-introducing-the-signoz-mcp-server-r5iwnkpxtsz88akwt6abqddn/)
  — AI assistants querying observability data.
- [Rustrak Docker image](https://hub.docker.com/r/abians7/rustrak-server) — lightweight self-hosted
  Sentry-compatible error tracking pressure.
