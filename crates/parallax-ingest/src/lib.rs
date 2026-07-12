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
use parallax_proto::semconv;

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

/// Resolve the run id from resource attributes. Parallax intentionally keeps
/// this to one key so one wrapped command has one lookup id.
fn run_id(resource_attrs: &[KeyValue]) -> Option<String> {
    attr_str(resource_attrs, semconv::PARALLAX_RUN_ID).map(str::to_string)
}

pub fn resource_run_ids(
    request: &ExportTraceServiceRequest,
) -> impl Iterator<Item = (String, u128)> + '_ {
    request.resource_spans.iter().filter_map(|rs| {
        let resource_attrs = rs
            .resource
            .as_ref()
            .map_or(&[][..], |r| r.attributes.as_slice());
        let run_id = run_id(resource_attrs)?;
        let ts = rs
            .scope_spans
            .iter()
            .flat_map(|ss| ss.spans.iter())
            .map(|span| u128::from(span.start_time_unix_nano))
            .min()
            .unwrap_or(0);
        Some((run_id, ts))
    })
}

#[cfg(test)]
mod tests;
