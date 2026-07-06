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

## Imported brainstorm: UI/UX and telemetry-playground ideas

The source file `parallax-ui-observability-brainstorm.md` was merged here on
2026-07-06 so the main research brief is the single handoff document. Heading
levels below are shifted by one level; otherwise the brainstorm content is
preserved as source material.

## Parallax — UI/UX & Telemetry-Playground Brainstorm

> Hand-off brief. Audience: a follow-up agent that will turn these ideas into
> concrete design + implementation tasks for the Parallax core repo
> (`parallax/`) and the sample ecosystem (`parallax-telemetry-playground/`).
> This document is **brainstorm only**. It is intentionally idea-broad, not
> decision-final. It is sourced from the existing Parallax research record
> (`parallax/docs/research/...`), the playground code, the OTel specification,
> and a competitive sweep of Grafana / Kibana / Sentry / Honeycomb / Jaeger /
> Tempo / SigNoz / OpenObserve / Coroot / Maple.
>
> Scope of this pass: **UI/UX and playground extension only.** CLI redesign is
> deliberately out of scope for this brief.
>
> Authoritative stack constraint: **Java (JVM), Rust, TypeScript** only. No new
> languages or frameworks. Storage: **GreptimeDB (telemetry) + Turso (metadata),
> no fallback engines**. Frontend: **TanStack Start + shadcn/ui on Base UI +
> Recharts v3 (shadcn charts)**, served by `parallax serve` from one binary.

---

### 0. TL;DR — the one-paragraph thesis

Parallax today is a credible **Sentry-shaped issues surface + Jaeger-shaped
trace waterfall + Kibana-shaped log table + a basic dashboards builder + a
read-only SQL console**, all running on one binary over GreptimeDB. To become
the **single source of truth that replaces Grafana, Kibana and Sentry at
once**, it needs to add the three surfaces those tools own that Parallax
currently lacks — **a topology / service-map view, a real metrics / dashboard
surface with a query language, and a session / run / agent lifecycle view** —
and it needs a sample ecosystem (the playground) rich enough to demonstrate
those surfaces against **every kind of execution path a real polyglot system
actually has**: a browser click, a CLI invocation, a CLI→daemon call, a
daemon→container spawn, an agent session inside the container, a microservice
fan-out, a GraphQL resolver tree, a gRPC stream, an async message, a scheduled
job, and a deploy. The brainstorm below lists, for each of those paths, what
OTel can carry, what Parallax should render, and what the playground should
synthesize so the rendering is demonstrable end-to-end.

---

### 1. Goal, non-goals, and the definition of "win"

#### 1.1 The win condition

A single Parallax instance must let a human answer **almost any** question of
the form:

- "What happened in this request, end-to-end, across every language and every
  process boundary, with timestamps and durations?"
- "Why did this fail, and was it the same failure as that other one?"
- "Who called whom, when, with what payload shape (redacted), and how long did
  each hop take?"
- "What was the user doing in the CLI / browser / TUI at the moment this
  backend error happened? Which screen were they on, which button did they
  press, which item did they select?"
- "What was the state of CPU / memory / Tokio runtime / JVM GC / connection
  pools during this trace?"
- "Was this caused by a deploy? Which deploy? Which commit? Which work item?
  Did the previous version have this bug?"
- "Did the agent that ran inside the Docker container see the same error the
  host CLI saw? Did it share a trace? Where did the trace break?"
- "Which of these 50 GraphQL fields cost the most, and which resolver pulled
  the database row that timed out?"

"Intuitive" means: **every chart is a filter, every row is a link, every span
is a doorway to its logs / metrics / siblings / parent trace / child traces /
linked traces / deploy / commit / release / agent step** — and the navigation
graph is small enough to live in muscle memory.

#### 1.2 Non-goals for this pass

