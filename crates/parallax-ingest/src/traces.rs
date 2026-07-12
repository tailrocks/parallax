use super::*;

fn span_kind_name(kind: i32) -> &'static str {
    match kind {
        1 => "SPAN_KIND_INTERNAL",
        2 => "SPAN_KIND_SERVER",
        3 => "SPAN_KIND_CLIENT",
        4 => "SPAN_KIND_PRODUCER",
        5 => "SPAN_KIND_CONSUMER",
        _ => "SPAN_KIND_UNSPECIFIED",
    }
}

fn status_code_name(code: i32) -> &'static str {
    match code {
        1 => "STATUS_CODE_OK",
        2 => "STATUS_CODE_ERROR",
        _ => "STATUS_CODE_UNSET",
    }
}

/// OTel span links → `[{traceId, spanId, attributes}]` JSON. Links are the
/// standard cross-trace correlation: a span references spans in other
/// traces (batch/async sub-operations) without a parent/child edge.
fn links_to_json(links: &[parallax_proto::trace::span::Link]) -> serde_json::Value {
    serde_json::Value::Array(
        links
            .iter()
            .map(|link| {
                serde_json::json!({
                    "traceId": hex(&link.trace_id),
                    "spanId": hex(&link.span_id),
                    "attributes": attributes_to_json(&link.attributes),
                })
            })
            .collect(),
    )
}

pub fn normalize_traces(request: &ExportTraceServiceRequest) -> Vec<SpanRow> {
    let mut rows = Vec::new();
    for rs in &request.resource_spans {
        let resource_attrs = rs
            .resource
            .as_ref()
            .map(|r| r.attributes.as_slice())
            .unwrap_or(&[]);
        let service = service_name(resource_attrs);
        let run_id = run_id(resource_attrs);
        let resource_json = attributes_to_json(resource_attrs);
        for ss in &rs.scope_spans {
            let scope_name = ss
                .scope
                .as_ref()
                .map(|s| s.name.clone())
                .unwrap_or_default();
            for span in &ss.spans {
                let (status_code, status_message) = span
                    .status
                    .as_ref()
                    .map(|s| (status_code_name(s.code), s.message.clone()))
                    .unwrap_or(("STATUS_CODE_UNSET", String::new()));
                rows.push(SpanRow {
                    ts_nanos: u128::from(span.start_time_unix_nano),
                    service: service.clone(),
                    trace_id: hex(&span.trace_id),
                    span_id: hex(&span.span_id),
                    parent_span_id: (!span.parent_span_id.is_empty())
                        .then(|| hex(&span.parent_span_id)),
                    name: span.name.clone(),
                    kind: span_kind_name(span.kind).to_string(),
                    status_code: status_code.to_string(),
                    status_message,
                    duration_ns: u128::from(
                        span.end_time_unix_nano
                            .saturating_sub(span.start_time_unix_nano),
                    ),
                    run_id: run_id.clone(),
                    scope_name: scope_name.clone(),
                    events: None,
                    links: links_to_json(&span.links),
                    attributes: attributes_to_json(&span.attributes),
                    resource: resource_json.clone(),
                });
            }
        }
    }
    rows
}
