# Telemetry Playground — Maximum-Fidelity OTel + Sentry Sample Stack

Research date: 2026-06-23
Status: design proposal (no code yet)
Relationship: feeds the [OTLP Fan-Out Comparison Lab](otlp-fanout-comparison-lab.md).
The lab is the *plumbing* (one stream → many backends via Rotel); this playground
is the *payload* — a realistic polyglot app instrumented to the maximum so every
backend (Parallax, Maple, SigNoz, OpenObserve, Sentry) receives identical,
feature-complete telemetry and we can compare how each renders it.

> Deep-review status: all version/API claims below were verified against live
> 2026 sources; corrections from that review are folded in. Items that depend on
> unbuilt Parallax features or non-stable upstream signals are marked
> **[NOT YET IMPLEMENTED]** or with a stability tag.

## 1. Why this exists

We are building Parallax. To know what "good" looks like — and what we must beat —
we need one application that *emits every signal a modern observability stack can
produce*, across our exact technologies, instrumented the way the industry
actually does it (OpenTelemetry **and** Sentry together). We fan it out through
Rotel to every competitor and to Parallax, then compare the rendered result.

Two hard requirements shape everything:

1. **Maximum fidelity.** Distributed traces, logs, metrics, exemplars, span
   links, exceptions with stack traces, profiles, RUM/web-vitals, session replay,
   issue grouping, source maps, feature-flag context — exercise
   the whole menu.
2. **Only our stack.** Nothing outside the locked list. Within it, latest stable
   versions and current best-practice setup.

And — per operator, 2026-06-23 — the app is a set of **distinct services in
different languages that cross-communicate richly** (some Rust, some Java, one TS
frontend, one Rust CLI). We do **not** reimplement the same service per language;
we build a real heterogeneous system, modeled on proven reference demos.

## 2. Prior art we model on

These are the canonical observability demo apps. We borrow their shapes and their
deliberate-failure catalogs, restricted to our stack.

