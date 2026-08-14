//! Incident-bundle column writes (plan 173). Kept off `alerts.rs` so the
//! anyhow-edge ratchet on that file does not grow.

use super::super::TursoMetadataStore;
use super::super::values::nanos_to_millis;
use crate::IncidentBundleSnapshot;

impl TursoMetadataStore {
    pub async fn alert_incident_set_bundle(
        &self,
        id: &str,
        snapshot: IncidentBundleSnapshot<'_>,
    ) -> anyhow::Result<()> {
        self.conn
            .lock()
            .await
            .execute(
                "UPDATE alert_incidents SET bundle_hash=?2, bundle_assembled_at=?3, bundle_top_hypothesis=?4, bundle_deploy_adjacency=?5, bundle_error=?6 WHERE id=?1",
                (
                    id,
                    snapshot.hash,
                    nanos_to_millis(snapshot.assembled_at_nanos),
                    snapshot.top_hypothesis,
                    snapshot.deploy_adjacency,
                    snapshot.error,
                ),
            )
            .await?;
        Ok(())
    }
}
