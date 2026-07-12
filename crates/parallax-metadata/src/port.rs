use crate::TursoMetadataStore;
use parallax_model::{
    Dashboard, Investigation, Issue, IssueOccurrence, IssueQuery, IssueSortKey, RunRecord,
    SavedView, TrendPoint,
};

#[async_trait::async_trait]
impl parallax_storage::metadata::MetadataStore for TursoMetadataStore {
    async fn upsert_issue_occurrence(&self, value: &IssueOccurrence<'_>) -> anyhow::Result<()> {
        Self::upsert_issue_occurrence(self, value).await
    }
    async fn upsert_issue_occurrences(&self, values: &[IssueOccurrence<'_>]) -> anyhow::Result<()> {
        Self::upsert_issue_occurrences(self, values).await
    }
    async fn issue_trend(
        &self,
        id: &str,
        since: u128,
        step: u32,
    ) -> anyhow::Result<Vec<TrendPoint>> {
        Self::issue_trend(self, id, since, step).await
    }
    async fn issues(&self, limit: usize) -> anyhow::Result<Vec<Issue>> {
        Self::issues(self, limit).await
    }
    async fn issue(&self, id: &str) -> anyhow::Result<Option<Issue>> {
        Self::issue(self, id).await
    }
    async fn issues_by_fingerprints(&self, ids: &[String]) -> anyhow::Result<Vec<Issue>> {
        Self::issues_by_fingerprints(self, ids).await
    }
    async fn issues_filtered(
        &self,
        filter: &IssueQuery,
        sort: IssueSortKey,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<(Vec<Issue>, usize)> {
        Self::issues_filtered(self, filter, sort, limit, offset).await
    }
    async fn set_issue_status(&self, id: &str, status: &str) -> anyhow::Result<()> {
        Self::set_issue_status(self, id, status).await
    }
    async fn start_run(
        &self,
        id: &str,
        command: Option<&str>,
        started: u128,
    ) -> anyhow::Result<()> {
        Self::start_run(self, id, command, started).await
    }
    async fn finish_run(&self, id: &str, ended: u128, code: i32) -> anyhow::Result<()> {
        Self::finish_run(self, id, ended, code).await
    }
    async fn runs(&self, limit: usize) -> anyhow::Result<Vec<RunRecord>> {
        Self::runs(self, limit).await
    }
    async fn run(&self, id: &str) -> anyhow::Result<Option<RunRecord>> {
        Self::run(self, id).await
    }
    async fn ensure_run(&self, id: &str, first_seen: u128) -> anyhow::Result<()> {
        Self::ensure_run(self, id, first_seen).await
    }
    async fn dashboard_save(
        &self,
        id: &str,
        name: &str,
        layout: &str,
        now: u128,
    ) -> anyhow::Result<()> {
        Self::dashboard_save(self, id, name, layout, now).await
    }
    async fn dashboard_delete(&self, id: &str) -> anyhow::Result<bool> {
        Self::dashboard_delete(self, id).await
    }
    async fn dashboards(&self) -> anyhow::Result<Vec<Dashboard>> {
        Self::dashboards(self).await
    }
    async fn dashboard(&self, id: &str) -> anyhow::Result<Option<Dashboard>> {
        Self::dashboard(self, id).await
    }
    async fn investigation_save(
        &self,
        id: &str,
        name: &str,
        state: &str,
        now: u128,
    ) -> anyhow::Result<()> {
        Self::investigation_save(self, id, name, state, now).await
    }
    async fn investigation_delete(&self, id: &str) -> anyhow::Result<bool> {
        Self::investigation_delete(self, id).await
    }
    async fn investigations(&self) -> anyhow::Result<Vec<Investigation>> {
        Self::investigations(self).await
    }
    async fn investigation(&self, id: &str) -> anyhow::Result<Option<Investigation>> {
        Self::investigation(self, id).await
    }
    async fn saved_view_save(
        &self,
        id: &str,
        name: &str,
        page: &str,
        state: &str,
        now: u128,
    ) -> anyhow::Result<()> {
        Self::saved_view_save(self, id, name, page, state, now).await
    }
    async fn saved_view_delete(&self, id: &str) -> anyhow::Result<bool> {
        Self::saved_view_delete(self, id).await
    }
    async fn saved_views(&self, page: Option<&str>) -> anyhow::Result<Vec<SavedView>> {
        Self::saved_views(self, page).await
    }
    async fn saved_view(&self, id: &str) -> anyhow::Result<Option<SavedView>> {
        Self::saved_view(self, id).await
    }
}
