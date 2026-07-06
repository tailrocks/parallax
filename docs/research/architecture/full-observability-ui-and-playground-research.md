# Full-Observability UI and Telemetry Playground Expansion

Research date: 2026-07-06

Status: brainstorming / design research. This is not an implementation plan and
does not authorize stack expansion. It is a detailed source-linked brief for a
future agent to inspect Parallax and `parallax-telemetry-playground`, then design
the next UI and playground generation.

## Executive thesis

Parallax should become a single execution-observability application that lets a
developer answer:

> What happened, where, why, who/what triggered it, what was affected, what did
> the system do next, and what evidence should a human or coding agent use to fix
> it?

The target is not "a cheaper Grafana", "a prettier Kibana", or "a local
Sentry". The stronger product shape is a single causal investigation console
that absorbs the best concepts from all three:

- **Grafana replacement:** service health, RED metrics, CPU/memory/runtime
  dashboards, custom metric panels, service/dependency map, historical windows.
- **Kibana replacement:** structured log search, field explorer, object inspect,
  saved columns, filters, query timeline, logs scoped by trace/run/span/service.
- **Sentry replacement:** grouped issues, stack traces, releases/regressions,
  trace-linked errors, breadcrumbs/user steps, frontend/backend context, issue
  lifecycle.
- **Parallax-only layer:** run-scoped CLI/coding-agent execution graph,
  evidence-bundle preview/export, redaction report, missing-evidence detection,
  and one canonical API for UI/CLI/agents.

Current Parallax already has the correct foundation: OpenTelemetry-native ingest,
GreptimeDB for telemetry, Turso for mutable metadata, issue derivation from spans
and logs, GraphQL as the only UI/API path, trace waterfalls, logs, runs, services,
dashboards, and run bundles. The next leap should be **causal UI**: not separate
pages for signals, but a navigable story graph that moves from ecosystem map →
time window → anomaly → trace/run/session → span/log/metric/profile/detail →
evidence bundle.

## Source base checked

### Local repositories

- Parallax repository: Rust workspace under `crates/`, TanStack Start UI under
  `ui/`, research under `docs/research/`.
- Playground repository: `parallax-telemetry-playground`, a Rust/Java/TypeScript
  polyglot OpenTelemetry + Sentry sample stack.
- Current Parallax product claim: OpenTelemetry-native execution context engine
  over traces/logs/metrics plus CLI/coding-agent execution traces, with grouped
  issues and bounded evidence bundles.
- Current UI stack: TanStack Start + shadcn/ui on Base UI, GraphQL-only data
  path, strict TypeScript, Bun-only tooling.
- Current storage decision: GreptimeDB only for telemetry, Turso for metadata.

### External sources checked

- OpenTelemetry semantic conventions 1.43.0 list official areas for HTTP, RPC,
  messaging, database, GraphQL, CLI, browser, feature flags, exceptions,
  resources, logs, metrics, traces, profiles, system/runtime, and CI/CD.
  Source: <https://opentelemetry.io/docs/specs/semconv/>.
- OpenTelemetry overview defines traces as DAGs of spans; span links represent
  async, batch, scatter/gather, cross-trace, and trusted-boundary causality;
  baggage propagates contextual key/value data for observability correlation.
  Source: <https://opentelemetry.io/docs/specs/otel/overview/>.
- OpenTelemetry CLI spans are development-stage; they cover short-lived
  programs, `process.executable.name`, `process.exit.code`, `process.pid`,
  `error.type`, and mark non-zero exit as error, but do not define a cross-trace
  run id. Source: <https://opentelemetry.io/docs/specs/semconv/cli/cli-spans/>.
- OpenTelemetry GraphQL server spans are development-stage; default span name is
  operation type, `graphql.operation.name` is client-provided/high-cardinality,
  `graphql.document` is opt-in and should be redacted. Source:
  <https://opentelemetry.io/docs/specs/semconv/graphql/graphql-spans/>.
- OpenTelemetry browser instrumentation remains experimental; browser docs show
  SSR `<meta name="traceparent">` for document-load correlation and add fetch,
  XHR, and user-interaction instrumentation. Source:
  <https://opentelemetry.io/docs/languages/js/getting-started/browser/>.
- OpenTelemetry baggage warns sensitive baggage can propagate to unintended
  downstream resources. Source:
  <https://opentelemetry.io/docs/concepts/signals/baggage/>.
- Grafana Explore is an ad-hoc query/analyze/aggregate entry point that avoids
  requiring dashboards before investigation. Source:
  <https://grafana.com/docs/grafana/latest/visualizations/explore/>.
- Grafana Tempo service graph derives RED metrics from traces, renders nodes and
  edges with request rate, error rate, duration, and links to traces/metrics.
  Source: <https://grafana.com/docs/grafana/latest/datasources/tempo/service-graph/>.
- Sentry Trace Explorer exposes span samples, span attributes, flexible queries,
  aggregate visualizations, waterfall navigation, attribute comparison, and
  cross-event querying. Source: <https://docs.sentry.io/product/trace-explorer/>.
- Sentry distributed tracing docs frame the ideal full-stack trace as a connected
  tree from frontend to backend to tasks/database, with async transactions that
  can outlive parents and orphan transactions as evidence gaps. Source:
  <https://docs.sentry.io/concepts/key-terms/tracing/distributed-tracing/>.
- OpenTelemetry Java agent supports broad auto-instrumentation including GraphQL
  Java, gRPC, Kafka, Spring WebFlux/WebMVC, Logback/Log4j, and opt-in Micrometer.
  Source: <https://github.com/open-telemetry/opentelemetry-java-instrumentation/blob/main/docs/supported-libraries.md>.
- OpenTelemetry Rust docs show traces, metrics, and logs using the Rust SDK plus
  `opentelemetry-appender-tracing`; `tracing-opentelemetry` bridges Rust
  `tracing` spans to OTel. Sources:
  <https://opentelemetry.io/docs/languages/rust/getting-started/> and
  <https://github.com/tokio-rs/tracing-opentelemetry>.
- OpenTelemetry Logs Data Model is stable and defines `Timestamp`,
  `ObservedTimestamp`, `TraceId`, `SpanId`, `TraceFlags`, `SeverityText`,
  `SeverityNumber`, `Body`, `Resource`, `InstrumentationScope`, `Attributes`,
  and `EventName`. It explicitly models logs and typed events from first-party,
  third-party, and system sources. Source:
  <https://opentelemetry.io/docs/specs/otel/logs/data-model/>.
