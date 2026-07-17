# Full-Observability UI and Telemetry Playground Expansion

Research date: 2026-07-06

> **Status (2026-07-17): historical brainstorming and design evidence, not an
> active implementation plan, backlog, checklist, or ordering authority.** Many
> candidate surfaces subsequently shipped: alerts, dashboards, ecosystem graph,
> investigations, invocations, issues, logs, metrics/runtime metrics, services,
> SQL, story, traces, evidence gaps, and SSE live tails. React Flow + ELK owns
> graph rendering. Closed plan references below are historical, not ownership.
> Only current files in
> [`plans/`](../../../plans/) authorize implementation. Lists below are dated
> option inventories and may be stale; do not implement from them.

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
  evidence-bundle preview/export, missing-evidence detection,
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
  `graphql.document` is opt-in. Source:
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
  cron success/fail/stuck, deploy regression, clock skew.

This is already better than a toy demo. The gap is that it still models mostly
an e-commerce microservice world. Parallax needs the playground to also model
**interactive execution systems**: host CLI → daemon → workspace/session →
container → multiplexer → multiple agents → tools/commands/files/tests.

## Product goal: one intuitive causal graph

The UI should organize every signal around five correlated identities:

| Identity | OTel/Parallax carrier | Purpose |
| --- | --- | --- |
| Service/process identity | `service.name`, `service.version`, resource attrs | Who emitted telemetry. |
| Trace identity | `trace_id`, `span_id`, parent/links | One request/workflow causality tree/DAG. |
| Run identity | `parallax.run.id` resource attr | One bounded CLI/session/workspace execution across many traces. |
| User/workspace/session identity | allowlisted baggage/resource attrs, future normalized rows | Human-visible path: screen, workspace, container/session, agent session. |
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
   - container/session;
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
00.084  docker         created container image=...
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
- Evidence-bundle preview: what a coding agent will receive, missing
  evidence, token size.

### 7. Runs/sessions: first-class local execution observability

Runs should become the bridge between application observability and CLI/agent
observability.

Required UI sections:

- Run header: id, command, status, exit code, duration, service count, trace
  count, issue count, last activity.
- Process tree: wrapper → child commands → daemon/container/agent processes.
- Screen timeline: TUI/screen/view transitions, selected items, button presses,
  background operations.
- Container/session panel: image, container id, workspace mount, attach time,
  multiplexer session id, environment policy.
- Agent timeline: agent start/end, prompts/context loads/tools/files/commands,
  validations, outcomes.
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

## Telemetry model recommendations

### Resource attributes: stable identity first

Every service/process should set:

- `service.name`
- `service.version`
- `service.namespace` where useful (`playground`, `parallax`, `tailrocks`)
- `deployment.environment.name`
- `telemetry.sdk.language`, `telemetry.sdk.name`, `telemetry.sdk.version`
- process/runtime attributes emitted by SDKs
- `parallax.run.id` when run-scoped and no OTel standard run identifier matches
  the Parallax run semantics
- `session.id` / `session.previous_id` for client-side browser/TUI/user sessions
  when the OTel session convention matches the meaning
- `parallax.workspace.id` only if non-sensitive/opaque; no current OTel standard
  workspace id covers Parallax's local workspace concept
- standard container/resource identity such as `container.id`, `container.name`,
  `oci.manifest.digest`, `host.id`, and `service.instance.id`
- standard screen/widget attributes `app.screen.id`, `app.screen.name`,
  `app.widget.id`, and `app.widget.name` for browser/native/TUI-visible screens
  and clicks

Do not put high-cardinality identity in resource attributes. Use opaque
ids, with metadata resolved inside Parallax/Turso.

### Standard OTel attributes vs Parallax custom attributes

Use OpenTelemetry semantic conventions first, then Parallax custom attributes
only for product concepts that the registry does not cover. Current OTel 1.43
findings:

| Concept | Standard OTel candidate | Decision for Parallax |
| --- | --- | --- |
| Generic Parallax run | `session.id` for client-side/user session; `cicd.pipeline.run.id` for CI/CD only; CLI `process.*` for one process execution | Keep `parallax.run.id` as Parallax's cross-process, cross-trace execution anchor. Also set `session.id` for real user/TUI/browser sessions and `cicd.pipeline.run.id` only for actual CI/CD pipeline runs. |
| Workspace | `vcs.repository.url.full`, `vcs.repository.name`, `vcs.ref.head.revision`, `process.working_directory` | Keep `parallax.workspace.id` as opaque product identity. Do not use `process.working_directory` or raw paths as workspace ids by default. |
| Container/session | `container.id`, `container.name`, `container.command`, `container.command_args`, `container.command_line`, `oci.manifest.digest`, `host.id`, `service.instance.id`, `session.id` | Use standard container/process/session attrs. Do not introduce a Parallax-specific container/session id. If one logical session spans multiple containers or restarts, link them with `session.id`, span links, and `parallax.run.id`. |
| Screen/view | `app.screen.id`, `app.screen.name`, `app.widget.id`, `app.widget.name`, events `app.screen.click`, `app.widget.click` | Prefer `app.screen.*` and `app.widget.*` for browser and TUI-visible screens/widgets. Use custom `tui.panel.id` and `tui.block.*` only for terminal-specific structure not covered by OTel. |
| CLI execution | CLI spans with `process.executable.name`, `process.exit.code`, `process.pid`, `process.command_args`, `process.executable.path`, `error.type` | Use standard CLI/process attrs for each command/span. Add `parallax.run.id` only to stitch many commands/processes/traces into one Parallax run. |

Custom naming rule:

- Never create custom attributes inside standard namespaces such as `http.*`,
  `db.*`, `messaging.*`, `container.*`, `process.*`, `app.*`, `session.*`, or
  `otel.*`.
- Put Parallax-only attributes under `parallax.*` or terminal-specific
  attributes under a documented `tui.*` overlay, then validate them through a
  future Weaver registry.
- Custom attributes are allowed only after checking the OTel registry and
  recording why the standard attribute is insufficient.
- High-cardinality custom ids are allowed on traces/logs/events for filtering
  and joining, but never as default metric labels.

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

Events should use low-cardinality names; variable values live in attributes with
cardinality rules.

### Span links: required for reality

Parent/child is not enough. Parallax should treat links as first-class UI edges.

Use links for:

- messaging producer → consumer;
- batch job processing many messages;
- GraphQL DataLoader batch spanning multiple resolver parents;
- fan-in aggregation where one span summarizes multiple upstream spans;
- retry attempts that start new traces;
- CLI host run → container-internal trace when a process boundary creates a new
  trace;
