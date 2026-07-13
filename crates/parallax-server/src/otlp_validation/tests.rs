use super::trace_ids;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_proto::trace::{ResourceSpans, ScopeSpans, Span};

fn request(trace_id: Vec<u8>) -> ExportTraceServiceRequest {
    ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            scope_spans: vec![ScopeSpans {
                spans: vec![Span {
                    trace_id,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

#[test]
fn otlp_trace_boundary_accepts_only_valid_trace_ids() {
    trace_ids(&request(vec![1; 16])).expect("valid trace id");
    trace_ids(&request(vec![0; 16])).expect_err("zero trace id rejected");
    trace_ids(&request(vec![1; 15])).expect_err("short trace id rejected");
}
