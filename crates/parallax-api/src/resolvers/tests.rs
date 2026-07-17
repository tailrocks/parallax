//! GraphQL test-reporting explorer projections.

use crate::{ApiContext, clamp_limit, field_err, internal_field_err, nanos_string};
use juniper::{FieldResult, graphql_object};
use parallax_storage::metadata::{TEST_EXPLORER_MAX_LIMIT, TEST_EXPLORER_MAX_OFFSET};
use parallax_storage::model;
use std::str::FromStr;

const TEST_DETAIL_VARIANT_LIMIT: usize = 20;
const TEST_DETAIL_HISTORY_LIMIT: usize = 50;

#[derive(juniper::GraphQLEnum, Clone, Copy, Debug)]
pub(crate) enum TestRollup {
    Passed,
    FlakyPass,
    Failed,
    Broken,
    Skipped,
    Unknown,
}

impl From<TestRollup> for model::AttemptRollup {
    fn from(value: TestRollup) -> Self {
        match value {
            TestRollup::Passed => Self::Passed,
            TestRollup::FlakyPass => Self::FlakyPass,
            TestRollup::Failed => Self::Failed,
            TestRollup::Broken => Self::Broken,
            TestRollup::Skipped => Self::Skipped,
            TestRollup::Unknown => Self::Unknown,
        }
    }
}

impl From<model::AttemptRollup> for TestRollup {
    fn from(value: model::AttemptRollup) -> Self {
        match value {
            model::AttemptRollup::Passed => Self::Passed,
            model::AttemptRollup::FlakyPass => Self::FlakyPass,
            model::AttemptRollup::Failed => Self::Failed,
            model::AttemptRollup::Broken => Self::Broken,
            model::AttemptRollup::Skipped => Self::Skipped,
            model::AttemptRollup::Unknown => Self::Unknown,
        }
    }
}

#[derive(juniper::GraphQLEnum, Clone, Copy, Debug)]
pub(crate) enum TestFlakyState {
    Healthy,
    Flaky,
    Fixed,
    Broken,
}

impl From<TestFlakyState> for model::FlakyState {
    fn from(value: TestFlakyState) -> Self {
        match value {
            TestFlakyState::Healthy => Self::Healthy,
            TestFlakyState::Flaky => Self::Flaky,
            TestFlakyState::Fixed => Self::Fixed,
            TestFlakyState::Broken => Self::Broken,
        }
    }
}

impl From<model::FlakyState> for TestFlakyState {
    fn from(value: model::FlakyState) -> Self {
        match value {
            model::FlakyState::Healthy => Self::Healthy,
            model::FlakyState::Flaky => Self::Flaky,
            model::FlakyState::Fixed => Self::Fixed,
            model::FlakyState::Broken => Self::Broken,
        }
    }
}

#[derive(juniper::GraphQLInputObject)]
pub(crate) struct TestConfigurationFilterInput {
    key: String,
    value: String,
}

#[derive(juniper::GraphQLEnum, Clone, Copy, Debug, Default)]
pub(crate) enum TestExplorerSort {
    #[default]
    LastSeen,
    Name,
}

impl From<TestExplorerSort> for model::TestExplorerSort {
    fn from(value: TestExplorerSort) -> Self {
        match value {
            TestExplorerSort::LastSeen => Self::LastSeen,
            TestExplorerSort::Name => Self::Name,
        }
    }
}

pub(crate) struct TestExplorerPage(model::TestExplorerPage);

#[graphql_object]
impl TestExplorerPage {
    fn items(&self) -> Vec<TestExplorerRow> {
        self.0
            .items
            .clone()
            .into_iter()
            .map(TestExplorerRow)
            .collect()
    }

    fn has_more(&self) -> bool {
        self.0.has_more
    }
}

pub(crate) struct TestExplorerRow(model::TestExplorerRow);

#[graphql_object]
impl TestExplorerRow {
    fn case_key(&self) -> &str {
        self.0.case.key.as_str()
    }
    fn variant_key(&self) -> &str {
        self.0.variant.key.as_str()
    }
    fn name(&self) -> &str {
        &self.0.case.name
    }
    fn suite_path(&self) -> &[String] {
        &self.0.case.suite_path
    }
    fn code_reference(&self) -> Option<&str> {
        self.0.case.code_reference.as_deref()
    }
    fn explicit_id(&self) -> Option<&str> {
        self.0.case.explicit_id.as_deref()
    }
    fn first_seen_nanos(&self) -> String {
        nanos_string(self.0.case.first_seen_nanos)
    }
    fn last_seen_nanos(&self) -> String {
        nanos_string(self.0.case.last_seen_nanos)
    }
    fn parameters(&self) -> Vec<TestParameter> {
        self.0
            .variant
            .parameters
            .clone()
            .into_iter()
            .map(TestParameter)
            .collect()
    }
    fn invocation_id(&self) -> &str {
        &self.0.invocation_id
    }
    fn rollup(&self) -> TestRollup {
        self.0.rollup.into()
    }
    fn attempt_count(&self) -> i32 {
        i32::try_from(self.0.attempt_count).unwrap_or(i32::MAX)
    }
    fn last_result(&self) -> TestResult {
        TestResult(self.0.last_result.clone())
    }
    fn flaky(&self) -> Option<TestFlaky> {
        self.0.flaky.clone().map(TestFlaky)
    }
}

