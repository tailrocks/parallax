//! Browser RUM sessions: first-class session entities derived from spans
//! grouped by `session.id` — not the CLI `sessions(invocationId:)` pairing,
//! which models coding-agent/CLI runs, not page visits.

use juniper::{FieldResult, graphql_object};
use parallax_storage::adapter;

use crate::{ApiContext, clamp_limit, nanos_string, saturate_i32};

pub(crate) struct RumSessionOut(adapter::RumSession);

#[graphql_object(context = ApiContext, name = "RumSession")]
impl RumSessionOut {
    fn session_id(&self) -> &str {
        &self.0.session_id
    }
    fn service(&self) -> &str {
        &self.0.service
    }
    fn start_nanos(&self) -> String {
        nanos_string(self.0.start_nanos)
    }
    /// Last activity in the session (browsers emit no explicit session end).
    fn end_nanos(&self) -> String {
        nanos_string(self.0.end_nanos)
    }
    fn span_count(&self) -> i32 {
        saturate_i32(self.0.span_count)
    }
    fn trace_count(&self) -> i32 {
        saturate_i32(self.0.trace_count)
    }
    fn view_count(&self) -> i32 {
        saturate_i32(self.0.view_count)
    }
    fn vital_count(&self) -> i32 {
        saturate_i32(self.0.vital_count)
    }
    fn error_count(&self) -> i32 {
        saturate_i32(self.0.error_count)
    }
    fn has_error(&self) -> bool {
        self.0.has_error
    }
}

pub(crate) struct RumSessionPageViewOut(adapter::RumSessionPageView);

#[graphql_object(context = ApiContext, name = "RumSessionPageView")]
impl RumSessionPageViewOut {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn screen(&self) -> &str {
        &self.0.screen
    }
    fn path(&self) -> Option<&str> {
        self.0.path.as_deref()
    }
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
    fn span_id(&self) -> &str {
        &self.0.span_id
    }
}

pub(crate) struct RumSessionVitalOut(adapter::RumSessionVital);

#[graphql_object(context = ApiContext, name = "RumSessionVital")]
impl RumSessionVitalOut {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn name(&self) -> &str {
        &self.0.name
    }
    fn value(&self) -> f64 {
        self.0.value
    }
    fn rating(&self) -> Option<&str> {
        self.0.rating.as_deref()
    }
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
    fn span_id(&self) -> &str {
        &self.0.span_id
    }
}

pub(crate) struct RumSessionErrorOut(adapter::RumSessionError);

#[graphql_object(context = ApiContext, name = "RumSessionError")]
impl RumSessionErrorOut {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn name(&self) -> &str {
        &self.0.name
    }
    fn error_type(&self) -> Option<&str> {
        self.0.error_type.as_deref()
    }
    fn message(&self) -> &str {
        &self.0.message
    }
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
    fn span_id(&self) -> &str {
        &self.0.span_id
    }
}

pub(crate) struct RumSessionDetailOut(adapter::RumSessionDetail);

#[graphql_object(context = ApiContext, name = "RumSessionDetail")]
impl RumSessionDetailOut {
    fn session(&self) -> RumSessionOut {
        RumSessionOut(self.0.session.clone())
    }
    fn views(&self) -> Vec<RumSessionPageViewOut> {
        self.0
            .views
            .iter()
            .cloned()
            .map(RumSessionPageViewOut)
            .collect()
    }
    fn vitals(&self) -> Vec<RumSessionVitalOut> {
        self.0
            .vitals
            .iter()
            .cloned()
            .map(RumSessionVitalOut)
            .collect()
    }
    fn errors(&self) -> Vec<RumSessionErrorOut> {
        self.0
            .errors
            .iter()
            .cloned()
            .map(RumSessionErrorOut)
            .collect()
    }
}

const PROJECTION_LIMIT: usize = 200;

pub(crate) async fn rum_sessions(
    context: &ApiContext,
    service: Option<String>,
    from_nanos: String,
    to_nanos: String,
    error_only: Option<bool>,
    limit: Option<i32>,
) -> FieldResult<Vec<RumSessionOut>> {
    let from: u128 = from_nanos
        .parse()
        .map_err(|_| crate::field_err("invalid fromNanos"))?;
    let to: u128 = to_nanos
        .parse()
        .map_err(|_| crate::field_err("invalid toNanos"))?;
    Ok(context
        .store
        .rum_sessions(
            service.as_deref(),
            from..=to,
            error_only.unwrap_or(false),
            clamp_limit(limit, PROJECTION_LIMIT),
        )
        .await
        .map_err(crate::internal_field_err)?
        .into_iter()
        .map(RumSessionOut)
        .collect())
}

pub(crate) async fn rum_session(
    context: &ApiContext,
    session_id: String,
    limit: Option<i32>,
) -> FieldResult<Option<RumSessionDetailOut>> {
    Ok(context
        .store
        .rum_session_detail(&session_id, clamp_limit(limit, PROJECTION_LIMIT))
        .await
        .map_err(crate::internal_field_err)?
        .map(RumSessionDetailOut))
}

#[cfg(test)]
mod tests;
