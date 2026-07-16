use super::*;
use serde_json::json;

fn span(span_id: &str, parent_span_id: Option<&str>, name: &str, ts_nanos: u128) -> SpanRow {
    SpanRow {
        ts_nanos,
        service: "agent".into(),
        trace_id: "trace-agent".into(),
        span_id: span_id.into(),
        parent_span_id: parent_span_id.map(str::to_string),
        name: name.into(),
        kind: "SPAN_KIND_INTERNAL".into(),
        status_code: "STATUS_CODE_UNSET".into(),
        status_message: String::new(),
        duration_ns: 10,
        invocation_id: Some("run-a".into()),
        session_id: None,
        scope_name: String::new(),
        events: None,
        links: Value::Null,
        attributes: Value::Null,
        resource: Value::Null,
    }
}

#[test]
fn projects_agent_subtree_in_time_order_with_errors() {
    let mut root = span("root", None, INVOKE_AGENT_SPAN, 100);
    root.duration_ns = 200;
    root.attributes = json!({ GEN_AI_OPERATION_ATTR: INVOKE_AGENT_SPAN });
    let mut tool_a = span("tool-a", Some("root"), EXECUTE_TOOL_SPAN, 120);
    tool_a.attributes = json!({
        GEN_AI_OPERATION_ATTR: EXECUTE_TOOL_SPAN,
        TOOL_NAME_ATTR: "inspect_repo",
        SHELL_COMMAND_ATTR: "rg --files",
    });
    let mut tool_b = span("tool-b", Some("root"), EXECUTE_TOOL_SPAN, 140);
    tool_b.attributes = json!({
        GEN_AI_OPERATION_ATTR: EXECUTE_TOOL_SPAN,
        GEN_AI_TOOL_NAME_ATTR: "read_file",
    });
    let mut shell = span("shell", Some("root"), EXECUTE_TOOL_SPAN, 160);
    shell.attributes = json!({
        GEN_AI_OPERATION_ATTR: EXECUTE_TOOL_SPAN,
        TOOL_NAME_ATTR: "shell_command",
        SHELL_COMMAND_ATTR: "false",
    });
    shell.status_code = "STATUS_CODE_ERROR".into();
    let mut tool_c = span("tool-c", Some("root"), EXECUTE_TOOL_SPAN, 180);
    tool_c.attributes = json!({ TOOL_NAME_ATTR: "summarize" });
    let unrelated = span("other", None, EXECUTE_TOOL_SPAN, 110);

    let result = project_agent_session(&[tool_c, unrelated, shell, tool_a, root, tool_b]).unwrap();

    assert_eq!(result.root_span_id.as_deref(), Some("root"));
    assert_eq!(result.error_count, 1);
    assert_eq!(
        result
            .steps
            .iter()
            .map(|step| (step.kind, step.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            (AgentStepKind::InvokeAgent, INVOKE_AGENT_SPAN),
            (AgentStepKind::ExecuteTool, "inspect_repo"),
            (AgentStepKind::ExecuteTool, "read_file"),
            (AgentStepKind::Shell, "false"),
            (AgentStepKind::ExecuteTool, "summarize"),
        ]
    );
}

#[test]
fn sums_tokens_only_when_present() {
    let root = span("root", None, INVOKE_AGENT_SPAN, 100);
    let mut child_a = span("child-a", Some("root"), EXECUTE_TOOL_SPAN, 110);
    child_a.attributes = json!({
        INPUT_TOKENS_ATTR: "12",
        OUTPUT_TOKENS_ATTR: 5,
    });
    let mut child_b = span("child-b", Some("root"), EXECUTE_TOOL_SPAN, 120);
    child_b.attributes = json!({ INPUT_TOKENS_ATTR: 8 });

    let result = project_agent_session(&[child_b, root, child_a]).unwrap();

    assert_eq!(result.total_input_tokens, 20);
    assert_eq!(result.total_output_tokens, 5);
    assert_eq!(result.steps[1].input_tokens, Some(12));
    assert_eq!(result.steps[2].output_tokens, None);
}

#[test]
fn returns_none_without_agent_root() {
    let spans = vec![span("tool", None, EXECUTE_TOOL_SPAN, 100)];

    assert!(project_agent_session(&spans).is_none());
}
