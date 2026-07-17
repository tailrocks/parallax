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

pub(crate) const ALERT_SIGNAL_TYPES: [&str; 6] = [
    "error_rate",
    "p95_latency",
    "p99_latency",
    "throughput",
    "log_count",
    "metric",
];
pub(crate) const ALERT_COMPARATORS: [&str; 6] =
    ["gt", "gte", "lt", "lte", "between", "not_between"];
pub(crate) const ALERT_SEVERITIES: [&str; 2] = ["warning", "critical"];
pub(crate) const ALERT_NO_DATA_BEHAVIORS: [&str; 2] = ["skip", "zero"];
pub(crate) const ALERT_DESTINATION_KINDS: [&str; 2] = ["webhook", "slack_webhook"];
pub(crate) const ALERT_NAME_MAX: usize = 120;
pub(crate) const ALERT_INCIDENTS_DEFAULT_LIMIT: usize = 100;
pub(crate) const ALERT_CHECKS_DEFAULT_LIMIT: usize = 100;

fn alerts(context: &ApiContext) -> FieldResult<&Arc<TursoMetadataStore>> {
    context
        .alerts
        .as_ref()
        .ok_or_else(|| field_err("alerting storage is not available on this server"))
}

fn store_err(error: anyhow::Error) -> FieldError {
    crate::internal_field_err(MetadataError::internal(error))
}

fn now_nanos() -> u128 {
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

fn json_string_array(values: Option<Vec<String>>) -> String {
    serde_json::to_string(&values.unwrap_or_default()).unwrap_or_else(|_| "[]".to_string())
}

fn positive_u32(value: Option<i32>, default: u32, label: &str) -> FieldResult<u32> {
    match value {
        None => Ok(default),
        Some(v) if v >= 1 => Ok(u32::try_from(v).unwrap_or(default)),
        Some(v) => Err(field_err(format!("{label} must be >= 1, got {v}"))),
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one typed rule-field validation pass"
)]
fn validated_rule(
    input: AlertRuleInput,
    existing: Option<&AlertRuleRecord>,
) -> FieldResult<AlertRuleRecord> {
    let name = input.name.trim().to_string();
    if name.is_empty() {
        return Err(field_err("rule name must not be empty"));
    }
    if name.chars().count() > ALERT_NAME_MAX {
        return Err(field_err(format!(
            "rule name exceeds {ALERT_NAME_MAX} characters"
        )));
    }
    if !ALERT_SIGNAL_TYPES.contains(&input.signal_type.as_str()) {
        return Err(field_err(format!(
            "unknown signal type: {:?}",
            input.signal_type
        )));
    }
    if !ALERT_COMPARATORS.contains(&input.comparator.as_str()) {
        return Err(field_err(format!(
            "unknown comparator: {:?}",
            input.comparator
        )));
    }
    if !ALERT_SEVERITIES.contains(&input.severity.as_str()) {
        return Err(field_err(format!("unknown severity: {:?}", input.severity)));
    }
    let no_data_behavior = input.no_data_behavior.unwrap_or_else(|| "skip".to_string());
    if !ALERT_NO_DATA_BEHAVIORS.contains(&no_data_behavior.as_str()) {
        return Err(field_err(format!(
            "unknown noDataBehavior: {no_data_behavior:?}"
        )));
    }
    let range_comparator = matches!(input.comparator.as_str(), "between" | "not_between");
    match (range_comparator, input.threshold_upper) {
        (true, None) => {
            return Err(field_err("between/not_between require thresholdUpper"));
        }
        (true, Some(upper)) if upper <= input.threshold => {
            return Err(field_err("thresholdUpper must be greater than threshold"));
        }
        (false, Some(_)) => {
            return Err(field_err(
                "thresholdUpper is only valid with between/not_between",
            ));
        }
        _ => {}
    }
    if input.signal_type == "metric"
        && input
            .metric_name
            .as_deref()
            .is_none_or(|name| name.trim().is_empty())
    {
        return Err(field_err("signalType metric requires metricName"));
    }
    if input.signal_type == "error_rate" && !(0.0..=1.0).contains(&input.threshold) {
        return Err(field_err("error_rate thresholds are fractions in [0, 1]"));
    }
    if let Some(group_by) = input.group_by.as_deref()
        && group_by != "service"
    {
        return Err(field_err(format!(
            "unsupported groupBy dimension: {group_by:?}"
        )));
    }
    if input.window_minutes < 1 {
        return Err(field_err("windowMinutes must be >= 1"));
    }
    if input.attribute_filters.as_deref().is_some_and(|filters| {
        serde_json::from_str::<serde_json::Value>(filters)
            .map(|value| !value.is_array())
            .unwrap_or(true)
    }) {
        return Err(field_err("attributeFilters must be a JSON array"));
    }
    let now = now_nanos();
    Ok(AlertRuleRecord {
        id: input.id.unwrap_or_else(|| format!("alr_{now:x}")),
        name,
        enabled: input.enabled.unwrap_or(true),
        signal_type: input.signal_type,
        services: json_string_array(input.services),
        exclude_services: json_string_array(input.exclude_services),
        attribute_filters: input.attribute_filters.unwrap_or_else(|| "[]".to_string()),
        group_by: input.group_by,
        comparator: input.comparator,
        threshold: input.threshold,
        threshold_upper: input.threshold_upper,
        window_minutes: u32::try_from(input.window_minutes).unwrap_or(1),
        minimum_sample_count: u64::from(positive_u32(
            input.minimum_sample_count,
            1,
            "minimumSampleCount",
        )?),
        consecutive_breaches_required: positive_u32(
            input.consecutive_breaches_required,
            2,
            "consecutiveBreachesRequired",
        )?,
        consecutive_healthy_required: positive_u32(
            input.consecutive_healthy_required,
            2,
            "consecutiveHealthyRequired",
        )?,
        no_data_behavior,
        severity: input.severity,
        renotify_interval_minutes: positive_u32(
            input.renotify_interval_minutes,
            30,
            "renotifyIntervalMinutes",
        )?,
        destination_ids: json_string_array(input.destination_ids),
        metric_name: input.metric_name,
        metric_aggregation: input.metric_aggregation,
        created_at_nanos: existing.map_or(now, |rule| rule.created_at_nanos),
        updated_at_nanos: now,
    })
}

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
mod tests;
