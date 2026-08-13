//! GraphQL types + resolver for `alertRulePreview` (plan 171).

use juniper::{FieldResult, graphql_object};

use super::{AlertRuleInput, now_nanos, validated_rule};
use crate::{ApiContext, field_err, saturate_i32};

pub(crate) struct AlertPreviewPoint {
    ts_nanos: String,
    value: Option<f64>,
    sample_count: i32,
    would_fire: bool,
}

#[graphql_object(context = ApiContext)]
impl AlertPreviewPoint {
    fn ts_nanos(&self) -> &str {
        &self.ts_nanos
    }
    fn value(&self) -> Option<f64> {
        self.value
    }
    fn sample_count(&self) -> i32 {
        self.sample_count
    }
    fn would_fire(&self) -> bool {
        self.would_fire
    }
}

pub(crate) struct AlertPreviewGroup {
    group_key: String,
    points: Vec<AlertPreviewPoint>,
    samples_sufficient: bool,
}

#[graphql_object(context = ApiContext)]
impl AlertPreviewGroup {
    fn group_key(&self) -> &str {
        &self.group_key
    }
    fn points(&self) -> &[AlertPreviewPoint] {
        &self.points
    }
    fn samples_sufficient(&self) -> bool {
        self.samples_sufficient
    }
}

pub(crate) struct AlertRulePreview {
    window_minutes: i32,
    groups: Vec<AlertPreviewGroup>,
}

#[graphql_object(context = ApiContext)]
impl AlertRulePreview {
    fn window_minutes(&self) -> i32 {
        self.window_minutes
    }
    fn groups(&self) -> &[AlertPreviewGroup] {
        &self.groups
    }
}

pub(crate) async fn alert_rule_preview(
    context: &ApiContext,
    input: AlertRuleInput,
    window_minutes: Option<i32>,
) -> FieldResult<AlertRulePreview> {
    let previewer = context
        .alert_previewer
        .as_ref()
        .ok_or_else(|| field_err("alert preview is not available on this server"))?;
    let rule = validated_rule(input, None)?;
    let window = match window_minutes {
        Some(minutes) if minutes >= 1 => u32::try_from(minutes).unwrap_or(rule.window_minutes),
        _ => rule.window_minutes,
    };
    let now = now_nanos();
    let data = previewer
        .preview(rule, window, now)
        .await
        .map_err(|error| field_err(format!("alert preview failed: {error}")))?;
    Ok(AlertRulePreview {
        window_minutes: i32::try_from(data.window_minutes).unwrap_or(i32::MAX),
        groups: data
            .groups
            .into_iter()
            .map(|group| AlertPreviewGroup {
                group_key: group.group_key,
                samples_sufficient: group.samples_sufficient,
                points: group
                    .points
                    .into_iter()
                    .map(|point| AlertPreviewPoint {
                        ts_nanos: point.ts_nanos,
                        value: point.value,
                        sample_count: saturate_i32(point.sample_count),
                        would_fire: point.would_fire,
                    })
                    .collect(),
            })
            .collect(),
    })
}
