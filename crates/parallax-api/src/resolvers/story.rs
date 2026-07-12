//! GraphQL story domain types and resolvers.

use juniper::{FieldResult, graphql_object};

use crate::{ApiContext, MAX_ROWS, field_err, nanos_string, retained_recent_range};

use parallax_evidence::{agent_session, story};

pub(crate) struct StoryBeat(pub(crate) story::StoryBeat);

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

pub(crate) struct AgentSessionOut {
    session: agent_session::AgentSession,
    truncated: bool,
}

pub(crate) struct AgentStepOut(pub(crate) agent_session::AgentStep);

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
        .spans_by_run(&run_id, MAX_ROWS, retained_recent_range())
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
                context
                    .store
                    .spans_by_run(&run_id, MAX_ROWS, retained_recent_range()),
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
mod tests;
