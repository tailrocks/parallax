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

    pub async fn test_case(&self, key: &str) -> anyhow::Result<Option<TestCaseRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT case_key, identity_source, explicit_id, code_reference,
                    suite_path, name, first_seen, last_seen
             FROM test_cases WHERE case_key = ?1",
                (key,),
            )
            .await?;
        rows.next()
            .await?
            .map(|row| decode_test_case(&row))
            .transpose()
    }

    pub async fn test_variant(&self, key: &str) -> anyhow::Result<Option<TestVariantRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT variant_key, case_key, parameters, first_seen, last_seen
             FROM test_variants WHERE variant_key = ?1",
                (key,),
            )
            .await?;
        rows.next()
            .await?
            .map(|row| decode_test_variant(&row))
            .transpose()
    }

    pub async fn test_results_for_invocation(
        &self,
        invocation_id: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<TestResultRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT variant_key, invocation_id, attempt, status, trace_id, span_id,
                    started_at, ended_at, service, service_version, vcs_head_revision,
                    configuration, failure_fingerprint
             FROM test_results WHERE invocation_id = ?1
             ORDER BY started_at, variant_key, attempt LIMIT ?2",
                (invocation_id, i64::try_from(limit).unwrap_or(i64::MAX)),
            )
            .await?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await? {
            results.push(decode_test_result(&row)?);
        }
        Ok(results)
    }

    pub async fn test_flaky_state(
        &self,
        variant_key: &str,
    ) -> anyhow::Result<Option<TestFlakyStateRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT variant_key, state, evidence, updated_at
             FROM test_flaky_states WHERE variant_key = ?1",
                (variant_key,),
            )
            .await?;
        rows.next()
            .await?
            .map(|row| decode_flaky_state(&row))
            .transpose()
    }
}

fn decode_test_case(row: &turso::Row) -> anyhow::Result<TestCaseRecord> {
    Ok(TestCaseRecord {
        key: text(row, 0).parse()?,
        identity_source: parse_identity_source(&text(row, 1))?,
        explicit_id: opt_text(row, 2),
        code_reference: opt_text(row, 3),
        suite_path: serde_json::from_str(&text(row, 4))?,
        name: text(row, 5),
        first_seen_nanos: millis_to_nanos(integer(row, 6)),
        last_seen_nanos: millis_to_nanos(integer(row, 7)),
    })
}

fn decode_test_variant(row: &turso::Row) -> anyhow::Result<TestVariantRecord> {
    Ok(TestVariantRecord {
        key: text(row, 0).parse()?,
        case_key: text(row, 1).parse()?,
        parameters: serde_json::from_str(&text(row, 2))?,
        first_seen_nanos: millis_to_nanos(integer(row, 3)),
        last_seen_nanos: millis_to_nanos(integer(row, 4)),
    })
}

fn decode_test_result(row: &turso::Row) -> anyhow::Result<TestResultRecord> {
    Ok(TestResultRecord {
        key: parallax_model::TestResultKey {
            variant_key: text(row, 0).parse()?,
            invocation_id: text(row, 1),
            attempt: parallax_model::TestAttempt::new(u32::try_from(integer(row, 2))?)?,
        },
        status: parse_test_status(&text(row, 3))?,
        trace_id: text(row, 4).parse()?,
        span_id: text(row, 5),
        started_at_nanos: millis_to_nanos(integer(row, 6)),
        ended_at_nanos: millis_to_nanos(integer(row, 7)),
        service: text(row, 8),
        service_version: opt_text(row, 9),
        vcs_head_revision: opt_text(row, 10),
        configuration: serde_json::from_str(&text(row, 11))?,
        failure_fingerprint: opt_text(row, 12),
    })
}

fn decode_flaky_state(row: &turso::Row) -> anyhow::Result<TestFlakyStateRecord> {
    Ok(TestFlakyStateRecord {
        variant_key: text(row, 0).parse()?,
        state: parse_flaky_state(&text(row, 1))?,
        evidence: serde_json::from_str(&text(row, 2))?,
        updated_at_nanos: millis_to_nanos(integer(row, 3)),
    })
}

fn parse_identity_source(value: &str) -> anyhow::Result<TestCaseIdentitySource> {
    match value {
        "explicit" => Ok(TestCaseIdentitySource::Explicit),
        "code_reference" => Ok(TestCaseIdentitySource::CodeReference),
        "name_path" => Ok(TestCaseIdentitySource::NamePath),
        _ => anyhow::bail!("unknown test identity source"),
    }
}

fn parse_test_status(value: &str) -> anyhow::Result<TestStatus> {
    match value {
        "passed" => Ok(TestStatus::Passed),
        "failed" => Ok(TestStatus::Failed),
        "broken" => Ok(TestStatus::Broken),
        "skipped" => Ok(TestStatus::Skipped),
        "unknown" => Ok(TestStatus::Unknown),
        _ => anyhow::bail!("unknown test status"),
    }
}

fn parse_flaky_state(value: &str) -> anyhow::Result<FlakyState> {
    match value {
        "healthy" => Ok(FlakyState::Healthy),
        "flaky" => Ok(FlakyState::Flaky),
        "fixed" => Ok(FlakyState::Fixed),
        "broken" => Ok(FlakyState::Broken),
        _ => anyhow::bail!("unknown flaky state"),
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