- OpenTelemetry Metrics Data Model is stable, supports delta/cumulative
  temporality, transformations/reaggregation, histograms, exponential
  histograms, and **exemplars** that attach trace/span context to metric
  measurements. Sources:
  <https://opentelemetry.io/docs/specs/otel/metrics/> and
  <https://opentelemetry.io/docs/specs/otel/metrics/data-model/>.
- OpenTelemetry browser event semantic conventions define `browser.web_vital`
  events for Web Vitals such as CLS/LCP/INP-style measurements; status is
  Development. Source:
  <https://opentelemetry.io/docs/specs/semconv/browser/browser-events/>.
- Elastic Discover is the primary Kibana data-exploration tool: search/filter,
  field statistics, pattern analysis, individual document inspection, quick
  visualizations, saved sessions, reports, and alerts. Source:
  <https://www.elastic.co/docs/explore-analyze/discover>.
- Elastic service maps rely on distributed traces and fail to draw edges when a
  service is not instrumented or `traceparent` is not propagated; they expose
  average duration, requests/minute, errors/minute, service-specific focus, and
  anomaly indicators. Source:
  <https://www.elastic.co/docs/solutions/observability/apm/service-map>.
- Datadog Service Map and Catalog combine real-time observed dependencies,
  service type, deploy/incident/monitor status, ownership, reliability,
  performance, infrastructure links, and automatically discovered entities.
  Sources: <https://docs.datadoghq.com/tracing/services/services_map/> and
  <https://docs.datadoghq.com/internal_developer_portal/catalog/>.
- Datadog Service Page rolls service health, monitors, Watchdog insights,
  dependencies, out-of-box graphs, resources, deployments, error tracking,
  traces, security, and log patterns into one service drilldown. Source:
  <https://docs.datadoghq.com/tracing/services/service_page/>.
- Honeycomb's differentiator is high-cardinality/high-dimensional exploration.
  BubbleUp compares a selected anomaly against baseline across all dimensions
  and ranks fields/values that stand out. Sources:
  <https://docs.honeycomb.io/get-started/observability/concepts/high-cardinality>
  and <https://docs.honeycomb.io/investigate/analyze/identify-outliers>.
- Jaeger represents traces as DAGs via span references; its UI supports system
  architecture graphs, deep dependency graphs, service/endpoint granularity, and
  service performance monitoring from RED metrics. Source:
  <https://www.jaegertracing.io/docs/1.76/features/>.
- Grafana Tempo can generate RED/span metrics and service graphs from traces,
  and can add exemplars so metric spikes link to representative traces. Sources:
  <https://grafana.com/docs/tempo/latest/metrics-from-traces/> and
  <https://grafana.com/docs/tempo/latest/metrics-from-traces/span-metrics/span-metrics-metrics-generator/>.
- Tokio metrics exposes task and runtime metrics such as worker count, busy
  duration, queue depth, live tasks, blocking queue depth, blocking thread
  counts, forced yields, and I/O readiness; it is intended for production
  metric reporting, while Tokio Console is primarily local debugging. Sources:
  <https://docs.rs/tokio-metrics> and
  <https://github.com/tokio-rs/tokio-metrics>.
- Java runtime metrics in OpenTelemetry-style ecosystems include JVM heap/
  non-heap memory, committed/init/limit, live thread count, class loading,
  process/system CPU utilization, direct/mapped buffer counts/usage/limit, and
  GC-related signals. Source:
  <https://docs.datadoghq.com/opentelemetry/integrations/runtime_metrics/>.

## Current Parallax capabilities to preserve

### Product architecture

Parallax already has these important properties:

1. **One canonical API boundary.** The UI calls `src/lib/api.ts` → `/graphql`;
   direct storage access is forbidden. CLI/agents should use the same conceptual
   boundary.
2. **Native OpenTelemetry rows.** Normalized rows include spans, logs, metric
   points, histograms, span events, span links, resource attributes, scope name,
   and optional `run_id`.
3. **Derived issue layer.** Parallax derives `ErrorEventRow` from:
   - span exception events;
   - span error status;
   - ERROR/FATAL logs;
   - exception-as-log attributes.
4. **Issue metadata.** Issues track fingerprint, title, error type, culprit,
   service, status, first/last seen, count, tags, trend, and last trace.
5. **Run scope.** `parallax.run.id` is the canonical internal run key. Runs are
   first-class UI/API objects with trace/log/error/bundle relationships.
6. **Trace detail.** The UI renders waterfalls, selected span detail, attributes,
   resource attributes, span links, span events, trace logs, run link, failed-span
   shortcut, and copyable IDs.
7. **Live mode.** Run detail can stream logs and spans via SSE for a running run.
8. **Service health.** Services list includes last seen, span count, errors, p95,
   error rate, and links into service pages.
9. **Dashboards.** User dashboards already exist as metadata rows and metric
   query surfaces.

These are exactly the primitives needed for a Grafana/Kibana/Sentry replacement.
The missing layer is not storage primitives; it is **an opinionated UI
information architecture that composes them into a causal narrative**.

### Current UI shape

Current pages:

- Overview: global counts, spans/errors series, RED chart, recent issues, slow
  traces.
- Issues list/detail: grouped errors, trends, events, stack traces, tags,
  trace/log links.
- Traces list/detail: lookup and waterfall.
- Logs: structured log table/filtering.
- Runs list/detail: command/run status, live logs/spans, traces, bundle.
- Services list/detail: health and service metrics.
- Dashboards/SQL: metric panels and ad-hoc SQL/data exploration.

This is a solid V1 console, but it still looks like signal pages. The next UI
should feel like a single investigation surface.

## Current playground capabilities to preserve

The playground already exercises many important cases:

- Rust services with Axum, tonic gRPC, reqwest, `tracing`, OTLP traces/logs/
  metrics, Sentry SDK, flush-on-exit discipline.
- Java services with Spring GraphQL, Spring gRPC, Kafka, JVM metrics, Sentry
  agent/SDK path, and GraphQL subscription support.
- TypeScript TanStack Start browser app with Sentry RUM/session replay path and
  OTel browser/document-load/fetch/user-interaction path.
- Cross-language trace propagation through W3C `traceparent`/`tracestate` and
  baggage.
- Async messaging branch with producer/consumer spans and span links.
- Rust CLI driver with short-lived telemetry flush discipline.
- Failure catalog: request failure, degraded response, retry/timeout, high CPU,
  cache leak, consumer lag, poison/dead-letter, N+1, lock contention, latency,
  cron success/fail/stuck, canary redaction, deploy regression, clock skew.

