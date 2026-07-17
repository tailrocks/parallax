//! Pure measurement math for the evaluator (plan 167 step 2, preliminary).
//!
//! Turns per-service aggregates (the shape `service_summaries` already
//! returns) into per-group [`AlertMeasurement`]s according to the rule's
//! signal type and scoping. The storage calls themselves (GreptimeDB via the
//! adapter traits) are peer-owned; this module owns the shared math so the
//! wired `MeasurementSource` stays a thin I/O shim.

use parallax_metadata::AlertRuleRecord;

use super::{AlertMeasurement, GroupMeasurement};

/// Signal types a rule can measure (plan 167 contract).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SignalType {
    ErrorRate,
    P95Latency,
    P99Latency,
    Throughput,
    LogCount,
    Metric,
}

impl SignalType {
    #[must_use]
    pub(crate) fn parse(s: &str) -> Option<Self> {
        match s {
            "error_rate" => Some(Self::ErrorRate),
            "p95_latency" => Some(Self::P95Latency),
            "p99_latency" => Some(Self::P99Latency),
            "throughput" => Some(Self::Throughput),
            "log_count" => Some(Self::LogCount),
            "metric" => Some(Self::Metric),
            _ => None,
        }
    }
}

/// Per-service aggregates over the rule window — the subset of
/// `ServiceSummary` plus p99 that the span-signal types consume.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ServiceWindowStats {
    pub service: String,
    pub span_count: u64,
    pub error_count: u64,
    pub p95_ms: Option<f64>,
    pub p99_ms: Option<f64>,
}

/// Rule scoping decision for one service, honoring `services` (empty = all)
/// and `exclude_services` JSON lists.
#[must_use]
pub(crate) fn service_in_scope(rule: &AlertRuleRecord, service: &str) -> bool {
    let include: Vec<String> = serde_json::from_str(&rule.services).unwrap_or_default();
    let exclude: Vec<String> = serde_json::from_str(&rule.exclude_services).unwrap_or_default();
    if exclude.iter().any(|s| s == service) {
        return false;
    }
    include.is_empty() || include.iter().any(|s| s == service)
}

/// Whether the rule groups per service (`group_by = "service"`); otherwise
/// scoped services aggregate into one ungrouped measurement.
#[must_use]
pub(crate) fn groups_by_service(rule: &AlertRuleRecord) -> bool {
    rule.group_by.as_deref() == Some("service")
}

fn span_signal_value(signal: SignalType, stats: &ServiceWindowStats) -> Option<f64> {
    #[expect(
        clippy::cast_precision_loss,
        reason = "window span counts fit well within f64 mantissa range"
    )]
    match signal {
        SignalType::ErrorRate => {
            if stats.span_count == 0 {
                None
            } else {
                Some(stats.error_count as f64 / stats.span_count as f64)
            }
        }
        SignalType::Throughput => Some(stats.span_count as f64),
        SignalType::P95Latency => stats.p95_ms,
        SignalType::P99Latency => stats.p99_ms,
        SignalType::LogCount | SignalType::Metric => None,
    }
}

/// Derive span-signal measurements (error_rate / latency / throughput) from
/// per-service window stats. Throughput is normalized to spans **per minute**
/// over the rule window so thresholds stay window-length independent.
#[must_use]
pub(crate) fn span_measurements(
    rule: &AlertRuleRecord,
    signal: SignalType,
    stats: &[ServiceWindowStats],
) -> Vec<GroupMeasurement> {
    let scoped: Vec<&ServiceWindowStats> = stats
        .iter()
        .filter(|s| service_in_scope(rule, &s.service))
        .collect();
    let window_minutes = f64::from(rule.window_minutes.max(1));
    let normalize = |signal: SignalType, value: f64| -> f64 {
        if signal == SignalType::Throughput {
            value / window_minutes
        } else {
            value
        }
    };
    if groups_by_service(rule) {
        return scoped
            .into_iter()
            .map(|s| GroupMeasurement {
                group_key: s.service.clone(),
                measurement: AlertMeasurement {
                    value: span_signal_value(signal, s).map(|v| normalize(signal, v)),
                    sample_count: s.span_count,
                },
            })
            .collect();
    }
    // Ungrouped: aggregate the scope into one measurement. Latency percentiles
    // cannot be averaged correctly across services — take the worst (max),
    // which is the alerting-safe direction for slow-latency rules.
    let span_count: u64 = scoped.iter().map(|s| s.span_count).sum();
    let error_count: u64 = scoped.iter().map(|s| s.error_count).sum();
    let merged = ServiceWindowStats {
        service: String::new(),
        span_count,
        error_count,
        p95_ms: scoped
            .iter()
            .filter_map(|s| s.p95_ms)
            .fold(None, |acc: Option<f64>, v| {
                Some(acc.map_or(v, |a| a.max(v)))
            }),
        p99_ms: scoped
            .iter()
            .filter_map(|s| s.p99_ms)
            .fold(None, |acc: Option<f64>, v| {
                Some(acc.map_or(v, |a| a.max(v)))
            }),
    };
    vec![GroupMeasurement {
        group_key: String::new(),
        measurement: AlertMeasurement {
            value: span_signal_value(signal, &merged).map(|v| normalize(signal, v)),
            sample_count: merged.span_count,
        },
    }]
}