- agent session → command traces;
- external trace import or remote tool run.

UI must show these as causal edges, not buried JSON.

### Logs: structured, trace-correlated, derivable

Logs should preserve the original body, severity, trace/span/run ids, and
attributes. For Parallax issue derivation and UI:

- ERROR/FATAL logs create candidate error events.
- `exception.type`, `exception.message`, `exception.stacktrace` should be emitted
  when available.
- Logs without trace/span/run context should be visible as evidence gaps.

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
   → session start → container → multiplexer
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

- Postgres query spans with `db.query.text`.
- Slow query and lock wait.
- Connection pool contention.
- N+1 sequential query pattern.
- Returned rows and query-plan-like metadata where safe.
- Service-local cache hit/miss and cache stampede/leak without adding new
  infrastructure.

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
10. What evidence bundle would be given to a coding agent?

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
- `agentSession(runId)` → normalized agent actions, content refs.

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
   interactions, and lifecycle events.

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
| 3 | `app.screen.id` / `app.screen.name` | `workspace-select` 81% | 7% | UI path-specific |
| 4 | `container.image.id` | `sha256:...` 74% | 3% | container-specific |

Guardrails:

- Do not group by raw high-cardinality free-text such as URL query text or
  stacktrace body.
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
screen contents stay absent by default.

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
- runtime coverage: process/runtime/container metrics exist;
- frontend/backend propagation: browser trace continues into backend;
- async coverage: producer/consumer links present.

UI placement:

- service catalog health card;
- trace/run story header;
- playground validation checklist;
- evidence bundle preview.

### K. Expanded query/API ideas from this pass

The research recorded these candidate GraphQL/query shapes. Current ownership is
limited to plans 100, 105, and 122:

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

Candidate materializations recorded by the same pass:

- `metric_exemplars`: metric name/time/value/resource attrs/trace/span/run ids.
- `field_stats_minute`: entity/key/top values/cardinality/coverage/safety.
- `service_catalog_snapshots`: current identity/resources/ownership/health.
- `topology_edges_minute`: graph mode, source, target, endpoint/resource attrs,
  RED metrics, evidence quality.
- `runtime_correlations`: runtime anomaly windows linked to traces/runs/issues.
- `investigations`: Turso metadata for saved human investigation state.

### L. Playground additions from this pass

The pass also recorded scenario candidates; plan 122 is closed — activation
needs an active plan (e.g. 154) or stays design-only:

- **Exemplar demo:** p99 checkout latency chart contains exemplar dots that open
  exact traces; control scenario lacks exemplars and shows lower confidence.
- **Field explorer demo:** a log/error spike where `service.version`,
  `graphql.field.name`, `app.screen.id`, and `app.screen.name` stand out.
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

## Consolidated brainstorm additions from `parallax-ui-observability-brainstorm.md`

The root-level brainstorm file was reread and deduplicated into this section on
2026-07-06. The raw import was removed so this research brief remains the single
source of truth instead of a repeated appendix. Items below are only the unique
or sharper details that were not already covered by the sections above.

### Scope and win condition refinements

- The UI goal is not only signal parity. The win condition is that every chart,
  row, span, issue, run, deploy, and agent step is a doorway to related logs,
  metrics, traces, linked traces, release context, runtime state, and bundles.
- "Replace Grafana/Kibana/Sentry" means:
  - Grafana: topology, metric query, exemplars, profiles, explore workflow;
  - Kibana: field facets, saved searches/views, structured log predicates;
  - Sentry: releases, regression lifecycle, breadcrumbs, frontend sessions,
    symbolication, cron/job monitoring.
- Still out of scope by default: CLI redesign, new runtimes, storage changes,
  frontend framework changes, multi-user/RBAC, alert routing, and full session
  replay playback.

### Current-state details to remember

- Current UI routes already cover overview, services, service detail, issues,
  issue detail, traces, trace detail, logs, runs, run detail, dashboards, and
  read-only SQL.
- Reusable UI primitives already exist for waterfalls, log tables, metric strips,
  live streams, stat cards, sparklines, heat cells, trends, range picking, data
  tables, and stack-frame parsing.
- Current GraphQL already exposes about thirty query fields plus mutations for
  issue status, run lifecycle, dashboard save/delete, traces, logs, services,
  runs, bundles, metrics, histogram quantiles, and read-only SQL.
- Parallax signatures worth preserving:
  - issue derivation from span status, span exception events, and ERROR/FATAL
    logs;
  - stable fingerprinting from error type, normalized message, and top frame;
  - bounded evidence bundles with canonical hash;
  - causal reconstruction as typed nodes/edges with strength tiers;
  - native OTLP GreptimeDB tables and `parallax.run.id` as run/session join key.

### Execution archetypes the playground should explicitly cover

The earlier sections already list domains. The sharper model is that Parallax
must explain these execution shapes, each with at least one playground scenario:

| Archetype | Required story |
| --- | --- |
| Browser interaction | click/route/user step → fetch → backend trace, including CORS/header propagation failure. |
| Sync fan-out | entry service fans out to HTTP/gRPC/GraphQL/DB branches; critical path explains latency. |
| gRPC unary/streaming | unary call plus server/client/bidi stream with per-message events and cancellation. |
| GraphQL operation | operation span → resolver/field tree → DataLoader batch or N+1 → DB/gRPC/GraphQL downstream. |
| Async messaging | producer → message headers → consumer span link, including batch and dead-letter cases. |
| Scheduled job | root job span with run id; missed, duplicate, stuck, and long-tail runs visible. |
| Monolith/internal subsystems | internal spans plus runtime metrics explain lock contention, GC, Tokio scheduling, queues. |
| Daemon/session/container/agent | host CLI → daemon → container/multiplexer → agent/tool/action, stitched by trace context plus `parallax.run.id`. |
| Deploy/change | deploy event + service version + VCS revision explain regressions, rollbacks, canaries, partial deploys. |
| Cross-trace fan-in | span links + baggage/correlation id + issue fingerprint reconstruct causality across traces. |

### OpenTelemetry modeling details to standardize

- Use all five OTel signal families as the mental model: traces, metrics, logs,
  baggage, and future profiles. Profiles are future-facing because OTel profile
  maturity is lower than traces/logs/metrics, but UI slots should be reserved.
- Propagation channels to test: HTTP headers, gRPC metadata, Kafka headers,
  RabbitMQ headers, WebSocket/SSE first-frame or query convention, Docker
  environment/first RPC, and in-process message context. Databases generally do
  not receive trace context; client spans model database calls.
