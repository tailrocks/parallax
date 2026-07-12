//! Adapter conformance scenarios shared by `MemoryStore` unit tests and the
//! gated real-engine greptime suite (plan 074).

use crate::adapter::{TelemetryStore, TraceQuery, TraceSort};
use crate::memory::MemoryStore;
use crate::model::{LogRow, SpanRow};
use std::ops::RangeInclusive;

fn sample_span(trace_id: &str, span_id: &str, ts: u128, duration: u128, error: bool) -> SpanRow {
    SpanRow {
        ts_nanos: ts,
        service: "api".into(),
        trace_id: trace_id.into(),
        span_id: span_id.into(),
        parent_span_id: None,
        name: "root".into(),
        kind: "SPAN_KIND_SERVER".into(),
        status_code: if error {
            "STATUS_CODE_ERROR".into()
        } else {
            "STATUS_CODE_UNSET".into()
        },
        status_message: String::new(),
        duration_ns: duration,
        run_id: None,
        scope_name: "test".into(),
        events: None,
        links: serde_json::Value::Null,
        attributes: serde_json::json!({}),
        resource: serde_json::json!({"service.name": "api"}),
    }
}

/// Seed a few spans and assert `traces_search` returns items.
pub async fn trace_search_scenario(store: &MemoryStore) -> anyhow::Result<()> {
    store.push_spans(vec![
        sample_span("t1", "s1", 1_000, 100, false),
        sample_span("t2", "s2", 2_000, 200, true),
    ]);
    let list = store
        .traces_search(&TraceQuery {
            limit: 10,
            sort: TraceSort::StartDesc,
            ..TraceQuery::default()
        })
        .await?;
    anyhow::ensure!(!list.items.is_empty(), "expected at least one trace");
    Ok(())
}

/// Seed logs and assert `log_count_series` returns a positive total over a window.
pub async fn log_count_series_scenario(store: &MemoryStore) -> anyhow::Result<()> {
    let logs = vec![LogRow {
        ts_nanos: 5_000,
        event_name: String::new(),
        observed_ts_nanos: 0,
        service: "api".into(),
        severity_num: 9,
        severity_text: "INFO".into(),
        body: "hello".into(),
        trace_id: String::new(),
        span_id: String::new(),
        run_id: None,
        scope_name: "test".into(),
        attributes: serde_json::json!({}),
        resource: serde_json::json!({"service.name": "api"}),
    }];
    store.push_logs(logs);
    let range: RangeInclusive<u128> = 0..=10_000;
    let series = store
        .log_count_series(Some("api"), range, None, None, None, 1_000)
        .await?;
    let total: f64 = series.iter().map(|p| p.value).sum();
    anyhow::ensure!(total >= 1.0, "expected log counts, got {total}");
    Ok(())
}

/// Overview totals over a seeded window should be non-negative.
pub async fn overview_totals_scenario(store: &dyn TelemetryStore) -> anyhow::Result<()> {
    let totals = store.overview_totals(0..=u128::MAX).await?;
    anyhow::ensure!(totals.trace_count < u64::MAX);
    anyhow::ensure!(totals.log_count < u64::MAX);
    Ok(())
}

/// Attribute compare should return without error on empty/minimal data.
pub async fn attribute_compare_scenario(store: &dyn TelemetryStore) -> anyhow::Result<()> {
    let _rows = store
        .attribute_compare(0..=10_000, 0..=10_000, None, false, &[], 5)
        .await?;
    Ok(())
}

/// Service map should return without error.
pub async fn service_map_scenario(store: &dyn TelemetryStore) -> anyhow::Result<()> {
    let _edges = store.service_map(0..=u128::MAX, 10).await?;
    Ok(())
}