This is already better than a toy demo. The gap is that it still models mostly
an e-commerce microservice world. Parallax needs the playground to also model
**interactive execution systems**: host CLI → daemon → workspace/session →
container capsule → multiplexer → multiple agents → tools/commands/files/tests.

## Product goal: one intuitive causal graph

The UI should organize every signal around five correlated identities:

| Identity | OTel/Parallax carrier | Purpose |
| --- | --- | --- |
| Service/process identity | `service.name`, `service.version`, resource attrs | Who emitted telemetry. |
| Trace identity | `trace_id`, `span_id`, parent/links | One request/workflow causality tree/DAG. |
| Run identity | `parallax.run.id` resource attr | One bounded CLI/session/workspace execution across many traces. |
| User/workspace/session identity | allowlisted baggage/resource attrs, future normalized rows | Human-visible path: screen, workspace, capsule, agent session. |
| Issue/fingerprint identity | Parallax fingerprint + issue metadata | Stable problem grouping across events/releases/runs. |

The user should never need to know which signal has the answer. The UI should
infer the best next view:

```text
ecosystem map → service edge hot/error → traces for that edge → slow span
  → span logs/attributes/events → related metric window → issue or run bundle

issue spike → compare attributes → overrepresented release/workspace/screen
  → sample traces → exact span/log/user step → evidence bundle

run/session → screen timeline → command/tool/container spans → error
  → affected service/trace/logs → bundle for agent
```

## UI information architecture proposal

### 1. Command center: "What changed, what broke, what is hot?"

Replace the generic overview with a dense investigation command center:

- **Global time brush**: every panel respects it; brushing any chart updates the
  whole page and offers "open investigation for this window".
- **Incident/anomaly lane**: grouped issue spikes, new services, new releases,
  failed runs, broken service edges, sudden p95/p99 changes, log-error bursts,
  missing trace continuation, consumer lag, CPU/memory/runtime anomalies.
- **Service map mini-panel**: nodes colored by health; edges colored by failure
  and latency; clicking an edge opens edge-specific traces/logs/metrics.
- **Top causal clues**: not AI prose, but deterministic summaries:
  - "release v2 overrepresented in selected spike";
  - "checkout → payment p95 up 6.4×";
  - "browser checkout button has errors but no backend continuation";
  - "consumer spans link to producer trace but parent chain is absent";
  - "run has failed CLI exit and 3 errored spans".
- **Fast pivots**: traces, issues, logs, runs, services, metrics, SQL.

Best borrowed ideas:

- Grafana Explore: allow ad-hoc query without dashboard setup.
- Sentry Trace Explorer: span samples, arbitrary attributes as columns, aggregate
  from any numeric attribute.
- Honeycomb-style compare: select spike → compare selected vs baseline
  attribute distributions.
- Kibana Discover: field browser and saved columns for logs/spans/events.

### 2. Ecosystem map: service graph + execution graph

Parallax should have a first-class **Ecosystem** page, not only a services list.

Views:

1. **Service dependency graph** derived from span parent/child edges and
   client/server span pairing:
   - node = service, database, broker, browser, CLI, external API, container;
   - edge = calls/messages/streams;
   - labels = rate, error rate, p50/p95/p99, recent issue count;
   - edge drilldown = traces/logs/metrics for exactly this relationship.
2. **Execution graph** for CLI/container/agent systems:
   - host CLI;
   - daemon;
   - workspace/session;
   - capsule/container;
   - multiplexer/session;
   - agent processes;
   - shell commands/tools/files/tests;
   - external services.
3. **Layer toggles**:
   - services only;
   - infra/resources;
   - frontend screens;
   - CLI/agent sessions;
   - databases/brokers;
   - failed edges only;
   - selected run only.

UI behavior:

- Click node: service/run/container detail drawer.
- Click edge: relationship drawer with rate/error/duration charts, last failed
  traces, representative logs, endpoint/method breakdown.
- Drag-select multiple nodes: ask "what traces crossed these components?".
- Time brush: graph animates change from baseline to selected window.

This page is the visual answer to "who connects to who, who replied, how long did
it take, where is the problem?"

### 3. Timeline/story view: user-visible steps + hidden work

For any trace, run, or issue occurrence, add a **Story** tab. It should render a
chronological, human-readable sequence, grouped by layer:

```text
00.000  user/browser   route /checkout entered
00.035  user/browser   clicked "checkout"
00.048  browser        fetch GET /checkout started
00.083  checkout       request accepted, tenant=demo, tier=free
00.096  checkout       fan-out: pricing, inventory, recommendation
00.144  pricing        Quote returned 3998 USD
00.211  inventory      N+1 loop, 8 sequential calls
00.390  checkout       returned 200
00.421  browser        rendered confirmation
```

For a CLI/container/agent system:

```text
00.000  host-cli       session start workspace=parallax
00.014  daemon         allocated run_id=...
00.084  docker         created capsule image=...
00.310  container      attached multiplexer session
00.442  agent-1        started model=...
01.120  agent-1        read docs/research/architecture/...
03.210  agent-1        ran cargo test -p parallax-core
03.800  shell          exit=101 error.type=test_failure
04.010  agent-1        proposed fix patch_hash=...
```

This is the key UI shape for "I was on this screen, selected that item, pressed
that button, these operations happened in parallel/background, then I entered a
container and started an agent."

Implementation concept for future agent:

- Use spans as the backbone.
- Use span events for micro-events inside a span: screen-enter, item-select,
  button-press, tool-start/tool-end, retry-attempt, state transition.
- Use logs for narrative details and diagnostics.
- Use metrics for side panels around the same time window.
- Use links for async causality and cross-trace relationships.
- Use `parallax.run.id` to stitch many traces into one execution story.

### 4. Trace detail: waterfall + DAG + compare + missing evidence

Current waterfall is necessary but not sufficient. Add modes:

- **Waterfall mode**: current tree view with selected span drawer.
- **DAG mode**: parent/child plus span links. Required for async, batch,
  scatter/gather, and producer/consumer cases. Linked spans should not be hidden
  in JSON.
- **Critical path mode**: highlight longest path and wait gaps.
- **Parallelism mode**: show fan-out/fan-in and idle/wait time.
- **Errors-only mode**: collapse non-error spans but preserve ancestors.
- **Semantic lane mode**: group spans by browser, HTTP, gRPC, GraphQL, DB,
  messaging, runtime, CLI, container, agent.
