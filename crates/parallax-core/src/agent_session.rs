//! Run-scoped projection for agent execution spans.

use parallax_storage::model::SpanRow;
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};

// Producer contract pinned from
// parallax-telemetry-playground/cli/src/main.rs.
pub const INVOKE_AGENT_SPAN: &str = "invoke_agent";
pub const EXECUTE_TOOL_SPAN: &str = "execute_tool";
pub const GEN_AI_OPERATION_ATTR: &str = "gen_ai.operation.name";
pub const GEN_AI_TOOL_NAME_ATTR: &str = "gen_ai.tool.name";
pub const TOOL_NAME_ATTR: &str = "tool.name";
pub const SHELL_COMMAND_ATTR: &str = "shell.command";
pub const PROCESS_COMMAND_ATTR: &str = "process.command";
pub const PROCESS_COMMAND_LINE_ATTR: &str = "process.command_line";
pub const PROCESS_EXECUTABLE_NAME_ATTR: &str = "process.executable.name";
pub const INPUT_TOKENS_ATTR: &str = "gen_ai.usage.input_tokens";
pub const OUTPUT_TOKENS_ATTR: &str = "gen_ai.usage.output_tokens";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStepKind {
    InvokeAgent,
    ExecuteTool,
    Shell,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentStep {
    pub span_id: String,
    pub trace_id: String,
    pub kind: AgentStepKind,
    pub name: String,
    pub start_nanos: u128,
    pub duration_ns: u128,
    pub is_error: bool,
    pub gen_ai_operation: Option<String>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSession {
    pub root_span_id: Option<String>,
    pub steps: Vec<AgentStep>,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub error_count: usize,
}

#[must_use]
pub fn project_agent_session(spans: &[SpanRow]) -> Option<AgentSession> {
    let root_index = spans
        .iter()
        .enumerate()
        .filter(|(_, span)| is_invoke_agent(span))
        .min_by(|(_, left), (_, right)| {
            left.ts_nanos
                .cmp(&right.ts_nanos)
                .then_with(|| left.span_id.cmp(&right.span_id))
        })
        .map(|(index, _)| index)?;

    let root_span_id = spans[root_index].span_id.clone();
    let included = subtree_indexes(spans, root_index);
    let mut steps: Vec<AgentStep> = included
        .into_iter()
        .map(|index| step_from_span(&spans[index]))
        .collect();
    steps.sort_by(|left, right| {
        left.start_nanos
            .cmp(&right.start_nanos)
            .then_with(|| left.span_id.cmp(&right.span_id))
    });

    let total_input_tokens = steps.iter().filter_map(|step| step.input_tokens).sum();
    let total_output_tokens = steps.iter().filter_map(|step| step.output_tokens).sum();
    let error_count = steps.iter().filter(|step| step.is_error).count();

    Some(AgentSession {
        root_span_id: Some(root_span_id),
        steps,
        total_input_tokens,
        total_output_tokens,
        error_count,
    })
}

fn subtree_indexes(spans: &[SpanRow], root_index: usize) -> Vec<usize> {
    let mut children: HashMap<&str, Vec<usize>> = HashMap::new();
    for (index, span) in spans.iter().enumerate() {
        let Some(parent_id) = span.parent_span_id.as_deref().filter(|id| !id.is_empty()) else {
            continue;
        };
        children.entry(parent_id).or_default().push(index);
    }

    let mut seen = HashSet::new();
    let mut result = Vec::new();
    let mut queue = VecDeque::from([root_index]);
    while let Some(index) = queue.pop_front() {
        if !seen.insert(index) {
            continue;
        }
        result.push(index);
        if let Some(next) = children.get(spans[index].span_id.as_str()) {
            queue.extend(next.iter().copied());
        }
    }
    result
}

fn step_from_span(span: &SpanRow) -> AgentStep {
    let gen_ai_operation = attr_string(&span.attributes, GEN_AI_OPERATION_ATTR);
    let input_tokens = attr_i64(&span.attributes, INPUT_TOKENS_ATTR);
    let output_tokens = attr_i64(&span.attributes, OUTPUT_TOKENS_ATTR);
    let kind = classify_span(span, gen_ai_operation.as_deref());
    let name = display_name(span, kind, gen_ai_operation.as_deref());

    AgentStep {
        span_id: span.span_id.clone(),
        trace_id: span.trace_id.clone(),
        kind,
        name,
        start_nanos: span.ts_nanos,
        duration_ns: span.duration_ns,
        is_error: span.status_code == "STATUS_CODE_ERROR",
        gen_ai_operation,
        input_tokens,
        output_tokens,
    }
}

fn is_invoke_agent(span: &SpanRow) -> bool {
    span.name == INVOKE_AGENT_SPAN
        || attr_string(&span.attributes, GEN_AI_OPERATION_ATTR).as_deref()
            == Some(INVOKE_AGENT_SPAN)
}

fn classify_span(span: &SpanRow, gen_ai_operation: Option<&str>) -> AgentStepKind {
    if span.name == INVOKE_AGENT_SPAN || gen_ai_operation == Some(INVOKE_AGENT_SPAN) {
        return AgentStepKind::InvokeAgent;
    }
    if is_shell_span(span) {
        return AgentStepKind::Shell;
    }
    if span.name == EXECUTE_TOOL_SPAN || gen_ai_operation == Some(EXECUTE_TOOL_SPAN) {
        return AgentStepKind::ExecuteTool;
    }
    AgentStepKind::Other
}

fn is_shell_span(span: &SpanRow) -> bool {
    attr_string(&span.attributes, TOOL_NAME_ATTR).as_deref() == Some("shell_command")
        || has_attr(&span.attributes, PROCESS_COMMAND_ATTR)
        || has_attr(&span.attributes, PROCESS_COMMAND_LINE_ATTR)
        || has_attr(&span.attributes, PROCESS_EXECUTABLE_NAME_ATTR)
}

fn display_name(span: &SpanRow, kind: AgentStepKind, gen_ai_operation: Option<&str>) -> String {
    match kind {
        AgentStepKind::InvokeAgent => gen_ai_operation.unwrap_or(&span.name).to_string(),
        AgentStepKind::ExecuteTool => attr_string(&span.attributes, GEN_AI_TOOL_NAME_ATTR)
            .or_else(|| attr_string(&span.attributes, TOOL_NAME_ATTR))
            .unwrap_or_else(|| span.name.clone()),
        AgentStepKind::Shell => attr_string(&span.attributes, SHELL_COMMAND_ATTR)
            .or_else(|| attr_string(&span.attributes, PROCESS_COMMAND_LINE_ATTR))
            .or_else(|| attr_string(&span.attributes, PROCESS_COMMAND_ATTR))
            .or_else(|| attr_string(&span.attributes, PROCESS_EXECUTABLE_NAME_ATTR))
            .unwrap_or_else(|| span.name.clone()),
        AgentStepKind::Other => gen_ai_operation.unwrap_or(&span.name).to_string(),
    }
}

fn has_attr(attributes: &Value, key: &str) -> bool {
    attributes.get(key).is_some()
}

fn attr_string(attributes: &Value, key: &str) -> Option<String> {
    let value = attributes.get(key)?;
    if let Some(text) = value.as_str() {
        return Some(text.to_string());
    }
    if value.is_null() {
        return None;
    }
    Some(value.to_string())
}

fn attr_i64(attributes: &Value, key: &str) -> Option<i64> {
    let value = attributes.get(key)?;
    value.as_i64().or_else(|| {
        value
            .as_u64()
            .and_then(|number| i64::try_from(number).ok())
            .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
    })
}

#[cfg(test)]
mod tests;
