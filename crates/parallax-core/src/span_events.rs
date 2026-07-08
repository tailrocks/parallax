//! Trace-wide span event parsing over normalized span rows.

use parallax_storage::model::SpanRow;
use serde_json::Value;

/// One parsed span event, joined to its carrying span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEvent {
    pub span_id: String,
    pub span_name: String,
    pub service: String,
    pub name: String,
    pub time_unix_nano: u128,
    /// Flat string map; non-string values JSON-stringified.
    pub attributes: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEvents {
    pub events: Vec<TraceEvent>,
    pub skipped_spans: usize,
    pub total_matching: usize,
}

impl TraceEvents {
    pub fn truncated(&self) -> bool {
        self.total_matching > self.events.len()
    }
}

/// Parse every span's `events` JSON, filter, sort by time ascending, cap.
///
/// Malformed JSON or malformed event-array shape on one span is skipped and
/// counted; individual malformed events are skipped without failing the span.
pub fn trace_events(spans: &[SpanRow], name_prefix: Option<&str>, limit: usize) -> TraceEvents {
    let mut events = Vec::new();
    let mut skipped_spans = 0;

    for span in spans {
        let Some(raw_events) = span.events.as_deref() else {
            continue;
        };
        let parsed: Value = match serde_json::from_str(raw_events) {
            Ok(value) => value,
            Err(_) => {
                skipped_spans += 1;
                continue;
            }
        };
        let Some(array) = parsed.as_array() else {
            skipped_spans += 1;
            continue;
        };

        for event in array {
            let Some(name) = event.get("name").and_then(Value::as_str) else {
                continue;
            };
            if name_prefix.is_some_and(|prefix| !name.starts_with(prefix)) {
                continue;
            }
            let Some(time_unix_nano) = event_time_unix_nano(event) else {
                continue;
            };
            let attributes = event
                .get("attributes")
                .and_then(Value::as_object)
                .map(|object| {
                    let mut attributes: Vec<_> = object
                        .iter()
                        .map(|(key, value)| {
                            (
                                key.clone(),
                                value
                                    .as_str()
                                    .map(str::to_string)
                                    .unwrap_or_else(|| value.to_string()),
                            )
                        })
                        .collect();
                    attributes.sort_by(|(a, _), (b, _)| a.cmp(b));
                    attributes
                })
                .unwrap_or_default();
            events.push(TraceEvent {
                span_id: span.span_id.clone(),
                span_name: span.name.clone(),
                service: span.service.clone(),
                name: name.to_string(),
                time_unix_nano,
                attributes,
            });
        }
    }

    events.sort_by(|left, right| {
        left.time_unix_nano
            .cmp(&right.time_unix_nano)
            .then_with(|| left.span_id.cmp(&right.span_id))
            .then_with(|| left.name.cmp(&right.name))
    });
    let total_matching = events.len();
    events.truncate(limit);

    TraceEvents {
        events,
        skipped_spans,
        total_matching,
    }
}

fn event_time_unix_nano(event: &Value) -> Option<u128> {
    event
        .get("time_unix_nano")
        .or_else(|| event.get("timeUnixNano"))
        .and_then(|value| {
            value
                .as_u64()
                .map(u128::from)
                .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(span_id: &str, events: Option<&str>) -> SpanRow {
        SpanRow {
            ts_nanos: 0,
            service: "checkout".into(),
            trace_id: "trace-a".into(),
            span_id: span_id.into(),
            parent_span_id: None,
            name: format!("span-{span_id}"),
            kind: "SPAN_KIND_INTERNAL".into(),
            status_code: "STATUS_CODE_UNSET".into(),
            status_message: String::new(),
            duration_ns: 1,
            run_id: None,
            scope_name: String::new(),
            events: events.map(str::to_string),
            links: Value::Null,
            attributes: Value::Null,
            resource: Value::Null,
        }
    }

    #[test]
    fn accepts_timestamp_spellings_and_stringifies_attributes() {
        let spans = vec![span(
            "a",
            Some(
                r#"[
                    {"name":"exception","time_unix_nano":"20","attributes":{"message":"bad","retry":true}},
                    {"name":"rpc.message","timeUnixNano":10,"attributes":{"id":7}}
                ]"#,
            ),
        )];

        let result = trace_events(&spans, None, 10);

        assert_eq!(result.skipped_spans, 0);
        assert_eq!(result.total_matching, 2);
        assert_eq!(
            result
                .events
                .iter()
                .map(|event| event.name.as_str())
                .collect::<Vec<_>>(),
            vec!["rpc.message", "exception"]
        );
        assert_eq!(
            result.events[0].attributes,
            vec![("id".to_string(), "7".to_string())]
        );
        assert_eq!(
            result.events[1].attributes,
            vec![
                ("message".to_string(), "bad".to_string()),
                ("retry".to_string(), "true".to_string())
            ]
        );
    }

    #[test]
    fn filters_by_prefix_and_counts_malformed_spans() {
        let spans = vec![
            span(
                "a",
                Some(
                    r#"[
                        {"name":"rpc.message.sent","time_unix_nano":30,"attributes":{}},
                        {"name":"exception","time_unix_nano":20,"attributes":{}}
                    ]"#,
                ),
            ),
            span("bad", Some("{not json")),
        ];

        let result = trace_events(&spans, Some("rpc.message"), 10);

        assert_eq!(result.skipped_spans, 1);
        assert_eq!(result.total_matching, 1);
        assert_eq!(result.events[0].name, "rpc.message.sent");
        assert!(!result.truncated());
    }

    #[test]
    fn caps_after_sort_and_reports_truncated() {
        let spans = vec![span(
            "a",
            Some(
                r#"[
                    {"name":"event.3","time_unix_nano":30,"attributes":{}},
                    {"name":"event.1","time_unix_nano":10,"attributes":{}},
                    {"name":"event.2","time_unix_nano":20,"attributes":{}}
                ]"#,
            ),
        )];

        let result = trace_events(&spans, None, 2);

        assert_eq!(result.total_matching, 3);
        assert!(result.truncated());
        assert_eq!(
            result
                .events
                .iter()
                .map(|event| event.name.as_str())
                .collect::<Vec<_>>(),
            vec!["event.1", "event.2"]
        );
    }

    #[test]
    fn deterministic_order_uses_time_then_span_then_name() {
        let spans = vec![
            span(
                "b",
                Some(r#"[{"name":"second","time_unix_nano":10,"attributes":{}}]"#),
            ),
            span(
                "a",
                Some(
                    r#"[
                        {"name":"z","time_unix_nano":10,"attributes":{}},
                        {"name":"a","time_unix_nano":10,"attributes":{}}
                    ]"#,
                ),
            ),
        ];

        let result = trace_events(&spans, None, 10);

        assert_eq!(
            result
                .events
                .iter()
                .map(|event| (event.span_id.as_str(), event.name.as_str()))
                .collect::<Vec<_>>(),
            vec![("a", "a"), ("a", "z"), ("b", "second")]
        );
    }
}
