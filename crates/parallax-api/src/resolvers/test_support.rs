//! Shared test helpers for resolver domain tests.

use crate::{ApiContext, RequestMemo};
use parallax_storage::metadata::MetadataStore;
use parallax_storage::model::{LogRow, SpanRow};
use parallax_test_support::MemoryStore;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) static TEST_DB_SEQ: AtomicU64 = AtomicU64::new(0);

pub(crate) fn span(
    service: &str,
    trace_id: &str,
    span_id: &str,
    ts_nanos: u128,
    duration_ns: u128,
) -> SpanRow {
    SpanRow {
        ts_nanos,
        service: service.into(),
        trace_id: trace_id.into(),
        span_id: span_id.into(),
        parent_span_id: None,
        name: "handler".into(),
        kind: "SPAN_KIND_SERVER".into(),
        status_code: "STATUS_CODE_UNSET".into(),
        status_message: String::new(),
        duration_ns,
        run_id: None,
        scope_name: String::new(),
        events: None,
        links: serde_json::Value::Null,
        attributes: serde_json::Value::Null,
        resource: serde_json::Value::Null,
    }
}

pub(crate) fn log_row(service: &str, trace_id: &str, ts_nanos: u128, body: &str) -> LogRow {
    LogRow {
        ts_nanos,
        event_name: String::new(),
        observed_ts_nanos: 0,
        service: service.into(),
        severity_num: 9,
        severity_text: "INFO".into(),
        body: body.into(),
        trace_id: trace_id.into(),
        span_id: format!("span-{ts_nanos}"),
        run_id: None,
        scope_name: String::new(),
        attributes: serde_json::Value::Null,
        resource: serde_json::Value::Null,
    }
}

pub(crate) fn span_with_release(
    service: &str,
    trace_id: &str,
    span_id: &str,
    ts_nanos: u128,
    version: &str,
) -> SpanRow {
    let mut row = span(service, trace_id, span_id, ts_nanos, 1_000);
    row.resource = serde_json::json!({ "service.version": version });
    row
}

pub(crate) async fn context_with_memory(store: Arc<MemoryStore>) -> ApiContext {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "parallax-api-test-{}-{}-{}.db",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        TEST_DB_SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    match std::fs::remove_file(&path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("remove stale test metadata {}: {error}", path.display()),
    }
    let metadata = MetadataStore::open(&path).await.unwrap();
    ApiContext {
        store,
        metadata: Arc::new(metadata),
        otlp_grpc_port: 4317,
        memo: RequestMemo::default(),
    }
}

pub(crate) fn error_messages(json: &serde_json::Value) -> Vec<String> {
    json.pointer("/errors")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|error| error.get("message").and_then(|message| message.as_str()))
        .map(str::to_string)
        .collect()
}
