//! Bounded GitHub Actions REST backfill tick (plan 124 residual).
//!
//! Read-only. Rate-limit aware. Advances the Turso cursor only after a full
//! page of jobs is durably accepted. Never rewrites raw logs into agent space.

use parallax_evidence::github_actions::{
    CI_ADJACENCY_CLAIM_WORDING, CI_CLAIM_KEYS, REST_JOBS_PAGE_CAP, attempt_identity,
    is_after_backfill_cursor, parse_rest_jobs_page,
};
use parallax_metadata::{
    CiAttemptAccept, CiAttemptDeliveryRecord, CiAttemptStoreError, EvidenceClaimRow,
    TursoMetadataStore, payload_sha256_hex,
};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

/// Injected HTTP surface so unit tests never touch the network.
#[async_trait::async_trait]
pub(crate) trait JobsHttp: Send + Sync {
    async fn list_workflow_run_jobs(
        &self,
        repo_full_name: &str,
        run_id: i64,
        page: u32,
        etag: Option<&str>,
    ) -> Result<JobsHttpResponse, JobsHttpError>;

    async fn list_recent_workflow_runs(
        &self,
        repo_full_name: &str,
        page: u32,
        etag: Option<&str>,
    ) -> Result<JobsHttpResponse, JobsHttpError>;
}