- **Evidence gaps**:
  - orphan server span;
  - browser span with no backend child;
  - producer span with no consumer link;
  - consumer span with no creation context;
  - logs without trace/span id;
  - high-cardinality or unsafe attributes redacted/dropped;
  - sampled-out children.

Span drawer sections:

- Summary: name, service, kind, duration, status, route/method/op.
- Causality: parent, children, linked spans, related traces, run id.
- Attributes: searchable, grouped by namespace (`http.*`, `rpc.*`, `db.*`,
  `messaging.*`, `graphql.*`, `process.*`, `cli.*`, `gen_ai.*`, custom).
- Events: exception, retries, UI steps, state transitions.
- Logs: only this span, then trace logs.
- Metrics near span: CPU/memory/runtime/request histograms around span window.
- Source/context: stack frames, release, repo commit, command, container.
- Actions: copy trace/span/run, open logs, open SQL, create bundle, compare
  selected-span attributes vs baseline.

### 5. Logs: Kibana-style field explorer, but trace/run-native

Logs should become an investigative object browser:

- Query bar for body and attributes.
- Field explorer: top values, cardinality hints, type, coverage percentage.
- Column presets: service/trace/run/span/severity/message; HTTP; GraphQL;
  database; CLI; agent; container.
- Inline trace/run/issue chips on every row.
- Surrounding logs: ±N seconds around selected log, grouped by trace/run.
- Pattern collapse: group repetitive logs; expand on demand.
- Error derivation marker: show if this log created an `ErrorEventRow`.
- Redaction marker: show dropped/masked fields and reason.

### 6. Issues: Sentry-grade grouping plus OTel-native context

Issue detail should answer more than "stack trace + events":

- Trend by environment/release/service/run/workspace/screen.
- Attribute compare for spike vs baseline.
- Representative traces: newest, slowest, highest fan-out, with/without logs,
  cross-language, async-linked.
- Related metrics: error-rate, request duration, CPU/memory/runtime, queue lag.
- Related logs: newest errors, span-correlated logs, uncorrelated logs in window.
- Related runs: failed CLI/agent/container sessions with this fingerprint.
- Regression lane: first seen in release X; resolved; reappeared in release Y.
- Evidence-bundle preview: what a coding agent will receive, redaction report,
  missing evidence, token size.

### 7. Runs/sessions: first-class local execution observability

Runs should become the bridge between application observability and CLI/agent
observability.

Required UI sections:

- Run header: id, command, status, exit code, duration, service count, trace
  count, issue count, last activity.
- Process tree: wrapper → child commands → daemon/container/agent processes.
- Screen timeline: TUI/screen/view transitions, selected items, button presses,
  background operations.
- Container/capsule panel: image, container id, workspace mount, attach time,
  multiplexer session id, environment policy.
- Agent timeline: agent start/end, prompts/context loads/tools/files/commands,
  validations, outcomes. Content redacted by default; structural facts visible.
- Trace list: all traces in the run, grouped by phase/screen/service.
- Logs: current live stream plus historical search.
- Metrics: process CPU/mem, tokio runtime, container CPU/mem/net/disk, queue
  depth, agent/tool latency.
- Bundle: preview/export/copy CLI command.

### 8. Metrics/dashboards: Grafana-grade enough, but investigation-first

Parallax does not need full Grafana dashboard parity to replace Grafana for the
operator's local/debugging use. It needs opinionated default panels and easy
custom panels over any metric.

Must-have default dashboards:

- Global RED and error budget style overview.
- Per-service RED: request rate, error rate, latency p50/p95/p99.
- Process CPU/memory/file descriptors/network/disk if available.
- Runtime dashboards:
  - Rust/Tokio task queue, poll time, blocking threads, runtime worker metrics;
  - JVM heap/non-heap, GC pause, thread count, class loading, safepoints if
    available;
  - browser web vitals and long tasks;
  - container CPU/memory/network/disk.
- Messaging: producer rate, consumer rate, lag, retries, dead letters.
- Database: query duration, returned rows, connection/pool wait, errors.
- GraphQL: operation rate, field/resolver latency, DataLoader batch size,
  partial errors, subscription lifetime.
- CLI/agent: command duration, exit-code histogram, tool latency, token counts
  if available, validation failures.

Investigation behavior:

- Brush any metric window → show errors/traces/logs in that exact window.
- Click a histogram bucket/outlier → sample traces/log rows.
- Compare selected window vs baseline by attributes.
- Every metric panel has "explain by traces" if exemplars/trace ids exist, else
  "nearby traces" with caveat.

### 9. SQL/ad-hoc explore: powerful but safe

Keep SQL for advanced users, but wrap it in safe affordances:

- Saved query snippets for common questions.
- Query builder for spans/logs/metrics/issues/runs.
- Explain/preview row count before wide scans where possible.
- Output rows link back to trace/run/issue/log/span.
- Redaction policy applies to query results shown to agents.

## Telemetry model recommendations

### Resource attributes: stable identity first

Every service/process should set:

- `service.name`
- `service.version`
- `service.namespace` where useful (`playground`, `parallax`, `tailrocks`)
- `deployment.environment.name`
- `telemetry.sdk.language`, `telemetry.sdk.name`, `telemetry.sdk.version`
- process/runtime attributes emitted by SDKs
- `parallax.run.id` when run-scoped
- `parallax.workspace.id` only if non-sensitive/opaque
- `parallax.capsule.id` for container/session scope
- `parallax.screen.name` only if stable low-cardinality screen identity can be
  stamped as resource for a short-lived process; otherwise span attributes/events

Do not put high-cardinality or PII identity in resource attributes. Use opaque
ids, with metadata resolved inside Parallax/Turso only when allowed.

### Span names: low-cardinality, human meaningful

Follow OTel guidance:

- HTTP server: `GET /checkout`, route template not raw path id.
- gRPC/RPC: package/service/method.
- GraphQL: default `query`, `mutation`, `subscription`; do not default to client
  operation name in span name.
- CLI callee/caller: executable name or documented low-cardinality command shape.
- UI action: `ui.click`, `ui.route`, `ui.submit` with target attributes, not raw
  user text.
- Agent tool: `agent.tool.call`, `agent.shell.command`, `agent.file.edit` with
  stable `tool.name`/`action.kind` attrs.

### Span events: use for user-visible and internal micro-steps

Span events should carry timeline details without exploding spans:

