use prost::Message;

use super::collector_trace::ExportTraceServiceRequest;
use super::common::{AnyValue, InstrumentationScope, KeyValue, any_value};
use super::resource::Resource;
use super::trace::{ResourceSpans, ScopeSpans, Span};

#[test]
fn generated_constants_survive_otlp_protobuf_roundtrip() -> Result<(), String> {
    let request = ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(Resource {
                attributes: vec![string_attribute(
                    parallax_semconv::SERVICE_NAME,
                    parallax_semconv::PLAYGROUND_NAMESPACE,
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
                        string_attribute(parallax_semconv::CLI_INVOCATION_ID, "run-fixture"),
                        string_attribute(parallax_semconv::TEST_CASE_RESULT_STATUS, "fail"),
                        string_attribute(parallax_semconv::GRAPHQL_FIELD_PATH, "Query.product"),
                    ],
                    ..Span::default()
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    };

    let encoded = request.encode_to_vec();
    let decoded =
        ExportTraceServiceRequest::decode(encoded.as_slice()).map_err(|error| error.to_string())?;
    let resource = decoded.resource_spans[0]
        .resource
        .as_ref()
        .ok_or("missing resource")?;
    let actual_resource = string_value(&resource.attributes[0])?;
    let expected_resource = (
        parallax_semconv::SERVICE_NAME,
        parallax_semconv::PLAYGROUND_NAMESPACE,
    );
    let attributes = &decoded.resource_spans[0].scope_spans[0].spans[0].attributes;
    let actual_attributes = attributes
        .iter()
        .map(string_value)
        .collect::<Result<Vec<_>, _>>()?;
    let expected_attributes = vec![
        (parallax_semconv::CLI_INVOCATION_ID, "run-fixture"),
        (parallax_semconv::TEST_CASE_RESULT_STATUS, "fail"),
        (parallax_semconv::GRAPHQL_FIELD_PATH, "Query.product"),
    ];
    if actual_resource != expected_resource || actual_attributes != expected_attributes {
        return Err(format!(
            "OTLP semantic-convention round trip drift: resource={actual_resource:?}, attributes={actual_attributes:?}"
        ));
    }
    Ok(())
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

fn string_value(attribute: &KeyValue) -> Result<(&str, &str), String> {
    let Some(AnyValue {
        value: Some(any_value::Value::StringValue(value)),
    }) = attribute.value.as_ref()
    else {
        return Err("fixture attribute must be a string".into());
    };
    Ok((&attribute.key, value))
}
