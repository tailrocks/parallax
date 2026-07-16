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
        invocation_id: None,
        session_id: None,
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
                {"name":"rpc.message","timeUnixNano":10,"attributes":{"id":7}},
                {"name":"native.message","time":"2026-07-10 06:31:40.754723148+0000","attributes":{"id":8}}
            ]"#,
        ),
    )];

    let result = trace_events(&spans, None, 10);

    assert_eq!(result.skipped_spans, 0);
    assert_eq!(result.total_matching, 3);
    assert_eq!(
        result
            .events
            .iter()
            .map(|event| event.name.as_str())
            .collect::<Vec<_>>(),
        vec!["rpc.message", "exception", "native.message"]
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
    assert_eq!(result.events[2].time_unix_nano, 1_783_665_100_754_723_148);
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
