use super::*;
use crate::resolvers::test_support::*;
use crate::{build_schema, execute};

use parallax_test_support::builders::MemoryStore;

use std::sync::Arc;

#[tokio::test]
async fn logs_around_returns_windowed_ascending_rows() {
    let store = Arc::new(MemoryStore::new());
    let anchor = 100_000_000_000;
    let mut anchor_log = log_row("api", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", anchor, "anchor");
    anchor_log.event_name = "checkout.completed".into();
    anchor_log.observed_ts_nanos = anchor + 2_000_000_000;
    store.push_logs(vec![
        log_row(
            "api",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            anchor - 60_000_000_000,
            "too-old",
        ),
        log_row(
            "api",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            anchor - 10_000_000_000,
            "before",
        ),
        anchor_log,
        log_row(
            "api",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            anchor + 10_000_000_000,
            "after",
        ),
        log_row(
            "api",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            anchor + 60_000_000_000,
            "too-new",
        ),
    ]);
    let schema = build_schema();
    let context = context_with_memory(store).await;
    let request = juniper::http::GraphQLRequest::new(
        format!(
            r#"{{
              logsAround(anchorNanos: "{anchor}", windowSeconds: 30, service: "api") {{
                tsNanos body eventName observedTsNanos
              }}
            }}"#
        ),
        None,
        None,
    );
    let response = execute(&schema, &context, request).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(error_messages(&json).is_empty(), "logsAround query: {json}");
    let rows = json
        .pointer("/data/logsAround")
        .and_then(|value| value.as_array())
        .unwrap();
    assert_eq!(
        rows.iter()
            .map(|row| row["body"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["before", "anchor", "after"]
    );
    assert_eq!(
        rows[1].get("eventName"),
        Some(&serde_json::json!("checkout.completed"))
    );
    assert_eq!(
        rows[1].get("observedTsNanos"),
        Some(&serde_json::json!("102000000000"))
    );
}

#[tokio::test]
async fn logs_around_can_scope_to_trace_inside_window() {
    let store = Arc::new(MemoryStore::new());
    let anchor = 100_000_000_000;
    store.push_logs(vec![
        log_row(
            "api",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            anchor - 1_000_000_000,
            "trace-a-before",
        ),
        log_row(
            "api",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            anchor,
            "trace-b-anchor",
        ),
        log_row(
            "api",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            anchor + 1_000_000_000,
            "trace-a-after",
        ),
    ]);
    let schema = build_schema();
    let context = context_with_memory(store).await;
    let request = juniper::http::GraphQLRequest::new(
        format!(
            r#"{{
              logsAround(anchorNanos: "{anchor}", windowSeconds: 30, traceId: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa") {{
                body traceId
              }}
            }}"#
        ),
        None,
        None,
    );
    let response = execute(&schema, &context, request).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(error_messages(&json).is_empty(), "logsAround trace: {json}");
    assert_eq!(
        json.pointer("/data/logsAround")
            .and_then(|value| value.as_array())
            .unwrap()
            .iter()
            .map(|row| row["body"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["trace-a-before", "trace-a-after"]
    );
}

#[tokio::test]
async fn logs_around_clamps_window_and_limit() {
    let store = Arc::new(MemoryStore::new());
    let anchor = 1_000_000_000_000;
    let mut rows = (0..550)
        .map(|index| {
            log_row(
                "api",
                "trace-a",
                anchor + index * 1_000_000,
                &format!("near-{index}"),
            )
        })
        .collect::<Vec<_>>();
    rows.push(log_row(
        "api",
        "trace-a",
        anchor + 700_000_000_000,
        "beyond-clamped-window",
    ));
    store.push_logs(rows);
    let schema = build_schema();
    let context = context_with_memory(store).await;
    let request = juniper::http::GraphQLRequest::new(
        format!(
            r#"{{
              logsAround(anchorNanos: "{anchor}", windowSeconds: 9999, limit: 9999) {{
                body
              }}
            }}"#
        ),
        None,
        None,
    );
    let response = execute(&schema, &context, request).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(error_messages(&json).is_empty(), "logsAround clamp: {json}");
    let rows = json
        .pointer("/data/logsAround")
        .and_then(|value| value.as_array())
        .unwrap();
    assert_eq!(rows.len(), MAX_ROWS);
    assert!(
        rows.iter()
            .all(|row| row["body"] != "beyond-clamped-window")
    );
}

#[tokio::test]
async fn logs_attribute_filters_narrow_rows_series_and_facets() {
    let store = Arc::new(MemoryStore::new());
    let mut checkout = log_row(
        "api",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        1_000,
        "checkout ok",
    );
    checkout.attributes = serde_json::json!({ "http.request.method": "POST" });
    let mut health = log_row(
        "api",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        2_000,
        "health ok",
    );
    health.attributes = serde_json::json!({ "http.request.method": "GET" });
    store.push_logs(vec![checkout, health]);

    let schema = build_schema();
    let context = context_with_memory(store).await;
    let request = juniper::http::GraphQLRequest::new(
        r#"
        {
          logs(fromNanos: "0", toNanos: "10000", attributeFilters: [{key: "http.request.method", op: "=", value: "POST"}], limit: 10) {
            body
          }
          logCountSeries(fromNanos: "0", toNanos: "10000", attributeFilters: [{key: "http.request.method", op: "=", value: "POST"}], stepSeconds: 1) {
            value
          }
          logFacets(fromNanos: "0", toNanos: "10000") {
            dimension
            values { value count }
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
        json.pointer("/data/logs"),
        Some(&serde_json::json!([{ "body": "checkout ok" }])),
        "narrowed: {json}"
    );
    let series_total: f64 = json
        .pointer("/data/logCountSeries")
        .and_then(|points| points.as_array())
        .into_iter()
        .flatten()
        .filter_map(|point| point.pointer("/value").and_then(|v| v.as_f64()))
        .sum();
    assert_eq!(series_total, 1.0, "series reflects the filter: {json}");
    let method_facet = json
        .pointer("/data/logFacets")
        .and_then(|facets| facets.as_array())
        .into_iter()
        .flatten()
        .find(|facet| {
            facet.pointer("/dimension").and_then(|d| d.as_str()) == Some("http.request.method")
        })
        .cloned()
        .unwrap_or_else(|| panic!("missing method facet: {json}"));
    assert_eq!(
        method_facet.pointer("/values"),
        Some(&serde_json::json!([
            {"value": "GET", "count": "1"},
            {"value": "POST", "count": "1"}
        ]))
    );
}
