//! GraphQL logs domain types and resolvers.

use juniper::{FieldResult, graphql_object};
use parallax_storage::model;

use crate::{ApiContext, MAX_ROWS, clamp_limit, field_err, nanos_string};

use crate::resolvers::common::Point;
use crate::resolvers::traces::AttributeFilterInput;

pub(crate) struct LogRecord(pub(crate) model::LogRow);

#[graphql_object(context = ApiContext)]
impl LogRecord {
    fn ts_nanos(&self) -> String {
        nanos_string(self.0.ts_nanos)
    }
    fn event_name(&self) -> &str {
        &self.0.event_name
    }
    fn observed_ts_nanos(&self) -> String {
        nanos_string(self.0.observed_ts_nanos)
    }
    fn service(&self) -> &str {
        &self.0.service
    }
    fn severity_num(&self) -> i32 {
        self.0.severity_num
    }
    fn severity_text(&self) -> &str {
        &self.0.severity_text
    }
    fn body(&self) -> &str {
        &self.0.body
    }
    fn trace_id(&self) -> &str {
        &self.0.trace_id
    }
    fn span_id(&self) -> &str {
        &self.0.span_id
    }
    fn invocation_id(&self) -> Option<&str> {
        self.0.invocation_id.as_deref()
    }
    fn scope_name(&self) -> &str {
        &self.0.scope_name
    }
    fn attributes(&self) -> String {
        self.0.attributes.to_string()
    }
    fn resource(&self) -> String {
        self.0.resource.to_string()
    }
}

pub(crate) async fn logs_by_trace(
    context: &ApiContext,
    trace_id: String,
) -> FieldResult<Vec<LogRecord>> {
    let trace_id = crate::validate_trace_id(trace_id)?;
    let logs = context.logs_for(&trace_id).await?;
    Ok(logs.iter().cloned().map(LogRecord).collect())
}

pub(crate) async fn logs_by_invocation(
    context: &ApiContext,
    invocation_id: String,
    limit: Option<i32>,
) -> FieldResult<Vec<LogRecord>> {
    let logs = context
        .store
        .logs_by_invocation(&invocation_id, clamp_limit(limit, 500))
        .await
        .map_err(crate::internal_field_err)?;
    Ok(logs.into_iter().map(LogRecord).collect())
}

#[expect(clippy::too_many_arguments, reason = "public GraphQL filter contract")]
pub(crate) async fn logs(
    context: &ApiContext,
    trace_id: Option<String>,
    invocation_id: Option<String>,
    service: Option<String>,
    from_nanos: Option<String>,
    to_nanos: Option<String>,
    severity_min: Option<i32>,
    severity_max: Option<i32>,
    query: Option<String>,
    attribute_filters: Option<Vec<AttributeFilterInput>>,
    limit: Option<i32>,
) -> FieldResult<Vec<LogRecord>> {
    let trace_id = crate::validate_optional_trace_id(trace_id)?;
    let attribute_filters = attribute_filters
        .unwrap_or_default()
        .into_iter()
        .map(|filter| filter.into_adapter().map_err(field_err))
        .collect::<FieldResult<Vec<_>>>()?;
    let from: u128 = match from_nanos {
        Some(s) => s.parse().map_err(|_| field_err("invalid fromNanos"))?,
        None => 0,
    };
    let to: u128 = match to_nanos {
        Some(s) => s.parse().map_err(|_| field_err("invalid toNanos"))?,
        None => u128::MAX,
    };
    let limit = clamp_limit(limit, 500);
    let mut logs = match (&trace_id, &invocation_id) {
        (Some(trace_id), _) => context
            .store
            .logs_by_trace(trace_id)
            .await
            .map_err(crate::internal_field_err)?,
        (None, Some(invocation_id)) => context
            .store
            .logs_by_invocation(invocation_id, MAX_ROWS)
            .await
            .map_err(crate::internal_field_err)?,
        (None, None) => {
            let logs = context
                .store
                .logs_search(
                    service.as_deref(),
                    from..=to,
                    severity_min,
                    severity_max,
                    query.as_deref(),
                    &attribute_filters,
                    limit,
                )
                .await
                .map_err(crate::internal_field_err)?;
            return Ok(logs.into_iter().map(LogRecord).collect());
        }
    };
    // Anchored reads come back ascending and unfiltered: apply the
    // remaining filters here, newest first.
    logs.retain(|l| {
        l.ts_nanos >= from
            && l.ts_nanos <= to
            && service.as_deref().is_none_or(|svc| l.service == svc)
            && severity_min.is_none_or(|min| l.severity_num >= min)
            && severity_max.is_none_or(|max| l.severity_num <= max)
            && query
                .as_deref()
                .is_none_or(|needle| l.body.contains(needle))
            && attribute_filters
                .iter()
                .all(|f| f.matches(log_filter_value(l, &f.key).as_deref()))
    });
    logs.sort_by_key(|l| std::cmp::Reverse(l.ts_nanos));
    logs.truncate(limit);
    Ok(logs.into_iter().map(LogRecord).collect())
}

