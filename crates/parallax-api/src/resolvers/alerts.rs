//! GraphQL alerting domain types and resolvers (plan 167 Step 4).
//!
//! Alert storage is Turso-only (the evaluator and delivery worker also bind
//! to the concrete store), so these resolvers go through the optional
//! `ApiContext::alerts` handle rather than the query-neutral `MetadataStore`
//! trait. When the handle is absent (pure in-memory test harnesses) every
//! resolver reports alerting as unavailable instead of panicking.

use juniper::{FieldError, FieldResult, graphql_object};
use parallax_metadata::{
    AlertCheckRecord, AlertDestinationRecord, AlertIncidentRecord, AlertRuleRecord,
    AlertRuleStateRecord, TursoMetadataStore,
};
use parallax_storage::metadata::MetadataError;
use std::sync::Arc;

use crate::{ApiContext, clamp_limit, field_err, nanos_string, saturate_i32};

mod consts;
pub(crate) use consts::*;

fn alerts(context: &ApiContext) -> FieldResult<&Arc<TursoMetadataStore>> {
    context
        .alerts
        .as_ref()
        .ok_or_else(|| field_err("alerting storage is not available on this server"))
}

fn store_err(error: anyhow::Error) -> FieldError {
    crate::internal_field_err(MetadataError::internal(error))
}

pub(crate) fn now_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos())
}

fn opt_nanos_string(nanos: Option<u128>) -> Option<String> {
    nanos.map(nanos_string)
}

pub(crate) struct AlertRule(pub(crate) AlertRuleRecord);

#[graphql_object(context = ApiContext)]
impl AlertRule {
    fn id(&self) -> &str {
        &self.0.id
    }
    fn name(&self) -> &str {
        &self.0.name
    }
    fn enabled(&self) -> bool {
        self.0.enabled
    }
    /// `error_rate|p95_latency|p99_latency|throughput|log_count|metric`.
    fn signal_type(&self) -> &str {
        &self.0.signal_type
    }
    /// JSON array of scoped service names; `[]` means all services.
    fn services(&self) -> &str {
        &self.0.services
    }
    /// JSON array of excluded service names.
    fn exclude_services(&self) -> &str {
        &self.0.exclude_services
    }
    /// JSON array of attribute filters (`{key, op, value}` — plan 164 shape).
    fn attribute_filters(&self) -> &str {
        &self.0.attribute_filters
    }
    /// Optional group-by dimension (`service`).
    fn group_by(&self) -> Option<&str> {
        self.0.group_by.as_deref()
    }
    /// `gt|gte|lt|lte|between|not_between`.
    fn comparator(&self) -> &str {
        &self.0.comparator
    }
    fn threshold(&self) -> f64 {
        self.0.threshold
    }
    fn threshold_upper(&self) -> Option<f64> {
        self.0.threshold_upper
    }
    fn window_minutes(&self) -> i32 {
        i32::try_from(self.0.window_minutes).unwrap_or(i32::MAX)
    }
    fn minimum_sample_count(&self) -> i32 {
        saturate_i32(self.0.minimum_sample_count)
    }
    fn consecutive_breaches_required(&self) -> i32 {
        i32::try_from(self.0.consecutive_breaches_required).unwrap_or(i32::MAX)
    }
    fn consecutive_healthy_required(&self) -> i32 {
        i32::try_from(self.0.consecutive_healthy_required).unwrap_or(i32::MAX)
    }
    /// `skip|zero`.
    fn no_data_behavior(&self) -> &str {
        &self.0.no_data_behavior
    }
    /// `warning|critical`.
    fn severity(&self) -> &str {
        &self.0.severity
    }
    fn renotify_interval_minutes(&self) -> i32 {
        i32::try_from(self.0.renotify_interval_minutes).unwrap_or(i32::MAX)
    }
    /// JSON array of destination ids.
    fn destination_ids(&self) -> &str {
        &self.0.destination_ids
    }
    fn metric_name(&self) -> Option<&str> {
        self.0.metric_name.as_deref()
    }
    fn metric_aggregation(&self) -> Option<&str> {
        self.0.metric_aggregation.as_deref()
    }
    fn created_at_nanos(&self) -> String {
        nanos_string(self.0.created_at_nanos)
    }
    fn updated_at_nanos(&self) -> String {
        nanos_string(self.0.updated_at_nanos)
    }
}