- `ui.screen.enter`, `ui.screen.exit`
- `ui.block.render`, `ui.block.action`
- `ui.select`, `ui.click`, `ui.submit`
- `cli.prompt`, `cli.selection`, `cli.attach`, `cli.detach`
- `container.create`, `container.start`, `container.exec`, `container.attach`
- `agent.context.load`, `agent.tool.start`, `agent.tool.end`
- `retry.attempt`, `deadline.exceeded`, `fallback.used`, `degraded.response`
- `feature_flag.evaluation`
- `exception`
- `redaction.canary.detected`

Events should use low-cardinality names; variable values live in attributes with
redaction/cardinality rules.

### Span links: required for reality

Parent/child is not enough. Parallax should treat links as first-class UI edges.

Use links for:

- messaging producer → consumer;
- batch job processing many messages;
- GraphQL DataLoader batch spanning multiple resolver parents;
- fan-in aggregation where one span summarizes multiple upstream spans;
- retry attempts that start new traces;
- CLI host run → container-internal trace when trust boundary creates a new
  trace;
- agent session → command traces;
- external trace import or remote tool run.

UI must show these as causal edges, not buried JSON.

### Baggage: useful but dangerous

Baggage can carry request context such as tenant tier or cart id downstream, but
OpenTelemetry warns it can reach unintended services. Parallax guidance:

- allowlist only non-sensitive keys;
- prefer opaque ids;
- never use raw emails, tokens, prompts, secrets, file contents;
- strip before third-party calls unless explicitly allowed;
- display baggage-origin and redaction status in span detail;
- treat baggage as untrusted input.

### Logs: structured, trace-correlated, derivable

Logs should preserve the original body, severity, trace/span/run ids, and
attributes. For Parallax issue derivation and UI:

- ERROR/FATAL logs create candidate error events.
- `exception.type`, `exception.message`, `exception.stacktrace` should be emitted
  when available.
- Logs without trace/span/run context should be visible as evidence gaps.
- Log bodies and attributes need redaction before bundle projection.

### Metrics: standard names + runtime-specific feeds

Prioritize:

- HTTP/RPC duration histograms and request counters.
- `process.*`, `system.*`, runtime metrics.
- JVM metrics from Java agent/Micrometer when enabled.
- Rust/Tokio metrics copied into OTel gauges/histograms.
- Container metrics from Docker/cgroup source.
- Queue lag and message processing duration.
- GraphQL resolver/DataLoader metrics.
- CLI/agent command duration, exit codes, tool latency.

The UI should prefer OTel semantic metric names, but preserve custom metrics and
make them discoverable in dashboards.

## Playground expansion proposal

### Keep the e-commerce stack, add an execution stack

The existing e-commerce topology is good for microservice observability:

```text
browser → checkout → pricing / inventory / recommendation / catalog / payment
                     └→ broker → fulfillment → notifications
cli → checkout
```

Add a second scenario family that models the host/daemon/container/agent world
without referencing any specific external project in product docs:

```text
host CLI → long-running daemon → workspace registry
   → session start → container capsule → multiplexer
      → agent A / agent B
         → shell commands → app services / files / tests / git
         → errors/logs/metrics/traces
```

This lets Parallax demonstrate something Grafana/Kibana/Sentry do not show well:
a single graph across local CLI state, container runtime, agent actions, and
application telemetry.

### New playground domains/scenarios

#### A. Frontend/browser scenarios

- Route enter/exit spans for `/`, `/checkout`, `/orders/:id`, `/admin`.
- User action events: select item, apply promo, submit checkout, cancel, retry.
- Background prefetch spans that can fail without user action.
- Rage click / unresponsive button.
- Web vitals: LCP/CLS/INP/FCP/TTFB plus attribution where available.
- Browser fetch with missing CORS propagation to create a visible evidence gap.
- Frontend error with source-mapped stack and backend trace link.
- Session replay reference as Sentry-only comparison, not default Parallax
  bundle content.

#### B. GraphQL scenarios

- Query with multiple resolvers and DataLoader batching.
- N+1 resolver path vs batched path.
- Partial error: GraphQL returns 200 with field-level error.
- Subscription: long-lived span, emitted updates, disconnect/reconnect.
- Client-supplied high-cardinality operation name to test span naming policy.
- Redacted `graphql.document` opt-in case.

#### C. gRPC scenarios

- Unary Rust→Rust, Rust→Java, Java→Rust.
- Server streaming quote stream.
- Deadline exceeded.
- Retry with per-attempt spans/events.
- Cancellation from upstream.
- Status code mapping and error.type.

#### D. Messaging/async scenarios

- Producer span injects creation context.
- Consumer process span links to producer.
- Batch consumer links to many producers.
- Poison message repeated redelivery + dead letter.
- Consumer lag metric and lag span attribute.
- Orphan consumer with missing creation context.

#### E. Database/cache scenarios

- Postgres query spans with sanitized `db.query.text`.
- Slow query and lock wait.
- Connection pool contention.
- N+1 sequential query pattern.
- Returned rows and query-plan-like metadata where safe.
- Redis/cache hit/miss and cache stampede/leak.

#### F. Runtime/system scenarios

- Rust Tokio runtime saturation: task queue/poll/blocking/thread metrics.
- Lock contention spans.
- High CPU busy loop.
- Memory leak/cache leak.
- JVM GC pause and heap growth.
- Container CPU/memory limits and throttling.
- Process crash/non-zero exit.

#### G. CLI/session/container/agent scenarios

- Short-lived CLI root span with `process.exit.code` and `parallax.run.id`.
- Daemon receives session command; links/parents to host CLI.
- Workspace selection screen and settings changes as span events.
- Container create/start/attach spans; container id/image/resource attrs.
- Multiplexer attach/detach/session spans.
- Multiple agents in one container; each agent has its own session trace but
  shares `parallax.run.id`.
- Agent tool calls: file read, file edit, shell command, test command, API call.
- Agent command failure and retry.
- Agent starts background task while user switches screens.
- User exits container while background process continues.
- Missing/late flush from short-lived process.

#### H. Release/change scenarios

- Deploy v1 clean, v2 regression.
- Feature flag flips from healthy to failing path.
- New service version overrepresented in spike.
- Issue resolved then regressed.
- Different environment names.

#### I. Redaction/safety scenarios

- Canary secrets in logs/span attributes/baggage/GraphQL document/CLI args.
- PII in URL query/referrer.
- Prompt-injection-like telemetry body that must be treated as data, not
  instructions.
- Bundle preview shows redaction report and blocked raw refs.

### Playground acceptance questions

