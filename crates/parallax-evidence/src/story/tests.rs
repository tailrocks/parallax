use super::*;
use serde_json::json;

fn span(id: &str, parent: Option<&str>, ts: u128, duration: u128, service: &str) -> SpanRow {
    SpanRow {
        ts_nanos: ts,
        service: service.to_string(),
        trace_id: "trace-a".to_string(),
        span_id: id.to_string(),
        parent_span_id: parent.map(str::to_string),
        name: format!("span {id}"),
        kind: "SPAN_KIND_INTERNAL".to_string(),
        status_code: "STATUS_CODE_UNSET".to_string(),
        status_message: String::new(),
        duration_ns: duration,
        run_id: Some("run-a".to_string()),
        scope_name: "test".to_string(),
        events: None,
        links: json!([]),
        attributes: json!({}),
        resource: json!({}),
    }
}

fn log(ts: u128, severity_num: i32, severity_text: &str, body: &str, span_id: &str) -> LogRow {
    LogRow {
        ts_nanos: ts,
        event_name: String::new(),
        observed_ts_nanos: 0,
        service: "api".to_string(),
        severity_num,
        severity_text: severity_text.to_string(),
        body: body.to_string(),
        trace_id: "trace-a".to_string(),
        span_id: span_id.to_string(),
        run_id: Some("run-a".to_string()),
        scope_name: "test".to_string(),
        attributes: json!({}),
        resource: json!({}),
    }
}

#[test]
fn parent_start_precedes_clock_skewed_child() {
    let root = span("root", None, 100, 50, "api");
    let child = span("child", Some("root"), 90, 20, "db");

    let beats = project_story(&[child, root], &[], &[]);
    let starts: Vec<_> = beats
        .iter()
        .filter(|beat| beat.kind == "span.start")
        .map(|beat| beat.span_id.as_deref().unwrap_or(""))
        .collect();

    assert_eq!(starts, vec!["root", "child"]);
}

#[test]
fn lanes_group_by_emitting_service() {
    let beats = project_story(
        &[
            span("root", None, 100, 50, "api"),
            span("db", Some("root"), 110, 10, "db"),
        ],
        &[],
        &[],
    );

    assert!(beats.iter().any(|beat| beat.lane == "api"));
    assert!(beats.iter().any(|beat| beat.lane == "db"));
}

#[test]
fn events_and_errors_become_beats() {
    let mut root = span("root", None, 100, 50, "api");
    root.status_code = "STATUS_CODE_ERROR".to_string();
    root.events = Some(
        r#"[{"name":"exception","timeUnixNano":"125","attributes":{"exception.message":"boom"}}]"#
            .to_string(),
    );
    let logs = vec![log(
        130,
        17,
        "ERROR",
        "payment 123 failed\nfull body",
        "root",
    )];

    let beats = project_story(&[root], &logs, &[]);

    assert!(
        beats
            .iter()
            .any(|beat| beat.kind == "event" && beat.title == "exception")
    );
    assert!(
        beats
            .iter()
            .any(|beat| beat.kind == "error" && beat.title == "span root error")
    );
    assert!(
        beats
            .iter()
            .any(|beat| beat.kind == "error" && beat.title == "ERROR payment <n> failed")
    );
}

#[test]
fn output_is_deterministic() {
    let spans = vec![
        span("root", None, 100, 50, "api"),
        span("child-a", Some("root"), 110, 10, "db"),
        span("child-b", Some("root"), 110, 10, "cache"),
    ];
    let logs = vec![log(115, 9, "INFO", "cache hit id=123", "child-b")];

    assert_eq!(
        project_story(&spans, &logs, &[]),
        project_story(&spans, &logs, &[])
    );
}
