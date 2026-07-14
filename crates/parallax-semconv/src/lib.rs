//! Generated semantic-convention names shared by Parallax producers and consumers.
//!
//! Source: `telemetry/semconv/contract.yaml`. Do not edit by hand;
//! run `cargo xtask semconv generate`. Product builds depend only on this
//! dependency-free crate, never on the generator or Weaver.

pub const SERVICE_NAME: &str = "service.name";
pub const SERVICE_NAMESPACE: &str = "service.namespace";
pub const SERVICE_INSTANCE_ID: &str = "service.instance.id";
pub const SERVICE_VERSION: &str = "service.version";
pub const DEPLOYMENT_ENVIRONMENT_NAME: &str = "deployment.environment.name";
pub const DEPLOYMENT_ENVIRONMENT: &str = "deployment.environment";
pub const TELEMETRY_SDK_LANGUAGE: &str = "telemetry.sdk.language";
pub const TELEMETRY_SDK_NAME: &str = "telemetry.sdk.name";
pub const TELEMETRY_SDK_VERSION: &str = "telemetry.sdk.version";
pub const EVENT_NAME: &str = "event.name";
pub const LOG_OBSERVED_TS_NANOS: &str = "observed_ts_nanos";
pub const EXCEPTION_EVENT_NAME: &str = "exception";
pub const EXCEPTION_TYPE: &str = "exception.type";
pub const EXCEPTION_MESSAGE: &str = "exception.message";
pub const EXCEPTION_STACKTRACE: &str = "exception.stacktrace";
pub const EXCEPTION_ESCAPED: &str = "exception.escaped";
pub const ERROR_TYPE: &str = "error.type";
pub const PARALLAX_RUN_ID: &str = "parallax.run.id";
pub const PARALLAX_SOURCE: &str = "parallax.source";
pub const JACKIN_OPERATION: &str = "jackin.operation";
pub const REQUEST_DURATION_METRICS: &[&str] = &["http.server.request.duration", "rpc.server.duration", ];
pub const CPU_METRICS: &[&str] = &["process.cpu.utilization", "process.cpu.usage", "system.cpu.utilization", ];
pub const MEMORY_METRICS: &[&str] = &["process.memory.usage", "process.memory.virtual", "system.memory.usage", ];
pub const BUNDLE_WINDOW_METRICS: &[&str] = &["process.cpu.utilization", "process.memory.usage", "tokio.runtime.alive_tasks", ];

#[must_use]
pub fn resource_json_path(attr: &str) -> String {
    format!(r#"$.\"{}\""#, attr.replace('"', "\\\""))
}

#[must_use]
pub fn resource_column(attr: &str) -> String {
    format!("resource_attributes.{attr}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_load_bearing_wire_names() -> Result<(), String> {
        let actual = (SERVICE_NAME, EVENT_NAME, PARALLAX_RUN_ID, BUNDLE_WINDOW_METRICS);
        let expected = (
            "service.name",
            "event.name",
            "parallax.run.id",
            &["process.cpu.utilization", "process.memory.usage", "tokio.runtime.alive_tasks"][..],
        );
        if actual != expected {
            return Err(format!("semantic-convention wire-name drift: {actual:?}"));
        }
        Ok(())
    }
}