The expanded playground should let a reviewer ask these questions in Parallax UI
and get an intuitive answer:

1. Which service edge got slow in the selected window?
2. Which release or feature flag caused the error spike?
3. Which exact user action triggered this backend trace?
4. Which GraphQL field/resolver caused the latency?
5. Was this async consumer caused by this producer, and where is the link?
6. Did a CLI run fail because of a command exit, service error, container issue,
   or agent action?
7. Which container/agent/file/command happened before this runtime failure?
8. Did CPU/memory/runtime metrics change before the error or after?
9. Are there missing spans/logs/links that make the answer incomplete?
10. What evidence bundle would be given to a coding agent, and what was redacted?

## Parallax data/API expansion ideas

These are future design concepts, not immediate code instructions.

### Query surfaces

Add GraphQL/query concepts for:

- `ecosystemGraph(from,to,scope)` → nodes/edges with RED metrics and issue counts.
- `relationship(source,target,from,to)` → edge metrics, traces, logs, attributes.
- `story(traceId|runId|issueEventId)` → normalized timeline rows from spans,
  events, logs, metrics, and metadata.
- `attributeCompare(selection, baseline, entity)` → overrepresented attributes.
- `evidenceGaps(traceId|runId)` → missing propagation/link/log/resource facts.
- `spanLinks(traceId)` → resolved linked spans/traces.
- `metricNames/search` → custom dashboard builder autocomplete.
- `runtimeMetrics(service|run|container)` → standard runtime panels.
- `agentSession(runId)` → normalized agent actions, redacted content refs.

### Derived tables/materializations

Potential GreptimeDB/Turso-derived data:

- `service_edges_minute`: source, target, transport, rate, errors, p50/p95/p99.
- `span_attribute_rollups`: key/value counts by time/service/span kind for
  compare UI.
- `evidence_gaps`: trace/run gap detections.
- `story_events`: normalized timeline rows from span events/logs/run metadata.
- `runtime_metric_rollups`: process/runtime/container summary by service/run.
- `agent_actions`: low-volume normalized action metadata in Turso, high-volume
  content refs elsewhere.

### UI components

Reusable components likely needed:

- `EcosystemGraph`: graph layout with node/edge drawers.
- `StoryTimeline`: grouped lane timeline.
- `AttributeCompare`: selected vs baseline breakdown table.
- `EvidenceGapList`: actionable instrumentation gaps.
- `SpanLinkGraph`: resolved links, async edges.
- `MetricBrush`: common brush-to-filter behavior.
- `FieldExplorer`: Kibana-like attributes sidebar.
- `RedactionBadge/Report`: visible safety state.
- `BundlePreview`: exact agent-visible evidence.

## Second research pass: additional gaps to add

This pass checked official OpenTelemetry specs plus Elastic, Datadog,
Honeycomb, Jaeger, Grafana Tempo, Tokio, and JVM-runtime observability sources.
The strongest missing idea is that Parallax should not merely correlate traces,
logs, and metrics. It should make every correlation **actionable from the UI**:
click a spike, explain the dimensions, jump to representative traces, inspect
runtime state, see evidence quality, and save the investigation.

### A. Metric exemplars: spike → exact trace, not "nearby traces"

OpenTelemetry exemplars attach context to a metric measurement, commonly
`trace_id` and `span_id`. Grafana/Tempo use this to put clickable exemplar dots
on metric charts and jump from a latency/error spike to the representative
trace.

Parallax should add an explicit exemplar-first UX:

- Every histogram/heatmap panel should render exemplar markers when the OTLP
  metric point includes trace/span context.
- Clicking a marker opens a compact popover:
  - metric name, value, bucket/window;
  - trace id / span id;
  - service, route/op, status, run id if present;
  - actions: open trace, open span, compare this bucket, add to bundle.
- If no exemplars exist, the panel should show a transparent fallback:
  "No trace exemplar attached; showing traces near this timestamp".
- The playground should include two paired scenarios:
  1. a latency histogram with exemplars enabled, where clicking p99 jumps to the
     exact slow trace;
  2. the same metric without exemplars, proving the UI clearly marks lower
     confidence.

This is critical for Grafana replacement because dashboards become investigation
entry points, not dead charts.

### B. Stable OTel logs/events model: logs as typed evidence, not text rows

The OTel Logs Data Model is stable and includes trace/span/run-correlatable
fields plus `EventName`. Parallax should treat logs in two tiers:

1. **Log records:** raw/semi-structured messages with severity, body,
   attributes, resource, trace/span id.
2. **Typed events:** log records with `EventName` and known semantic attributes,
   such as exceptions, browser web vitals, feature-flag evaluations, user
   interactions, redaction detections, and lifecycle events.

UI additions:

- Logs page should have a **Type/Event** column separate from severity.
- Story timeline should prefer typed events over parsing log body text.
- Issue derivation should record whether the issue came from:
  - span status;
  - exception span event;
  - exception log event;
  - plain ERROR/FATAL log;
  - runtime/container process exit.
- Log detail should show:
  - source timestamp vs observed timestamp;
  - resource attributes;
  - instrumentation scope;
  - trace/span/run chips;
  - whether body/attrs were redacted before bundle projection.

Playground additions:

- Java Logback/Log4j bridge path emits OTel logs with trace context.
- Rust `tracing`/OTel appender path emits logs with trace context.
- TypeScript frontend emits browser events and errors with trace context where
  possible.
- One scenario intentionally emits uncorrelated logs so Parallax can display
  "evidence gap: log has no trace/span/run id".

### C. Field statistics and pattern analysis: Kibana Discover parity, but scoped

Elastic Discover's strongest idea is not its query language; it is immediate
field understanding: top values, field statistics, patterns, saved columns, and
document inspection. Parallax should add this pattern to spans, logs, events,
and issues.

Add a reusable **Field Explorer** drawer:

- Works for logs, spans, span events, metrics attributes, issue tags, and run
  metadata.
- Shows for each key:
  - type;
  - coverage percentage in current selection;
  - top values;
  - approximate cardinality;
  - semantic namespace (`http`, `rpc`, `db`, `messaging`, `graphql`, `process`,
    `resource`, `parallax`, custom);
  - safety status: safe, redacted, denied, high-cardinality warning.
- Actions per value:
  - filter include/exclude;
  - group by;
  - compare selected vs baseline;
  - add/remove column;
  - copy field path;
  - open examples.

This gives Kibana replacement behavior without making users write SQL first.

### D. BubbleUp-style attribute compare: make "why" visible

