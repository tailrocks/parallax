//! Turso persistence for derived test identities, result references, and flaky state.

use super::*;
use parallax_model::{
    AttemptRollup, FlakyState, TestCaseIdentitySource, TestCaseRecord, TestExplorerPage,
    TestExplorerQuery, TestExplorerRow, TestExplorerSort, TestFlakyCandidate,
    TestFlakyCandidatePage, TestFlakyCursor, TestFlakyStateRecord, TestResultRecord,
    TestResultWindow, TestStatus, TestVariantRecord,
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

    pub async fn test_variants_for_case(
        &self,
        case_key: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<TestVariantRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT variant_key, case_key, parameters, first_seen, last_seen
                 FROM test_variants WHERE case_key = ?1
                 ORDER BY last_seen DESC, variant_key
                 LIMIT ?2",
                (case_key, i64::try_from(limit).unwrap_or(i64::MAX)),
            )
            .await?;
        let mut variants = Vec::new();
        while let Some(row) = rows.next().await? {
            variants.push(decode_test_variant(&row)?);
        }
        Ok(variants)
    }

    pub async fn test_results_for_variant(
        &self,
        variant_key: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<TestResultRecord>> {
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT variant_key, invocation_id, attempt, status, trace_id, span_id,
                        started_at, ended_at, service, service_version, vcs_head_revision,
                        configuration, failure_fingerprint
                 FROM test_results WHERE variant_key = ?1
                 ORDER BY started_at DESC, invocation_id, attempt DESC
                 LIMIT ?2",
                (variant_key, i64::try_from(limit).unwrap_or(i64::MAX)),
            )
            .await?;
        let mut results = Vec::new();
        while let Some(row) = rows.next().await? {
            results.push(decode_test_result(&row)?);
        }
        Ok(results)
    }

    pub async fn test_flaky_candidates(
        &self,
        from_nanos: u128,
        to_nanos: u128,
        after: Option<&TestFlakyCursor>,
        limit: usize,
    ) -> anyhow::Result<TestFlakyCandidatePage> {
        if limit == 0 {
            return Ok(TestFlakyCandidatePage::default());
        }
        let fetch = limit.saturating_add(1);
        let conn = self.conn.lock().await;
        let mut rows = if let Some(cursor) = after {
            conn.query(
                "SELECT variant_key, MAX(ended_at) AS last_ended
                 FROM test_results
                 WHERE ended_at BETWEEN ?1 AND ?2
                 GROUP BY variant_key
                 HAVING MAX(ended_at) > ?3
                    OR (MAX(ended_at) = ?3 AND variant_key > ?4)
                 ORDER BY last_ended, variant_key
                 LIMIT ?5",
                (
                    nanos_to_millis(from_nanos),
                    nanos_to_millis(to_nanos),
                    nanos_to_millis(cursor.last_ended_nanos),
                    cursor.variant_key.as_str(),
                    i64::try_from(fetch)?,
                ),
            )
            .await?
        } else {
            conn.query(
                "SELECT variant_key, MAX(ended_at) AS last_ended
                 FROM test_results
                 WHERE ended_at BETWEEN ?1 AND ?2
                 GROUP BY variant_key
                 ORDER BY last_ended, variant_key
                 LIMIT ?3",
                (
                    nanos_to_millis(from_nanos),
                    nanos_to_millis(to_nanos),
                    i64::try_from(fetch)?,
                ),
            )
            .await?
        };
        let mut items = Vec::new();
        while let Some(row) = rows.next().await? {
            items.push(TestFlakyCandidate {
                variant_key: text(&row, 0).parse()?,
                last_ended_nanos: millis_to_nanos(integer(&row, 1)),
            });
        }
        let has_more = items.len() > limit;
        items.truncate(limit);
        Ok(TestFlakyCandidatePage { items, has_more })
    }

    pub async fn test_results_for_variant_window(
        &self,
        variant_key: &str,
        from_nanos: u128,
        to_nanos: u128,
        limit: usize,
    ) -> anyhow::Result<TestResultWindow> {
        if limit == 0 {
            return Ok(TestResultWindow::default());
        }
        let conn = self.conn.lock().await;
        let mut rows = conn
            .query(
                "SELECT variant_key, invocation_id, attempt, status, trace_id, span_id,
                        started_at, ended_at, service, service_version, vcs_head_revision,
                        configuration, failure_fingerprint
                 FROM test_results
                 WHERE variant_key = ?1 AND ended_at BETWEEN ?2 AND ?3
                 ORDER BY ended_at, invocation_id, attempt
                 LIMIT ?4",
                (
                    variant_key,
                    nanos_to_millis(from_nanos),
                    nanos_to_millis(to_nanos),
                    i64::try_from(limit.saturating_add(1))?,
                ),
            )
            .await?;
        let mut items = Vec::new();
        while let Some(row) = rows.next().await? {
            items.push(decode_test_result(&row)?);
        }
        let truncated = items.len() > limit;
        items.truncate(limit);
        Ok(TestResultWindow { items, truncated })
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

    pub async fn test_explorer(
        &self,
        filter: &TestExplorerQuery,
        sort: TestExplorerSort,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<TestExplorerPage> {
        if limit == 0 {
            return Ok(TestExplorerPage::default());
        }
        let (sql, params) = build_test_explorer_sql(filter, sort, limit, offset)?;
        let conn = self.conn.lock().await;
        let mut rows = conn.query(&sql, params).await?;
        let mut items = Vec::new();
        while let Some(row) = rows.next().await? {
            items.push(decode_test_explorer_row(&row)?);
        }
        let has_more = items.len() > limit;
        items.truncate(limit);
        Ok(TestExplorerPage { items, has_more })
    }
}

fn bind_param(params: &mut Vec<Value>, value: Value) -> String {
    params.push(value);
    format!("?{}", params.len())
}

fn build_test_explorer_sql(
    filter: &TestExplorerQuery,
    sort: TestExplorerSort,
    limit: usize,
    offset: usize,
) -> anyhow::Result<(String, Vec<Value>)> {
    let mut params = Vec::new();
    let eligible_where = explorer_eligible_where(filter, &mut params);
    let final_where = explorer_final_where(filter, &mut params);
    let order = match sort {
        TestExplorerSort::LastSeen => "e.last_ended DESC, tc.case_key, tv.variant_key",
        TestExplorerSort::Name => "tc.name COLLATE NOCASE, tc.case_key, tv.variant_key",
    };
    let fetch = limit.saturating_add(1);
    let limit_bind = bind_param(&mut params, Value::Integer(i64::try_from(fetch)?));
    let offset_bind = bind_param(&mut params, Value::Integer(i64::try_from(offset)?));
    let cte = TEST_EXPLORER_CTE.replace("{eligible_where}", &eligible_where);
    let sql = format!(
        "{cte}
         SELECT e.invocation_id, e.rollup, e.attempt_count,
                tc.case_key, tc.identity_source, tc.explicit_id, tc.code_reference,
                tc.suite_path, tc.name, tc.first_seen, tc.last_seen,
                tv.variant_key, tv.case_key, tv.parameters, tv.first_seen, tv.last_seen,
                lr.variant_key, lr.invocation_id, lr.attempt, lr.status, lr.trace_id, lr.span_id,
                lr.started_at, lr.ended_at, lr.service, lr.service_version,
                lr.vcs_head_revision, lr.configuration, lr.failure_fingerprint,
                fs.variant_key, fs.state, fs.evidence, fs.updated_at
         FROM eligible e
         JOIN test_results lr ON lr.variant_key = e.variant_key
           AND lr.invocation_id = e.invocation_id AND lr.attempt = e.last_attempt
         JOIN test_variants tv ON tv.variant_key = e.variant_key
         JOIN test_cases tc ON tc.case_key = tv.case_key
         LEFT JOIN test_flaky_states fs ON fs.variant_key = tv.variant_key
         WHERE {final_where}
         ORDER BY {order}
         LIMIT {limit_bind} OFFSET {offset_bind}"
    );
    Ok((sql, params))
}

const TEST_EXPLORER_CTE: &str = "WITH attempt_groups AS (
               SELECT r.variant_key, r.invocation_id,
                      MIN(r.started_at) AS first_started, MAX(r.ended_at) AS last_ended,
                      COUNT(*) AS attempt_count, MAX(r.attempt) AS last_attempt,
                      CASE
                        WHEN EXISTS (
                          SELECT 1 FROM test_results p JOIN test_results f
                            ON f.variant_key = p.variant_key AND f.invocation_id = p.invocation_id
                           AND f.attempt < p.attempt
                          WHERE p.variant_key = r.variant_key AND p.invocation_id = r.invocation_id
                            AND p.status = 'passed' AND f.status IN ('failed', 'broken')
                        ) THEN 'flaky_pass'
                        WHEN MAX(r.status = 'failed') = 1 THEN 'failed'
                        WHEN MAX(r.status = 'broken') = 1 THEN 'broken'
                        WHEN MAX(r.status = 'passed') = 1 THEN 'passed'
                        WHEN MIN(r.status = 'skipped') = 1 THEN 'skipped'
                        ELSE 'unknown'
                      END AS rollup
               FROM test_results r GROUP BY r.variant_key, r.invocation_id
             ), eligible AS (
               SELECT ag.*, ROW_NUMBER() OVER (
                 PARTITION BY ag.variant_key ORDER BY ag.last_ended DESC, ag.invocation_id DESC
               ) AS variant_rank
               FROM attempt_groups ag
               JOIN test_results lr ON lr.variant_key = ag.variant_key
                 AND lr.invocation_id = ag.invocation_id AND lr.attempt = ag.last_attempt
               {eligible_where}
             )";

