# Rust Stack Instrumentation Matrix: What the Operator's Apps Can Send

<!-- markdownlint-disable MD013 -->

Research date: 2026-06-12 (all crate versions verified against crates.io/GitHub/docs.rs on this
date). Operator statement #7 defines the concrete capture targets: a Ratatui TUI, a CLI driving
Docker over the socket (bollard), tonic-gRPC microservices (some axum HTTP), Juniper GraphQL with
DataLoaders, **tokio-postgres** and the official **`clickhouse`** crate (operator correction:
not sqlx), Redis, RabbitMQ (lapin). This note answers "what can a Rust application actually send
to Parallax, and how" — the full emission scope per layer, with gaps named.

**Ecosystem state:** `opentelemetry-rust` 0.32 (2026-05) — **logs + metrics API/SDK stable;
traces still beta; OTLP exporter RC**. One `tracing_subscriber::registry()` carries everything:
spans via `tracing-opentelemetry`, logs via `opentelemetry-appender-tracing` (with automatic
trace/span-ID correlation), metrics via the OTel Meter API; `opentelemetry-otlp` exports over
gRPC :4317 / HTTP :4318. **Version lockstep is the #1 operational hazard:** otel 0.32 ⇄
tracing-opentelemetry 0.33 ⇄ davidB middleware crates 0.38 ⇄ tower-otel 0.10 are aligned today;
system-metrics and the metrics-crate bridge lag at 0.31.

## The matrix

| Layer | Crate(s) @ version | Emits automatically | Gaps / manual work |
| --- | --- | --- | --- |
| Core spans/logs/metrics | `tracing` 0.1.44, `tracing-subscriber` 0.3.23, `tracing-opentelemetry` 0.33, `opentelemetry`/`-sdk`/`-otlp`/`-appender-tracing` 0.32 | Spans w/ events; logs w/ trace correlation; `MetricsLayer` turns `counter.*`/`histogram.*` event fields into metrics; `otel.name/kind/status_*` overrides | Span fields must be declared at creation (`field::Empty` + `Span::record`); filter the log bridge to exclude `opentelemetry`/`hyper`/`tonic`/`h2` targets (telemetry-loop guard) |
| Errors/panics | `tracing-error` 0.2.1 (`SpanTrace`), `tracing-panic` 0.1.2 | Panics as ERROR log records w/ span context; SpanTrace in error payloads | **OTel `exception.*` mapping is always manual** — Parallax derives error events from both encodings, so emitters should set `exception.type/message/stacktrace` where possible |
| gRPC (tonic 0.14.6) | `tonic-tracing-opentelemetry` 0.38 or `tower-otel` 0.10 | Server span + W3C traceparent extract; client span + inject; tower-otel adds RPC metrics from the same layer | tonic-tracing-otel self-describes early; full `rpc.*` semconv "TODO" — verify attributes; prefer tower-otel when metrics wanted |
| HTTP (axum 0.8) | `axum-tracing-opentelemetry` 0.38 (`OtelAxumLayer` + `OtelInResponseLayer`) | Request spans w/ `http.*` semconv, context extract; trace context returned in response headers | `tower-http TraceLayer` alone is NOT distributed tracing (no propagation/semconv) — logging only |
| Postgres (`tokio-postgres` 0.7.17) | **Nothing** — `log` events only, no tracing dep | — | **Manual `#[instrument]` wrappers** at the repository layer: `db.system.name="postgresql"`, `db.query.text` (own the redaction decision), `db.operation.name`; span lifetime = duration. No maintained third-party wrapper exists; deadpool hooks are lifecycle-only |
| ClickHouse (official `clickhouse` 0.15.1) | **Partial:** `opentelemetry` feature (0.15+) injects `traceparent` into HTTP requests → ClickHouse server-side spans join your trace | No client-side spans around `query()`/`insert()` — wrap manually with db.* semconv; enable the feature + global propagator |
| Redis | `fred` 10.1 has first-class `partial-tracing` (span per command); `redis` 1.2 has none (wrapper crate `otel-instrumentation-redis` 0.1 experimental) | fred: command spans w/ latency | redis-rs users: manual wrappers (`db.system.name="redis"`, `db.operation.name`) |
| RabbitMQ (`lapin` 4.10) | Nothing (internal logs only) | — | Standard OTel messaging recipe by hand: producer span + propagator inject into `BasicProperties` headers; consumer extract + `process` span; `messaging.system="rabbitmq"`, destination/routing-key attrs |
| GraphQL (Juniper 0.17.1) | **Nothing** — tracking issue #423 open since 2019; no per-operation/per-field spans | — | **Operator decision 2026-06-12: deferred — the operator instruments his own resolvers**; Parallax just consumes whatever spans arrive. Pattern when wanted: manual `#[instrument]` on the operation entry + expensive resolvers (per-scalar-field spans = noise) |
| DataLoader (`dataloader` 0.18) | Nothing | — | Wrap the batch `load` fn in a span. Known attribution gap: batched loads coalesce across resolvers — the batch span parents to whichever resolver flushed it; record the batch key-count as an attribute |
| App metrics | OTel Meter API (stable) → `http.server.request.duration` histograms etc.; or `tower-otel` metrics module | Request rate/duration/size per route | Define histograms once in a shared telemetry crate; `metrics`-crate bridge (`metrics-exporter-opentelemetry` 0.2) lags at otel 0.31 |
| System metrics (CPU/mem) | `opentelemetry-system-metrics` 0.31 (`init_process_observer`) — process CPU/mem/disk/network | The operator's CPU/RAM dashboard feed | otel 0.31 skew; skew-free fallback = `sysinfo` 0.39 gauges by hand |
| Tokio runtime | `tokio-metrics` 0.5 (`RuntimeMonitor`/`TaskMonitor`) | Worker/queue depth on stable tokio | No OTel exporter built in — periodic task copies intervals into OTel gauges; poll-time detail needs `tokio_unstable` |
| TUI (Ratatui 0.30) | Standard tracing | — | **A TUI owns stdout/stderr: no fmt layer** — OTLP-only logging (or `tracing-appender` file); flush/shutdown providers before exit or final spans drop |
| Docker socket (bollard 0.21) | Internal debug events only | — | Manual spans around socket calls (`container.id` etc. as attrs) |

