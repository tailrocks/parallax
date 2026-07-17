use crate::TursoMetadataStore;
use parallax_model::{
    Dashboard, Investigation, InvocationRecord, Issue, IssueOccurrence, IssueQuery, IssueSortKey,
    SavedView, TrendPoint,
};
use parallax_storage::metadata::{MetadataError, MetadataResult};
use parallax_storage::{
    MetadataPruneJournalStore, MetadataPruneStore, PruneItem, PruneJournal, PrunePlan,
    PrunePlanLimits, PruneStepStart,
};

#[async_trait::async_trait]
impl MetadataPruneJournalStore for TursoMetadataStore {
    async fn create_prune_journal(
        &self,
        plan: &PrunePlan,
        now_nanos: u128,
        limits: PrunePlanLimits,
    ) -> MetadataResult<PruneJournal> {
        Self::create_prune_journal(self, plan, now_nanos, limits)
            .await
            .map_err(MetadataError::internal)
    }

    async fn prune_journal(
        &self,
        plan_id: &str,
        limits: PrunePlanLimits,
    ) -> MetadataResult<Option<PruneJournal>> {
        Self::prune_journal(self, plan_id, limits)
            .await
            .map_err(MetadataError::internal)
    }

    async fn begin_prune_step(
        &self,
        plan_id: &str,
        step_index: u32,
        now_nanos: u128,
    ) -> MetadataResult<PruneStepStart> {
        Self::begin_prune_step(self, plan_id, step_index, now_nanos)
            .await
            .map_err(MetadataError::internal)
    }

    async fn record_prune_step_failure(
        &self,
        plan_id: &str,
        step_index: u32,
        error: &str,
        now_nanos: u128,
    ) -> MetadataResult<()> {
        Self::record_prune_step_failure(self, plan_id, step_index, error, now_nanos)
            .await
            .map_err(MetadataError::internal)
    }

    async fn complete_prune_step(
        &self,
        plan_id: &str,
        step_index: u32,
        now_nanos: u128,
    ) -> MetadataResult<()> {
        Self::complete_prune_step(self, plan_id, step_index, now_nanos)
            .await
            .map_err(MetadataError::internal)
    }
}

#[async_trait::async_trait]
impl MetadataPruneStore for TursoMetadataStore {
    async fn retained_alert_prune_items(
        &self,
        cutoff_nanos: u128,
    ) -> MetadataResult<Vec<PruneItem>> {
        Self::retained_alert_prune_items(self, cutoff_nanos)
            .await
            .map_err(MetadataError::internal)
    }

    async fn retained_saved_state_prune_items(
        &self,
        cutoff_nanos: u128,
    ) -> MetadataResult<Vec<PruneItem>> {
        Self::retained_saved_state_prune_items(self, cutoff_nanos)
            .await
            .map_err(MetadataError::internal)
    }

    async fn issue_prune_item(&self, cutoff_nanos: u128) -> MetadataResult<PruneItem> {
        Self::issue_prune_item(self, cutoff_nanos)
            .await
            .map_err(MetadataError::internal)
    }

    async fn issue_dependent_prune_items(
        &self,
        cutoff_nanos: u128,
    ) -> MetadataResult<Vec<PruneItem>> {
        Self::issue_dependent_prune_items(self, cutoff_nanos)
            .await
            .map_err(MetadataError::internal)
    }

    async fn invocation_prune_item(&self, cutoff_nanos: u128) -> MetadataResult<PruneItem> {
        Self::invocation_prune_item(self, cutoff_nanos)
            .await
            .map_err(MetadataError::internal)
    }
}

