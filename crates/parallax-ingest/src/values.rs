use parallax_proto::common::any_value::Value as AnyValueEnum;
use parallax_proto::common::{AnyValue, KeyValue};

pub(super) fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

pub(super) fn any_value_to_json(value: &AnyValue) -> serde_json::Value {
    match &value.value {
        Some(AnyValueEnum::StringValue(value)) => serde_json::Value::String(value.clone()),
        Some(AnyValueEnum::BoolValue(value)) => serde_json::Value::Bool(*value),
        Some(AnyValueEnum::IntValue(value)) => serde_json::Value::from(*value),
        Some(AnyValueEnum::DoubleValue(value)) => serde_json::Number::from_f64(*value)
            .map_or(serde_json::Value::Null, serde_json::Value::Number),
        Some(AnyValueEnum::ArrayValue(items)) => {
            serde_json::Value::Array(items.values.iter().map(any_value_to_json).collect())
        }
        Some(AnyValueEnum::KvlistValue(values)) => attributes_to_json(&values.values),
        Some(AnyValueEnum::BytesValue(value)) => serde_json::Value::String(hex(value)),
        // Indexed values need their string-table context; SDK exports do not use them.
        Some(_) | None => serde_json::Value::Null,
    }
}

pub(super) fn attributes_to_json(attributes: &[KeyValue]) -> serde_json::Value {
    serde_json::Value::Object(
        attributes
            .iter()
            .map(|item| {
                (
                    item.key.clone(),
                    item.value
                        .as_ref()
                        .map_or(serde_json::Value::Null, any_value_to_json),
                )
            })
            .collect(),
    )
}

pub(super) fn attr_str<'a>(attributes: &'a [KeyValue], key: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|item| item.key == key)
        .and_then(|item| match item.value.as_ref()?.value.as_ref()? {
            AnyValueEnum::StringValue(value) => Some(value.as_str()),
            _ => None,
        })
}
