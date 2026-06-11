//! Seed a running Parallax with demo telemetry for UI work and demos:
//! a cross-service error trace (gateway → checkout, DB + GraphQL spans),
//! a panic-shaped FATAL log, point metrics, and an HTTP-duration histogram.
//!
//! Usage: start `parallax serve`, then
//! `cargo run -p parallax-server --example seed`
//! (override the target with OTEL_EXPORTER_OTLP_ENDPOINT).

use opentelemetry::logs::{AnyValue, LogRecord as _, Logger as _, LoggerProvider as _, Severity};
use opentelemetry::metrics::MeterProvider as _;
use opentelemetry::trace::{
    Span as _, SpanContext, SpanKind, Status, TraceContextExt as _, Tracer as _,
    TracerProvider as _,
};
use opentelemetry::{Context as OtelContext, KeyValue};
use opentelemetry_otlp::WithExportConfig;

fn resource(service: &str) -> opentelemetry_sdk::Resource {
    opentelemetry_sdk::Resource::builder()
        .with_attributes([
            KeyValue::new("service.name", service.to_string()),
            KeyValue::new("service.version", "0.1.0"),
        ])
        .build()
}

fn tracer_provider(service: &str, endpoint: &str) -> opentelemetry_sdk::trace::SdkTracerProvider {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.to_string())
        .build()
        .expect("span exporter");
    opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource(service))
        .build()
}

#[tokio::main]
async fn main() {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:4317".into());

    // One failing checkout workflow, three times (the issue trends).
    let gateway = tracer_provider("api-gateway", &endpoint);
    let checkout = tracer_provider("checkout", &endpoint);
    let mut last_trace = String::new();
    for attempt in 0..3 {
        let gateway_tracer = gateway.tracer("gateway");
        let mut client_span = gateway_tracer
            .span_builder("grpc client checkout.Authorize")
            .with_kind(SpanKind::Client)
            .start(&gateway_tracer);
        last_trace = format!("{:032x}", client_span.span_context().trace_id());
        let parent: SpanContext = client_span.span_context().clone();

        let tracer = checkout.tracer("checkout");
        let remote = OtelContext::new().with_remote_span_context(parent);
        let mut server_span = tracer
            .span_builder("grpc server checkout.Authorize")
            .with_kind(SpanKind::Server)
            .start_with_context(&tracer, &remote);
        let server_ctx =
            OtelContext::new().with_remote_span_context(server_span.span_context().clone());

        let mut graphql_span = tracer
            .span_builder("graphql.execute checkoutSummary")
            .start_with_context(&tracer, &server_ctx);
        let gql_ctx =
            OtelContext::new().with_remote_span_context(graphql_span.span_context().clone());
        let mut db_span = tracer
            .span_builder("query orders")
            .with_kind(SpanKind::Client)
            .with_attributes([
                KeyValue::new("db.system.name", "postgresql"),
                KeyValue::new("db.operation.name", "SELECT"),
                KeyValue::new(
                    "db.query.text",
                    "SELECT id, total FROM orders WHERE cart_id = $1",
                ),
            ])
            .start_with_context(&tracer, &gql_ctx);
        tokio::time::sleep(std::time::Duration::from_millis(12)).await;
        db_span.end();
        graphql_span.end();
        server_span.add_event(
            "exception",
            vec![
                KeyValue::new("exception.type", "tonic::Status"),
                KeyValue::new(
                    "exception.message",
                    format!("deadline exceeded talking to payments (attempt {attempt})"),
                ),
                KeyValue::new(
                    "exception.stacktrace",
                    "checkout::payment::authorize at src/payment.rs:184",
                ),
            ],
        );
        server_span.set_status(Status::error("deadline exceeded"));
        server_span.end();
        client_span.end();
    }
    gateway.force_flush().expect("gateway flush");
    checkout.force_flush().expect("checkout flush");

    // A panic-shaped FATAL log correlated to the last trace.
    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.clone())
        .build()
        .expect("log exporter");
    let logger_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_batch_exporter(log_exporter)
        .with_resource(resource("checkout"))
        .build();
    let logger = logger_provider.logger("seed");
    let mut record = logger.create_log_record();
    record.set_severity_number(Severity::Fatal);
    record.set_severity_text("FATAL");
    record.set_body(AnyValue::from(
        "panicked: checkout total overflowed at row 4242",
    ));
    record.add_attribute("exception.type", "panic");
    record.add_attribute("exception.message", "checkout total overflowed at row 4242");
    record.add_attribute(
        "exception.stacktrace",
        "checkout::cart::total at src/cart.rs:99",
    );
    logger.emit(record);
    let mut info = logger.create_log_record();
    info.set_severity_number(Severity::Info);
    info.set_severity_text("INFO");
    info.set_body(AnyValue::from("retrying payment authorization"));
    logger.emit(info);
    logger_provider.force_flush().expect("log flush");

    // Metrics: a gauge, a counter, and an HTTP-duration histogram.
    let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()
        .expect("metric exporter");
    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(metric_exporter).build();
    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource("checkout"))
        .build();
    let meter = meter_provider.meter("seed");
    let queue = meter.u64_gauge("checkout.queue.depth").build();
    let orders = meter.u64_counter("checkout.orders.total").build();
    let duration = meter
        .f64_histogram("http.server.request.duration")
        .with_unit("s")
        .build();
    for i in 0..30u64 {
        queue.record(3 + (i % 7), &[]);
        orders.add(1, &[]);
        duration.record(0.030 + (i as f64 % 9.0) * 0.012, &[]);
        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
    }
    meter_provider.force_flush().expect("metric flush");

    println!("seeded. last trace id: {last_trace}");
}
