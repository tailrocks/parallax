# Parallax API Concept

<!-- markdownlint-disable MD013 -->

Decision date: 2026-06-03

> **Decision — Parallax API is GraphQL-first for query/exploration and OTLP-first for V1 ingest.**
> Sentry-compatible ingest is a future adapter, not V1 scope. Product clients use Parallax API only.
> UI, CLI, agents, and future MCP adapters must not query GreptimeDB, Turso, Postgres, ClickHouse, or
> any future backend directly.

## API Roles

Parallax has three different API jobs:

| Job | API | Why |
| --- | --- | --- |
| Telemetry ingest | OTLP HTTP/gRPC | Standard path for traces, logs, and metrics; Parallax derives `error_event` rows from exception span events, span error status, and ERROR/FATAL logs. |
| Error compatibility ingest | Future minimal Sentry envelope endpoint | Migration path for Sentry-style error events and grouping fields after V1 proves the OTLP/local loop. |
| Query/exploration | GraphQL | Runs/issues/traces/logs/metrics/bundles are graph-shaped and need flexible field selection. |

Keep these separate. GraphQL should not ingest raw telemetry.

## Product Boundary

All product clients go through Parallax API:

```text
CLI
UI
agents
future MCP adapter
  -> Parallax API
     -> services
        -> storage adapters
           -> GreptimeDB / Turso / Postgres / ClickHouse / future backend
```

Only storage adapters talk directly to databases. This centralizes:

- redaction;
- grouping;
- auth/policy;
- pagination/time-window limits;
- bundle projection;
- backend portability.

## Endpoints

Recommended V1/V2 surface:

```text
POST /graphql
GET  /graphql/ws       # later subscriptions
POST /v1/traces        # OTLP HTTP
POST /v1/logs          # OTLP HTTP
POST /v1/metrics       # OTLP HTTP
GET  /healthz
GET  /readyz
GET  /version
```

OTLP/gRPC can listen on the normal OTLP gRPC port when implemented:

```text
4317  OTLP/gRPC
4318  OTLP/HTTP
```

## GraphQL Query Shape

Initial schema sketch:

```graphql
type Query {
  run(id: ID!): Run
  runs(filter: RunFilter, page: PageInput): RunConnection!

  issue(id: ID!): Issue
  issues(filter: IssueFilter, page: PageInput): IssueConnection!

  trace(id: ID!): Trace
  logs(filter: LogFilter!, page: PageInput): LogConnection!
  metricWindow(input: MetricWindowInput!): MetricWindow!

  evidenceBundle(anchor: EvidenceAnchorInput!): EvidenceBundle!
}

type Mutation {
  startRun(input: StartRunInput!): Run!
  finishRun(id: ID!, status: RunStatus!): Run!
  pruneRuns(input: PruneRunsInput!): PruneResult!
}

type Subscription {
  runUpdated(id: ID!): RunUpdate!
}
```

Subscriptions are optional. Do them after query/mutation works.

## Core Types

```graphql
type Run {
  id: ID!
  project: Project!
  status: RunStatus!
  startedAt: DateTime!
  finishedAt: DateTime
  services: [Service!]!
  issueCount: Int!
  errorCount: Int!
  spanCount: Int!
  logCount: Int!
  metricCount: Int!
  issues(page: PageInput): IssueConnection!
  timeline(filter: TimelineFilter): [TimelineItem!]!
}

type Issue {
  id: ID!
  fingerprint: String!
  title: String!
  status: IssueStatus!
  firstSeen: DateTime!
  lastSeen: DateTime!
  eventCount: Int!
  affectedRuns: [Run!]!
  stackTrace: StackTrace
  occurrences(page: PageInput): ErrorEventConnection!
  linkedSpans: [Span!]!
  logWindow(input: LogWindowInput!): LogConnection!
  metricWindows(input: MetricWindowInput!): [MetricWindow!]!
}

type Trace {
  id: ID!
  rootSpan: Span
  spans: [Span!]!
  durationMs: Float
  errors: [ErrorEvent!]!
  logs(page: PageInput): LogConnection!
}

type LogRecord {
  id: ID!
  timestamp: DateTime!
  severity: String
  serviceName: String
  traceId: String
  spanId: String
  body: String
  fields: JSON!
  redaction: RedactionStatus!
}

type EvidenceBundle {
  id: ID!
  anchor: EvidenceAnchor!
  generatedAt: DateTime!
  json: JSON!
  markdown: String!
  redactionReport: RedactionReport!
  missingEvidence: [MissingEvidence!]!
  queryManifest: [QueryManifestItem!]!
}
```

## Ingest APIs

### OTLP

Parallax accepts OTLP for:

- traces;
- logs;
- metrics.

The ingest layer normalizes data into Parallax evidence rows and writes through storage adapters.
`error_event` is a Parallax model, not a fourth OpenTelemetry signal or endpoint. V1 derives it
from span events named `exception`, spans with error status and `error.type`, and OTLP log records
with ERROR/FATAL severity plus `exception.*`, `trace_id`, and `span_id` when present.

### Future Sentry Envelope

Parallax may later expose minimal Sentry-compatible ingest:

```text
POST /api/<project_id>/envelope/
```

Future scope:

- accept `event` item;
- parse exception, stacktrace, release, environment, tags, breadcrumbs, trace context, debug metadata,
  fingerprint;
- reject or metadata-only-store unsupported items;
- normalize into Parallax issue/error evidence.

No full Sentry API parity.

## Guardrails

GraphQL must be safe by default:

- query depth limit;
- query complexity/cost limit;
- required pagination for logs/events/spans;
- max time-window per request;
- max log rows per page;
- no arbitrary SQL/PromQL passthrough in V1;
- no direct backend object IDs unless wrapped as evidence refs;
- every bundle includes redaction and missing-evidence fields.

## Client Use

| Client | API path |
| --- | --- |
| CLI | GraphQL + health/version endpoints. |
| TanStack Start UI | GraphQL only. |
| Coding agent | GraphQL, later MCP adapter over same service methods. |
| App telemetry | OTLP in V1; future optional Sentry envelope adapter. |
| Admin/ops | health/version; later limited GraphQL mutations. |

## Rust Implementation Direction

Likely server stack:

- `axum` for HTTP server and health endpoints;
- `async-graphql` for GraphQL schema/resolvers;
- `tonic`/OTLP crates for OTLP/gRPC later;
- service layer between GraphQL resolvers and storage adapters.

Important rule:

```text
GraphQL resolver -> service -> storage adapter
```

Never:

```text
GraphQL resolver -> GreptimeDB SQL directly
```

## Source Anchors

- [GraphQL specification](https://spec.graphql.org/) — typed schema, query/mutation/subscription root
  operation model, introspection.
- [async-graphql](https://github.com/async-graphql/async-graphql) — Rust GraphQL server library with
  framework integrations and subscriptions.
- [OpenTelemetry OTLP specification](https://opentelemetry.io/docs/specs/otlp/) — telemetry ingest
  protocol for traces/logs/metrics.
- [Sentry envelopes](https://develop.sentry.dev/sdk/foundations/envelopes/) — compatibility endpoint
  and event envelope format.
