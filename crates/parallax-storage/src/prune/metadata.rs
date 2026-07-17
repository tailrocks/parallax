use super::PruneItem;
use crate::metadata::MetadataResult;

#[async_trait::async_trait]
pub trait MetadataPruneStore: Send + Sync {
    /// Assemble every current Turso lifecycle class in deterministic order.
    async fn metadata_prune_items(&self, cutoff_nanos: u128) -> MetadataResult<Vec<PruneItem>> {
        let mut items = vec![
            self.issue_prune_item(cutoff_nanos).await?,
            self.invocation_prune_item(cutoff_nanos).await?,
        ];
        items.extend(self.issue_dependent_prune_items(cutoff_nanos).await?);
        items.extend(self.retained_saved_state_prune_items(cutoff_nanos).await?);
        items.extend(self.retained_alert_prune_items(cutoff_nanos).await?);
        items.sort_by(|left, right| {
            (left.class, left.target.as_str()).cmp(&(right.class, right.target.as_str()))
        });
        Ok(items)
    }

    /// One bounded aggregate over invocation metadata. `cutoff_nanos` is the
    /// terminal-time eligibility boundary chosen by the immutable plan.
    async fn invocation_prune_item(&self, cutoff_nanos: u128) -> MetadataResult<PruneItem>;

    /// One bounded aggregate over issue metadata. `cutoff_nanos` is the
    /// persisted-resolution-time eligibility boundary chosen by the plan.
    async fn issue_prune_item(&self, cutoff_nanos: u128) -> MetadataResult<PruneItem>;

    /// Bounded estimates for rows owned by issues and deleted only with their
    /// eligible owner cascade.
    async fn issue_dependent_prune_items(
        &self,
        cutoff_nanos: u128,
    ) -> MetadataResult<Vec<PruneItem>>;

    /// Bounded zero-eligibility items for user-owned saved state which normal
    /// prune must disclose but never select.
    async fn retained_saved_state_prune_items(
        &self,
        cutoff_nanos: u128,
    ) -> MetadataResult<Vec<PruneItem>>;

    /// Bounded zero-eligibility items for alert-owned configuration and audit
    /// state whose lifecycle is not normal prune.
    async fn retained_alert_prune_items(
        &self,
        cutoff_nanos: u128,
    ) -> MetadataResult<Vec<PruneItem>>;
}
