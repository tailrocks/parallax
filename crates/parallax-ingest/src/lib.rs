//! OTLP → normalized rows, per the implementation-spec §7 mapping.
#![expect(
    clippy::cast_precision_loss,
    clippy::excessive_nesting,
    reason = "mechanically transferred OTLP projection; Plan 098 owns the split"
)]
#![cfg_attr(test, allow(clippy::float_cmp, reason = "exact fixture arithmetic"))]

use parallax_model::{HistogramRow, LogRow, MetricExemplarRow, MetricPointRow, SpanRow};
use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_proto::common::any_value::Value as AnyValueEnum;
use parallax_proto::common::{AnyValue, KeyValue};
use parallax_proto::metrics::exemplar::Value as ExemplarValue;
use parallax_proto::metrics::metric::Data;
use parallax_proto::metrics::number_data_point::Value as NumberValue;
use parallax_semconv as semconv;

mod logs;
mod metrics;
mod traces;
mod values;

pub use logs::{normalize_logs, promote_log_identity_attributes};
pub use metrics::{NormalizedMetrics, normalize_metrics};
pub use traces::normalize_traces;

use values::{any_value_to_json, attr_str, attributes_to_json, hex};

fn service_name(resource_attrs: &[KeyValue]) -> String {
    attr_str(resource_attrs, semconv::SERVICE_NAME)
        .unwrap_or("unknown")
        .to_string()
}

/// Resolve the CLI invocation id. Priority: explicit span/log attribute
/// (the jackin shape — ids never live on Resource there), then resource
/// attribute (generic wrapped emitters). No legacy key is consulted.
fn invocation_id(signal_attrs: &[KeyValue], resource_attrs: &[KeyValue]) -> Option<String> {
    attr_str(signal_attrs, semconv::CLI_INVOCATION_ID)
        .or_else(|| attr_str(resource_attrs, semconv::CLI_INVOCATION_ID))
        .map(str::to_string)
}

/// Resolve the interactive session id with the same signal-then-resource
/// priority as [`invocation_id`].
fn session_id(signal_attrs: &[KeyValue], resource_attrs: &[KeyValue]) -> Option<String> {
    attr_str(signal_attrs, semconv::SESSION_ID)
        .or_else(|| attr_str(resource_attrs, semconv::SESSION_ID))
        .map(str::to_string)
}

/// Attributes of the root span (no parent) in one resource-spans group; the
/// group's identity source when ids are stamped on root spans, not Resource.
fn root_span_attrs(rs: &parallax_proto::trace::ResourceSpans) -> &[KeyValue] {
    rs.scope_spans
        .iter()
        .flat_map(|ss| ss.spans.iter())
        .find(|span| span.parent_span_id.is_empty())
        .map_or(&[][..], |span| span.attributes.as_slice())
}

pub fn resource_invocation_ids(
    request: &ExportTraceServiceRequest,
) -> impl Iterator<Item = (String, u128)> + '_ {
    request.resource_spans.iter().filter_map(|rs| {
        let resource_attrs = rs
            .resource
            .as_ref()
            .map_or(&[][..], |r| r.attributes.as_slice());
        let invocation_id = invocation_id(root_span_attrs(rs), resource_attrs)?;
        let ts = rs
            .scope_spans
            .iter()
            .flat_map(|ss| ss.spans.iter())
            .map(|span| u128::from(span.start_time_unix_nano))
            .min()
            .unwrap_or(0);
        Some((invocation_id, ts))
    })
}

#[cfg(test)]
mod tests;
