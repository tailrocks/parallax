# Telemetry Playground — Maximum-Fidelity OTel + Sentry Sample Stack

Research date: 2026-06-23
Status: design proposal (no code yet)
Relationship: feeds the [OTLP Fan-Out Comparison Lab](otlp-fanout-comparison-lab.md).
The lab is the *plumbing* (one stream → many backends); this playground is the
*payload* — a real cross-language app instrumented to the maximum so every
backend (Parallax, Maple, SigNoz, OpenObserve, Sentry) receives identical,
feature-complete telemetry and we can compare how each renders it.

## Why this exists

We are building Parallax. To know what "good" looks like — and what we must beat
— we need a single application that *emits every signal a modern observability
stack can produce*, across the exact technologies we care about, instrumented the
way the industry actually does it (OpenTelemetry **and** Sentry together). Then we
fan it out (via Rotel) to every competitor and to Parallax, and compare the
rendered result side by side.

Two hard requirements shape everything:

1. **Maximum fidelity.** Distributed traces, logs, metrics, exemplars, span
   links, exceptions with stack traces, RUM/web-vitals, session replay, issue
   grouping, profiling, source maps — exercise the whole menu. If a signal
   exists, the playground produces it.
2. **Only our stack.** No technology outside the list below. Within it, use the
   latest stable versions and the current best-practice setup for each.

## The locked stack

| Tier | Technology | Instrumentation |
|---|---|---|
| **Frontend** | TypeScript + **TanStack Start** (React, on Vite/Nitro) | OTel Web SDK + `@sentry/tanstackstart-react` |
| **Backend (Rust)** | **Rust + Axum** (HTTP) + **tonic** (gRPC), microservices | `tracing` + `tracing-opentelemetry` + `opentelemetry-otlp` + `sentry` |
| **Backend (Java)** | **Spring Boot + GraphQL** and **Spring Boot + gRPC** (separate services) | OTel Java agent via **`sentry-opentelemetry-agent`** + Sentry Spring Boot starter |
| **Client** | **Rust CLI** (admin/load tool) | same Rust stack, tuned for short-lived processes |
| **Edges** | Postgres (DB spans), a message broker (queue producer/consumer spans + links) | auto-instrumented per language |

Java is included deliberately as a **cross-stack control**: the *same* business
operation is implemented in both Rust and Java so we can compare how identical
logic, identically instrumented, renders across languages and across backends.

## Core principle: two ingest paths, run in parallel

No single protocol carries every feature. The playground emits **both**, always:

- **OpenTelemetry → OTLP → Rotel → all backends.** The portable path: traces,
  logs, metrics, exemplars, span links, distributed context. This is what every
  backend (incl. Parallax) consumes and what we compare apples-to-apples.
- **Sentry SDK → Sentry envelope (DSN) → Sentry, directly.** The product path:
  managed **issues** (grouping/lifecycle), **breadcrumbs**, **releases +
  regression tracking**, **source maps / source context**, **session replay**,
  **profiling**, **user feedback**. These are Sentry-only objects that the OTLP
  path structurally cannot represent. Sentry *also* has an OTLP ingest (traces +
  logs, **no metrics**, open beta) — we use the SDK/envelope path for Sentry
  because it unlocks the features above; OTLP-into-Sentry is exercised only as a
  secondary comparison.

> **Parallax note (operator, 2026-06-23):** Parallax does not consume the Sentry
> envelope yet, and that's fine — the *sample stack* must still be set up with
> OTel **+** Sentry because that is how the industry runs it. Parallax is judged
> on the OTLP path today; the envelope path is the bar Parallax's future
> Sentry-compatibility wedge must clear.

The non-negotiable engineering rule in every service: **OpenTelemetry owns the
spans; Sentry rides along.** One OTel SDK per process is the single source of
spans; the Sentry layer consumes them. This avoids the double-instrumentation /
broken-span-nesting trap that hits all three languages.

## Architecture — one domain, many stacks

A small but realistic **e-commerce checkout** domain. Chosen because checkout
naturally fans across services, mixes sync and async, touches a DB, and has
obvious error paths — so it generates every signal without contrivance.

