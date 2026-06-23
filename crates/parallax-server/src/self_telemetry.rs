//! Self-telemetry: `parallax serve` emitting its **own** OTLP spans and logs.
//!
//! Parallax is normally a pure sink — it receives OTLP and never says anything
//! about itself. When `[telemetry] self_otlp_endpoint` (or `PARALLAX_SELF_OTLP`)
//! names a collector, serve also *exports* its internal `tracing` spans/logs
//! there over OTLP/gRPC, tagged `service.name = parallax`. In the OTLP fan-out
//! lab that endpoint is Rotel, so Parallax's own telemetry fans out beside the
//! workload it ingests.
//!
//! The one hazard is a feedback loop: if the configured sink fans telemetry
//! back to Parallax (Rotel does), exporting our *ingest-path* spans would make
//! every received batch generate spans that get exported, received again, and
//! so on. [`export_filter`] turns the ingest receivers/worker and the
//! transport/export crates OFF for this exporter, breaking the loop while still
//! emitting Parallax's API/serve telemetry.

use crate::config::Config;
use opentelemetry::KeyValue;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::registry::Registry;

/// A boxed subscriber layer fixed to the root [`Registry`] so the CLI can
/// attach it first in the subscriber stack (empty when self-telemetry is off).
pub type BoxLayer = Box<dyn Layer<Registry> + Send + Sync>;

/// What this exporter must never emit: the OTLP ingest receivers + worker (the
/// self → sink → self loop), and the transport/export crates they ride on.
/// Everything else of Parallax's own at INFO+ is exported.
fn export_filter() -> Targets {
    Targets::new()
        .with_default(LevelFilter::INFO)
        .with_target("parallax_server::otlp_grpc", LevelFilter::OFF)
        .with_target("parallax_server::otlp_http", LevelFilter::OFF)
        .with_target("parallax_server::worker", LevelFilter::OFF)
        .with_target("h2", LevelFilter::OFF)
        .with_target("hyper", LevelFilter::OFF)
        .with_target("hyper_util", LevelFilter::OFF)
        .with_target("tonic", LevelFilter::OFF)
        .with_target("tower", LevelFilter::OFF)
        .with_target("reqwest", LevelFilter::OFF)
        .with_target("opentelemetry", LevelFilter::OFF)
        .with_target("opentelemetry_sdk", LevelFilter::OFF)
        .with_target("opentelemetry_otlp", LevelFilter::OFF)
}

/// Owns the export pipelines so they can be flushed and shut down on serve
/// exit; dropping without [`SelfTelemetry::shutdown`] risks losing buffered
/// spans/logs.
pub struct SelfTelemetry {
    tracer_provider: SdkTracerProvider,
    logger_provider: SdkLoggerProvider,
}

impl SelfTelemetry {
    /// Flush and shut the exporters down (call before process exit).
    pub fn shutdown(&self) {
        let _ = self.tracer_provider.force_flush();
        let _ = self.tracer_provider.shutdown();
        let _ = self.logger_provider.shutdown();
    }
}

/// The result of [`install`]: layers to attach to the subscriber, the guard to
/// flush on exit, and the resolved endpoint (for the ready banner).
pub struct Installed {
    pub layers: Vec<BoxLayer>,
    pub guard: SelfTelemetry,
    pub endpoint: String,
}

/// Resolve the self-telemetry endpoint: `PARALLAX_SELF_OTLP` wins (even `off`),
/// else `[telemetry] self_otlp_endpoint`. Empty / `off` ⇒ disabled (`None`).
pub fn resolve_endpoint(config: &Config) -> Option<String> {
    fn pick(raw: &str) -> Option<String> {
        let raw = raw.trim();
        if raw.is_empty() || raw.eq_ignore_ascii_case("off") {
            None
        } else {
            Some(raw.to_string())
        }
    }
    match std::env::var("PARALLAX_SELF_OTLP") {
        Ok(env) => pick(&env),
        Err(_) => pick(&config.telemetry.self_otlp_endpoint),
    }
}

fn resource() -> Resource {
    Resource::builder()
        .with_service_name("parallax")
        .with_attribute(KeyValue::new("service.version", env!("CARGO_PKG_VERSION")))
        .build()
}

/// Build the OTLP exporters and subscriber layers for self-telemetry. Call once
/// — only when [`resolve_endpoint`] returned `Some` — inside the tokio runtime,
/// before the subscriber is installed.
pub fn install(endpoint: &str) -> anyhow::Result<Installed> {
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.to_string())
        .build()?;
    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(resource())
        .with_batch_exporter(span_exporter)
        .build();

    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.to_string())
        .build()?;
    let logger_provider = SdkLoggerProvider::builder()
        .with_resource(resource())
        .with_batch_exporter(log_exporter)
        .build();

    let trace_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer_provider.tracer("parallax"))
        .with_filter(export_filter())
        .boxed();
    let log_layer =
        opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logger_provider)
            .with_filter(export_filter())
            .boxed();

    Ok(Installed {
        layers: vec![trace_layer, log_layer],
        guard: SelfTelemetry {
            tracer_provider,
            logger_provider,
        },
        endpoint: endpoint.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with(endpoint: &str) -> Config {
        let mut config = Config::default();
        config.telemetry.self_otlp_endpoint = endpoint.to_string();
        config
    }

    #[test]
    fn endpoint_off_and_empty_disable() {
        // SAFETY: single-threaded test; no other thread reads the env here.
        unsafe { std::env::remove_var("PARALLAX_SELF_OTLP") };
        assert_eq!(resolve_endpoint(&config_with("")), None);
        assert_eq!(resolve_endpoint(&config_with("off")), None);
        assert_eq!(
            resolve_endpoint(&config_with("http://localhost:4317")).as_deref(),
            Some("http://localhost:4317"),
        );
    }

    #[test]
    fn env_overrides_config_including_off() {
        // SAFETY: single-threaded test.
        unsafe { std::env::set_var("PARALLAX_SELF_OTLP", "off") };
        assert_eq!(
            resolve_endpoint(&config_with("http://localhost:4317")),
            None
        );
        unsafe { std::env::set_var("PARALLAX_SELF_OTLP", "http://rotel:4317") };
        assert_eq!(
            resolve_endpoint(&config_with("")).as_deref(),
            Some("http://rotel:4317"),
        );
        unsafe { std::env::remove_var("PARALLAX_SELF_OTLP") };
    }

    // The ingest-path suppression (the self → sink → self loop guard) is
    // verified live against a running serve in the validation note — exporting
    // to Parallax's own receiver and asserting only non-ingest spans return.
}
