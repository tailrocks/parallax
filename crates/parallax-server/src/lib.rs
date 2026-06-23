//! Parallax server library.
//!
//! Hosts the OTLP receivers (gRPC :4317, HTTP on the API port), the GraphQL
//! API, and (from M1) the workers and engine supervision. The installed
//! `parallax` binary (crate `parallax-cli`) embeds this library for the
//! `serve` subcommand.

pub mod config;
pub mod greptime_supervisor;
pub mod live;
pub mod otlp_grpc;
pub mod otlp_http;
pub mod self_telemetry;
pub mod serve;
pub mod worker;

pub use config::Config;
pub use serve::{ServerHandle, start};