```
   ┌────────────────────────────────────────────────────────────────────┐
   │                         ONE DISTRIBUTED TRACE                        │
   └────────────────────────────────────────────────────────────────────┘

  Browser (TanStack Start, TS)
     │  fetch  (W3C traceparent over CORS)
     ▼
  gateway        Rust + Axum (HTTP/REST + GraphQL entry for the browser)
     ├── gRPC ─► pricing-rs     Rust + tonic        ┐  A/B PAIR: identical
     ├── gRPC ─► pricing-java   Spring Boot + gRPC   ┘  contract, two stacks
     ├── gRPC ─► inventory      Rust + tonic
     ├── GraphQL ─► catalog     Spring Boot + GraphQL (Java)
     └── HTTP  ─► orders        Rust + Axum
                    │  publish (broker; producer span)
                    ▼
                 broker (Kafka or NATS)
                    │  consume (consumer span, linked to producer)
                    ▼
                 fulfillment   Spring Boot (Java) consumer
                    └── DB (Postgres)  ← all services hit Postgres for DB spans

  cli (Rust, short-lived)  ── HTTP ─►  gateway   (seed data / trigger load)
```

### Services and what each is for

| Service | Stack | Transport | Why it's here (signals it showcases) |
|---|---|---|---|
| `web` | TanStack Start (TS) | browser + SSR + server functions | RUM, web vitals, session replay, browser→server trace handoff, source maps |
| `gateway` | Rust + Axum | HTTP/REST + GraphQL (server) + gRPC (client) | SERVER + CLIENT spans, fan-out, the trace root for browser requests |
| `pricing-rs` | Rust + tonic | gRPC (server) | Rust gRPC server spans; **A/B vs pricing-java** |
| `pricing-java` | Spring Boot + gRPC | gRPC (server) | Java gRPC server spans; **A/B vs pricing-rs**; Rust→Java gRPC propagation |
| `inventory` | Rust + tonic | gRPC (server) | Rust→Rust gRPC; DB latency histogram + exemplar |
| `catalog` | Spring Boot + GraphQL | GraphQL (server) | per-resolver/data-fetcher spans, N+1/DataLoader, GraphQL partial errors |
| `orders` | Rust + Axum | HTTP (server) + broker producer | PRODUCER span, deliberate error path, DB write |
| `fulfillment` | Spring Boot (Java) | broker consumer | CONSUMER span + **span link** to producer; cross-language async trace |
| `cli` | Rust (CLI) | HTTP client | short-lived process telemetry, flush-on-exit, `process.*` attributes, trace entry point |

The **A/B pair** (`pricing-rs` vs `pricing-java`) is the heart of the cross-stack
comparison: same proto contract, same logic, same scenarios — so any difference
in the rendered trace/metrics/errors is attributable to language/SDK, not to the
app.

## Cross-language distributed tracing

One trace must span Browser → Rust → Java → broker → Java, in both call
directions. The mechanism is uniform:

- **W3C Trace Context** (`traceparent` + `tracestate`) is the propagator on every
  hop; **W3C Baggage** carries business context (`tenant.id`, `user.tier`,
  `cart.id`). Every service sets `propagators = tracecontext,baggage` explicitly.
- Per-hop propagation responsibilities:

| Hop | Carrier | Inject (client) | Extract (server) | Notes |
|---|---|---|---|---|
| Browser → gateway | HTTP headers | OTel-web fetch instrumentation | axum middleware | needs CORS `Access-Control-Allow-Headers: traceparent,tracestate,baggage` or headers silently drop |
| SSR → browser (first paint) | `<meta name="traceparent">` | TanStack server middleware emits meta | `instrumentation-document-load` reads it | initial navigation has no fetch to inject into |
| gateway → pricing-* | gRPC metadata | tonic interceptor | tonic / OTel Java agent | Rust→Rust and Rust→Java, same trace |
| gateway → catalog | HTTP headers (GraphQL) | reqwest `HeaderInjector` | OTel Java agent | Rust→Java GraphQL |
| orders → broker → fulfillment | message headers | producer injects | consumer extracts + **span link** | async; link producer↔consumer |
| cli → gateway | HTTP headers | reqwest injector | axum middleware | short-lived root |

