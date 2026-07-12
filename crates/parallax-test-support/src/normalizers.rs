use parallax_model::{LogRow, SpanRow};
use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;

pub(super) type TraceNormalizer =
    std::sync::Arc<dyn Fn(&ExportTraceServiceRequest) -> Vec<SpanRow> + Send + Sync>;
pub(super) type LogNormalizer =
    std::sync::Arc<dyn Fn(&ExportLogsServiceRequest) -> Vec<LogRow> + Send + Sync>;