- Span links are mandatory for batch consumers, scatter/gather, trust-boundary
  trace restarts, long-lived async jobs, DataLoader fan-in, and cross-agent/tool
  causality.
- Attribute/event/log rule:
  - attribute = stable metadata/final values;
  - span event = timestamped micro-step/retry/feature flag/cache hit/exception;
  - log = verbose or high-volume diagnostic body, correlated by trace/span/run.
- Commit to low-cardinality OTel semantic conventions for HTTP, RPC, database,
  messaging, GraphQL, feature flags, deployment, VCS, process/system/runtime,
  and agent/gen-ai-like tool activity where useful.

### UI surfaces sharpened by the brainstorm

- **Investigation shell:** persistent pinned context panel; pinning a trace/run/
  issue/deploy carries time window, services, run id, filters, and related
  artefacts across pages.
- **Topology/service map:** support service, operation, container/host, and
  GraphQL-field sub-modes; edges expose RED metrics, operations, traces, logs,
  propagation gaps, and baseline deltas.
- **Trace detail:** add collapsed flame/group view, critical-path highlight,
  color-by attribute, minimap brush, clock-skew warning, side-by-side trace
  compare, and virtualized rendering for very large traces.
- **Linked-traces graph:** recursively walk span links across traces and render
  trace cards with root service, duration, status, and thumbnail waterfall.
- **Run timeline:** render swim lanes for process lifecycle, CLI/agent phases,
  backend calls, errors, logs, runtime metrics, and agent steps. For container
  sessions, nest container timeline inside daemon/run timeline.
- **Issues:** add deploy/release attribution, regression badge, grouping
  controls, breadcrumb lane, brush-and-drill occurrence trend, and future suspect
  commit hooks.
- **Logs:** add structured-field query DSL, facets, saved views, live-tail
  virtualization, and log-to-trace chips.
- **Metrics:** add PromQL/SQL code mode beside visual builder, metric math,
  template variables later, anomaly overlay, SLO/burn-rate panels, and exemplar
  trace jumps.
- **Profiles:** reserve flamegraph/icicle UI for CPU, allocation, lock/contention,
  JFR, pprof, and span-scoped profile links.
- **Frontend/RUM:** add session list/detail, route/user-step/fetch/error/vitals
  lanes, web-vitals dashboard, and frontend-error → backend-trace links.
- **GraphQL explorer:** render operation/field tree with resolver latency,
  DataLoader batches, field errors, persisted-query lookup, operation cost, and
  GraphQL→gRPC/GraphQL downstream jumps.
- **gRPC streaming explorer:** show per-message sent/received events, sizes,
  mid-stream errors, and cancellation on long-lived RPC spans.
- **Causal graph:** render typed nodes and strength-tier edges from causal
  reconstruction, including supporting and contradicting evidence.
- **Evidence bundle:** later support multi-anchor bundles, typed node/edge
  projection, MCP `outputSchema`, and bundle diff.
- **Global UX:** global time/environment/release/service filters, command
  palette, keyboard navigation, saved investigations, and URL state everywhere.

### Historical Playground Scenario Option Catalog

These are design inputs for plan 122, not executable tasks in this file:

| ID | Scenario | Proves |
| --- | --- | --- |
| A2b | Rust metric exemplar | chart spike jumps to exact trace. |
| A6b | GraphQL field-level trace | resolver tree, DataLoader, N+1. |
| A7b | gRPC per-message events | streaming explorer and mid-stream failure. |
| A9b | Structured-field logs | Field Explorer/facets and log query DSL. |
| A10b | Baggage-driven branch | baggage-carried business context propagation. |
| A17b | Rust/JVM profiles | future span-scoped flamegraph surface. |
| A19 | Long/wide trace | virtualization, grouping, minimap, critical path. |
| A20 | Cross-trace causal chain | linked traces and causal graph. |
| A21 | Real deploy regression | deploy/release attribution and regression lifecycle. |
| A22 | Tokio runtime under load | runtime panels tied to slow traces. |
| A23 | GraphQL→gRPC gateway | field tree with downstream RPC. |
| A24 | GraphQL→GraphQL | upstream/downstream GraphQL operations. |
| A25 | Real Postgres spans/pool metrics | DB spans, pool contention. |
| A26 | Cache behavior without new infra | cache hit/miss, stampede, and leak using existing service-local cache paths. |
| A27 | Daemon/child nested run | CLI→daemon→container/agent run timeline. |
| A28 | Frontend RUM session | route/user-step/web-vitals/error story. |

Additional chaos cases:

- Tokio starvation, Postgres pool exhaustion, JVM GC storm, cache stampede, slow
  GraphQL field, persisted-query mismatch, gRPC stream disconnect, 10% sampling
  drop, extended clock skew, frontend rage/dead clicks, propagation break, and
  container-spawn timeout.

### Historical Playground Topology Options

Plan 122 is closed; any selected topology/instrumentation change needs a new
or active plan (e.g. 154), not this historical brief.

- Wire inventory/catalog to the existing Postgres container.
- Keep cache scenarios inside existing services; do not add Redis or another
  infrastructure dependency unless the playground separately adopts it later.
- Add a Rust gateway only if it uses the existing Rust/TypeScript/Java language
  constraint and does not become a new framework commitment for Parallax core.
- Add a small playground daemon plus `enter` child process to model
  host/daemon/container/agent propagation without copying any external project.
- Expand web to three routes and drive it with Playwright scenario scripts only
  if Playwright already fits playground/dev-test constraints.
- Upgrade `libs/playground-telemetry` with baggage propagator, parent-based
  ratio sampler, Tokio metrics → OTel gauges, stable semconv constants,
  Rust exemplar helper, and automatic `parallax.run.id`
  stamping.
- Upgrade Java config with GraphQL data-fetcher spans/links, Hikari pool metrics,
  Micrometer exemplars, and optional profiler/JFR path.

### Cross-cutting rules from the brainstorm

- **Propagation continuity is a metric.** Show trace-context rate, validity,
  frontend/backend continuation, same-trace bundle rate, async-link rate, and
  sampled-out/missing evidence per service and edge.
  Exact metric names from existing capture research:
  `trace_context_rate`, `trace_context_validity_rate`,
  `frontend_backend_continuation_rate`, `same_trace_bundle_rate`,
  `async_link_rate`, and `compare_base_rate`.
