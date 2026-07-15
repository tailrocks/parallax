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

pub mod semconv;

#[cfg(test)]
mod tests {
    use prost::Message;

    use super::collector_trace::ExportTraceServiceRequest;
    use super::common::{AnyValue, InstrumentationScope, KeyValue, any_value};
    use super::resource::Resource;
    use super::trace::{ResourceSpans, ScopeSpans, Span};

    #[test]
    fn generated_constants_survive_otlp_protobuf_roundtrip() {
        let request = ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(Resource {
                    attributes: vec![string_attribute(
                        crate::semconv::SERVICE_NAME,
                        crate::semconv::PLAYGROUND_NAMESPACE,
                    )],
                    dropped_attributes_count: 0,
                    entity_refs: Vec::new(),
                }),
                scope_spans: vec![ScopeSpans {
                    scope: Some(InstrumentationScope::default()),
                    spans: vec![Span {
                        trace_id: vec![1; 16],
                        span_id: vec![2; 8],
                        name: "semconv-wire-contract".into(),
                        attributes: vec![
                            string_attribute(crate::semconv::PARALLAX_RUN_ID, "run-fixture"),
                            string_attribute(crate::semconv::TEST_CASE_RESULT_STATUS, "fail"),
                            string_attribute(crate::semconv::GRAPHQL_FIELD_PATH, "Query.product"),
                        ],
                        ..Span::default()
                    }],
                    schema_url: String::new(),
                }],
                schema_url: String::new(),
            }],
        };

        let encoded = request.encode_to_vec();
        let decoded = ExportTraceServiceRequest::decode(encoded.as_slice()).expect("valid OTLP");
        let resource = decoded.resource_spans[0]
            .resource
            .as_ref()
            .expect("resource");
        assert_eq!(
            string_value(&resource.attributes[0]),
            (
                crate::semconv::SERVICE_NAME,
                crate::semconv::PLAYGROUND_NAMESPACE
            )
        );
        let attributes = &decoded.resource_spans[0].scope_spans[0].spans[0].attributes;
        assert_eq!(
            attributes.iter().map(string_value).collect::<Vec<_>>(),
            vec![
                (crate::semconv::PARALLAX_RUN_ID, "run-fixture"),
                (crate::semconv::TEST_CASE_RESULT_STATUS, "fail"),
                (crate::semconv::GRAPHQL_FIELD_PATH, "Query.product"),
            ]
        );
    }

    fn string_attribute(key: &str, value: &str) -> KeyValue {
        KeyValue {
            key: key.into(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(value.into())),
            }),
            ..KeyValue::default()
        }
    }

    fn string_value(attribute: &KeyValue) -> (&str, &str) {
        let Some(AnyValue {
            value: Some(any_value::Value::StringValue(value)),
        }) = attribute.value.as_ref()
        else {
            panic!("fixture attribute must be a string")
        };
        (&attribute.key, value)
    }
}
