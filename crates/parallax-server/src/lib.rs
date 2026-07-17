//! Parallax server library.
#![cfg_attr(test, allow(clippy::float_cmp, reason = "exact fixture arithmetic"))]
//!
//! Hosts the OTLP receivers (gRPC :4317, HTTP on the API port), the GraphQL
//! API, and (from M1) the workers and engine supervision. The installed
//! `parallax` binary (crate `parallax-cli`) embeds this library for the
//! `serve` subcommand.
//!
//! Supported lifecycle paths are available from the crate root:
//!
//! ```
//! use parallax_server::{Config, ServerHandle, start};
//! # let _ = (Config::default(), start);
//! # fn accepts(_: Option<ServerHandle>) {}
//! ```
//!
//! Implementation modules are intentionally private:
//!
//! ```compile_fail
//! use parallax_server::worker::Worker;
//! ```
//!
//! ```compile_fail
//! use parallax_server::self_telemetry::Installed;
//! ```

mod alerting;
mod config;
mod engine_io;
mod errors;
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "progress math"
)]
#[expect(
    clippy::excessive_nesting,
    clippy::too_many_lines,
    reason = "engine lifecycle"
)]
mod greptime_supervisor;
mod ingest_health;
mod ingest_runtime;
mod live;
mod otlp_grpc;
mod otlp_http;
mod otlp_validation;
mod outcomes;
mod self_telemetry;
mod serve;
mod worker;

pub use config::{
    AlertingConfig, Config, LimitsConfig, RetentionConfig, ServerConfig, StorageConfig,
    TelemetryConfig,
};
pub use errors::{
    ConfigError, ConfigErrorKind, ConfigResult, ServerError, ServerErrorKind, ServerResult,
};
pub use greptime_supervisor::{GreptimeSupervisor, ensure_binary as ensure_greptime_binary};
pub use self_telemetry::{
    Installed as InstalledSelfTelemetry, SelfTelemetry, install as install_self_telemetry,
    resolve_endpoint as resolve_self_telemetry_endpoint,
};
pub use serve::{ServerHandle, start, start_with_capabilities};