- CLI redesign. Out of scope. (Already covered by jackin' integration work.)
- New language runtimes. JVM, Rust, TypeScript only.
- Replacing the storage engine. GreptimeDB + Turso is locked.
- Building a new frontend framework. TanStack Start + shadcn stays.
- Alerting UI, RBAC, multi-user, SLO dashboards. These are explicit V1
  non-goals in `docs/research/architecture/simple-ui-v2.md` and stay deferred.
- Implementing anything. This document proposes, it does not build.

#### 1.3 What "replace Grafana, Kibana, Sentry" specifically means

| Replacee | What it owns today | What Parallax must add to truly replace it |
|---|---|---|
| **Grafana** | Dashboards, PromQL, service maps, alerting, profiles (Pyroscope), explore mode, tempo traces, loki logs | A real **metrics query surface** (PromQL-through-GreptimeDB Flow or a builder that compiles to it), a **service-map / topology** view, a **continuous-profiles** view (OTel profiles alpha → pprof/JFR), and an **explore** mode that is a free-form cross-signal browser. |
| **Kibana** | Log search (KQL/Lucene), log field facets, saved searches, ECS, APM, SIEM, Lens | A **log query DSL** with structured field predicates and facets, **saved views** per page, **field-explorer** on log records, **discover**-style left-sidebar field aggregation. |
| **Sentry** | Issue grouping, lifecycle (resolve/regress/ignore/assign), releases, source maps, breadcrumbs, session replay, performance, profiling, cron, uptime | Parallax already has issues + fingerprinting + release linkage conceptually. To match Sentry it needs: **release / deploy / commit surfaces in the UI**, **frontend session + breadcrumb + RUM surfaces** (data path is planned in `capture/frontend.md`), **regression tracking** ("this issue reappeared after deploy X"), **cron / scheduled-job monitoring**, **stack-trace symbolication** end-to-end, and **session replay** (deferred — `frontend.md` says opt-in). |

---

### 2. Current state — what already exists (so we do not re-propose it)

#### 2.1 Parallax UI today (per `ui/src/routes/`)

Twelve routes. Grouped:

- **Overview / RED / latency bands**: `/`, `/services`, `/services/$service`.
- **Issues (Sentry-shaped)**: `/issues`, `/issues/$fingerprint`.
- **Traces (Jaeger-shaped)**: `/traces`, `/traces/$traceId` with `TraceWaterfall`, span inspector with events/attributes/resource/links/logs/`db.query.text`/`exception.stacktrace`.
- **Logs (Kibana-shaped)**: `/logs` with severity filter, volume histogram, column toggle, doc viewer, SSE live tail.
- **Runs (Parallax-distinguishing)**: `/runs`, `/runs/$runId` with live stream, metric strip, evidence-bundle preview + download.
- **Dashboards**: `/dashboards`, `/dashboards/$id` (metric + agg + groupBy + chart type).
- **SQL**: `/sql` read-only GreptimeDB console.

Reusable primitives: `TraceWaterfall`, `LogsTable`, `MetricStrip`, `LiveStreamPanel`, `StatCard` / `CardSparkline` / `PillMeter` / `DeltaBadge`, `HeatCell`, `TrendChart`, `RangePicker`, `data-table` (search, filter select, sortable head, pagination). Stack parsing (`parseStacktrace`, `Frame` with `isApp`).

#### 2.2 Parallax GraphQL API today (per `crates/parallax-api/src/lib.rs`)

~30 query fields, 5 mutations, no subscriptions. Notable: `overview`, `serviceList`, `serviceRed`, `issues`, `issue`, `issueTrend`, `trace`, `logsByTrace`, `tracesByRun`, `logsByRun`, `logs`, `sql` (read-only), `run`, `dashboard`, `serviceOverview` (CPU/memory/RED), `observedRuns`, `traces`, `tracesPage`, `bundle(fingerprint?|runId?|traceId?, maxTokens?)`, `metricNames`, `services`, `metricSeries`, `histogramQuantile`, `dashboards`, `runs`. Mutations limited to `issueSetStatus`, `runStart`, `runFinish`, `dashboardSave`, `dashboardDelete`.

#### 2.3 Parallax signatures

- **Derived `error_event` + `Issue`** from span status `ERROR` + `exception.*` events and ERROR/FATAL logs (`crates/parallax-core/src/derive.rs`).
- **Fingerprint** = SHA-256 first 8 bytes over `error_type \0 normalize(message) \0 top_frame`, with regex normalizers (`<uuid>`, `<hex>`, `<n>`).
- **Evidence Bundle** (`crates/parallax-core/src/bundle.rs`): single-anchor (issue / run / trace), bounded to a token budget, redaction-lite-v1, canonical hash, hypothesis ranking (dependency_failure / slow_span / database_involved / insufficient_evidence). Projections: JSON + Markdown + clipboard snippet.
- **Causal reconstruction** (`docs/research/architecture/causal-reconstruction.md`): typed nodes + edges with strength tiers (strong / medium / weak / inferred), contradiction-first scoring.
- **Native OTLP tables** in GreptimeDB (`docs/research/decisions/native-otel-tables.md`): `opentelemetry_traces` (1 row/span, every attribute → typed column, BLOOM + 16-way partition on `trace_id`), `opentelemetry_logs` (append-mode), one logical table per metric name.
- **`parallax.run.id`** as the one canonical correlation key for runs / agent sessions / CLI invocations (`docs/research/capture/run-id-standardization.md`).

#### 2.4 Playground today (per `parallax-telemetry-playground/`)

Polyglot stack: Rust axum + tonic, Java Spring Boot 4.1 (gRPC + GraphQL + Kafka), TanStack Start web. Eight services + CLI + flagd + Redpanda + Postgres. Eighteen signal scenarios (A1–A18) and ~18 chaos scenarios (B1–B18). OTLP emitted to Rotel; dual-emitted to Sentry via SDKs. Real cross-language gRPC (Rust→Java), real Kafka round-trip (Java→Java→Rust), real GraphQL DataLoader + subscription, real OpenFeature flag evals, real canary-redaction corpus. Documented in `parallax/docs/research/validation/telemetry-playground-sample-project.md` (701 lines).

#### 2.5 What the playground does **not** have

- No Docker-in-Docker / container-spawn scenario.
- No CLI→daemon→container topology (the jackin' shape).
- No agent-session trace inside a container.
- No browser-side observability **displayed** (it only emits).
- No GraphQL field-level spans enabled by default (`otel.instrumentation.graphql.data-fetcher.enabled` is unset in `services/catalog/src/main/resources/application.yml`).
- No real database load (Postgres container exists but nothing wires it).
- No Redis, RabbitMQ, ClickHouse client spans.
- No JVM GC / class-loading / pool metrics surfaced as scenarios.
- No Tokio runtime metrics emitted by the Rust services.
- No profiling signal (pprof / JFR / async-profiler).
- No frontend RUM session, no breadcrumb chain, no rage-click beyond a single button.
- No deploy webhook ingest — A13 simulates a regression with an env var.
- No multi-tenant baggage scenarios beyond the `?tenant=` parameter.
- No metrics exemplars linked to traces from the Rust side (Java side has Micrometer counter).
- No "long trace" stress (10k+ spans) to exercise rendering.
- No trace comparison (diff two traces).
- No scheduled-job/cron UI past the CLI `cron` subcommand.
- No metrics cardinality explosion scenario (only flagd-driven chaos).
- No log structured-fields scenario (everything is plain bodies).

---

### 3. The execution archetypes a real ecosystem has

Real polyglot systems are not flat request/response graphs. They are nested
lifecycles. Parallax must explain all of the archetypes below, and the
playground must synthesize at least one of each. They are listed in roughly
increasing order of "trace-context difficulty".

#### 3.1 Archetype A — Browser interaction

A user clicks. The click is a `user_interaction` span (OTel
`UserInteraction` instrumentation). It opens a `fetch` CLIENT span that
injects `traceparent` into a same-origin request. The backend opens a SERVER
span with the same `trace_id`. A failure here is **silent on the backend
side** if CORS eats the trace header — the playground already calls this out
as the #1 footgun (`docs/research/capture/frontend.md:112-114`). A click can
be **a rage-click, a dead click, or a frustration signal** (Sentry RUM).

#### 3.2 Archetype B — Synchronous request fan-out (the HotROD pattern)

A SERVER span at an entry service fans out into N internal CLIENT→SERVER
pairs (HTTP, gRPC, GraphQL). Each fan-out branch has its own DB / cache /
downstream-call subtree. Latency is dominated by the slowest branch
(contention) or by an N+1 pattern (sequential same-target calls). Already in
the playground (A1).

#### 3.3 Archetype C — gRPC unary and streaming

Unary: one CLIENT→SERVER, `rpc.system="grpc"`, `rpc.method`,
`rpc.grpc.status_code`. Streaming (server-streaming, client-streaming, bidi):
the SERVER span **stays open for the lifetime of the stream**; each message
can be its own child span (message-level granularity) or the whole stream can
be one span with a `rpc.message.id` sequence of events. Already in playground
(A7) for server-streaming; client-streaming and bidi not yet covered.

#### 3.4 Archetype D — GraphQL operation, field-level

A GraphQL operation is naturally a tree. The root is a `graphql.request`
operation span (Spring for GraphQL emits this; Apollo emits
`graphql.execute`). Underneath, each "non-trivial" data fetcher is a
`graphql.fetch` span carrying `graphql.field.name`, `graphql.field.path`
(e.g. `products.2.reviews`), `graphql.field.type`, parent type, and the
operation kind. DataLoader batch loads coalesce multiple field spans into one
`graphql.dataloader.load` span. A bad N+1 looks like **8 sequential sibling
spans to the same DB target** under one field path; a good DataLoader batch
looks like **one batched span with `graphql.dataloader.batch.size=8`**.

Three sub-archetypes:

- **D1 — GraphQL → DB** (resolver hits Postgres / Redis directly).
- **D2 — GraphQL → gRPC → DB** (resolver is a thin GraphQL gateway over a
  gRPC service; the canonical "GraphQL-to-gRPC" pattern).
- **D3 — GraphQL → GraphQL** (a GraphQL gateway that itself queries another
  GraphQL service — subgraphs / schema stitching / federation).

#### 3.5 Archetype E — Async messaging

Producer emits a PRODUCER span, injects `traceparent` into the message
headers (Kafka: record headers; RabbitMQ: `BasicProperties.headers`; inproc:
mpsc channel is just an in-process context propagation). Consumer emits a
CONSUMER span with a **span link** back to the PRODUCER span (because the
parent might be hours old and the consumer should start a fresh trace or
extend the producer's, depending on policy). Already in playground (A3, A4,
B7, B8).

#### 3.6 Archetype F — Scheduled job / cron

A scheduler fires. The job run is a root span with `parallax.run.id` (CLI
cron) or a synthetic resource attr. It may produce child spans for each
phase. The interesting failure modes: **missed schedule**, **stuck run** (no
END span), **long-tail run** (END arrives but very late), **duplicate run**
(two traces with the same scheduled time). The playground has the success /
fail / stuck weighted bucket (`cli/src/main.rs:45-58`), but no UI surface
treats it as a cron.

#### 3.7 Archetype G — Monolith with internal subsystems

A single JVM or single Rust process that internally does many things
(orchestration, persistence, scheduling, queues). Tracing here is **all
INTERNAL spans** under one root. The failure mode is "the monolith is slow
but no external call is slow" — the culprit is lock contention, GC, Tokio
task scheduling, connection-pool wait, in-process queue depth. This is where
**runtime metrics** (Tokio `RuntimeMonitor`, JVM GC / pool / class-loading)
become essential because the spans alone don't show the contention.

#### 3.8 Archetype H — Long-lived daemon and per-session work

The jackin' shape: a host CLI (`jackin`) talks to a long-running daemon over
a local socket. The daemon spawns a Docker container, attaches a
multiplexer, and the user enters an interactive session inside the
container. Telemetry-wise:

- **CLI invocation** = one root trace, `parallax.run.id` = run id, ends when
  the CLI exits. Short.
- **Daemon** = long-lived process, **one tracer per session**, every session
  is its own trace anchored to the same `parallax.run.id`. The daemon must
  **inject** `traceparent` into the spawn-container call so the container's
  entrypoint inherits the trace context.
- **Container entrypoint** = the daemon's child trace; resource attrs include
  `container.id`, `host.id`, and the daemon's `parallax.run.id`.
- **Multiplexer attach** = a span that represents "tmux/zed/zellij session
  attached for user X on run Y".
- **Agent session inside the container** = a sub-trace rooted in the
  container's trace context, with `agent_session` → `agent_action(kind=...)`
  spans per the gen-ai semconv mapping in
  `docs/research/capture/agent-cli-tracing.md:560-570`.

The hard problem here is **propagation across the Docker boundary**: the
daemon must inject `traceparent` + `baggage` into the container's environment
or stdin or first RPC, and the in-container entrypoint must extract it and
use it as the parent context for every span it creates.

#### 3.9 Archetype I — Deploy / release / change

A deploy event lands via `POST /v1/deploys` (`integration-contract.md:90`).
Every telemetry record emitted by the new version carries
`service.version` + `vcs.ref.head.revision`. Parallax correlates errors to
the most-recent preceding deploy and can answer "did this regression appear
after deploy X?". The interesting UI failure modes: **roll-forward**,
**rollback**, **canary vs. primary**, **partial deploy** (some pods old,
some new).

#### 3.10 Archetype J — Cross-trace causal fan-in

One user action triggers a backend job that triggers a Kafka message that
triggers a consumer that calls another service that fails. None of these
share a single trace — they share a chain of **span links** plus a **baggage
correlation id** plus the **same fingerprint** at the end. Reconstructing
this chain is what Parallax's causal-reconstruction pipeline is for. The
playground should produce at least one explicit cross-trace causal chain so
the reconstruction pipeline has something non-trivial to chew on.

---

### 4. OpenTelemetry — what the standard can actually carry

This is the data-budget section: every Parallax UI surface below is
constrained by what OTel lets a service emit. It is more than people think.

#### 4.1 The five signals

1. **Traces** — a DAG of spans. Each span: name, start/end, parent, links,
   events, attributes, status (`Unset`/`Ok`/`Error`), kind
   (`INTERNAL`/`CLIENT`/`SERVER`/`PRODUCER`/`CONSUMER`), span context
   (`trace_id`, `span_id`, `trace_flags`, `trace_state`).
2. **Metrics** — counters, up-down-counters, gauges, histograms (explicit
   bucket boundaries are advisory), exponential histograms. Each data point
   may carry **exemplars** (a `trace_id`/`span_id` + value) so a metric
   spike can be jumped straight into the trace that produced it.
3. **Logs** — timestamp, severity (`severity_number` numeric, `severity_text`
   token), body (any `AnyValue`), attributes, **and the `trace_id`/`span_id`
   of the active context** so a log is joinable to a span. This is the
   single most under-used feature.
4. **Baggage** — W3C name/value pairs propagated alongside `traceparent`.
   The right channel for **business context** (tenant id, user tier,
   experiment id, feature flags) that should ride every span without each
   service re-emitting it.
5. **Profiles** (alpha) — pprof / JFR / linux_perf samples, **linkable to a
   span via `Link`** (`docs/specs/otel/profiles/`). When a span is slow,
   Parallax can show "here is the CPU profile sampled during this span".

#### 4.2 Propagation channels OTel supports

`traceparent` / `tracestate` / `baggage` ride:

- **HTTP** headers (W3C).
- **gRPC** metadata (`MetadataInjector` — already used in the playground at
  `services/checkout/src/main.rs:221-230`).
- **Kafka** record headers (Spring Kafka auto-propagates with the OTel agent).
- **RabbitMQ** `BasicProperties.headers` (manual, per
  `capture/rust-stack-instrumentation.md:31`).
- **WebSocket / SSE** — first-frame header or query param (no spec; convention).
- **Postgres / ClickHouse / Redis** — **not propagated to the DB itself**;
  instead the client wraps each call in a CLIENT span. Redis has no
  propagation channel and you don't need one.
- **Docker container spawn** — env var (`OTEL_EXPORTER_OTLP_ENDPOINT`,
  `traceparent` as an env var per the W3C env-var convention) or first RPC.
  This is what makes archetype H work.
- **In-process mpsc / channel / actor mailbox** — `Context::current()` is
  carried with the message; the consumer's first span is an INTERNAL child
  of the producer's last span.

#### 4.3 Span links — the unsung hero

Links are how OTel models **fan-in** without lying about parentage. Cases:

- Batch consumer: one CONSUMER span linked to N producer spans.
- Scatter/gather: an aggregation span linked to N fan-out spans.
- Trace restart across a trust boundary: the new root links to the prior
  root.
- Long-lived async: a job starts a new trace, links to the trace that
  enqueued it (so the chain is walkable without one 8-hour trace).

Parallax already stores links in the GreptimeDB `span_links` JSON column and
shows a `↗ N linked` badge — but the UI does not yet **walk** the link graph
(see §5.4).

#### 4.4 Events vs attributes vs logs

Rule of thumb:

- **Span attribute** — static or final-value metadata (the route, the
  status, the user id, the SQL query text).
- **Span event** — a timestamped point-in-time annotation on a span (a cache
  hit, a retry attempt, a thread-pool stall, a `exception.*` triple).
- **Log record (correlated to the span)** — anything verbose or
  high-volume (a stack frame print, a debug dump, a request body excerpt).

The playground currently logs to OTLP logs but does not surface span events
richly. Span events are the right channel for "retry attempt #2 at
+1.2s with error `connection reset`", "cache miss for key X", "feature flag
`catalogPromo` evaluated to `false`".

#### 4.5 Semantic conventions worth committing to

The OTel semconv registry has Stable + Development tiers. Parallax should
mandate the Stable set and **opt into** Development where it adds value:

- **HTTP Stable** — `http.request.method`, `url.path`, `url.query`,
  `http.response.status_code`, `http.route`, `network.protocol.version`,
  `server.address`, `server.port`.
- **RPC Stable** — `rpc.system`, `rpc.service`, `rpc.method`,
  `rpc.grpc.status_code`, `rpc.message.type` (SENT/RECEIVED),
  `rpc.message.id`, `rpc.message.compressed_size`.
- **Database Stable** — `db.system.name`, `db.namespace`,
  `db.collection.name`, `db.operation.name`, `db.query.text` (opt-in),
  `db.query.summary`, `db.response.status_code`, `error.type`,
  `network.peer.address`/`port`. Plus the **stable metrics**
  `db.client.operation.duration` and the connection-pool metrics
  (`db.client.connection.count` / `.pending_requests` / `.timeouts` /
  `.wait_time` / `.use_time` / `.create_time` / `.idle.max` /
  `.idle.min` / `.max`).
- **Messaging (Development)** — `messaging.system`,
  `messaging.destination.name`, `messaging.operation.name`,
  `messaging.message.id`, `messaging.message.conversation_id`,
  `messaging.batch.message_count`.
- **GraphQL (Development)** — `graphql.operation.name`,
  `graphql.operation.type`, `graphql.document`,
  `graphql.field.name`, `graphql.field.path`, `graphql.field.type`.
- **Feature flags (Stable)** — `feature_flag.context.id`,
  `feature_flag.provider_name`, `feature_flag.key`, `feature_flag.variant`.
- **Gen-AI (Development, semconv-genai repo)** — `gen_ai.operation.name`,
  `gen_ai.request.model`, `gen_ai.usage.input_tokens`,
  `gen_ai.usage.output_tokens`, `gen_ai.tool.name` — the basis for the
  agent-session view.
- **System / process (Development)** — `process.cpu.utilization`,
  `process.memory.usage`, `process.memory.utilization`,
  `tokio.runtime.alive_tasks`, `tokio.runtime.worker_count`,
  `jvm.gc.time`, `jvm.threads.count` etc. — the basis for runtime panels.
- **Deployment (Development)** — `deployment.environment.name`,
  `deployment.id`, `deployment.name`, `deployment.status`.
- **VCS (Development)** — `vcs.ref.head.revision`,
  `vcs.ref.head.name`, `vcs.repository.url.full`.

#### 4.6 The resource-attribute correlation contract

Parallax's existing contract (`docs/research/architecture/integration-contract.md`)
is correct and minimal. Restating it because it is the join key for every UI
surface below:

| Attribute | Why it exists | Which view uses it |
|---|---|---|
| `service.name` | Anchor of the per-service view | `/services`, service map |
| `service.version` | Release linkage | `/releases`, regression detection |
| `deployment.environment.name` | Env scoping | env filter everywhere |
| `vcs.ref.head.revision` | Deployed commit | `/deploys`, `issue.affectedReleases` |
| `vcs.repository.url.full` | Repo targeting for fixer | evidence bundle, MCP |
| `parallax.run.id` | Run / agent session correlation | `/runs`, run timeline |
| `host.id`, `container.id` | Topology | service map, host view |
| `telemetry.sdk.language` / `.version` | "Why does this Rust span look different from this Java span?" | span inspector |
| `process.pid`, `host.name` | Process identity | runtime panel |

---

### 5. Parallax UI/UX extensions — the brainstorm

This is the core of the document. Each subsection is one new surface, ordered
by leverage. For each: **the gap**, **what it should show**, **data source**,
**playground scenario needed**.

#### 5.1 Investigation console (the unifying shell)

**Gap.** Today every page is independent. The win condition in §1.1 requires
that from any artefact (a span, a log, an issue, a metric spike, a deploy, a
run, an agent step) the user can navigate to any other artefact in 2-3
clicks.

**Shape.** A persistent right-hand **investigation panel** that the user
"pins" artefacts to. Pinning a trace pins its trace_id, run_id (if any),
service set, time window. Every other page respects the pinned context (the
logs page pre-filters to the trace's window + services; the metrics page
pre-filters to the trace's window + services; the issues page pre-filters to
the same window + services). Each pin is a chip at the top with a remove
button. Replaces the mental model of "I have to copy a trace_id from one tab
to another".

**Data source.** Existing queries, plus a new `pinnedContext` GraphQL input
type so the panel state is a single source of truth.

**Playground need.** None — this is a shell feature, exercised by every
existing scenario.

#### 5.2 Service map / topology (the missing Grafana/Tempo surface)

**Gap.** Listed as ❌ for Parallax in `observability-feature-matrix.md:124`.
Every competitor except Gonzo has one. A topology view is the fastest way to
answer "who depends on whom, and which edge is red right now?".

**Shape.** A force-directed graph (or a layered DAG — switchable). Nodes are
services, sized by request rate, colored by error rate (HeatCell scale),
bordered by p95 latency band. Edges are directed (caller → callee), labelled
with rate / error rate / p95, colored red if degraded vs. baseline. The
graph is computed by aggregating CLIENT→SERVER span pairs over the selected
window. Clicking a node enters `/services/$service`. Clicking an edge opens
a side panel listing the operations on that edge (RPC method / HTTP route /
GraphQL operation), with a jump to the slowest / erroring trace per
operation.

**Sub-modes.**

- **Service map** — service-to-service (current playground's natural shape).
- **Operation map** — for one service, the tree of operations and their
  downstream calls (a per-service zoom-in).
- **Container / host map** — when `container.id` / `host.id` are present,
  show the deployment topology (which containers are behind which service,
  how many replicas, where the load is landing). This is the surface that
  makes archetype H visible.
- **GraphQL field map** — for a GraphQL service, the field tree with
  per-field rate / latency / error, so a user can see "the `reviews` field
  is the slow part of `products`".

**Data source.** New query `serviceMap(from, to, env?)` returning nodes +
edges with aggregated RED metrics. GreptimeDB can compute this with a Flow
or a SQL GROUP-BY over `(service.name, peer.service.name)` from
`http.*` / `rpc.*` spans. The peer service is read from the SERVER-side span
matched by `(trace_id, span_id) = parent(remote parent)` of the CLIENT span.

**Playground need.** Current A1 is enough for the basic map. To exercise
the **container/host** variant and the **GraphQL field** variant we need
(archetype H) and a richer catalog (D1/D2 in §3.4).

#### 5.3 Trace waterfall extensions

**Gap.** `TraceWaterfall` is good for <500 spans. Real HotROD-style N+1
traces, GraphQL operation traces with hundreds of field spans, and long
streaming traces blow it up.

**Sub-surfaces.**

- **Flame view** (collapsed by default above 200 spans) — group siblings
  that call the same operation into a single aggregate row ("8×
  `POST /inventory/reserve` 3.2-5.1ms each"); expand on click.
- **Critical-path highlight** — compute the longest chain through the trace
  (dominant-latency path) and render it as a thick stroke. This is the
  Honeycomb/Tempo pattern.
- **Span-group color-by** — color by `service.name` (default),
  `otel.kind`, `status.code`, error, `db.system.name`,
  `rpc.system`, `messaging.system`, or any user-picked attribute. This is
  the single highest-leverage UI change.
- **Mini-map / brush** — for long traces, a 60px minimap above the
  waterfall with a draggable window.
- **Clock-skew tolerance** — B18 in the playground already produces
  overlapping/negative span timing; the renderer must clamp or warn, not
  crash.
- **Side-by-side trace comparison** — pick two traces (e.g. a fast one and
  a slow one with the same operation name), the waterfall renders them in
  two columns with the diff in duration per span highlighted.
- **Virtualized rendering** — current 500-row window is fine; a virtual
  list with 50k spans must still scroll smoothly.

**Data source.** Existing `trace(traceId)`; needs a new `traceCompare(a, b)`
for the diff view, and a `traceCriticalPath(traceId)` resolver (computable
server-side or client-side; server-side is cheaper).

**Playground need.** A new A19 scenario "long trace" — a synthetic
deep-fan-out (depth 6, fan-out 5, 10k spans) using the existing services
behind a `?deep=6&fan=5` flag.

#### 5.4 Linked-traces graph (the cross-trace walker)

**Gap.** Span links exist but the UI only shows a `↗ N linked` badge.
Archetype E and J require walking link chains across multiple traces.

**Shape.** From a span with links, a button "Open linked traces graph"
opens a small modal/page that shows the current trace in the center and
each linked trace as a card with its own root service, duration, status,
and a thumbnail waterfall. The graph is recursive (linked traces can have
links). Edges are labelled with the link attributes. Clicking a card swaps
the center trace.

**Data source.** New query `linkedTraces(traceId, depth=2)` returning
trace summaries + edge metadata. Implemented by reading `span_links` from
GreptimeDB and joining into `opentelemetry_traces` by `trace_id`.

**Playground need.** A new A20 scenario "cross-trace causal chain" — the
checkout service fires an async job (orders → Kafka → fulfillment), the
job fails, the failure is captured as a separate trace linked back to the
checkout trace. Today A4 goes one hop; A20 goes three hops with an explicit
causal chain that can only be reconstructed via links + baggage.

#### 5.5 Run timeline (the Parallax-distinguishing surface)

**Gap.** `/runs/$runId` exists but renders a flat list. Archetype F, H, and
the agent-session story require a **timeline**.

**Shape.** A horizontal swim-lane timeline. Lanes (top-to-bottom):

1. **Process lifecycle** — run start → exit, colored by status, with exit
   code.
2. **CLI / agent phase spans** — `parse_args`, `load_config`,
   `execute_subcommand`, `spawn_process`, `exit`. From
   `agent-cli-tracing.md:342-353`.
3. **Backend calls** — outbound HTTP/gRPC from this run, one row per call
   with a colored chip for the target service.
4. **Errors** — red diamonds on the relevant lane.
5. **Logs** — small severity-colored ticks (drill into LogsTable on click).
6. **Metrics** — `MetricStrip` of CPU / memory / Tokio tasks / JVM threads
   for the duration of the run.
7. **Agent steps** (if `parallax.run.id` corresponds to an agent session) —
   `agent_session` → `agent_action(kind=context_load | model_call |
   tool_call | mcp_tool_call | shell_command | file_read | file_edit |
   permission_decision | state_verification | validation | outcome)`. Each
   step is a chip; clicking it shows the prompt excerpt, tool I/O, and the
   resulting file diff if any.

For archetype H (the jackin' shape), the run timeline should render the
**container spawn as a sub-timeline nested inside the daemon's timeline** —
the same run_id, the container's resource attrs as a "nested process"
indicator. This is the single most distinctive Parallax surface and it
does not exist anywhere else.

**Data source.** Existing `run(runId)` + `tracesByRun` + `logsByRun` +
new `agentSteps(runId)` resolver (reads Turso per
`agent-cli-tracing.md:263-320`). New `metricSeries(name, runId=...)` already
supports run-scoping.

**Playground need.** A new archetype-H scenario in the playground — even a
toy version: a `playground daemon` long-lived process + a `playground enter`
subcommand that "spawns" a child `playground agent` (just a subprocess, not
real Docker) which emits a nested trace carrying the same
`parallax.run.id`. This single scenario is what makes the run-timeline view
demoable.

#### 5.6 Issues surface extensions (Sentry parity)

**Gap.** Issues + fingerprinting + status lifecycle exist. Missing: release
linkage in the UI, regression tracking, exception grouping controls,
breadcrumbs, occurrence sparkline drill-down.

**Sub-surfaces.**

- **Release & deploy attribution** — every Issue has a "First seen after
  deploy X (commit Y)" panel. This requires the deploy-event ingest and a
  `/deploys` page (also missing). When `service.version` + commit SHA are
  stamped, Parallax can compute "issue first seen within 1h of deploy X" →
  strong causal edge.
- **Regression badge** — when a resolved issue re-appears after a new
  deploy, mark it `REGRESSED` and link both deploys.
- **Grouping controls** — let the user pick the fingerprint key stack-frame
  depth (top-1 vs top-3 vs full), merge two issues, split one issue by an
  attribute. Sentry has this; Parallax should too.
- **Breadcrumbs** — for backend issues, breadcrumbs are the parent trace's
  spans in chronological order. For frontend issues, breadcrumbs are the
  `user_step` events (RUM). Already planned in `capture/frontend.md:154`.
- **Occurrence trend with brush-and-drill** — the issue detail's trend bar
  chart should let the user brush a spike and jump to the traces of that
  bucket (already partially there; needs the cross-navigation to actually
  run a trace query scoped to the bucket window + fingerprint).
- **Suspect commits / blame** — when `vcs.ref.head.revision` is set,
  optionally blame the relevant code path. This is Sentry's "Suspect
  Commits". Out-of-scope for V1 but a natural follow-up.

**Data source.** New `deploys` query + `Issue.affectedReleases` (already in
the spec sketch but unimplemented). Existing `issueTrend` is sufficient for
the brush.

**Playground need.** A13 simulates a regression via an env var. To exercise
the real deploy-event path, add an A21 scenario that POSTs to
`/v1/deploys` between two `?release=` tagged runs, so the deploy marker is
real data not an env var.

#### 5.7 Logs surface extensions (Kibana parity)

**Gap.** `/logs` has service/severity/query/cols filters and a volume
histogram. Missing: structured-field predicates, facets, saved views,
field explorer, live-tail virtualization.

**Sub-surfaces.**

- **Structured-field query DSL** — `service:checkout AND severity>=ERROR
  AND db.system.name:postgresql AND trace_id:<id>`. Implemented as a parser
  that compiles to a GreptimeDB WHERE clause over the
  `opentelemetry_logs` attributes JSON. (Native shortfall per
  `decisions/native-otel-tables.md:49-61`: logs need a body FULLTEXT index
  and a `trace_id` INVERTED INDEX; both are ALTER-TABLE additions on the
  Parallax side.)
- **Field explorer** — Kibana's left sidebar: for the current result set,
  show every attribute that appears, with top-N values and counts. Clicking
  a value adds it as a filter. This is the dominant Kibana workflow.
- **Faceted facets** — service, severity, host, container, error.type,
  http.route, rpc.method — each as a top-N facet.
- **Saved views** — name a filter set, get a URL. localStorage is enough
  for V1.
- **Live-tail virtualization** — current SSE panel prepends; for >5k events
  use a windowed virtual list (`@tanstack/react-virtual`).
- **Log-to-trace jump** — every log row with a `trace_id` gets a chip;
  clicking jumps to `/traces/$traceId` scoped to the log's timestamp.
- **Log redaction state badge** — every record that was scrubbed shows what
  was scrubbed (count per policy bucket) so the user trusts the data.

**Data source.** Existing `logs(...)` + new `logFacets(query, fields[])`
resolver that returns top-N values per field. New saved-view table in Turso
(planned).

**Playground need.** A9 is "structured logging during a request" but the
playground's Rust services log plain bodies. Add an A9b that emits
structured fields (`tenant.id`, `cart.id`, `request.size_bytes`,
`db.statement_count`) so the field explorer has something to chew on.

#### 5.8 Metrics surface extensions (Grafana parity)

**Gap.** Dashboards builder is metric + agg + groupBy + chart type. Missing:
metrics query language, math across series, alerting, exemplars, templates.

**Sub-surfaces.**

- **PromQL / SQL editor** — a "code" mode alongside the visual builder that
  accepts PromQL (GreptimeDB supports a PromQL subset natively) or SQL.
  The visual builder compiles to one of these. Reuse the `/sql` page
  primitives.
- **Exemplar jump** — every histogram bucket can be annotated with
  exemplars (trace_id, value). Render exemplars as dots on the chart;
  clicking jumps to the trace. This is Grafana's killer feature for
  metrics↔trace cross-navigation. The Java side has Micrometer exemplars
  (`services/catalog/src/main/java/dev/tailrocks/catalog/CatalogApplication.java:51-57`);
  the Rust side needs to add them.
- **Multi-metric math** — `rate(http.server.request.duration[5m]) /
  rate(http.server.request.duration{status=5xx}[5m])` for error ratio.
- **Template variables** — pick a service at the top, every panel
  re-resolves. Out-of-scope for V1 but worth designing for.
- **Anomaly overlay** — compute a simple baseline ( EWMA or z-score) and
  shade regions that deviate. No need for ML; the GreptimeDB `udf` host can
  do this server-side.
- **SLO / burn-rate** — let a user define "p99 < 200ms for checkout" and
  show the error budget burn-down. Listed as ❌ in the feature matrix but
  cheap to demo against the existing histogram.
- **Continuous-profile overlay** — when OTel profiles land, overlay profile
  samples on the metric chart so a CPU spike can be jumped straight into
  the flamegraph (Pyroscope pattern).

**Data source.** Existing `metricSeries`, `histogramQuantile`; new
`metricExemplars(name, from, to, ...)`, `sloBurn(sloId, from, to)`. PromQL
is served by GreptimeDB's PromQL frontend directly through `/sql`-like
endpoint.

**Playground need.** A Rust-side exemplars scenario (the Java side already
has one). Add a `playground_telemetry` helper that records a counter with
an explicit exemplar (`opentelemetry::metrics::Counter::add` with
`Context::current()`).

#### 5.9 Continuous profiling surface (Pyroscope / Parquet-Profiles)

**Gap.** Listed as ❌ in the feature matrix. OTel Profiles is **alpha** but
the data model is already linkable to spans. Sentry, Coroot, and SigNoz all
have it.

**Shape.** A **flamegraph** view (top-down or icicle) over profile samples
filterable by `service.name`, `parallax.run.id`, time window, and —
critically — **by trace/span**. "Show me the CPU profile sampled during
trace X" is the killer query. This requires the profile samples to carry a
`Link{trace_id, span_id}` per the OTel profiles spec.

**Sub-modes.**

- CPU flame (pprof for Rust, JFR for Java).
- Allocation flame (JFR per-allocation; Rust alloc-counter via
  `tracing-alloc` or `pprof-rs`).
- Lock-contention flame (JFR sync-statistics; Tokio task-poll durations
  from `tokio-metrics`).
- Goroutine / thread / virtual-thread timeline (JFR thread-states; Tokio
  task counts).

**Data source.** New GreptimeDB table `opentelemetry_profiles` (alpha spec
format). New GraphQL `profiles(service, from, to, traceId?)` and
`flamegraph(profileId, groupBy=function|file|module)` returning a folded
tree.

**Playground need.** A17 is the slot. Add: Rust services run `pprof-rs`
profiling under a feature flag; Java services run async-profiler / JFR
continuously at 100Hz; both emit OTel profiles via the OTLP profiles
exporter (alpha) or, until that's stable, via the pprof exporter with
Parallax-side conversion.

#### 5.10 Runtime metrics panels (Tokio + JVM)

**Gap.** `/services/$service` shows CPU/memory but not the deep runtime
internals that explain archetype G.

**Shape.** Per-service runtime panel with sub-tabs:

- **CPU** — `process.cpu.utilization`, `process.cpu.time` (user/sys).
- **Memory** — `process.memory.usage`, `process.memory.utilization`,
  RSS vs virtual, JVM heap vs non-heap vs metaspace, JVM GC time per pool.
- **Tokio (Rust)** — from `tokio-metrics` `RuntimeMonitor`:
  `workers_count`, `alive_tasks`, `blocking_pool_depth`,
  `budget_forced_yield_count`, `io_driver_ready_count`,
  `poll_count_histogram`, `schedule_wait_duration`, `task.polls`.
  Plus `TaskMonitor` per critical task: `instrumented_count`,
  `dropped_count`, `first_poll_delay`, `total_poll_duration`,
  `total_schedule_duration`, `total_idle_duration`, `mean_poll_duration`.
  These are the metrics that answer "did my runtime deadlock? why?".
- **JVM (Java)** — from the OTel JVM instrumentation: `jvm.gc.time`,
  `jvm.gc.count`, `jvm.threads.count`, `jvm.memory.used`,
  `jvm.class.loaded`, `jvm.cpu.time`, `jvm.buffer.pool.*`.
- **Connection pools** — the stable `db.client.connection.*` metric family
  (count, idle, used, pending, timeouts, wait_time, use_time, create_time).
  These directly diagnose B10.
- **HTTP / RPC client pools** — `http.client.connection.*` (Development)
  for keep-alive pools.

Every runtime metric should be **joinable to a trace via exemplars**, so a
GC-spike panel can be jumped into the trace that was running during the
spike.

**Data source.** New well-known metric names in the API; most are already
emitted by `opentelemetry-system-resources` (Rust) and the OTel JVM agent
(Java). The Tokio-specific ones require wiring `tokio-metrics` → OTel
gauges (already documented in
`capture/rust-stack-instrumentation.md:36`).

**Playground need.** A22 scenario "Rust runtime under load" — drive the
Rust checkout with loadgen while recording Tokio metrics; B5 already
covers JVM GC; add a B23 "Tokio runtime starvation" — saturate the runtime
with CPU-bound tasks and watch `budget_forced_yield_count` climb.

#### 5.11 Frontend / RUM surface (Sentry-parity, data-path is planned)

**Gap.** `capture/frontend.md` defines the data path (OTel JS + Sentry
browser envelopes) but the UI has no surface to render it.

**Shape.**

- **Sessions list** — one row per browser session (`frontend_session`),
  with start/end, route views, error count, web vitals (LCP/CLS/INP),
  rage-click count, replay availability.
- **Session detail timeline** — lanes for route changes (`route_view`),
  fetch calls, user interactions (`user_step` breadcrumbs), web vitals,
  long tasks, errors (`frontend_error`). This is the surface that
  visualizes "the user was on the cart page, clicked checkout three times,
  got a 500, then a JS error". Replay is a deferred chip.
- **Web vitals dashboard** — per-route LCP/CLS/INP histograms over time.
- **Frontend error → backend trace correlation** — the
  `frontend_error_caused_by_backend` edge (planned in `frontend.md:164`)
  rendered as a link from a frontend_error to the backend trace it
  triggered.

**Data source.** Planned nodes in `frontend.md:154-160`. New GraphQL
`frontendSessions(...)`, `frontendSession(id)`, `frontendErrors(...)`.

**Playground need.** A5 exists but only as a single button. Expand the web
app to a small 3-route SPA (catalog → cart → checkout) with several
interactive affordances that produce rich breadcrumbs: navigate, search,
add-to-cart, remove-from-cart, checkout. Plus a scenario that drives it
with Playwright.

#### 5.12 GraphQL operation explorer (the field-tree view)

**Gap.** GraphQL is treated as just HTTP. The killer GraphQL view is the
field tree.

**Shape.** For a `graphql.request` trace, render the **resolver tree**
mirroring the operation's selection set: root operation (`Query.products`)
at the top, each field as a child node (`products[].id`, `products[].name`,
`products[].reviews`), each field node carrying its latency, error rate,
and the downstream calls it made (DB / gRPC / Redis). DataLoader-batched
fields show as a single coalesced node with `batch.size` and the constituent
field paths it covered. This is what Apollo Studio shows; SigNoz has a
weaker version.

**Sub-features.**

- **Persisted-query awareness** — when `graphql.document` is a hash,
  resolve it via a persisted-query registry (lookup table).
- **Operation cost** — computed from the schema and the actual field tree.
- **Field-level error** — render `graphql.error.path` as a red badge on the
  offending field node.
- **Jump to upstream operation** — for D3 (GraphQL → GraphQL), a button
  on a `graphql.fetch` CLIENT span that opens the downstream service's
  `graphql.request` SERVER span.

**Data source.** Existing trace data — but only if the OTel Java agent is
configured with `otel.instrumentation.graphql.data-fetcher.enabled=true`
and `create_or_add_link` between operation and data-fetcher spans. The
playground's catalog service already has the right deps; just needs the
flag flipped on and an operation-driven scenario.

**Playground need.** A6b — drive the catalog GraphQL endpoint with a query
that exercises DataLoader (`products { reviews }` × N), N+1 (without
DataLoader), and a subscription. Plus D2 — a GraphQL field in `catalog`
that proxies to the gRPC `pricing` service (this is the canonical
GraphQL-to-gRPC pattern, currently absent).

#### 5.13 gRPC streaming explorer

**Gap.** gRPC streaming is treated as one long span. For bidi / long
streams the per-message cadence is the point.

**Shape.** For a long-lived SERVER span on a streaming RPC, render a
**message timeline** underneath the span: SENT and RECEIVED events as
arrows, each with `rpc.message.id`, `rpc.message.compressed_size`, and a
drill-into the per-message log/error. Useful for A7's `QuoteStream` and
for the GraphQL subscription over WebSocket (catalog A7 sub).

**Data source.** The OTel `rpc.message.*` events are emitted by the OTel
Java agent and by `tonic-tracing-opentelemetry` on the Rust side. Today
the playground's `services/checkout` does manual metadata injection but
does not yet emit per-message events.

**Playground need.** A7b — extend `pricing` `quote_stream` to emit a
per-message event with `rpc.message.id` and `rpc.message.compressed_size`;
drive it from a checkout that streams 50 messages and watches one mid-stream
message fail.

#### 5.14 Causal graph view (the evidence-graph renderer)

**Gap.** `causal-reconstruction.md` describes typed nodes + edges with
strength tiers, but there is no UI to render the resulting graph.

**Shape.** For any anchor (issue, run, trace, deploy), a "Causal graph"
tab renders the reconstructed evidence graph: nodes are coloured by type
(error / span / log / metric-window / release / deploy / code-change /
runtime-resource / agent-action / CI-test), edges are coloured by strength
(strong = solid, medium = dashed, weak = dotted, inferred = ghosted). The
anchor is centered; the user can expand nodes outward. A side panel lists
the hypotheses with their supporting + contradicting edges and the
missing-evidence items. This is the natural renderer for the evidence
bundle's node/edge catalog from `evidence-bundle-schema.md`.

**Data source.** New `causalGraph(anchor)` returning the assembled graph
per the pipeline in `causal-reconstruction.md:239-250`.

**Playground need.** A20 (cross-trace causal chain) plus A13 (deploy
regression) — together they produce a graph with strong edges (same trace,
span links) and medium edges (deploy-precedes-regression, depends-on).

#### 5.15 Evidence-bundle extensions

**Gap.** Bundle is single-anchor, redaction-lite, with hypothesis
confidence as a string tier. Spec is much richer.

**Extensions.**

- **Multi-anchor bundles** — anchor by `(issue, deploy)` or
  `(trace, release)` to assemble bundles that cross the failure/deploys
  boundary.
- **Typed node/edge projection** — the JSON projection should expose the
  typed nodes/edges from §5.14, not the flat `{logs:[], metric_windows:[],
  hypotheses:[]}` shape today. Backwards-compatible via schema versioning.
- **MCP `outputSchema`** — every bundle ships with a JSON Schema so an MCP
  consumer can validate it. Also unlocks structured agent consumption.
- **Bundle diff** — diff two bundles (e.g. before and after a fix attempt)
  to see what changed in the evidence.
- **Bundle redaction upgrade** — replace `redaction-lite-v1` with the
  full default-deny policy (`redaction.md:98-115`), retroactive purge,
  projection manifest hashing, and a redaction-report viewer in the UI.

**Data source.** `bundle.rs:15-37` upgrade. New `bundleDiff(a, b)`.

**Playground need.** A18 canary corpus already exists; expand to cover
each new redaction policy bucket.

#### 5.16 Global cross-cutting UX

- **Time-range picker as a global** — currently per-page; should be in the
  shell, synced across pages (Sentry/Grafana pattern).
- **Environment filter as a global** — `deployment.environment.name`.
- **Release filter** — `service.version` + commit SHA. New `/releases`
  page listing releases with health, error-rate-delta vs previous, deploy
  events.
- **Service filter** — global multi-select of `service.name`.
- **Saved investigations** — name a pinned-context state, share by URL.
- **Command palette** (⌘K) — jump to any trace_id, run_id, issue
  fingerprint, service, release — like Sentry/Grafana.
- **Keyboard navigation** on every list — j/k to move, enter to open.
- **URL state on every filter** — every chip is in the URL; shareable.

#### 5.17 Out-of-scope-for-V1 but worth designing for

- Alerting UI and notification channels.
- SLO dashboards (the metric surface can demo it; the alerting is what is
  deferred).
- Auth / RBAC / API tokens (V1 is local single-user).
- Dashboard templating variables.
- Saved searches cross-service.
- Multi-tenancy.
- Session replay playback (the data path is opt-in; the player is
  significant work).

---

### 6. Playground extension brainstorm — scenarios to add

The playground is the demo material. Every UI surface in §5 needs at least
one scenario that produces the data shape it renders. The current A1–A18 +
B1–B18 set is a strong base. Additions:

#### 6.1 New signal scenarios (A-extensions)

| # | Scenario | What it exercises | Which UI surface it demoes |
|---|---|---|---|
| **A2b** | Rust-side metric exemplar | a Rust service records `http.server.request.duration` with an explicit exemplar pointing at the current `Context` | §5.8 metrics exemplar jump |
| **A6b** | GraphQL field-level trace | flip on `data-fetcher.enabled`, drive `products { id name reviews { text stars } }` over N products — shows the field tree with and without DataLoader | §5.12 GraphQL explorer |
| **A7b** | gRPC per-message events | extend `quote_stream` to emit `rpc.message.id` + `rpc.message.compressed_size` SENT/RECEIVED events; one message mid-stream fails | §5.13 streaming explorer |
| **A9b** | Structured-field logs | Rust services emit JSON log bodies with `tenant.id`, `cart.id`, `db.statement_count`, `request.size_bytes` | §5.7 field explorer |
| **A10b** | Baggage-driven branch | propagate `tenant.id` + `user.tier` via baggage; downstream services branch on it; UI shows the baggage on every span | §5.16 + investigation console |
| **A17b** | Continuous profiling (Rust + JVM) | pprof-rs on Rust; JFR / async-profiler on Java; emit OTel profiles alpha with span `Link`s | §5.9 flamegraph |
| **A19** | Long / wide trace | synthetic deep fan-out — depth 6, fan-out 5, ~10k spans — using the existing services behind `?deep=6&fan=5` | §5.3 flame + minimap |
| **A20** | Cross-trace causal chain | checkout → orders (Kafka) → fulfillment → notifications, with the message-handling failure as a separate trace **linked** back; same `correlation.id` in baggage | §5.4 linked-traces graph + §5.14 causal graph |
| **A21** | Real deploy-event regression | POST `/v1/deploys` between two `?release=`-tagged checkout runs; the second run fails; the issue is attributed to the deploy | §5.6 release/regression UI + `/deploys` |
| **A22** | Tokio runtime under load | drive checkout with loadgen while emitting `tokio.runtime.*` + `tokio.task.*` metrics | §5.10 runtime panel |
| **A23** | GraphQL → gRPC gateway | catalog gains a `Quote` field that proxies to the `pricing` gRPC service (the D2 pattern) | §5.12 GraphQL explorer with downstream gRPC |
| **A24** | GraphQL → GraphQL (federation subgraph) | a second tiny GraphQL service that the catalog queries for "inventory status" — D3 pattern | §5.12 GraphQL explorer with upstream/downstream operations |
| **A25** | Real DB spans (Postgres) | wire inventory + catalog to the existing Postgres container; emit `db.client.operation.duration` + `db.query.text` + connection-pool metrics | §5.7 log redaction + §5.10 pools + §5.3 trace |
| **A26** | Redis cache spans | add a Redis container; recommendation service caches; emit `db.system.name="redis"` spans + `db.client.operation.duration` for cache hits/misses | §5.3 trace + §5.10 |
| **A27** | Real Docker-spawn nested run (archetype H) | a `playground daemon` long-lived process + `playground enter <session>` that spawns a child process carrying the same `parallax.run.id` and inheriting `traceparent` from env; the child emits a nested trace | §5.5 run timeline (the signature Parallax surface) |
| **A28** | Frontend RUM session | expand web to a 3-route SPA; drive with Playwright; emit `route_view`, `user_step`, `frontend_error`, web vitals | §5.11 RUM surface |

#### 6.2 New chaos scenarios (B-extensions)

| # | Failure | Signals tested |
|---|---|---|
| **B19** | Tokio runtime starvation | saturate runtime with CPU-bound tasks; `budget_forced_yield_count` climbs; latency degrades; profiles show one task hogging polls |
| **B20** | Connection-pool exhaustion | Postgres pool size 2; loadgen drives 20 concurrent requests; `db.client.connection.pending_requests` + `.timeouts` rise |
| **B21** | JVM GC storm | trigger System.gc() in a loop on payment; watch `jvm.gc.time` spike, latency follow, profiles show GC frames |
| **B22** | Cache stampede | Redis cold-cache + loadgen thundering herd; recommendation spans pile up; show cache-miss vs hit ratio |
| **B23** | Slow GraphQL field | one resolver in catalog sleeps 500ms; field tree immediately shows `reviews` as the red node |
| **B24** | GraphQL persisted-query mismatch | client sends a hash that doesn't match the server's registry; show `graphql.error` |
| **B25** | gRPC stream client-disconnect | checkout cancels `quote_stream` mid-flight; show the CANCELLED status and the half-open server span |
| **B26** | Trace-context sampling drop | set sampler to 10%; show that some linked traces are missing and the causal graph reports `missing_evidence: sampled_out_trace` |
| **B27** | Clock-skew between two services (B18+) | extend to make one service's spans appear "before" the parent — exercise the renderer's clamping |
| **B28** | Frontend frustration (rage-click + dead click + ESC) | drive web with Playwright; produce rage-click cluster + a dead click + an ESC-to-close that triggers a JS error |
| **B29** | Cross-language propagation break | checkout → catalog, but the `traceparent` header is stripped by a misconfigured proxy; show the broken-link edge in the service map and the `missing_backend_continuation` in causal graph |
| **B30** | Container spawn timeout | archetype-H daemon's child process fails to start within the deadline; show the `agent_session` with `outcome=timeout` |

#### 6.3 Topology extensions to the playground

- **`services/inventory` + `services/catalog` wired to Postgres** (A25).
- **New `services/cache` (Redis)** — Rust fred-based cache used by
  recommendation (A26).
- **New `services/gateway` (Rust)** — a GraphQL server in Rust (Juniper or
  async-graphql) that fronts `pricing` gRPC and `catalog` GraphQL. Exercises
  the GraphQL-in-Rust path and gives D2 in Rust, not just Java.
- **`playground daemon`** — a long-lived Rust process listening on a Unix
  socket; `playground enter` connects and spawns a worker child. This is
  archetype-H in miniature.
- **`profile-collector`** — extend the OTLP fan-out to accept the OTel
  profiles signal (or a pprof/JFR HTTP endpoint) so profile data has
  somewhere to land.
- **`web` expanded to 3 routes** with a router that emits `route_view`
  spans.

#### 6.4 Telemetry-library upgrades (`libs/playground-telemetry`)

Concrete gaps today (per the inventory):

- Add the W3C `BaggagePropagator` programmatically (today only via env).
- Add `ParentBased(TraceIdRatioBased)` sampler so B26 is demoable.
- Add `tokio-metrics` `RuntimeMonitor` → OTel gauges for all Rust services
  (A22).
- Add `opentelemetry-semantic-conventions` constants for stable HTTP / RPC /
  DB attrs so the playground's spans are spec-accurate.
- Add a `redact_then_emit` log layer that demonstrates ingest-time redaction
  (currently Parallax defers redaction to bundle-build per
  `decisions/native-otel-tables.md:69-71`).
- Add a Rust metric-exemplar helper (A2b).
- Add `parallax.run.id` stamping inside `init()` (today it relies on env
  injection from `parallax run start`).

#### 6.5 Java-instrumentation upgrades

- Flip `otel.instrumentation.graphql.data-fetcher.enabled=true` in
  `services/catalog/src/main/resources/application.yml` (A6b).
- Enable `otel.instrumentation.graphql.data-fetcher.create_or_add_link=true`
  so each field span links to the operation span (Apollo pattern).
- Add `spring-boot-starter-actuator` metrics for the connection pool
  (HikariCP) — `hikaricp.connections.*` map to the OTel
  `db.client.connection.*` metrics (A25/B20).
- Add async-profiler agent startup in `deploy/Dockerfile.java` for
  continuous JFR (A17b).
- Add `otel.instrumentation.micrometer.enabled=true` and verify exemplars
  propagate from Micrometer into OTLP histograms.

#### 6.6 Web-app upgrades

- Expand from 1 route to 3: `/` (catalog), `/cart`, `/checkout`.
- Add interactions that produce breadcrumbs: search, add-to-cart,
  remove-from-cart, checkout.
- Emit `route_view` spans on route changes (TanStack Router middleware).
- Emit `user_step` events on each interaction.
- Wire the OTel `LongTask` instrumentation for INP / long-task reporting.
- Add a Playwright-driven scenario runner (`scenarios/a28-rum.sh`).

---

### 7. Cross-cutting concerns that must be solved once

#### 7.1 Propagation-continuity as a first-class metric

The single biggest "I can't explain what happened" cause is a broken
`traceparent` chain. The correlation doc (`capture/correlation.md`) already
names `trace_context_rate`, `trace_context_validity_rate`,
`frontend_backend_continuation_rate`, `same_trace_bundle_rate`,
`async_link_rate`, `compare_base_rate`. These should be **visible in the UI**
as a "Trace Health" panel per service and per edge in the service map. A red
edge in the service map means "12% of requests to this service arrive
without a trace context" — that is exactly the proxy/Sentry/CORS bug the
playground warns about.

#### 7.2 Sampling strategy

Head sampling (`ParentBased(TraceIdRatioBased)`) is the V1 default. The UI
must always show **whether a given trace was sampled** and what its sampling
probability was. For the playground, a 100% sampler is fine for signal
scenarios; B26 explicitly demonstrates what happens at 10%. Tail-based
sampling (errors 100%, slow traces 10%, rest 1%) is a Parallax-side feature
to demo on the OTLP ingest path — out of scope for V1 but worth a design
note.

#### 7.3 High-cardinality safety

The native-OTLP decision (`decisions/native-otel-tables.md:49-61`) says
`parallax.run.id` should **never** be a metric tag (only a trace/log attr).
Every UI surface that aggregates metrics must enforce this — group by
`service.name`, `service.version`, `deployment.environment.name`,
`http.route`, `rpc.method`, `db.operation.name`, status code — never by
`trace_id`, `run_id`, `user_id`, `session_id`. The metrics builder UI must
refuse high-cardinality group-bys.

#### 7.4 Redaction as a visible property

Every span, log, and bundle rendered in the UI should carry a small badge:
"raw", "redacted (3 fields)", "ref-only", "hashed". This is the only way a
user trusts the data when handing it to an agent. The redaction-report
viewer (per `redaction.md:257-310`) should be reachable from every issue
and every bundle.

#### 7.5 Symbolication

Backend: Rust demangling + Java frame-source-map resolution. Sentry's
Symbolicator is a separate Rust service; Parallax can run a much smaller
in-process symbolicator that reads DWARF (Rust) + sourcemaps (TS) +
JVM line tables (Java) on demand. Required for the issue stacktrace view
to be useful on release builds.

#### 7.6 Multi-language trace shape differences

Rust tracing spans and Java OTel agent spans look different (Rust emits
`otel.name` overrides via `#[instrument]`, Java auto-generates from class
+ method). The trace inspector should show `telemetry.sdk.language` on
every span and offer a "normalized span name" alongside the raw one. The
service map and operation list should aggregate by normalized name.

#### 7.7 Time

All timestamps in nanos since epoch UTC, stored as `ts_nanos`. UI renders
in the user's local timezone with a toggle for UTC. Clock-skew between
services (B18/B27) should be detected and shown as a warning on the span
(`inferred_skew_ms`).

---

### 8. Competitive comparison — what each rival teaches Parallax

Drawn from `docs/research/market/` deep-dives. Each row is "the one idea
worth stealing".

| Rival | The one idea worth stealing |
|---|---|
| **Sentry** | Issue grouping + lifecycle + release attribution + breadcrumbs + Suspect Commits. Parallax already plans grouping; release attribution and breadcrumbs are the gaps. |
| **Grafana Tempo + Grafana** | Service map; metrics exemplars as trace jump-points; explore mode; trace-to-logs navigation; the "click anything to filter" UX. |
| **Jaeger** | `Compare traces` feature (already shipped in Jaeger UI); deep dependency graph; `Find traces with same operation` button. |
| **Honeycomb** | "Group-by" as the dominant UI primitive; BubbleUp for outlier dimensions; high-cardinality-by-design query model; the query builder that compiles to a clear pipeline. |
| **Datadog APM** | The flame-host-list view (trace + the hosts running at the time); live tail in every view; deployment markers overlaid on time-series; trace-to-profile flamegraph jump. |
| **Kibana / Elasticsearch** | KQL + field explorer + saved searches; the left-sidebar facets; log-to-APM cross-navigation. |
| **SigNoz** | "Open investigation format" framing; minimum-span-filter for log search; per-service RED pages with minute-bucket sparklines. |
| **OpenObserve** | Single-binary Rust + DataFusion + Parquet + tantivy — the architectural cousin; VRL pipelines as a query-time transformation language. |
| **Coroot** | eBPF-derived service map without instrumentation (inspiration for an "ingest from `parallax run start` + eBPF side-channel" future); 2-stage ML+LLM RCA. |
| **Maple** | The best local single-binary UX in the comparison set; the Effect-TS pipeline as a query-builder model. |
| **Apollo Studio** | The GraphQL field tree (per-field latency, error rate, N+1 detection) — directly relevant for §5.12. |
| **Pyroscope** | Continuous profiling as a first-class signal, with the CPU/alloc/lock flame modes. |
| **Grafana Faro** | Browser RUM with web vitals + long-tasks + errors + user-step breadcrumbs — directly relevant for §5.11. |
| **TMA1** | GreptimeDB-embedded single-binary; OTLP reverse-proxy on `:14318`; the closest architecture — Parallax should beat it on the evidence-bundle + redaction + lifecycle dimensions it omits. |

---

### 9. Suggested sequencing for the next agent

This is **not** a commitment — it is a proposed order of work that respects
dependency: each step unblocks the next.

1. **Propagation contract enforcers.** Baggage propagator; sampler;
   `parallax.run.id` stamping in `libs/playground-telemetry`. (A22/A10b
   unblocked.)
2. **Run-timeline data path.** `parallax run start` injection confirmed;
   `agent-cli-tracing.md` schema implemented in Turso; `/runs/$runId`
   timeline view. (A27 archetype-H scenario unblocked.)
3. **Service map + topology.** `serviceMap(from, to, env?)` query;
   force-directed graph component; container/host sub-mode.
4. **GraphQL field explorer.** Flip the flag in catalog; build the field-tree
   renderer; add the D2 (catalog → pricing gRPC) resolver.
5. **Structured logs + field explorer.** Body FULLTEXT + trace_id INVERTED
   indexes in GreptimeDB; `logFacets(...)` query; field-explorer sidebar.
6. **Metrics exemplars + PromQL editor.** Add exemplars on the Rust side;
   expose a PromQL/SQL code mode; exemplar dots on charts.
7. **Release + deploy surface.** `/v1/deploys` ingest; `/releases` page;
   issue release-attribution; regression badge.
8. **Frontend / RUM surface.** Expand web to 3 routes; Playwright scenario;
   session timeline view.
9. **Continuous profiling.** OTel profiles alpha ingest; flamegraph
   component; span-scoped profile query.
10. **Causal graph + evidence-bundle v2.** Typed node/edge schema; multi-
    anchor bundles; causal-graph renderer.
11. **Polish: command palette, saved investigations, keyboard nav, URL
    state everywhere.**

Each step is independently demoable.

---

### 10. Open questions for the follow-up agent

1. **Sampling story.** Is head-based `ParentBased(TraceIdRatioBased)`
   enough, or should Parallax ship tail-based on the ingest path before
   V1 ends? Affects which traces are "missing" in the causal graph.
2. **Profiles signal maturity.** OTel profiles is alpha — is the data model
   stable enough to commit a GreptimeDB table to it now, or should V1
   ingest pprof/JFR raw and convert later?
3. **GraphQL field-level tracing across the polyglot.** Java path is the
   OTel agent's `data-fetcher` flag. Rust path is not standardized (Juniper
   has nothing; async-graphql has nothing). Do we write the instrumentation
   in Parallax, or upstream?
4. **Service map edge attribution.** SERVER-side span's `peer.service` is
   often missing in the OTel Java agent default config; CLIENT-side span's
   `peer.address` is set. Which side is authoritative? Affects the
   `serviceMap` query design.
5. **Run-timeline nesting for archetype H.** Should the daemon→container
   nesting be encoded as a span hierarchy (one trace, the container's
   spans are children of the daemon's spans across the exec boundary), or
   as two traces linked by `parallax.run.id` (same run, two roots)? The
   former is more intuitive to render; the latter is more robust to
   container-startup failure.
6. **Redaction retrofit.** `decisions/native-otel-tables.md:69-71` defers
   redaction to query-time / bundle-build. Should the playground demo a
   stricter mode (redact at ingest) so the redaction-report viewer has
   something to chew on before query-time redaction ships?
7. **MCP outputSchema.** Should every Parallax GraphQL query also have a
   JSON Schema projection so MCP agents can validate? (Relevant to §5.15.)
8. **Operation-name normalization across languages.** Is there a canonical
   mapping (`dev.tailrocks.checkout.CheckoutHandler#handle` ≈ `GET
   /checkout` ≈ `playground.checkout.handle`)? Affects service-map
   aggregation.
9. **Persistent-query registry.** Where does the GraphQL persisted-query
   hash → document lookup live? Turso table; populated how?
10. **Time-window budget for `serviceMap` over long ranges.** Computing the
    graph over 30d of spans is expensive — is a daily pre-aggregation
    (`rollups_service_edge_minute`) worth the storage?

---

### 11. Appendix — pointer map

#### 11.1 Parallax research docs that constrain this brief

- `docs/research/architecture/simple-ui-v2.md` — UI design intent.
- `docs/research/architecture/api-concept.md` — GraphQL design intent.
- `docs/research/architecture/causal-reconstruction.md` — typed node/edge
  pipeline.
- `docs/research/architecture/evidence-bundle-schema.md` — bundle spec.
- `docs/research/architecture/integration-contract.md` — required resource
  attrs.
- `docs/research/architecture/trace-linking.md` — span link semantics.
- `docs/research/capture/rust-stack-instrumentation.md` — Rust instrumentation
  matrix.
- `docs/research/capture/frontend.md` — RUM data path.
- `docs/research/capture/agent-cli-tracing.md` — CLI/agent-session model.
- `docs/research/capture/run-id-standardization.md` — `parallax.run.id`.
- `docs/research/capture/correlation.md` — A4 correlation gates.
- `docs/research/capture/redaction.md` — A6 redaction pipeline.
- `docs/research/decisions/native-otel-tables.md` — GreptimeDB native model.
- `docs/research/decisions/storage-engine.md` — engine rationale.
- `docs/research/decisions/metadata-store.md` — Turso mandate.
- `docs/research/market/observability-feature-matrix.md` — competitor
  feature map.
- `docs/research/market/backend-and-data-flow.md` — competitor data-flow map.
- `docs/research/market/closest-to-parallax-ranked.md` — closeness ranking.
- `docs/research/validation/telemetry-playground-sample-project.md` —
  playground design doc (701 lines).

#### 11.2 Playground code that this brief references

- `libs/playground-telemetry/src/lib.rs:61-66` — resource attrs.
- `libs/playground-telemetry/src/lib.rs:111-125` — Sentry init.
- `services/checkout/src/main.rs:27-63` — chaos knobs.
- `services/checkout/src/main.rs:221-230` — gRPC `MetadataInjector`.
- `services/catalog/src/main/java/dev/tailrocks/catalog/CatalogApplication.java:46-92`
  — flag eval, DataLoader, subscription.
- `services/orders/src/main.rs:62` — span link to producer.
- `services/pricing/src/main.rs:38` — server-streaming.
- `cli/src/main.rs:45-58` — cron weighted bucket.
- `web/src/telemetry.ts:23-45` — browser OTel init.
- `web/src/routes/__root.tsx:15` — SSR `traceparent`.
- `deploy/docker-compose.yml` — service topology.
- `deploy/docker-compose.xlang.yml` — cross-language overlay.

#### 11.3 Parallax UI code that this brief references

- `ui/src/routes/traces.$traceId.tsx:65` — trace detail page.
- `ui/src/components/console/trace-waterfall.tsx:22` — waterfall.
- `ui/src/components/logs-table.tsx` — logs table.
- `ui/src/components/metric-strip.tsx:34` — cross-signal strip.
- `ui/src/routes/runs.$runId.tsx:83` — run detail page.
- `ui/src/routes/issues.$fingerprint.tsx:75` — issue detail page.
- `ui/src/routes/dashboards.$dashboardId.tsx:73` — dashboard detail.
- `ui/src/routes/sql.tsx:34` — SQL console.
- `crates/parallax-api/src/lib.rs:879-1926` — GraphQL schema.
- `crates/parallax-core/src/derive.rs:18` — error-event derivation.
- `crates/parallax-core/src/fingerprint.rs:54` — fingerprinting.
- `crates/parallax-core/src/bundle.rs:15-37` — evidence bundle.

#### 11.4 External specs referenced

- OTel traces spec — `opentelemetry.io/docs/concepts/signals/traces/`.
- OTel overview (signals, propagation, semconv, resources) —
  `opentelemetry.io/docs/specs/otel/overview/`.
- OTel profiles (alpha) — `opentelemetry.io/docs/specs/otel/profiles/`.
- OTel database metrics (Stable + Development) —
  `opentelemetry.io/docs/specs/semconv/database/database-metrics/`.
- OTel GenAI semconv — `github.com/open-telemetry/semantic-conventions-genai`.
- W3C trace context — `w3.org/TR/trace-context/`.
- W3C baggage — `w3.org/TR/baggage/`.
- tokio-metrics — `docs.rs/tokio-metrics/latest/tokio_metrics/`.
- Spring for GraphQL observability — Spring docs `observability.adoc`.

---

End of document.


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