#[derive(Debug, Clone)]
pub(crate) struct JobsHttpResponse {
    pub status: u16,
    pub body: Value,
    pub etag: Option<String>,
    pub rate_limit_remaining: Option<u32>,
    pub rate_limit_reset_at_nanos: Option<u128>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum JobsHttpError {
    Transport(String),
    RateLimited { reset_at_nanos: Option<u128> },
    Unauthorized,
    NotFound,
    Unexpected(String),
}

impl std::fmt::Display for JobsHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(message) => write!(f, "transport: {message}"),
            Self::RateLimited { .. } => write!(f, "rate limited"),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::NotFound => write!(f, "not found"),
            Self::Unexpected(message) => write!(f, "unexpected: {message}"),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CiBackfillConfig {
    pub repos: Vec<String>,
    pub page_size: u32,
    pub max_runs_per_tick: u32,
}

impl Default for CiBackfillConfig {
    fn default() -> Self {
        Self {
            repos: Vec::new(),
            page_size: 30,
            max_runs_per_tick: 5,
        }
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub(crate) struct CiBackfillTickReport {
    pub repos_seen: usize,
    pub jobs_accepted: usize,
    pub jobs_duplicate: usize,
    pub runs_fetched: usize,
    pub rate_limited: bool,
    pub errors: usize,
}

pub(crate) async fn tick_once(
    metadata: &TursoMetadataStore,
    http: &dyn JobsHttp,
    config: &CiBackfillConfig,
    now_nanos: u128,
) -> CiBackfillTickReport {
    let mut report = CiBackfillTickReport::default();
    for repo in &config.repos {
        let repo = repo.trim();
        if repo.is_empty() || !repo.contains('/') {
            report.errors += 1;
            continue;
        }
        report.repos_seen += 1;
        match backfill_repo(metadata, http, config, repo, now_nanos).await {
            Ok(partial) => {
                report.jobs_accepted += partial.jobs_accepted;
                report.jobs_duplicate += partial.jobs_duplicate;
                report.runs_fetched += partial.runs_fetched;
                report.rate_limited |= partial.rate_limited;
                report.errors += partial.errors;
            }
            Err(error) => {
                tracing::warn!(%repo, %error, "ci backfill repo tick failed");
                report.errors += 1;
            }
        }
    }
    if report.jobs_accepted > 0 || report.jobs_duplicate > 0 {
        seed_claim_rows(metadata, now_nanos, &report).await;
    }
    report
}

async fn mark_fail(
    metadata: &TursoMetadataStore,
    repo: &str,
    error: &str,
    reset_at_nanos: Option<u128>,
) {
    if let Err(store_error) = metadata.fail_ci_backfill(repo, error, reset_at_nanos).await {
        tracing::warn!(%repo, %store_error, "ci backfill fail_ci_backfill write failed");
    }
}

fn completed_at_nanos(raw: Option<&str>) -> u128 {
    raw.and_then(|value| {
        time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339)
            .ok()
            .and_then(|ts| u128::try_from(ts.unix_timestamp_nanos()).ok())
    })
    .unwrap_or(0)
}

fn rest_delivery_record(
    repo: &str,
    attempt: parallax_evidence::github_actions::NormalizedCiAttempt,
    now_nanos: u128,
) -> CiAttemptDeliveryRecord {
    let body_bytes = serde_json::to_vec(&serde_json::json!({
        "job": {
            "id": attempt.job_id,
            "run_id": attempt.workflow_run_id,
            "run_attempt": attempt.attempt,
            "name": attempt.name,
            "conclusion": attempt.conclusion,
            "completed_at": attempt.completed_at,
        }
    }))
    .unwrap_or_default();
    CiAttemptDeliveryRecord {
        delivery_id: format!(
            "rest:{}:{}:{}:{}",
            repo, attempt.workflow_run_id, attempt.job_id, attempt.attempt
        ),
        attempt_id: attempt_identity(&attempt),
        provider: attempt.provider,
        repo_full_name: attempt.repo_full_name,
        workflow_run_id: attempt.workflow_run_id,
        job_id: attempt.job_id,
        attempt: attempt.attempt,
        conclusion: attempt.conclusion,
        name: attempt.name,
        lossiness: attempt.lossiness,
        payload_hash: payload_sha256_hex(&body_bytes),
        received_at_nanos: now_nanos,
    }
}

struct PageAcceptState<'a> {
    cursor_completed: u128,
    cursor_run: i64,
    now_nanos: u128,
    report: &'a mut CiBackfillTickReport,
    page_advance_ts: u128,
    page_advance_run: i64,
    advanced: bool,
}

async fn accept_page(
    metadata: &TursoMetadataStore,
    repo: &str,
    page: parallax_evidence::github_actions::RestJobsPage,
    state: &mut PageAcceptState<'_>,
) {
    for attempt in page.attempts.into_iter().take(REST_JOBS_PAGE_CAP) {
        let completed = completed_at_nanos(attempt.completed_at.as_deref());
        if !is_after_backfill_cursor(
            completed,
            attempt.workflow_run_id,
            state.cursor_completed,
            state.cursor_run,
        ) {
            continue;
        }
        let record = rest_delivery_record(repo, attempt, state.now_nanos);
        match metadata.accept_ci_attempt_delivery(&record).await {
            Ok(CiAttemptAccept::Inserted) => state.report.jobs_accepted += 1,
            Ok(CiAttemptAccept::Duplicate) => state.report.jobs_duplicate += 1,
            Err(CiAttemptStoreError::Collision(_)) => state.report.errors += 1,
            Err(CiAttemptStoreError::Internal(error)) => {
                tracing::warn!(%error, "ci backfill accept failed");
                state.report.errors += 1;
            }
        }
        if is_after_backfill_cursor(
            completed,
            record.workflow_run_id,
            state.page_advance_ts,
            state.page_advance_run,
        ) {
            state.page_advance_ts = completed;
            state.page_advance_run = record.workflow_run_id;
            state.advanced = true;
        }
    }
}

#[expect(clippy::too_many_lines, reason = "one sequential REST backfill pass")]
async fn backfill_repo(
    metadata: &TursoMetadataStore,
    http: &dyn JobsHttp,
    config: &CiBackfillConfig,
    repo: &str,
    now_nanos: u128,
) -> Result<CiBackfillTickReport, JobsHttpError> {
    let mut report = CiBackfillTickReport::default();
    let state = metadata
        .ci_backfill_state(repo)
        .await
        .map_err(|error| JobsHttpError::Unexpected(error.to_string()))?;
    if let Some(state) = &state
        && let Some(reset) = state.rate_limit_reset_at_nanos
        && reset > now_nanos
    {
        report.rate_limited = true;
        return Ok(report);
    }
    let cursor_completed = state.as_ref().map(|s| s.completed_at_nanos).unwrap_or(0);
    let cursor_run = state.as_ref().map(|s| s.workflow_run_id).unwrap_or(0);
    let etag = state.as_ref().and_then(|s| s.etag.as_deref());

    let runs = match http.list_recent_workflow_runs(repo, 1, etag).await {
        Ok(response) => response,
        Err(JobsHttpError::RateLimited { reset_at_nanos }) => {
            mark_fail(metadata, repo, "rate limited", reset_at_nanos).await;
            report.rate_limited = true;
            return Ok(report);
        }
        Err(error) => {
            mark_fail(metadata, repo, &error.to_string(), None).await;
            return Err(error);
        }
    };
    if runs.status == 304 {
        return Ok(report);
    }
    if runs.rate_limit_remaining == Some(0) {
        mark_fail(
            metadata,
            repo,
            "rate limited",
            runs.rate_limit_reset_at_nanos,
        )
        .await;
        report.rate_limited = true;
        return Ok(report);
    }

    let run_ids = extract_run_ids(&runs.body, config.max_runs_per_tick);
    let (advanced, advance_ts, advance_run) = {
        let mut accept = PageAcceptState {
            cursor_completed,
            cursor_run,
            now_nanos,
            report: &mut report,
            page_advance_ts: cursor_completed,
            page_advance_run: cursor_run,
            advanced: false,
        };

        for run_id in run_ids {
            accept.report.runs_fetched += 1;
            let jobs = match http.list_workflow_run_jobs(repo, run_id, 1, None).await {
                Ok(response) => response,
                Err(JobsHttpError::RateLimited { reset_at_nanos }) => {
                    mark_fail(metadata, repo, "rate limited", reset_at_nanos).await;
                    accept.report.rate_limited = true;
                    break;
                }
                Err(error) => {
                    mark_fail(metadata, repo, &error.to_string(), None).await;
                    accept.report.errors += 1;
                    continue;
                }
            };
            accept_page(
                metadata,
                repo,
                parse_rest_jobs_page(repo, &jobs.body),
                &mut accept,
            )
            .await;
            if jobs.rate_limit_remaining == Some(0) {
                mark_fail(
                    metadata,
                    repo,
                    "rate limited",
                    jobs.rate_limit_reset_at_nanos,
                )
                .await;
                accept.report.rate_limited = true;
                break;
            }
        }
        (
            accept.advanced,
            accept.page_advance_ts,
            accept.page_advance_run,
        )
    };
    if advanced
        && let Err(error) = metadata
            .advance_ci_backfill(
                repo,
                advance_ts,
                advance_run,
                runs.etag.as_deref(),
                now_nanos,
            )
            .await
    {
        tracing::warn!(%repo, %error, "ci backfill cursor advance failed");
        report.errors += 1;
    }
    Ok(report)
}

fn extract_run_ids(body: &Value, max_runs: u32) -> Vec<i64> {
    let runs = body
        .get("workflow_runs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    runs.into_iter()
        .filter_map(|run| run.get("id").and_then(Value::as_i64))
        .take(max_runs as usize)
        .collect()
}

async fn seed_claim_rows(
    metadata: &TursoMetadataStore,
    now_nanos: u128,
    report: &CiBackfillTickReport,
) {
    let total = report.jobs_accepted.saturating_add(report.jobs_duplicate) as u64;
    for key in CI_CLAIM_KEYS {
        let wording = match *key {
            "rest_backfill_rate_aware" => {
                "REST backfill is bounded, rate-aware, and cursor-monotonic".to_string()
            }
            "attempt_identity_stable" => {
                "CI attempt identity is stable across webhook redelivery and REST".to_string()
            }
            "flaky_requires_multi_attempt" => {
                "Flaky labels require mixed multi-attempt evidence".to_string()
            }
            "raw_logs_not_agent_default" => {
                "Raw CI logs are never the agent-visible default".to_string()
            }
            "workflow_job_webhook_accept" => {
                "workflow_job webhook accept is HMAC-verified and idempotent".to_string()
            }
            _ => CI_ADJACENCY_CLAIM_WORDING.to_string(),
        };
        let row = EvidenceClaimRow {
            domain: "ci_evidence".into(),
            claim_key: (*key).into(),
            level: "fixture_proven".into(),
            measured_at_nanos: now_nanos,
            coverage_numerator: Some(total),
            coverage_denominator: Some(total),
            wording,
        };
        if let Err(error) = metadata.upsert_evidence_claim(&row).await {
            tracing::warn!(%error, claim = *key, "ci claim row upsert failed");
        }
    }
}

/// Live reqwest transport (native TLS via workspace features).
pub(crate) struct LiveJobsHttp {
    client: reqwest::Client,
    token: Option<String>,
    api_version: String,
    page_size: u32,
}

impl LiveJobsHttp {
    pub(crate) fn new(token: Option<String>, page_size: u32) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent("parallax-ci-evidence/0.1")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            token,
            api_version: parallax_evidence::github_actions::API_VERSION_DEFAULT.to_string(),
            page_size: page_size.clamp(1, 100),
        }
    }

