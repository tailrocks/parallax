//! GitHub Actions CI evidence normalizer (plan 124 residual).
//!
//! Read-only product evidence adapter. Provider text is untrusted. Flaky labels
//! require multi-attempt evidence — a single retry is never enough.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub const PROVIDER: &str = "github_actions";
pub const API_VERSION_DEFAULT: &str = "2022-11-28";

/// Stable attempt identity across redelivery/retry of the same job attempt.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NormalizedCiAttempt {
    pub provider: String,
    pub repo_full_name: String,
    pub workflow_run_id: i64,
    pub job_id: i64,
    pub attempt: u32,
    pub check_run_id: Option<i64>,
    pub conclusion: Option<String>,
    pub name: Option<String>,
    pub html_url_present: bool,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub lossiness: Vec<String>,
}

/// Flaky claim requires mixed outcomes across distinct attempts for one test.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FlakyClaimEvidence {
    pub test_name: String,
    pub attempt_ids: Vec<String>,
    pub fail_count: u32,
    pub pass_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FlakyClaimError {
    InsufficientAttempts,
    SingleAttemptOnly,
    ConflictingAttempt,
    TooManyAttempts,
}

impl FlakyClaimError {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InsufficientAttempts => "flaky claim requires multi-attempt evidence",
            Self::SingleAttemptOnly => "a single retry is not flaky evidence",
            Self::ConflictingAttempt => "one attempt identity has conflicting outcomes",
            Self::TooManyAttempts => "attempt evidence exceeds the supported count",
        }
    }
}

/// Normalize a GitHub `workflow_job` webhook payload (or Actions REST job body).
#[must_use]
pub fn normalize_workflow_job(event_name: &str, body: &Value) -> Option<NormalizedCiAttempt> {
    if event_name != "workflow_job" && event_name != "check_run" && event_name != "rest.job" {
        return None;
    }
    let job = body
        .get("workflow_job")
        .or_else(|| body.get("check_run"))
        .or_else(|| body.get("job"))
        .unwrap_or(body);

    let workflow_run_id = job
        .get("run_id")
        .or_else(|| body.pointer("/workflow_run/id"))
        .and_then(Value::as_i64)?;
    let job_id = job.get("id").and_then(Value::as_i64)?;
    let attempt = u32::try_from(
        job.get("run_attempt")
            .and_then(Value::as_u64)
            .unwrap_or(1)
            .clamp(1, u64::from(u32::MAX)),
    )
    .unwrap_or(u32::MAX);

    let mut lossiness = Vec::new();
    let repo_full_name = body
        .pointer("/repository/full_name")
        .and_then(Value::as_str)
        .or_else(|| job.get("repository").and_then(Value::as_str))
        .unwrap_or_else(|| {
            lossiness.push("missing_repo_full_name".into());
            "unknown/unknown"
        })
        .to_owned();

    let conclusion = job
        .get("conclusion")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let name = job.get("name").and_then(Value::as_str).map(str::to_owned);
    let html_url_present = job
        .get("html_url")
        .and_then(Value::as_str)
        .is_some_and(|url| !url.is_empty());
    if !html_url_present {
        lossiness.push("missing_html_url".into());
    }
    let check_run_id = if event_name == "check_run" {
        Some(job_id)
    } else {
        job.get("check_run_id").and_then(Value::as_i64)
    };

    Some(NormalizedCiAttempt {
        provider: PROVIDER.into(),
        repo_full_name,
        workflow_run_id,
        job_id,
        attempt,
        check_run_id,
        conclusion,
        name,
        html_url_present,
        started_at: job
            .get("started_at")
            .and_then(Value::as_str)
            .map(str::to_owned),
        completed_at: job
            .get("completed_at")
            .and_then(Value::as_str)
            .map(str::to_owned),
        lossiness,
    })
}

/// Stable id for (run, job, attempt) used in flaky multi-attempt evidence.
#[must_use]
pub fn attempt_identity(attempt: &NormalizedCiAttempt) -> String {
    format!(
        "{provider}:{repo}:{run}:{job}:{attempt}",
        provider = attempt.provider,
        repo = attempt.repo_full_name,
        run = attempt.workflow_run_id,
        job = attempt.job_id,
        attempt = attempt.attempt
    )
}

/// Bounded fingerprint of a redacted job log fragment (never raw logs).
#[must_use]
pub fn redacted_log_fingerprint(redacted_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(redacted_bytes);
    format!("sha256:{}", encode_hex(&hasher.finalize()))
}

/// Product claim wording for CI adjacency — never root-cause language.
pub const CI_ADJACENCY_CLAIM_WORDING: &str =
    "CI job attempt is adjacent evidence only; never treat adjacency as root cause.";

/// Allowed claim keys for the CI evidence domain (dated coverage rows).
pub const CI_CLAIM_KEYS: &[&str] = &[
    "workflow_job_webhook_accept",
    "rest_backfill_rate_aware",
    "attempt_identity_stable",
    "flaky_requires_multi_attempt",
    "raw_logs_not_agent_default",
];

