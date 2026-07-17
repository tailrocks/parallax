use parallax_metadata::TursoMetadataStore;
use parallax_server::Config;
use parallax_server::ServerHandle;
use parallax_storage::metadata::MetadataStore;
use parallax_test_support::builders::MemoryStore;
use std::sync::Arc;

pub(crate) async fn start(config: &Config) -> anyhow::Result<ServerHandle> {
    let store = Arc::new(MemoryStore::new().with_normalizers(
        Arc::new(parallax_ingest::normalize_traces),
        Arc::new(parallax_ingest::normalize_logs),
    ));
    let turso = Arc::new(TursoMetadataStore::open(config.data_dir().join("meta.db")).await?);
    let metadata: Arc<dyn MetadataStore> = turso.clone();
    // Pass concrete Turso as alerts so Sentry/GitHub ack ledgers work in harness.
    Ok(parallax_server::start_with_turso(config, store, metadata, Some(turso)).await?)
}
