//! Query-neutral mutable metadata capability.

use async_trait::async_trait;
use parallax_model::{
    Dashboard, Investigation, Issue, IssueQuery, IssueSortKey, InvocationRecord, SavedView, TrendPoint,
};
use thiserror::Error;

pub use parallax_model::IssueOccurrence;

/// Stable failure classes exposed by mutable metadata capabilities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataErrorKind {
    InvalidInput,
    NotFound,
    Conflict,
    Unavailable,
    Timeout,
    Schema,
    Internal,
}

/// Typed mutable-metadata boundary error. Its display text is operator-facing;
/// API clients receive a sanitized mapping of [`MetadataErrorKind`].
#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("invalid metadata input: {0}")]
    InvalidInput(String),
    #[error("metadata record not found: {0}")]
    NotFound(String),
    #[error("metadata conflict: {0}")]
    Conflict(String),
    #[error("metadata store unavailable: {source}")]
    Unavailable {
        #[source]
        source: anyhow::Error,
    },
    #[error("metadata operation timed out: {source}")]
    Timeout {
        #[source]
        source: anyhow::Error,
    },
    #[error("metadata schema operation failed: {source}")]
    Schema {
        #[source]
        source: anyhow::Error,
    },
    #[error("metadata operation failed: {source}")]
    Internal {
        #[source]
        source: anyhow::Error,
    },
}

impl MetadataError {
    #[must_use]
    pub fn kind(&self) -> MetadataErrorKind {
        match self {
            Self::InvalidInput(_) => MetadataErrorKind::InvalidInput,
            Self::NotFound(_) => MetadataErrorKind::NotFound,
            Self::Conflict(_) => MetadataErrorKind::Conflict,
            Self::Unavailable { .. } => MetadataErrorKind::Unavailable,
            Self::Timeout { .. } => MetadataErrorKind::Timeout,
            Self::Schema { .. } => MetadataErrorKind::Schema,
            Self::Internal { .. } => MetadataErrorKind::Internal,
        }
    }

    pub fn internal(source: impl Into<anyhow::Error>) -> Self {
        Self::Internal {
            source: source.into(),
        }
    }
}

pub type MetadataResult<T> = Result<T, MetadataError>;

#[async_trait]
pub trait MetadataStore: Send + Sync {
    async fn upsert_issue_occurrence(&self, occurrence: &IssueOccurrence<'_>)
    -> MetadataResult<()>;
    async fn upsert_issue_occurrences(
        &self,
        occurrences: &[IssueOccurrence<'_>],
    ) -> MetadataResult<()>;
    async fn issue_trend(
        &self,
        fingerprint: &str,
        since_nanos: u128,
        step_seconds: u32,
    ) -> MetadataResult<Vec<TrendPoint>>;
    async fn issues(&self, limit: usize) -> MetadataResult<Vec<Issue>>;
    async fn issue(&self, fingerprint: &str) -> MetadataResult<Option<Issue>>;
    async fn issues_by_fingerprints(&self, fingerprints: &[String]) -> MetadataResult<Vec<Issue>>;
    async fn issues_filtered(
        &self,
        filter: &IssueQuery,
        sort: IssueSortKey,
        limit: usize,
        offset: usize,
    ) -> MetadataResult<(Vec<Issue>, usize)>;
    async fn set_issue_status(&self, fingerprint: &str, status: &str) -> MetadataResult<()>;
    async fn start_invocation(
        &self,
        invocation_id: &str,
        command: Option<&str>,
        app_mode: Option<&str>,
        started_at_nanos: u128,
    ) -> MetadataResult<()>;
    async fn finish_invocation(
        &self,
        invocation_id: &str,
        ended_at_nanos: u128,
        exit_code: i32,
        outcome: Option<&str>,
    ) -> MetadataResult<()>;
    async fn invocations(&self, limit: usize) -> MetadataResult<Vec<InvocationRecord>>;
    async fn invocation(&self, invocation_id: &str)
    -> MetadataResult<Option<InvocationRecord>>;
    async fn ensure_invocation(
        &self,
        invocation_id: &str,
        first_seen_nanos: u128,
    ) -> MetadataResult<()>;
    async fn dashboard_save(
        &self,
        id: &str,
        name: &str,
        layout: &str,
        now_nanos: u128,
    ) -> MetadataResult<()>;
    async fn dashboard_delete(&self, id: &str) -> MetadataResult<bool>;
    async fn dashboards(&self) -> MetadataResult<Vec<Dashboard>>;
    async fn dashboard(&self, id: &str) -> MetadataResult<Option<Dashboard>>;
    async fn investigation_save(
        &self,
        id: &str,
        name: &str,
        state: &str,
        now_nanos: u128,
    ) -> MetadataResult<()>;
    async fn investigation_delete(&self, id: &str) -> MetadataResult<bool>;
    async fn investigations(&self) -> MetadataResult<Vec<Investigation>>;
    async fn investigation(&self, id: &str) -> MetadataResult<Option<Investigation>>;
    async fn saved_view_save(
        &self,
        id: &str,
        name: &str,
        page: &str,
        state: &str,
        now_nanos: u128,
    ) -> MetadataResult<()>;
    async fn saved_view_delete(&self, id: &str) -> MetadataResult<bool>;
    async fn saved_views(&self, page: Option<&str>) -> MetadataResult<Vec<SavedView>>;
    async fn saved_view(&self, id: &str) -> MetadataResult<Option<SavedView>>;
}

#[cfg(test)]
mod tests;
