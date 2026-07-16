//! GraphQL projections of the generic CLI-application signal families:
//! sessions, screen visits, actions, cycles, jobs, and conversations.

use juniper::{FieldResult, graphql_object};
use parallax_storage::adapter;

use crate::{ApiContext, clamp_limit, nanos_string, retained_recent_range, saturate_i32};

pub(crate) struct SessionOut(adapter::InvocationSession);

#[graphql_object(context = ApiContext, name = "Session")]
impl SessionOut {
    fn session_id(&self) -> &str {
        &self.0.session_id
    }
    fn previous_session_id(&self) -> Option<&str> {
        self.0.previous_session_id.as_deref()
    }
    fn start_nanos(&self) -> String {
        nanos_string(self.0.start_nanos)
    }
    /// Null while the session is still open.
    fn end_nanos(&self) -> Option<String> {
        self.0.end_nanos.map(nanos_string)
    }
}

pub(crate) struct ScreenVisitOut(adapter::ScreenVisit);

#[graphql_object(context = ApiContext, name = "ScreenVisit")]
impl ScreenVisitOut {
    fn screen_id(&self) -> &str {
        &self.0.screen_id
    }
    fn visit_id(&self) -> &str {
        &self.0.visit_id
    }
    fn session_id(&self) -> Option<&str> {
        self.0.session_id.as_deref()
    }
    fn navigation_sequence(&self) -> Option<i32> {
        self.0
            .navigation_sequence
            .map(|sequence| i32::try_from(sequence).unwrap_or(i32::MAX))
    }
    fn transition_reason(&self) -> Option<&str> {
        self.0.transition_reason.as_deref()
    }
    fn entered_nanos(&self) -> String {
        nanos_string(self.0.entered_nanos)
    }
    /// Null while the screen is still active.
    fn exited_nanos(&self) -> Option<String> {
        self.0.exited_nanos.map(nanos_string)
    }
}

pub(crate) struct UiActionOut(adapter::UiAction);

#[graphql_object(context = ApiContext, name = "UiAction")]
impl UiActionOut {
    fn name(&self) -> &str {
        &self.0.name
    }
    fn screen_id(&self) -> Option<&str> {
        self.0.screen_id.as_deref()
    }
    fn session_id(&self) -> Option<&str> {
        self.0.session_id.as_deref()
    }
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
    fn start_nanos(&self) -> String {
        nanos_string(self.0.start_nanos)
    }
    fn duration_ms(&self) -> f64 {
        #[expect(clippy::cast_precision_loss, reason = "display duration")]
        {
            self.0.duration_ns as f64 / 1_000_000.0
        }
    }
    fn outcome(&self) -> Option<&str> {
        self.0.outcome.as_deref()
    }
    fn has_error(&self) -> bool {
        self.0.has_error
    }
}

pub(crate) struct BackgroundCycleOut(adapter::BackgroundCycleSummary);

#[graphql_object(context = ApiContext, name = "BackgroundCycle")]
impl BackgroundCycleOut {
    fn name(&self) -> &str {
        &self.0.name
    }
    fn count(&self) -> i32 {
        saturate_i32(self.0.count)
    }
    fn error_count(&self) -> i32 {
        saturate_i32(self.0.error_count)
    }
    fn p50_ms(&self) -> Option<f64> {
        self.0.p50_ns.map(|ns| ns / 1_000_000.0)
    }
    fn p95_ms(&self) -> Option<f64> {
        self.0.p95_ns.map(|ns| ns / 1_000_000.0)
    }
    fn last_nanos(&self) -> String {
        nanos_string(self.0.last_nanos)
    }
    fn last_trace_id(&self) -> &str {
        &self.0.last_trace_id
    }
}

pub(crate) struct JobAttemptOut(adapter::JobAttempt);

#[graphql_object(context = ApiContext, name = "JobAttempt")]
impl JobAttemptOut {
    fn start_nanos(&self) -> String {
        nanos_string(self.0.start_nanos)
    }
    fn duration_ms(&self) -> f64 {
        #[expect(clippy::cast_precision_loss, reason = "display duration")]
        {
            self.0.duration_ns as f64 / 1_000_000.0
        }
    }
    fn outcome(&self) -> Option<&str> {
        self.0.outcome.as_deref()
    }
    fn has_error(&self) -> bool {
        self.0.has_error
    }
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
}

pub(crate) struct JobOut(adapter::JobSummary);

#[graphql_object(context = ApiContext, name = "Job")]
impl JobOut {
    fn job_id(&self) -> &str {
        &self.0.job_id
    }
    fn job_type(&self) -> Option<&str> {
        self.0.job_type.as_deref()
    }
    fn produced_nanos(&self) -> Option<String> {
        self.0.produced_nanos.map(nanos_string)
    }
    fn attempts(&self) -> Vec<JobAttemptOut> {
        self.0.attempts.iter().cloned().map(JobAttemptOut).collect()
    }
    fn last_trace_id(&self) -> &str {
        &self.0.last_trace_id
    }
}