pub(crate) async fn logs_around(
    context: &ApiContext,
    anchor_nanos: String,
    window_seconds: Option<i32>,
    service: Option<String>,
    trace_id: Option<String>,
    limit: Option<i32>,
) -> FieldResult<Vec<LogRecord>> {
    let trace_id = crate::validate_optional_trace_id(trace_id)?;
    let anchor: u128 = anchor_nanos
        .parse()
        .map_err(|_| field_err("invalid anchorNanos"))?;
    let window =
        u128::try_from(window_seconds.unwrap_or(30).clamp(1, 600)).unwrap_or(30) * 1_000_000_000;
    let from = anchor.saturating_sub(window);
    let to = anchor.saturating_add(window);
    let limit = clamp_limit(limit, 200);
    let mut logs =
        if let Some(trace_id) = trace_id.as_deref().filter(|trace_id| !trace_id.is_empty()) {
            context
                .logs_for(trace_id)
                .await?
                .iter()
                .filter(|log| {
                    log.ts_nanos >= from
                        && log.ts_nanos <= to
                        && service.as_deref().is_none_or(|svc| log.service == svc)
                })
                .cloned()
                .collect::<Vec<_>>()
        } else {
            context
                .store
                .logs_search(service.as_deref(), from..=to, None, None, None, &[], limit)
                .await
                .map_err(crate::internal_field_err)?
        };
    logs.sort_by_key(|log| log.ts_nanos);
    logs.truncate(limit);
    Ok(logs.into_iter().map(LogRecord).collect())
}

#[expect(clippy::too_many_arguments, reason = "public GraphQL filter contract")]
pub(crate) async fn log_count_series(
    context: &ApiContext,
    from_nanos: String,
    to_nanos: String,
    service: Option<String>,
    severity_min: Option<i32>,
    severity_max: Option<i32>,
    query: Option<String>,
    attribute_filters: Option<Vec<AttributeFilterInput>>,
    step_seconds: Option<i32>,
) -> FieldResult<Vec<Point>> {
    let from: u128 = from_nanos
        .parse()
        .map_err(|_| field_err("invalid fromNanos"))?;
    let to: u128 = to_nanos.parse().map_err(|_| field_err("invalid toNanos"))?;
    let step =
        u128::try_from(step_seconds.unwrap_or(60).clamp(1, 86_400)).unwrap_or(60) * 1_000_000_000;
    let series = context
        .store
        .log_count_series(
            service.as_deref(),
            from..=to,
            severity_min,
            severity_max,
            query.as_deref(),
            &attribute_filters
                .unwrap_or_default()
                .into_iter()
                .map(|filter| filter.into_adapter().map_err(field_err))
                .collect::<FieldResult<Vec<_>>>()?,
            step,
        )
        .await
        .map_err(crate::internal_field_err)?;
    Ok(series.into_iter().map(Point).collect())
}

#[cfg(test)]
mod tests;

/// Observed value for a where-clause key on one log row (anchored reads are
/// filtered API-side); mirrors the adapters' log column mapping.
fn log_filter_value(log: &model::LogRow, key: &str) -> Option<String> {
    match key.trim() {
        "service.name" | "service" => Some(log.service.clone()),
        "severity" | "severity_text" => Some(log.severity_text.clone()),
        "severity_number" => Some(log.severity_num.to_string()),
        "body" => Some(log.body.clone()),
        "trace_id" => Some(log.trace_id.clone()),
        "span_id" => Some(log.span_id.clone()),
        key => match log.attributes.get(key)? {
            serde_json::Value::String(value) => Some(value.clone()),
            serde_json::Value::Bool(value) => Some(value.to_string()),
            serde_json::Value::Number(value) => Some(value.to_string()),
            _ => None,
        },
    }
}