Honeycomb's BubbleUp compares an anomalous selection against a baseline across
all dimensions. Parallax should implement the same mental model in a
stack-specific way.

Entry points:

- drag-select a metric spike;
- click a service-map edge with errors;
- select failed traces on trace list;
- open an issue spike;
- select slow GraphQL resolver spans;
- select failed CLI runs or agent commands;
- select one container/session window.

Output should be deterministic and inspectable:

| Rank | Field | Selected | Baseline | Why useful |
| --- | --- | --- | --- | --- |
| 1 | `service.version` | `2.0.0` 92% | `2.0.0` 4% | likely deploy regression |
| 2 | `graphql.field.name` | `Order.items` 88% | 11% | resolver-specific |
| 3 | `parallax.screen.name` | `workspace-select` 81% | 7% | UI path-specific |
| 4 | `container.image.id` | `sha256:...` 74% | 3% | capsule-specific |

Guardrails:

- Do not group by raw user text, prompt text, URL query, secrets, stacktrace
  body, or high-risk baggage.
- High-cardinality ids are allowed for **filtering and sample drilldown**, but
  the compare UI should label them as exact identifiers, not stable categories.
- Prefer low-cardinality semantic fields first, then expose raw/high-cardinality
  fields behind "show exact ids".

### E. Service catalog: operational ownership plus telemetry reality

Datadog Catalog and Service Page show a useful pattern: a service is not only a
node in a graph. It has owners, deploys, health, dependencies, monitors,
resources, security signals, incidents, and out-of-box graphs.

Parallax should add a local-first **Service Catalog** page/drawer:

- Identity: `service.name`, namespace, version, runtime/language, framework,
  telemetry SDK.
- Ownership: repo/path, local workspace, team label if configured, run/source
  that last emitted telemetry.
- Health: RED, runtime health, open issues, failed runs, latest regression,
  evidence gaps.
- Dependencies:
  - observed upstream/downstream from traces;
  - manually declared relationships in metadata if future product needs them;
  - missing/unknown edges caused by propagation failures.
- Runtime/resources:
  - processes/containers that host the service;
  - CPU/memory/thread/task metrics;
  - recent deploy/release/version changes.
- Developer actions:
  - open ecosystem focus;
  - open service story;
  - open traces/logs/issues/metrics;
  - create evidence bundle for this service.

Playground should include service metadata files or emitted resource attrs for
Rust, Java, and TypeScript services so the catalog has meaningful content.

### F. Topology graph levels: one-hop, transitive, endpoint, run-specific

Jaeger's distinction between system architecture and deep dependency graphs is
important. A one-hop edge graph does **not** prove full request path causality.

Parallax should expose four graph modes:

1. **Observed one-hop graph:** all observed direct edges in selected window.
2. **Trace path graph:** only edges that appear together in matching traces.
3. **Transitive/focal graph:** all downstream/upstream paths through selected
   service, endpoint, run, screen, container, or agent.
4. **Endpoint/resource graph:** node granularity can switch from service to
   route/RPC method/GraphQL field/queue/topic/database/cache/container/agent.

UI must label which graph mode is active. Otherwise users may infer causality
that the data does not prove.

### G. Runtime/profiling lane: explain CPU, memory, GC, Tokio starvation

The original file mentions CPU/memory and runtime dashboards, but the second
research pass found enough detail to make this a first-class design axis.

Add a **Runtime** lane to service, trace, run, and story views:

- Rust/Tokio:
  - worker count;
  - busy ratio/duration;
  - global/local queue depth;
  - live task count;
  - blocking queue depth;
  - blocking thread count;
  - budget-forced yield count;
  - task poll-duration histogram where available;
  - lock contention spans/events.
- Java/JVM:
  - heap/non-heap usage/committed/limit;
  - GC pause/count/time;
  - live thread count;
  - loaded class count;
  - process/system CPU utilization;
  - direct/mapped buffer usage/count/limit;
  - Spring/GraphQL/gRPC/Kafka runtime correlations.
- Browser/TypeScript:
  - Web Vitals events;
  - long tasks;
  - route transition timing;
  - fetch/XHR timing;
  - user interaction latency.
- Container/process:
  - CPU throttling;
  - memory limit/working set;
  - OOM/crash/non-zero exit;
  - file descriptors;
  - network/disk I/O.

UI behavior:

- In trace view, show runtime panels for services active during the trace time
  window, not only global service dashboards.
- In run view, align runtime metrics with CLI/container/agent story rows.
- In issue view, compare runtime metrics in issue windows vs baseline.
- In bundle preview, include runtime snapshots only when they explain the
  failure, with source metric names and time windows.

Longer-term research note: OpenTelemetry has profile semantic conventions, but
profiling support is not yet as uniformly mature as traces/logs/metrics across
the current stack. Treat profiles as future-aligned UI slots, not required V1
scope.

### H. Frontend/session observability without full session replay dependency

Grafana Frontend Observability and Sentry-style experiences emphasize user
sessions, journeys, navigation performance, and trace links from frontend to
backend. Parallax can capture the high-value structure without making replay a
default dependency.

Add a **User Journey** concept for browser and CLI/TUI:

- Browser:
  - route enter/exit;
  - web vitals;
  - user interactions;
  - fetch/XHR spans;
  - JS exceptions;
  - source map / release / environment;
  - traceparent propagation status.
- CLI/TUI:
  - screen enter/exit;
  - selected list item/menu item;
  - command submitted;
  - background task started/completed;
  - daemon/container/agent attach/detach;
  - exit code.

Important UX rule: call this **journey/story**, not replay, unless actual
screen replay exists. Parallax should show structured facts and evidence; raw
screen contents/prompts stay redacted or absent by default.

### I. Investigations/cases: save the causal path, not just dashboards

Elastic Cases and Grafana Explore/Notebooks show a missing workflow: users need
to preserve an investigation state. Parallax should add a lightweight
**Investigation** object later:

- time window;
- selected services/edges/runs/issues;
- filters and query history;
- pinned traces/logs/spans/metrics;
- findings/notes;
- evidence gaps;
- bundle preview/export history;
- redaction report snapshot.

This is especially useful for coding agents: the UI can become a human-readable
case file, while the bundle is the machine-readable evidence subset.

### J. Telemetry quality score: show why Parallax cannot answer yet

Competitive products often hide missing instrumentation. Parallax can turn it
into a differentiator by grading evidence completeness per trace/run/service.

Score dimensions:

