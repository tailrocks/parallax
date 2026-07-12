#[tokio::test]
async fn span_field_keys_and_stats_cover_span_and_resource_attrs() {
    let store = MemoryStore::new();
    let mut s1 = span_with_attrs(
        "trace-1",
        "root",
        10,
        serde_json::json!({
            "http.request.method": "GET",
            "request.id": "req-1"
        }),
    );
    s1.resource = serde_json::json!({ "service.name": "checkout" });
    let mut s2 = span_with_attrs(
        "trace-2",
        "root",
        20,
        serde_json::json!({
            "http.request.method": "GET",
            "request.id": "req-2"
        }),
    );
    s2.resource = serde_json::json!({ "service.name": "checkout" });
    let mut s3 = span_with_attrs(
        "trace-3",
        "root",
        30,
        serde_json::json!({
            "http.request.method": "POST",
            "request.id": "req-3"
        }),
    );
    s3.resource = serde_json::json!({ "service.name": "checkout" });
    let mut s4 = span("trace-4", "root", None, "checkout", 40);
    s4.resource = serde_json::json!({ "service.name": "checkout" });
    store.push_spans(vec![s1, s2, s3, s4]);

    let keys = store.span_field_keys(0..=100).await.unwrap();
    let method_key = keys
        .iter()
        .find(|key| key.key == "http.request.method")
        .unwrap();
    assert_eq!(method_key.namespace, "http");
    assert_eq!(method_key.non_null_count, 3);
    assert!((method_key.coverage - 0.75).abs() < f64::EPSILON);
    assert!(
        keys.iter()
            .any(|key| key.key == "resource.service.name" && key.source == FieldSource::Resource)
    );
    assert!(
        keys.iter()
            .any(|key| key.key == "request.id" && key.is_identifier)
    );

    let stats = store
        .span_field_stats("http.request.method", 0..=100, Some("checkout"))
        .await
        .unwrap();
    assert_eq!(stats.row_count, 4);
    assert_eq!(stats.non_null_count, 3);
    assert_eq!(stats.distinct_count, 2);
    assert_eq!(stats.top_values[0].value, "GET");
    assert_eq!(stats.top_values[0].count, 2);
}

#[tokio::test]
async fn span_field_stats_rejects_disallowed_keys() {
    let store = MemoryStore::new();
    store.push_spans(vec![span_with_attrs(
        "trace-1",
        "root",
        10,
        serde_json::json!({ "authorization": "secret" }),
    )]);

    assert!(!span_field_key_allowed("authorization"));
    let err = store
        .span_field_stats("authorization", 0..=100, None)
        .await
        .unwrap_err();
    assert!(err.to_string().contains("invalid field key"));
}

#[tokio::test]
async fn service_catalog_returns_identity_and_nulls() {
    let store = MemoryStore::new();
    store.push_spans(vec![
        span_with_resource(
            "checkout-old",
            "root",
            "checkout",
            10,
            serde_json::json!({
                "service.version": "v1",
                "service.namespace": "shop",
                "deployment.environment.name": "staging",
                "telemetry.sdk.language": "rust",
                "telemetry.sdk.name": "opentelemetry",
                "telemetry.sdk.version": "0.31.0",
                "service.instance.id": "checkout-a"
            }),
        ),
        span_with_resource(
            "checkout-new",
            "root",
            "checkout",
            20,
            serde_json::json!({
                "service.version": "v2",
                "service.namespace": "shop",
                "deployment.environment.name": "prod",
                "telemetry.sdk.language": "rust",
                "telemetry.sdk.name": "opentelemetry",
                "telemetry.sdk.version": "0.32.1",
                "service.instance.id": "checkout-b"
            }),
        ),
        span("bare", "root", None, "bare", 30),
    ]);

    let rows = store.service_catalog(0..=100).await.unwrap();

    let bare = rows.iter().find(|row| row.name == "bare").unwrap();
    assert_eq!(bare.service_version, None);
    assert_eq!(bare.telemetry_sdk_language, None);
    assert_eq!(bare.instance_count, 0);
    let checkout = rows.iter().find(|row| row.name == "checkout").unwrap();
    assert_eq!(checkout.service_version.as_deref(), Some("v2"));
    assert_eq!(checkout.service_namespace.as_deref(), Some("shop"));
    assert_eq!(checkout.deployment_environment.as_deref(), Some("prod"));
    assert_eq!(checkout.telemetry_sdk_language.as_deref(), Some("rust"));
    assert_eq!(
        checkout.telemetry_sdk_name.as_deref(),
        Some("opentelemetry")
    );
    assert_eq!(checkout.telemetry_sdk_version.as_deref(), Some("0.32.1"));
    assert_eq!(checkout.last_seen_nanos, 20);
    assert_eq!(checkout.instance_count, 2);
}