/// Bounded log facet dimensions with per-value counts under the same
/// filters as `logs` (plan 164 facet sidebar).
#[expect(clippy::too_many_arguments, reason = "public GraphQL filter contract")]
pub(crate) async fn log_facets(
    context: &ApiContext,
    from_nanos: Option<String>,
    to_nanos: Option<String>,
    service: Option<String>,
    severity_min: Option<i32>,
    severity_max: Option<i32>,
    query: Option<String>,
    attribute_filters: Option<Vec<AttributeFilterInput>>,
) -> FieldResult<Vec<crate::resolvers::traces::Facet>> {
    let from: u128 = match from_nanos {
        Some(s) => s.parse().map_err(|_| field_err("invalid fromNanos"))?,
        None => 0,
    };
    let to: u128 = match to_nanos {
        Some(s) => s.parse().map_err(|_| field_err("invalid toNanos"))?,
        None => u128::MAX,
    };
    let attribute_filters = attribute_filters
        .unwrap_or_default()
        .into_iter()
        .map(|filter| filter.into_adapter().map_err(field_err))
        .collect::<FieldResult<Vec<_>>>()?;
    let facets = context
        .store
        .log_facets(
            service.as_deref(),
            from..=to,
            severity_min,
            severity_max,
            query.as_deref(),
            &attribute_filters,
        )
        .await
        .map_err(crate::internal_field_err)?;
    Ok(facets
        .into_iter()
        .map(crate::resolvers::traces::Facet)
        .collect())
}

/// One Drain pattern cluster (plan 165). Built in-process over a bounded
/// sample of log bodies from the same filter set as `logs`.
pub(crate) struct LogPattern(pub(crate) parallax_analysis::log_patterns::LogPatternCluster);

#[graphql_object(context = ApiContext)]
impl LogPattern {
    fn template(&self) -> &str {
        &self.0.template
    }
    fn count(&self) -> String {
        self.0.count.to_string()
    }
    fn sample_log_id(&self) -> Option<&str> {
        self.0.sample_log_id.as_deref()
    }
    fn first_nanos(&self) -> Option<String> {
        self.0.first_nanos.map(|n| n.to_string())
    }
    fn last_nanos(&self) -> Option<String> {
        self.0.last_nanos.map(|n| n.to_string())
    }
    /// JSON object of severity → count for mix bars.
    fn severity_mix_json(&self) -> String {
        let map: serde_json::Map<String, serde_json::Value> = self
            .0
            .severity_counts
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::from(*v)))
            .collect();
        serde_json::Value::Object(map).to_string()
    }
}

const LOG_PATTERNS_SAMPLE_LIMIT: usize = 10_000;

/// Drain-style log body clusters for the Patterns view (plan 165). Samples up
/// to 10k log rows under the same filters as `logs`, then clusters in-process.
#[expect(clippy::too_many_arguments, reason = "public GraphQL filter contract")]
pub(crate) async fn log_patterns(
    context: &ApiContext,
    from_nanos: Option<String>,
    to_nanos: Option<String>,
    service: Option<String>,
    severity_min: Option<i32>,
    severity_max: Option<i32>,
    query: Option<String>,
    attribute_filters: Option<Vec<AttributeFilterInput>>,
    limit: Option<i32>,
) -> FieldResult<Vec<LogPattern>> {
    let attribute_filters = attribute_filters
        .unwrap_or_default()
        .into_iter()
        .map(|filter| filter.into_adapter().map_err(field_err))
        .collect::<FieldResult<Vec<_>>>()?;
    let from: u128 = match from_nanos {
        Some(s) => s.parse().map_err(|_| field_err("invalid fromNanos"))?,
        None => 0,
    };
    let to: u128 = match to_nanos {
        Some(s) => s.parse().map_err(|_| field_err("invalid toNanos"))?,
        None => u128::MAX,
    };
    let sample_limit = clamp_limit(limit, LOG_PATTERNS_SAMPLE_LIMIT).min(LOG_PATTERNS_SAMPLE_LIMIT);
    let logs = context
        .store
        .logs_search(
            service.as_deref(),
            from..=to,
            severity_min,
            severity_max,
            query.as_deref(),
            &attribute_filters,
            sample_limit,
        )
        .await
        .map_err(crate::internal_field_err)?;
    let log_ids: Vec<String> = logs
        .iter()
        .map(|row| format!("{}:{}:{}", row.ts_nanos, row.service, row.trace_id))
        .collect();
    let inputs: Vec<parallax_analysis::log_patterns::LogLineInput<'_>> = logs
        .iter()
        .zip(log_ids.iter())
        .map(|(row, id)| parallax_analysis::log_patterns::LogLineInput {
            body: row.body.as_str(),
            severity: Some(row.severity_text.as_str()).filter(|s| !s.is_empty()),
            timestamp_nanos: u64::try_from(row.ts_nanos.min(u128::from(u64::MAX))).ok(),
            log_id: Some(id.as_str()),
        })
        .collect();
    let clusters = parallax_analysis::log_patterns::cluster_logs(
        &inputs,
        parallax_analysis::log_patterns::DrainConfig::default(),
    );
    Ok(clusters.into_iter().map(LogPattern).collect())
}