- **Sentry interop:** all SDKs are configured to emit/accept W3C `traceparent`
  (Sentry browser uses `propagateTraceparent`; Sentry Java/Node ride on OTel), so
  the Sentry transaction tree and the OTLP trace share one `trace_id`.

## Per-component instrumentation spec

Versions below are **known-good floors (June 2026)** — per repo policy, resolve
the latest mutually-compatible stable set at implementation and pin them.

### Frontend — TanStack Start (TypeScript)

- **OTel (portable):** `@opentelemetry/sdk-trace-web` 2.8, `@opentelemetry/
  context-zone`, `@opentelemetry/exporter-trace-otlp-http` 0.219, instrumentations
  `-fetch` / `-xml-http-request` / `-document-load` / `-user-interaction`,
  `@opentelemetry/core` composite W3C propagators.
- **Sentry (product RUM):** `@sentry/tanstackstart-react` (beta, v10 line) +
  `@sentry/vite-plugin` for source maps; integrations `replayIntegration`,
  `browserTracingIntegration` (LCP/CLS/INP/FCP/TTFB), `feedbackIntegration`,
  `consoleLoggingIntegration`, `tanstackRouterBrowserTracingIntegration`.
- **One owner of the global OTel SDK** — let the app own the WebTracerProvider;
  Sentry browser tracing coexists.
- **Export path:** browser → **same-origin `/v1/traces` proxy** → Rotel (avoids
  collector CORS entirely; hides the endpoint; allows server-side auth). SSR /
  server-function spans go through the Node-side OTel SDK to Rotel directly.
- **Wiring points:** `instrument.client.ts` (imported first in `client.tsx`),
  `instrument.server.mjs` (via `NODE_OPTIONS=--import`), `createStart()`
  request/function middleware for server spans, `<meta traceparent>` emit in
  request middleware, set `generateFunctionId` so server-function spans aren't
  opaque SHA-256 hashes.
- **Gotchas:** `ZoneContextManager` needs ES2015 transpile (native async/await
  breaks async context); cross-origin trace headers need the CORS allow-list +
  `propagateTraceHeaderCorsUrls` (use RegExp); web-vitals must flush on
  `visibilitychange` (CLS/INP finalize on hide); browser OTel logs are
  experimental — use Sentry logs for the browser.
- **Unlocks:** document-load/resource/fetch/XHR/user-interaction spans;
  browser→backend distributed trace; web vitals; session replay; source-mapped
  React error boundaries; breadcrumbs; releases; user feedback.

### Rust services — Axum + tonic (`gateway`, `pricing-rs`, `inventory`, `orders`)

- **Crates (floors):** `tracing`, `tracing-subscriber` 0.3, `tracing-opentelemetry`
  0.33, `opentelemetry` 0.32, `opentelemetry_sdk` 0.32, `opentelemetry-otlp` 0.32
  (enable `grpc-tonic` + a TLS backend explicitly — 0.32 changed defaults),
  `opentelemetry-semantic-conventions` 0.32, `opentelemetry-appender-tracing`
  0.32 (logs bridge), `sentry` 0.48 + `sentry-tracing` (+ `sentry-tower` /
  `sentry-tonic` for request-scoped hubs).
- **Single subscriber, parallel consumers** (the clean "both at once"):
  `registry().with(fmt).with(OpenTelemetryLayer).with(MetricsLayer)
  .with(OpenTelemetryTracingBridge /*logs*/).with(sentry::integrations::tracing::layer())`.
  `tracing` is the only span source → no double-instrumentation.
- **Traces:** `#[instrument]` (with `skip_all`, `fields`, `err`, `ret`),
  `otel.kind`/`otel.status_code` reserved fields, span events via `event!`.
- **Logs:** structured `tracing` events exported as OTLP logs, auto-stamped with
  `trace_id`/`span_id`; filter `hyper/tonic/h2/reqwest/opentelemetry` out of the
  log layer to avoid self-logging loops.
- **Metrics:** `opentelemetry` counters/histograms with **exemplars** linking to
  traces; periodic OTLP exporter.
- **Propagation:** global `TraceContextPropagator` + baggage; reqwest
  `HeaderInjector` / axum `HeaderExtractor`; tonic interceptor inject/extract on
  metadata.