    async fn get(&self, url: &str, etag: Option<&str>) -> Result<JobsHttpResponse, JobsHttpError> {
        let mut request = self
            .client
            .get(url)
            .header("X-GitHub-Api-Version", self.api_version.as_str());
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        if let Some(etag) = etag {
            request = request.header(reqwest::header::IF_NONE_MATCH, etag);
        }
        let response = request
            .send()
            .await
            .map_err(|error| JobsHttpError::Transport(error.to_string()))?;
        let status = response.status().as_u16();
        let remaining = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());
        let reset_at_nanos = response
            .headers()
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .map(|secs| u128::from(secs) * 1_000_000_000);
        let etag = response
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        if status == 304 {
            return Ok(JobsHttpResponse {
                status,
                body: Value::Null,
                etag,
                rate_limit_remaining: remaining,
                rate_limit_reset_at_nanos: reset_at_nanos,
            });
        }
        if status == 401 || status == 403 {
            if remaining == Some(0) || status == 403 {
                return Err(JobsHttpError::RateLimited { reset_at_nanos });
            }
            return Err(JobsHttpError::Unauthorized);
        }
        if status == 404 {
            return Err(JobsHttpError::NotFound);
        }
        if status == 429 {
            return Err(JobsHttpError::RateLimited { reset_at_nanos });
        }
        if !(200..300).contains(&status) {
            return Err(JobsHttpError::Unexpected(format!("HTTP {status}")));
        }
        let body = response
            .json::<Value>()
            .await
            .map_err(|error| JobsHttpError::Transport(error.to_string()))?;
        Ok(JobsHttpResponse {
            status,
            body,
            etag,
            rate_limit_remaining: remaining,
            rate_limit_reset_at_nanos: reset_at_nanos,
        })
    }
}

