//! Shared adapter conformance seeds and assertions for memory and GreptimeDB.

use crate::memory::MemoryStore;
use parallax_model::{
    ErrorEventRow, ErrorSource, HistogramRow, LogRow, MetricExemplarRow, MetricPointRow, SpanRow,
};
use parallax_storage::adapter::{LogCountStore, TelemetryStore, TraceQuery};
use std::ops::RangeInclusive;

pub const SERVICE: &str = r#"api\'雪"#;
pub const METRIC: &str = "conformance.duration";
pub const TRACE_ID: &str = "000000000000000000000000000000a1";
pub const SPAN_ID: &str = "00000000000000b1";
pub const START: u128 = 1_741_437_296_000_000_000;
pub const END: u128 = START + 10_000_000_000;

pub fn range() -> RangeInclusive<u128> {
    START..=END
}

pub fn seed_memory(store: &MemoryStore) {
    store.push_spans(vec![SpanRow {
        ts_nanos: START + 1_000,
        service: SERVICE.into(),
        trace_id: TRACE_ID.into(),
        span_id: SPAN_ID.into(),
        parent_span_id: None,
        name: "conformance.root".into(),
        kind: "SPAN_KIND_SERVER".into(),
        status_code: "STATUS_CODE_ERROR".into(),
        status_message: "boom".into(),
        duration_ns: 2_000_000,
        invocation_id: Some("run-conformance".into()),
        session_id: None,
        scope_name: "conformance".into(),
        events: None,
        links: serde_json::Value::Null,
        attributes: serde_json::json!({"http.route": "/quoted"}),
        resource: serde_json::json!({"service.name": SERVICE}),
    }]);
    store.push_logs(vec![LogRow {
        ts_nanos: START + 2_000,
        event_name: "conformance.log".into(),
        observed_ts_nanos: START + 2_001,
        service: SERVICE.into(),
        severity_num: 17,
        severity_text: "ERROR".into(),
        body: "quoted backslash unicode failure".into(),
        trace_id: TRACE_ID.into(),
        span_id: SPAN_ID.into(),
        invocation_id: Some("run-conformance".into()),
        session_id: None,
        scope_name: "conformance".into(),
        attributes: serde_json::json!({"test.case": "full"}),
        resource: serde_json::json!({"service.name": SERVICE}),
    }]);
    store.push_metrics(
        vec![MetricPointRow {
            ts_nanos: START + 3_000,
            service: SERVICE.into(),
            name: METRIC.into(),
            value: 0.48,
            is_monotonic: false,
            invocation_id: Some("run-conformance".into()),
            attributes: serde_json::json!({"route": "quoted"}),
        }],
        vec![HistogramRow {
            ts_nanos: START + 4_000,
            service: SERVICE.into(),
            name: METRIC.into(),
            count: 2,
            sum: 0.6,
            bucket_counts: vec![1, 1, 0],
            bounds: vec![0.25, 0.5],
            attributes: serde_json::json!({}),
        }],
        vec![MetricExemplarRow {
            ts_nanos: START + 4_000,
            service: SERVICE.into(),
            name: METRIC.into(),
            value: 0.48,
            trace_id: TRACE_ID.into(),
            span_id: SPAN_ID.into(),
            invocation_id: Some("run-conformance".into()),
            attributes: serde_json::json!({"route": "quoted"}),
        }],
    );
    store.push_error_events(vec![ErrorEventRow {
        ts_nanos: START + 1_000,
        service: SERVICE.into(),
        fingerprint: "conformance-fingerprint".into(),
        error_type: "ConformanceError".into(),
        message: "boom".into(),
        stacktrace: None,
        source: ErrorSource::SpanStatus,
        trace_id: TRACE_ID.into(),
        span_id: SPAN_ID.into(),
        attributes: serde_json::json!({}),
    }]);
}

pub async fn assert_empty(
    store: &dyn TelemetryStore,
    window: RangeInclusive<u128>,
) -> anyhow::Result<()> {
    anyhow::ensure!(store.service_names(window.clone()).await?.is_empty());
    anyhow::ensure!(
        store
            .traces_search(&TraceQuery::default())
            .await?
            .items
            .is_empty()
    );
    anyhow::ensure!(
        store
            .logs_search(None, window.clone(), None, None, None, &[], 10)
            .await?
            .is_empty()
    );
    anyhow::ensure!(
        store
            .histogram_count_series("absent", None, window, 1_000)
            .await?
            .is_empty()
    );
    Ok(())
}

