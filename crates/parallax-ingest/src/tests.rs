use super::*;
use parallax_proto::metrics::{
    Exemplar, ExponentialHistogram, Gauge, Histogram, HistogramDataPoint, Metric, NumberDataPoint,
    Summary, exemplar::Value as ExemplarValue, metric::Data,
    number_data_point::Value as NumberValue,
};

fn string_kv(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(AnyValueEnum::StringValue(value.to_string())),
        }),
        key_strindex: 0,
    }
}

fn exemplar(value: ExemplarValue, ts: u64) -> Exemplar {
    Exemplar {
        time_unix_nano: ts,
        trace_id: vec![1; 16],
        span_id: vec![2; 8],
        value: Some(value),
        filtered_attributes: vec![string_kv("route", "/checkout")],
    }
}

fn logs_request() -> ExportLogsServiceRequest {
    ExportLogsServiceRequest {
        resource_logs: vec![parallax_proto::logs::ResourceLogs {
            resource: Some(parallax_proto::resource::Resource {
                attributes: vec![string_kv("service.name", "checkout")],
                ..Default::default()
            }),
            scope_logs: vec![parallax_proto::logs::ScopeLogs {
                log_records: vec![parallax_proto::logs::LogRecord {
                    time_unix_nano: 1_000_000_000,
                    observed_time_unix_nano: 5_000_000_000,
                    event_name: "checkout.completed".to_string(),
                    body: Some(AnyValue {
                        value: Some(AnyValueEnum::StringValue("done".to_string())),
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

#[test]
fn hex_encodes_known_bytes() {
    assert_eq!(hex(&[0x00, 0xff, 0x1a]), "00ff1a");
    assert_eq!(hex(&[]), "");
    assert_eq!(hex(&[0xab; 16]), "abababababababababababababababab");
}

#[test]
fn normalize_logs_carries_event_name_and_observed_timestamp() {
    let rows = normalize_logs(&logs_request());

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].ts_nanos, 1_000_000_000);
    assert_eq!(rows[0].event_name, "checkout.completed");
    assert_eq!(rows[0].observed_ts_nanos, 5_000_000_000);
}

#[test]
fn normalize_logs_defaults_unset_event_name_and_observed_timestamp() {
    let mut request = logs_request();
    let record = &mut request.resource_logs[0].scope_logs[0].log_records[0];
    record.event_name.clear();
    record.time_unix_nano = 0;
    record.observed_time_unix_nano = 0;

    let rows = normalize_logs(&request);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].ts_nanos, 0);
    assert_eq!(rows[0].event_name, "");
    assert_eq!(rows[0].observed_ts_nanos, 0);
}

#[test]
fn promote_log_identity_attributes_adds_native_greptime_keys() {
    let mut request = logs_request();

    assert!(promote_log_identity_attributes(&mut request));
    let record = &request.resource_logs[0].scope_logs[0].log_records[0];
    assert_eq!(
        attr_str(&record.attributes, semconv::EVENT_NAME),
        Some("checkout.completed")
    );
    let observed = record
        .attributes
        .iter()
        .find(|kv| kv.key == semconv::LOG_OBSERVED_TS_NANOS)
        .and_then(|kv| kv.value.as_ref())
        .and_then(|value| match &value.value {
            Some(AnyValueEnum::IntValue(value)) => Some(*value),
            _ => None,
        });
    assert_eq!(observed, Some(5_000_000_000));
    assert!(!promote_log_identity_attributes(&mut request));
}

#[test]
fn normalize_metrics_collects_number_and_histogram_exemplars() {
    let request = ExportMetricsServiceRequest {
        resource_metrics: vec![parallax_proto::metrics::ResourceMetrics {
            resource: Some(parallax_proto::resource::Resource {
                attributes: vec![
                    string_kv("service.name", "checkout"),
                    string_kv("cli.invocation.id", "run-a"),
                ],
                ..Default::default()
            }),
            scope_metrics: vec![parallax_proto::metrics::ScopeMetrics {
                metrics: vec![
                    Metric {
                        name: "process.cpu.utilization".into(),
                        data: Some(Data::Gauge(Gauge {
                            data_points: vec![NumberDataPoint {
                                time_unix_nano: 10,
                                value: Some(NumberValue::AsDouble(0.8)),
                                exemplars: vec![exemplar(ExemplarValue::AsDouble(0.9), 11)],
                                ..Default::default()
                            }],
                        })),
                        ..Default::default()
                    },
                    Metric {
                        name: "http.server.request.duration".into(),
                        data: Some(Data::Histogram(Histogram {
                            data_points: vec![HistogramDataPoint {
                                time_unix_nano: 20,
                                count: 1,
                                sum: Some(120.0),
                                bucket_counts: vec![0, 1],
                                explicit_bounds: vec![100.0],
                                exemplars: vec![exemplar(ExemplarValue::AsInt(120), 21)],
                                ..Default::default()
                            }],
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        }],
    };

    let normalized = normalize_metrics(&request);

    assert_eq!(normalized.points.len(), 1);
    assert_eq!(normalized.histograms.len(), 1);
    assert_eq!(normalized.exemplars.len(), 2);
    assert_eq!(normalized.exemplars[0].service, "checkout");
    assert_eq!(
        normalized.exemplars[0].invocation_id.as_deref(),
        Some("run-a")
    );
    assert_eq!(
        normalized.exemplars[0].trace_id,
        "01010101010101010101010101010101"
    );
    assert_eq!(normalized.exemplars[0].span_id, "0202020202020202");
    assert_eq!(normalized.exemplars[0].attributes["route"], "/checkout");
    assert_eq!(normalized.exemplars[1].name, "http.server.request.duration");
    assert_eq!(normalized.exemplars[1].value, 120.0);
}

fn trace_request(
    resource_attrs: Vec<KeyValue>,
    root_span_attrs: Vec<KeyValue>,
) -> ExportTraceServiceRequest {
    ExportTraceServiceRequest {
        resource_spans: vec![parallax_proto::trace::ResourceSpans {
            resource: Some(parallax_proto::resource::Resource {
                attributes: resource_attrs,
                ..Default::default()
            }),
            scope_spans: vec![parallax_proto::trace::ScopeSpans {
                spans: vec![parallax_proto::trace::Span {
                    trace_id: vec![0xab; 16],
                    span_id: vec![0xcd; 8],
                    name: "cli.command".into(),
                    start_time_unix_nano: 1_000,
                    end_time_unix_nano: 2_000,
                    attributes: root_span_attrs,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

fn log_request(
    resource_attrs: Vec<KeyValue>,
    log_attrs: Vec<KeyValue>,
) -> ExportLogsServiceRequest {
    ExportLogsServiceRequest {
        resource_logs: vec![parallax_proto::logs::ResourceLogs {
            resource: Some(parallax_proto::resource::Resource {
                attributes: resource_attrs,
                ..Default::default()
            }),
            scope_logs: vec![parallax_proto::logs::ScopeLogs {
                log_records: vec![parallax_proto::logs::LogRecord {
                    time_unix_nano: 1_000_000_000,
                    attributes: log_attrs,
                    body: Some(AnyValue {
                        value: Some(AnyValueEnum::StringValue("hello".into())),
                    }),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

#[test]
fn normalize_traces_prefers_root_span_cli_invocation_id_over_resource() {
    let request = trace_request(
        vec![
            string_kv("service.name", "checkout"),
            string_kv(semconv::CLI_INVOCATION_ID, "from-resource"),
        ],
        vec![string_kv(semconv::CLI_INVOCATION_ID, "from-span")],
    );
    let rows = normalize_traces(&request);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].invocation_id.as_deref(), Some("from-span"));
}

#[test]
fn normalize_traces_accepts_resource_only_cli_invocation_id() {
    let request = trace_request(
        vec![
            string_kv("service.name", "checkout"),
            string_kv(semconv::CLI_INVOCATION_ID, "from-resource"),
        ],
        vec![],
    );
    let rows = normalize_traces(&request);
    assert_eq!(rows[0].invocation_id.as_deref(), Some("from-resource"));
}

#[test]
fn normalize_traces_ignores_legacy_parallax_run_id() {
    // Operator 2026-07-17: parallax.run.id is never read.
    let request = trace_request(
        vec![
            string_kv("service.name", "checkout"),
            string_kv("parallax.run.id", "legacy-only"),
        ],
        vec![string_kv("parallax.run.id", "legacy-span")],
    );
    let rows = normalize_traces(&request);
    assert_eq!(rows[0].invocation_id, None);
}

#[test]
fn normalize_logs_resolves_session_id_signal_then_resource() {
    let signal_wins = log_request(
        vec![
            string_kv("service.name", "checkout"),
            string_kv(semconv::SESSION_ID, "sess-resource"),
            string_kv(semconv::CLI_INVOCATION_ID, "inv-resource"),
        ],
        vec![
            string_kv(semconv::SESSION_ID, "sess-log"),
            string_kv(semconv::CLI_INVOCATION_ID, "inv-log"),
        ],
    );
    let rows = normalize_logs(&signal_wins);
    assert_eq!(rows[0].session_id.as_deref(), Some("sess-log"));
    assert_eq!(rows[0].invocation_id.as_deref(), Some("inv-log"));

    let resource_only = log_request(
        vec![
            string_kv("service.name", "checkout"),
            string_kv(semconv::SESSION_ID, "sess-resource"),
        ],
        vec![],
    );
    let rows = normalize_logs(&resource_only);
    assert_eq!(rows[0].session_id.as_deref(), Some("sess-resource"));
    assert_eq!(rows[0].invocation_id, None);
}

fn trace_request_with_child(
    resource_attrs: Vec<KeyValue>,
    root_attrs: Vec<KeyValue>,
) -> ExportTraceServiceRequest {
    ExportTraceServiceRequest {
        resource_spans: vec![parallax_proto::trace::ResourceSpans {
            resource: Some(parallax_proto::resource::Resource {
                attributes: resource_attrs,
                ..Default::default()
            }),
            scope_spans: vec![parallax_proto::trace::ScopeSpans {
                spans: vec![
                    parallax_proto::trace::Span {
                        trace_id: vec![1; 16],
                        span_id: vec![2; 8],
                        name: "root".to_string(),
                        start_time_unix_nano: 10,
                        end_time_unix_nano: 20,
                        attributes: root_attrs,
                        ..Default::default()
                    },
                    parallax_proto::trace::Span {
                        trace_id: vec![1; 16],
                        span_id: vec![3; 8],
                        parent_span_id: vec![2; 8],
                        name: "child".to_string(),
                        start_time_unix_nano: 12,
                        end_time_unix_nano: 18,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

#[test]
fn root_span_attribute_wins_over_resource_attribute() {
    let rows = normalize_traces(&trace_request_with_child(
        vec![
            string_kv("service.name", "cli"),
            string_kv("cli.invocation.id", "inv-res"),
        ],
        vec![string_kv("cli.invocation.id", "inv-span")],
    ));
    for row in &rows {
        assert_eq!(row.invocation_id.as_deref(), Some("inv-span"));
    }
}

#[test]
fn resource_invocation_ids_reads_root_span_attribute() {
    let request = trace_request_with_child(
        vec![string_kv("service.name", "cli")],
        vec![string_kv("cli.invocation.id", "inv-root")],
    );
    let ids: Vec<(String, u128)> = resource_invocation_ids(&request).collect();
    assert_eq!(ids, vec![("inv-root".to_string(), 10)]);
}

/// Plan-103: OTLP normalize is a pure function of the export request.
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    fn span_request(
        service: &str,
        span_name: &str,
        start: u64,
        end: u64,
        status_code: i32,
    ) -> ExportTraceServiceRequest {
        use parallax_proto::trace::{ResourceSpans, ScopeSpans, Span, Status};
        ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: Some(parallax_proto::resource::Resource {
                    attributes: vec![string_kv("service.name", service)],
                    ..Default::default()
                }),
                scope_spans: vec![ScopeSpans {
                    spans: vec![Span {
                        trace_id: vec![0xab; 16],
                        span_id: vec![0xcd; 8],
                        name: span_name.to_string(),
                        start_time_unix_nano: start,
                        end_time_unix_nano: end.max(start),
                        status: Some(Status {
                            code: status_code,
                            message: String::new(),
                        }),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    proptest! {
        /// Span-count conservation: one OTLP span in, one SpanRow out.
        #[test]
        fn normalize_traces_conserves_span_count(
            service in "[a-zA-Z0-9._-]{1,32}",
            name in "[a-zA-Z0-9._/ -]{0,64}",
            start in 0u64..1_000_000_000_000u64,
            duration in 0u64..1_000_000_000u64,
            status in 0i32..=2,
        ) {
            let request = span_request(&service, &name, start, start.saturating_add(duration), status);
            let rows = normalize_traces(&request);
            prop_assert_eq!(rows.len(), 1);
            prop_assert_eq!(&rows[0].service, &service);
            prop_assert_eq!(&rows[0].name, &name);
            let keys: Vec<&String> = rows[0]
                .attributes
                .as_object()
                .map(|map| map.keys().collect())
                .unwrap_or_default();
            prop_assert!(keys.iter().all(|key| !key.is_empty()));
        }
    }

    proptest! {
        /// Log-record conservation: one OTLP record in, one LogRow out.
        #[test]
        fn normalize_logs_conserves_record_count(
            service in "[a-zA-Z0-9._-]{1,32}",
            body in ".*{0,128}",
            severity in 1i32..=24,
        ) {
            let request = ExportLogsServiceRequest {
                resource_logs: vec![parallax_proto::logs::ResourceLogs {
                    resource: Some(parallax_proto::resource::Resource {
                        attributes: vec![string_kv("service.name", &service)],
                        ..Default::default()
                    }),
                    scope_logs: vec![parallax_proto::logs::ScopeLogs {
                        log_records: vec![parallax_proto::logs::LogRecord {
                            time_unix_nano: 1_000_000_000,
                            severity_number: severity,
                            body: Some(AnyValue {
                                value: Some(AnyValueEnum::StringValue(body.clone())),
                            }),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            };
            let rows = normalize_logs(&request);
            prop_assert_eq!(rows.len(), 1);
            prop_assert_eq!(&rows[0].service, &service);
            prop_assert_eq!(&rows[0].body, &body);
            let keys: Vec<&String> = rows[0]
                .attributes
                .as_object()
                .map(|map| map.keys().collect())
                .unwrap_or_default();
            prop_assert!(keys.iter().all(|key| !key.is_empty()));
        }
    }
}

fn metric_request(metric: Metric) -> ExportMetricsServiceRequest {
    ExportMetricsServiceRequest {
        resource_metrics: vec![parallax_proto::metrics::ResourceMetrics {
            resource: Some(parallax_proto::resource::Resource {
                attributes: vec![string_kv("service.name", "checkout")],
                ..Default::default()
            }),
            scope_metrics: vec![parallax_proto::metrics::ScopeMetrics {
                metrics: vec![metric],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

#[test]
fn exponential_histogram_is_dropped_today() {
    // Plan 166 owns whether this drop stays. This test pins current behavior.
    let request = metric_request(Metric {
        name: "exp.hist".into(),
        data: Some(Data::ExponentialHistogram(ExponentialHistogram::default())),
        ..Default::default()
    });
    let normalized = normalize_metrics(&request);
    assert!(normalized.points.is_empty());
    assert!(normalized.histograms.is_empty());
    assert!(normalized.exemplars.is_empty());
}

#[test]
fn summary_is_dropped_today() {
    let request = metric_request(Metric {
        name: "summary".into(),
        data: Some(Data::Summary(Summary::default())),
        ..Default::default()
    });
    let normalized = normalize_metrics(&request);
    assert!(normalized.points.is_empty());
    assert!(normalized.histograms.is_empty());
}

#[test]
fn metric_with_no_data_is_empty() {
    let request = metric_request(Metric {
        name: "empty".into(),
        data: None,
        ..Default::default()
    });
    let normalized = normalize_metrics(&request);
    assert!(normalized.points.is_empty());
    assert!(normalized.histograms.is_empty());
}

#[test]
fn number_point_absent_value_becomes_zero() {
    // Plan 166 decision: None => 0.0 fabricates a zero sample.
    let request = metric_request(Metric {
        name: "gauge.none".into(),
        data: Some(Data::Gauge(Gauge {
            data_points: vec![NumberDataPoint {
                time_unix_nano: 1,
                value: None,
                ..Default::default()
            }],
        })),
        ..Default::default()
    });
    let normalized = normalize_metrics(&request);
    assert_eq!(normalized.points.len(), 1);
    assert_eq!(normalized.points[0].value, 0.0);
}

#[test]
fn exemplar_missing_value_or_ids_is_skipped() {
    let mut missing_value = exemplar(ExemplarValue::AsDouble(1.0), 11);
    missing_value.value = None;
    let mut missing_ids = exemplar(ExemplarValue::AsDouble(1.0), 11);
    missing_ids.trace_id.clear();
    missing_ids.span_id.clear();
    let request = metric_request(Metric {
        name: "gauge".into(),
        data: Some(Data::Gauge(Gauge {
            data_points: vec![NumberDataPoint {
                time_unix_nano: 10,
                value: Some(NumberValue::AsDouble(1.0)),
                exemplars: vec![missing_value, missing_ids],
                ..Default::default()
            }],
        })),
        ..Default::default()
    });
    let normalized = normalize_metrics(&request);
    assert_eq!(normalized.points.len(), 1);
    assert!(normalized.exemplars.is_empty());
}

#[test]
fn gauge_plus_histogram_conserves_point_count() {
    let request = ExportMetricsServiceRequest {
        resource_metrics: vec![parallax_proto::metrics::ResourceMetrics {
            resource: Some(parallax_proto::resource::Resource {
                attributes: vec![string_kv("service.name", "checkout")],
                ..Default::default()
            }),
            scope_metrics: vec![parallax_proto::metrics::ScopeMetrics {
                metrics: vec![
                    Metric {
                        name: "g".into(),
                        data: Some(Data::Gauge(Gauge {
                            data_points: vec![
                                NumberDataPoint {
                                    time_unix_nano: 1,
                                    value: Some(NumberValue::AsDouble(1.0)),
                                    ..Default::default()
                                },
                                NumberDataPoint {
                                    time_unix_nano: 2,
                                    value: Some(NumberValue::AsDouble(2.0)),
                                    ..Default::default()
                                },
                            ],
                        })),
                        ..Default::default()
                    },
                    Metric {
                        name: "h".into(),
                        data: Some(Data::Histogram(Histogram {
                            data_points: vec![HistogramDataPoint {
                                time_unix_nano: 3,
                                count: 1,
                                ..Default::default()
                            }],
                            ..Default::default()
                        })),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        }],
    };
    let normalized = normalize_metrics(&request);
    assert_eq!(normalized.points.len() + normalized.histograms.len(), 3);
}
