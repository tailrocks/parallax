//! OTLP protocol types for Parallax.
//!
//! Re-exports the generated `opentelemetry-proto` types (tonic services +
//! serde-serializable messages) so the rest of the workspace depends on one
//! pinned protocol surface.

pub use opentelemetry_proto::tonic::collector::logs::v1 as collector_logs;
pub use opentelemetry_proto::tonic::collector::metrics::v1 as collector_metrics;
pub use opentelemetry_proto::tonic::collector::trace::v1 as collector_trace;
pub use opentelemetry_proto::tonic::common::v1 as common;
pub use opentelemetry_proto::tonic::logs::v1 as logs;
pub use opentelemetry_proto::tonic::metrics::v1 as metrics;
pub use opentelemetry_proto::tonic::resource::v1 as resource;
pub use opentelemetry_proto::tonic::trace::v1 as trace;
