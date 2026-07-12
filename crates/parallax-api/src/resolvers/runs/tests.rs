use super::*;
use crate::resolvers::test_support::*;
use crate::{RequestMemo, build_schema, execute};
use parallax_storage::adapter::TelemetryStore;
use parallax_storage::memory::MemoryStore;

use parallax_storage::model::{ErrorEventRow, ErrorSource};
use std::sync::Arc;

#[tokio::test]
async fn memo_helper_truncates_and_reuses_spans_for_same_trace() {
    let store = Arc::new(MemoryStore::new());
    let mut spans = Vec::new();
    for i in 0..(MAX_ROWS + 25) {
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
    assert_eq!(first.len(), MAX_ROWS);
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
            row.run_id = Some(run.into());
            spans.push(row);
        }
    }
    store.push_spans(spans);
    store
        .write_error_events(vec![
            ErrorEventRow {
                ts_nanos: 2_000_000_000,
                service: "api".into(),
                fingerprint: "fp-a".into(),
                error_type: "Error".into(),
                message: "boom-a".into(),
                stacktrace: None,
                source: ErrorSource::SpanStatus,
                trace_id: "ta1".into(),
                span_id: "run-a-s0".into(),
                attributes: serde_json::Value::Null,
            },
            ErrorEventRow {
                ts_nanos: 2_100_000_000,
                service: "api".into(),
                fingerprint: "fp-c".into(),
                error_type: "Error".into(),
                message: "boom-c".into(),
                stacktrace: None,
                source: ErrorSource::SpanStatus,
                trace_id: "tc2".into(),
                span_id: "run-c-s1".into(),
                attributes: serde_json::Value::Null,
            },
        ])
        .await
        .unwrap();
    let context = context_with_memory(store).await;
    for (run_id, command) in [("run-a", "a"), ("run-b", "b"), ("run-c", "c")] {
        context
            .metadata
            .start_run(run_id, Some(command), 1_000_000_000)
            .await
            .unwrap();
    }
    let schema = build_schema();
    let list = juniper::http::GraphQLRequest::new(
        r#"{ runs { runId errorCount traceCount } }"#.into(),
        None,
        None,
    );
    let list_json = serde_json::to_value(execute(&schema, &context, list).await).unwrap();
    assert!(error_messages(&list_json).is_empty(), "{list_json}");
    let mut by_id = std::collections::BTreeMap::new();
    for row in list_json
        .pointer("/data/runs")
        .and_then(|v| v.as_array())
        .unwrap()
    {
        by_id.insert(
            row["runId"].as_str().unwrap().to_string(),
            (
                row["errorCount"].as_i64().unwrap(),
                row["traceCount"].as_i64().unwrap(),
            ),
        );
    }
    assert_eq!(by_id["run-a"], (1, 2));
    assert_eq!(by_id["run-b"], (0, 1));
    assert_eq!(by_id["run-c"], (1, 3));
    for run_id in ["run-a", "run-b", "run-c"] {
        let single_ctx = ApiContext {
            store: context.store.clone(),
            metadata: context.metadata.clone(),
            otlp_grpc_port: 4317,
            memo: RequestMemo::default(),
        };
        let q = juniper::http::GraphQLRequest::new(
            format!(r#"{{ run(runId: "{run_id}") {{ errorCount traceCount }} }}"#),
            None,
            None,
        );
        let single = serde_json::to_value(execute(&schema, &single_ctx, q).await).unwrap();
        assert_eq!(
            (
                single
                    .pointer("/data/run/errorCount")
                    .and_then(|v| v.as_i64())
                    .unwrap(),
                single
                    .pointer("/data/run/traceCount")
                    .and_then(|v| v.as_i64())
                    .unwrap(),
            ),
            by_id[run_id],
        );
    }
}
