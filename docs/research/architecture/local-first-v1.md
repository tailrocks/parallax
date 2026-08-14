# Historical Local-First V1 Concept

<!-- markdownlint-disable MD013 -->

Decision date: 2026-06-03

> **Status (2026-07-17): implemented V1 design record, not an active plan or
> supported-profile contract.** The local profile and its expanded product
> surfaces shipped. Closed plans cited in historical sections no longer own
> work; only an operator-opened numbered plan does.
> GreptimeDB plus Turso is mandatory. The older `--no-greptime`, Turso-only,
> Postgres, and engine-substitution projections below are superseded.

> **Decision record — V1 started as a local-first evidence server, not as a production observability
> cluster.** The first useful Parallax setup ran on a developer machine, managed a local
> GreptimeDB standalone process for observability evidence, use Turso for local
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

## Shipped V1 Shape

V1 was designed to feel like one self-contained command while managing the
required local engines:

```text
parallax serve
  -> managed local GreptimeDB standalone
  -> embedded Turso metadata DB
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

V1 output is useful to humans and agents:

- JSON bundle;
- Markdown bundle;
- compact terminal summary;
- raw refs for deeper local reads.

## API Surface

V1 exposes a stable API for agents and tools.

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

No product client should query GreptimeDB or Turso directly. This keeps
redaction, grouping, auth, and bundle projection in one place; it is not a
promise that product engines are substitutable.

The API contract is specified in [Parallax API Concept](api-concept.md).

## Adopted Local Storage Contract

V1 adopted managed or external GreptimeDB plus Turso metadata:

| Need | Local V1 answer |
| --- | --- |
| install simplicity | one command starts Parallax plus managed GreptimeDB |
| observability evidence | local GreptimeDB standalone |
| grouping/state/config | local Turso file |
| local run retention | short TTL / manual prune |
| query scope | one developer machine, one or few projects |
| data volume | enough for local tests and small microservice runs |
| durability | good enough for debugging, not production compliance |

GreptimeDB is suitable locally because it runs in standalone mode as a binary (`greptime standalone
start`) and can be installed through the Greptime Homebrew tap on macOS. Docker is optional, not
required. The supported placements are:

```text
parallax serve --manage-greptime   # default local mode
parallax serve --greptime-url ...  # use existing GreptimeDB
```

Turso is the mandatory metadata engine. GreptimeDB stores telemetry evidence;
Turso stores product state. Failures are fixed in Parallax or upstream, not by
substituting an engine. In-memory adapters remain test/dev harnesses only.

## Historical Placement Projection

V1 local-first does not weaken the GreptimeDB decision. It clarifies tiers:

| Stage | Default storage | Why |
| --- | --- | --- |
| V1 local | managed or external GreptimeDB + Turso | one-command local agent debugging with real observability storage. |
| Future server | GreptimeDB + Turso | Exact supported placement is owned by plan 115. |
| Future concurrency | GreptimeDB + Turso | Worker/concurrency changes require plan 110's measured trigger. |

GreptimeDB is the evidence backend and Turso the metadata backend in every
profile. The table is a placement record, not a storage-choice roadmap.

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
> capture, managed local GreptimeDB, Turso metadata, and a bundle contract that
> preserves the same ownership boundaries across approved profiles.

If OpenObserve, SigNoz, or another tool ships this exact local developer loop with a strong agent-ready
bundle, Parallax must narrow or pivot.

## V1 Non-Goals

- no full dashboard suite;
- no production HA;
- no full Sentry API parity;
- no full Grafana replacement;
- no engine-free product mode;
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