- **Errors:** `attach_stacktrace: true` + `RUST_BACKTRACE=1` for Sentry; record
  exception span events (`exception.type/message/stacktrace`) + `otel.status_code
  = ERROR` for OTLP; panics + `Result::Err` captured both ways.
- **Unlocks:** stitched HTTP+gRPC traces, logs↔traces, metrics↔traces exemplars,
  Sentry issues with Rust backtraces.

### Rust CLI (`cli`)

- Same crate stack, but **short-lived-process tuning**: wrap the whole invocation
  in a root span; record `process.command_args`, parsed subcommand,
  `process.exit.code`; prefer a **simple/synchronous exporter** (or batch with
  guaranteed flush); **force-flush `tracer/meter/logger` providers before exit**
  (call shutdown off the current-thread runtime to avoid the known deadlock);
  hold the Sentry init guard for the whole program.
- **Unlocks:** CLI telemetry as a first-class trace entry point; proves
  short-lived flush discipline (a known place tools lose data).

### Java — Spring Boot + GraphQL (`catalog`) and Spring Boot + gRPC (`pricing-java`)

- **One agent for both OTel and Sentry:** run **`io.sentry:sentry-opentelemetry-agent`**
  (8.44.x) as `-javaagent` with `SENTRY_AUTO_INIT=false`, and let the
  **`sentry-spring-boot-starter-jakarta`** (via `sentry-bom`) initialize the SDK.
  This is Sentry's sanctioned combo: the agent bundles upstream OTel
  auto-instrumentation (150+ libs incl. Spring GraphQL + grpc-java + JDBC +
  Logback) and feeds Sentry without double-instrumentation. **Version lock: SDK ==
  agent version, exactly** (SDK throws on mismatch since 8.6).
- **GraphQL variant:** `spring-boot-starter-graphql`; agent auto-instruments
  graphql-java → per-operation + per-data-fetcher spans, `graphql.operation.name/
  type`, sanitized `graphql.document` (keep sanitization on — high cardinality /
  PII risk). **Partial errors** (GraphQL returns HTTP 200 with `errors[]`): the
  transport span looks healthy, so add a `DataFetcherExceptionResolver` that calls
  `Sentry.captureException` for field errors. Watch DataLoader/N+1 span shapes.
- **gRPC variant:** Spring gRPC starter (`org.springframework.grpc:
  spring-grpc-spring-boot-starter`) + grpc-java; agent auto-instruments
  server+client with W3C over gRPC metadata → interops with Rust tonic both
  directions on one trace.
- **Logs:** agent bridges Logback → OTLP with trace correlation; MDC pattern
  surfaces `trace_id`/`span_id` in console logs too.
