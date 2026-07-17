//! Adapter-backed [`MeasurementSource`] (plan 167 step 2 I/O shim, preliminary).
//!
//! Thin I/O layer between the evaluator and the storage adapter traits: all
//! shared math stays in [`super::measurement`]. Signal mapping:
//!
//! - `error_rate` / `p95_latency` / `throughput` — one `service_summaries`
//!   scan over the rule window.
//! - `p99_latency` — `service_summaries` for counts plus one single-bucket
//!   `span_red_series` call per scoped service for the p99 value (summaries
//!   carry p95 only). Peer may collapse this fan-out into one SQL query.
//! - `log_count` — `log_count_series` with a single window-wide bucket,
//!   severity floor ERROR (OTLP severity number 17; the plan's "logs at
//!   >= severity" — the rule record carries no explicit log-severity field,
//!   peer re-verifies this floor) and the rule's attribute filters parsed as
//!   the plan-164 `{key, op, value}` shape.
//! - `metric` — `metric_series` with a single window-wide bucket using the
//!   rule's `metric_name`/`metric_aggregation`; missing name or unknown
//!   aggregation is a config error (surfaces as a `status='error'` audit row).
//!
//! Scoping: `services`/`exclude_services` resolve to per-service fan-out when
//! non-empty or when the rule groups by service; the whole-system case stays a
//! single unfiltered call. Sample counts for `log_count` are the counted logs
//! themselves; for `metric` they are the number of aggregated buckets (weak —
//! a real datapoint count needs a new adapter surface; peer decides).

use std::ops::RangeInclusive;
use std::sync::Arc;

use anyhow::Context as _;
use parallax_metadata::AlertRuleRecord;
use parallax_storage::adapter::{
    AttributeFilter, AttributeFilterOp, LogCountStore, MetricAnalyticsStore, ServiceAnalyticsStore,
};
use parallax_storage::model::MetricAgg;

use super::{
    GroupMeasurement, MeasurementSource, ServiceWindowStats, SignalType, groups_by_service,
    scalar_measurement, service_in_scope, span_measurements,
};

/// OTLP `SeverityNumber` floor counted by `log_count` rules (ERROR = 17).
pub const LOG_COUNT_SEVERITY_FLOOR: i32 = 17;

/// [`MeasurementSource`] over the storage adapter traits. Generic over the
/// concrete store (GreptimeDB in production, the in-memory store in tests)
/// because `dyn TelemetryStore` does not satisfy per-trait bounds; wire it
/// with the concrete `Arc` before type erasure.
pub struct AdapterMeasurementSource<S: ?Sized> {
    store: Arc<S>,
}

impl<S: ?Sized> AdapterMeasurementSource<S> {
    pub fn new(store: Arc<S>) -> Self {
        Self { store }
    }
}

/// Parse the rule's stored attribute filters (JSON array of
/// `{key, op, value}` with the plan-164 operator tokens). Invalid entries are
/// config errors, not silent skips.
fn parse_attribute_filters(json: &str) -> anyhow::Result<Vec<AttributeFilter>> {
    let value: serde_json::Value =
        serde_json::from_str(json).context("attribute_filters is not valid JSON")?;
    let items = value
        .as_array()
        .context("attribute_filters must be a JSON array")?;
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let key = item
            .get("key")
            .and_then(serde_json::Value::as_str)
            .context("attribute filter missing key")?;
        let op_token = item
            .get("op")
            .and_then(serde_json::Value::as_str)
            .context("attribute filter missing op")?;
        let op = AttributeFilterOp::parse(op_token)
            .with_context(|| format!("unknown attribute filter op: {op_token}"))?;
        let filter_value = item
            .get("value")
            .and_then(serde_json::Value::as_str)
            .context("attribute filter missing value")?;
        out.push(AttributeFilter {
            key: key.to_string(),
            op,
            value: filter_value.to_string(),
        });
    }
    Ok(out)
}