## The reference init (one shared `telemetry` crate)

Every operator app initializes the same way — three providers, one registry; this is also what
Parallax's quickstart documents:

```rust
// deps: tracing, tracing-subscriber, tracing-opentelemetry 0.33,
//       opentelemetry{,_sdk,-otlp,-appender-tracing,-semantic-conventions} 0.32
let resource = /* service.name, service.version, deployment.environment.name,
                  vcs.ref.head.revision, parallax.run_id from env */;

let tracer_provider = SdkTracerProvider::builder()
    .with_batch_exporter(SpanExporter::builder().with_tonic().build()?)
    .with_resource(resource.clone()).build();
let logger_provider = SdkLoggerProvider::builder()
    .with_batch_exporter(LogExporter::builder().with_tonic().build()?)
    .with_resource(resource.clone()).build();
let meter_provider = SdkMeterProvider::builder()
    .with_reader(PeriodicReader::builder(MetricExporter::builder().with_tonic().build()?).build())
    .with_resource(resource).build();

global::set_text_map_propagator(TraceContextPropagator::new());

let log_bridge = OpenTelemetryTracingBridge::new(&logger_provider)
    .with_filter(/* exclude opentelemetry,hyper,tonic,h2 targets */);

tracing_subscriber::registry()
    .with(OpenTelemetryLayer::new(tracer_provider.tracer("app")))
    .with(MetricsLayer::new(meter_provider.clone()))
    .with(log_bridge)
    // .with(fmt_layer)  // NOT in TUI apps — OTLP/file only
    .init();
// On exit (critical for CLIs/TUIs): tracer_provider.shutdown(); logger_provider.shutdown(); meter_provider.shutdown();
```

## What this means for Parallax

1. **The operator's cross-service story works today**: tonic + axum middlewares propagate W3C
   context; ClickHouse 0.15 even joins server-side spans. Parallax receives one trace across
   TUI → gRPC → service → DB.
2. **The DB story is wrapper-based by ecosystem reality**: tokio-postgres emits nothing;
   Parallax's docs ship the repository-layer `#[instrument]` pattern (query text, operation,
   duration) rather than pretending auto-instrumentation exists.
3. **Juniper visibility is manual but cheap**: operation + resolver spans cover "how each part
   was generated and how long"; the DataLoader batch-attribution gap is documented, not hidden.
4. **Metrics for the operator's dashboards** come from three feeds: HTTP/RPC histograms
   (middleware), process CPU/mem (`opentelemetry-system-metrics` or sysinfo gauges), and any
   custom `counter.*`/`histogram.*` fields — all arriving as standard OTLP metrics Parallax
   stores and charts.
5. **Version lockstep belongs in the quickstart**: Parallax documents one known-good crate set
   per release and re-verifies on each otel-rust minor (traces are still beta upstream).
6. **Exception encoding guidance**: emitters set `exception.*` where they can; Parallax derives
   error events from exception span events, ERROR/FATAL logs, and exception-as-log either way
   ([otlp.md](otlp.md)).

Sources: crates.io API snapshots (2026-06-12); otel-rust 0.32 release notes; docs.rs for
tracing-opentelemetry / opentelemetry-otlp / appender-tracing / axum-tracing-opentelemetry /
tonic-tracing-opentelemetry / tower-otel / fred / clickhouse; GitHub: rust-postgres Cargo.toml,
ClickHouse/clickhouse-rs CHANGELOG + opentelemetry example, sqlx CHANGELOG (superseded target),
juniper #423 + CHANGELOG, tokio-metrics, OTel semconv (db, messaging, http).