/// Wrap a scalar count (log_count / metric aggregate result) into the
/// measurement shape. `None` count = no data in the window.
#[must_use]
pub(crate) fn scalar_measurement(
    group_key: &str,
    value: Option<f64>,
    samples: u64,
) -> GroupMeasurement {
    GroupMeasurement {
        group_key: group_key.to_string(),
        measurement: AlertMeasurement {
            value,
            sample_count: samples,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: u128 = 60 * 1_000_000_000;

    fn rule(services: &str, exclude: &str, group_by: Option<&str>) -> AlertRuleRecord {
        AlertRuleRecord {
            id: "r1".to_string(),
            name: "rule".to_string(),
            enabled: true,
            signal_type: "error_rate".to_string(),
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
            created_at_nanos: MIN,
            updated_at_nanos: MIN,
        }
    }

    fn stats(service: &str, spans: u64, errors: u64, p95: f64, p99: f64) -> ServiceWindowStats {
        ServiceWindowStats {
            service: service.to_string(),
            span_count: spans,
            error_count: errors,
            p95_ms: Some(p95),
            p99_ms: Some(p99),
        }
    }

    #[test]
    fn signal_type_parse_round_trip() {
        for (s, t) in [
            ("error_rate", SignalType::ErrorRate),
            ("p95_latency", SignalType::P95Latency),
            ("p99_latency", SignalType::P99Latency),
            ("throughput", SignalType::Throughput),
            ("log_count", SignalType::LogCount),
            ("metric", SignalType::Metric),
        ] {
            assert_eq!(SignalType::parse(s), Some(t));
        }
        assert_eq!(SignalType::parse("nope"), None);
    }

    #[test]
    fn scope_include_exclude() {
        let all = rule("[]", "[]", None);
        assert!(service_in_scope(&all, "checkout"));
        let only = rule("[\"checkout\"]", "[]", None);
        assert!(service_in_scope(&only, "checkout"));
        assert!(!service_in_scope(&only, "pricing"));
        let minus = rule("[]", "[\"pricing\"]", None);
        assert!(service_in_scope(&minus, "checkout"));
        assert!(!service_in_scope(&minus, "pricing"));
        // Exclude wins over include.
        let both = rule("[\"checkout\"]", "[\"checkout\"]", None);
        assert!(!service_in_scope(&both, "checkout"));
    }

    #[test]
    fn grouped_error_rate_per_service() {
        let rule = rule("[]", "[]", Some("service"));
        let measurements = span_measurements(
            &rule,
            SignalType::ErrorRate,
            &[stats("a", 100, 30, 10.0, 20.0), stats("b", 50, 0, 5.0, 8.0)],
        );
        assert_eq!(measurements.len(), 2);
        assert_eq!(measurements[0].group_key, "a");
        assert_eq!(measurements[0].measurement.value, Some(0.3));
        assert_eq!(measurements[0].measurement.sample_count, 100);
        assert_eq!(measurements[1].measurement.value, Some(0.0));
    }

    #[test]
    fn ungrouped_aggregates_scope_into_one() {
        let rule = rule("[]", "[]", None);
        let measurements = span_measurements(
            &rule,
            SignalType::ErrorRate,
            &[
                stats("a", 100, 30, 10.0, 20.0),
                stats("b", 100, 10, 5.0, 8.0),
            ],
        );
        assert_eq!(measurements.len(), 1);
        assert_eq!(measurements[0].group_key, "");
        assert_eq!(measurements[0].measurement.value, Some(0.2));
        assert_eq!(measurements[0].measurement.sample_count, 200);
    }

    #[test]
    fn ungrouped_latency_takes_worst_service() {
        let rule = rule("[]", "[]", None);
        let measurements = span_measurements(
            &rule,
            SignalType::P95Latency,
            &[stats("a", 10, 0, 10.0, 20.0), stats("b", 10, 0, 90.0, 99.0)],
        );
        assert_eq!(measurements[0].measurement.value, Some(90.0));
        let p99 = span_measurements(
            &rule,
            SignalType::P99Latency,
            &[stats("a", 10, 0, 10.0, 20.0), stats("b", 10, 0, 90.0, 99.0)],
        );
        assert_eq!(p99[0].measurement.value, Some(99.0));
    }

    #[test]
    fn throughput_normalizes_per_minute() {
        let rule = rule("[]", "[]", Some("service"));
        // window_minutes = 5, 500 spans → 100 spans/min.
        let measurements = span_measurements(
            &rule,
            SignalType::Throughput,
            &[stats("a", 500, 0, 1.0, 2.0)],
        );
        assert_eq!(measurements[0].measurement.value, Some(100.0));
    }

    #[test]
    fn zero_spans_is_no_data_for_error_rate() {
        let rule = rule("[]", "[]", Some("service"));
        let measurements = span_measurements(
            &rule,
            SignalType::ErrorRate,
            &[ServiceWindowStats {
                service: "a".to_string(),
                span_count: 0,
                error_count: 0,
                p95_ms: None,
                p99_ms: None,
            }],
        );
        assert_eq!(measurements[0].measurement.value, None);
        assert_eq!(measurements[0].measurement.sample_count, 0);
    }

    #[test]
    fn excluded_services_never_measure() {
        let rule = rule("[]", "[\"b\"]", Some("service"));
        let measurements = span_measurements(
            &rule,
            SignalType::ErrorRate,
            &[stats("a", 10, 1, 1.0, 2.0), stats("b", 10, 9, 1.0, 2.0)],
        );
        assert_eq!(measurements.len(), 1);
        assert_eq!(measurements[0].group_key, "a");
    }

    #[test]
    fn scalar_measurement_wraps() {
        let m = scalar_measurement("checkout", Some(42.0), 42);
        assert_eq!(m.group_key, "checkout");
        assert_eq!(m.measurement.value, Some(42.0));
        assert_eq!(m.measurement.sample_count, 42);
        let none = scalar_measurement("", None, 0);
        assert_eq!(none.measurement.value, None);
    }
}
