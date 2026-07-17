use super::PruneItem;
use crate::metadata::MetadataResult;

#[async_trait::async_trait]
pub trait MetadataPruneStore: Send + Sync {
    /// One bounded aggregate over invocation metadata. `cutoff_nanos` is the
    /// terminal-time eligibility boundary chosen by the immutable plan.
    async fn invocation_prune_item(&self, cutoff_nanos: u128) -> MetadataResult<PruneItem>;

    /// One bounded aggregate over issue metadata. `cutoff_nanos` is the
    /// persisted-resolution-time eligibility boundary chosen by the plan.
    async fn issue_prune_item(&self, cutoff_nanos: u128) -> MetadataResult<PruneItem>;
}