/// Combine per-service scalar values into one ungrouped value per the rule's
/// aggregation. `Rate`/`Sum` add, `Avg` means, `Min`/`Max` take extremes.
fn combine(agg: MetricAgg, values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "value-list lengths fit well within f64 mantissa range"
    )]
    Some(match agg {
        MetricAgg::Sum | MetricAgg::Rate | MetricAgg::Increase => values.iter().sum(),
        MetricAgg::Last => *values.last().unwrap_or(&0.0),
        MetricAgg::Avg => values.iter().sum::<f64>() / values.len() as f64,
        MetricAgg::Min => values.iter().copied().fold(f64::INFINITY, f64::min),
        MetricAgg::Max => values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    })
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "log counts are non-negative engine COUNT() results"
)]
fn count_to_samples(count: f64) -> u64 {
    count.max(0.0).round() as u64
}

impl<S> AdapterMeasurementSource<S>
where
    S: ServiceAnalyticsStore + LogCountStore + MetricAnalyticsStore + ?Sized,
{
    /// Resolve the rule's scope to a concrete service list, or `None` for the
    /// unrestricted whole-system scope (single unfiltered engine call).
    async fn scoped_service_list(
        &self,
        rule: &AlertRuleRecord,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Option<Vec<String>>> {
        let include: Vec<String> = serde_json::from_str(&rule.services).unwrap_or_default();
        let exclude: Vec<String> = serde_json::from_str(&rule.exclude_services).unwrap_or_default();
        if include.is_empty() && exclude.is_empty() {
            return Ok(None);
        }
        let candidates = if include.is_empty() {
            self.store.service_names(range).await?
        } else {
            include
        };
        Ok(Some(
            candidates
                .into_iter()
                .filter(|s| service_in_scope(rule, s))
                .collect(),
        ))
    }

    /// The scope as a concrete list even for the whole system (grouped rules
    /// need one measurement per service).
    async fn grouped_service_list(
        &self,
        rule: &AlertRuleRecord,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<String>> {
        match self.scoped_service_list(rule, range.clone()).await? {
            Some(list) => Ok(list),
            None => Ok(self.store.service_names(range).await?),
        }
    }

    async fn measure_spans(
        &self,
        rule: &AlertRuleRecord,
        signal: SignalType,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<GroupMeasurement>> {
        let summaries = self.store.service_summaries(range.clone()).await?;
        let mut stats: Vec<ServiceWindowStats> = summaries
            .into_iter()
            .map(|s| ServiceWindowStats {
                service: s.name,
                span_count: s.span_count,
                error_count: s.error_count,
                p95_ms: s.p95_ms,
                p99_ms: None,
            })
            .collect();
        if signal == SignalType::P99Latency {
            // Summaries carry p95 only; fetch p99 per scoped service from the
            // trace-derived RED series with one window-wide bucket.
            for stat in &mut stats {
                if stat.span_count == 0 || !service_in_scope(rule, &stat.service) {
                    continue;
                }
                let red = self
                    .store
                    .span_red_series(Some(&stat.service), range.clone(), step_nanos)
                    .await?;
                stat.p99_ms = red
                    .p99
                    .iter()
                    .map(|p| p.value)
                    .fold(None, |acc: Option<f64>, v| {
                        Some(acc.map_or(v, |a| a.max(v)))
                    });
            }
        }
        Ok(span_measurements(rule, signal, &stats))
    }

    async fn log_window_count(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        filters: &[AttributeFilter],
        step_nanos: u128,
    ) -> anyhow::Result<f64> {
        let series = self
            .store
            .log_count_series(
                service,
                range,
                Some(LOG_COUNT_SEVERITY_FLOOR),
                None,
                None,
                filters,
                step_nanos,
            )
            .await?;
        Ok(series.iter().map(|p| p.value).sum())
    }

    async fn measure_logs(
        &self,
        rule: &AlertRuleRecord,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<GroupMeasurement>> {
        let filters = parse_attribute_filters(&rule.attribute_filters)?;
        if groups_by_service(rule) {
            let services = self.grouped_service_list(rule, range.clone()).await?;
            let mut out = Vec::with_capacity(services.len());
            for service in services {
                let count = self
                    .log_window_count(Some(&service), range.clone(), &filters, step_nanos)
                    .await?;
                out.push(scalar_measurement(
                    &service,
                    Some(count),
                    count_to_samples(count),
                ));
            }
            return Ok(out);
        }
        let total = match self.scoped_service_list(rule, range.clone()).await? {
            None => {
                self.log_window_count(None, range, &filters, step_nanos)
                    .await?
            }
            Some(services) => {
                let mut total = 0.0;
                for service in services {
                    total += self
                        .log_window_count(Some(&service), range.clone(), &filters, step_nanos)
                        .await?;
                }
                total
            }
        };
        Ok(vec![scalar_measurement(
            "",
            Some(total),
            count_to_samples(total),
        )])
    }

    async fn metric_window_value(
        &self,
        name: &str,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
        agg: MetricAgg,
    ) -> anyhow::Result<(Option<f64>, u64)> {
        let points = self
            .store
            .metric_series(name, service, None, &[], range, step_nanos, agg)
            .await?;
        let values: Vec<f64> = points.iter().map(|p| p.value).collect();
        Ok((combine(agg, &values), points.len() as u64))
    }

    async fn measure_metric(
        &self,
        rule: &AlertRuleRecord,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<GroupMeasurement>> {
        let name = rule
            .metric_name
            .as_deref()
            .context("metric rule missing metric_name")?;
        let agg_token = rule.metric_aggregation.as_deref().unwrap_or("avg");
        let agg = MetricAgg::parse(agg_token)
            .with_context(|| format!("unknown metric_aggregation: {agg_token}"))?;
        if groups_by_service(rule) {
            let services = self.grouped_service_list(rule, range.clone()).await?;
            let mut out = Vec::with_capacity(services.len());
            for service in services {
                let (value, samples) = self
                    .metric_window_value(name, Some(&service), range.clone(), step_nanos, agg)
                    .await?;
                out.push(scalar_measurement(&service, value, samples));
            }
            return Ok(out);
        }
        let (value, samples) = match self.scoped_service_list(rule, range.clone()).await? {
            None => {
                self.metric_window_value(name, None, range, step_nanos, agg)
                    .await?
            }
            Some(services) => {
                let mut values = Vec::with_capacity(services.len());
                let mut samples = 0u64;
                for service in services {
                    let (value, count) = self
                        .metric_window_value(name, Some(&service), range.clone(), step_nanos, agg)
                        .await?;
                    if let Some(value) = value {
                        values.push(value);
                    }
                    samples += count;
                }
                (combine(agg, &values), samples)
            }
        };
        Ok(vec![scalar_measurement("", value, samples)])
    }
}

#[async_trait::async_trait]
impl<S> MeasurementSource for AdapterMeasurementSource<S>
where
    S: ServiceAnalyticsStore + LogCountStore + MetricAnalyticsStore + ?Sized,
{
    async fn measure(
        &self,
        rule: &AlertRuleRecord,
        from_nanos: u128,
        to_nanos: u128,
    ) -> anyhow::Result<Vec<GroupMeasurement>> {
        let signal = SignalType::parse(&rule.signal_type)
            .with_context(|| format!("unknown signal_type: {}", rule.signal_type))?;
        let range = from_nanos..=to_nanos;
        // One bucket spanning the whole rule window.
        let step_nanos = to_nanos.saturating_sub(from_nanos).max(1);
        match signal {
            SignalType::ErrorRate
            | SignalType::P95Latency
            | SignalType::P99Latency
            | SignalType::Throughput => self.measure_spans(rule, signal, range, step_nanos).await,
            SignalType::LogCount => self.measure_logs(rule, range, step_nanos).await,
            SignalType::Metric => self.measure_metric(rule, range, step_nanos).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use parallax_storage::adapter::{
        OverviewTotals, ReleaseWindow, ServiceCatalogRow, ServiceSummary, SignalKind, SpanRed,
        StorageResult,
    };
    use parallax_storage::model::{MetricExemplarRow, SeriesPoint};

    use super::*;

    const MIN_NANOS: u128 = 60 * 1_000_000_000;

    #[derive(Default)]
    struct StubStore {
        summaries: Vec<ServiceSummary>,
        p99_by_service: HashMap<String, f64>,
        service_names: Vec<String>,
        log_counts_by_service: HashMap<String, f64>,
        log_count_all: f64,
        metric_by_service: HashMap<String, f64>,
        log_calls: Mutex<Vec<(Option<String>, Option<i32>, usize)>>,
    }

    fn summary(name: &str, spans: u64, errors: u64, p95: f64) -> ServiceSummary {
        ServiceSummary {
            name: name.to_string(),
            last_seen_nanos: MIN_NANOS,
            span_count: spans,
            error_count: errors,
            p95_ms: Some(p95),
        }
    }

    #[async_trait::async_trait]
    impl ServiceAnalyticsStore for StubStore {
        async fn service_names(&self, _range: RangeInclusive<u128>) -> StorageResult<Vec<String>> {
            Ok(self.service_names.clone())
        }
        async fn overview_totals(
            &self,
            _range: RangeInclusive<u128>,
        ) -> StorageResult<OverviewTotals> {
            unimplemented!("not used by the measurement shim")
        }
        async fn signal_count_series(
            &self,
            _kind: SignalKind,
            _service: Option<&str>,
            _range: RangeInclusive<u128>,
            _step_nanos: u128,
        ) -> StorageResult<Vec<SeriesPoint>> {
            unimplemented!("not used by the measurement shim")
        }
        async fn service_summaries(
            &self,
            _range: RangeInclusive<u128>,
        ) -> StorageResult<Vec<ServiceSummary>> {
            Ok(self.summaries.clone())
        }
        async fn release_windows(
            &self,
            _service: &str,
            _range: RangeInclusive<u128>,
        ) -> StorageResult<Vec<ReleaseWindow>> {
            unimplemented!("not used by the measurement shim")
        }
        async fn service_catalog(
            &self,
            _range: RangeInclusive<u128>,
        ) -> StorageResult<Vec<ServiceCatalogRow>> {
            unimplemented!("not used by the measurement shim")
        }
        async fn span_red_series(
            &self,
            service: Option<&str>,
            _range: RangeInclusive<u128>,
            _step_nanos: u128,
        ) -> StorageResult<SpanRed> {
            let mut red = SpanRed::default();
            if let Some(p99) = service.and_then(|s| self.p99_by_service.get(s)) {
                red.p99 = vec![SeriesPoint {
                    ts_nanos: MIN_NANOS,
                    value: *p99,
                }];
            }
            Ok(red)
        }
    }

    #[async_trait::async_trait]
    impl LogCountStore for StubStore {
        async fn log_count_series(
            &self,
            service: Option<&str>,
            _range: RangeInclusive<u128>,
            severity_min: Option<i32>,
            _severity_max: Option<i32>,
            _body_contains: Option<&str>,
            attribute_filters: &[AttributeFilter],
            _step_nanos: u128,
        ) -> StorageResult<Vec<SeriesPoint>> {
            self.log_calls.lock().unwrap().push((
                service.map(str::to_string),
                severity_min,
                attribute_filters.len(),
            ));
            let count = match service {
                Some(service) => self
                    .log_counts_by_service
                    .get(service)
                    .copied()
                    .unwrap_or(0.0),
                None => self.log_count_all,
            };
            Ok(vec![SeriesPoint {
                ts_nanos: MIN_NANOS,
                value: count,
            }])
        }
    }

    #[async_trait::async_trait]
    impl MetricAnalyticsStore for StubStore {
        async fn metric_series(
            &self,
            _name: &str,
            service: Option<&str>,
            _invocation_id: Option<&str>,
            _range: RangeInclusive<u128>,
            _step_nanos: u128,
            _agg: MetricAgg,
        ) -> StorageResult<Vec<SeriesPoint>> {
            let value = match service {
                Some(service) => self.metric_by_service.get(service).copied(),
                None => Some(
                    self.metric_by_service.values().sum::<f64>()
                        / self.metric_by_service.len().max(1) as f64,
                ),
            };
            Ok(value
                .map(|v| {
                    vec![SeriesPoint {
                        ts_nanos: MIN_NANOS,
                        value: v,
                    }]
                })
                .unwrap_or_default())
        }
        async fn histogram_quantile(
            &self,
            _name: &str,
            _service: Option<&str>,
            _range: RangeInclusive<u128>,
            _step_nanos: u128,
            _q: f64,
        ) -> StorageResult<Vec<SeriesPoint>> {
            unimplemented!("not used by the measurement shim")
        }
        async fn metric_exemplars(
            &self,
            _name: &str,
            _service: Option<&str>,
            _range: RangeInclusive<u128>,
            _limit: usize,
        ) -> StorageResult<Vec<MetricExemplarRow>> {
            unimplemented!("not used by the measurement shim")
        }
    }

    fn rule(
        signal: &str,
        services: &str,
        exclude: &str,
        group_by: Option<&str>,
    ) -> AlertRuleRecord {
        AlertRuleRecord {
            id: "r1".to_string(),
            name: "rule".to_string(),
            enabled: true,
            signal_type: signal.to_string(),
            services: services.to_string(),
            exclude_services: exclude.to_string(),
            attribute_filters: "[]".to_string(),
            group_by: group_by.map(str::to_string),
            comparator: "gt".to_string(),
            threshold: 0.2,
            threshold_upper: None,
            window_minutes: 5,
            minimum_sample_count: 1,
            consecutive_breaches_required: 2,
            consecutive_healthy_required: 2,
            no_data_behavior: "skip".to_string(),
            severity: "warning".to_string(),
            renotify_interval_minutes: 30,
            destination_ids: "[]".to_string(),
            metric_name: None,
            metric_aggregation: None,
            created_at_nanos: MIN_NANOS,
            updated_at_nanos: MIN_NANOS,
        }
    }

    fn source(stub: StubStore) -> AdapterMeasurementSource<StubStore> {
        AdapterMeasurementSource::new(Arc::new(stub))
    }

    #[tokio::test]
    async fn error_rate_grouped_from_summaries() {
        let source = source(StubStore {
            summaries: vec![summary("a", 100, 30, 10.0), summary("b", 50, 0, 5.0)],
            ..StubStore::default()
        });
        let rule = rule("error_rate", "[]", "[]", Some("service"));
        let groups = source.measure(&rule, 0, 5 * MIN_NANOS).await.unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].group_key, "a");
        assert_eq!(groups[0].measurement.value, Some(0.3));
        assert_eq!(groups[1].measurement.value, Some(0.0));
    }

    #[tokio::test]
    async fn p99_pulls_red_series_per_scoped_service() {
        let source = source(StubStore {
            summaries: vec![summary("a", 10, 0, 10.0), summary("b", 10, 0, 20.0)],
            p99_by_service: HashMap::from([("a".to_string(), 42.0), ("b".to_string(), 99.0)]),
            ..StubStore::default()
        });
        // Ungrouped p99 takes the worst scoped service.
        let ungrouped = rule("p99_latency", "[]", "[]", None);
        let groups = source.measure(&ungrouped, 0, 5 * MIN_NANOS).await.unwrap();
        assert_eq!(groups[0].measurement.value, Some(99.0));
        // Excluding the slow service drops its p99 from scope.
        let scoped = rule("p99_latency", "[]", "[\"b\"]", None);
        let groups = source.measure(&scoped, 0, 5 * MIN_NANOS).await.unwrap();
        assert_eq!(groups[0].measurement.value, Some(42.0));
    }

    #[tokio::test]
    async fn log_count_whole_system_is_one_unfiltered_call() {
        let source = source(StubStore {
            log_count_all: 120.0,
            ..StubStore::default()
        });
        let rule = rule("log_count", "[]", "[]", None);
        let groups = source.measure(&rule, 0, 5 * MIN_NANOS).await.unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].measurement.value, Some(120.0));
        assert_eq!(groups[0].measurement.sample_count, 120);
        let calls = source.store.log_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, None);
        assert_eq!(calls[0].1, Some(LOG_COUNT_SEVERITY_FLOOR));
    }

    #[tokio::test]
    async fn log_count_grouped_resolves_exclusions_via_service_names() {
        let source = source(StubStore {
            service_names: vec!["a".to_string(), "b".to_string()],
            log_counts_by_service: HashMap::from([
                ("a".to_string(), 7.0),
                ("b".to_string(), 100.0),
            ]),
            ..StubStore::default()
        });
        let rule = rule("log_count", "[]", "[\"b\"]", Some("service"));
        let groups = source.measure(&rule, 0, 5 * MIN_NANOS).await.unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].group_key, "a");
        assert_eq!(groups[0].measurement.value, Some(7.0));
    }

    #[tokio::test]
    async fn metric_requires_name_and_known_aggregation() {
        let source = source(StubStore::default());
        let nameless = rule("metric", "[]", "[]", None);
        assert!(source.measure(&nameless, 0, MIN_NANOS).await.is_err());
        let mut bad_agg = rule("metric", "[]", "[]", None);
        bad_agg.metric_name = Some("shapes.region.load".to_string());
        bad_agg.metric_aggregation = Some("median".to_string());
        assert!(source.measure(&bad_agg, 0, MIN_NANOS).await.is_err());
    }

    #[tokio::test]
    async fn metric_grouped_measures_each_included_service() {
        let source = source(StubStore {
            metric_by_service: HashMap::from([("a".to_string(), 6.0), ("b".to_string(), 3.0)]),
            ..StubStore::default()
        });
        let mut rule = rule("metric", "[\"a\",\"b\"]", "[]", Some("service"));
        rule.metric_name = Some("shapes.region.load".to_string());
        rule.metric_aggregation = Some("max".to_string());
        let groups = source.measure(&rule, 0, 5 * MIN_NANOS).await.unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].group_key, "a");
        assert_eq!(groups[0].measurement.value, Some(6.0));
        assert_eq!(groups[1].measurement.value, Some(3.0));
        assert_eq!(groups[0].measurement.sample_count, 1);
    }

    #[test]
    fn attribute_filter_parsing_round_trip_and_errors() {
        let parsed =
            parse_attribute_filters(r#"[{"key":"http.route","op":"=","value":"/checkout"}]"#)
                .unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].key, "http.route");
        assert!(parse_attribute_filters("not json").is_err());
        assert!(parse_attribute_filters(r#"{"key":"x"}"#).is_err());
        assert!(parse_attribute_filters(r#"[{"key":"x","op":"~~","value":"y"}]"#).is_err());
    }

    #[test]
    fn combine_covers_all_aggregations() {
        let values = [1.0, 3.0, 8.0];
        assert_eq!(combine(MetricAgg::Sum, &values), Some(12.0));
        assert_eq!(combine(MetricAgg::Rate, &values), Some(12.0));
        assert_eq!(combine(MetricAgg::Avg, &values), Some(4.0));
        assert_eq!(combine(MetricAgg::Min, &values), Some(1.0));
        assert_eq!(combine(MetricAgg::Max, &values), Some(8.0));
        assert_eq!(combine(MetricAgg::Sum, &[]), None);
    }
}
