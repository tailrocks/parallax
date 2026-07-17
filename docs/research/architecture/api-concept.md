# Parallax API Concept

<!-- markdownlint-disable MD013 -->

Decision date: 2026-06-03

> **Status (2026-07-17): implemented and substantially broader than the original
> sketch.** Parallax uses Juniper code-first GraphQL with 76 queries, 14
> mutations, and no subscriptions. OTLP traces/logs/metrics and Sentry envelopes
> are accepted now; GitHub webhooks are also implemented. Live delivery uses
> SSE. Product clients use the Parallax API and never query GreptimeDB or Turso
> directly. The schema generated at `ui/graphql/schema.graphql` is authoritative.

## API Roles

Parallax has three different API jobs:

| Job | API | Why |
| --- | --- | --- |
| Telemetry ingest | OTLP HTTP/gRPC | Standard path for traces, logs, and metrics; Parallax derives `error_event` rows from exception span events, span error status, and ERROR/FATAL logs. |
| Error compatibility ingest | Sentry envelope HTTP endpoint | Shipped migration path for Sentry-style events. Raw frames are spooled before queue acknowledgement. |
| Integration ingest | GitHub webhooks | Shipped deploy, workflow, check, pull-request, and review context. |
| Query/exploration | GraphQL | The shipped code-first schema covers observability, evidence, product state, testing, and alerting. |

Keep these separate. GraphQL should not ingest raw telemetry.

## Product Boundary

All product clients go through Parallax API:

```text
CLI
UI
agents
  -> Parallax API
     -> services
        -> storage adapters
           -> GreptimeDB / Turso
```

Only storage adapters talk directly to databases. This centralizes:

- redaction;
- grouping;
- auth/policy;
- pagination/time-window limits;
- bundle projection;
- backend portability.

## Endpoints

Implemented core transport surface:

```text
POST /graphql
POST /v1/traces        # OTLP HTTP
POST /v1/logs          # OTLP HTTP
POST /v1/metrics       # OTLP HTTP
GET  /v1/logs/stream   # SSE
GET  /v1/traces/stream # SSE
GET  /healthz
GET  /readyz
GET  /version
```

OTLP/gRPC and HTTP listen on the standard ports:

```text
4317  OTLP/gRPC
4318  OTLP/HTTP
```

## GraphQL Query Shape

The original schema sketch below is historical. Current truth: 76 query fields
and 14 mutation fields span health/version; services, topology, RED analytics;
issues and trends; tests and flakiness; traces, links, events, critical paths,
and compare; logs, facets, and patterns; story/agent sessions and evidence gaps;
fields and read-only SQL; journeys, sessions, jobs, and conversations;
dashboards, investigations, and saved views; metric catalog, queries, summaries,
exemplars, and runtime metrics; invocations; and alert rules, destinations,
incidents, and checks. Mutations cover issue status, invocation lifecycle,
dashboards, investigations, saved views, alert rules, and alert destinations.

There is deliberately no GraphQL `Subscription` root; live logs and spans use
SSE. See generated schema for exact names and arguments.

Historical initial sketch:

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

The `Subscription` example is superseded: the implemented schema uses
`EmptySubscription`, and live transport is SSE.

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

### Sentry Envelope (Implemented)

Parallax exposes Sentry-compatible envelope ingest and normalizes supported
items into its evidence model:

```text
POST /api/<project_id>/envelope/
```

Implemented scope includes envelope framing, bounded HTTP ingest, raw-frame
spooling, acknowledgement tracking, and normalized error evidence. Unsupported
items remain bounded by the endpoint contract; Parallax does not claim full
Sentry API parity.

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
- read-only SQL is exposed through the guarded `sql` query;
- no direct backend object IDs unless wrapped as evidence refs;
- every bundle includes redaction and missing-evidence fields.

## Client Use

| Client | API path |
| --- | --- |
| CLI | GraphQL + health/version endpoints. |
| TanStack Start UI | GraphQL only. |
| Coding agent | CLI/GraphQL over the same service boundary. |
| App telemetry | OTLP traces/logs/metrics; Sentry envelope compatibility ingest. |
| Admin/ops | Health/version plus the implemented bounded GraphQL mutations. |

## Rust Implementation Direction

Implemented server stack:

- `axum` for HTTP server and health endpoints;
- Juniper for the code-first GraphQL schema and resolvers;
- `tonic`/OTLP crates for OTLP/gRPC;
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
- [Juniper](https://github.com/graphql-rust/juniper) — implemented code-first
  Rust GraphQL server library.
- [OpenTelemetry OTLP specification](https://opentelemetry.io/docs/specs/otlp/) — telemetry ingest
  protocol for traces/logs/metrics.
- [Sentry envelopes](https://develop.sentry.dev/sdk/foundations/envelopes/) — compatibility endpoint
  and event envelope format.