/// One REST page of completed workflow jobs (plan 124 backfill).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RestJobsPage {
    pub attempts: Vec<NormalizedCiAttempt>,
    /// Max `(completed_at_nanos, workflow_run_id)` among completed jobs.
    pub advance_completed_at_nanos: Option<u128>,
    pub advance_workflow_run_id: Option<i64>,
    pub truncated: bool,
}

/// Cap jobs accepted from a single REST page (rate/output budget).
pub const REST_JOBS_PAGE_CAP: usize = 100;

/// Parse a GitHub Actions list-jobs-for-a-workflow-run JSON body.
///
/// Expects either `{ "jobs": [ ... ] }` or a bare job array. Each job is
/// normalized as `rest.job` with an injected repository full name.
#[must_use]
pub fn parse_rest_jobs_page(repo_full_name: &str, body: &Value) -> RestJobsPage {
    let jobs = body
        .get("jobs")
        .and_then(Value::as_array)
        .or_else(|| body.as_array());
    let Some(jobs) = jobs else {
        return RestJobsPage::default();
    };
    let truncated = jobs.len() > REST_JOBS_PAGE_CAP;
    let mut page = RestJobsPage {
        truncated,
        ..RestJobsPage::default()
    };
    for job in jobs.iter().take(REST_JOBS_PAGE_CAP) {
        let mut wrapped = job.clone();
        if let Some(object) = wrapped.as_object_mut() {
            object
                .entry("repository".to_string())
                .or_insert_with(|| Value::String(repo_full_name.to_owned()));
        }
        let Some(mut attempt) = normalize_workflow_job("rest.job", &wrapped) else {
            continue;
        };
        if attempt.repo_full_name == "unknown/unknown" {
            repo_full_name.clone_into(&mut attempt.repo_full_name);
            attempt
                .lossiness
                .retain(|flag| flag != "missing_repo_full_name");
        }
        if let Some(completed_at) = attempt.completed_at.as_deref().and_then(rfc3339_to_nanos) {
            let advance = page
                .advance_completed_at_nanos
                .zip(page.advance_workflow_run_id)
                .is_none_or(|(prev_ts, prev_run)| {
                    completed_at > prev_ts
                        || (completed_at == prev_ts && attempt.workflow_run_id > prev_run)
                });
            if advance {
                page.advance_completed_at_nanos = Some(completed_at);
                page.advance_workflow_run_id = Some(attempt.workflow_run_id);
            }
        }
        page.attempts.push(attempt);
    }
    page
}

/// Whether a durable cursor should skip a job (already covered by prior ticks).
#[must_use]
pub fn is_after_backfill_cursor(
    completed_at_nanos: u128,
    workflow_run_id: i64,
    cursor_completed_at_nanos: u128,
    cursor_workflow_run_id: i64,
) -> bool {
    completed_at_nanos > cursor_completed_at_nanos
        || (completed_at_nanos == cursor_completed_at_nanos
            && workflow_run_id > cursor_workflow_run_id)
}

/// Linkage-only hypothesis statement for a CI attempt next to an issue window.
#[must_use]
pub fn ci_adjacency_statement(attempt: &NormalizedCiAttempt) -> String {
    let name = attempt.name.as_deref().unwrap_or("job");
    let conclusion = attempt.conclusion.as_deref().unwrap_or("unknown");
    format!(
        "CI attempt `{name}` on {} (run {}, attempt {}, conclusion={conclusion}) is \
         adjacent timing evidence only — {CI_ADJACENCY_CLAIM_WORDING}",
        attempt.repo_full_name, attempt.workflow_run_id, attempt.attempt
    )
}

