use super::*;

fn has_attr(attributes: &[KeyValue], key: &str) -> bool {
    attributes.iter().any(|kv| kv.key == key)
}

fn string_attr(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(AnyValueEnum::StringValue(value.to_string())),
        }),
        key_strindex: 0,
    }
}

fn int_attr(key: &str, value: u64) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(AnyValueEnum::IntValue(
                i64::try_from(value).unwrap_or(i64::MAX),
            )),
        }),
        key_strindex: 0,
    }
}

/// Mirror top-level OTLP log identity fields into attributes before native
/// GreptimeDB ingest. GreptimeDB v1.1.2/v1.2 nightly do not map
/// `LogRecord.event_name` or `observed_time_unix_nano` to native log columns,
/// but they do persist/extract log attributes into `opentelemetry_logs`.
pub fn promote_log_identity_attributes(request: &mut ExportLogsServiceRequest) -> bool {
    let mut changed = false;
    for resource_logs in &mut request.resource_logs {
        for scope_logs in &mut resource_logs.scope_logs {
            for record in &mut scope_logs.log_records {
                if !record.event_name.is_empty()
                    && !has_attr(&record.attributes, semconv::EVENT_NAME)
                {
                    record
                        .attributes
                        .push(string_attr(semconv::EVENT_NAME, &record.event_name));
                    changed = true;
                }
                if record.observed_time_unix_nano != 0
                    && !has_attr(&record.attributes, semconv::LOG_OBSERVED_TS_NANOS)
                {
                    record.attributes.push(int_attr(
                        semconv::LOG_OBSERVED_TS_NANOS,
                        record.observed_time_unix_nano,
                    ));
                    changed = true;
                }
            }
        }
    }
    changed
}

pub fn normalize_logs(request: &ExportLogsServiceRequest) -> Vec<LogRow> {
    let mut rows = Vec::new();
    for rl in &request.resource_logs {
        let resource_attrs = rl
            .resource
            .as_ref()
            .map_or(&[][..], |r| r.attributes.as_slice());
        let service = service_name(resource_attrs);
        let run_id = run_id(resource_attrs);
        let resource_json = attributes_to_json(resource_attrs);
        for sl in &rl.scope_logs {
            let scope_name = sl
                .scope
                .as_ref()
                .map(|s| s.name.clone())
                .unwrap_or_default();
            for record in &sl.log_records {
                let body = record
                    .body
                    .as_ref()
                    .map(|b| match any_value_to_json(b) {
                        serde_json::Value::String(s) => s,
                        other => other.to_string(),
                    })
                    .unwrap_or_default();
                let ts = if record.time_unix_nano != 0 {
                    record.time_unix_nano
                } else {
                    record.observed_time_unix_nano
                };
                rows.push(LogRow {
                    ts_nanos: u128::from(ts),
                    event_name: record.event_name.clone(),
                    observed_ts_nanos: u128::from(record.observed_time_unix_nano),
                    service: service.clone(),
                    severity_num: record.severity_number,
                    severity_text: record.severity_text.clone(),
                    body,
                    trace_id: hex(&record.trace_id),
                    span_id: hex(&record.span_id),
                    run_id: run_id.clone(),
                    scope_name: scope_name.clone(),
                    attributes: attributes_to_json(&record.attributes),
                    resource: resource_json.clone(),
                });
            }
        }
    }
    rows
}