pub(crate) struct ConversationOut(adapter::ConversationSummary);

#[graphql_object(context = ApiContext, name = "Conversation")]
impl ConversationOut {
    fn conversation_id(&self) -> &str {
        &self.0.conversation_id
    }
    fn agent_name(&self) -> Option<&str> {
        self.0.agent_name.as_deref()
    }
    fn provider_name(&self) -> Option<&str> {
        self.0.provider_name.as_deref()
    }
    fn first_nanos(&self) -> String {
        nanos_string(self.0.first_nanos)
    }
    fn last_nanos(&self) -> String {
        nanos_string(self.0.last_nanos)
    }
    fn span_count(&self) -> i32 {
        saturate_i32(self.0.span_count)
    }
    fn input_tokens(&self) -> Option<f64> {
        self.0.input_tokens
    }
    fn output_tokens(&self) -> Option<f64> {
        self.0.output_tokens
    }
}

const PROJECTION_LIMIT: usize = 200;

fn parse_range(from_nanos: &str, to_nanos: &str) -> FieldResult<std::ops::RangeInclusive<u128>> {
    let from: u128 = from_nanos
        .parse()
        .map_err(|_| crate::field_err("invalid fromNanos"))?;
    let to: u128 = to_nanos
        .parse()
        .map_err(|_| crate::field_err("invalid toNanos"))?;
    Ok(from..=to)
}

pub(crate) async fn sessions(
    context: &ApiContext,
    invocation_id: String,
) -> FieldResult<Vec<SessionOut>> {
    Ok(context
        .store
        .sessions_by_invocation(&invocation_id, retained_recent_range(), PROJECTION_LIMIT)
        .await
        .map_err(crate::internal_field_err)?
        .into_iter()
        .map(SessionOut)
        .collect())
}

pub(crate) async fn screen_visits(
    context: &ApiContext,
    invocation_id: Option<String>,
    session_id: Option<String>,
) -> FieldResult<Vec<ScreenVisitOut>> {
    if invocation_id.is_none() && session_id.is_none() {
        return Err(crate::field_err(
            "screenVisits requires invocationId or sessionId",
        ));
    }
    Ok(context
        .store
        .screen_visits(
            invocation_id.as_deref(),
            session_id.as_deref(),
            retained_recent_range(),
            PROJECTION_LIMIT,
        )
        .await
        .map_err(crate::internal_field_err)?
        .into_iter()
        .map(ScreenVisitOut)
        .collect())
}

pub(crate) async fn ui_actions(
    context: &ApiContext,
    invocation_id: String,
    limit: Option<i32>,
) -> FieldResult<Vec<UiActionOut>> {
    Ok(context
        .store
        .ui_actions(
            &invocation_id,
            retained_recent_range(),
            clamp_limit(limit, PROJECTION_LIMIT),
        )
        .await
        .map_err(crate::internal_field_err)?
        .into_iter()
        .map(UiActionOut)
        .collect())
}

pub(crate) async fn background_cycles(
    context: &ApiContext,
    invocation_id: Option<String>,
    from_nanos: String,
    to_nanos: String,
) -> FieldResult<Vec<BackgroundCycleOut>> {
    Ok(context
        .store
        .background_cycles(
            invocation_id.as_deref(),
            parse_range(&from_nanos, &to_nanos)?,
            PROJECTION_LIMIT,
        )
        .await
        .map_err(crate::internal_field_err)?
        .into_iter()
        .map(BackgroundCycleOut)
        .collect())
}

pub(crate) async fn jobs(
    context: &ApiContext,
    invocation_id: Option<String>,
    from_nanos: String,
    to_nanos: String,
) -> FieldResult<Vec<JobOut>> {
    Ok(context
        .store
        .jobs(
            invocation_id.as_deref(),
            parse_range(&from_nanos, &to_nanos)?,
            PROJECTION_LIMIT,
        )
        .await
        .map_err(crate::internal_field_err)?
        .into_iter()
        .map(JobOut)
        .collect())
}

pub(crate) async fn conversations(
    context: &ApiContext,
    invocation_id: String,
) -> FieldResult<Vec<ConversationOut>> {
    Ok(context
        .store
        .conversations(&invocation_id, retained_recent_range(), PROJECTION_LIMIT)
        .await
        .map_err(crate::internal_field_err)?
        .into_iter()
        .map(ConversationOut)
        .collect())
}

#[cfg(test)]
mod tests;