fn explorer_eligible_where(filter: &TestExplorerQuery, params: &mut Vec<Value>) -> String {
    let mut clauses = Vec::new();
    if let Some(service) = &filter.service {
        let p = bind_param(params, Value::Text(service.clone()));
        clauses.push(format!("lr.service = {p}"));
    }
    if let Some(version) = &filter.service_version {
        let p = bind_param(params, Value::Text(version.clone()));
        clauses.push(format!("lr.service_version = {p}"));
    }
    if let Some(from) = filter.from_nanos {
        let p = bind_param(params, Value::Integer(nanos_to_millis(from)));
        clauses.push(format!("ag.last_ended >= {p}"));
    }
    if let Some(to) = filter.to_nanos {
        let p = bind_param(params, Value::Integer(nanos_to_millis(to)));
        clauses.push(format!("ag.last_ended <= {p}"));
    }
    if let Some(configuration) = &filter.configuration {
        let key = bind_param(params, Value::Text(configuration.key.clone()));
        let value = bind_param(params, Value::Text(configuration.value.clone()));
        clauses.push(format!(
            "EXISTS (SELECT 1 FROM json_each(json_extract(lr.configuration, '$.dimensions')) cfg \
             WHERE cfg.key = {key} AND cfg.value = {value})"
        ));
    }
    if clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", clauses.join(" AND "))
    }
}