pub async fn assert_seeded(
    store: &dyn TelemetryStore,
    metric_name: &str,
    window: RangeInclusive<u128>,
) -> anyhow::Result<()> {
    let services = store.service_names(window.clone()).await?;
    anyhow::ensure!(
        services.iter().any(|service| service == SERVICE),
        "service names: {services:?}"
    );
    let traces = store.spans_by_trace(TRACE_ID).await?;
    anyhow::ensure!(
        traces.len() == 1
            && traces.first().is_some_and(
                |trace| trace.status_code == "STATUS_CODE_ERROR" && trace.service == SERVICE
            ),
        "traces: {traces:?}"
    );
    let logs = store
        .logs_search(
            Some(SERVICE),
            window.clone(),
            Some(17),
            None,
            Some("unicode"),
            &[],
            1,
        )
        .await?;
    anyhow::ensure!(logs.len() == 1, "logs: {logs:?}");
    let counts = store
        .log_count_series(
            Some(SERVICE),
            window.clone(),
            Some(17),
            None,
            Some("unicode"),
            &[],
            1_000_000_000,
        )
        .await?;
    anyhow::ensure!(counts.iter().map(|point| point.value).sum::<f64>() > 0.9);
    anyhow::ensure!(
        !store
            .histogram_count_series(metric_name, Some(SERVICE), window.clone(), 1_000_000_000,)
            .await?
            .is_empty()
    );
    anyhow::ensure!(
        store
            .histogram_count_series(
                metric_name,
                Some(SERVICE),
                (*window.end() + 1)..=(*window.end() + 2),
                1_000,
            )
            .await?
            .is_empty()
    );
    anyhow::ensure!(
        !store
            .metric_exemplars(METRIC, Some(SERVICE), window.clone(), 1)
            .await?
            .is_empty()
    );
    let trace_events = store.error_events_by_traces(&[TRACE_ID.into()], 1).await?;
    anyhow::ensure!(!trace_events.is_empty());
    let fingerprint = trace_events[0].fingerprint.clone();
    let fingerprints = vec![fingerprint.clone(), "absent-fingerprint".to_string()];
    let batched = store
        .error_events_by_fingerprints(&fingerprints, window, 1)
        .await?;
    anyhow::ensure!(
        batched
            .get(&fingerprint)
            .is_some_and(|events| events.len() == 1),
        "batched fingerprint events: {batched:?}"
    );
    anyhow::ensure!(
        batched.get("absent-fingerprint").is_some_and(Vec::is_empty),
        "batched missing fingerprint: {batched:?}"
    );
    Ok(())
}

// Kept as small compatibility scenarios for focused unit callers.
pub async fn trace_search_scenario(store: &MemoryStore) -> anyhow::Result<()> {
    seed_memory(store);
    assert_seeded(store, METRIC, range()).await
}

pub async fn log_count_series_scenario(store: &MemoryStore) -> anyhow::Result<()> {
    let total: f64 = store
        .log_count_series(Some(SERVICE), range(), None, None, None, &[], 1_000)
        .await?
        .iter()
        .map(|point| point.value)
        .sum();
    anyhow::ensure!(total >= 1.0);
    Ok(())
}

pub async fn overview_totals_scenario(store: &dyn TelemetryStore) -> anyhow::Result<()> {
    let totals = store.overview_totals(range()).await?;
    anyhow::ensure!(totals.trace_count < u64::MAX && totals.log_count < u64::MAX);
    Ok(())
}

pub async fn attribute_compare_scenario(store: &dyn TelemetryStore) -> anyhow::Result<()> {
    store
        .attribute_compare(range(), range(), None, false, &[], 5)
        .await?;
    Ok(())
}

pub async fn service_map_scenario(store: &dyn TelemetryStore) -> anyhow::Result<()> {
    store.service_map(range(), 10).await?;
    Ok(())
}
