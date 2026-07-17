use crate::TursoMetadataStore;
use parallax_model::{
    Dashboard, Investigation, InvocationRecord, Issue, IssueOccurrence, IssueQuery, IssueSortKey,
    SavedView, TestCaseRecord, TestExplorerPage, TestExplorerQuery, TestExplorerSort,
    TestFlakyStateRecord, TestResultRecord, TestVariantRecord, TrendPoint,
};
use parallax_storage::metadata::{
    MetadataError, MetadataResult, TEST_CASE_VARIANTS_MAX_LIMIT, TEST_EXPLORER_MAX_LIMIT,
    TEST_EXPLORER_MAX_OFFSET, TEST_VARIANT_RESULTS_MAX_LIMIT,
};
use parallax_storage::{
    MetadataPruneJournalStore, MetadataPruneStore, PruneItem, PruneJournal, PrunePlan,
    PrunePlanLimits, PruneStepStart,
};
use std::str::FromStr;

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

    async fn issue_prune_item(
        &self,
        cutoff_nanos: u128,
        protection_at_nanos: u128,
    ) -> MetadataResult<PruneItem> {
        Self::issue_prune_item(self, cutoff_nanos, protection_at_nanos)
            .await
            .map_err(MetadataError::internal)
    }

    async fn issue_dependent_prune_items(
        &self,
        cutoff_nanos: u128,
        protection_at_nanos: u128,
    ) -> MetadataResult<Vec<PruneItem>> {
        Self::issue_dependent_prune_items(self, cutoff_nanos, protection_at_nanos)
            .await
            .map_err(MetadataError::internal)
    }

    async fn invocation_prune_item(
        &self,
        cutoff_nanos: u128,
        protection_at_nanos: u128,
    ) -> MetadataResult<PruneItem> {
        Self::invocation_prune_item(self, cutoff_nanos, protection_at_nanos)
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
    async fn upsert_test_case(&self, record: &TestCaseRecord) -> MetadataResult<()> {
        Self::upsert_test_case(self, record)
            .await
            .map_err(MetadataError::internal)
    }
    async fn upsert_test_variant(&self, record: &TestVariantRecord) -> MetadataResult<()> {
        Self::upsert_test_variant(self, record)
            .await
            .map_err(MetadataError::internal)
    }
    async fn upsert_test_result(&self, record: &TestResultRecord) -> MetadataResult<()> {
        Self::upsert_test_result(self, record)
            .await
            .map_err(MetadataError::internal)
    }
    async fn upsert_test_flaky_state(&self, record: &TestFlakyStateRecord) -> MetadataResult<()> {
        Self::upsert_test_flaky_state(self, record)
            .await
            .map_err(MetadataError::internal)
    }
    async fn test_case(&self, key: &str) -> MetadataResult<Option<TestCaseRecord>> {
        Self::test_case(self, key)
            .await
            .map_err(MetadataError::internal)
    }
    async fn test_variant(&self, key: &str) -> MetadataResult<Option<TestVariantRecord>> {
        Self::test_variant(self, key)
            .await
            .map_err(MetadataError::internal)
    }
    async fn test_variants_for_case(
        &self,
        case_key: &str,
        limit: usize,
    ) -> MetadataResult<Vec<TestVariantRecord>> {
        parallax_model::TestCaseKey::from_str(case_key)
            .map_err(|_| MetadataError::InvalidInput("invalid test case key".into()))?;
        Self::test_variants_for_case(self, case_key, limit.min(TEST_CASE_VARIANTS_MAX_LIMIT))
            .await
            .map_err(MetadataError::internal)
    }
    async fn test_results_for_variant(
        &self,
        variant_key: &str,
        limit: usize,
    ) -> MetadataResult<Vec<TestResultRecord>> {
        parallax_model::TestVariantKey::from_str(variant_key)
            .map_err(|_| MetadataError::InvalidInput("invalid test variant key".into()))?;
        Self::test_results_for_variant(self, variant_key, limit.min(TEST_VARIANT_RESULTS_MAX_LIMIT))
            .await
            .map_err(MetadataError::internal)
    }
    async fn test_results_for_invocation(
        &self,
        invocation_id: &str,
        limit: usize,
    ) -> MetadataResult<Vec<TestResultRecord>> {
        Self::test_results_for_invocation(self, invocation_id, limit)
            .await
            .map_err(MetadataError::internal)
    }
    async fn test_flaky_state(
        &self,
        variant_key: &str,
    ) -> MetadataResult<Option<TestFlakyStateRecord>> {
        Self::test_flaky_state(self, variant_key)
            .await
            .map_err(MetadataError::internal)
    }
    async fn test_explorer(
        &self,
        query: &TestExplorerQuery,
        sort: TestExplorerSort,
        limit: usize,
        offset: usize,
    ) -> MetadataResult<TestExplorerPage> {
        validate_test_explorer_query(query)?;
        Self::test_explorer(
            self,
            query,
            sort,
            limit.min(TEST_EXPLORER_MAX_LIMIT),
            offset.min(TEST_EXPLORER_MAX_OFFSET),
        )
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

fn validate_test_explorer_query(query: &TestExplorerQuery) -> MetadataResult<()> {
    if query
        .from_nanos
        .zip(query.to_nanos)
        .is_some_and(|(from, to)| from > to)
    {
        return Err(MetadataError::InvalidInput(
            "test explorer time range is reversed".into(),
        ));
    }
    for value in [
        query.query.as_deref(),
        query.suite.as_deref(),
        query.service.as_deref(),
        query.service_version.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        if value.trim().is_empty() || value.len() > 256 {
            return Err(MetadataError::InvalidInput(
                "test explorer filter must be nonblank and at most 256 bytes".into(),
            ));
        }
    }
    if let Some(configuration) = &query.configuration
        && (!configuration.key.starts_with("test.configuration.")
            || configuration.key.len() > 256
            || configuration.value.len() > 256)
    {
        return Err(MetadataError::InvalidInput(
            "test configuration filter is invalid".into(),
        ));
    }
    Ok(())
}
