//! Shared telemetry semantic-convention names used by Parallax.
//!
//! This crate is the lowest common dependency for core and storage, so it can
//! hold the wire-name registry without introducing a dependency cycle.

/// `OTel` resource attribute: service name.
pub const SERVICE_NAME: &str = "service.name";
/// `OTel` resource attribute: service namespace.
pub const SERVICE_NAMESPACE: &str = "service.namespace";
/// `OTel` resource attribute: service instance id.
pub const SERVICE_INSTANCE_ID: &str = "service.instance.id";
/// `OTel` resource attribute: service version.
pub const SERVICE_VERSION: &str = "service.version";
/// `OTel` resource attribute: deployment environment name.
pub const DEPLOYMENT_ENVIRONMENT_NAME: &str = "deployment.environment.name";
/// Legacy deployment environment resource attribute.
pub const DEPLOYMENT_ENVIRONMENT: &str = "deployment.environment";
/// `OTel` resource attribute: telemetry SDK language.
pub const TELEMETRY_SDK_LANGUAGE: &str = "telemetry.sdk.language";
/// `OTel` resource attribute: telemetry SDK name.
pub const TELEMETRY_SDK_NAME: &str = "telemetry.sdk.name";
/// `OTel` resource attribute: telemetry SDK version.
pub const TELEMETRY_SDK_VERSION: &str = "telemetry.sdk.version";

/// `OTel` log/event name attribute.
pub const EVENT_NAME: &str = "event.name";
/// Parallax bridge attribute for `OTel` `LogRecord` `ObservedTimestamp` in ns.
pub const LOG_OBSERVED_TS_NANOS: &str = "observed_ts_nanos";
/// `OTel` exception event name.
pub const EXCEPTION_EVENT_NAME: &str = "exception";
/// `OTel` exception type attribute.
pub const EXCEPTION_TYPE: &str = "exception.type";
/// `OTel` exception message attribute.
pub const EXCEPTION_MESSAGE: &str = "exception.message";
/// `OTel` exception stacktrace attribute.
pub const EXCEPTION_STACKTRACE: &str = "exception.stacktrace";
/// `OTel` exception escaped attribute.
pub const EXCEPTION_ESCAPED: &str = "exception.escaped";
/// `OTel` error type attribute.
pub const ERROR_TYPE: &str = "error.type";

/// Parallax run-id resource attribute.
pub const PARALLAX_RUN_ID: &str = "parallax.run.id";
/// Parallax source/lane hint resource attribute.
pub const PARALLAX_SOURCE: &str = "parallax.source";
/// Parallax/jackin operation grouping hint.
pub const JACKIN_OPERATION: &str = "jackin.operation";

/// Preferred request-duration histograms, in lookup order.
pub const REQUEST_DURATION_METRICS: &[&str] =
    &["http.server.request.duration", "rpc.server.duration"];

/// Runtime CPU metric names, in lookup order.
pub const CPU_METRICS: &[&str] = &[
    "process.cpu.utilization",
    "process.cpu.usage",
    "system.cpu.utilization",
];

/// Runtime memory metric names, in lookup order.
pub const MEMORY_METRICS: &[&str] = &[
    "process.memory.usage",
    "process.memory.virtual",
    "system.memory.usage",
];

/// Process/runtime metric windows attached to evidence bundles.
pub const BUNDLE_WINDOW_METRICS: &[&str] = &[
    "process.cpu.utilization",
    "process.memory.usage",
    "tokio.runtime.alive_tasks",
];

/// Greptime JSON-path fragment for a resource attribute.
#[must_use]
pub fn resource_json_path(attr: &str) -> String {
    format!(r#"$."{}""#, attr.replace('"', "\\\""))
}

/// Greptime native column name for a flattened resource attribute.
#[must_use]
pub fn resource_column(attr: &str) -> String {
    format!("resource_attributes.{attr}")
}