fn explorer_final_where(filter: &TestExplorerQuery, params: &mut Vec<Value>) -> String {
    let mut clauses = vec!["e.variant_rank = 1".to_string()];
    if let Some(status) = filter.status {
        let p = bind_param(params, Value::Text(attempt_rollup(status).to_string()));
        clauses.push(format!("e.rollup = {p}"));
    }
    if let Some(state) = filter.flaky_state {
        let p = bind_param(params, Value::Text(flaky_state(state).to_string()));
        clauses.push(format!("fs.state = {p}"));
    }
    if let Some(suite) = &filter.suite {
        let p = bind_param(params, Value::Text(suite.clone()));
        clauses.push(format!(
            "EXISTS (SELECT 1 FROM json_each(tc.suite_path) suite WHERE suite.value = {p})"
        ));
    }
    if let Some(query) = &filter.query {
        let escaped = query
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let p = bind_param(params, Value::Text(format!("%{escaped}%")));
        clauses.push(format!(
            "(tc.name LIKE {p} ESCAPE '\\' OR tc.code_reference LIKE {p} ESCAPE '\\' \
             OR tc.explicit_id LIKE {p} ESCAPE '\\' OR tc.case_key LIKE {p} ESCAPE '\\' \
             OR tv.variant_key LIKE {p} ESCAPE '\\')"
        ));
    }
    clauses.join(" AND ")
}

fn decode_test_explorer_row(row: &turso::Row) -> anyhow::Result<TestExplorerRow> {
    Ok(TestExplorerRow {
        invocation_id: text(row, 0),
        rollup: parse_attempt_rollup(&text(row, 1))?,
        attempt_count: u32::try_from(integer(row, 2))?,
        case: decode_test_case_at(row, 3)?,
        variant: decode_test_variant_at(row, 11)?,
        last_result: decode_test_result_at(row, 16)?,
        flaky: opt_text(row, 29)
            .map(|_| decode_flaky_state_at(row, 29))
            .transpose()?,
    })
}

