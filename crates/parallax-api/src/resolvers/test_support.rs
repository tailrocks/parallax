//! Shared test helpers for resolver domain tests.

use crate::{ApiContext, RequestMemo};
use parallax_metadata::TursoMetadataStore;
use parallax_test_support::builders::MemoryStore;
pub(crate) use parallax_test_support::builders::{log_row, span, span_with_release};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) static TEST_DB_SEQ: AtomicU64 = AtomicU64::new(0);

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
    let metadata = Arc::new(TursoMetadataStore::open(&path).await.unwrap());
    ApiContext {
        store,
        metadata: metadata.clone(),
        alerts: Some(metadata),
        otlp_grpc_port: 4317,
        otlp_http_port: 4318,
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
