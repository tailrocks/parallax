//! Query-neutral mutable metadata capability.

use async_trait::async_trait;
use parallax_model::{
    Dashboard, Investigation, Issue, IssueQuery, IssueSortKey, RunRecord, SavedView, TrendPoint,
};

pub use parallax_model::IssueOccurrence;

#[async_trait]
pub trait MetadataStore: Send + Sync {
    async fn upsert_issue_occurrence(&self, occurrence: &IssueOccurrence<'_>)
    -> anyhow::Result<()>;
    async fn upsert_issue_occurrences(
        &self,
        occurrences: &[IssueOccurrence<'_>],
    ) -> anyhow::Result<()>;
    async fn issue_trend(
        &self,
        fingerprint: &str,
        since_nanos: u128,
        step_seconds: u32,
    ) -> anyhow::Result<Vec<TrendPoint>>;
    async fn issues(&self, limit: usize) -> anyhow::Result<Vec<Issue>>;
    async fn issue(&self, fingerprint: &str) -> anyhow::Result<Option<Issue>>;
    async fn issues_by_fingerprints(&self, fingerprints: &[String]) -> anyhow::Result<Vec<Issue>>;
    async fn issues_filtered(
        &self,
        filter: &IssueQuery,
        sort: IssueSortKey,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<(Vec<Issue>, usize)>;
    async fn set_issue_status(&self, fingerprint: &str, status: &str) -> anyhow::Result<()>;
    async fn start_run(
        &self,
        run_id: &str,
        command: Option<&str>,
        started_at_nanos: u128,
    ) -> anyhow::Result<()>;
    async fn finish_run(
        &self,
        run_id: &str,
        ended_at_nanos: u128,
        exit_code: i32,
    ) -> anyhow::Result<()>;
    async fn runs(&self, limit: usize) -> anyhow::Result<Vec<RunRecord>>;
    async fn run(&self, run_id: &str) -> anyhow::Result<Option<RunRecord>>;
    async fn ensure_run(&self, run_id: &str, first_seen_nanos: u128) -> anyhow::Result<()>;
    async fn dashboard_save(
        &self,
        id: &str,
        name: &str,
        layout: &str,
        now_nanos: u128,
    ) -> anyhow::Result<()>;
    async fn dashboard_delete(&self, id: &str) -> anyhow::Result<bool>;
    async fn dashboards(&self) -> anyhow::Result<Vec<Dashboard>>;
    async fn dashboard(&self, id: &str) -> anyhow::Result<Option<Dashboard>>;
    async fn investigation_save(
        &self,
        id: &str,
        name: &str,
        state: &str,
        now_nanos: u128,
    ) -> anyhow::Result<()>;
    async fn investigation_delete(&self, id: &str) -> anyhow::Result<bool>;
    async fn investigations(&self) -> anyhow::Result<Vec<Investigation>>;
    async fn investigation(&self, id: &str) -> anyhow::Result<Option<Investigation>>;
    async fn saved_view_save(
        &self,
        id: &str,
        name: &str,
        page: &str,
        state: &str,
        now_nanos: u128,
    ) -> anyhow::Result<()>;
    async fn saved_view_delete(&self, id: &str) -> anyhow::Result<bool>;
    async fn saved_views(&self, page: Option<&str>) -> anyhow::Result<Vec<SavedView>>;
    async fn saved_view(&self, id: &str) -> anyhow::Result<Option<SavedView>>;
}
