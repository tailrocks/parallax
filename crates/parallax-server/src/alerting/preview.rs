//! Draft-rule preview: measure + state machine, no persistence (plan 171).

use anyhow::Context as _;
use parallax_api::{AlertPreviewData, AlertPreviewGroupData, AlertPreviewPointData};
use parallax_metadata::AlertRuleRecord;
use std::collections::BTreeMap;

use super::{
    AlertMeasurement, AlertTransition, GroupMeasurement, MeasurementSource, RuleEvalState,
    eval_config, evaluate_rule,
};

/// Hard cap on preview buckets (metric-summary bound).
pub(crate) const PREVIEW_MAX_BUCKETS: u32 = 24;
pub(crate) const MINUTES_NS: u128 = 60 * 1_000_000_000;

/// Walk `window_minutes` as up to [`PREVIEW_MAX_BUCKETS`] buckets and run the
/// pure evaluator from a blank state. Never writes incidents or rule state.
pub(crate) async fn preview_rule(
    source: &dyn MeasurementSource,
    rule: &AlertRuleRecord,
    window_minutes: u32,
    now_nanos: u128,
) -> anyhow::Result<AlertPreviewData> {
    let window_minutes = window_minutes.clamp(1, 24 * 60);
    let config = eval_config(rule).context("preview eval config")?;
    let buckets = PREVIEW_MAX_BUCKETS.min(window_minutes.max(1));
    let bucket_minutes = window_minutes.div_ceil(buckets).max(1);
    let bucket_ns = u128::from(bucket_minutes) * MINUTES_NS;
    let start = now_nanos.saturating_sub(u128::from(window_minutes) * MINUTES_NS);

    let mut states: BTreeMap<String, RuleEvalState> = BTreeMap::new();
    let mut series: BTreeMap<String, Vec<AlertPreviewPointData>> = BTreeMap::new();
    let mut sufficient: BTreeMap<String, bool> = BTreeMap::new();

    for index in 0..buckets {
        let from = start + u128::from(index) * bucket_ns;
        let to = from.saturating_add(bucket_ns).min(now_nanos);
        if to <= from {
            continue;
        }
        let measured = source
            .measure(rule, from, to)
            .await
            .with_context(|| format!("preview measure bucket {index}"))?;
        let groups = if measured.is_empty() {
            vec![GroupMeasurement {
                group_key: String::new(),
                measurement: AlertMeasurement {
                    value: None,
                    sample_count: 0,
                },
            }]
        } else {
            measured
        };
        let now_secs = i64::try_from(to / 1_000_000_000).unwrap_or(i64::MAX);
        for group in groups {
            let prev = states.get(&group.group_key).cloned().unwrap_or_default();
            let outcome = evaluate_rule(&config, &prev, group.measurement, now_secs);
            let would_fire = matches!(
                outcome.transition,
                AlertTransition::OpenIncident | AlertTransition::Renotify
            ) || outcome.state.incident_open;
            let samples_ok = group.measurement.sample_count >= config.minimum_sample_count;
            sufficient
                .entry(group.group_key.clone())
                .and_modify(|ok| *ok = *ok || samples_ok)
                .or_insert(samples_ok);
            series
                .entry(group.group_key.clone())
                .or_default()
                .push(AlertPreviewPointData {
                    ts_nanos: to.to_string(),
                    value: outcome.effective_value,
                    sample_count: group.measurement.sample_count,
                    would_fire,
                });
            states.insert(group.group_key, outcome.state);
        }
    }

    let groups = series
        .into_iter()
        .map(|(group_key, points)| AlertPreviewGroupData {
            samples_sufficient: sufficient.get(&group_key).copied().unwrap_or(false),
            group_key,
            points,
        })
        .collect();
    Ok(AlertPreviewData {
        window_minutes,
        groups,
    })
}