- **OpenTelemetry Demo ("Astronomy Shop")** — the canonical polyglot,
  Apache-2.0, full-signal. Distinct services per language (incl. a **Rust**
  Shipping service and **Java/Kotlin** Ad/Fraud services), gRPC service-to-service
  + HTTP at the edge + **Kafka** async branch, an **OpenFeature/flagd** flag
  system with a flag UI, a **Locust** load generator, and the richest catalog of
  flag-driven failure scenarios (service failure, GC pauses, high CPU, memory
  leak, cache leak, kafka lag, slow load, LLM faults). Its trace shape — *edge
  fan-out → deep synchronous orchestrator → async broker branch* — is exactly the
  payload shape we want. [demo](https://github.com/open-telemetry/opentelemetry-demo) ·
  [services](https://opentelemetry.io/docs/demo/services/) ·
  [feature flags](https://opentelemetry.io/docs/demo/feature-flags/)
- **Sentry "Empower Plant"** (`sentry-demos/empower`) — Sentry's flagship demo;
  shows issues, releases/release-health, session replay, profiling, source maps,
  distributed tracing, user feedback, crons, rage-clicks, Statsig feature flags,
  and a Test-Data-Automation load driver. We borrow its **deterministic,
  parameterized fault injection** (query-param/flag toggles), per-demoer data
  segmentation, and the **crons** probabilistic success/fail/stuck pattern.
  [empower](https://github.com/sentry-demos/empower) ·
  [distributed-tracing-examples](https://github.com/getsentry/distributed-tracing-examples)
- **Jaeger HotROD** — the best *teaching* demo for trace pathologies: context
  propagation, worker-pool latency waves, **mutex/connection-pool contention**,
  **N+1 sequential calls**, and **baggage** naming which request a span queues
  behind. We steal these micro-pathologies as scenarios.
  [hotrod](https://github.com/jaegertracing/jaeger/tree/main/examples/hotrod)
- Also reviewed: Google **Online Boutique** (gRPC shop, Java adservice),
  **Grafana intro-to-mltp** (four-signal correlation incl. trace→profile span
  profiles + Faro RUM + Beyla eBPF), **Datadog Storedog** and **Elastic
  Elastiflix** (flag-driven chaos), **Honeycomb greeting-service**
  (cross-language propagation), **DeathStarBench** (scale).

What we deliberately do **not** copy: Empower's "same backend reimplemented per
language" approach — the operator wants distinct services. We copy its
*fault-injection mechanics*, not its topology.

## 3. The locked stack

| Tier | Technology | Instrumentation |
|---|---|---|
| **Frontend** | TypeScript + **TanStack Start** (React, Vite/Nitro) | OTel Web SDK + `@sentry/tanstackstart-react` |
| **Backend (Rust)** | **Rust + Axum** (HTTP) + **tonic** (gRPC) services | `tracing` + `tracing-opentelemetry` + `opentelemetry-otlp` + `sentry` + `sentry-opentelemetry` |
| **Backend (Java)** | **Spring Boot + GraphQL** and **Spring Boot + gRPC** services | OTel Java agent via **`sentry-opentelemetry-agent`** + Sentry Spring Boot starter |
| **Client / driver** | **Rust CLI** (traffic driver, flag toggler, cron job) | Rust stack, tuned for short-lived processes |
| **Edges** | Postgres (DB spans), a message broker (producer/consumer spans + links), an OpenFeature flag daemon, a load generator | auto-instrumented per language |

## 4. Core principle: two ingest paths, run in parallel

No single protocol carries every feature. The playground emits **both**, always:

- **OpenTelemetry → OTLP → Rotel → all backends.** The portable path: traces,
  logs, metrics, exemplars, span links, profiles, distributed context. What every
  backend (incl. Parallax) consumes; the apples-to-apples comparison surface.
- **Sentry SDK → Sentry envelope (DSN) → Sentry, directly.** The product path:
  managed **issues** (grouping/lifecycle), **breadcrumbs**, **releases +
  regression tracking**, **source maps / source context**, **session replay**,
  **profiling**, **user feedback**, **release health**, **feature-flag context**.
  Sentry-only objects that OTLP cannot represent. (Sentry *also* has OTLP ingest —
  traces + logs, **no metrics**, open beta — exercised only as a secondary
  comparison; the SDK/envelope path is primary because it unlocks the features
  above.)

> **Parallax note (operator, 2026-06-23):** Parallax does not consume the Sentry
> envelope yet, and that's fine — the *sample stack* must still run OTel **+**
> Sentry because that is the industry norm. Parallax is judged on the OTLP path
> today; the envelope path is the bar Parallax's future Sentry-compatibility
> wedge must clear.

**The non-negotiable engineering rule: OpenTelemetry owns the spans; Sentry rides
along.** One OTel SDK per process is the single span source; the Sentry layer
consumes/correlates. This avoids double-instrumentation across all three
languages. Per-language reality (corrected in the deep review):

- **TS/Node/Bun:** Sentry SDK v8+ (v10 line) is *built on OpenTelemetry*. Use
  custom setup (`skipOpenTelemetrySetup: true`) + `SentrySampler` /
  `SentrySpanProcessor` / `SentryPropagator` / `SentryContextManager` so we own
  the tracer and fan out, and Sentry rides the same spans.
- **Java:** run **one agent** (`sentry-opentelemetry-agent`) with
  `SENTRY_AUTO_INIT=false`, Spring Boot starter inits the SDK. SDK version ==
  agent version, exactly.
- **Rust:** a single `tracing` subscriber feeds `OpenTelemetryLayer` +
  `MetricsLayer` + the logs bridge + `sentry-tracing` as parallel consumers.
  **Correction:** `sentry-tracing` alone makes *separate* Sentry transactions
  with a *different* trace_id; to share one trace_id with OTLP you must add the
  **`sentry-opentelemetry`** crate (the Rust OTel `SpanProcessor`/propagator).

## 5. Architecture — distinct services, one trace

A small but realistic **e-commerce "shop"** domain (proven by OTel Demo / Online
Boutique). Distinct services; language chosen by what each best demonstrates.

```
                              ONE DISTRIBUTED TRACE (W3C traceparent + baggage)

  web  (TanStack Start, TS)  ──HTTP──►  checkout (Rust axum/tonic) ── deep orchestrator
   │  RUM, replay, web vitals               │  ├─ gRPC ─► pricing      (Rust tonic)   high-QPS, histograms
   │                                        │  ├─ gRPC ─► inventory    (Rust tonic)   DB-heavy, N+1, pool contention
   │  GraphQL (browser→catalog)             │  ├─ GraphQL ─► catalog   (Java Spring GraphQL)  resolvers, subs
   │                                        │  ├─ gRPC ─► payment      (Java Spring gRPC)  JVM GC/CPU, failures
   │  SSE/WebSocket (order status) ◄────────┤  └─ HTTP ─► recommendation (Rust axum)  cache-leak
   │                                        │
   │                                        └─ publish ─► broker (Kafka/NATS)
   │                                                          │ consume (span link)
   │                                                          ▼
   │                                                     fulfillment (Java Spring consumer)
   │                                                          └─ HTTP ─► notifications (Rust axum)  ◄ REVERSE Java→Rust hop
   │
  flagd (OpenFeature daemon) + flag UI       loadgen (k6 / telemetrygen)
  cli (Rust, short-lived) ──HTTP──► checkout   drives traffic, toggles flags, runs cron job,
                                               stamps parallax.run.id
  Postgres (per service)   ← DB spans everywhere
```

### Services

| Service | Lang | Transport(s) | Showcases | Built-in failure modes |
|---|---|---|---|---|
| `web` | TS / TanStack Start | HTTP/GraphQL client, SSE/WS | RUM, web vitals, session replay, source maps, browser→server trace handoff | rage-clicks, frontend slowdown, error boundary |
| `checkout` | Rust axum + tonic | HTTP in; gRPC/GraphQL/HTTP out; broker producer | deep orchestrator (trace spine), CLIENT+SERVER spans, baggage origin, **PRODUCER** span | cascading failure, deadline/timeout + retry, partial degradation |
| `pricing` | Rust tonic | gRPC server (incl. **server-streaming**) | high-QPS counters/histograms, streaming spans | latency knob, high CPU |
| `inventory` | Rust tonic | gRPC server | DB CLIENT spans (`db.system.name`/`db.query.text`), **N+1**, **connection-pool contention** | slow query, pool exhaustion |
| `catalog` | Java Spring **GraphQL** | GraphQL server (queries + **subscription**) | per-resolver/data-fetcher spans, DataLoader/N+1, GraphQL partial errors, subscription long-lived spans | resolver error, N+1 storm |
| `payment` | Java Spring **gRPC** | gRPC server | Java gRPC server spans, JVM runtime metrics, **exemplars** (Micrometer) | **GC pauses**, high CPU, hard failure, unreachable |
| `recommendation` | Rust axum | HTTP server, calls `catalog` | cache-backed reads | **cache leak** (exponential per-request), stampede |
| `fulfillment` | Java Spring | broker consumer; HTTP client | **CONSUMER** span + **span link** to producer; cross-language async branch | **poison message** (repeated redelivery), consumer lag |
| `notifications` | Rust axum | HTTP server (called by Java `fulfillment`) | **reverse Java→Rust** propagation, HTTP leaf | timeout, 5xx |
| `flagd` | OpenFeature daemon (+UI) | gRPC | feature-flag evaluation as the chaos toggle mechanism | — (the control plane) |
| `loadgen` | k6 / `telemetrygen` | OTLP + HTTP | reproducible volume, synthetic OTLP | traffic flood |
| `cli` | Rust (short-lived) | HTTP client | short-lived process telemetry, flush-on-exit, `process.*`, **`parallax.run.id`** stamping, cron job | stuck/missed cron, nonzero exit |

This shape gives every span kind, both call directions (incl. **Java→Rust** via
`fulfillment → notifications`), GraphQL + gRPC (unary **and** streaming) + HTTP +
messaging + DB, and a flag-driven chaos control plane.

## 6. Cross-language distributed tracing

One trace must span Browser → Rust → Java → broker → Java → Rust, both
directions. Mechanism is uniform: **W3C Trace Context** (`traceparent` +
`tracestate`) on every hop; **W3C Baggage** for per-request business context
(`tenant.id`, `user.tier`, `cart.id`). Every service sets
`propagators = tracecontext,baggage` explicitly. (Note: W3C Baggage is a *W3C
Candidate Recommendation*, still the OTel default — fine, just not full-Rec
maturity.)

| Hop | Carrier | Inject (client) | Extract (server) | Notes |
|---|---|---|---|---|
| browser → checkout | HTTP headers | OTel-web fetch instr. | axum middleware | CORS must allow `traceparent,tracestate,baggage` or headers silently drop |
| SSR → browser (first paint) | `<meta name="traceparent">` | TanStack server middleware emits | `instrumentation-document-load` reads | initial nav has no fetch to inject into; this is the *documented* pattern (a `Server-Timing` alternative also exists) |
| checkout → pricing/inventory | gRPC metadata | tonic interceptor | tonic | Rust→Rust |
| checkout → payment | gRPC metadata | tonic interceptor | OTel Java agent | Rust→Java gRPC |
| checkout → catalog | HTTP headers (GraphQL) | reqwest `HeaderInjector` | OTel Java agent | Rust→Java GraphQL |
| checkout → broker → fulfillment | message headers | producer injects | consumer extracts + **span link** | async; cross-language |
| **fulfillment → notifications** | HTTP headers | OTel Java agent | axum middleware | **reverse Java→Rust hop** |
| cli → checkout | HTTP headers | reqwest injector | axum middleware | short-lived root; carries `parallax.run.id` (resource attr) |

**Sentry interop:** all SDKs share one `trace_id` with OTLP — but note Sentry's
default carriers are `sentry-trace` + `baggage`; **W3C `traceparent` emission is
opt-in** (browser `propagateTraceparent`, default false; Java SDK 8.22+). Enable
it so the Sentry transaction tree and the OTLP trace share IDs.

## 7. Parallax purpose-fit — the signals that actually test Parallax

A generic obs payload would miss the four things that make Parallax *Parallax*.
These are first-class requirements, not nice-to-haves (from the adversarial
review against the thesis, evidence-bundle schema, run-id, and redaction notes):

1. **`parallax.run.id` run-scoping.** Parallax's mandatory correlation key — if
   absent, telemetry is *not run-scoped*, and Parallax does **not** accept
   `session.id`/`cicd.pipeline.run.id` as aliases. The `cli` and `parallax run
   start` must stamp `parallax.run.id` as a **resource attribute** (run-lifetime,
   not per-request baggage) across all child traces/logs/metrics. **Comparison
   value:** how does each backend render a custom resource attribute that ties N
   traces into one run? This is where Parallax should win and competitors show
   nothing. See [run-id standardization](../capture/run-id-standardization.md) +
   [agent/CLI tracing](../capture/agent-cli-tracing.md).
2. **Evidence-bundle inputs (deploy / commit / CI / work-item).** Parallax's
   product is a cross-source evidence graph ("error rate increased after release
   X"). The playground must emit not just live OTLP but **deploy/release markers,
   a commit sha, a CI run id, and a work-item ref**, and run a
   **regression-after-deploy** scenario (v2 introduces a panic) so
   `deploy_precedes_regression` becomes testable. Without this, the playground
   shows telemetry but cannot exercise Parallax's actual differentiator. See
   [evidence-bundle schema](../architecture/evidence-bundle-schema.md).
3. **Seeded-canary redaction.** Not one PII field — a **canary corpus** (provider
   tokens, private keys, DB URLs, JWTs, cookies, auth headers, emails, phones,
   IPs, payment-like numbers) planted across **span attributes, log bodies,
   exception messages, `db.query.text`, baggage, and the GraphQL document**, then
   compare what each backend stores raw vs scrubs. Feeds Parallax's
   redaction-at-ingest / canary gate. See [redaction](../capture/redaction.md).

Each is a scenario in §11; together they convert the playground from "shows
telemetry in five UIs" to "proves where Parallax's evidence layer beats raw OTLP
backends."

## 8. Per-component instrumentation spec

Versions are **known-good floors (June 2026)** — per repo policy, resolve latest
mutually-compatible stable at impl and pin. Corrections from the deep review are
inline.

### Frontend — TanStack Start (TypeScript)

- **OTel (portable):** `@opentelemetry/sdk-trace-web` 2.8, `context-zone`,
  `exporter-trace-otlp-http` 0.219, instrumentations `-fetch`/`-xml-http-request`
  (0.219) / `-document-load` (0.64) / `-user-interaction` (0.63), `core`
  composite W3C propagators.
- **Sentry (product RUM):** `@sentry/react` 10.59 + `@sentry/tanstackstart-react`
  (beta, v10) + `@sentry/vite-plugin` **5.3.0 (pin exactly)**; integrations
  `replayIntegration`, `browserTracingIntegration` (LCP/CLS/INP/FCP/TTFB),
  `feedbackIntegration`, `consoleLoggingIntegration` (needs
  `_experiments.enableLogs`), `tanstackRouterBrowserTracingIntegration`,
  `openFeatureIntegration` (feature-flag context).
- **One owner of the global OTel SDK** (the app); Sentry browser tracing coexists.
- **Export:** browser → **same-origin `/v1/traces` proxy** → Rotel (avoids
  collector CORS, hides endpoint, allows auth). Add a **CSP `connect-src`** entry
  for the proxy *and* the Sentry DSN host (or use Sentry `tunnel`). SSR /
  server-function spans go through the Node-side OTel SDK to Rotel.
- **Source maps:** Sentry **Debug IDs** (`sentry-cli sourcemaps inject` + the
  Vite plugin), verified in CI with `sentry-cli sourcemaps explain <event>` —
  not the legacy release+dist flow.
- **Bun caveats (TS *server* tier only):** `requestHook`s don't fire (generic
  span names) — use manual spans; `Bun.serve`/`bun:sqlite` not auto-instrumented
  — wrap manually; disable `instrumentation-fs` (crashes); `@sentry/profiling-node`
  is a native addon that won't load on Bun (v10 `@sentry/node-cpu-profiler`) —
  don't rely on profiling there. Consider the dedicated **`@sentry/bun`** SDK
  (`bunServerIntegration`) for the server tier.
- **Gotchas:** `ZoneContextManager` needs ES2015 transpile; `propagateTraceparent`
  is **opt-in**; web-vitals must flush on `visibilitychange`; browser OTel logs
  experimental (use Sentry logs); JS metric **exemplars are experimental** — don't
  promise web-vital→trace exemplars from the browser; TanStack **Query has no
  dedicated Sentry integration** (its fetches are caught by HTTP instr.).
- **Unlocks:** document-load/resource/fetch/XHR/user-interaction spans; browser→
  backend distributed trace; web vitals; session replay; **release health**
  (auto session tracking + `setUser`); source-mapped React error boundaries
  (prefer per-route boundaries); breadcrumbs; releases; user feedback;
  feature-flag context.

### Rust services — Axum + tonic (`checkout`, `pricing`, `inventory`, `recommendation`, `notifications`)

- **Crates (floors):** `tracing`, `tracing-subscriber` 0.3, `tracing-opentelemetry`
  0.33, `opentelemetry`/`opentelemetry_sdk` 0.32, `opentelemetry-otlp` 0.32
  (**enable `grpc-tonic` + a TLS feature explicitly** — 0.32 changed defaults to
  `http-proto`), `opentelemetry-semantic-conventions` 0.32,
  `opentelemetry-appender-tracing` 0.32 (logs bridge), `sentry` 0.48.2 +
  `sentry-tracing`, **`sentry-opentelemetry` 0.48** (shared trace_id),
  **`sentry-tower`** (request-scoped hubs — tonic rides on tower; **there is no
  `sentry-tonic`**), `sentry-anyhow` (error-chain capture), `tracing-error`
  `ErrorLayer` (`SpanTrace`).
- **Single subscriber, parallel consumers:** `registry().with(fmt)
  .with(OpenTelemetryLayer).with(MetricsLayer).with(OpenTelemetryTracingBridge)
  .with(ErrorLayer).with(sentry::integrations::tracing::layer())`. `tracing` is
  the only span source. Add `sentry-opentelemetry`'s span processor/propagator so
  Sentry issues and OTLP traces carry the **same** trace_id.
- **Traces:** `#[instrument]` (`skip_all`/`fields`/`err`/`ret`), `otel.kind`/
  `otel.status_code` reserved fields, span events via `event!`.
- **Logs:** `tracing` events → OTLP logs, auto-stamped trace/span id; filter
  `hyper/tonic/h2/reqwest/opentelemetry` out of the log layer (self-logging loop).
- **Metrics:** counters/histograms/gauges over OTLP. **Exemplars correction: the
  Rust SDK does NOT implement metric exemplars yet** (open issue #3369 — every
  aggregator emits empty exemplars). So metric→trace exemplar jumps come from the
  **Java tier**, not Rust. From Rust, either skip exemplars or attach `trace_id`
  as a sampled high-card attribute (document the asymmetry).
- **Propagation:** global `TraceContextPropagator` + baggage; reqwest
  `HeaderInjector` / axum `HeaderExtractor`; tonic interceptor on metadata.
- **Errors:** `attach_stacktrace: true` + `RUST_BACKTRACE=1`; `sentry-anyhow` for
  full error chains; exception span events (`exception.type/message/stacktrace`) +
  `otel.status_code=ERROR` for OTLP; panics + `Result::Err` both ways.
- **Sampling:** `ParentBased(TraceIdRatioBased)`, ratio pinned per run (100% for
  lab repeatability). Head-sampling only in-process; tail-sampling belongs at the
  collector (§13/§14).
- **Runtime health (extension):** `opentelemetry-instrumentation-tokio` (tokio
  `unstable`) for task/scheduler metrics; optional `tokio-console` for dev.
- **Profiling:** OTel profiling signal has no Rust impl → Rust profiles come via
  Sentry continuous profiling (pprof-based), not OTLP.

### Rust CLI (`cli`)

- Same stack, **short-lived tuning**: wrap the invocation in a root span; record
  `process.command_args`, subcommand, `process.exit.code`; stamp
  **`parallax.run.id`** as a resource attribute; prefer a simple/synchronous
  exporter (or batch + guaranteed flush); **force-flush providers before exit**
  (call shutdown off the current-thread runtime to avoid the known deadlock);
  hold the Sentry guard for the whole program. Also drives flag toggles and the
  cron job (probabilistic success/fail/stuck).

### Java — Spring Boot + GraphQL (`catalog`) and Spring Boot + gRPC (`payment`, `fulfillment`)

- **Spring Boot 4 GA** (4.1.x current in 2026; 3.x is maintenance). **One agent
  for both OTel and Sentry:** run **`io.sentry:sentry-opentelemetry-agent`** 8.44.x
  as `-javaagent` with `SENTRY_AUTO_INIT=false`; **`sentry-spring-boot-starter-jakarta`**
  (via `sentry-bom`) inits the SDK. The agent bundles upstream OTel
  auto-instrumentation (150+ libs incl. Spring GraphQL, grpc-java, JDBC, Logback).
  **Version lock: SDK == agent, exactly** (throws on mismatch since 8.6). OTel
  Java agent baseline is 2.29.0.
- **GraphQL variant (`catalog`):** `spring-boot-starter-graphql`; agent
  auto-instruments graphql-java → per-operation spans + `graphql.operation.name/
  type`, sanitized `graphql.document`. **Corrections:** per-**data-fetcher** spans
  are **opt-in** (`otel.instrumentation.graphql.data-fetcher.enabled=true`,
  graphql-java 20+); the sanitization flag is
  `otel.instrumentation.graphql.query-sanitization.enabled` (default on — keep it,
  high-cardinality/PII risk). **Partial errors** (HTTP 200 + `errors[]`): add a
  `DataFetcherExceptionResolver` that calls `Sentry.captureException`. Add a
  **subscription** for long-lived span shapes.
- **gRPC variant (`payment`):** **Spring gRPC 1.1 GA**
  (`org.springframework.grpc:spring-grpc-spring-boot-starter`) + grpc-java; agent
  auto-instruments server+client with W3C over gRPC metadata → interops with Rust
  tonic both directions.
- **Logs:** agent bridges Logback → OTLP with trace correlation; MDC pattern
  surfaces ids in console. Spring Boot 3.4+/4 **built-in structured logging**
  (`logging.structured.format` ECS/Logstash/GELF) — use it.
- **Metrics + exemplars:** Micrometer/Actuator → OTLP (enable the agent
  micrometer bridge); JVM runtime metrics; **exemplars work here** via Micrometer
  (`management.tracing.exemplars.include`) — this is the playground's real
  exemplar source.
- **Errors:** exception span events with full Java stack traces; **`SentrySpanProcessor`
  NOOPs `recordException`/`addEvent`**, so explicitly `Sentry.captureException`
  (in `@ExceptionHandler` / GraphQL error resolvers) for reliable issues.
  **Source context** (readable Java stacks in Sentry) via the Sentry Gradle/Maven
  plugin (`includeSourceContext = true`).
- **Profiling:** Sentry JVM continuous profiling is **async-profiler**-based
  (`sentry-async-profiler`, GA since SDK 8.23) — covers the CPU-hot-path scenario
  on the JVM (not JFR, not OTLP profiles).
- **Resource:** `service.name` from `spring.application.name`,
  `deployment.environment.name`, `service.namespace/version/instance.id`.

## 9. Telemetry feature-coverage checklist

The acceptance checklist — the playground must produce all of it (semconv
stability corrected per deep review):

- **Traces:** spans; all five span kinds (CLIENT/SERVER/PRODUCER/CONSUMER/
  INTERNAL); span status (Unset/Ok/Error); span events; **span links**; rich
  attributes (semconv + custom + a high-card stress attr); deep waterfalls;
  **streaming spans** (gRPC streaming, GraphQL subscription, SSE/WS — long-lived).
- **Propagation:** W3C trace context every hop; W3C baggage; sampling-flag
  propagation.
- **Logs:** OTLP LogRecords; `SeverityNumber` + `SeverityText`; trace correlation;
  **exception-as-log** path via the **Logs API** (the post-deprecation direction)
  *and* the legacy span-event path — emit both with
  `OTEL_SEMCONV_EXCEPTION_SIGNAL_OPT_IN=logs/dup` (span-event API deprecation
  announced 2026-03-17); instrumentation scope.
- **Metrics:** Counter, UpDownCounter, Histogram (explicit + exponential
  buckets), async Gauge; **exemplars** (Java/JVM tier — **not Rust**, not browser);
  Views; **SDK cardinality limit** (stable; default 2000 sets/metric → surface
  `otel.metric.overflow`). *(No `cardinalitylimitprocessor` exists in the
  collector — don't reference one.)*
- **Profiles [Alpha]:** OTel **profiling signal** (4th signal, public alpha
  2026-03-26) where a runtime supports it; plus Sentry continuous profiling
  (Rust pprof / JVM async-profiler). Profiles correlate to traces (`trace_id`/
  `span_id` → click span → flamegraph).
- **Exceptions:** `exception.type/message/stacktrace/escaped`; `error.type`; span
  status = Error.
- **Resource / semconv (stability):** `service.{name,version,namespace,instance.id}`
  + `deployment.environment.name` (**Stable**); HTTP, general/network,
  **`db.system.name`/`db.query.text` (now Stable)**, **`code.*` (now Stable)** —
  exercise as stable; RPC/gRPC (**RC**); messaging, GraphQL, user/session,
  feature-flag (**Development**) — exercise on purpose to test how each
  backend handles non-frozen names. **Record the semconv version per run** so
  attribute-rename diffs are attributable to backend behavior vs semconv drift.
- **Feature flags [Development]:** `feature_flag.*` evaluation events
  (OpenFeature OTel hooks) + Sentry OpenFeature integration — annotate traces +
  errors with active variants.
- **Parallax-specific:** `parallax.run.id` resource attr;
  deploy/release/commit/CI/work-item markers; seeded canary corpus.

## 10. Feature flags + load generation (the control plane)

- **OpenFeature + flagd** (a dedicated service + flag UI) is the chaos toggle
  mechanism — every deliberate failure in §11.B is a flag, flipped at runtime,
  and the flag evaluation is itself telemetry (`feature_flag.*`). Mirrors the OTel
  Demo. Sentry gets the same flag context via its OpenFeature integration.
- **Load generation:** a `loadgen` service — `k6` (`--out opentelemetry`) and/or
  the OTel `telemetrygen` for synthetic OTLP — plus the Rust `cli` as a
  scriptable driver. Drives volume scenarios (§11.B #16) reproducibly.
- *(Trace-based testing note: Tracetest is abandonware as of 2026 — cite as prior
  art only; do not adopt as a dependency.)*

## 11. Scenario catalog

Two parts. **(A)** signal-generating happy/edge scenarios; **(B)** the deliberate
failure/chaos catalog (flag-toggled, adapted from OTel Demo + Sentry + HotROD +
Storedog + Elastiflix). Each row notes the **signals** and whether it produces
**Parallax-comparable** output (Sentry-envelope-only scenarios yield blanks for
Parallax — that's expected, and flagged).

### (A) Signal-generating scenarios

| # | Scenario | Signals | Parallax-comparable |
|---|---|---|---|
| A1 | Checkout flow (browser → checkout → fan-out) | full distributed trace, all span kinds, HTTP/gRPC/GraphQL semconv | Y |
| A2 | Slow DB query (`inventory`) | DB CLIENT span (`db.system.name`/`db.query.text`, **Stable**), latency Histogram + **exemplar** (recorded on the JVM caller path) | Y |
| A3 | Queue publish + async consume (`checkout`→broker→`fulfillment`) | PRODUCER + CONSUMER spans, **span link**, messaging semconv, cross-language async branch | Y |
| A4 | Reverse hop (`fulfillment` Java → `notifications` Rust) | Java→Rust W3C propagation, one trace across runtimes both directions | Y |
| A5 | Frontend interaction → backend | browser span → server span via traceparent; web vitals; session replay; end-to-end trace | Y (OTLP) / replay = Sentry-only |
| A6 | GraphQL query w/ nested resolvers + DataLoader (`catalog`) | per-resolver/data-fetcher spans (opt-in), GraphQL semconv, N+1 shape, partial errors | Y |
| A7 | gRPC **server-streaming** (`pricing`) + **GraphQL subscription** (`catalog`) + **SSE/WS** (`web`←`checkout`) | long-lived streaming spans (a known backend weak spot) | Y |
| A8 | High request volume (`loadgen`) | Counter / Histogram / UpDownCounter; cardinality-limit overflow | Y |
| A9 | Structured logging during a request | OTLP logs + severity + trace correlation; ECS/Logstash format on JVM | Y |
| A10 | Baggage business context | W3C baggage (`tenant.id`,`user.tier`) surfaced downstream | Y |
| A12 | **CLI run** end-to-end | short-lived process telemetry, `process.*`, flush-on-exit, **`parallax.run.id`** resource attr tying N traces into one run | Y (Parallax-distinguishing) |
| A13 | **Deploy + regression** (release v1 clean → v2 introduces a panic) | deploy/release marker + commit sha + CI run id + work-item ref; `deploy_precedes_regression` | Y (evidence-bundle test) |
| A14 | Feature-flag evaluation | `feature_flag.*` events on traces + Sentry flag context | Y (OTLP) + Sentry |
| A15 | Sentry-SDK error w/ breadcrumbs (envelope) | Sentry issue + breadcrumbs + release + source maps; regression tracking | **N — Sentry-only** |
| A16 | Recurring → resolved error | Sentry issue lifecycle (resolve→regress) | **N — Sentry-only** |
| A17 | CPU-heavy hot path | profiling (Sentry: Rust pprof / JVM async-profiler; OTLP profiles where alpha) | partial (OTLP profiles alpha) |
| A18 | **Canary-redaction corpus** request | canaries in span attrs, log bodies, exception msgs, `db.query.text`, baggage, GraphQL doc → compare raw-vs-scrubbed | Y (Parallax-distinguishing) |

### (B) Deliberate failure / chaos catalog (flag-toggled)

| # | Failure (flag) | Signals / what it tests | Parallax-comparable |
|---|---|---|---|
| B1 | Service hard failure / 5xx (`paymentFailure`, `catalogFailure`) | error spans, exception fidelity, issue grouping | Y (OTLP) + Sentry grouping |
| B2 | Service unreachable / conn refused (`paymentUnreachable`) | client-side error, retry behavior | Y |
| B3 | gRPC **deadline/timeout + retry** | `rpc.grpc.status_code` propagation, retry fan in waterfall | Y |
| B4 | **Cascading failure** (`pricing` down → `checkout` degraded → `orders` partial) | cross-service error propagation, partial degradation | Y |
| B5 | **JVM GC pauses / high CPU** (`paymentManualGc`, `paymentHighCpu`) | JVM runtime metrics, latency spikes, profiling | Y |
| B6 | **Memory / cache leak** (`recommendationCacheLeak`, exponential per-req) | growing metric, slow degradation over time | Y |
| B7 | **Broker overload + consumer lag** (`brokerQueueProblems`) | queue-depth UpDownCounter, consumer-lag, span links across redelivery | Y |
| B8 | **Poison message** (`fulfillment` fails repeatedly) | repeated CONSUMER spans, dead-letter, link to redelivery | Y |
| B9 | **N+1 sequential calls** (`inventory`/`catalog`) | many sibling spans (HotROD pattern) | Y |
| B10 | **Connection-pool / mutex contention** (`inventory`) | "waiting behind N" baggage, lock-wait spans | Y |
| B11 | Injected latency knob (`*ServiceDelay`) | latency histograms, slow spans | Y |
| B12 | **Canary/version failure** (`canaryFailure`) | release-health, regression-after-deploy | Y + Sentry release health |
| B13 | Slow asset / endpoint (proxy fault) | frontend slowness, resource-timing spans | Y |
| B15 | Frontend UX faults (rage-click, frustration) | RUM signals, session replay | Sentry/RUM backends |
| B16 | Loadgen flood / traffic spike | volume metrics, sampling behavior under load | Y |
| B17 | Cron/scheduled-job faults (success 90% / fail 5% / **stuck-missed-checkin** 5%) | CLI cron telemetry, missed-checkin | Y (Parallax cli) |
| B18 | **Clock skew** between two services | negative/overlapping span timing — tests how each backend handles it | Y (rendering stress) |

## 12. Comparison method — manual for now (scored harness DEFERRED)

**Out of scope for the build (operator, 2026-06-23): no automated comparison
harness / scoring rubric / per-backend extraction yet.** Comparison is **manual**:
emit a scenario, open each backend's UI, and eyeball how it renders the same data.
That is enough to decide what features to build into Parallax.

A future scored harness (per-signal rubric of preserved/renamed/dropped/mangled,
per-backend read-API extraction, pinned-id fixtures, recorded `semconv_version`)
can be added later if we want a defensible quantitative comparison — but it is
**not** part of what we build now, and it is distinct from otlp.md's L4
conformance gate.

## 13. Extensions / forward-looking

Real, current 2026 additions worth building once the core works:

- **OTel Profiling signal** (4th OTLP signal, alpha) + Collector pprof receiver —
  profiles correlated to traces; de-Sentry-izes profiling (A17).
- **eBPF zero-code** (**OpenTelemetry eBPF Instrumentation / OBI**, ex-Beyla) —
  instrument one service *both* via SDK and via eBPF, compare breadth-no-code vs
  depth-custom-spans.
- **Tail-based sampling + OTTL at the Collector** — `tailsamplingprocessor`
  (keep-errors/keep-slow on complete traces, two-layer topology with
  loadbalancing exporter) and **`redactionprocessor` / OTTL transforms** as the
  real backing for canary redaction (A18) + span-name normalization.
- **Spring Boot 4 official OTel starter** (`spring-boot-starter-opentelemetry`,
  Micrometer/Observation-based) as an A/B against the agent on one Java service;
  and a **GraalVM native-image** variant using the OTel-community starter (agent
  can't work in native images).
- **Micrometer Observation API** demo (one Observation → metric + span +
  correlated logs) on one Java service.
- **Sentry Spotlight** for local dev (inspect envelopes without sending);
  **Sentry release health** (crash-free sessions/users).
- **Reactive/Loom caveats** if any Java service uses WebFlux/R2DBC or virtual
  threads — context propagation needs `Hooks.enableAutomaticContextPropagation()`
  / Micrometer context-propagation (scoped to custom/Observation spans, not
  agent-created ones).

## 14. Repository layout, location, and lab integration

**Location (DECIDED, operator 2026-06-23): a separate repository
`tailrocks/parallax-telemetry-playground`** (Apache-2.0, Tailrocks-attributed),
**full build** (all services per the spec, not a thin slice). Rationale: polyglot
toolchains (Gradle/Maven + Cargo + Bun) should not bloat the Parallax workspace,
and it's a demo artifact, not product code.

```
parallax-telemetry-playground/
  web/                 # TanStack Start (TS) — Bun
  services/
    checkout/          # Rust axum + tonic (orchestrator)
    pricing/           # Rust tonic (streaming)
    inventory/         # Rust tonic (DB-heavy)
    recommendation/    # Rust axum (cache)
    notifications/     # Rust axum (reverse hop target)
    catalog/           # Spring Boot + GraphQL (Java)
    payment/           # Spring Boot + gRPC (Java)
    fulfillment/       # Spring Boot consumer (Java)
  cli/                 # Rust CLI (driver, cron, run.id)
  proto/               # shared .proto (pricing/inventory/payment)
  graphql/             # shared GraphQL schema (catalog)
  flags/               # flagd config + flag UI
  loadgen/             # k6 / telemetrygen
  deploy/
    docker-compose.yml # all services + Postgres + broker + flagd; OTLP → Rotel
    otel/              # shared resource attrs, sampling
  scenarios/           # scripts driving A1–A18 + B1–B18
  releases/            # v1 clean, v2 regressed (for A13/B12 regression track)
  README.md
```

**Lab integration:**

- Every service's OTLP exporter targets **Rotel**: in-compose `http://rotel:4317`;
  host processes (`cli`, dev) `http://localhost:4317`. Rotel fans out to Parallax
  (host) + Maple + SigNoz + OpenObserve + Sentry-via-OTLP.
- Every service's **Sentry SDK** points at the lab's self-hosted Sentry DSN
  (envelope path), independent of OTLP.
- Pin one `deployment.environment.name=playground` + a shared release id; for the
  regression scenario, two pinned releases (`v1` clean, `v2` regressed).
- `scenarios/` reuses the lab's pinned-id fixtures and asserts the Parallax copy
  arrived for every scenario.

## 15. Risks / open questions

- **Parallax-side prerequisites** (from the lab doc): port offset + `0.0.0.0` bind
  are config-only (built); **`parallax.run.id` child-process stamping already
  exists** (`run start` injects `OTEL_RESOURCE_ATTRIBUTES=parallax.run.id=<id>` +
  the full per-signal OTel env); what's unbuilt is the **compare-mode forward**
  (the OTLP destination is a hardcoded const — make it ambient-configurable per
  the lab's "Compare mode" DevEx section) and Parallax **self-telemetry**.
- **Rust metric exemplars don't exist** (#3369) — exemplars come from the JVM
  tier; don't promise them from Rust or the browser.
- **Sentry has no OTLP metrics; Java `SentrySpanProcessor` NOOPs span events** —
  use the envelope path + explicit `captureException`.
- **Span-event API deprecation (2026)** — emit both paths (`logs/dup`).
- **Bun OTel gaps** (TS server tier): manual spans, disable `instrumentation-fs`,
  profiling native addon won't load on Bun.
- **Beta/alpha surfaces:** `@sentry/tanstackstart-react` (beta), OTel profiling
  (alpha), feature-flag + messaging + GraphQL semconv (Development), eBPF OBI
  (pre-1.0). Pin and mark.
- **Version churn:** OTel Rust 0.32 / tracing-opentelemetry 0.33 / sentry-rust
  0.48.2 / OTel JS 2.8+0.219 / Sentry JS v10 / OTel Java agent 2.29 / Sentry Java
  8.44 / Spring Boot 4.1 / Spring gRPC 1.1 are **floors** — resolve latest
  mutually-compatible at impl and pin.
- **Resource weight:** 5 Rust + 4 JVM + broker + Postgres + flagd + the TS app,
  on top of the 5-backend lab, is heavy — full set on a server, reduced subset on
  a laptop (benchmark two-tier rule).
- **Comparison rigor depends on determinism** — pinned ids/seeds and recorded
  semconv version are mandatory, or the cross-backend diff isn't reproducible.

## 16. Suggested phasing

1. **One trace, real topology** — `web` → `checkout` (Rust) → `payment` (Java
   gRPC) + `catalog` (Java GraphQL), OTLP → Rotel → Parallax + one other backend.
   Prove browser→Rust→Java W3C stitch + assert the Parallax copy.
2. **Signal breadth** — logs, metrics+exemplars (JVM), DB spans, streaming
   (A7); Sentry envelope path everywhere; scenarios A1–A2, A5–A9, A14.
3. **Async + reverse + errors** — broker (`checkout`→`fulfillment`→`notifications`,
   reverse hop), span links, deliberate failures both languages, Sentry issues +
   source context; A3–A4, A15–A16, B1–B11.
4. **Parallax-differentiators** — `parallax.run.id` (A12), deploy+regression
   (A13), canary redaction (A18).
5. **Flags, load, full chaos, full fan-out** — flagd + loadgen, all backends up,
   run the scenario catalog; compare **manually** in each UI (no scored harness —
   §12).

## 17. Sources

- Reference demos: [OpenTelemetry Demo](https://github.com/open-telemetry/opentelemetry-demo) ·
  [demo feature flags](https://opentelemetry.io/docs/demo/feature-flags/) ·
  [Sentry Empower](https://github.com/sentry-demos/empower) ·
  [Jaeger HotROD](https://github.com/jaegertracing/jaeger/tree/main/examples/hotrod) ·
  [Online Boutique](https://github.com/GoogleCloudPlatform/microservices-demo) ·
  [Grafana intro-to-mltp](https://github.com/grafana/intro-to-mltp)
- Rust: [OTel Rust](https://opentelemetry.io/docs/languages/rust/) ·
  [tracing-opentelemetry](https://docs.rs/tracing-opentelemetry) ·
  [opentelemetry-otlp 0.32 CHANGELOG](https://docs.rs/crate/opentelemetry-otlp/latest/source/CHANGELOG.md) ·
  [exemplars issue #3369](https://github.com/open-telemetry/opentelemetry-rust/issues/3369) ·
  [sentry-opentelemetry](https://crates.io/crates/sentry-opentelemetry) ·
  [Sentry Rust](https://docs.sentry.io/platforms/rust/)
- TS/Frontend: [OTel JS](https://opentelemetry.io/docs/languages/js/) ·
  [OTel browser](https://opentelemetry.io/docs/languages/js/getting-started/browser/) ·
  [Sentry TanStack Start](https://docs.sentry.io/platforms/javascript/guides/tanstackstart-react/) ·
  [Sentry Bun](https://docs.sentry.io/platforms/javascript/guides/bun/) ·
  [Bun OTel gap #26536](https://github.com/oven-sh/bun/issues/26536)
- Java: [OTel Java agent](https://opentelemetry.io/docs/zero-code/java/agent/) ·
  [Spring Boot 4 OTel](https://spring.io/blog/2025/11/18/opentelemetry-with-spring-boot/) ·
  [Spring gRPC](https://docs.spring.io/spring-grpc/reference/) ·
  [Sentry Spring Boot + OTel](https://docs.sentry.io/platforms/java/guides/spring-boot/opentelemetry/) ·
  [GraphQL semconv](https://opentelemetry.io/docs/specs/semconv/graphql/)
- Cross-cutting: [semantic conventions](https://opentelemetry.io/docs/concepts/semantic-conventions/) ·
  [span-event deprecation](https://opentelemetry.io/blog/2026/deprecating-span-events/) ·
  [OTel Profiles alpha](https://opentelemetry.io/blog/2026/profiles-alpha/) ·
  [eBPF OBI](https://opentelemetry.io/docs/zero-code/obi/) ·
  [feature-flag semconv](https://opentelemetry.io/docs/specs/semconv/feature-flags/) ·
  [W3C Trace Context](https://www.w3.org/TR/trace-context/) ·
  [Sentry envelopes](https://develop.sentry.dev/sdk/data-model/envelopes/) ·
  [Sentry OTLP](https://docs.sentry.io/concepts/otlp/)
- Internal: [otlp-fanout-comparison-lab.md](otlp-fanout-comparison-lab.md),
  [`capture/otlp.md`](../capture/otlp.md),
  [`capture/agent-cli-tracing.md`](../capture/agent-cli-tracing.md),
  [`capture/redaction.md`](../capture/redaction.md), market deep-research for
  [sentry](../market/sentry-deep-research.md) /
  [signoz](../market/signoz-deep-research.md) /
  [openobserve](../market/openobserve-deep-research.md) /
  [maple](../market/maple-deep-research.md)
</content>
