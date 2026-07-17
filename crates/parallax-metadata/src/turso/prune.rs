//! Bounded Turso prune candidate discovery.

use super::*;
use parallax_storage::{
    PruneClass, PruneEstimate, PruneExclusion, PruneExclusionKind, PruneItem, PruneStore,
};

impl TursoMetadataStore {
    pub async fn issue_dependent_prune_items(
        &self,
        cutoff_nanos: u128,
    ) -> anyhow::Result<Vec<PruneItem>> {
        let cutoff_millis = nanos_to_millis(cutoff_nanos);
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT
                   (SELECT COUNT(*) FROM issue_buckets b JOIN issues i USING (fingerprint)
                    WHERE i.status = 'resolved' AND i.resolved_at IS NOT NULL AND i.resolved_at <= ?1),
                   (SELECT COUNT(*) FROM issue_occurrences o JOIN issues i USING (fingerprint)
                    WHERE i.status = 'resolved' AND i.resolved_at IS NOT NULL AND i.resolved_at <= ?1)",
                [Value::Integer(cutoff_millis)],
            )
            .await?;
        let row = rows
            .next()
            .await?
            .ok_or_else(|| anyhow::anyhow!("issue dependent prune aggregate returned no row"))?;
        let buckets = u64::try_from(integer(&row, 0)).unwrap_or(0);
        let occurrences = u64::try_from(integer(&row, 1)).unwrap_or(0);
        Ok(vec![
            issue_dependent_item(
                PruneClass::IssueBuckets,
                "issue_buckets",
                cutoff_nanos,
                buckets,
            ),
            issue_dependent_item(
                PruneClass::IssueOccurrences,
                "issue_occurrences",
                cutoff_nanos,
                occurrences,
            ),
        ])
    }

    pub async fn issue_prune_item(&self, cutoff_nanos: u128) -> anyhow::Result<PruneItem> {
        let cutoff_millis = nanos_to_millis(cutoff_nanos);
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT
                   COALESCE(SUM(CASE WHEN status = 'resolved' AND resolved_at IS NOT NULL AND resolved_at <= ?1 THEN 1 ELSE 0 END), 0),
                   COALESCE(SUM(CASE WHEN status != 'resolved' OR resolved_at IS NULL THEN 1 ELSE 0 END), 0),
                   COALESCE(SUM(CASE WHEN status = 'resolved' AND resolved_at IS NOT NULL AND resolved_at > ?1 THEN 1 ELSE 0 END), 0)
                 FROM issues",
                [Value::Integer(cutoff_millis)],
            )
            .await?;
        let row = rows
            .next()
            .await?
            .ok_or_else(|| anyhow::anyhow!("issue prune aggregate returned no row"))?;
        let eligible = u64::try_from(integer(&row, 0)).unwrap_or(0);
        let unresolved = u64::try_from(integer(&row, 1)).unwrap_or(0);
        let not_expired = u64::try_from(integer(&row, 2)).unwrap_or(0);
        let mut exclusions = Vec::new();
        if unresolved > 0 {
            exclusions.push(PruneExclusion {
                kind: PruneExclusionKind::Unresolved,
                count: unresolved,
            });
        }
        if not_expired > 0 {
            exclusions.push(PruneExclusion {
                kind: PruneExclusionKind::NotExpired,
                count: not_expired,
            });
        }
        Ok(PruneItem {
            store: PruneStore::Turso,
            class: PruneClass::Issues,
            target: "issues".to_string(),
            cutoff_nanos,
            estimate: PruneEstimate {
                rows: Some(eligible),
                objects: None,
                bytes: None,
            },
            exclusions,
            warnings: Vec::new(),
        })
    }

    pub async fn invocation_prune_item(&self, cutoff_nanos: u128) -> anyhow::Result<PruneItem> {
        let cutoff_millis = nanos_to_millis(cutoff_nanos);
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT
                   COALESCE(SUM(CASE WHEN status = 'finished' AND ended_at IS NOT NULL AND ended_at <= ?1 THEN 1 ELSE 0 END), 0),
                   COALESCE(SUM(CASE WHEN status != 'finished' OR ended_at IS NULL THEN 1 ELSE 0 END), 0),
                   COALESCE(SUM(CASE WHEN status = 'finished' AND ended_at IS NOT NULL AND ended_at > ?1 THEN 1 ELSE 0 END), 0)
                 FROM invocations",
                [Value::Integer(cutoff_millis)],
            )
            .await?;
        let row = rows
            .next()
            .await?
            .ok_or_else(|| anyhow::anyhow!("invocation prune aggregate returned no row"))?;
        let eligible = u64::try_from(integer(&row, 0)).unwrap_or(0);
        let active = u64::try_from(integer(&row, 1)).unwrap_or(0);
        let not_expired = u64::try_from(integer(&row, 2)).unwrap_or(0);
        let mut exclusions = Vec::new();
        if active > 0 {
            exclusions.push(PruneExclusion {
                kind: PruneExclusionKind::Active,
                count: active,
            });
        }
        if not_expired > 0 {
            exclusions.push(PruneExclusion {
                kind: PruneExclusionKind::NotExpired,
                count: not_expired,
            });
        }
        Ok(PruneItem {
            store: PruneStore::Turso,
            class: PruneClass::Invocations,
            target: "invocations".to_string(),
            cutoff_nanos,
            estimate: PruneEstimate {
                rows: Some(eligible),
                objects: None,
                bytes: None,
            },
            exclusions,
            warnings: Vec::new(),
        })
    }
}

fn issue_dependent_item(
    class: PruneClass,
    target: &str,
    cutoff_nanos: u128,
    rows: u64,
) -> PruneItem {
    PruneItem {
        store: PruneStore::Turso,
        class,
        target: target.to_string(),
        cutoff_nanos,
        estimate: PruneEstimate {
            rows: Some(rows),
            objects: None,
            bytes: None,
        },
        exclusions: Vec::new(),
        warnings: vec!["deleted only through the eligible issue owner cascade".to_string()],
    }
}

#[cfg(test)]
mod tests;