pub(crate) struct AlertRuleState(pub(crate) AlertRuleStateRecord);

#[graphql_object(context = ApiContext)]
impl AlertRuleState {
    fn rule_id(&self) -> &str {
        &self.0.rule_id
    }
    fn group_key(&self) -> &str {
        &self.0.group_key
    }
    fn consecutive_breaches(&self) -> i32 {
        i32::try_from(self.0.consecutive_breaches).unwrap_or(i32::MAX)
    }
    fn consecutive_healthy(&self) -> i32 {
        i32::try_from(self.0.consecutive_healthy).unwrap_or(i32::MAX)
    }
    fn incident_open(&self) -> bool {
        self.0.incident_open
    }
    fn last_notified_at_nanos(&self) -> Option<String> {
        opt_nanos_string(self.0.last_notified_at_nanos)
    }
    /// `breach|healthy|no_data|error`.
    fn last_status(&self) -> Option<&str> {
        self.0.last_status.as_deref()
    }
    fn last_value(&self) -> Option<f64> {
        self.0.last_value
    }
    fn last_sample_count(&self) -> i32 {
        saturate_i32(self.0.last_sample_count)
    }
    fn last_evaluated_at_nanos(&self) -> Option<String> {
        opt_nanos_string(self.0.last_evaluated_at_nanos)
    }
    fn last_error(&self) -> Option<&str> {
        self.0.last_error.as_deref()
    }
}

pub(crate) struct AlertIncident(pub(crate) AlertIncidentRecord);

#[graphql_object(context = ApiContext)]
impl AlertIncident {
    fn id(&self) -> &str {
        &self.0.id
    }
    fn rule_id(&self) -> &str {
        &self.0.rule_id
    }
    fn group_key(&self) -> &str {
        &self.0.group_key
    }
    /// `open|resolved`.
    fn status(&self) -> &str {
        &self.0.status
    }
    /// `warning|critical`.
    fn severity(&self) -> &str {
        &self.0.severity
    }
    fn first_triggered_at_nanos(&self) -> String {
        nanos_string(self.0.first_triggered_at_nanos)
    }
    fn last_triggered_at_nanos(&self) -> String {
        nanos_string(self.0.last_triggered_at_nanos)
    }
    fn resolved_at_nanos(&self) -> Option<String> {
        opt_nanos_string(self.0.resolved_at_nanos)
    }
    fn last_value(&self) -> Option<f64> {
        self.0.last_value
    }
    fn last_notified_at_nanos(&self) -> Option<String> {
        opt_nanos_string(self.0.last_notified_at_nanos)
    }
    /// The owning rule, if it still exists.
    async fn rule(&self, context: &ApiContext) -> FieldResult<Option<AlertRule>> {
        Ok(alerts(context)?
            .alert_rule(&self.0.rule_id)
            .await
            .map_err(store_err)?
            .map(AlertRule))
    }
    async fn bundle(
        &self,
        ctx: &ApiContext,
    ) -> FieldResult<Option<crate::resolvers::issues::BundleOut>> {
        crate::resolvers::issues::bundle(ctx, None, None, None, Some(self.0.id.clone()), None).await
    }
}

pub(crate) struct AlertDestination(pub(crate) AlertDestinationRecord);

#[graphql_object(context = ApiContext)]
impl AlertDestination {
    fn id(&self) -> &str {
        &self.0.id
    }
    fn name(&self) -> &str {
        &self.0.name
    }
    /// `webhook|slack_webhook`.
    fn kind(&self) -> &str {
        &self.0.kind
    }
    /// Destination config JSON (`{"url": ...}`); V1 stores it plaintext
    /// (single-operator local-first scope, recorded in plan 167).
    fn config(&self) -> &str {
        &self.0.config
    }
    fn created_at_nanos(&self) -> String {
        nanos_string(self.0.created_at_nanos)
    }
    fn updated_at_nanos(&self) -> String {
        nanos_string(self.0.updated_at_nanos)
    }
}

mod preview;
mod validate;
pub(crate) use preview::{AlertRulePreview, alert_rule_preview};

pub(crate) struct AlertCheck(pub(crate) AlertCheckRecord);

