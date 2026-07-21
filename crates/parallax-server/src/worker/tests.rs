use super::*;
use parallax_metadata::TursoMetadataStore as MetadataStore;
use parallax_proto::common::any_value::Value as AnyValueEnum;
use parallax_proto::common::{AnyValue, KeyValue};
use parallax_proto::metrics::{
    Exemplar, Gauge, Metric, NumberDataPoint, ResourceMetrics, ScopeMetrics,
    exemplar::Value as ExemplarValue, metric::Data, number_data_point::Value as NumberValue,
};
use parallax_proto::resource::Resource;
use parallax_proto::trace::{ResourceSpans, ScopeSpans, Span, Status, span, status};
use parallax_storage::adapter::{InvocationStore, MetricAnalyticsStore, TraceStore};
use parallax_test_support::builders::MemoryStore;
use serde_json::json;
use tokio::sync::oneshot;

use crate::ingest_health::QueueSnapshot;

fn queued(item: IngestItem) -> QueuedItem {
    QueuedItem::fixture(item)
}

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
                    string_kv("cli.invocation.id", "run-a"),
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
                    string_kv("cli.invocation.id", "run-failure-oracle"),
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

fn trace_request_with_test_result() -> ExportTraceServiceRequest {
    let mut request = trace_request_with_run_and_error();
    let span = &mut request.resource_spans[0].scope_spans[0].spans[0];
    span.attributes = vec![
        string_kv("test.case.id", "checkout-authorize"),
        string_kv("test.case.name", "authorizes card"),
        string_kv("test.suite.name", "checkout"),
        string_kv("test.case.result.status", "fail"),
        string_kv("test.case.failure.kind", "assertion_failure"),
    ];
    span.parent_span_id = vec![9; 8];
    request
}