pub(crate) struct TestCaseDetail {
    case: model::TestCaseRecord,
    variants: Vec<TestVariantDetail>,
}

#[graphql_object]
impl TestCaseDetail {
    fn case_key(&self) -> &str {
        self.case.key.as_str()
    }
    fn name(&self) -> &str {
        &self.case.name
    }
    fn identity_source(&self) -> TestIdentitySource {
        self.case.identity_source.into()
    }
    fn suite_path(&self) -> &[String] {
        &self.case.suite_path
    }
    fn code_reference(&self) -> Option<&str> {
        self.case.code_reference.as_deref()
    }
    fn explicit_id(&self) -> Option<&str> {
        self.case.explicit_id.as_deref()
    }
    fn first_seen_nanos(&self) -> String {
        nanos_string(self.case.first_seen_nanos)
    }
    fn last_seen_nanos(&self) -> String {
        nanos_string(self.case.last_seen_nanos)
    }
    fn variants(&self) -> &[TestVariantDetail] {
        &self.variants
    }
}

#[derive(juniper::GraphQLEnum, Clone, Copy, Debug)]
pub(crate) enum TestIdentitySource {
    Explicit,
    CodeReference,
    NamePath,
}

impl From<model::TestCaseIdentitySource> for TestIdentitySource {
    fn from(value: model::TestCaseIdentitySource) -> Self {
        match value {
            model::TestCaseIdentitySource::Explicit => Self::Explicit,
            model::TestCaseIdentitySource::CodeReference => Self::CodeReference,
            model::TestCaseIdentitySource::NamePath => Self::NamePath,
        }
    }
}

pub(crate) struct TestVariantDetail {
    variant: model::TestVariantRecord,
    results: Vec<TestResult>,
    flaky: Option<TestFlaky>,
}

#[graphql_object]
impl TestVariantDetail {
    fn variant_key(&self) -> &str {
        self.variant.key.as_str()
    }
    fn parameters(&self) -> Vec<TestParameter> {
        self.variant
            .parameters
            .clone()
            .into_iter()
            .map(TestParameter)
            .collect()
    }
    fn first_seen_nanos(&self) -> String {
        nanos_string(self.variant.first_seen_nanos)
    }
    fn last_seen_nanos(&self) -> String {
        nanos_string(self.variant.last_seen_nanos)
    }
    fn history(&self) -> &[TestResult] {
        &self.results
    }
    fn flaky(&self) -> Option<&TestFlaky> {
        self.flaky.as_ref()
    }
}

pub(crate) struct TestParameter(model::TestParameter);

#[graphql_object]
impl TestParameter {
    fn name(&self) -> &str {
        &self.0.name
    }
    fn value(&self) -> &str {
        &self.0.value
    }
    fn excluded(&self) -> bool {
        self.0.excluded
    }
}

pub(crate) struct TestDimension(String, String);

#[graphql_object]
impl TestDimension {
    fn key(&self) -> &str {
        &self.0
    }
    fn value(&self) -> &str {
        &self.1
    }
}

pub(crate) struct TestResult(model::TestResultRecord);

#[graphql_object]
impl TestResult {
    fn invocation_id(&self) -> &str {
        &self.0.key.invocation_id
    }
    fn attempt(&self) -> i32 {
        i32::try_from(self.0.key.attempt.get()).unwrap_or(i32::MAX)
    }
    fn status(&self) -> TestResultStatus {
        match self.0.status {
            model::TestStatus::Passed => TestResultStatus::Passed,
            model::TestStatus::Failed => TestResultStatus::Failed,
            model::TestStatus::Broken => TestResultStatus::Broken,
            model::TestStatus::Skipped => TestResultStatus::Skipped,
            model::TestStatus::Unknown => TestResultStatus::Unknown,
        }
    }
    fn trace_id(&self) -> &str {
        self.0.trace_id.as_str()
    }
    fn span_id(&self) -> &str {
        &self.0.span_id
    }
    fn started_at_nanos(&self) -> String {
        nanos_string(self.0.started_at_nanos)
    }
    fn ended_at_nanos(&self) -> String {
        nanos_string(self.0.ended_at_nanos)
    }
    fn service(&self) -> &str {
        &self.0.service
    }
    fn service_version(&self) -> Option<&str> {
        self.0.service_version.as_deref()
    }
    fn vcs_head_revision(&self) -> Option<&str> {
        self.0.vcs_head_revision.as_deref()
    }
    fn failure_fingerprint(&self) -> Option<&str> {
        self.0.failure_fingerprint.as_deref()
    }
    fn configuration(&self) -> Vec<TestDimension> {
        self.0
            .configuration
            .dimensions
            .iter()
            .map(|(key, value)| TestDimension(key.clone(), value.clone()))
            .collect()
    }
}