- trace continuity: parent/child and links resolve;
- log correlation: ERROR/WARN logs have trace/span/run ids;
- metric correlation: exemplars or time-aligned metric windows exist;
- resource identity: service/version/environment/runtime present;
- semantic quality: low-cardinality span names and standard attrs;
- redaction quality: secrets masked without destroying useful context;
- runtime coverage: process/runtime/container metrics exist;
- frontend/backend propagation: browser trace continues into backend;
- async coverage: producer/consumer links present.

UI placement:

- service catalog health card;
- trace/run story header;
- playground validation checklist;
- evidence bundle preview.

### K. Expanded query/API ideas from this pass

Add these to the future GraphQL/query backlog:

- `metricExemplars(metric, from, to, filters)` → trace/span-linked metric points.
- `fieldStats(entity, from, to, filters)` → top values, coverage, cardinality,
  types, safety status.
- `topology(mode, scope, from, to)` → one-hop/path/transitive/endpoint graphs.
- `serviceCatalog(scope)` → service identity, owners, resources, health,
  dependencies, deploys, gaps.
- `runtimeSnapshot(scope, from, to)` → runtime/process/container metrics aligned
  to traces/runs/issues.
- `investigation(id)` / `saveInvestigation(input)` → saved filters, pins,
  bundle previews, notes.
- `telemetryQuality(scope)` → completeness/gap scoring.

Add these to potential materializations:

- `metric_exemplars`: metric name/time/value/resource attrs/trace/span/run ids.
- `field_stats_minute`: entity/key/top values/cardinality/coverage/safety.
- `service_catalog_snapshots`: current identity/resources/ownership/health.
- `topology_edges_minute`: graph mode, source, target, endpoint/resource attrs,
  RED metrics, evidence quality.
- `runtime_correlations`: runtime anomaly windows linked to traces/runs/issues.
- `investigations`: Turso metadata for saved human investigation state.

### L. Playground additions from this pass

Add explicit scenarios that prove the new concepts:

- **Exemplar demo:** p99 checkout latency chart contains exemplar dots that open
  exact traces; control scenario lacks exemplars and shows lower confidence.
- **Field explorer demo:** a log/error spike where `service.version`,
  `graphql.field.name`, and `parallax.screen.name` stand out.
- **Topology mode demo:** one-hop graph suggests A-B-C, while trace-path graph
  proves only A-B and B-C occur separately; another trace proves full A→B→C.
- **Telemetry quality demo:** missing browser traceparent, missing consumer link,
  and uncorrelated log are all visible as evidence gaps.
- **Runtime demo:** Tokio blocking queue/forced-yield spike aligns with slow Rust
  service span; JVM GC pause aligns with Java GraphQL latency; container memory
  limit aligns with process crash.
- **Investigation demo:** saved case pins metric spike, BubbleUp comparison,
  slow trace, error log, runtime panel, and bundle preview.
- **Service catalog demo:** services expose version/environment/runtime/resource
  attrs; one service has new release and rising error rate; another lacks owner
  metadata and is visibly incomplete.

## Design principles

1. **Causality over pages.** Pages are entry points; the product is the graph of
   relationships between signals.
2. **Everything clickable.** Chart → time window → traces/errors/logs → span →
   bundle. Dead ends are UI bugs.
3. **Default to low-cardinality semantic names.** High-cardinality values belong
   in attributes with guardrails.
4. **Links are first-class.** Async reality needs span links, not fake parents.
5. **Show missing evidence.** An orphan span or dropped traceparent is an answer:
   instrumentation is incomplete.
6. **Human story + raw detail.** Start with a readable timeline; keep exact
   spans/logs/attributes one click away.
7. **Agent parity.** If the UI can see it, the CLI/API can reference it; if an
   agent receives it, the UI can preview it.
8. **Redaction visible by design.** Users need to know what was hidden and why.
9. **No stack expansion by default.** Use Rust, Java, TypeScript, GreptimeDB,
   Turso, TanStack, shadcn, OpenTelemetry, Sentry path only where already scoped.
10. **Local-first remains sacred.** One binary/local workflow should stay simpler
    than self-hosted Sentry and less fragmented than Grafana+Kibana.

## Suggested future execution order

1. **Inventory current UI/API gaps.** Compare this note against existing GraphQL
   schema and UI routes; list missing query surfaces only.
2. **Design the Ecosystem page first.** It changes how the whole product feels
   and reuses existing spans/service summaries.
3. **Make span links visible.** Smallest high-value trace improvement: links are
   already stored; UI should resolve/render them.
4. **Add Story timeline for trace/run.** Start with spans + events + logs;
   later add normalized UI/agent events.
5. **Add attribute compare.** Needed for Sentry/Honeycomb-grade spike diagnosis.
6. **Extend playground with execution-stack scenarios.** Create host CLI/daemon/
   container/agent simulation after UI can visualize run stories.
7. **Add evidence-gap detector.** Useful both as UI feature and instrumentation
   quality gate for the playground.
8. **Only then deepen dashboards.** Metrics are important, but causal navigation
   is the differentiator.

## Questions a future agent should answer before implementation

- Which current GraphQL resolvers already expose enough data for a service graph?
- Are span links queryable by trace id and resolvable across traces, or only
  stored as JSON on each span?
- Which GreptimeDB native OTLP tables expose span events/links and attributes
  efficiently enough for interactive UI?
- Where should low-volume story/agent metadata live: Turso rows vs derived from
  span events on read?
- How much attribute-compare can be computed live before materialization is
  required?
- Which UI graph library fits the existing TanStack/shadcn constraints without
  violating repo style?
- What exact redaction report schema should bundle preview show?
- What is the minimum execution-stack playground that proves the CLI/container/
  agent story without building a full clone of an external tool?

## Bottom line

Parallax can plausibly become the single observability application for the
operator's stack if it does not copy incumbent information architecture. Grafana,
Kibana, and Sentry split the world into metrics, logs, traces, and issues.
Parallax should split the world by **causal question**:

- What changed?
- What is failing?
- Which path did the user/run/agent take?
- Which components communicated?
- Which span/log/metric proves it?
- What evidence can safely go to an agent?

The current codebase has enough primitives to start. The next research-backed UI
move is an ecosystem graph plus story timeline, fed by OpenTelemetry semantics,
span links, run ids, and visible evidence gaps. The playground should evolve from
"polyglot shop demo" into "polyglot shop + CLI/container/agent execution lab" so
Parallax demonstrates a category Sentry/Grafana/Kibana do not cover as one
coherent product.