- **Sampling must be visible.** Show whether a trace was sampled and which policy
  applied. Playground can run 100% for normal scenarios and 10% for sampling-gap
  scenarios.
- **High-cardinality guardrails are UI constraints.** Metric group-bys should
  refuse `trace_id`, `run_id`, `user_id`, and `session_id`; those belong in
  traces/logs and filtering, not metric labels.
- **Symbolication matters.** Future issue detail needs Rust demangling, Java
  frame/source mapping, and TypeScript sourcemaps for release builds.
- **Normalize operation names across languages.** Show raw span name plus
  normalized operation key; aggregate service-map operations by normalized key.
- **Time/clock skew must be explicit.** Store nanos UTC; render local/UTC toggle;
  warn on inferred skew and clamp broken visuals instead of crashing.

### Competitive lessons after deduplication

| Rival | Lesson to keep |
| --- | --- |
| Sentry | Issue lifecycle, releases, breadcrumbs, suspect commits, frontend sessions. |
| Grafana/Tempo | Explore, service graph, exemplars, trace/log/metric pivots. |
| Jaeger | Trace compare, deep dependency graph, same-operation trace search. |
| Honeycomb | Group-by-first exploration and BubbleUp/outlier dimensions. |
| Datadog | Service Catalog/Page, deploy markers, runtime/profile pivots. |
| Kibana/Elastic | Field explorer, facets, saved searches, Discover workflow. |
| Apollo Studio | GraphQL field tree and resolver-level performance. |
| Pyroscope | Flamegraph as first-class signal, eventually span-scoped. |
| Grafana Faro/Sentry RUM | Web vitals, long tasks, errors, user-step breadcrumbs. |
| SigNoz/OpenObserve/Coroot/Maple/TMA1 | Use as pattern references only; keep Parallax stack unchanged. |

## Structural adequacy audit after brainstorm import

The keeper document is now detailed enough to act as the single research source
for a future UI/playground design agent because it covers these layers in order:

1. **Why:** executive thesis and product goal.
2. **What exists:** current Parallax and playground capabilities to preserve.
3. **What user questions matter:** causal graph, story, ecosystem, run/session,
   issue, logs, metrics, and evidence-bundle workflows.
4. **What data can carry it:** OpenTelemetry signal, propagation, resource,
   span-name, span-event, link, baggage, log, metric, profile, and runtime
   guidance.
5. **What UI surfaces need to exist:** command center, ecosystem map, story,
   trace detail, logs, issues, runs, metrics, SQL, service catalog, runtime lane,
   investigations, quality score.
6. **What the playground must prove:** browser, GraphQL, gRPC, messaging,
   database/cache, runtime, CLI/session/container/agent, release,
   quality-gap, exemplar, topology, and investigation scenarios.
7. **What API/materializations are implied:** query backlog, derived tables,
   reusable UI components, resolver ownership, and data-contract enforcement.
8. **What remains undecided:** future execution order and implementation
   questions.

The root brainstorm still had valuable detail that the earlier consolidation
compressed too much: exact current route/API inventory, lower-level semantic
contracts, TUI/agent conventions, env-var propagation, Weaver/OBI research,
algorithms, visualization primitives, and source pointers. The sections below
preserve those details without keeping the duplicate root file.

### Current UI and API inventory from the removed brainstorm

The dated inventory below is superseded. Current routes are `/`, `/alerts`,
`/dashboards`, `/dashboards/$id`, `/ecosystem`, `/investigations`,
`/invocations`, `/issues`, `/logs`, `/metrics`, `/metrics/$name`, `/services`,
`/services/$service`, `/sql`, `/traces`, and `/traces/$traceId`.

Historical inventory:

- Routes: `/`, `/services`, `/services/$service`, `/issues`,
  `/issues/$fingerprint`, `/traces`, `/traces/$traceId`, `/logs`, `/runs`,
  `/runs/$runId`, `/dashboards`, `/dashboards/$id`, `/sql`.
- Trace detail already has `TraceWaterfall`, selected span detail, attributes,
  resource attributes, span links, span events, trace logs, run link,
  failed-span shortcut, stacktrace parsing, `db.query.text`, and
  `exception.stacktrace`.
- Logs already have severity filtering, volume histogram, column toggles,
  document viewer, and SSE live tail.
- Runs already have live stream, metric strip, evidence-bundle preview, and
  bundle download.
- Dashboard/SQL surfaces already prove metric panel metadata plus read-only
  GreptimeDB exploration.
- Reusable primitives to reuse: `TraceWaterfall`, `LogsTable`, `MetricStrip`,
  `LiveStreamPanel`, `StatCard`, `CardSparkline`, `PillMeter`, `DeltaBadge`,
  `HeatCell`, `TrendChart`, `RangePicker`, data-table search/filter/sort/
  pagination, and stack-frame parsing with app-frame classification.

Historical GraphQL snapshot (the generated schema now has **80** queries, **15**
mutations, and zero subscriptions — live count 2026-07-17; names below may lag):

- Queries: `overview`, `serviceList`, `serviceRed`, `issues`, `issue`,
  `issueTrend`, `trace`, `logsByTrace`, `tracesByRun`, `logsByRun`, `logs`,
  `sql`, `run`, `dashboard`, `serviceOverview`, `observedRuns`, `traces`,
  `tracesPage`, `bundle`, `metricNames`, `services`, `metricSeries`,
  `histogramQuantile`, `dashboards`, and `runs`.
- Mutations: `issueSetStatus`, `runStart`, `runFinish`, `dashboardSave`, and
  `dashboardDelete`.
- Constraint: UI calls the GraphQL/API boundary only. No UI page should query
  GreptimeDB or Turso directly.

Current Parallax signatures to keep stable:

- `ErrorEventRow` and `Issue` derive from span exception events, span error
  status, ERROR/FATAL logs, and exception-as-log attributes.
- Fingerprint is based on error type, normalized message, and top frame, with
  volatile values normalized out. Exact earlier formula:
  `error_type \0 normalize(message) \0 top_frame`, with normalizers for
  `<uuid>`, `<hex>`, and `<n>`.
- Evidence bundles are bounded, hashable, and anchorable to issue,
  run, or trace. Existing implementation is single-anchor `(issue|run|trace)`,
  token-budgeted, canonical-hash based, and exports JSON,
  Markdown, and clipboard snippets.
- Causal reconstruction uses typed nodes/edges and strength tiers.
- Native OTLP rows in GreptimeDB plus `parallax.run.id` are the correlation
  spine.

### Absences Recorded By The Dated Audit