fn decode_test_case(row: &turso::Row) -> anyhow::Result<TestCaseRecord> {
    decode_test_case_at(row, 0)
}

fn decode_test_case_at(row: &turso::Row, offset: usize) -> anyhow::Result<TestCaseRecord> {
    Ok(TestCaseRecord {
        key: text(row, offset).parse()?,
        identity_source: parse_identity_source(&text(row, offset + 1))?,
        explicit_id: opt_text(row, offset + 2),
        code_reference: opt_text(row, offset + 3),
        suite_path: serde_json::from_str(&text(row, offset + 4))?,
        name: text(row, offset + 5),
        first_seen_nanos: millis_to_nanos(integer(row, offset + 6)),
        last_seen_nanos: millis_to_nanos(integer(row, offset + 7)),
    })
}

fn decode_test_variant(row: &turso::Row) -> anyhow::Result<TestVariantRecord> {
    decode_test_variant_at(row, 0)
}

fn decode_test_variant_at(row: &turso::Row, offset: usize) -> anyhow::Result<TestVariantRecord> {
    Ok(TestVariantRecord {
        key: text(row, offset).parse()?,
        case_key: text(row, offset + 1).parse()?,
        parameters: serde_json::from_str(&text(row, offset + 2))?,
        first_seen_nanos: millis_to_nanos(integer(row, offset + 3)),
        last_seen_nanos: millis_to_nanos(integer(row, offset + 4)),
    })
}

fn decode_test_result(row: &turso::Row) -> anyhow::Result<TestResultRecord> {
    decode_test_result_at(row, 0)
}

fn decode_test_result_at(row: &turso::Row, offset: usize) -> anyhow::Result<TestResultRecord> {
    Ok(TestResultRecord {
        key: parallax_model::TestResultKey {
            variant_key: text(row, offset).parse()?,
            invocation_id: text(row, offset + 1),
            attempt: parallax_model::TestAttempt::new(u32::try_from(integer(row, offset + 2))?)?,
        },
        status: parse_test_status(&text(row, offset + 3))?,
        trace_id: text(row, offset + 4).parse()?,
        span_id: text(row, offset + 5),
        started_at_nanos: millis_to_nanos(integer(row, offset + 6)),
        ended_at_nanos: millis_to_nanos(integer(row, offset + 7)),
        service: text(row, offset + 8),
        service_version: opt_text(row, offset + 9),
        vcs_head_revision: opt_text(row, offset + 10),
        configuration: serde_json::from_str(&text(row, offset + 11))?,
        failure_fingerprint: opt_text(row, offset + 12),
    })
}

fn decode_flaky_state(row: &turso::Row) -> anyhow::Result<TestFlakyStateRecord> {
    decode_flaky_state_at(row, 0)
}

fn decode_flaky_state_at(row: &turso::Row, offset: usize) -> anyhow::Result<TestFlakyStateRecord> {
    Ok(TestFlakyStateRecord {
        variant_key: text(row, offset).parse()?,
        state: parse_flaky_state(&text(row, offset + 1))?,
        evidence: serde_json::from_str(&text(row, offset + 2))?,
        updated_at_nanos: millis_to_nanos(integer(row, offset + 3)),
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

fn parse_attempt_rollup(value: &str) -> anyhow::Result<AttemptRollup> {
    match value {
        "passed" => Ok(AttemptRollup::Passed),
        "flaky_pass" => Ok(AttemptRollup::FlakyPass),
        "failed" => Ok(AttemptRollup::Failed),
        "broken" => Ok(AttemptRollup::Broken),
        "skipped" => Ok(AttemptRollup::Skipped),
        "unknown" => Ok(AttemptRollup::Unknown),
        _ => anyhow::bail!("unknown test attempt rollup"),
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

const fn attempt_rollup(rollup: AttemptRollup) -> &'static str {
    match rollup {
        AttemptRollup::Passed => "passed",
        AttemptRollup::FlakyPass => "flaky_pass",
        AttemptRollup::Failed => "failed",
        AttemptRollup::Broken => "broken",
        AttemptRollup::Skipped => "skipped",
        AttemptRollup::Unknown => "unknown",
    }
}
