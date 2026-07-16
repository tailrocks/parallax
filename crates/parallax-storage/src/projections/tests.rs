use super::*;

fn log(
    ts: u128,
    event: &str,
    session: Option<&str>,
    attributes: serde_json::Value,
) -> LogRow {
    LogRow {
        ts_nanos: ts,
        event_name: event.to_string(),
        observed_ts_nanos: ts,
        service: "cli".to_string(),
        severity_num: 9,
        severity_text: "INFO".to_string(),
        body: String::new(),
        trace_id: String::new(),
        span_id: String::new(),
        invocation_id: Some("inv-1".to_string()),
        session_id: session.map(str::to_string),
        scope_name: String::new(),
        attributes,
        resource: serde_json::json!({}),
    }
}

fn span(
    ts: u128,
    name: &str,
    kind: &str,
    parent: Option<&str>,
    status: &str,
    attributes: serde_json::Value,
) -> SpanRow {
    SpanRow {
        ts_nanos: ts,
        service: "cli".to_string(),
        trace_id: format!("trace-{ts}"),
        span_id: format!("span-{ts}"),
        parent_span_id: parent.map(str::to_string),
        name: name.to_string(),
        kind: kind.to_string(),
        status_code: status.to_string(),
        status_message: String::new(),
        duration_ns: 1_000_000,
        invocation_id: Some("inv-1".to_string()),
        session_id: None,
        scope_name: String::new(),
        events: None,
        links: serde_json::json!([]),
        attributes,
        resource: serde_json::json!({}),
    }
}

#[test]
fn sessions_pair_start_and_end_and_leave_open_sessions() {
    let rows = vec![
        log(10, "session.start", Some("s1"), serde_json::json!({})),
        log(20, "session.end", Some("s1"), serde_json::json!({})),
        log(
            30,
            "session.start",
            Some("s2"),
            serde_json::json!({"session.previous_id": "s1"}),
        ),
        log(5, "other.event", Some("sX"), serde_json::json!({})),
    ];
    let sessions = pair_sessions(&rows, 10);
    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].session_id, "s1");
    assert_eq!(sessions[0].end_nanos, Some(20));
    assert_eq!(sessions[1].session_id, "s2");
    assert_eq!(sessions[1].previous_session_id.as_deref(), Some("s1"));
    assert_eq!(sessions[1].end_nanos, None);
}

#[test]
fn screen_visits_pair_by_visit_id_and_filter_by_session() {
    let rows = vec![
        log(
            10,
            "ui.screen.entered",
            Some("s1"),
            serde_json::json!({
                "ui.screen.visit.id": "v1", "app.screen.id": "home",
                "ui.navigation.sequence": 1
            }),
        ),
        log(
            20,
            "ui.screen.exited",
            Some("s1"),
            serde_json::json!({"ui.screen.visit.id": "v1"}),
        ),
        log(
            30,
            "ui.screen.entered",
            Some("s2"),
            serde_json::json!({
                "ui.screen.visit.id": "v2", "app.screen.id": "settings",
                "ui.navigation.sequence": 2, "ui.transition.reason": "user_navigation"
            }),
        ),
    ];
    let all = pair_screen_visits(&rows, None, 10);
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].screen_id, "home");
    assert_eq!(all[0].exited_nanos, Some(20));
    assert_eq!(all[1].exited_nanos, None);
    assert_eq!(all[1].transition_reason.as_deref(), Some("user_navigation"));
    let scoped = pair_screen_visits(&rows, Some("s2"), 10);
    assert_eq!(scoped.len(), 1);
    assert_eq!(scoped[0].screen_id, "settings");
}