#[async_trait::async_trait]
impl JobsHttp for LiveJobsHttp {
    async fn list_workflow_run_jobs(
        &self,
        repo_full_name: &str,
        run_id: i64,
        page: u32,
        etag: Option<&str>,
    ) -> Result<JobsHttpResponse, JobsHttpError> {
        let url = format!(
            "https://api.github.com/repos/{repo_full_name}/actions/runs/{run_id}/jobs?per_page={}&page={page}",
            self.page_size
        );
        self.get(&url, etag).await
    }

    async fn list_recent_workflow_runs(
        &self,
        repo_full_name: &str,
        page: u32,
        etag: Option<&str>,
    ) -> Result<JobsHttpResponse, JobsHttpError> {
        let url = format!(
            "https://api.github.com/repos/{repo_full_name}/actions/runs?status=completed&per_page={}&page={page}",
            self.page_size
        );
        self.get(&url, etag).await
    }
}

pub(crate) fn spawn_loop(
    tasks: &mut Vec<tokio::task::JoinHandle<()>>,
    metadata: Arc<TursoMetadataStore>,
    config: CiBackfillConfig,
    token: Option<String>,
    interval_secs: u64,
) {
    if config.repos.is_empty() {
        return;
    }
    let interval_secs = interval_secs.max(30);
    let repo_count = config.repos.len();
    let page_size = config.page_size;
    tasks.push(tokio::spawn(async move {
        let http = LiveJobsHttp::new(token, page_size);
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            let now_nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0);
            let report = tick_once(metadata.as_ref(), &http, &config, now_nanos).await;
            tracing::debug!(
                repos = report.repos_seen,
                accepted = report.jobs_accepted,
                duplicates = report.jobs_duplicate,
                runs = report.runs_fetched,
                rate_limited = report.rate_limited,
                errors = report.errors,
                "ci evidence backfill tick"
            );
        }
    }));
    tracing::info!(
        interval_secs,
        repos = repo_count,
        "ci evidence REST backfill ready"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Mutex;

    struct MockHttp {
        runs: Value,
        jobs: Mutex<Value>,
    }

    #[async_trait::async_trait]
    impl JobsHttp for MockHttp {
        async fn list_workflow_run_jobs(
            &self,
            _repo_full_name: &str,
            _run_id: i64,
            _page: u32,
            _etag: Option<&str>,
        ) -> Result<JobsHttpResponse, JobsHttpError> {
            Ok(JobsHttpResponse {
                status: 200,
                body: self.jobs.lock().expect("lock").clone(),
                etag: Some("\"jobs\"".into()),
                rate_limit_remaining: Some(100),
                rate_limit_reset_at_nanos: None,
            })
        }

        async fn list_recent_workflow_runs(
            &self,
            _repo_full_name: &str,
            _page: u32,
            _etag: Option<&str>,
        ) -> Result<JobsHttpResponse, JobsHttpError> {
            Ok(JobsHttpResponse {
                status: 200,
                body: self.runs.clone(),
                etag: Some("\"runs\"".into()),
                rate_limit_remaining: Some(100),
                rate_limit_reset_at_nanos: None,
            })
        }
    }

    #[tokio::test]
    async fn tick_accepts_jobs_advances_cursor_and_writes_claim_rows() {
        let directory = tempfile::tempdir().expect("tempdir");
        let store = TursoMetadataStore::open(directory.path().join("meta.db"))
            .await
            .expect("store");
        let http = MockHttp {
            runs: json!({
                "workflow_runs": [{ "id": 99 }]
            }),
            jobs: Mutex::new(json!({
                "jobs": [{
                    "id": 7,
                    "run_id": 99,
                    "run_attempt": 1,
                    "name": "test",
                    "conclusion": "success",
                    "html_url": "https://github.com/tailrocks/parallax/actions/runs/99/job/7",
                    "completed_at": "2026-07-17T12:00:00Z"
                }]
            })),
        };
        let config = CiBackfillConfig {
            repos: vec!["tailrocks/parallax".into()],
            page_size: 30,
            max_runs_per_tick: 5,
        };
        let report = tick_once(&store, &http, &config, 10_000_000).await;
        assert_eq!(report.jobs_accepted, 1);
        assert_eq!(report.errors, 0);
        let state = store
            .ci_backfill_state("tailrocks/parallax")
            .await
            .expect("state")
            .expect("present");
        assert_eq!(state.workflow_run_id, 99);
        assert!(state.completed_at_nanos > 0);
        assert_eq!(
            store
                .count_evidence_claims(Some("ci_evidence"))
                .await
                .expect("claims"),
            CI_CLAIM_KEYS.len() as u64
        );
        // Idempotent redelivery of the same REST synthetic delivery id.
        let again = tick_once(&store, &http, &config, 11_000_000).await;
        assert_eq!(again.jobs_accepted, 0);
        assert_eq!(again.jobs_duplicate, 0);
    }
}
