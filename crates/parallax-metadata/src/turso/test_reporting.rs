//! Turso persistence for derived test identities, result references, and flaky state.

use super::*;
use parallax_model::{
    FlakyState, TestCaseIdentitySource, TestCaseRecord, TestFlakyStateRecord, TestResultRecord,
    TestStatus, TestVariantRecord,
};

impl TursoMetadataStore {
    pub async fn upsert_test_case(&self, record: &TestCaseRecord) -> anyhow::Result<()> {
        self.conn
            .lock()
            .await
            .execute(
                "INSERT INTO test_cases
               (case_key, identity_source, explicit_id, code_reference, suite_path,
                name, first_seen, last_seen)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(case_key) DO UPDATE SET
               identity_source = excluded.identity_source,
               explicit_id = excluded.explicit_id,
               code_reference = excluded.code_reference,
               suite_path = excluded.suite_path,
               name = excluded.name,
               first_seen = MIN(test_cases.first_seen, excluded.first_seen),
               last_seen = MAX(test_cases.last_seen, excluded.last_seen)",
                (
                    record.key.as_str(),
                    identity_source(record.identity_source),
                    record.explicit_id.clone(),
                    record.code_reference.clone(),
                    serde_json::to_string(&record.suite_path)?,
                    record.name.clone(),
                    nanos_to_millis(record.first_seen_nanos),
                    nanos_to_millis(record.last_seen_nanos),
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn upsert_test_variant(&self, record: &TestVariantRecord) -> anyhow::Result<()> {
        self.conn
            .lock()
            .await
            .execute(
                "INSERT INTO test_variants
               (variant_key, case_key, parameters, first_seen, last_seen)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(variant_key) DO UPDATE SET
               case_key = excluded.case_key,
               parameters = excluded.parameters,
               first_seen = MIN(test_variants.first_seen, excluded.first_seen),
               last_seen = MAX(test_variants.last_seen, excluded.last_seen)",
                (
                    record.key.as_str(),
                    record.case_key.as_str(),
                    serde_json::to_string(&record.parameters)?,
                    nanos_to_millis(record.first_seen_nanos),
                    nanos_to_millis(record.last_seen_nanos),
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn upsert_test_result(&self, record: &TestResultRecord) -> anyhow::Result<()> {
        self.conn
            .lock()
            .await
            .execute(
                "INSERT INTO test_results
               (variant_key, invocation_id, attempt, status, trace_id, span_id,
                started_at, ended_at, service, service_version, vcs_head_revision,
                configuration, failure_fingerprint)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(variant_key, invocation_id, attempt) DO UPDATE SET
               status = excluded.status, trace_id = excluded.trace_id,
               span_id = excluded.span_id, started_at = excluded.started_at,
               ended_at = excluded.ended_at, service = excluded.service,
               service_version = excluded.service_version,
               vcs_head_revision = excluded.vcs_head_revision,
               configuration = excluded.configuration,
               failure_fingerprint = excluded.failure_fingerprint",
                (
                    record.key.variant_key.as_str(),
                    record.key.invocation_id.clone(),
                    i64::from(record.key.attempt.get()),
                    test_status(record.status),
                    record.trace_id.as_str(),
                    record.span_id.clone(),
                    nanos_to_millis(record.started_at_nanos),
                    nanos_to_millis(record.ended_at_nanos),
                    record.service.clone(),
                    record.service_version.clone(),
                    record.vcs_head_revision.clone(),
                    serde_json::to_string(&record.configuration)?,
                    record.failure_fingerprint.clone(),
                ),
            )
            .await?;
        Ok(())
    }

    pub async fn upsert_test_flaky_state(
        &self,
        record: &TestFlakyStateRecord,
    ) -> anyhow::Result<()> {
        self.conn
            .lock()
            .await
            .execute(
                "INSERT INTO test_flaky_states (variant_key, state, evidence, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(variant_key) DO UPDATE SET
               state = excluded.state, evidence = excluded.evidence,
               updated_at = excluded.updated_at",
                (
                    record.variant_key.as_str(),
                    flaky_state(record.state),
                    serde_json::to_string(&record.evidence)?,
                    nanos_to_millis(record.updated_at_nanos),
                ),
            )
            .await?;
        Ok(())
    }
}

const fn identity_source(source: TestCaseIdentitySource) -> &'static str {
    match source {
        TestCaseIdentitySource::Explicit => "explicit",
        TestCaseIdentitySource::CodeReference => "code_reference",
        TestCaseIdentitySource::NamePath => "name_path",
    }
}

const fn test_status(status: TestStatus) -> &'static str {
    match status {
        TestStatus::Passed => "passed",
        TestStatus::Failed => "failed",
        TestStatus::Broken => "broken",
        TestStatus::Skipped => "skipped",
        TestStatus::Unknown => "unknown",
    }
}

const fn flaky_state(state: FlakyState) -> &'static str {
    match state {
        FlakyState::Healthy => "healthy",
        FlakyState::Flaky => "flaky",
        FlakyState::Fixed => "fixed",
        FlakyState::Broken => "broken",
    }
}
