use parallax_server::Config;
use parallax_server::serve::ServerHandle;
use parallax_storage::memory::MemoryStore;
use parallax_storage::metadata::MetadataStore;
use std::sync::Arc;

pub async fn start(config: &Config) -> anyhow::Result<ServerHandle> {
    let store = Arc::new(MemoryStore::new().with_normalizers(
        Arc::new(parallax_core::normalize::normalize_traces),
        Arc::new(parallax_core::normalize::normalize_logs),
    ));
    let metadata = Arc::new(MetadataStore::open(config.data_dir().join("meta.db")).await?);
    parallax_server::serve::start_with_capabilities(config, store, metadata).await
}