#[async_trait::async_trait]
impl parallax_storage::metadata::MetadataStore for TursoMetadataStore {
    async fn upsert_issue_occurrence(&self, value: &IssueOccurrence<'_>) -> MetadataResult<()> {
        Self::upsert_issue_occurrence(self, value)
            .await
            .map_err(MetadataError::internal)
    }
    async fn upsert_issue_occurrences(&self, values: &[IssueOccurrence<'_>]) -> MetadataResult<()> {
        Self::upsert_issue_occurrences(self, values)
            .await
            .map_err(MetadataError::internal)
    }
    async fn issue_trend(
        &self,
        id: &str,
        since: u128,
        step: u32,
    ) -> MetadataResult<Vec<TrendPoint>> {
        Self::issue_trend(self, id, since, step)
            .await
            .map_err(MetadataError::internal)
    }
    async fn issues(&self, limit: usize) -> MetadataResult<Vec<Issue>> {
        Self::issues(self, limit)
            .await
            .map_err(MetadataError::internal)
    }
    async fn issue(&self, id: &str) -> MetadataResult<Option<Issue>> {
        Self::issue(self, id).await.map_err(MetadataError::internal)
    }
    async fn issues_by_fingerprints(&self, ids: &[String]) -> MetadataResult<Vec<Issue>> {
        Self::issues_by_fingerprints(self, ids)
            .await
            .map_err(MetadataError::internal)
    }
    async fn issues_filtered(
        &self,
        filter: &IssueQuery,
        sort: IssueSortKey,
        limit: usize,
        offset: usize,
    ) -> MetadataResult<(Vec<Issue>, usize)> {
        Self::issues_filtered(self, filter, sort, limit, offset)
            .await
            .map_err(MetadataError::internal)
    }
    async fn set_issue_status(
        &self,
        id: &str,
        status: &str,
        changed_at_nanos: u128,
    ) -> MetadataResult<()> {
        Self::set_issue_status(self, id, status, changed_at_nanos)
            .await
            .map_err(MetadataError::internal)
    }
    async fn start_invocation(
        &self,
        id: &str,
        command: Option<&str>,
        app_mode: Option<&str>,
        started: u128,
    ) -> MetadataResult<()> {
        Self::start_invocation(self, id, command, app_mode, started)
            .await
            .map_err(MetadataError::internal)
    }
    async fn finish_invocation(
        &self,
        id: &str,
        ended: u128,
        code: i32,
        outcome: Option<&str>,
    ) -> MetadataResult<()> {
        Self::finish_invocation(self, id, ended, code, outcome)
            .await
            .map_err(MetadataError::internal)
    }
    async fn invocations(&self, limit: usize) -> MetadataResult<Vec<InvocationRecord>> {
        Self::invocations(self, limit)
            .await
            .map_err(MetadataError::internal)
    }
    async fn invocation(&self, id: &str) -> MetadataResult<Option<InvocationRecord>> {
        Self::invocation(self, id)
            .await
            .map_err(MetadataError::internal)
    }
    async fn ensure_invocation(&self, id: &str, first_seen: u128) -> MetadataResult<()> {
        Self::ensure_invocation(self, id, first_seen)
            .await
            .map_err(MetadataError::internal)
    }
    async fn dashboard_save(
        &self,
        id: &str,
        name: &str,
        layout: &str,
        now: u128,
    ) -> MetadataResult<()> {
        Self::dashboard_save(self, id, name, layout, now)
            .await
            .map_err(MetadataError::internal)
    }
    async fn dashboard_delete(&self, id: &str) -> MetadataResult<bool> {
        Self::dashboard_delete(self, id)
            .await
            .map_err(MetadataError::internal)
    }
    async fn dashboards(&self) -> MetadataResult<Vec<Dashboard>> {
        Self::dashboards(self)
            .await
            .map_err(MetadataError::internal)
    }
    async fn dashboard(&self, id: &str) -> MetadataResult<Option<Dashboard>> {
        Self::dashboard(self, id)
            .await
            .map_err(MetadataError::internal)
    }
    async fn investigation_save(
        &self,
        id: &str,
        name: &str,
        state: &str,
        now: u128,
    ) -> MetadataResult<()> {
        Self::investigation_save(self, id, name, state, now)
            .await
            .map_err(MetadataError::internal)
    }
    async fn investigation_delete(&self, id: &str) -> MetadataResult<bool> {
        Self::investigation_delete(self, id)
            .await
            .map_err(MetadataError::internal)
    }
    async fn investigations(&self) -> MetadataResult<Vec<Investigation>> {
        Self::investigations(self)
            .await
            .map_err(MetadataError::internal)
    }
    async fn investigation(&self, id: &str) -> MetadataResult<Option<Investigation>> {
        Self::investigation(self, id)
            .await
            .map_err(MetadataError::internal)
    }
    async fn saved_view_save(
        &self,
        id: &str,
        name: &str,
        page: &str,
        state: &str,
        now: u128,
    ) -> MetadataResult<()> {
        Self::saved_view_save(self, id, name, page, state, now)
            .await
            .map_err(MetadataError::internal)
    }
    async fn saved_view_delete(&self, id: &str) -> MetadataResult<bool> {
        Self::saved_view_delete(self, id)
            .await
            .map_err(MetadataError::internal)
    }
    async fn saved_views(&self, page: Option<&str>) -> MetadataResult<Vec<SavedView>> {
        Self::saved_views(self, page)
            .await
            .map_err(MetadataError::internal)
    }
    async fn saved_view(&self, id: &str) -> MetadataResult<Option<SavedView>> {
        Self::saved_view(self, id)
            .await
            .map_err(MetadataError::internal)
    }
}