This snapshot is retained for provenance and is not a current checklist. Plan
122 must revalidate any residual before adopting it:

- No Docker/container-spawn or daemon/session/container topology.
- No agent-session trace inside a container.
- Browser-side observability is emitted but not richly displayed.
- GraphQL field-level data-fetcher spans are not enabled by default.
- Postgres exists but is not wired into real database-load scenarios.
- No JVM GC/class-loading/pool scenario surface and no Rust Tokio runtime metric
  scenario.
- No continuous profile signal.
- No frontend RUM journey with multi-route breadcrumbs and frustration signals.
- No real deploy webhook ingest; regression is simulated through environment.
- No Rust metric exemplars linked to traces.
- No long-trace stress case for rendering.
- No trace comparison scenario.
- Scheduled job/cron signals exist, but no cron-specific UI treatment.
- No metrics-cardinality explosion scenario.
- Logs are mostly plain bodies; structured-field scenarios are still needed.

### Semantic convention commitment table

| Domain | Attributes/metrics to standardize | UI reason |
| --- | --- | --- |
| HTTP | `http.request.method`, `http.route`, `url.path`, `http.response.status_code`, `server.address`, `server.port` | Route-level RED, service edges, trace filters. |
| RPC/gRPC | `rpc.system`, `rpc.service`, `rpc.method`, `rpc.grpc.status_code`, `rpc.message.type`, `rpc.message.id` | Unary/stream trace detail and message timeline. |
| Database | `db.system.name`, `db.namespace`, `db.collection.name`, `db.operation.name`, `db.query.summary`, opt-in `db.query.text`, `db.client.operation.duration`, `db.client.connection.*` | DB spans, pool contention, runtime lane. |
| Messaging | `messaging.system`, `messaging.destination.name`, `messaging.operation.name`, `messaging.message.id`, `messaging.message.conversation_id`, `messaging.batch.message_count` | Producer/consumer links, batch, lag, dead-letter flows. |
| GraphQL | `graphql.operation.type`, `graphql.operation.name`, opt-in `graphql.document`, `graphql.field.name`, `graphql.field.path`, `graphql.field.type` | Field tree, N+1, DataLoader, partial errors. |
| Feature flags | `feature_flag.context.id`, `feature_flag.provider_name`, `feature_flag.key`, `feature_flag.variant` | Change attribution and branch explanation. |
| Deployment/VCS | `deployment.environment.name`, `deployment.id`, `deployment.name`, `deployment.status`, `vcs.ref.head.revision`, `vcs.ref.head.name`, `vcs.repository.url.full` | Release/deploy regressions, suspect commits, evidence bundles. |
| Runtime/process | `process.cpu.utilization`, `process.memory.usage`, JVM metrics, Tokio metrics, container/cgroup metrics | CPU, memory, GC, task starvation, pool pressure. |
| CLI/TUI/agent | `parallax.run.id`, `cli.*`, `tui.*`, adopted `gen_ai.*`, adopted `mcp.*` | Run/session/story/agent lanes. |

### Detailed TUI and terminal journey model

Interactive terminal workflows should use a Parallax custom semantic convention
overlay, validated later through Weaver:

| TUI concept | OTel primitive | Key attributes/events |
| --- | --- | --- |
| Whole interactive session | Root span | `parallax.run.id`, `session.id`, terminal size, `$TERM`, mux type. |
| Screen/view | Child span | `app.screen.id`, `app.screen.name`, enter/leave timestamps. |
| Panel/pane | Child span or span event | `tui.panel.id`, `tui.panel.focused`. |
| Visible output/work block | Child span | `tui.block.id`, `tui.block.kind`. |
| Foreground operation | Child span of block | Normal span semantics, rendered inside block card. |
| Background operation | Linked span or new root | Link to origin screen plus `app.screen.id`. |
| Keystroke/command/selection | Span event | `tui.input.kind`, `tui.input.value`, `tui.target`. |
| Focus change | Span event | `tui.focus.from`, `tui.focus.to`. |
| Navigation | Span event plus new screen span | `tui.nav.from`, `tui.nav.to`, `tui.nav.trigger`. |

Rendering rule:

- Foreground work appears inside the current screen/block story chapter.
- Background work appears as a detached lane linked to its origin screen and can
  outlive that screen.
- Parallel preparation appears as concurrent sibling spans; critical-path logic
  decides which sibling gated the outcome.

Optional session recording:

- Use asciicast v3-style terminal recording only when explicitly enabled.
- Key recordings by `parallax.run.id` plus `session.id`.
- Story beats may offer "replay at this beat" only when recording exists.
- Raw terminal content is never part of the default bundle; it is a
  reference-only artifact.

### Boundary propagation contract for CLI, daemon, container, mux, and agent

The hard propagation path is not HTTP; it is process/session boundaries.
Standardize it this way:

- CLI to daemon: inject W3C `traceparent`, `tracestate`, and baggage into local
  RPC/Unix-socket metadata.
- Daemon to spawned process/container: set `TRACEPARENT`, `TRACESTATE`, and
  `BAGGAGE` environment variables, plus `OTEL_EXPORTER_OTLP_ENDPOINT`, on the
  child process.
- Child entrypoint: extract the environment context at startup and use it as the
  parent context for all spans.
- Multiplexer attach: model attach/detach as spans; carry `parallax.run.id` and
  context in the pane environment.
- Agent process: inherit `TRACEPARENT`; its `invoke_agent` span becomes a child
  of the container/mux context.

Failure mode to deliberately test:

- Missing env injection creates an orphan container or MCP/server trace.
- Story and causal graph should render this as a broken-continuation evidence
  gap, not hide it as "no data".

### Agent and MCP observability subset

Adopt only the stable-enough core of the GenAI/MCP conventions at first:

- Model/client call spans: `gen_ai.operation.name`, provider, request model,
  response model, token usage, duration, and streaming metrics when present.
- Agent/workflow spans: `invoke_agent`, `invoke_workflow`, `execute_tool`,
  child `invoke_agent` for sub-agents, repeated-tool loop detection.
- MCP spans: `mcp.method.name`, `mcp.session.id`, `mcp.protocol.version`,
  `jsonrpc.request.id`, `network.transport=pipe` for stdio, and W3C context
  propagation across client/server process boundaries.
- Content capture: prompt, message, tool I/O, and terminal input content are
  opt-in and never required for structural debugging.
- Metrics: token usage, operation duration, tool latency, validation failures,
  and exit/failure counts.

Agent-session UI should render:

- Agent trace tree: `invoke_agent` with `chat`, `execute_tool`, shell command,
  MCP call, file read/edit, validation, and sub-agent children.
- Thought-action-observation chapters when content capture is allowed; structural
  step names when content is denied.
- Token/cost/time strip per agent and per step.
- Conversation/session grouping by conversation id or `parallax.run.id`.
- Failure clustering by Parallax issue fingerprint extended to agent/tool
  errors.
- Trace-to-eval or trace-to-bundle promotion as a future evidence-bundle
  projection.

### Story assembly algorithm

Story is deterministic data projection first, optional prose second:

1. Collect spans, span events, span links, correlated logs, metric exemplars,
   runtime windows, RUM route/user events, TUI events, agent spans, deploy
   events, issues, and linked traces for the anchor.
2. Order by `ts_nanos`, then by parent/child and span-link causality to avoid
   clock-skew lying about order.
3. Chapter by boundary, in priority order: browser route, TUI screen, service
   hop, agent handoff, process/container boundary, then issue/deploy boundary.
4. Annotate slow/error beats with strongest causal edge available: pool timeout,
   GC pause, Tokio starvation, downstream error, deploy marker, propagation gap,
   sampled-out trace.
5. Collapse healthy non-critical beats. Keep errors, anomalies, causal edges,
   critical-path spans, user actions, agent actions, and evidence gaps expanded.
6. If a summarizer is later used, it can name chapters and summarize evidence,
   but links, ordering, severity, and "why" edges must remain deterministic.

### Investigation analytics algorithms

BubbleUp/attribute compare:

- Input: selected set S, baseline set B, candidate attributes, entity type
  (span/log/issue/run/metric exemplar).
- For each attribute, compute top values and counts for S and B.
- Rank by a bounded divergence score such as top-value proportion delta or
  Jensen-Shannon divergence.
- Prefer low-cardinality semantic fields first; allow exact identifiers for
  drilldown but label them as identifiers, not general categories.

Trace analysis:

- `traceCompare(a,b)`: align by normalized operation name, depth, sibling order,
  service, and kind; show added/removed spans, duration deltas, status changes,
  and missing links.
- `traceCriticalPath(id)`: compute the latency-gating chain with parallel
  siblings contributing max duration, not sum; use this to keep story beats
  expanded.
- `aggregateTrace(op, from, to)`: group many traces of one operation by
  normalized structure, compute p50/p95/p99 per span group, and mark how often
  each hop is on the critical path.

Topology analysis:

- One-hop graph shows observed direct edges only.
- Trace-path graph shows edges that co-occur in matching traces.
- Transitive/focal graph expands upstream/downstream paths around one focus.
- Endpoint/resource graph changes node granularity to route, RPC method,
  GraphQL field, queue/topic, database/cache, container, or agent.
- UI must label the graph mode because each mode makes different causality
  claims.

### Historical Resolver And Materialization Option Map

Plans 100, 105, and 122 own any current work represented by this map.

| Surface | Query/API concept | Likely materialization |
| --- | --- | --- |
| Ecosystem map | `serviceMap` / `ecosystemGraph`, `relationship`, `topology` | `service_edges_minute`, `topology_edges_minute`. |
| Linked traces | `linkedTraces(traceId, depth)`, `spanLinks(traceId)` | Link index extracted from span rows if JSON scans are slow. |
| Story | `story(anchor, opts)` | Optional `story_events` for low-latency normalized rows. |
| Trace comparison | `traceCompare`, `traceCriticalPath`, `aggregateTrace` | Aggregate trace rollups by normalized operation. |
| Logs/fields | `logFacets`, `fieldStats` | `field_stats_minute`, body full-text/index additions. |
| Metrics | `metricExemplars`, `heatmap`, `sloBurn` | `metric_exemplars`, runtime/anomaly windows. |
| Runtime | `runtimeSnapshot`, `runtimeMetrics` | `runtime_metric_rollups`, `runtime_correlations`. |
| Issues/releases | `deploys`, `releases`, `Issue.affectedReleases` | Deploy/release metadata in Turso plus issue-release joins. |
| Frontend/RUM | `frontendSessions`, `frontendSession`, `frontendErrors` | Session/journey metadata plus source-map refs. |
| Agent/session | `agentSession(runId|conversationId)` | `agent_actions` in Turso, content refs elsewhere. |
| Causal graph | `causalGraph(anchor)` | Typed node/edge cache if assembly becomes slow. |
| Bundles | `bundleDiff`, multi-anchor `bundle`, schema version | Bundle projections. |
| Investigations | `investigation`, `saveInvestigation` | `investigations` in Turso. |
| Quality | `evidenceGaps`, `telemetryQuality` | `evidence_gaps` by trace/run/service. |

### Telemetry contract enforcement with OpenTelemetry Weaver

**Plan 119 is DONE/deleted.** `parallax-semconv` + `cargo xtask semconv` ship
generated constants today; the following remains design evidence (no exclusive
active plan owner for Weaver productization):

Parallax should treat telemetry names as public API:

- Keep a registry under a future `semconv/` directory: OTel base conventions plus
  a Parallax overlay for `parallax.*`, `cli.*`, `tui.*`, selected `gen_ai.*`,
  and selected `mcp.*`.
- Use registry checks/diffs to make telemetry schema changes reviewable.
- Generate type-safe constants for Rust, Java, and TypeScript so playground and
  product code use the same attribute names.
- Run live checks against playground OTLP output to catch missing required
  attributes, type mismatches, deprecated names, and invalid enum values.
- Generate example signals so UI work can start before every playground scenario
  exists.
- Feed registry docs into future MCP/agent surfaces so attributes are
  self-describing to humans and tools.

### Optional eBPF/OBI ingest research note

OTel eBPF Instrumentation/OBI is not a replacement for SDK traces. It may be a
future optional breadth source:

- Useful for zero-code RED metrics, service graph edges, HTTP/gRPC/SQL/Redis/
  Kafka/GraphQL visibility, and automatic service-name resolution.
- Can coexist with SDK telemetry when it detects already-instrumented processes
  and avoids duplicate counting.
- Weak for deep distributed tracing in complex JVM/Tokio/reactive shapes, so SDK
  instrumentation remains the depth source for spans, links, TUI, GenAI/MCP, and
  critical path.
- UI must label provenance on nodes/edges/spans as `SDK` vs `eBPF`.
- Possible future `parallax observe` onboarding mode is Linux/container-only and
  out of V1 scope.

### Visualization primitive catalog

