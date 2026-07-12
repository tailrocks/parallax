use super::*;
use crate::resolvers::test_support::*;
use crate::{build_schema, execute};

use parallax_storage::memory::MemoryStore;

use parallax_storage::model::SpanRow;
use std::sync::Arc;

#[tokio::test]
async fn trace_events_filters_orders_and_reports_caps() {
    let store = Arc::new(MemoryStore::new());
    let mut root = span("checkout", "trace-a", "span-a", 1_000, 100);
    root.name = "root".into();
    root.events = Some(
        r#"[
            {"name":"exception","time_unix_nano":30,"attributes":{"message":"bad"}},
            {"name":"rpc.message.sent","timeUnixNano":"10","attributes":{"message.type":"SENT","id":7}}
        ]"#
        .into(),
    );
    let mut child = span("payments", "trace-a", "span-b", 2_000, 100);
    child.name = "client".into();
    child.events = Some(
        r#"[{"name":"rpc.message.received","time_unix_nano":20,"attributes":{"message.type":"RECEIVED"}}]"#
            .into(),
    );
    store.push_spans(vec![root, child]);

    let schema = build_schema();
    let context = context_with_memory(store).await;
    let request = juniper::http::GraphQLRequest::new(
        r#"
        {
          traceEvents(traceId: "trace-a", namePrefix: "rpc.message", limit: 1) {
            truncated
            skippedSpans
            events { name spanId spanName service timeUnixNano attributes }
          }
        }
        "#
        .into(),
        None,
        None,
    );
    let response = execute(&schema, &context, request).await;
    let json = serde_json::to_value(response).unwrap();

    assert!(
        error_messages(&json).is_empty(),
        "traceEvents query succeeds: {json}"
    );
    assert_eq!(
        json.pointer("/data/traceEvents/truncated"),
        Some(&serde_json::json!(true))
    );
    assert_eq!(
        json.pointer("/data/traceEvents/skippedSpans"),
        Some(&serde_json::json!(0))
    );
    assert_eq!(
        json.pointer("/data/traceEvents/events/0/name"),
        Some(&serde_json::json!("rpc.message.sent"))
    );
    assert_eq!(
        json.pointer("/data/traceEvents/events/0/spanId"),
        Some(&serde_json::json!("span-a"))
    );
    assert_eq!(
        json.pointer("/data/traceEvents/events/0/timeUnixNano"),
        Some(&serde_json::json!("10"))
    );
    assert_eq!(
        json.pointer("/data/traceEvents/events/0/attributes"),
        Some(&serde_json::json!(r#"{"id":"7","message.type":"SENT"}"#))
    );
}

#[tokio::test]
async fn trace_events_counts_malformed_span_events() {
    let store = Arc::new(MemoryStore::new());
    let mut good = span("checkout", "trace-a", "span-a", 1_000, 100);
    good.events = Some(r#"[{"name":"rpc.message","time_unix_nano":10,"attributes":{}}]"#.into());
    let mut bad = span("checkout", "trace-a", "span-b", 2_000, 100);
    bad.events = Some("{not json".into());
    store.push_spans(vec![good, bad]);

    let schema = build_schema();
    let context = context_with_memory(store).await;
    let request = juniper::http::GraphQLRequest::new(
        r#"
        {
          traceEvents(traceId: "trace-a") {
            truncated
            skippedSpans
            events { name spanId }
          }
        }
        "#
        .into(),
        None,
        None,
    );
    let response = execute(&schema, &context, request).await;
    let json = serde_json::to_value(response).unwrap();

    assert!(
        error_messages(&json).is_empty(),
        "traceEvents malformed span query succeeds: {json}"
    );
    assert_eq!(
        json.pointer("/data/traceEvents/skippedSpans"),
        Some(&serde_json::json!(1))
    );
    assert_eq!(
        json.pointer("/data/traceEvents/truncated"),
        Some(&serde_json::json!(false))
    );
    assert_eq!(
        json.pointer("/data/traceEvents/events/0/name"),
        Some(&serde_json::json!("rpc.message"))
    );
}

#[test]
fn parses_typed_span_links_from_stored_json() {
    let links = serde_json::json!([
        {
            "traceId": "target-trace",
            "spanId": "target-span",
            "attributes": { "link.kind": "batch" }
        },
        {
            "trace_id": "native-target",
            "span_id": "native-span",
            "attributes": { "link.kind": "native" }
        },
        { "traceId": "", "spanId": "ignored" },
        { "spanId": "missing-trace" }
    ]);

    let parsed = span_links_from_value(&links);

    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].trace_id, "target-trace");
    assert_eq!(parsed[0].span_id, "target-span");
    assert_eq!(parsed[0].attributes, r#"{"link.kind":"batch"}"#);
    assert_eq!(parsed[1].trace_id, "native-target");
    assert_eq!(parsed[1].span_id, "native-span");
    assert_eq!(parsed[1].attributes, r#"{"link.kind":"native"}"#);
}

#[tokio::test]
async fn linked_traces_resolves_span_link_targets() {
    let store = Arc::new(MemoryStore::new());
    let mut source = span("api", "source", "source-root", 10, 10_000_000);
    source.name = "publish".into();
    source.links = serde_json::json!([
        {
            "traceId": "target",
            "spanId": "target-root",
            "attributes": { "messaging.operation": "publish" }
        }
    ]);
    let mut target = span("worker", "target", "target-root", 20, 20_000_000);
    target.name = "consume".into();
    store.push_spans(vec![source, target]);

    let schema = build_schema();
    let context = context_with_memory(store).await;
    let request = juniper::http::GraphQLRequest::new(
        r#"
        {
          trace(traceId: "source") {
            spans {
              spanId
              typedLinks { traceId spanId attributes }
            }
          }
          linkedTraces(traceId: "source") {
            traceId
            rootName
            service
            spanCount
            hasError
          }
        }
        "#
        .into(),
        None,
        None,
    );
    let response = execute(&schema, &context, request).await;
    let json = serde_json::to_value(response).unwrap();

    assert!(
        error_messages(&json).is_empty(),
        "linkedTraces query: {json}"
    );
    assert_eq!(
        json.pointer("/data/trace/spans/0/typedLinks/0/traceId"),
        Some(&serde_json::json!("target"))
    );
    assert_eq!(
        json.pointer("/data/trace/spans/0/typedLinks/0/spanId"),
        Some(&serde_json::json!("target-root"))
    );
    assert_eq!(
        json.pointer("/data/linkedTraces/0/traceId"),
        Some(&serde_json::json!("target"))
    );
    assert_eq!(
        json.pointer("/data/linkedTraces/0/rootName"),
        Some(&serde_json::json!("consume"))
    );
    assert_eq!(
        json.pointer("/data/linkedTraces/0/service"),
        Some(&serde_json::json!("worker"))
    );
}

#[tokio::test]
async fn trace_analysis_resolvers_return_path_and_diff() {
    let store = Arc::new(MemoryStore::new());
    let a_root = span("api", "a", "a-root", 0, 100);
    let mut a_db = span("db", "a", "a-db", 20, 40);
    a_db.parent_span_id = Some("a-root".into());
    let mut b_root = span("api", "b", "b-root", 0, 120);
    b_root.name = "handler".into();
    let mut b_db = span("db", "b", "b-db", 20, 60);
    b_db.parent_span_id = Some("b-root".into());
    b_db.status_code = "STATUS_CODE_ERROR".into();
    let mut b_retry = span("api", "b", "b-retry", 90, 10);
    b_retry.parent_span_id = Some("b-root".into());
    b_retry.name = "retry".into();
    store.push_spans(vec![a_root, a_db, b_root, b_db, b_retry]);

    let schema = build_schema();
    let context = context_with_memory(store).await;
    let request = juniper::http::GraphQLRequest::new(
        r#"
        {
          traceCriticalPath(traceId: "a") {
            totalGatedNs
            hops { spanId gatedByChild selfTimeNs clockSuspect }
            unattached
          }
          traceCompare(traceIdA: "a", traceIdB: "b") {
            added { name service }
            removed { name }
            changed {
              durationDeltaNs
              statusChanged
              before { name statusCode }
              after { name statusCode }
            }
          }
        }
        "#
        .into(),
        None,
        None,
    );
    let response = execute(&schema, &context, request).await;
    let json = serde_json::to_value(response).unwrap();

    assert!(
        error_messages(&json).is_empty(),
        "trace analysis query: {json}"
    );
    assert_eq!(
        json.pointer("/data/traceCriticalPath/totalGatedNs"),
        Some(&serde_json::json!("100"))
    );
    assert_eq!(
        json.pointer("/data/traceCriticalPath/hops/0/gatedByChild"),
        Some(&serde_json::json!("a-db"))
    );
    assert_eq!(
        json.pointer("/data/traceCompare/added/0/name"),
        Some(&serde_json::json!("retry"))
    );
    assert_eq!(
        json.pointer("/data/traceCompare/changed/0/durationDeltaNs"),
        Some(&serde_json::json!("20"))
    );
    assert_eq!(
        json.pointer("/data/traceCompare/changed/1/statusChanged"),
        Some(&serde_json::json!(true))
    );
}

#[tokio::test]
async fn trace_critical_path_errors_for_empty_trace() {
    let schema = build_schema();
    let context = context_with_memory(Arc::new(MemoryStore::new())).await;
    let request = juniper::http::GraphQLRequest::new(
        r#"{ traceCriticalPath(traceId: "missing") { totalGatedNs } }"#.into(),
        None,
        None,
    );
    let response = execute(&schema, &context, request).await;
    let json = serde_json::to_value(response).unwrap();

    assert!(
        error_messages(&json)
            .iter()
            .any(|message| message.contains("trace has no spans")),
        "empty trace rejected: {json}"
    );
}

#[tokio::test]
async fn traces_page_returns_total_and_span_events_json() {
    let store = Arc::new(MemoryStore::new());
    let mut mid = span("api", "mid", "b", 20, 20_000_000);
    mid.events = Some(
        r#"[{"name":"exception","timeUnixNano":"20","attributes":{"message":"bad"}}]"#.to_string(),
    );
    store.push_spans(vec![
        span("api", "fast", "a", 10, 10_000_000),
        mid,
        span("api", "slow", "c", 30, 30_000_000),
    ]);

    let schema = build_schema();
    let context = context_with_memory(store).await;
    let request = juniper::http::GraphQLRequest::new(
        r#"
        {
          tracesPage(sort: DURATION_DESC, limit: 2, offset: 1) {
            total
            items { traceId durationNs }
          }
          trace(traceId: "mid") {
            spans { spanId events }
          }
        }
        "#
        .into(),
        None,
        None,
    );
    let response = execute(&schema, &context, request).await;
    let json = serde_json::to_value(response).unwrap();
    assert_eq!(
        json.pointer("/data/tracesPage/total"),
        Some(&serde_json::json!("3"))
    );
    assert_eq!(
        json.pointer("/data/tracesPage/items/0/traceId"),
        Some(&serde_json::json!("mid"))
    );
    assert_eq!(
        json.pointer("/data/tracesPage/items/1/traceId"),
        Some(&serde_json::json!("fast"))
    );
    let events = json
        .pointer("/data/trace/spans/0/events")
        .and_then(serde_json::Value::as_str)
        .unwrap();
    assert!(events.contains("exception"));
}

#[test]
fn compare_two_500_span_traces_timing() {
    let make = |trace_id: &str| -> Vec<SpanRow> {
        (0..500)
            .map(|i| {
                let mut row = span(
                    "svc",
                    trace_id,
                    &format!("{trace_id}-{i}"),
                    1_000_000_000 + i as u128 * 1_000,
                    5_000,
                );
                if i > 0 {
                    row.parent_span_id = Some(format!("{trace_id}-{}", i / 2));
                }
                row.name = format!("op.{}", i % 40);
                row
            })
            .collect()
    };
    let start = std::time::Instant::now();
    drop(trace_analysis::compare(&make("a"), &make("b")));
    let elapsed = start.elapsed();
    eprintln!(
        "trace_analysis::compare on 2x500 spans: {:.3} ms",
        elapsed.as_secs_f64() * 1000.0
    );
    // Budget is deliberately loose: CI shared runners can be 2–4× slower
    // than a laptop for pure CPU work; 50 ms failed at ~55 ms on GHA.
    assert!(elapsed.as_millis() < 200, "compare slow: {elapsed:?}");
}
