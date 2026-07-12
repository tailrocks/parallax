#![expect(clippy::too_many_lines, reason = "measured integration scenario")]

use super::*;
use crate::INVESTIGATION_PIN_CAP;
use crate::resolvers::test_support::*;
use crate::{build_schema, execute};

use parallax_test_support::builders::MemoryStore;

use std::sync::Arc;

#[tokio::test]
async fn saved_view_resolvers_round_trip_filter_delete_and_cap() {
    let schema = build_schema();
    let context = context_with_memory(Arc::new(MemoryStore::new())).await;
    let save = juniper::http::GraphQLRequest::new(
        r#"
        mutation {
          savedViewSave(name: "Errors", page: "/logs", state: "?sev=17&cols=trace") {
            id name page state
          }
        }
        "#
        .into(),
        None,
        None,
    );
    let response = execute(&schema, &context, save).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(error_messages(&json).is_empty(), "savedViewSave: {json}");
    let id = json
        .pointer("/data/savedViewSave/id")
        .and_then(|value| value.as_str())
        .unwrap()
        .to_string();
    assert_eq!(
        json.pointer("/data/savedViewSave/state"),
        Some(&serde_json::json!("?sev=17&cols=trace"))
    );

    let list = juniper::http::GraphQLRequest::new(
        r#"{ savedViews(page: "/logs") { id name page state } }"#.into(),
        None,
        None,
    );
    let response = execute(&schema, &context, list).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(error_messages(&json).is_empty(), "savedViews: {json}");
    assert_eq!(
        json.pointer("/data/savedViews/0/id"),
        Some(&serde_json::json!(id.as_str()))
    );

    let delete = juniper::http::GraphQLRequest::new(
        format!(r#"mutation {{ savedViewDelete(id: "{id}") }}"#),
        None,
        None,
    );
    let response = execute(&schema, &context, delete).await;
    let json = serde_json::to_value(response).unwrap();
    assert_eq!(
        json.pointer("/data/savedViewDelete"),
        Some(&serde_json::json!(true))
    );

    for index in 0..SAVED_VIEWS_PER_PAGE {
        context
            .metadata
            .saved_view_save(
                &format!("view-{index}"),
                "View",
                "/logs",
                "?q=x",
                index as u128,
            )
            .await
            .unwrap();
    }
    let capped = juniper::http::GraphQLRequest::new(
        r#"mutation { savedViewSave(name: "Too many", page: "/logs", state: "?q=y") { id } }"#
            .into(),
        None,
        None,
    );
    let response = execute(&schema, &context, capped).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(
        error_messages(&json)
            .iter()
            .any(|message| message.contains("saved view cap")),
        "saved view cap enforced: {json}"
    );
}

#[tokio::test]
async fn investigation_resolvers_round_trip_and_validate_state() {
    let schema = build_schema();
    let context = context_with_memory(Arc::new(MemoryStore::new())).await;
    let state = r#"{"version":1,"window":{"range":"24h"},"pins":[{"kind":"trace","ref":"/traces/t1","label":"trace"}],"notes":"triage"}"#;
    let save = juniper::http::GraphQLRequest::new(
        format!(
            r#"
            mutation {{
              investigationSave(name: "Checkout case", state: "{}") {{
                id name state
              }}
            }}
            "#,
            state.replace('"', "\\\"")
        ),
        None,
        None,
    );
    let response = execute(&schema, &context, save).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(
        error_messages(&json).is_empty(),
        "investigationSave: {json}"
    );
    let id = json
        .pointer("/data/investigationSave/id")
        .and_then(|value| value.as_str())
        .unwrap()
        .to_string();
    assert_eq!(
        json.pointer("/data/investigationSave/name"),
        Some(&serde_json::json!("Checkout case"))
    );

    let list = juniper::http::GraphQLRequest::new(
        r"{ investigations { id name state updatedAtNanos } }".into(),
        None,
        None,
    );
    let response = execute(&schema, &context, list).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(error_messages(&json).is_empty(), "investigations: {json}");
    assert_eq!(
        json.pointer("/data/investigations/0/id"),
        Some(&serde_json::json!(id.as_str()))
    );

    let get = juniper::http::GraphQLRequest::new(
        format!(r#"{{ investigation(id: "{id}") {{ id name state }} }}"#),
        None,
        None,
    );
    let response = execute(&schema, &context, get).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(error_messages(&json).is_empty(), "investigation: {json}");
    assert_eq!(
        json.pointer("/data/investigation/id"),
        Some(&serde_json::json!(id.as_str()))
    );

    let delete = juniper::http::GraphQLRequest::new(
        format!(r#"mutation {{ investigationDelete(id: "{id}") }}"#),
        None,
        None,
    );
    let response = execute(&schema, &context, delete).await;
    let json = serde_json::to_value(response).unwrap();
    assert_eq!(
        json.pointer("/data/investigationDelete"),
        Some(&serde_json::json!(true))
    );

    let bad_json = juniper::http::GraphQLRequest::new(
        r#"mutation { investigationSave(name: "Bad", state: "{bad json") { id } }"#.into(),
        None,
        None,
    );
    let response = execute(&schema, &context, bad_json).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(
        error_messages(&json)
            .iter()
            .any(|message| message.contains("state must be valid JSON")),
        "bad JSON rejected: {json}"
    );

    let pins = (0..=INVESTIGATION_PIN_CAP)
        .map(|index| {
            serde_json::json!({
                "kind": "trace",
                "ref": format!("/traces/{index}"),
                "label": format!("trace {index}")
            })
        })
        .collect::<Vec<_>>();
    let capped_state = serde_json::json!({
        "version": 1,
        "window": {"range": "24h"},
        "pins": pins,
        "notes": ""
    })
    .to_string();
    let capped = juniper::http::GraphQLRequest::new(
        format!(
            r#"mutation {{ investigationSave(name: "Too many", state: "{}") {{ id }} }}"#,
            capped_state.replace('"', "\\\"")
        ),
        None,
        None,
    );
    let response = execute(&schema, &context, capped).await;
    let json = serde_json::to_value(response).unwrap();
    assert!(
        error_messages(&json)
            .iter()
            .any(|message| message.contains("pin cap")),
        "pin cap enforced: {json}"
    );
}