| Data shape | Preferred primitive | Parallax use |
| --- | --- | --- |
| Latency distribution over time | Brushable heatmap | Trace list, span duration, logs, BubbleUp entry. |
| One request across services | Waterfall plus critical-path stroke | Trace detail and story expansion. |
| High-volume service traffic | Service graph or Sankey/flow mode | Ecosystem map. |
| Periodic runtime patterns | Calendar/time heatmap | Cron health, CPU/GC/task saturation. |
| CPU/allocation/lock samples | Flamegraph/icicle, later diff mode | Profiling/runtime lane. |
| Agent run | Trace tree or agent graph | Agent-session view. |
| Ordered cross-signal facts | Story/narrative beats | Signature Parallax surface. |
| Selection vs baseline | Paired mini-histograms | BubbleUp/attribute compare. |
| Metric-to-trace | Exemplar dots | Dashboards and runtime charts. |
| Two traces | Structural diff columns | Trace compare. |
| Cross-trace causality | Strength-tiered node-link graph | Causal graph and linked traces. |

Rule: every visual mark should be a filter, a drilldown, or a bundle candidate.
Dead marks are research-only, not product-grade UI.

### Exact technical knobs retained for future implementation

These names came from the removed appendix and should survive as searchable
implementation references:

- Frontend/UI stack stays `TanStack Start + shadcn/ui on Base UI + Recharts v3`
  and the whole UI is served by `parallax serve`. Use `@tanstack/react-virtual`
  for high-volume trace/log/live-tail virtualization if the current table/list
  primitives are not enough.
- Span status/kind vocabulary to preserve in UI filters: `Unset`, `Ok`,
  `Error`, `INTERNAL`, `CLIENT`, `SERVER`, `PRODUCER`, `CONSUMER`, plus
  `trace_flags`, `trace_state`, `span_links`, and typed log/event `AnyValue`
  bodies.
- Logs data model fields to keep visible in inspectors: `Timestamp`,
  `ObservedTimestamp`, `TraceId`, `SpanId`, `TraceFlags`, `SeverityText`,
  `SeverityNumber`, `Body`, `Resource`, `InstrumentationScope`, `Attributes`,
  and `EventName`; in Parallax row names this maps to fields such as
  `severity_text` and `severity_number`.
- CLI/process spans should use `process.executable.name`, `process.exit.code`,
  `process.pid`, `process.command_args`, `process.executable.path`, and
  `error.type`.
- Propagation helpers/config: `BaggagePropagator`,
  `ParentBased(TraceIdRatioBased)`, `Context::current()`, gRPC
  `MetadataInjector`, Kafka headers, RabbitMQ `BasicProperties.headers`,
  W3C `traceparent`, `tracestate`, and `baggage`.
- GraphQL Java flags: set
  `otel.instrumentation.graphql.data-fetcher.enabled=true` and
  `otel.instrumentation.graphql.data-fetcher.create_or_add_link=true` for field
  spans and operation links. Also keep GraphQL concepts `graphql.request`,
  `graphql.execute`, `graphql.fetch`, `graphql.dataloader.load`,
  `graphql.dataloader.batch.size`, `graphql.error.path`, and
  `graphql.document`.
- gRPC streaming details: `rpc.system="grpc"`, `rpc.message.type`,
  `rpc.message.id`, `rpc.message.compressed_size`, `rpc.grpc.status_code`,
  `QuoteStream`, and `tonic-tracing-opentelemetry` are concrete playground/UI
  references.
- Database/pool metrics should spell out exact names where possible:
  `db.client.operation.duration`, `db.client.connection.count`,
  `db.client.connection.pending_requests`, `db.client.connection.timeouts`,
  `db.client.connection.wait_time`, `db.client.connection.use_time`,
  `db.client.connection.create_time`, `db.client.connection.idle.max`,
  `db.client.connection.idle.min`, and `db.client.connection.max`.
- Java runtime/config hooks: `spring-boot-starter-actuator`,
  `hikaricp.connections.*`, `otel.instrumentation.micrometer.enabled=true`,
  `jvm.gc.time`, `jvm.gc.count`, `jvm.threads.count`, `jvm.memory.used`,
  `jvm.class.loaded`, `jvm.cpu.time`, and `jvm.buffer.pool.*`.
- Rust runtime hooks: `tokio-metrics`, `RuntimeMonitor`, `TaskMonitor`,
  `tokio.runtime.*`, `tokio.task.*`, `tokio.runtime.alive_tasks`,
  `tokio.runtime.worker_count`, `workers_count`, `alive_tasks`,
  `blocking_pool_depth`, `budget_forced_yield_count`,
  `io_driver_ready_count`, `poll_count_histogram`, `schedule_wait_duration`,
  `task.polls`, `instrumented_count`, `dropped_count`, `first_poll_delay`,
  `total_poll_duration`, `total_schedule_duration`, `total_idle_duration`, and
  `mean_poll_duration`.
- Profiling placeholders: `opentelemetry_profiles`, `pprof-rs`, JFR,
  `async-profiler`, `tracing-alloc`, `profiles(service, from, to, traceId?)`,
  and `flamegraph(profileId, groupBy=function|file|module)`.
- Specific future resolvers/actions: `serviceMap(from, to, env?)`,
  `linkedTraces(traceId, depth=2)`, `agentSteps(runId)`,
  `logFacets(query, fields[])`, `metricExemplars(name, from, to, ...)`,
  `sloBurn(sloId, from, to)`, `traceCompare(a, b)`,
  `traceCriticalPath(traceId)`, `bundleDiff(a, b)`, `frontendSessions(...)`,
  `frontendSession(id)`, `frontendErrors(...)`, `profiles(service, from, to,
  traceId?)`, `flamegraph(profileId, groupBy=function|file|module)`,
  `bundle(fingerprint?|runId?|traceId?, maxTokens?)`, `run(runId)`,
  `trace(traceId)`, `logs(...)`, and `metricSeries(name, runId=...)`.
- Scenario/example literals worth preserving for future search: `?deep=6&fan=5`,
  `?release=`, `?tenant=`, `POST /v1/deploys`, `REGRESSED`,
  `missing_backend_continuation`, `missing_evidence: sampled_out_trace`,
  `frontend_error_caused_by_backend`, `route_view`, `user_step`,
  `user_interaction`, `tenant.id`, `user.tier`, `cart.id`,
  `request.size_bytes`, `db.statement_count`, `correlation.id`,
  `catalogPromo`, `Quote`, `QuoteStream`, `products { reviews }`, and
  `products { id name reviews { text stars } }`.
