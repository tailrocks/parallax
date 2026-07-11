//! GraphQL story domain types and resolvers.

use juniper::{FieldResult, graphql_object};

use crate::{ApiContext, MAX_ROWS, field_err, nanos_string};

use parallax_core::{agent_session, story};

pub struct StoryBeat(pub(crate) story::StoryBeat);

#[graphql_object(context = ApiContext)]
impl StoryBeat {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn lane(&self) -> &str {
        &self.0.lane
    }
    fn kind(&self) -> &str {
        &self.0.kind
    }
    fn title(&self) -> &str {
        &self.0.title
    }
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
    fn span_id(&self) -> Option<&str> {
        self.0.span_id.as_deref()
    }
    fn severity(&self) -> Option<&str> {
        self.0.severity.as_deref()
    }
    fn duration_ns(&self) -> Option<String> {
        self.0.duration_ns.map(nanos_string)
    }
}

pub struct AgentSessionOut {
    session: agent_session::AgentSession,
    truncated: bool,
}

pub struct AgentStepOut(pub(crate) agent_session::AgentStep);

fn agent_step_kind_name(kind: agent_session::AgentStepKind) -> &'static str {
    match kind {
        agent_session::AgentStepKind::InvokeAgent => "INVOKE_AGENT",
        agent_session::AgentStepKind::ExecuteTool => "EXECUTE_TOOL",
        agent_session::AgentStepKind::Shell => "SHELL",
        agent_session::AgentStepKind::Other => "OTHER",
    }
}

#[graphql_object(context = ApiContext)]
impl AgentSessionOut {
    fn root_span_id(&self) -> Option<&str> {
        self.session.root_span_id.as_deref()
    }
    fn steps(&self) -> Vec<AgentStepOut> {
        self.session
            .steps
            .iter()
            .cloned()
            .map(AgentStepOut)
            .collect()
    }
    fn total_input_tokens(&self) -> String {
        self.session.total_input_tokens.to_string()
    }
    fn total_output_tokens(&self) -> String {
        self.session.total_output_tokens.to_string()
    }
    fn error_count(&self) -> i32 {
        i32::try_from(self.session.error_count).unwrap_or(i32::MAX)
    }
    fn truncated(&self) -> bool {
        self.truncated
    }
}

#[graphql_object(context = ApiContext)]
impl AgentStepOut {
    fn span_id(&self) -> &str {
        &self.0.span_id
    }
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
    fn kind(&self) -> &str {
        agent_step_kind_name(self.0.kind)
    }
    fn name(&self) -> &str {
        &self.0.name
    }
    fn start_nanos(&self) -> String {
        nanos_string(self.0.start_nanos)
    }
    fn duration_ns(&self) -> String {
        nanos_string(self.0.duration_ns)
    }
    fn is_error(&self) -> bool {
        self.0.is_error
    }
    fn gen_ai_operation(&self) -> Option<&str> {
        self.0.gen_ai_operation.as_deref()
    }
    fn input_tokens(&self) -> Option<String> {
        self.0.input_tokens.map(|tokens| tokens.to_string())
    }
    fn output_tokens(&self) -> Option<String> {
        self.0.output_tokens.map(|tokens| tokens.to_string())
    }
}

pub(crate) async fn agent_session(
    context: &ApiContext,
    run_id: String,
) -> FieldResult<Option<AgentSessionOut>> {
    let spans = context
        .store
        .spans_by_run(&run_id, MAX_ROWS)
        .await
        .map_err(field_err)?;
    let truncated = spans.len() == MAX_ROWS;
    Ok(agent_session::project_agent_session(&spans)
        .map(|session| AgentSessionOut { session, truncated }))
}

pub(crate) async fn story(
    context: &ApiContext,
    trace_id: Option<String>,
    run_id: Option<String>,
) -> FieldResult<Vec<StoryBeat>> {
    match (trace_id, run_id) {
        (Some(trace_id), None) => {
            let (spans, logs) =
                tokio::try_join!(context.spans_for(&trace_id), context.logs_for(&trace_id),)?;
            Ok(story::project_story(&spans, &logs, &[])
                .into_iter()
                .map(StoryBeat)
                .collect())
        }
        (None, Some(run_id)) => {
            let (spans, logs) = tokio::try_join!(
                context.store.spans_by_run(&run_id, MAX_ROWS),
                context.store.logs_by_run(&run_id, MAX_ROWS),
            )
            .map_err(field_err)?;
            Ok(story::project_story(&spans, &logs, &[])
                .into_iter()
                .map(StoryBeat)
                .collect())
        }
        _ => Err(field_err(
            "story takes exactly one anchor: traceId or runId",
        )),
    }
}

