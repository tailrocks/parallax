//! Shared validation and range helpers for GraphQL resolvers.

use juniper::FieldResult;
use parallax_storage::adapter::metric_group_label_allowed;
use parallax_storage::model;

use crate::{
    INVESTIGATION_NAME_MAX, INVESTIGATION_NOTES_MAX_BYTES, INVESTIGATION_PIN_CAP,
    SAVED_VIEW_NAME_MAX, field_err,
};

pub(crate) fn validate_saved_view_name(name: &str) -> FieldResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(field_err("saved view name is required"));
    }
    if name.chars().count() > SAVED_VIEW_NAME_MAX {
        return Err(field_err("saved view name is too long"));
    }
    Ok(name.to_string())
}

pub(crate) fn validate_saved_view_page(page: &str) -> FieldResult<()> {
    if page.is_empty() || page.len() > 128 || !page.starts_with('/') {
        return Err(field_err("saved view page must be a route path"));
    }
    Ok(())
}

pub(crate) fn validate_investigation_name(name: &str) -> FieldResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(field_err("investigation name is required"));
    }
    if name.chars().count() > INVESTIGATION_NAME_MAX {
        return Err(field_err("investigation name is too long"));
    }
    Ok(name.to_string())
}

pub(crate) fn validate_investigation_state(state: &str) -> FieldResult<()> {
    let parsed: model::InvestigationState =
        serde_json::from_str(state).map_err(|_| field_err("state must be valid JSON"))?;
    if parsed.version != 1 {
        return Err(field_err("investigation state version must be 1"));
    }
    if parsed.pins.len() > INVESTIGATION_PIN_CAP {
        return Err(field_err("investigation pin cap exceeded"));
    }
    if parsed.notes.len() > INVESTIGATION_NOTES_MAX_BYTES {
        return Err(field_err("investigation notes are too long"));
    }
    Ok(())
}

/// Metric names flow into storage identifiers; keep them inside the OTel metric-name grammar.
pub(crate) fn validate_metric_name(name: &str) -> FieldResult<()> {
    let ok = !name.is_empty()
        && name.len() <= 255
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'));
    if ok {
        Ok(())
    } else {
        Err(field_err("invalid metric name"))
    }
}

pub(crate) fn validate_metric_group_label(label: &str) -> FieldResult<()> {
    let ok = label
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'))
        && metric_group_label_allowed(label);
    if ok {
        Ok(())
    } else {
        Err(field_err(
            "high-cardinality identifier - filter, don't group",
        ))
    }
}

pub(crate) fn parse_range(from_nanos: &str, to_nanos: &str) -> juniper::FieldResult<(u128, u128)> {
    let from: u128 = from_nanos
        .parse()
        .map_err(|_| field_err("invalid fromNanos"))?;
    let to: u128 = to_nanos.parse().map_err(|_| field_err("invalid toNanos"))?;
    if from > to {
        return Err(field_err("fromNanos must be <= toNanos"));
    }
    Ok((from, to))
}

pub(crate) fn step_nanos(step_seconds: Option<i32>) -> u128 {
    u128::try_from(step_seconds.unwrap_or(60).max(1)).unwrap_or(60) * 1_000_000_000
}

/// Default window for windowless pickers (plan 085): now−24h ..= max.
pub(crate) fn retained_recent_range() -> std::ops::RangeInclusive<u128> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    now.saturating_sub(24 * 3_600 * 1_000_000_000)..=u128::MAX
}