async fn characterize_failure_after(
    stage: FailureStage,
    failures: usize,
) -> (usize, usize, u64, usize, usize, usize) {
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
    worker.inject_failures_after(stage, failures).await;
    let item = IngestItem::Traces(trace_request_with_test_result(), bytes::Bytes::new());
    let mut progress = EffectProgress::default();
    for _ in 0..failures {
        worker
            .process_with_progress(&item, &mut progress)
            .await
            .expect_err("injected attempt fails");
    }
    worker
        .process_with_progress(&item, &mut progress)
        .await
        .expect("retry succeeds");

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
    let runs = metadata.invocations(10).await.expect("invocations").len();
    let tests = metadata
        .test_results_for_invocation("run-failure-oracle", 10)
        .await
        .expect("test results")
        .len();
    (broadcasts, spans, issue_count, errors, runs, tests)
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

#[test]
fn occurrence_identity_collapses_echoes_but_preserves_distinct_events() {
    let span = error_event(ErrorSource::SpanException, "span-a", "fp");
    let log_echo = error_event(ErrorSource::LogException, "span-a", "fp");
    let other_span = error_event(ErrorSource::SpanException, "span-b", "fp");
    let mut first_log = error_event(ErrorSource::LogRecord, "", "fp");
    first_log.trace_id.clear();
    let mut later_log = error_event(ErrorSource::LogRecord, "", "fp");
    later_log.trace_id.clear();
    later_log.ts_nanos = 2;

    assert_eq!(occurrence_id(&span), occurrence_id(&log_echo));
    assert_ne!(occurrence_id(&span), occurrence_id(&other_span));
    assert_ne!(occurrence_id(&first_log), occurrence_id(&later_log));
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
async fn otlp_and_sentry_echo_share_one_occurrence() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let store = Arc::new(MemoryStore::new());
    let metadata = Arc::new(
        MetadataStore::open(tmp.path().join("meta.db"))
            .await
            .expect("metadata"),
    );
    let worker = Worker::new(store.clone(), metadata.clone(), crate::live::channels());

    let mut request = trace_request_with_run_and_error();
    request.resource_spans[0].scope_spans[0].spans[0].events[0]
        .attributes
        .retain(|attribute| attribute.key != "exception.stacktrace");
    let otlp = derive::derive_from_traces(&request)
        .into_iter()
        .find(|event| event.source == ErrorSource::SpanException)
        .expect("OTLP exception");
    let sentry = parallax_analysis::sentry::derive_from_sentry_event(&json!({
        "event_id": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "timestamp": 0.000000042,
        "exception": {"values": [{"type": "test::Boom", "value": "boom"}]},
        "contexts": {"trace": {
            "trace_id": "01010101010101010101010101010101",
            "span_id": "0202020202020202"
        }},
        "tags": {"service": "checkout"}
    }))
    .expect("Sentry exception");

    assert_eq!(sentry.fingerprint, otlp.fingerprint);
    assert_eq!(occurrence_id(&sentry), occurrence_id(&otlp));
    worker
        .record_errors(vec![sentry, otlp])
        .await
        .expect("record cross-source echoes");

    let issues = metadata.issues(10).await.expect("issues");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].event_count, 1);
    let events = store
        .error_events_by_fingerprint(&issues[0].fingerprint, 0..=u128::MAX, 10)
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
    // stored error rows, registered runs, stored test results.
    assert_eq!(
        characterize_failure_after(FailureStage::Registration, 1).await,
        (1, 1, 1, 1, 1, 1),
        "registration succeeds before failure; its seen-run cache prevents a duplicate"
    );
    assert_eq!(
        characterize_failure_after(FailureStage::Broadcast, 1).await,
        (1, 1, 1, 1, 1, 1),
        "completed broadcast is checkpointed before retry"
    );
    assert_eq!(
        characterize_failure_after(FailureStage::TelemetryStorage, 1).await,
        (1, 1, 1, 1, 1, 1),
        "completed telemetry and earlier effects are checkpointed before retry"
    );
    assert_eq!(
        characterize_failure_after(FailureStage::IssueRecording, 1).await,
        (1, 1, 1, 1, 1, 1),
        "issue recording is checkpointed before test persistence"
    );
    assert_eq!(
        characterize_failure_after(FailureStage::TestRecording, 1).await,
        (1, 1, 1, 1, 1, 1),
        "the final test stage is checkpointed before retry"
    );
}

#[tokio::test]
async fn completed_effects_are_not_replayed_after_late_retries() {
    // This is a finite 5 x 3 state space. Random property cases repeated
    // expensive SQLite setup without increasing coverage and could exceed
    // nextest's slow timeout under parallel CI load.
    for stage in [
        FailureStage::Registration,
        FailureStage::Broadcast,
        FailureStage::TelemetryStorage,
        FailureStage::IssueRecording,
        FailureStage::TestRecording,
    ] {
        for failures in 1..=INGEST_RETRIES {
            assert_eq!(
                characterize_failure_after(stage, failures).await,
                (1, 1, 1, 1, 1, 1),
                "stage={stage:?}, failures={failures}",
            );
        }
    }
}

#[tokio::test]
async fn retry_exhaustion_metrics_match_worker_attempts() -> Result<(), String> {
    let tmp = tempfile::tempdir().expect("tempdir");
    let store = Arc::new(MemoryStore::new());
    let metadata = Arc::new(
        MetadataStore::open(tmp.path().join("meta.db"))
            .await
            .expect("metadata"),
    );
    let health = Arc::new(IngestHealth::new(1));
    let worker = Worker::new_with_health(store, metadata, crate::live::channels(), health.clone());
    worker
        .inject_failures_after(FailureStage::Registration, INGEST_RETRIES + 1)
        .await;
    let (senders, receivers) = channels(1);
    let enqueued_at = health.enqueued(Signal::Traces, Duration::ZERO, true);
    senders
        .traces
        .send(QueuedItem {
            item: IngestItem::Traces(ExportTraceServiceRequest::default(), bytes::Bytes::new()),
            enqueued_at,
            observed: true,
        })
        .await
        .expect("enqueue traces");
    drop(senders);

    worker.run(Signal::Traces, receivers.traces).await;

    let actual = health.snapshot(Signal::Traces);
    let expected = QueueSnapshot {
        depth: 0,
        capacity: 1,
        high_water: 1,
        retries: 3,
        drops: 1,
    };
    if actual != expected {
        return Err(format!("retry exhaustion health mismatch: {actual:?}"));
    }
    Ok(())
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
    assert_eq!(rows[0].invocation_id.as_deref(), Some("run-a"));
    assert_eq!(rows[0].attributes["route"], "/checkout");
}