- Legacy implementation/source literals that were useful in the raw brainstorm:
  `parallax run start`, `pinnedContext`, `opentelemetry_traces`,
  `opentelemetry_logs`, `opentelemetry_profiles`,
  `rollups_service_edge_minute`, `playground_telemetry`,
  `profile-collector`, `playground daemon`, `playground enter`,
  `playground enter <session>`, `playground agent`, `POST /inventory/reserve`,
  `dev.tailrocks.checkout.CheckoutHandler#handle`, `network.peer.address`,
  `peer.address`, `peer.service`, `status.code`, `otel.kind`, `otel.name`,
  `http.server.request.duration`, `http.client.connection.*`,
  `process.cpu.time`, `process.memory.utilization`, `db.response.status_code`,
  `db.system.name="redis"`, `url.query`, `inferred_skew_ms`, and
  `outcome=timeout`.

### Source pointer map retained from the removed brainstorm

Local research constraints:

- `docs/research/architecture/simple-ui-v2.md`
- `docs/research/architecture/api-concept.md`
- `docs/research/architecture/causal-reconstruction.md`
- `docs/research/architecture/evidence-bundle-schema.md`
- `docs/research/architecture/integration-contract.md`
- `docs/research/architecture/trace-linking.md`
- `docs/research/capture/rust-stack-instrumentation.md`
- `docs/research/capture/frontend.md`
- `docs/research/capture/agent-cli-tracing.md`
- `docs/research/capture/run-id-standardization.md`
- `docs/research/capture/correlation.md`
- `docs/research/decisions/native-otel-tables.md`
- `docs/research/decisions/storage-engine.md`
- `docs/research/decisions/metadata-store.md`
- `docs/research/market/observability-feature-matrix.md`
- `docs/research/market/backend-and-data-flow.md`
- `docs/research/market/closest-to-parallax-ranked.md`
- `docs/research/validation/telemetry-playground-sample-project.md`

Historical code pointers (paths may no longer exist; current agents must locate
the owning feature/crate rather than use these as implementation pointers):

- `ui/src/routes/traces.$traceId.tsx:65`
- `ui/src/components/console/trace-waterfall.tsx:22`
- `ui/src/components/logs-table.tsx`
- `ui/src/components/metric-strip.tsx:34`
- `ui/src/routes/runs.$runId.tsx:83`
- `ui/src/routes/issues.$fingerprint.tsx:75`
- `ui/src/routes/dashboards.$dashboardId.tsx:73`
- `ui/src/routes/sql.tsx:34`
- `crates/parallax-api/src/lib.rs:879-1926`
- `crates/parallax-core/src/derive.rs:18`
- `crates/parallax-core/src/fingerprint.rs:54`
- `crates/parallax-core/src/bundle.rs:15-37`
- `parallax-telemetry-playground/libs/playground-telemetry/src/lib.rs:61-66`
- `parallax-telemetry-playground/libs/playground-telemetry/src/lib.rs:111-125`
- `parallax-telemetry-playground/services/checkout/src/main.rs:27-63`
- `parallax-telemetry-playground/services/checkout/src/main.rs:221-230`
- `parallax-telemetry-playground/services/catalog/src/main/resources/application.yml`
- `parallax-telemetry-playground/services/catalog/src/main/java/dev/tailrocks/catalog/CatalogApplication.java:46-92`
- `parallax-telemetry-playground/services/catalog/src/main/java/dev/tailrocks/catalog/CatalogApplication.java:51-57`
- `parallax-telemetry-playground/services/orders/src/main.rs:62`
- `parallax-telemetry-playground/services/pricing/src/main.rs:38`
- `parallax-telemetry-playground/cli/src/main.rs:45-58`
- `parallax-telemetry-playground/web/src/telemetry.ts:23-45`
- `parallax-telemetry-playground/web/src/routes/__root.tsx:15`
- `parallax-telemetry-playground/deploy/docker-compose.yml`
- `parallax-telemetry-playground/deploy/docker-compose.xlang.yml`

External references retained for follow-up research:

- OpenTelemetry overview, traces, logs data model, metrics data model, baggage,
  profiles, semantic conventions, CLI spans, GraphQL spans, browser events,
  browser JS instrumentation, GenAI, MCP, env-var context carriers, Weaver, and
  OBI/eBPF instrumentation.
- W3C Trace Context and W3C Baggage.
- Grafana Explore, Tempo service graph, Tempo metrics-from-traces/exemplars, and
  Faro browser observability.
- Sentry Trace Explorer, distributed tracing, releases/regressions, breadcrumbs,
  frontend sessions, and profiling.
- Elastic Discover, service maps, and Cases.
- Datadog Service Map, Service Catalog/Page, runtime metrics, and deploy
  markers.
- Honeycomb high-cardinality exploration, BubbleUp, and Agent Timeline.
- Jaeger trace DAGs, deep dependency graphs, service performance monitoring, and
  trace comparison.
- Apollo Studio GraphQL field tree patterns.
- Tokio metrics and Tokio Console.
- Pyroscope/profile flamegraph patterns.
- Coroot eBPF service map research.
- AI-observability references: Langfuse, Arize Phoenix/OpenInference,
  Braintrust, LangSmith, Galileo, MLflow, Comet Opik.

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
8. **No stack expansion by default.** Use Rust, Java, TypeScript, GreptimeDB,
   Turso, TanStack, shadcn, OpenTelemetry, Sentry path only where already scoped.
9. **Local-first remains sacred.** One binary/local workflow should stay simpler
   than self-hosted Sentry and less fragmented than Grafana+Kibana.

## Retired Execution Order And Current Ownership

The former sequence covered inventory, Ecosystem, span links, Story, attribute
compare, execution-stack scenarios, evidence gaps, and dashboards. The first
six product surfaces largely shipped through retired plans; this file no longer
orders follow-up work. Plans 100/105/119/122 are **closed/deleted**. Residual
UI/playground ownership is only active numbered `plans/*.md` (e.g. 154/155);
this file is design history.

## Historical Design Questions

These questions are historical design prompts (plan 122 closed); revalidate
only under an active plan (e.g. 154) or leave as research questions.

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
- What evidence goes to an agent?

The current codebase has enough primitives to start. The next research-backed UI
move is an ecosystem graph plus story timeline, fed by OpenTelemetry semantics,
span links, run ids, and visible evidence gaps. The playground should evolve from
"polyglot shop demo" into "polyglot shop + CLI/container/agent execution lab" so
Parallax demonstrates a category Sentry/Grafana/Kibana do not cover as one
coherent product.