#[tokio::test]
async fn release_windows_group_versions_by_service_and_range() {
    let store = MemoryStore::new();
    store.push_spans(vec![
        span_with_release("t1", "a", 10, "v1"),
        span_with_release("t2", "a", 20, "v1"),
        span_with_release("t3", "a", 40, "v2"),
        span_with_release("t4", "a", 60, "v2"),
        span("other", "a", None, "catalog", 30),
        span_with_release("too-late", "a", 90, "v3"),
    ]);

    let windows = store.release_windows("checkout", 0..=80).await.unwrap();

    assert_eq!(windows.len(), 2);
    assert_eq!(windows[0].version, "v1");
    assert_eq!(windows[0].first_seen_nanos, 10);
    assert_eq!(windows[0].last_seen_nanos, 20);
    assert_eq!(windows[0].span_count, 2);
    assert_eq!(windows[1].version, "v2");
    assert_eq!(windows[1].first_seen_nanos, 40);
    assert_eq!(windows[1].last_seen_nanos, 60);
    assert_eq!(windows[1].span_count, 2);
}

#[tokio::test]
async fn attribute_compare_ranks_overrepresented_value_first() {
    let store = MemoryStore::new();
    let mut spans = Vec::new();
    for index in 0..20 {
        let version = if index == 0 { "2.0.0" } else { "1.0.0" };
        spans.push(span_with_attrs(
            &format!("baseline-{index}"),
            "root",
            index,
            serde_json::json!({
                "service.version": version,
                "http.route": "/checkout"
            }),
        ));
    }
    for index in 0..10 {
        let version = if index < 9 { "2.0.0" } else { "1.0.0" };
        spans.push(span_with_attrs(
            &format!("selected-{index}"),
            "root",
            100 + index,
            serde_json::json!({
                "service.version": version,
                "http.route": "/checkout"
            }),
        ));
    }
    store.push_spans(spans);

    let rows = store
        .attribute_compare(100..=200, 0..=99, Some("checkout"), false, &[], 10)
        .await
        .unwrap();

    let first = rows.first().expect("overrepresented value");
    assert_eq!(first.key, "service.version");
    assert_eq!(first.value, "2.0.0");
    assert_eq!(first.selected_count, 9);
    assert_eq!(first.selected_total, 10);
    assert_eq!(first.baseline_count, 1);
    assert_eq!(first.baseline_total, 20);
    assert!(first.score > 0.8, "{first:?}");
}

#[tokio::test]
async fn metric_exemplars_filters_by_metric_service_range_and_limit() {
    let store = MemoryStore::new();
    store
        .ingest_metrics(
            Vec::new(),
            Vec::new(),
            vec![
                MetricExemplarRow {
                    ts_nanos: 20,
                    service: "checkout".into(),
                    name: "http.server.request.duration".into(),
                    value: 120.0,
                    trace_id: "trace-a".into(),
                    span_id: "span-a".into(),
                    run_id: Some("run-a".into()),
                    attributes: serde_json::json!({"route": "/checkout"}),
                },
                MetricExemplarRow {
                    ts_nanos: 10,
                    service: "checkout".into(),
                    name: "http.server.request.duration".into(),
                    value: 90.0,
                    trace_id: "trace-b".into(),
                    span_id: "span-b".into(),
                    run_id: None,
                    attributes: serde_json::Value::Null,
                },
                MetricExemplarRow {
                    ts_nanos: 30,
                    service: "catalog".into(),
                    name: "http.server.request.duration".into(),
                    value: 80.0,
                    trace_id: "trace-c".into(),
                    span_id: "span-c".into(),
                    run_id: None,
                    attributes: serde_json::Value::Null,
                },
            ],
            bytes::Bytes::new(),
        )
        .await
        .unwrap();

    let rows = store
        .metric_exemplars("http.server.request.duration", Some("checkout"), 0..=25, 1)
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].trace_id, "trace-a");
    assert_eq!(rows[0].run_id.as_deref(), Some("run-a"));
    assert_eq!(rows[0].attributes["route"], "/checkout");
}