#[derive(juniper::GraphQLEnum, Clone, Copy, Debug)]
pub(crate) enum TestResultStatus {
    Passed,
    Failed,
    Broken,
    Skipped,
    Unknown,
}

pub(crate) struct TestFlaky(model::TestFlakyStateRecord);

#[graphql_object]
impl TestFlaky {
    fn state(&self) -> TestFlakyState {
        self.0.state.into()
    }
    fn same_commit_divergence(&self) -> bool {
        self.0.evidence.same_commit_divergence
    }
    fn intra_invocation_mix(&self) -> bool {
        self.0.evidence.intra_invocation_mix
    }
    fn transition_count(&self) -> i32 {
        i32::try_from(self.0.evidence.window_transition_count).unwrap_or(i32::MAX)
    }
    fn consecutive_passes(&self) -> i32 {
        i32::try_from(self.0.evidence.consecutive_passes).unwrap_or(i32::MAX)
    }
    fn updated_at_nanos(&self) -> String {
        nanos_string(self.0.updated_at_nanos)
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "public GraphQL test explorer contract"
)]
pub(crate) async fn test_cases(
    context: &ApiContext,
    query: Option<String>,
    suite: Option<String>,
    service: Option<String>,
    service_version: Option<String>,
    status: Option<TestRollup>,
    flaky_state: Option<TestFlakyState>,
    configuration: Option<TestConfigurationFilterInput>,
    from_nanos: Option<String>,
    to_nanos: Option<String>,
    sort: Option<TestExplorerSort>,
    limit: Option<i32>,
    offset: Option<i32>,
) -> FieldResult<TestExplorerPage> {
    let configuration = configuration.map(|input| model::TestConfigurationFilter {
        key: input.key,
        value: input.value,
    });
    let filter = model::TestExplorerQuery {
        query,
        suite,
        service,
        service_version,
        status: status.map(Into::into),
        flaky_state: flaky_state.map(Into::into),
        configuration,
        from_nanos: from_nanos
            .map(|value| value.parse().map_err(|_| field_err("invalid fromNanos")))
            .transpose()?,
        to_nanos: to_nanos
            .map(|value| value.parse().map_err(|_| field_err("invalid toNanos")))
            .transpose()?,
    };
    let page = context
        .metadata
        .test_explorer(
            &filter,
            sort.unwrap_or_default().into(),
            clamp_limit(limit, 50).min(TEST_EXPLORER_MAX_LIMIT),
            usize::try_from(offset.unwrap_or(0).max(0))
                .unwrap_or(0)
                .min(TEST_EXPLORER_MAX_OFFSET),
        )
        .await
        .map_err(map_metadata_err)?;
    Ok(TestExplorerPage(page))
}

pub(crate) async fn test_case(
    context: &ApiContext,
    case_key: String,
    variant_limit: Option<i32>,
    result_limit: Option<i32>,
) -> FieldResult<Option<TestCaseDetail>> {
    let case_key =
        model::TestCaseKey::from_str(&case_key).map_err(|_| field_err("invalid test case key"))?;
    let case_key = case_key.as_str();
    let Some(case) = context
        .metadata
        .test_case(case_key)
        .await
        .map_err(map_metadata_err)?
    else {
        return Ok(None);
    };
    let variants = context
        .metadata
        .test_variants_for_case(
            case_key,
            clamp_limit(variant_limit, TEST_DETAIL_VARIANT_LIMIT).min(TEST_DETAIL_VARIANT_LIMIT),
        )
        .await
        .map_err(map_metadata_err)?;
    let result_limit =
        clamp_limit(result_limit, TEST_DETAIL_HISTORY_LIMIT).min(TEST_DETAIL_HISTORY_LIMIT);
    let mut details = Vec::with_capacity(variants.len());
    for variant in variants {
        let (results, flaky) = tokio::try_join!(
            context
                .metadata
                .test_results_for_variant(variant.key.as_str(), result_limit),
            context.metadata.test_flaky_state(variant.key.as_str()),
        )
        .map_err(map_metadata_err)?;
        let results = results.into_iter().map(TestResult).collect();
        let flaky = flaky.map(TestFlaky);
        details.push(TestVariantDetail {
            variant,
            results,
            flaky,
        });
    }
    Ok(Some(TestCaseDetail {
        case,
        variants: details,
    }))
}

fn map_metadata_err(error: parallax_storage::metadata::MetadataError) -> juniper::FieldError {
    use parallax_storage::metadata::MetadataErrorKind;
    match error.kind() {
        MetadataErrorKind::InvalidInput => field_err(error),
        _ => internal_field_err(error),
    }
}

#[cfg(test)]
#[path = "tests/tests.rs"]
mod resolver_tests;
