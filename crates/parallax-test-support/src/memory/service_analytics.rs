//! In-memory service analytics capability.

use super::*;

#[async_trait::async_trait]
impl adapter::ServiceAnalyticsStore for MemoryStore {
    async fn service_names(&self, range: RangeInclusive<u128>) -> anyhow::Result<Vec<String>> {
        let inner = self.lock();
        let mut names: Vec<String> = inner
            .metric_points
            .iter()
            .filter(|p| range.contains(&p.ts_nanos))
            .map(|p| p.service.clone())
            .chain(
                inner
                    .spans
                    .iter()
                    .filter(|s| range.contains(&s.ts_nanos))
                    .map(|s| s.service.clone()),
            )
            .chain(
                inner
                    .logs
                    .iter()
                    .filter(|l| range.contains(&l.ts_nanos))
                    .map(|l| l.service.clone()),
            )
            .collect();
        names.sort();
        names.dedup();
        Ok(names)
    }

    async fn overview_totals(&self, range: RangeInclusive<u128>) -> anyhow::Result<OverviewTotals> {
        let inner = self.lock();
        let spans: Vec<&SpanRow> = inner
            .spans
            .iter()
            .filter(|s| range.contains(&s.ts_nanos))
            .collect();
        let logs = inner
            .logs
            .iter()
            .filter(|l| range.contains(&l.ts_nanos))
            .count() as u64;
        let metric_points = inner
            .metric_points
            .iter()
            .filter(|p| range.contains(&p.ts_nanos))
            .count() as u64
            + inner
                .histograms
                .iter()
                .filter(|h| range.contains(&h.ts_nanos))
                .count() as u64;
        let errors = spans
            .iter()
            .filter(|s| s.status_code == "STATUS_CODE_ERROR")
            .count() as u64;
        let trace_count = spans
            .iter()
            .map(|s| s.trace_id.as_str())
            .collect::<BTreeSet<_>>()
            .len() as u64;
        let active_services = spans
            .iter()
            .map(|s| s.service.as_str())
            .chain(
                inner
                    .logs
                    .iter()
                    .filter(|l| range.contains(&l.ts_nanos))
                    .map(|l| l.service.as_str()),
            )
            .chain(
                inner
                    .metric_points
                    .iter()
                    .filter(|p| range.contains(&p.ts_nanos))
                    .map(|p| p.service.as_str()),
            )
            .chain(
                inner
                    .histograms
                    .iter()
                    .filter(|h| range.contains(&h.ts_nanos))
                    .map(|h| h.service.as_str()),
            )
            .collect::<BTreeSet<_>>()
            .len() as u64;
        let span_count = spans.len() as u64;
        Ok(OverviewTotals {
            span_count,
            trace_count,
            log_count: logs,
            metric_point_count: metric_points,
            error_count: errors,
            error_rate: if span_count == 0 {
                0.0
            } else {
                errors as f64 / span_count as f64
            },
            active_services,
        })
    }

