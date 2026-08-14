//! Deterministic seed/reset for browser product contracts.

use std::sync::Arc;

use anyhow::{Context, Result, bail};
use parallax_metadata::TursoMetadataStore;
use parallax_storage::metadata::MetadataStore;

use super::datasets::{DatasetId, ScenarioManifest, manifest_for};
use super::seed_builders::{
    seed_alerts_pilot, seed_dashboards_pilot, seed_investigations_pilot, seed_logs_pilot,
    seed_metrics_pilot, seed_sql_pilot, seed_traces_pilot,
};
use crate::builders::MemoryStore;

/// Wipe Turso-like metadata tables used by browser contracts.
pub async fn clear_metadata(metadata: &dyn MetadataStore) -> Result<()> {
    for investigation in metadata
        .investigations()
        .await
        .context("list investigations for clear")?
    {
        metadata
            .investigation_delete(&investigation.id)
            .await
            .with_context(|| format!("delete investigation {}", investigation.id))?;
    }
    for dashboard in metadata
        .dashboards()
        .await
        .context("list dashboards for clear")?
    {
        metadata
            .dashboard_delete(&dashboard.id)
            .await
            .with_context(|| format!("delete dashboard {}", dashboard.id))?;
    }
    for view in metadata
        .saved_views(None)
        .await
        .context("list saved views for clear")?
    {
        metadata
            .saved_view_delete(&view.id)
            .await
            .with_context(|| format!("delete saved view {}", view.id))?;
    }
    Ok(())
}

/// Reset telemetry + metadata and seed the requested dataset.
pub async fn reset_and_seed(
    store: &MemoryStore,
    metadata: Arc<TursoMetadataStore>,
    dataset: DatasetId,
) -> Result<ScenarioManifest> {
    store.clear();
    clear_metadata(metadata.as_ref()).await?;
    metadata.alert_reset().await;
    seed_dataset(store, metadata.as_ref(), dataset).await
}

/// Seed without clearing first (tests that compose seeds deliberately).
pub async fn seed_dataset(
    store: &MemoryStore,
    metadata: &TursoMetadataStore,
    dataset: DatasetId,
) -> Result<ScenarioManifest> {
    let manifest = manifest_for(dataset);
    match dataset {
        DatasetId::ShellEmpty => {}
        DatasetId::InvestigationsPilot => seed_investigations_pilot(store, metadata).await?,
        DatasetId::LogsPilot => seed_logs_pilot(store),
        DatasetId::TracesPilot => seed_traces_pilot(store),
        DatasetId::DashboardsPilot => seed_dashboards_pilot(store, metadata).await?,
        DatasetId::SqlPilot => seed_sql_pilot(store),
        DatasetId::AlertsPilot => seed_alerts_pilot(metadata).await?,
        DatasetId::MetricsPilot => seed_metrics_pilot(store),
    }
    Ok(manifest)
}

/// Typed investigation snapshot for postcondition checks.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InvestigationSnapshot {
    pub id: String,
    pub name: String,
    pub state: String,
}

pub async fn investigation_snapshot(
    metadata: &dyn MetadataStore,
) -> Result<Vec<InvestigationSnapshot>> {
    let rows = metadata
        .investigations()
        .await
        .context("list investigations")?;
    Ok(rows
        .into_iter()
        .map(|row| InvestigationSnapshot {
            id: row.id,
            name: row.name,
            state: row.state,
        })
        .collect())
}

