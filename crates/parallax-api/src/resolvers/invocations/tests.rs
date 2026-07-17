use super::*;
use crate::resolvers::test_support::*;
use crate::{RequestMemo, build_schema, execute};
use parallax_storage::adapter::IngestStore;
use parallax_test_support::builders::MemoryStore;

use parallax_storage::model::{ErrorEventRow, ErrorSource};
use std::sync::Arc;

#[tokio::test]
async fn memo_helper_truncates_and_reuses_spans_for_same_trace() {
    let store = Arc::new(MemoryStore::new());
    let mut spans = Vec::new();
    // Whole-trace reads cap at TRACE_SPANS_MAX (memory guard), NOT list-page
    // MAX_ROWS: a 521-span trace must arrive complete (corpus id t-wide).
    for i in 0..(crate::TRACE_SPANS_MAX + 25) {
        spans.push(span(
            "api",
            "big-trace",
            &format!("s{i}"),
            1_000_000_000 + i as u128,
            1_000,
        ));
    }
    store.push_spans(spans);
    let context = context_with_memory(store).await;
    let first = context.spans_for("big-trace").await.unwrap();
    let second = context.spans_for("big-trace").await.unwrap();
    assert_eq!(first.len(), crate::TRACE_SPANS_MAX);
    assert!(first.len() > MAX_ROWS, "trace reads outgrow list pagination");
    assert!(Arc::ptr_eq(&first, &second));
}

#[tokio::test]
async fn runs_list_stats_match_single_run() {
    let store = Arc::new(MemoryStore::new());
    let mut spans = Vec::new();
    for (run, traces) in [
        ("run-a", &["ta1", "ta2"][..]),
        ("run-b", &["tb1"][..]),
        ("run-c", &["tc1", "tc2", "tc3"][..]),
    ] {
        for (i, trace) in traces.iter().enumerate() {
            let mut row = span(
                "api",
                trace,
                &format!("{run}-s{i}"),
                1_000_000_000 + i as u128,
                5_000,
            );
            row.invocation_id = Some(run.into());
            spans.push(row);
        }
    }
    store.push_spans(spans);
    store
        .write_error_events(vec![
            run_error(2_000_000_000, "fp-a", "boom-a", "ta1", "run-a-s0"),
            run_error(2_100_000_000, "fp-c", "boom-c", "tc2", "run-c-s1"),
        ])
        .await
        .unwrap();
    let context = context_with_memory(store).await;
    for (invocation_id, command) in [("run-a", "a"), ("run-b", "b"), ("run-c", "c")] {
        context
            .metadata
            .start_invocation(invocation_id, Some(command), None, 1_000_000_000)
            .await
            .unwrap();
    }
    let schema = build_schema();
    let list = juniper::http::GraphQLRequest::new(
        r"{ invocations { invocationId errorCount traceCount } }".into(),
        None,
        None,
    );
    let list_json = serde_json::to_value(execute(&schema, &context, list).await).unwrap();
    assert!(error_messages(&list_json).is_empty(), "{list_json}");
    let mut by_id = std::collections::BTreeMap::new();
    for row in list_json
        .pointer("/data/invocations")
        .and_then(|v| v.as_array())
        .unwrap()
    {
        by_id.insert(
            row["invocationId"].as_str().unwrap().to_string(),
            (
                row["errorCount"].as_i64().unwrap(),
                row["traceCount"].as_i64().unwrap(),
            ),
        );
    }
    assert_eq!(by_id["run-a"], (1, 2));
    assert_eq!(by_id["run-b"], (0, 1));
    assert_eq!(by_id["run-c"], (1, 3));
    for invocation_id in ["run-a", "run-b", "run-c"] {
        let single_ctx = ApiContext {
            store: context.store.clone(),
            metadata: context.metadata.clone(),
            otlp_grpc_port: 4317,
            otlp_http_port: 4318,
            memo: RequestMemo::default(),
        };
        let q = juniper::http::GraphQLRequest::new(
            format!(
                r#"{{ invocation(invocationId: "{invocation_id}") {{ errorCount traceCount }} }}"#
            ),
            None,
            None,
        );
        let single = serde_json::to_value(execute(&schema, &single_ctx, q).await).unwrap();
        assert_eq!(
            (
                single
                    .pointer("/data/invocation/errorCount")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap(),
                single
                    .pointer("/data/invocation/traceCount")
                    .and_then(serde_json::Value::as_i64)
                    .unwrap(),
            ),
            by_id[invocation_id],
        );
    }
}

fn run_error(
    ts_nanos: u128,
    fingerprint: &str,
    message: &str,
    trace_id: &str,
    span_id: &str,
) -> ErrorEventRow {
    ErrorEventRow {
        ts_nanos,
        service: "api".into(),
        fingerprint: fingerprint.into(),
        error_type: "Error".into(),
        message: message.into(),
        stacktrace: None,
        source: ErrorSource::SpanStatus,
        trace_id: trace_id.into(),
        span_id: span_id.into(),
        attributes: serde_json::Value::Null,
    }
}
