use crate::TursoMetadataStore;
use parallax_model::{
    Dashboard, Investigation, Issue, IssueOccurrence, IssueQuery, IssueSortKey, RunRecord,
    SavedView, TrendPoint,
};
use parallax_storage::metadata::{MetadataError, MetadataResult};

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
    async fn set_issue_status(&self, id: &str, status: &str) -> MetadataResult<()> {
        Self::set_issue_status(self, id, status)
            .await
            .map_err(MetadataError::internal)
    }
    async fn start_run(
        &self,
        id: &str,
        command: Option<&str>,
        started: u128,
    ) -> MetadataResult<()> {
        Self::start_run(self, id, command, started)
            .await
            .map_err(MetadataError::internal)
    }
    async fn finish_run(&self, id: &str, ended: u128, code: i32) -> MetadataResult<()> {
        Self::finish_run(self, id, ended, code)
            .await
            .map_err(MetadataError::internal)
    }
    async fn runs(&self, limit: usize) -> MetadataResult<Vec<RunRecord>> {
        Self::runs(self, limit)
            .await
            .map_err(MetadataError::internal)
    }
    async fn run(&self, id: &str) -> MetadataResult<Option<RunRecord>> {
        Self::run(self, id).await.map_err(MetadataError::internal)
    }
    async fn ensure_run(&self, id: &str, first_seen: u128) -> MetadataResult<()> {
        Self::ensure_run(self, id, first_seen)
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