#[cfg(test)]
mod tests {

    use crate::resolvers::test_support::*;
    use crate::{build_schema, execute};
    use parallax_storage::adapter::TelemetryStore;
    use parallax_storage::memory::MemoryStore;

    use parallax_storage::model::LogRow;
    use std::sync::Arc;

    #[tokio::test]
    async fn agent_session_projects_run_scoped_agent_spans() {
        let store = Arc::new(MemoryStore::new());
        let mut root = span("agent", "trace-agent", "root", 1_000, 100);
        root.name = "invoke_agent".into();
        root.run_id = Some("run-agent".into());
        root.attributes = serde_json::json!({
            "gen_ai.operation.name": "invoke_agent"
        });
        let mut tool = span("agent", "trace-agent", "tool", 1_100, 25);
        tool.name = "execute_tool".into();
        tool.parent_span_id = Some("root".into());
        tool.run_id = Some("run-agent".into());
        tool.attributes = serde_json::json!({
            "gen_ai.operation.name": "execute_tool",
            "tool.name": "inspect_repo",
            "gen_ai.usage.input_tokens": "7"
        });
        let mut shell = span("agent", "trace-agent", "shell", 1_200, 25);
        shell.name = "execute_tool".into();
        shell.parent_span_id = Some("root".into());
        shell.run_id = Some("run-agent".into());
        shell.status_code = "STATUS_CODE_ERROR".into();
        shell.attributes = serde_json::json!({
            "gen_ai.operation.name": "execute_tool",
            "tool.name": "shell_command",
            "shell.command": "false",
            "gen_ai.usage.output_tokens": 3
        });
        let mut unrelated = span("agent", "trace-other", "other", 1_050, 10);
        unrelated.name = "execute_tool".into();
        unrelated.run_id = Some("run-other".into());
        store
            .ingest_traces(vec![shell, unrelated, root, tool], Default::default())
            .await
            .unwrap();

        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              agentSession(runId: "run-agent") {
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
              unrelated: agentSession(runId: "run-other") { rootSpanId }
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
        let mut root = span("api", "story-trace", "root", 100, 50);
        root.run_id = Some("run-story".into());
        root.name = "checkout".into();
        root.events = Some(r#"[{"name":"exception","timeUnixNano":"120"}]"#.into());
        let mut child = span("db", "story-trace", "child", 110, 10);
        child.run_id = Some("run-story".into());
        child.parent_span_id = Some("root".into());
        child.name = "SELECT orders".into();
        child.status_code = "STATUS_CODE_ERROR".into();
        store
            .ingest_traces(vec![root, child], Default::default())
            .await
            .unwrap();
        store
            .ingest_logs(
                vec![LogRow {
                    ts_nanos: 130,
                    event_name: String::new(),
                    observed_ts_nanos: 0,
                    service: "api".into(),
                    severity_num: 17,
                    severity_text: "ERROR".into(),
                    body: "payment 123 failed".into(),
                    trace_id: "story-trace".into(),
                    span_id: "child".into(),
                    run_id: Some("run-story".into()),
                    scope_name: String::new(),
                    attributes: serde_json::Value::Null,
                    resource: serde_json::Value::Null,
                }],
                Default::default(),
            )
            .await
            .unwrap();

        let schema = build_schema();
        let context = context_with_memory(store).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"
            {
              traceStory: story(traceId: "story-trace") {
                tsNanos lane kind title traceId spanId severity durationNs
              }
              runStory: story(runId: "run-story") {
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
                .is_some_and(|beats| beats
                    .iter()
                    .any(|beat| { beat["traceId"] == "story-trace" && beat["spanId"] == "child" })),
            "run story contains trace spans: {json}"
        );
    }

    #[tokio::test]
    async fn story_requires_exactly_one_anchor() {
        let schema = build_schema();
        let context = context_with_memory(Arc::new(MemoryStore::new())).await;
        let request = juniper::http::GraphQLRequest::new(
            r#"{ story(traceId: "a", runId: "b") { kind } }"#.into(),
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
}
