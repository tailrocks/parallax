//! Alert rule input validation (plan 167 / 171).

use juniper::FieldResult;
use parallax_metadata::AlertRuleRecord;

use super::{
    ALERT_COMPARATORS, ALERT_NAME_MAX, ALERT_NO_DATA_BEHAVIORS, ALERT_SEVERITIES, ALERT_SIGNAL_TYPES,
    AlertRuleInput, now_nanos,
};
use crate::field_err;

pub(super) fn json_string_array(values: Option<Vec<String>>) -> String {
    serde_json::to_string(&values.unwrap_or_default()).unwrap_or_else(|_| "[]".to_string())
}

pub(super) fn positive_u32(value: Option<i32>, default: u32, label: &str) -> FieldResult<u32> {
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
pub(crate) fn validated_rule(
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

