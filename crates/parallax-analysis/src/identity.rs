//! Signal/resource identity resolution shared by derived telemetry views.

use parallax_proto::common::KeyValue;
use parallax_proto::common::any_value::Value as AnyValueEnum;
use parallax_proto::trace::ResourceSpans;
use parallax_semconv as semconv;

fn attr_str<'a>(attributes: &'a [KeyValue], key: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|attribute| attribute.key == key)
        .and_then(|item| match item.value.as_ref()?.value.as_ref()? {
            AnyValueEnum::StringValue(value) => Some(value.as_str()),
            _ => None,
        })
}

pub(super) fn invocation_id(
    signal_attrs: &[KeyValue],
    resource_attrs: &[KeyValue],
) -> Option<String> {
    attr_str(signal_attrs, semconv::CLI_INVOCATION_ID)
        .or_else(|| attr_str(resource_attrs, semconv::CLI_INVOCATION_ID))
        .map(str::to_string)
}

pub(super) fn session_id(signal_attrs: &[KeyValue], resource_attrs: &[KeyValue]) -> Option<String> {
    attr_str(signal_attrs, semconv::SESSION_ID)
        .or_else(|| attr_str(resource_attrs, semconv::SESSION_ID))
        .map(str::to_string)
}

pub(super) fn service_version(resource_attrs: &[KeyValue]) -> Option<String> {
    attr_str(resource_attrs, semconv::SERVICE_VERSION).map(str::to_string)
}

pub(super) fn environment(resource_attrs: &[KeyValue]) -> Option<String> {
    attr_str(resource_attrs, semconv::DEPLOYMENT_ENVIRONMENT_NAME)
        .or_else(|| attr_str(resource_attrs, semconv::DEPLOYMENT_ENVIRONMENT))
        .map(str::to_string)
}

pub(super) fn root_span_attrs(rs: &ResourceSpans) -> &[KeyValue] {
    rs.scope_spans
        .iter()
        .flat_map(|ss| ss.spans.iter())
        .find(|span| span.parent_span_id.is_empty())
        .map_or(&[][..], |span| span.attributes.as_slice())
}