#[graphql_object(context = ApiContext)]
impl AlertCheck {
    fn rule_id(&self) -> &str {
        &self.0.rule_id
    }
    fn group_key(&self) -> &str {
        &self.0.group_key
    }
    fn checked_at_nanos(&self) -> String {
        nanos_string(self.0.checked_at_nanos)
    }
    fn value(&self) -> Option<f64> {
        self.0.value
    }
    fn sample_count(&self) -> i32 {
        saturate_i32(self.0.sample_count)
    }
    /// `breach|healthy|no_data|error`.
    fn status(&self) -> &str {
        &self.0.status
    }
    fn error(&self) -> Option<&str> {
        self.0.error.as_deref()
    }
}

/// Create/update payload for an alert rule. Optional knobs fall back to the
/// plan-167 defaults (2 consecutive breaches/healthy, `skip` on no data,
/// 30-minute renotify, minimum sample count 1).
#[derive(juniper::GraphQLInputObject, Clone, Debug)]
pub(crate) struct AlertRuleInput {
    pub(crate) id: Option<String>,
    pub(crate) name: String,
    pub(crate) enabled: Option<bool>,
    pub(crate) signal_type: String,
    pub(crate) services: Option<Vec<String>>,
    pub(crate) exclude_services: Option<Vec<String>>,
    /// JSON array of attribute filters (`{key, op, value}`), stored opaquely.
    pub(crate) attribute_filters: Option<String>,
    pub(crate) group_by: Option<String>,
    pub(crate) comparator: String,
    pub(crate) threshold: f64,
    pub(crate) threshold_upper: Option<f64>,
    pub(crate) window_minutes: i32,
    pub(crate) minimum_sample_count: Option<i32>,
    pub(crate) consecutive_breaches_required: Option<i32>,
    pub(crate) consecutive_healthy_required: Option<i32>,
    pub(crate) no_data_behavior: Option<String>,
    pub(crate) severity: String,
    pub(crate) renotify_interval_minutes: Option<i32>,
    pub(crate) destination_ids: Option<Vec<String>>,
    pub(crate) metric_name: Option<String>,
    pub(crate) metric_aggregation: Option<String>,
}

pub(crate) use validate::validated_rule;

pub(crate) async fn alert_rules(context: &ApiContext) -> FieldResult<Vec<AlertRule>> {
    let rules = alerts(context)?.alert_rules().await.map_err(store_err)?;
    Ok(rules.into_iter().map(AlertRule).collect())
}

pub(crate) async fn alert_rule(context: &ApiContext, id: String) -> FieldResult<Option<AlertRule>> {
    Ok(alerts(context)?
        .alert_rule(&id)
        .await
        .map_err(store_err)?
        .map(AlertRule))
}

pub(crate) async fn alert_rule_states(
    context: &ApiContext,
    rule_id: String,
) -> FieldResult<Vec<AlertRuleState>> {
    let states = alerts(context)?
        .alert_rule_states(&rule_id)
        .await
        .map_err(store_err)?;
    Ok(states.into_iter().map(AlertRuleState).collect())
}

pub(crate) async fn alert_incidents(
    context: &ApiContext,
    status: Option<String>,
    rule_id: Option<String>,
    limit: Option<i32>,
) -> FieldResult<Vec<AlertIncident>> {
    if let Some(status) = status.as_deref()
        && status != "open"
        && status != "resolved"
    {
        return Err(field_err(format!("unknown incident status: {status:?}")));
    }
    let incidents = alerts(context)?
        .alert_incidents(
            status.as_deref(),
            rule_id.as_deref(),
            clamp_limit(limit, ALERT_INCIDENTS_DEFAULT_LIMIT),
        )
        .await
        .map_err(store_err)?;
    Ok(incidents.into_iter().map(AlertIncident).collect())
}

pub(crate) async fn alert_incident(
    context: &ApiContext,
    id: String,
) -> FieldResult<Option<AlertIncident>> {
    Ok(alerts(context)?
        .alert_incident(&id)
        .await
        .map_err(store_err)?
        .map(AlertIncident))
}

