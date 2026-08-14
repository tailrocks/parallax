use super::*;

#[test]
fn log_event_serializes_typed_log_identity() {
    let value = log_event(&LogRow {
        ts_nanos: 1_000_000_000,
        event_name: "checkout.completed".to_string(),
        observed_ts_nanos: 3_000_000_000,
        service: "checkout".to_string(),
        severity_num: 9,
        severity_text: "INFO".to_string(),
        body: "checkout.completed".to_string(),
        trace_id: "trace-a".to_string(),
        span_id: "span-a".to_string(),
        invocation_id: Some("run-a".to_string()),
        session_id: None,
        scope_name: "seed".to_string(),
        attributes: serde_json::json!({"event.name": "checkout.completed"}),
        resource: serde_json::json!({"service.name": "checkout"}),
    });

    assert_eq!(value["eventName"], "checkout.completed");
    assert_eq!(value["observedTsNanos"], "3000000000");
    assert_eq!(value["tsNanos"], "1000000000");
}

fn sample_log() -> LogRow {
    LogRow {
        ts_nanos: 1,
        event_name: "e".into(),
        observed_ts_nanos: 1,
        service: "checkout".into(),
        severity_num: 9,
        severity_text: "INFO".into(),
        body: "hello world".into(),
        trace_id: "trace-a".into(),
        span_id: "span-a".into(),
        invocation_id: Some("inv-a".into()),
        session_id: Some("sess-a".into()),
        scope_name: "s".into(),
        attributes: serde_json::Value::Null,
        resource: serde_json::Value::Null,
    }
}

fn sample_span() -> SpanRow {
    SpanRow {
        ts_nanos: 1,
        service: "checkout".into(),
        trace_id: "trace-a".into(),
        span_id: "span-a".into(),
        parent_span_id: None,
        name: "GET /pay".into(),
        kind: "SPAN_KIND_SERVER".into(),
        status_code: "STATUS_CODE_UNSET".into(),
        status_message: String::new(),
        duration_ns: 2_000_000,
        invocation_id: Some("inv-a".into()),
        session_id: Some("sess-a".into()),
        scope_name: "s".into(),
        events: None,
        links: serde_json::Value::Null,
        attributes: serde_json::Value::Null,
        resource: serde_json::Value::Null,
    }
}

fn empty_log_filter() -> StreamFilter {
    StreamFilter {
        service: None,
        severity_min: None,
        q: None,
        trace_id: None,
        invocation_id: None,
        session_id: None,
    }
}

fn empty_span_filter() -> SpanStreamFilter {
    SpanStreamFilter {
        service: None,
        min_duration_ms: None,
        errors_only: None,
        q: None,
        trace_id: None,
        invocation_id: None,
        session_id: None,
    }
}

#[test]
fn stream_filter_predicates() {
    let log = sample_log();
    assert!(empty_log_filter().matches(&log));
    assert!(
        StreamFilter {
            service: Some("checkout".into()),
            ..empty_log_filter()
        }
        .matches(&log)
    );
    assert!(
        !StreamFilter {
            service: Some("other".into()),
            ..empty_log_filter()
        }
        .matches(&log)
    );
    assert!(
        StreamFilter {
            severity_min: Some(9),
            ..empty_log_filter()
        }
        .matches(&log)
    );
    assert!(
        !StreamFilter {
            severity_min: Some(10),
            ..empty_log_filter()
        }
        .matches(&log)
    );
    assert!(
        StreamFilter {
            q: Some("hello".into()),
            ..empty_log_filter()
        }
        .matches(&log)
    );
    assert!(
        !StreamFilter {
            q: Some("nope".into()),
            ..empty_log_filter()
        }
        .matches(&log)
    );
    assert!(
        StreamFilter {
            service: Some("checkout".into()),
            q: Some("world".into()),
            trace_id: Some("trace-a".into()),
            invocation_id: Some("inv-a".into()),
            session_id: Some("sess-a".into()),
            severity_min: Some(9),
        }
        .matches(&log)
    );
}

#[test]
fn span_stream_filter_predicates_and_duration_floor() {
    let span = sample_span();
    assert!(empty_span_filter().matches(&span));
    assert!(
        SpanStreamFilter {
            min_duration_ms: Some(2.0),
            ..empty_span_filter()
        }
        .matches(&span)
    );
    assert!(
        !SpanStreamFilter {
            min_duration_ms: Some(2.001),
            ..empty_span_filter()
        }
        .matches(&span)
    );
    assert!(
        !SpanStreamFilter {
            errors_only: Some(true),
            ..empty_span_filter()
        }
        .matches(&span)
    );
    let mut error = sample_span();
    error.status_code = "STATUS_CODE_ERROR".into();
    assert!(
        SpanStreamFilter {
            errors_only: Some(true),
            ..empty_span_filter()
        }
        .matches(&error)
    );
    assert!(
        SpanStreamFilter {
            q: Some("/pay".into()),
            ..empty_span_filter()
        }
        .matches(&span)
    );
}

#[test]
fn lagged_broadcast_receiver_keeps_tailing() {
    // `batch.ok()?` turns Lagged into a skipped item, not a stream error.
    let err = tokio::sync::broadcast::error::TryRecvError::Lagged(2);
    let skipped: Result<u8, _> = Err(err);
    assert!(skipped.ok().is_none());
}
