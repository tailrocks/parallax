use crate::resolvers::test_support::{context_with_memory, error_messages};
use crate::{build_schema, execute};
use parallax_storage::adapter::IngestStore;
use parallax_storage::model::{ErrorEventRow, ErrorSource, IssueOccurrence};
use parallax_test_support::builders::MemoryStore;
use std::sync::Arc;

fn fixture_map() -> String {
    serde_json::json!({
        "version": 3,
        "file": "app.min.js",
        "sources": ["src/app.ts"],
        "names": ["onClick", "render"],
        "mappings": "AAAA,UAAIA,eAEFC"
    })
    .to_string()
}

fn stacktrace() -> String {
    "Error: boom\n    at onClick (https://cdn.example.com/app.min.js:1:25)\n    at https://cdn.example.com/app.min.js:1:10".to_string()
}

fn graphql(query: String) -> juniper::http::GraphQLRequest {
    juniper::http::GraphQLRequest::new(query, None, None)
}

async fn seed_issue_with_stack(
    store: &Arc<MemoryStore>,
    context: &crate::ApiContext,
    service_version: Option<&str>,
) {
    let attributes = serde_json::json!({"env": "test"});
    context
        .metadata
        .upsert_issue_occurrence(&IssueOccurrence {
            occurrence_id: "fp-map:1".into(),
            fingerprint: "fp-map",
            title: "Error fp-map".to_string(),
            error_type: "test::Minified",
            culprit: None,
            service: "web",
            ts_nanos: 100,
            trace_id: None,
            attributes: &attributes,
        })
        .await
        .unwrap();
    store
        .write_error_events(vec![ErrorEventRow {
            ts_nanos: 100,
            service: "web".to_string(),
            fingerprint: "fp-map".to_string(),
            error_type: "test::Minified".to_string(),
            message: "boom".to_string(),
            stacktrace: Some(stacktrace()),
            source: ErrorSource::LogRecord,
            trace_id: String::new(),
            span_id: String::new(),
            invocation_id: None,
            session_id: None,
            service_version: service_version.map(str::to_string),
            environment: None,
            attributes,
        }])
        .await
        .unwrap();
}

#[tokio::test]
async fn upload_then_mapped_frames_resolve_at_issue_detail() {
    let store = Arc::new(MemoryStore::new());
    let context = context_with_memory(Arc::clone(&store)).await;
    seed_issue_with_stack(&store, &context, Some("1.2.3")).await;
    let map_json = serde_json::to_string(&fixture_map()).expect("escape");

    let upload = execute(
        &build_schema(),
        &context,
        graphql(format!(
            r#"mutation {{ sourceMapUpload(service: "web", version: "1.2.3", file: "app.min.js", map: {map_json}) {{ service version file mapSha256 mapBytes }} }}"#
        )),
    )
    .await;
    let json = serde_json::to_value(upload).unwrap();
    assert!(error_messages(&json).is_empty(), "upload: {json}");
    assert_eq!(
        json.pointer("/data/sourceMapUpload/file"),
        Some(&serde_json::Value::String("app.min.js".to_string()))
    );

    // The exact issue-detail path resolves frames through the stored map.
    let detail = execute(
        &build_schema(),
        &context,
        graphql(
            r#"{ issue(service: "web", fingerprint: "fp-map") { events(limit: 5) { mappedFrames { raw file line column resolved source sourceLine sourceColumn name } } } }"#
                .to_string(),
        ),
    )
    .await;
    let json = serde_json::to_value(detail).unwrap();
    assert!(error_messages(&json).is_empty(), "detail: {json}");
    let frames = json
        .pointer("/data/issue/events/0/mappedFrames")
        .and_then(serde_json::Value::as_array)
        .expect("frames array");
    assert_eq!(frames.len(), 2);
    assert_eq!(frames[0]["resolved"], true);
    assert_eq!(frames[0]["source"], "src/app.ts");
    assert_eq!(frames[0]["sourceLine"], 3);
    assert_eq!(frames[0]["sourceColumn"], 2);
    assert_eq!(frames[0]["name"], "render");
    assert_eq!(frames[1]["resolved"], true);
    assert_eq!(frames[1]["sourceLine"], 1);
    assert_eq!(frames[1]["name"], "onClick");

    // Listing exposes metadata only: no map-content field exists.
    let list = execute(
        &build_schema(),
        &context,
        graphql(
            r#"{ sourceMaps(service: "web", version: "1.2.3") { service version file debugId uploadedAtNanos mapBytes mapSha256 } }"#
                .to_string(),
        ),
    )
    .await;
    let json = serde_json::to_value(list).unwrap();
    assert!(error_messages(&json).is_empty(), "list: {json}");
    let items = json
        .pointer("/data/sourceMaps")
        .and_then(serde_json::Value::as_array)
        .expect("list array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["version"], "1.2.3");
    assert!(
        items[0].get("mapJson").is_none(),
        "map content must stay private: {json}"
    );
    let sdl = crate::export_schema_sdl();
    assert!(
        !sdl.contains("mapJson"),
        "schema must not expose map content"
    );
}

#[tokio::test]
async fn frames_stay_unresolved_without_matching_artifact() {
    let store = Arc::new(MemoryStore::new());
    let context = context_with_memory(Arc::clone(&store)).await;
    seed_issue_with_stack(&store, &context, Some("9.9.9")).await;
    let detail = execute(
        &build_schema(),
        &context,
        graphql(
            r#"{ issue(service: "web", fingerprint: "fp-map") { events(limit: 5) { mappedFrames { resolved source } } } }"#
                .to_string(),
        ),
    )
    .await;
    let json = serde_json::to_value(detail).unwrap();
    assert!(error_messages(&json).is_empty(), "detail: {json}");
    let frames = json
        .pointer("/data/issue/events/0/mappedFrames")
        .and_then(serde_json::Value::as_array)
        .expect("frames array");
    assert_eq!(frames.len(), 2, "frames parse even when unmapped");
    assert!(frames.iter().all(|frame| frame["resolved"] == false));
}

#[tokio::test]
async fn upload_rejects_malformed_maps_and_blank_identity() {
    let store = Arc::new(MemoryStore::new());
    let context = context_with_memory(Arc::clone(&store)).await;
    for (name, service, version, file, map) in [
        ("blank service", "", "1.0", "a.js", r#"{"version":3}"#),
        ("blank version", "web", "", "a.js", r#"{"version":3}"#),
        ("blank file", "web", "1.0", "", r#"{"version":3}"#),
        ("not json", "web", "1.0", "a.js", "{nope"),
        ("wrong version", "web", "1.0", "a.js", r#"{"version":2}"#),
        (
            "no mappings",
            "web",
            "1.0",
            "a.js",
            r#"{"version":3,"sources":["a"]}"#,
        ),
    ] {
        let escaped = serde_json::to_string(&map).expect("escape");
        let upload = execute(
            &build_schema(),
            &context,
            graphql(format!(
                r#"mutation {{ sourceMapUpload(service: "{service}", version: "{version}", file: "{file}", map: {escaped}) {{ file }} }}"#
            )),
        )
        .await;
        let json = serde_json::to_value(upload).unwrap();
        assert!(
            !error_messages(&json).is_empty(),
            "{name} must be rejected: {json}"
        );
    }
    // Nothing was stored.
    let stored = context.metadata.source_maps("web", "1.0").await.unwrap();
    assert!(stored.is_empty());
}
