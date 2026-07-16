use crate::resolvers::test_support::*;
use crate::{build_schema, execute};

use parallax_test_support::builders::MemoryStore;

use parallax_storage::model::LogRow;
use std::sync::Arc;

#[tokio::test]
async fn agent_session_projects_run_scoped_agent_spans() {
    let store = Arc::new(MemoryStore::new());
    let mut root = span("agent", "trace-agent", "root", 1_000, 100);
    root.name = "invoke_agent".into();
    root.invocation_id = Some("run-agent".into());
    root.attributes = serde_json::json!({
        "gen_ai.operation.name": "invoke_agent"
    });
    let mut tool = span("agent", "trace-agent", "tool", 1_100, 25);
    tool.name = "execute_tool".into();
    tool.parent_span_id = Some("root".into());
    tool.invocation_id = Some("run-agent".into());
    tool.attributes = serde_json::json!({
        "gen_ai.operation.name": "execute_tool",
        "tool.name": "inspect_repo",
        "gen_ai.usage.input_tokens": "7"
    });
    let mut shell = span("agent", "trace-agent", "shell", 1_200, 25);
    shell.name = "execute_tool".into();
    shell.parent_span_id = Some("root".into());
    shell.invocation_id = Some("run-agent".into());
    shell.status_code = "STATUS_CODE_ERROR".into();
    shell.attributes = serde_json::json!({
        "gen_ai.operation.name": "execute_tool",
        "tool.name": "shell_command",
        "shell.command": "false",
        "gen_ai.usage.output_tokens": 3
    });
    let mut unrelated = span("agent", "trace-other", "other", 1_050, 10);
    unrelated.name = "execute_tool".into();
    unrelated.invocation_id = Some("run-other".into());
    store.push_spans(vec![shell, unrelated, root, tool]);

    let schema = build_schema();
    let context = context_with_memory(store).await;
    let request = juniper::http::GraphQLRequest::new(
        r#"
        {
          agentSession(invocationId: "run-agent") {
            rootSpanId
            truncated
            totalInputTokens
            totalOutputTokens
            errorCount
            steps {
              kind name spanId traceId startNanos durationNs isError
              genAiOperation inputTokens outputTokens
            }
          }
          unrelated: agentSession(invocationId: "run-other") { rootSpanId }
        }
        "#
        .into(),
        None,
        None,
    );
    let response = execute(&schema, &context, request).await;
    let json = serde_json::to_value(response).unwrap();

    assert!(
        error_messages(&json).is_empty(),
        "agentSession query succeeds: {json}"
    );
    assert_eq!(
        json.pointer("/data/agentSession/rootSpanId"),
        Some(&serde_json::json!("root"))
    );
    assert_eq!(
        json.pointer("/data/agentSession/truncated"),
        Some(&serde_json::json!(false))
    );
    assert_eq!(
        json.pointer("/data/agentSession/totalInputTokens"),
        Some(&serde_json::json!("7"))
    );
    assert_eq!(
        json.pointer("/data/agentSession/totalOutputTokens"),
        Some(&serde_json::json!("3"))
    );
    assert_eq!(
        json.pointer("/data/agentSession/errorCount"),
        Some(&serde_json::json!(1))
    );
    assert_eq!(
        json.pointer("/data/agentSession/steps/0/kind"),
        Some(&serde_json::json!("INVOKE_AGENT"))
    );
    assert_eq!(
        json.pointer("/data/agentSession/steps/1/name"),
        Some(&serde_json::json!("inspect_repo"))
    );
    assert_eq!(
        json.pointer("/data/agentSession/steps/2/kind"),
        Some(&serde_json::json!("SHELL"))
    );
    assert_eq!(
        json.pointer("/data/agentSession/steps/2/isError"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        json.pointer("/data/unrelated"),
        Some(&serde_json::Value::Null)
    );
}

#[tokio::test]
async fn story_resolver_returns_trace_and_run_beats() {
    let store = Arc::new(MemoryStore::new());
    let mut root = span("api", "cccccccccccccccccccccccccccccccc", "root", 100, 50);
    root.invocation_id = Some("run-story".into());
    root.name = "checkout".into();
    root.events = Some(r#"[{"name":"exception","timeUnixNano":"120"}]"#.into());
    let mut child = span("db", "cccccccccccccccccccccccccccccccc", "child", 110, 10);
    child.invocation_id = Some("run-story".into());
    child.parent_span_id = Some("root".into());
    child.name = "SELECT orders".into();
    child.status_code = "STATUS_CODE_ERROR".into();
    store.push_spans(vec![root, child]);
    store.push_logs(vec![LogRow {
        ts_nanos: 130,
        event_name: String::new(),
        observed_ts_nanos: 0,
        service: "api".into(),
        severity_num: 17,
        severity_text: "ERROR".into(),
        body: "payment 123 failed".into(),
        trace_id: "cccccccccccccccccccccccccccccccc".into(),
        span_id: "child".into(),
        invocation_id: Some("run-story".into()),
        session_id: None,
        scope_name: String::new(),
        attributes: serde_json::Value::Null,
        resource: serde_json::Value::Null,
    }]);

    let schema = build_schema();
    let context = context_with_memory(store).await;
    let request = juniper::http::GraphQLRequest::new(
        r#"
        {
          traceStory: story(traceId: "cccccccccccccccccccccccccccccccc") {
            tsNanos lane kind title traceId spanId severity durationNs
          }
          runStory: story(invocationId: "run-story") {
            kind traceId spanId
          }
        }
        "#
        .into(),
        None,
        None,
    );
    let response = execute(&schema, &context, request).await;
    let json = serde_json::to_value(response).unwrap();

    assert!(error_messages(&json).is_empty(), "story query: {json}");
    assert_eq!(
        json.pointer("/data/traceStory/0/kind"),
        Some(&serde_json::json!("span.start"))
    );
    assert!(
        json.pointer("/data/traceStory")
            .and_then(|value| value.as_array())
            .is_some_and(|beats| beats.iter().any(|beat| {
                beat["kind"] == "error" && beat["title"] == "ERROR payment <n> failed"
            })),
        "trace story has normalized error log beat: {json}"
    );
    assert!(
        json.pointer("/data/runStory")
            .and_then(|value| value.as_array())
            .is_some_and(|beats| beats.iter().any(|beat| {
                beat["traceId"] == "cccccccccccccccccccccccccccccccc" && beat["spanId"] == "child"
            })),
        "run story contains trace spans: {json}"
    );
}

#[tokio::test]
async fn story_requires_exactly_one_anchor() {
    let schema = build_schema();
    let context = context_with_memory(Arc::new(MemoryStore::new())).await;
    let request = juniper::http::GraphQLRequest::new(
        r#"{ story(traceId: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", invocationId: "b") { kind } }"#.into(),
        None,
        None,
    );
    let response = execute(&schema, &context, request).await;
    let json = serde_json::to_value(response).unwrap();

    assert!(
        error_messages(&json)
            .iter()
            .any(|message| message.contains("exactly one anchor")),
        "story anchor guard: {json}"
    );
}