fn rfc3339_to_nanos(raw: &str) -> Option<u128> {
    let parsed =
        time::OffsetDateTime::parse(raw, &time::format_description::well_known::Rfc3339).ok()?;
    u128::try_from(parsed.unix_timestamp_nanos()).ok()
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

/// Build a flaky claim only from mixed outcomes across distinct attempt IDs.
pub fn flaky_claim_from_attempts(
    test_name: &str,
    attempts: &[(String, bool)],
) -> Result<FlakyClaimEvidence, FlakyClaimError> {
    let mut canonical = BTreeMap::new();
    for (attempt_id, passed) in attempts {
        if canonical
            .insert(attempt_id.as_str(), *passed)
            .is_some_and(|previous| previous != *passed)
        {
            return Err(FlakyClaimError::ConflictingAttempt);
        }
    }
    if canonical.len() < 2 {
        return Err(FlakyClaimError::InsufficientAttempts);
    }
    let fail_count = u32::try_from(canonical.values().filter(|passed| !**passed).count())
        .map_err(|_| FlakyClaimError::TooManyAttempts)?;
    let pass_count = u32::try_from(canonical.values().filter(|passed| **passed).count())
        .map_err(|_| FlakyClaimError::TooManyAttempts)?;
    if fail_count < 1 || pass_count < 1 {
        // Fail-only multi-attempt is broken, not flaky; pass-only is healthy.
        // Still require both sides for the flaky claim path.
        if canonical.len() == 2
            && fail_count + pass_count == 2
            && (fail_count == 0 || pass_count == 0)
        {
            // two same-side outcomes without mix — not flaky
        }
        if fail_count == 0 || pass_count == 0 {
            return Err(FlakyClaimError::SingleAttemptOnly);
        }
    }
    // Explicit: a lone fail→retry-pass pair with only one fail is flaky only if
    // we observed ≥1 fail and ≥1 pass across ≥2 attempts.
    let attempt_ids = canonical.keys().map(|id| (*id).to_owned()).collect();
    Ok(FlakyClaimEvidence {
        test_name: test_name.to_owned(),
        attempt_ids,
        fail_count,
        pass_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn normalizes_workflow_job_webhook() {
        let body = json!({
            "repository": { "full_name": "tailrocks/parallax" },
            "workflow_job": {
                "id": 99,
                "run_id": 1001,
                "run_attempt": 2,
                "name": "test",
                "conclusion": "failure",
                "html_url": "https://github.com/tailrocks/parallax/actions/runs/1001/job/99",
                "started_at": "2026-07-17T00:00:00Z",
                "completed_at": "2026-07-17T00:01:00Z"
            }
        });
        let row = normalize_workflow_job("workflow_job", &body).expect("job");
        assert_eq!(row.provider, PROVIDER);
        assert_eq!(row.repo_full_name, "tailrocks/parallax");
        assert_eq!(row.workflow_run_id, 1001);
        assert_eq!(row.job_id, 99);
        assert_eq!(row.attempt, 2);
        assert_eq!(row.conclusion.as_deref(), Some("failure"));
        assert!(row.html_url_present);
        assert!(attempt_identity(&row).contains(":2"));
    }

    #[test]
    fn rejects_unknown_event() {
        assert!(normalize_workflow_job("push", &json!({})).is_none());
    }

    #[test]
    fn flaky_requires_mixed_multi_attempt() {
        assert_eq!(
            flaky_claim_from_attempts("t", &[("a1".into(), false)]),
            Err(FlakyClaimError::InsufficientAttempts)
        );
        assert_eq!(
            flaky_claim_from_attempts("t", &[("a1".into(), false), ("a2".into(), false)]),
            Err(FlakyClaimError::SingleAttemptOnly)
        );
        let claim =
            flaky_claim_from_attempts("t", &[("a1".into(), false), ("a2".into(), true)]).unwrap();
        assert_eq!(claim.fail_count, 1);
        assert_eq!(claim.pass_count, 1);
        assert_eq!(claim.attempt_ids.len(), 2);
    }

    #[test]
    fn flaky_attempt_identity_is_redelivery_safe_and_canonical() {
        assert_eq!(
            flaky_claim_from_attempts("t", &[("a1".into(), false), ("a1".into(), false)]),
            Err(FlakyClaimError::InsufficientAttempts)
        );
        assert_eq!(
            flaky_claim_from_attempts("t", &[("a1".into(), false), ("a1".into(), true)]),
            Err(FlakyClaimError::ConflictingAttempt)
        );

        let claim = flaky_claim_from_attempts(
            "t",
            &[
                ("a2".into(), true),
                ("a1".into(), false),
                ("a2".into(), true),
            ],
        )
        .expect("distinct fail then pass is flaky");
        assert_eq!(claim.attempt_ids, ["a1", "a2"]);
        assert_eq!((claim.fail_count, claim.pass_count), (1, 1));
    }

    #[test]
    fn log_fingerprint_is_stable() {
        assert_eq!(
            redacted_log_fingerprint(b"redacted"),
            redacted_log_fingerprint(b"redacted")
        );
        assert_ne!(
            redacted_log_fingerprint(b"a"),
            redacted_log_fingerprint(b"b")
        );
    }

    #[test]
    fn parses_rest_jobs_page_and_advances_cursor() {
        let body = json!({
            "jobs": [
                {
                    "id": 1,
                    "run_id": 50,
                    "run_attempt": 1,
                    "name": "unit",
                    "conclusion": "success",
                    "html_url": "https://github.com/tailrocks/parallax/actions/runs/50/job/1",
                    "completed_at": "2026-07-17T01:00:00Z"
                },
                {
                    "id": 2,
                    "run_id": 51,
                    "run_attempt": 2,
                    "name": "unit",
                    "conclusion": "failure",
                    "html_url": "https://github.com/tailrocks/parallax/actions/runs/51/job/2",
                    "completed_at": "2026-07-17T02:00:00Z"
                }
            ]
        });
        let page = parse_rest_jobs_page("tailrocks/parallax", &body);
        assert_eq!(page.attempts.len(), 2);
        assert_eq!(page.advance_workflow_run_id, Some(51));
        assert!(page.advance_completed_at_nanos.is_some());
        assert!(is_after_backfill_cursor(
            page.advance_completed_at_nanos.unwrap(),
            51,
            0,
            0
        ));
        assert!(!is_after_backfill_cursor(1, 1, 2, 0));
        assert!(ci_adjacency_statement(&page.attempts[0]).contains("adjacent"));
        assert!(CI_CLAIM_KEYS.contains(&"rest_backfill_rate_aware"));
    }
}