- **Metrics:** Micrometer/Actuator → OTLP (enable the agent's micrometer bridge);
  JVM runtime metrics; **exemplars** via Micrometer (`management.tracing.
  exemplars.include`).
- **Errors:** exception span events with full Java stack traces; **Sentry caveat
  — the current `SentrySpanProcessor` NOOPs `recordException`**, so explicitly
  `Sentry.captureException` in `@ExceptionHandler` / GraphQL error resolvers to
  get reliable Sentry issues. **Source context** (readable stacks in Sentry) via
  the Sentry Gradle/Maven plugin (`includeSourceContext = true`).
- **Resource:** `service.name` from `spring.application.name`
  (`catalog`, `pricing-java`), `deployment.environment.name`, `service.namespace`.
- **Native-image caveat:** if any Java service is built as a GraalVM native image,
  the `-javaagent` won't work → switch that one to the OTel Spring Boot starter.
- **Unlocks:** zero-code GraphQL + gRPC + JDBC traces, OTLP logs, JVM/HTTP/DB/RPC
  metrics + exemplars, Java stack traces in Sentry with source context.

## Telemetry feature-coverage checklist

The playground must produce every item below (grouped by signal). This is the
acceptance checklist.

- **Traces:** spans; all five span kinds (CLIENT/SERVER/PRODUCER/CONSUMER/
  INTERNAL); span status (Unset/Ok/Error); span events; **span links**; rich
  attributes (semconv + custom, incl. a high-cardinality stress attribute); deep
  nested waterfalls.
- **Propagation:** W3C trace context across every hop; W3C baggage; sampling-flag
  propagation (so backends can extrapolate true rate).
- **Logs:** OTLP LogRecords; `SeverityNumber` + `SeverityText`; trace correlation
  (trace/span id on records); the **new exception-as-log path** *and* the legacy
  span-event path (the span-event API is being deprecated, 2026 — exercise both
  so we see which backends followed it); instrumentation scope on logs.
- **Metrics:** Counter, UpDownCounter, Histogram (explicit + exponential buckets),
  async Gauge; **exemplars** (metric→trace jump); Views.
- **Exceptions:** `exception.type/message/stacktrace/escaped`; `error.type`; span
  status = Error correlated with the exception.
- **Resource / semconv:** `service.{name,version,namespace,instance.id}`,
  `deployment.environment.name`, `host.*`, `process.*`, `container.*`; HTTP
  (stable), RPC/gRPC (RC), DB (`db.system.name`/`db.query.text`, RC), messaging
  (experimental), GraphQL (development), `code.*` (experimental), `user/session`
  (experimental). Exercising the RC/experimental groups on purpose tests how each
  backend handles non-frozen attribute names.

## Scenario catalog — operations that generate each signal

The app must perform these (triggerable from the CLI, the frontend, or a
load-gen) so every signal appears naturally:

| # | Scenario | Signals exercised |
|---|---|---|
| 1 | Slow DB query (intentional latency) | DB CLIENT span (`db.system.name`, `db.query.text`), latency Histogram + exemplar |
| 2 | Deliberate panic / unhandled exception (Rust + Java) | exception (type/msg/stacktrace) as span event **and** log; span status=Error; Sentry grouped issue |
| 3 | Cross-service HTTP call (browser→gateway) | W3C propagation over CORS, CLIENT+SERVER spans, HTTP semconv, waterfall |
| 4 | Queue publish + async consume (orders→broker→fulfillment) | PRODUCER + CONSUMER spans, **span link**, messaging semconv, cross-language async trace |
| 5 | Frontend button click → backend | browser span → backend SERVER span via traceparent; web vitals; session replay; end-to-end trace |
| 6 | gRPC call Rust→Java and Rust→Rust (pricing A/B) | RPC semconv, gRPC status, CLIENT/SERVER spans; cross-stack comparison of identical op |
| 7 | GraphQL query with nested resolvers (catalog) | per-resolver/data-fetcher spans, GraphQL semconv, N+1/DataLoader shape, partial errors |
| 8 | Background batch job (CLI-triggered) | INTERNAL spans, multiple span links fanning into one batch trace |
| 9 | High request volume (load-gen) | Counter (requests), Histogram (latency), UpDownCounter (in-flight) |
| 10 | Resource saturation | UpDownCounter / async Gauge (queue depth, pool, memory) |
| 11 | Structured logging during a request | OTLP logs with severity + trace correlation |
| 12 | Baggage business context | W3C baggage (`tenant.id`, `user.tier`) surfaced as attributes downstream |
| 13 | Multi-instance deploy | resource attrs `service.instance.id`, `deployment.environment.name`, `host.*` |
| 14 | Sentry-SDK error w/ breadcrumbs (envelope path) | Sentry issue + breadcrumbs + release/commit + source maps; regression tracking |
| 15 | Recurring then resolved error | Sentry issue lifecycle (resolve→regress) — Sentry-only |
| 16 | CPU-heavy hot path | profiling (Sentry continuous/UI), slow-function detection |
| 17 | PII-bearing field (e.g. email) | tests redaction-at-ingest (Sentry scrubbing; Parallax's structured redaction) |
| 18 | CLI invocation end-to-end | short-lived process telemetry, `process.*` attrs, flush-on-exit, trace entry |

## How this exercises each backend (compare here)

Reuses the lab's fan-out. The point is to see the *same* scenario rendered five
ways. Highlights from the backend support matrix
([fan-out lab](otlp-fanout-comparison-lab.md), and the market deep-research
files):

- **OTLP metrics + exemplars** appear in SigNoz / OpenObserve / Maple / Parallax
  but **not in Sentry** (Sentry OTLP has no metrics) — the cleanest Sentry
  differentiator. Scenarios 1, 9, 10 separate them.
- **Issues / lifecycle / breadcrumbs / replay / profiling / source context** are
  Sentry-only (envelope path). Scenarios 14, 15, 16 separate Sentry from the raw
  OTLP backends.
- **Trace waterfall, span links, exceptions-with-stacktrace, logs↔traces** should
  appear everywhere — scenarios 2, 4, 7 test rendering quality and fidelity loss
  (e.g. who drops span links, who renames fields, whose GraphQL view is usable).
- **RUM / web vitals / session replay** (scenario 5) appear in Sentry / SigNoz /
  OpenObserve / Maple; Parallax does not target RUM — useful to confirm the gap
  is intentional.

The comparison output (what each backend kept vs dropped vs renamed, per
scenario) feeds the market matrices and Parallax capture/UI work.

## Repository layout and where it lives

This is a **multi-language application** (Rust + Java + TypeScript) — it does not
belong in the Parallax Cargo/Bun workspace and is not product code. Two options:

- **Recommended:** a **separate repository** `tailrocks/parallax-telemetry-playground`,
  Apache-2.0, Tailrocks-attributed — keeps the polyglot toolchains (Gradle/Maven,
  Cargo, Bun) out of the Parallax repo. Referenced from here.
- **Fallback:** under `bench/telemetry-playground/` in this repo (the repo already
  uses `bench/` for compose-based stacks). If chosen, add it to
  `PROJECT_STRUCTURE.md` in the same change.

Proposed layout (either location):

```
telemetry-playground/
  web/                 # TanStack Start (TS) — Bun
  services/
    gateway/           # Rust + Axum
    pricing-rs/        # Rust + tonic
    inventory/         # Rust + tonic
    orders/            # Rust + Axum
    pricing-java/      # Spring Boot + gRPC
    catalog/           # Spring Boot + GraphQL
    fulfillment/       # Spring Boot consumer
  cli/                 # Rust CLI
  proto/               # shared .proto (pricing/inventory contracts)
  graphql/             # shared GraphQL schema (catalog)
  deploy/
    docker-compose.yml # all services + Postgres + broker; OTLP → Rotel
    otel/              # shared resource attrs, sampling config
  scenarios/           # scripts that drive scenarios 1–18
  README.md
```

## Integration with the fan-out lab

- Every service's OTLP exporter targets **Rotel** (the lab hub): inside the
  playground compose, `http://rotel:4317`; from host processes (CLI, dev),
  `http://localhost:4317`. Rotel fans out to Parallax (host) + Maple + SigNoz +
  OpenObserve + Sentry-via-OTLP per the lab.
- Every service's **Sentry SDK** points at the lab's self-hosted Sentry DSN
  (envelope path), independent of OTLP — so Sentry gets the rich envelope while
  every other backend gets OTLP.
- Pin one `deployment.environment.name` (e.g. `playground`) and a shared release
  id across all services + the frontend so cross-service/release views line up.
- Drive scenarios with the CLI / `scenarios/` scripts using **pinned trace ids +
  timestamps** (per the lab's clock/ID-alignment rule) so "the same event" is
  retrievable across all five backends' read APIs.

## Risks / open questions

- **Bun OTel gaps (frontend server + any Bun service).** Under Bun,
  Express/Fastify `requestHook`s don't fire (generic span names) and Bun-native
  APIs (`Bun.serve`, `bun:sqlite`) aren't auto-instrumented — needs manual spans;
  `instrumentation-fs` can crash (disable it). The frontend's *server* tier is
  Bun; budget for manual spans. (Backend services are Rust/Java, so this is
  scoped to the TS tier.)
- **JS exemplars are still experimental** — metric→trace jump may not work from
  the TS tier; it does from Rust/Java. Don't promise exemplars on the frontend.
- **Sentry OTLP has no metrics; Java `SentrySpanProcessor` NOOPs span events** —
  use the Sentry SDK envelope path and explicit `captureException` for reliable
  Java issues.
- **Span-event API deprecation (2026)** — exercise both span-event and
  exception-as-log paths; expect backends to differ on which they render.
- **GraphQL high cardinality / PII** — keep query sanitization on; don't put raw
  operation names/documents in span names.
- **`@sentry/tanstackstart-react` is beta** — pin with `@sentry/react` v10 and
  verify the trio installs cleanly; `@sentry/profiling-node` is a native addon —
  verify it loads under Bun before relying on profiling there.
- **Version churn** — OTel Rust 0.32 / tracing-opentelemetry 0.33 / Sentry Java
  8.44 / Sentry JS v10 / OTel Java agent 2.29 are floors; resolve latest
  mutually-compatible at impl and pin.
- **Resource weight** — this stack (3 Java JVMs + 4 Rust services + broker +
  Postgres + the TS app) on top of the 5-backend lab is heavy; run the full thing
  on a server, a reduced subset on a laptop (mirror the benchmark two-tier rule).

## Suggested phasing

1. **One trace, two stacks** — `web` → `gateway` (Rust) → `pricing-rs` (Rust) and
   `pricing-java` (Java), OTLP → Rotel → Parallax + one other backend. Prove the
   browser→Rust→Java W3C stitch and the A/B pair render.
2. **Add the signal breadth** — logs, metrics+exemplars, GraphQL (`catalog`), DB
   spans; wire Sentry SDK envelope path everywhere; scenarios 1–3, 6, 7, 11.
3. **Async + errors** — broker (`orders`→`fulfillment`), span links, deliberate
   panics/exceptions both languages, Sentry issues + source context; scenarios 2,
   4, 8, 14, 15.
4. **RUM + product features** — session replay, web vitals, profiling, user
   feedback, source maps; scenarios 5, 16, 17.
5. **CLI + load + full fan-out** — CLI entry, load-gen, all five backends up,
   run the full scenario catalog; produce the cross-backend comparison table that
   feeds the market matrices.

## Sources

- Rust: [OTel Rust](https://opentelemetry.io/docs/languages/rust/) ·
  [tracing-opentelemetry](https://docs.rs/tracing-opentelemetry) ·
  [opentelemetry-otlp 0.32 CHANGELOG](https://docs.rs/crate/opentelemetry-otlp/latest/source/CHANGELOG.md) ·
  [Sentry Rust](https://docs.sentry.io/platforms/rust/) ·
  [Sentry + OTel](https://blog.sentry.io/sentry-opentelemetry-work-together/)
- TS backend: [OTel JS](https://opentelemetry.io/docs/languages/js/) ·
  [Sentry Node OTel](https://docs.sentry.io/platforms/javascript/guides/node/opentelemetry/) ·
  [Bun OTel caveats](https://github.com/oven-sh/bun/issues/26536)
- Frontend: [OTel browser](https://opentelemetry.io/docs/languages/js/getting-started/browser/) ·
  [Sentry TanStack Start](https://docs.sentry.io/platforms/javascript/guides/tanstackstart-react/) ·
  [TanStack Start v1](https://tanstack.com/blog/announcing-tanstack-start-v1)
- Java: [OTel Java agent](https://opentelemetry.io/docs/zero-code/java/agent/) ·
  [GraphQL semconv](https://opentelemetry.io/docs/specs/semconv/graphql/) ·
  [Sentry Spring Boot + OTel](https://docs.sentry.io/platforms/java/guides/spring-boot/opentelemetry/) ·
  [Spring gRPC](https://docs.spring.io/spring-grpc/reference/)
- Cross-cutting: [OTel semantic conventions](https://opentelemetry.io/docs/concepts/semantic-conventions/) ·
  [span-event deprecation](https://opentelemetry.io/blog/2026/deprecating-span-events/) ·
  [W3C Trace Context](https://www.w3.org/TR/trace-context/) ·
  [Sentry envelopes](https://develop.sentry.dev/sdk/data-model/envelopes/) ·
  [Sentry OTLP](https://docs.sentry.io/concepts/otlp/)
- Internal: [otlp-fanout-comparison-lab.md](otlp-fanout-comparison-lab.md),
  [`capture/otlp.md`](../capture/otlp.md), market deep-research for
  [sentry](../market/sentry-deep-research.md) /
  [signoz](../market/signoz-deep-research.md) /
  [openobserve](../market/openobserve-deep-research.md) /
  [maple](../market/maple-deep-research.md)
</content>