    async fn signal_count_series(
        &self,
        kind: SignalKind,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<Vec<SeriesPoint>> {
        let step = step_nanos.max(1);
        let inner = self.lock();
        let mut buckets: BTreeMap<u128, u64> = Default::default();
        match kind {
            SignalKind::Spans => {
                for span in inner.spans.iter().filter(|s| {
                    range.contains(&s.ts_nanos) && service.is_none_or(|svc| s.service == svc)
                }) {
                    *buckets.entry((span.ts_nanos / step) * step).or_default() += 1;
                }
            }
            SignalKind::Traces => {
                let mut traces: BTreeMap<u128, BTreeSet<&str>> = Default::default();
                for span in inner.spans.iter().filter(|s| {
                    range.contains(&s.ts_nanos) && service.is_none_or(|svc| s.service == svc)
                }) {
                    traces
                        .entry((span.ts_nanos / step) * step)
                        .or_default()
                        .insert(span.trace_id.as_str());
                }
                return Ok(traces
                    .into_iter()
                    .map(|(ts_nanos, trace_ids)| SeriesPoint {
                        ts_nanos,
                        value: trace_ids.len() as f64,
                    })
                    .collect());
            }
            SignalKind::Logs => {
                for log in inner.logs.iter().filter(|l| {
                    range.contains(&l.ts_nanos) && service.is_none_or(|svc| l.service == svc)
                }) {
                    *buckets.entry((log.ts_nanos / step) * step).or_default() += 1;
                }
            }
            SignalKind::Errors => {
                for event in inner.error_events.iter().filter(|e| {
                    range.contains(&e.ts_nanos) && service.is_none_or(|svc| e.service == svc)
                }) {
                    *buckets.entry((event.ts_nanos / step) * step).or_default() += 1;
                }
            }
            SignalKind::MetricPoints => {
                for point in inner.metric_points.iter().filter(|p| {
                    range.contains(&p.ts_nanos) && service.is_none_or(|svc| p.service == svc)
                }) {
                    *buckets.entry((point.ts_nanos / step) * step).or_default() += 1;
                }
                for row in inner.histograms.iter().filter(|h| {
                    range.contains(&h.ts_nanos) && service.is_none_or(|svc| h.service == svc)
                }) {
                    *buckets.entry((row.ts_nanos / step) * step).or_default() += 1;
                }
            }
        }
        Ok(buckets
            .into_iter()
            .map(|(ts_nanos, count)| SeriesPoint {
                ts_nanos,
                value: count as f64,
            })
            .collect())
    }

    async fn service_summaries(
        &self,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<ServiceSummary>> {
        let inner = self.lock();
        let mut by_service: BTreeMap<&str, Vec<&SpanRow>> = Default::default();
        for span in inner.spans.iter().filter(|s| range.contains(&s.ts_nanos)) {
            by_service.entry(&span.service).or_default().push(span);
        }
        let mut summaries: Vec<_> = by_service
            .into_iter()
            .map(|(name, spans)| {
                let mut durations: Vec<u128> = spans.iter().map(|s| s.duration_ns).collect();
                durations.sort_unstable();
                ServiceSummary {
                    name: name.to_owned(),
                    last_seen_nanos: spans.iter().map(|s| s.ts_nanos).max().unwrap_or(0),
                    span_count: spans.len() as u64,
                    error_count: spans
                        .iter()
                        .filter(|s| s.status_code == "STATUS_CODE_ERROR")
                        .count() as u64,
                    p95_ms: Some(quantile_from_sorted(&durations, 0.95) / 1_000_000.0),
                }
            })
            .collect();
        summaries.sort_by_key(|s| std::cmp::Reverse(s.last_seen_nanos));
        Ok(summaries)
    }

    async fn release_windows(
        &self,
        service: &str,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<ReleaseWindow>> {
        let inner = self.lock();
        let mut by_version: BTreeMap<String, ReleaseWindow> = BTreeMap::new();
        for span in inner
            .spans
            .iter()
            .filter(|s| s.service == service && range.contains(&s.ts_nanos))
        {
            let Some(version) = span
                .resource
                .get(semconv::SERVICE_VERSION)
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|value| !value.is_empty())
            else {
                continue;
            };
            let window = by_version
                .entry(version.to_string())
                .or_insert_with(|| ReleaseWindow {
                    version: version.to_string(),
                    first_seen_nanos: span.ts_nanos,
                    last_seen_nanos: span.ts_nanos,
                    span_count: 0,
                });
            window.first_seen_nanos = window.first_seen_nanos.min(span.ts_nanos);
            window.last_seen_nanos = window.last_seen_nanos.max(span.ts_nanos);
            window.span_count += 1;
        }
        let mut windows: Vec<_> = by_version.into_values().collect();
        windows.sort_by_key(|window| (window.first_seen_nanos, window.version.clone()));
        Ok(windows)
    }

    async fn service_catalog(
        &self,
        range: RangeInclusive<u128>,
    ) -> anyhow::Result<Vec<ServiceCatalogRow>> {
        #[derive(Default)]
        struct CatalogAgg {
            latest: Option<SpanRow>,
            instances: BTreeSet<String>,
        }

        let inner = self.lock();
        let mut by_service: BTreeMap<String, CatalogAgg> = BTreeMap::new();
        for span in inner.spans.iter().filter(|s| range.contains(&s.ts_nanos)) {
            let entry = by_service.entry(span.service.clone()).or_default();
            if entry
                .latest
                .as_ref()
                .is_none_or(|latest| span.ts_nanos >= latest.ts_nanos)
            {
                entry.latest = Some(span.clone());
            }
            if let Some(instance) = resource_string(&span.resource, "service.instance.id") {
                entry.instances.insert(instance);
            }
        }

        let mut rows = Vec::new();
        for (name, agg) in by_service {
            let Some(latest) = agg.latest else { continue };
            rows.push(ServiceCatalogRow {
                name,
                service_version: resource_string(&latest.resource, semconv::SERVICE_VERSION),
                service_namespace: resource_string(&latest.resource, semconv::SERVICE_NAMESPACE),
                deployment_environment: resource_string(
                    &latest.resource,
                    semconv::DEPLOYMENT_ENVIRONMENT_NAME,
                )
                .or_else(|| resource_string(&latest.resource, semconv::DEPLOYMENT_ENVIRONMENT)),
                telemetry_sdk_language: resource_string(&latest.resource, "telemetry.sdk.language"),
                telemetry_sdk_name: resource_string(&latest.resource, "telemetry.sdk.name"),
                telemetry_sdk_version: resource_string(&latest.resource, "telemetry.sdk.version"),
                last_seen_nanos: latest.ts_nanos,
                instance_count: agg.instances.len() as u64,
            });
        }
        rows.sort_by_key(|row| row.name.clone());
        rows.truncate(MAX_ROWS);
        Ok(rows)
    }

    async fn span_red_series(
        &self,
        service: Option<&str>,
        range: RangeInclusive<u128>,
        step_nanos: u128,
    ) -> anyhow::Result<SpanRed> {
        let step = step_nanos.max(1);
        let step_secs = step as f64 / 1_000_000_000.0;
        let inner = self.lock();
        let mut buckets: BTreeMap<u128, Vec<&SpanRow>> = Default::default();
        for span in inner
            .spans
            .iter()
            .filter(|s| range.contains(&s.ts_nanos) && service.is_none_or(|svc| s.service == svc))
        {
            buckets
                .entry((span.ts_nanos / step) * step)
                .or_default()
                .push(span);
        }
        let mut red = SpanRed::default();
        for (ts_nanos, spans) in buckets {
            let count = spans.len() as f64;
            let errors = spans
                .iter()
                .filter(|s| s.status_code == "STATUS_CODE_ERROR")
                .count() as f64;
            let mut durations: Vec<u128> = spans.iter().map(|s| s.duration_ns).collect();
            durations.sort_unstable();
            red.rate.push(SeriesPoint {
                ts_nanos,
                value: count / step_secs,
            });
            red.error_rate.push(SeriesPoint {
                ts_nanos,
                value: if count == 0.0 { 0.0 } else { errors / count },
            });
            red.p50.push(SeriesPoint {
                ts_nanos,
                value: quantile_from_sorted(&durations, 0.50) / 1_000_000.0,
            });
            red.p95.push(SeriesPoint {
                ts_nanos,
                value: quantile_from_sorted(&durations, 0.95) / 1_000_000.0,
            });
            red.p99.push(SeriesPoint {
                ts_nanos,
                value: quantile_from_sorted(&durations, 0.99) / 1_000_000.0,
            });
        }
        Ok(red)
    }
}