pub(crate) async fn alert_destinations(context: &ApiContext) -> FieldResult<Vec<AlertDestination>> {
    let destinations = alerts(context)?
        .alert_destinations()
        .await
        .map_err(store_err)?;
    Ok(destinations.into_iter().map(AlertDestination).collect())
}

pub(crate) async fn alert_checks(
    context: &ApiContext,
    rule_id: String,
    limit: Option<i32>,
) -> FieldResult<Vec<AlertCheck>> {
    let checks = alerts(context)?
        .alert_checks(&rule_id, clamp_limit(limit, ALERT_CHECKS_DEFAULT_LIMIT))
        .await
        .map_err(store_err)?;
    Ok(checks.into_iter().map(AlertCheck).collect())
}

pub(crate) async fn alert_rule_save(
    context: &ApiContext,
    input: AlertRuleInput,
) -> FieldResult<AlertRule> {
    let store = alerts(context)?;
    let existing = match input.id.as_deref() {
        Some(id) => store.alert_rule(id).await.map_err(store_err)?,
        None => None,
    };
    let rule = validated_rule(input, existing.as_ref())?;
    store.alert_rule_save(&rule).await.map_err(store_err)?;
    store
        .alert_rule(&rule.id)
        .await
        .map_err(store_err)?
        .map(AlertRule)
        .ok_or_else(|| field_err("alert rule save did not persist"))
}

pub(crate) async fn alert_rule_delete(context: &ApiContext, id: String) -> FieldResult<bool> {
    alerts(context)?
        .alert_rule_delete(&id)
        .await
        .map_err(store_err)
}

pub(crate) async fn alert_rule_set_enabled(
    context: &ApiContext,
    id: String,
    enabled: bool,
) -> FieldResult<AlertRule> {
    let store = alerts(context)?;
    store
        .alert_rule_set_enabled(&id, enabled)
        .await
        .map_err(store_err)?;
    store
        .alert_rule(&id)
        .await
        .map_err(store_err)?
        .map(AlertRule)
        .ok_or_else(|| field_err(format!("alert rule not found: {id}")))
}

pub(crate) async fn alert_destination_save(
    context: &ApiContext,
    name: String,
    kind: String,
    config: String,
    id: Option<String>,
) -> FieldResult<AlertDestination> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(field_err("destination name must not be empty"));
    }
    if name.chars().count() > ALERT_NAME_MAX {
        return Err(field_err(format!(
            "destination name exceeds {ALERT_NAME_MAX} characters"
        )));
    }
    // `email` is deferred in V1 (no in-tree mail transport; plan 167 STOP
    // condition), so the API refuses it rather than storing a dead type.
    if !ALERT_DESTINATION_KINDS.contains(&kind.as_str()) {
        return Err(field_err(format!("unknown destination kind: {kind:?}")));
    }
    let url = serde_json::from_str::<serde_json::Value>(&config)
        .ok()
        .and_then(|value| {
            value
                .get("url")
                .and_then(|url| url.as_str().map(str::to_string))
        });
    match url {
        Some(url) if url.starts_with("http://") || url.starts_with("https://") => {}
        _ => {
            return Err(field_err(
                "destination config must be JSON with an http(s) \"url\" field",
            ));
        }
    }
    let store = alerts(context)?;
    let now = now_nanos();
    let existing = match id.as_deref() {
        Some(id) => store.alert_destination(id).await.map_err(store_err)?,
        None => None,
    };
    let destination = AlertDestinationRecord {
        id: id.unwrap_or_else(|| format!("dst_{now:x}")),
        name,
        kind,
        config,
        created_at_nanos: existing.as_ref().map_or(now, |d| d.created_at_nanos),
        updated_at_nanos: now,
    };
    store
        .alert_destination_save(&destination)
        .await
        .map_err(store_err)?;
    store
        .alert_destination(&destination.id)
        .await
        .map_err(store_err)?
        .map(AlertDestination)
        .ok_or_else(|| field_err("alert destination save did not persist"))
}

pub(crate) async fn alert_destination_delete(
    context: &ApiContext,
    id: String,
) -> FieldResult<bool> {
    alerts(context)?
        .alert_destination_delete(&id)
        .await
        .map_err(store_err)
}

#[cfg(test)]
#[path = "alerts/preview_tests.rs"]
mod preview_tests;
#[cfg(test)]
mod tests;