#[tokio::test]
async fn trace_worker_persists_test_result_with_shared_issue_fingerprint() {
    use parallax_storage::metadata::MetadataStore as MetadataStorePort;
    use parallax_storage::model::TestStatus;

    let tmp = tempfile::tempdir().expect("tempdir");
    let store = Arc::new(MemoryStore::new());
    let metadata = Arc::new(
        MetadataStore::open(tmp.path().join("meta.db"))
            .await
            .expect("metadata"),
    );
    let worker = Worker::new(store, metadata.clone(), crate::live::channels());
    worker
        .process(&IngestItem::Traces(
            trace_request_with_test_result(),
            bytes::Bytes::new(),
        ))
        .await
        .expect("process test trace");

    let port: &dyn MetadataStorePort = metadata.as_ref();
    let results = port
        .test_results_for_invocation("run-failure-oracle", 10)
        .await
        .expect("test results");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].status, TestStatus::Failed);
    let fingerprint = results[0]
        .failure_fingerprint
        .as_deref()
        .expect("shared fingerprint");
    assert!(metadata.issue(fingerprint).await.expect("issue").is_some());
    let variant = port
        .test_variant(results[0].key.variant_key.as_str())
        .await
        .expect("variant")
        .expect("variant exists");
    assert!(
        port.test_case(variant.case_key.as_str())
            .await
            .expect("case")
            .is_some()
    );
    assert_eq!(
        results[0].trace_id.as_str(),
        "01010101010101010101010101010101"
    );
    assert_eq!(results[0].span_id, "0202020202020202");
}

#[tokio::test]
async fn malformed_test_projection_does_not_drop_native_trace() {
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
    let worker = Worker::new(store.clone(), metadata.clone(), crate::live::channels());
    let mut request = trace_request_with_test_result();
    request.resource_spans[0].scope_spans[0].spans[0]
        .attributes
        .push(string_kv("test.attempt.ordinal", "zero"));
    worker
        .process(&IngestItem::Traces(request, bytes::Bytes::new()))
        .await
        .expect("raw trace remains ingestible");

    assert_eq!(
        store
            .spans_by_trace("01010101010101010101010101010101")
            .await
            .expect("native spans")
            .len(),
        1
    );
    assert!(
        metadata
            .test_results_for_invocation("run-failure-oracle", 10)
            .await
            .expect("test results")
            .is_empty()
    );
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

    let traces_task = tokio::spawn(worker.clone().run(Signal::Traces, receivers.traces));
    let worker_logs = worker.clone();
    let logs_task = tokio::spawn(async move {
        let mut rx = receivers.logs;
        let mut logs_done_tx = Some(logs_done_tx);
        while let Some(item) = rx.recv().await {
            worker_logs.process(&item.item).await.expect("logs");
            if let Some(done) = logs_done_tx.take() {
                done.send(()).expect("signal logs completion");
            }
        }
    });
    let metrics_task = tokio::spawn(worker.run(Signal::Metrics, receivers.metrics));

    senders
        .traces
        .send(queued(IngestItem::Traces(
            ExportTraceServiceRequest::default(),
            bytes::Bytes::new(),
        )))
        .await
        .expect("enqueue traces");
    senders
        .logs
        .send(queued(IngestItem::Logs(
            ExportLogsServiceRequest::default(),
            bytes::Bytes::new(),
        )))
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
