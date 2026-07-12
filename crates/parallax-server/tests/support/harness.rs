use parallax_metadata::TursoMetadataStore;
use parallax_server::Config;
use parallax_server::serve::ServerHandle;
use parallax_test_support::builders::MemoryStore;
use std::sync::Arc;

pub(crate) async fn start(config: &Config) -> anyhow::Result<ServerHandle> {
    let store = Arc::new(MemoryStore::new().with_normalizers(
        Arc::new(parallax_ingest::normalize_traces),
        Arc::new(parallax_ingest::normalize_logs),
    ));
    let metadata = Arc::new(TursoMetadataStore::open(config.data_dir().join("meta.db")).await?);
    parallax_server::serve::start_with_capabilities(config, store, metadata).await
}