#[tokio::test]
async fn metric_labels_values_and_runtime_snapshot_derive_from_points() {
    let store = MemoryStore::new();
    store
        .ingest_metrics(
            vec![
                MetricPointRow {
                    ts_nanos: 1_000_000_000,
                    service: "checkout".into(),
                    name: "process.cpu.utilization".into(),
                    value: 0.42,
                    is_monotonic: false,
                    run_id: Some("run-a".into()),
                    attributes: serde_json::json!({
                        "runtime.name": "tokio",
                        "payment.method": "card",
                        "trace_id": "trace-a"
                    }),
                },
                MetricPointRow {
                    ts_nanos: 2_000_000_000,
                    service: "checkout".into(),
                    name: "jvm.gc.time".into(),
                    value: 12.0,
                    is_monotonic: false,
                    run_id: None,
                    attributes: serde_json::json!({
                        "payment.method": "wire"
                    }),
                },
            ],
            Vec::new(),
            Vec::new(),
            bytes::Bytes::new(),
        )
        .await
        .unwrap();

    let labels = store
        .metric_labels("process.cpu.utilization")
        .await
        .unwrap();
    assert!(labels.contains(&"runtime.name".to_string()));
    assert!(labels.contains(&"payment.method".to_string()));
    assert!(!labels.contains(&"trace_id".to_string()));

    let values = store
        .metric_label_values(
            "process.cpu.utilization",
            "payment.method",
            0..=3_000_000_000,
        )
        .await
        .unwrap();
    assert_eq!(values, vec!["card".to_string()]);

    let mut capped_points = Vec::new();
    for index in 0..110 {
        capped_points.push(MetricPointRow {
            ts_nanos: 4_000_000_000 + index,
            service: "checkout".into(),
            name: "process.cpu.utilization".into(),
            value: index as f64,
            is_monotonic: false,
            run_id: None,
            attributes: serde_json::json!({
                "runtime.name": format!("runtime-{index:03}")
            }),
        });
    }
    store
        .ingest_metrics(capped_points, Vec::new(), Vec::new(), bytes::Bytes::new())
        .await
        .unwrap();

    let capped = store
        .metric_label_values("process.cpu.utilization", "runtime.name", 0..=5_000_000_000)
        .await
        .unwrap();
    assert_eq!(capped.len(), 100);

    let runtime = store
        .runtime_snapshot(Some("checkout"), None, 0..=3_000_000_000, 1_000_000_000)
        .await
        .unwrap();
    assert_eq!(runtime.len(), 2);
    assert!(runtime.iter().any(|row| row.family == "process"));
    assert!(runtime.iter().any(|row| row.family == "jvm"));

    let run_runtime = store
        .runtime_snapshot(None, Some("run-a"), 0..=3_000_000_000, 1_000_000_000)
        .await
        .unwrap();
    assert_eq!(run_runtime.len(), 1);
    assert_eq!(run_runtime[0].metric, "process.cpu.utilization");

    let denied = store
        .metric_series_grouped(
            "process.cpu.utilization",
            Some("checkout"),
            "trace_id",
            0..=3_000_000_000,
            1_000_000_000,
            MetricAgg::Avg,
        )
        .await
        .unwrap_err()
        .to_string();
    assert!(denied.contains("high-cardinality identifier"));
}

#[tokio::test]
async fn attribute_compare_denies_identifier_keys() {
    let store = MemoryStore::new();
    store.push_spans(vec![
        span_with_attrs(
            "baseline",
            "root",
            1,
            serde_json::json!({
                "service.version": "1.0.0",
                "trace_id": "trace-baseline",
                "run_id": "run-baseline",
                "session.id": "session-baseline",
                "user.id": "user-baseline"
            }),
        ),
        span_with_attrs(
            "selected",
            "root",
            100,
            serde_json::json!({
                "service.version": "2.0.0",
                "trace_id": "trace-selected",
                "run_id": "run-selected",
                "session.id": "session-selected",
                "user.id": "user-selected"
            }),
        ),
    ]);
    let keys = vec![
        "trace_id".to_string(),
        "run_id".to_string(),
        "session.id".to_string(),
        "user.id".to_string(),
        "service.version".to_string(),
    ];

    let rows = store
        .attribute_compare(100..=200, 0..=99, None, false, &keys, 10)
        .await
        .unwrap();

    assert!(rows.iter().all(|row| {
        !matches!(
            row.key.as_str(),
            "trace_id" | "run_id" | "session.id" | "user.id"
        )
    }));
    assert!(rows.iter().any(|row| row.key == "service.version"));
}

#[tokio::test]
async fn attribute_compare_is_deterministic() {
    let store = MemoryStore::new();
    store.push_spans(vec![
        span_with_attrs(
            "baseline-a",
            "root",
            1,
            serde_json::json!({"service.version": "1.0.0"}),
        ),
        span_with_attrs(
            "selected-a",
            "root",
            100,
            serde_json::json!({"service.version": "2.0.0"}),
        ),
    ]);

    let first = store
        .attribute_compare(100..=200, 0..=99, None, false, &[], 10)
        .await
        .unwrap();
    let second = store
        .attribute_compare(100..=200, 0..=99, None, false, &[], 10)
        .await
        .unwrap();

    assert_eq!(first, second);
}
