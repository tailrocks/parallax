use super::*;
use parallax_proto::common::any_value::Value as AnyValueEnum;
use parallax_proto::common::{AnyValue, KeyValue};
use parallax_proto::metrics::{
    Exemplar, Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics,
    exemplar::Value as ExemplarValue, metric::Data, number_data_point::Value as NumberValue,
};
use parallax_proto::resource::Resource;
use parallax_proto::trace::{ResourceSpans, ScopeSpans, Span, Status, span, status};
use parallax_test_support::MemoryStore;
use serde_json::json;
use tokio::sync::oneshot;

fn error_event(source: ErrorSource, span_id: &str, fingerprint: &str) -> ErrorEventRow {
    ErrorEventRow {
        ts_nanos: 1,
        service: "checkout".to_string(),
        fingerprint: fingerprint.to_string(),
        error_type: "test::Boom".to_string(),
        message: "boom".to_string(),
        stacktrace: Some("top\nbottom".to_string()),
        source,
        trace_id: "trace".to_string(),
        span_id: span_id.to_string(),
        attributes: json!({}),
    }
}

fn string_kv(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(AnyValueEnum::StringValue(value.to_string())),
        }),
        key_strindex: 0,
    }
}

fn metrics_request_with_exemplar() -> ExportMetricsServiceRequest {
    ExportMetricsServiceRequest {
        resource_metrics: vec![ResourceMetrics {
            resource: Some(Resource {
                attributes: vec![
                    string_kv("service.name", "checkout"),
                    string_kv("parallax.run.id", "run-a"),
                ],
                ..Default::default()
            }),
            scope_metrics: vec![ScopeMetrics {
                metrics: vec![Metric {
                    name: "http.server.request.duration".into(),
                    data: Some(Data::Gauge(Gauge {
                        data_points: vec![NumberDataPoint {
                            time_unix_nano: 20,
                            value: Some(NumberValue::AsDouble(100.0)),
                            exemplars: vec![Exemplar {
                                time_unix_nano: 21,
                                trace_id: vec![0xab; 16],
                                span_id: vec![0xcd; 8],
                                value: Some(ExemplarValue::AsDouble(120.0)),
                                filtered_attributes: vec![string_kv("route", "/checkout")],
                            }],
                            ..Default::default()
                        }],
                    })),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

fn trace_request_with_run_and_error() -> ExportTraceServiceRequest {
    ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(Resource {
                attributes: vec![
                    string_kv("service.name", "checkout"),
                    string_kv("parallax.run.id", "run-failure-oracle"),
                ],
                ..Default::default()
            }),
            scope_spans: vec![ScopeSpans {
                spans: vec![Span {
                    trace_id: vec![1; 16],
                    span_id: vec![2; 8],
                    name: "checkout.authorize".to_string(),
                    start_time_unix_nano: 10,
                    end_time_unix_nano: 99,
                    status: Some(Status {
                        code: status::StatusCode::Error as i32,
                        message: "status failed".to_string(),
                    }),
                    events: vec![span::Event {
                        time_unix_nano: 42,
                        name: "exception".to_string(),
                        attributes: vec![
                            string_kv("exception.type", "test::Boom"),
                            string_kv("exception.message", "boom"),
                            string_kv("exception.stacktrace", "top\nbottom"),
                        ],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }],
    }
}

async fn characterize_failure_after(stage: FailureStage) -> (usize, usize, u64, usize, usize) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let store = Arc::new(MemoryStore::new().with_normalizers(
        Arc::new(normalize::normalize_traces),
        Arc::new(normalize::normalize_logs),
    ));
    let metadata = Arc::new(
        MetadataStore::open(tmp.path().join("meta.db"))
            .await
            .expect("metadata"),
    );
    let live = crate::live::channels();
    let mut live_spans = live.spans.subscribe();
    let worker = Worker::new(store.clone(), metadata.clone(), live);
    worker.inject_failure_once_after(stage).await;
    let item = IngestItem::Traces(trace_request_with_run_and_error(), bytes::Bytes::new());
    worker
        .process(&item)
        .await
        .expect_err("first attempt fails");
    worker.process(&item).await.expect("retry succeeds");

    let mut broadcasts = 0;
    while live_spans.try_recv().is_ok() {
        broadcasts += 1;
    }
    let spans = store
        .spans_by_trace("01010101010101010101010101010101")
        .await
        .expect("spans")
        .len();
    let issues = metadata.issues(10).await.expect("issues");
    let issue_count = issues.first().map_or(0, |issue| issue.event_count);
    let errors = if let Some(issue) = issues.first() {
        store
            .error_events_by_fingerprint(&issue.fingerprint, 0..=u128::MAX, 10)
            .await
            .expect("error events")
            .len()
    } else {
        0
    };
    let runs = metadata.runs(10).await.expect("runs").len();
    (broadcasts, spans, issue_count, errors, runs)
}

#[test]
fn dedup_prefers_span_exception_for_same_failure() {
    let events = dedup_error_events(vec![
        error_event(ErrorSource::LogException, "span-a", "fp"),
        error_event(ErrorSource::SpanException, "span-a", "fp"),
    ]);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].source, ErrorSource::SpanException);
}

#[test]
fn dedup_preserves_distinct_span_failures() {
    let events = dedup_error_events(vec![
        error_event(ErrorSource::SpanException, "span-a", "fp"),
        error_event(ErrorSource::SpanException, "span-b", "fp"),
    ]);
    assert_eq!(events.len(), 2);
}

#[tokio::test]
async fn record_errors_counts_one_occurrence_after_dedup() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let store = Arc::new(MemoryStore::new());
    let metadata = Arc::new(
        MetadataStore::open(tmp.path().join("meta.db"))
            .await
            .expect("metadata"),
    );
    let worker = Worker::new(store.clone(), metadata.clone(), crate::live::channels());
    worker
        .record_errors(vec![
            error_event(ErrorSource::LogException, "span-a", "fp"),
            error_event(ErrorSource::SpanException, "span-a", "fp"),
        ])
        .await
        .expect("record errors");
    let issues = metadata.issues(10).await.expect("issues");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].event_count, 1);
    let events = store
        .error_events_by_fingerprint("fp", 0..=u128::MAX, 10)
        .await
        .expect("error events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].source, ErrorSource::SpanException);
}

#[tokio::test]
async fn process_is_reentrant_after_failure_shape() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let store = Arc::new(MemoryStore::new());
    let metadata = Arc::new(
        MetadataStore::open(tmp.path().join("meta.db"))
            .await
            .expect("metadata"),
    );
    let worker = Worker::new(store.clone(), metadata, crate::live::channels());
    let item = IngestItem::Metrics(metrics_request_with_exemplar(), bytes::Bytes::new());
    worker.process(&item).await.expect("first");
    worker.process(&item).await.expect("second");
    let rows = store
        .metric_exemplars(
            "http.server.request.duration",
            Some("checkout"),
            0..=100,
            10,
        )
        .await
        .expect("metric exemplars");
    assert!(!rows.is_empty());
}

#[test]
fn ingest_retry_constants_are_bounded() {
    assert_eq!(INGEST_RETRIES, 3);
    assert_eq!(INGEST_BACKOFF.len(), 3);
    assert!(INGEST_BACKOFF[0] < INGEST_BACKOFF[1]);
    assert!(INGEST_BACKOFF[1] < INGEST_BACKOFF[2]);
}

#[tokio::test]
async fn failure_stage_replay_behavior_is_characterized() {
    // Tuple: live broadcasts, stored spans, issue occurrences,
    // stored error rows, registered runs.
    assert_eq!(
        characterize_failure_after(FailureStage::Registration).await,
        (1, 1, 1, 1, 1),
        "registration succeeds before failure; its seen-run cache prevents a duplicate"
    );
    assert_eq!(
        characterize_failure_after(FailureStage::Broadcast).await,
        (2, 1, 1, 1, 1),
        "broadcast repeats because it has no idempotency boundary"
    );
    assert_eq!(
        characterize_failure_after(FailureStage::TelemetryStorage).await,
        (2, 2, 1, 1, 1),
        "telemetry and earlier broadcast repeat after a late storage failure"
    );
    assert_eq!(
        characterize_failure_after(FailureStage::IssueRecording).await,
        (2, 2, 2, 2, 1),
        "all completed effects replay after the final stage fails"
    );
}

#[tokio::test]
async fn metric_exemplar_round_trips_through_worker_and_store() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let store = Arc::new(MemoryStore::new());
    let metadata = Arc::new(
        MetadataStore::open(tmp.path().join("meta.db"))
            .await
            .expect("metadata"),
    );
    let worker = Worker::new(store.clone(), metadata, crate::live::channels());
    worker
        .process(&IngestItem::Metrics(
            metrics_request_with_exemplar(),
            bytes::Bytes::new(),
        ))
        .await
        .expect("process metrics");
    let rows = store
        .metric_exemplars(
            "http.server.request.duration",
            Some("checkout"),
            0..=100,
            10,
        )
        .await
        .expect("metric exemplars");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].value, 120.0);
    assert_eq!(rows[0].trace_id, "abababababababababababababababab");
    assert_eq!(rows[0].span_id, "cdcdcdcdcdcdcdcd");
    assert_eq!(rows[0].run_id.as_deref(), Some("run-a"));
    assert_eq!(rows[0].attributes["route"], "/checkout");
}

#[tokio::test]
async fn per_signal_workers_isolate_slow_traces_from_logs() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (gate_tx, gate_rx) = oneshot::channel();
    let store = Arc::new(MemoryStore::new());
    store.set_traces_gate(gate_rx).await;
    let (logs_done_tx, logs_done_rx) = oneshot::channel();
    let metadata = Arc::new(
        MetadataStore::open(tmp.path().join("meta.db"))
            .await
            .expect("metadata"),
    );
    let live = crate::live::channels();
    let worker = Worker::new(store.clone(), metadata, live);
    let (senders, receivers) = channels(8);

    let traces_task = tokio::spawn(worker.clone().run(receivers.traces));
    let worker_logs = worker.clone();
    let logs_task = tokio::spawn(async move {
        let mut rx = receivers.logs;
        let mut logs_done_tx = Some(logs_done_tx);
        while let Some(item) = rx.recv().await {
            worker_logs.process(&item).await.expect("logs");
            if let Some(done) = logs_done_tx.take() {
                done.send(()).expect("signal logs completion");
            }
        }
    });
    let metrics_task = tokio::spawn(worker.run(receivers.metrics));

    senders
        .traces
        .send(IngestItem::Traces(
            ExportTraceServiceRequest::default(),
            bytes::Bytes::new(),
        ))
        .await
        .expect("enqueue traces");
    senders
        .logs
        .send(IngestItem::Logs(
            ExportLogsServiceRequest::default(),
            bytes::Bytes::new(),
        ))
        .await
        .expect("enqueue logs");

    tokio::time::timeout(Duration::from_secs(2), logs_done_rx)
        .await
        .expect("logs worker must not wait for a blocked traces forward")
        .expect("logs worker must report completion");

    gate_tx.send(()).expect("release traces gate");
    drop(senders);
    traces_task.await.expect("traces worker join");
    logs_task.await.expect("logs worker join");
    metrics_task.await.expect("metrics worker join");
}