#[test]
fn ui_actions_are_root_spans_named_ui_action() {
    let spans = vec![
        span(
            10,
            "ui.action",
            "SPAN_KIND_INTERNAL",
            None,
            "STATUS_CODE_OK",
            serde_json::json!({"ui.action.name": "submit", "app.screen.id": "home",
                              "outcome": "success"}),
        ),
        span(
            20,
            "ui.action",
            "SPAN_KIND_INTERNAL",
            Some("parent"),
            "STATUS_CODE_OK",
            serde_json::json!({}),
        ),
        span(
            30,
            "other",
            "SPAN_KIND_INTERNAL",
            None,
            "STATUS_CODE_OK",
            serde_json::json!({}),
        ),
    ];
    let actions = project_ui_actions(&spans, 10);
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].name, "submit");
    assert_eq!(actions[0].outcome.as_deref(), Some("success"));
}

#[test]
fn background_cycles_group_and_aggregate() {
    let mut spans = vec![
        span(
            10,
            "background.cycle",
            "SPAN_KIND_INTERNAL",
            None,
            "STATUS_CODE_OK",
            serde_json::json!({"background.cycle.name": "sync"}),
        ),
        span(
            20,
            "background.cycle",
            "SPAN_KIND_INTERNAL",
            None,
            "STATUS_CODE_ERROR",
            serde_json::json!({"background.cycle.name": "sync"}),
        ),
    ];
    spans[1].duration_ns = 9_000_000;
    let cycles = summarize_background_cycles(&spans, 10);
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].name, "sync");
    assert_eq!(cycles[0].count, 2);
    assert_eq!(cycles[0].error_count, 1);
    assert_eq!(cycles[0].last_trace_id, "trace-20");
    assert!(cycles[0].p95_ns.unwrap() >= cycles[0].p50_ns.unwrap());
}

#[test]
fn jobs_group_producer_and_consumer_attempts() {
    let spans = vec![
        span(
            10,
            "job.publish",
            "SPAN_KIND_PRODUCER",
            None,
            "STATUS_CODE_OK",
            serde_json::json!({"job.id": "j1", "job.type": "index.rebuild"}),
        ),
        span(
            20,
            "job.consume",
            "SPAN_KIND_CONSUMER",
            None,
            "STATUS_CODE_ERROR",
            serde_json::json!({"job.id": "j1", "outcome": "error"}),
        ),
        span(
            30,
            "job.consume",
            "SPAN_KIND_CONSUMER",
            None,
            "STATUS_CODE_OK",
            serde_json::json!({"job.id": "j1", "outcome": "success"}),
        ),
    ];
    let jobs = summarize_jobs(&spans, 10);
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].job_type.as_deref(), Some("index.rebuild"));
    assert_eq!(jobs[0].produced_nanos, Some(10));
    assert_eq!(jobs[0].attempts.len(), 2);
    assert!(jobs[0].attempts[0].has_error);
    assert_eq!(jobs[0].attempts[1].outcome.as_deref(), Some("success"));
}

#[test]
fn conversations_sum_tokens_where_present() {
    let spans = vec![
        span(
            10,
            "chat claude",
            "SPAN_KIND_CLIENT",
            None,
            "STATUS_CODE_OK",
            serde_json::json!({
                "gen_ai.conversation.id": "c1", "gen_ai.agent.name": "navigator",
                "gen_ai.provider.name": "anthropic",
                "gen_ai.usage.input_tokens": 100, "gen_ai.usage.output_tokens": 30
            }),
        ),
        span(
            20,
            "chat claude",
            "SPAN_KIND_CLIENT",
            None,
            "STATUS_CODE_OK",
            serde_json::json!({
                "gen_ai.conversation.id": "c1",
                "gen_ai.usage.input_tokens": 50, "gen_ai.usage.output_tokens": 20
            }),
        ),
    ];
    let conversations = summarize_conversations(&spans, 10);
    assert_eq!(conversations.len(), 1);
    assert_eq!(conversations[0].agent_name.as_deref(), Some("navigator"));
    assert_eq!(conversations[0].span_count, 2);
    assert_eq!(conversations[0].input_tokens, Some(150.0));
    assert_eq!(conversations[0].output_tokens, Some(50.0));
}