/// Verify postconditions for a seeded (or mutated) dataset identity.
pub async fn postconditions_hold(
    store: &MemoryStore,
    metadata: &dyn MetadataStore,
    dataset: DatasetId,
) -> Result<()> {
    let expected = manifest_for(dataset);
    let (spans, logs, metrics, _errors) = store.counts();
    if spans != expected.span_count {
        bail!(
            "dataset {dataset}: span_count {spans} != expected {}",
            expected.span_count
        );
    }
    if logs != expected.log_count {
        bail!(
            "dataset {dataset}: log_count {logs} != expected {}",
            expected.log_count
        );
    }
    if metrics != expected.metric_count {
        bail!(
            "dataset {dataset}: metric_count {metrics} != expected {}",
            expected.metric_count
        );
    }
    let investigations = investigation_snapshot(metadata).await?;
    let names: Vec<_> = investigations.iter().map(|row| row.name.clone()).collect();
    if names != expected.expected_investigation_names {
        bail!(
            "dataset {dataset}: investigation names {names:?} != expected {:?}",
            expected.expected_investigation_names
        );
    }
    let ids: Vec<_> = investigations.iter().map(|row| row.id.clone()).collect();
    if ids != expected.investigation_ids {
        bail!(
            "dataset {dataset}: investigation ids {ids:?} != expected {:?}",
            expected.investigation_ids
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::datasets::ANCHOR_TS_NANOS;
    use super::*;
    use crate::builders::span;
    use parallax_metadata::TursoMetadataStore;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    async fn temp_metadata() -> Arc<TursoMetadataStore> {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "parallax-browser-seed-{}-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        drop(std::fs::remove_file(&path));
        Arc::new(TursoMetadataStore::open(&path).await.expect("open turso"))
    }

    #[tokio::test]
    async fn every_dataset_meets_postconditions() {
        let store = MemoryStore::new();
        let metadata = temp_metadata().await;
        for dataset in crate::browser::dataset_ids() {
            reset_and_seed(&store, metadata.clone(), dataset)
                .await
                .unwrap_or_else(|error| panic!("{dataset}: {error:#}"));
            postconditions_hold(&store, metadata.as_ref(), dataset)
                .await
                .unwrap_or_else(|error| panic!("{dataset} post: {error:#}"));
        }
    }

    #[tokio::test]
    async fn same_seed_is_deterministic() {
        let store = MemoryStore::new();
        let metadata = temp_metadata().await;
        let first = reset_and_seed(&store, metadata.clone(), DatasetId::InvestigationsPilot)
            .await
            .expect("seed");
        let snap_a = investigation_snapshot(metadata.as_ref())
            .await
            .expect("snap a");
        let second = reset_and_seed(&store, metadata.clone(), DatasetId::InvestigationsPilot)
            .await
            .expect("reseed");
        let snap_b = investigation_snapshot(metadata.as_ref())
            .await
            .expect("snap b");
        assert_eq!(first, second);
        assert_eq!(snap_a, snap_b);
        postconditions_hold(&store, metadata.as_ref(), DatasetId::InvestigationsPilot)
            .await
            .expect("post");
    }

    #[tokio::test]
    async fn different_datasets_isolate() {
        let store = MemoryStore::new();
        let metadata = temp_metadata().await;
        reset_and_seed(&store, metadata.clone(), DatasetId::InvestigationsPilot)
            .await
            .expect("pilot");
        assert_eq!(store.counts().0, 1);
        reset_and_seed(&store, metadata.clone(), DatasetId::ShellEmpty)
            .await
            .expect("empty");
        assert_eq!(store.counts().0, 0);
        postconditions_hold(&store, metadata.as_ref(), DatasetId::ShellEmpty)
            .await
            .expect("empty post");
    }

    #[tokio::test]
    async fn reset_after_mutation_clears_extra_rows() {
        let store = MemoryStore::new();
        let metadata = temp_metadata().await;
        reset_and_seed(&store, metadata.clone(), DatasetId::ShellEmpty)
            .await
            .expect("empty");
        metadata
            .investigation_save("extra", "Extra", "{}", ANCHOR_TS_NANOS)
            .await
            .expect("mutate");
        store.push_spans(vec![span(
            "noise",
            "cccccccccccccccccccccccccccccccc",
            "dddddddddddddddd",
            ANCHOR_TS_NANOS,
            1,
        )]);
        reset_and_seed(&store, metadata.clone(), DatasetId::ShellEmpty)
            .await
            .expect("reset");
        postconditions_hold(&store, metadata.as_ref(), DatasetId::ShellEmpty)
            .await
            .expect("clean");
    }
}
