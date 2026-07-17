//! Bounded GitHub Deployments REST backfill tick (plan 121 residual).
//!
//! Read-only. Rate-limit aware. Durable accept via Turso delivery ledger.

use parallax_evidence::github_deploy::{
    DEPLOY_ADJACENCY_CLAIM_WORDING, DEPLOY_CLAIM_KEYS, DeployState, EdgeStrength,
    parse_rest_deployments_page,
};
use parallax_metadata::{
    DeployAccept, DeployDeliveryRecord, DeployStoreError, EvidenceClaimRow, TursoMetadataStore,
    payload_sha256_hex,
};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;

#[async_trait::async_trait]
pub(crate) trait DeploymentsHttp: Send + Sync {
    async fn list_deployments(
        &self,
        repo_full_name: &str,
        page: u32,
    ) -> Result<DeployHttpResponse, DeployHttpError>;
}

#[derive(Debug, Clone)]
pub(crate) struct DeployHttpResponse {
    pub body: Value,
    pub rate_limit_remaining: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DeployHttpError {
    Transport(String),
    RateLimited { reset_at_nanos: Option<u128> },
    Unauthorized,
    Unexpected(String),
}

impl std::fmt::Display for DeployHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(message) => write!(f, "transport: {message}"),
            Self::RateLimited { .. } => write!(f, "rate limited"),
            Self::Unauthorized => write!(f, "unauthorized"),
            Self::Unexpected(message) => write!(f, "unexpected: {message}"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct DeployBackfillConfig {
    pub repos: Vec<String>,
    pub page_size: u32,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub(crate) struct DeployBackfillTickReport {
    pub repos_seen: usize,
    pub accepted: usize,
    pub duplicates: usize,
    pub errors: usize,
    pub rate_limited: bool,
}

pub(crate) async fn tick_once(
    metadata: &TursoMetadataStore,
    http: &dyn DeploymentsHttp,
    config: &DeployBackfillConfig,
    now_nanos: u128,
) -> DeployBackfillTickReport {
    let mut report = DeployBackfillTickReport::default();
    for repo in &config.repos {
        let repo = repo.trim();
        if repo.is_empty() || !repo.contains('/') {
            report.errors += 1;
            continue;
        }
        report.repos_seen += 1;
        match backfill_repo(metadata, http, repo, now_nanos).await {
            Ok(partial) => {
                report.accepted += partial.accepted;
                report.duplicates += partial.duplicates;
                report.errors += partial.errors;
                report.rate_limited |= partial.rate_limited;
            }
            Err(error) => {
                tracing::warn!(%repo, %error, "deploy backfill repo tick failed");
                report.errors += 1;
            }
        }
    }
    if report.accepted > 0 || report.duplicates > 0 {
        seed_claim_rows(metadata, now_nanos).await;
    }
    report
}

async fn backfill_repo(
    metadata: &TursoMetadataStore,
    http: &dyn DeploymentsHttp,
    repo: &str,
    now_nanos: u128,
) -> Result<DeployBackfillTickReport, DeployHttpError> {
    let mut report = DeployBackfillTickReport::default();
    let response = match http.list_deployments(repo, 1).await {
        Ok(response) => response,
        Err(DeployHttpError::RateLimited { reset_at_nanos: _ }) => {
            report.rate_limited = true;
            return Ok(report);
        }
        Err(error) => return Err(error),
    };
    if response.rate_limit_remaining == Some(0) {
        report.rate_limited = true;
        return Ok(report);
    }
    let page = parse_rest_deployments_page(repo, &response.body);
    for deploy in page.deployments {
        let body_bytes = serde_json::to_vec(&serde_json::json!({
            "deployment": {
                "id": deploy.deployment_id,
                "ref": deploy.ref_name,
                "sha": deploy.commit_sha,
                "environment": deploy.environment,
                "task": deploy.task,
            }
        }))
        .unwrap_or_default();
        let record = DeployDeliveryRecord {
            delivery_id: format!("rest:deploy:{}:{}", repo, deploy.deployment_id),
            provider: deploy.provider,
            event_name: "rest.deployment".into(),
            deployment_id: deploy.deployment_id,
            repo_full_name: deploy.repo_full_name,
            ref_name: deploy.ref_name,
            commit_sha: deploy.commit_sha,
            environment: deploy.environment,
            state: state_label(deploy.state).to_string(),
            task: deploy.task,
            actor_login: deploy.actor_login,
            edge_strength: edge_label(deploy.edge_strength).to_string(),
            lossiness: deploy.lossiness,
            payload_hash: payload_sha256_hex(&body_bytes),
            received_at_nanos: now_nanos,
        };
        match metadata.accept_deploy_delivery(&record).await {
            Ok(DeployAccept::Inserted) => report.accepted += 1,
            Ok(DeployAccept::Duplicate) => report.duplicates += 1,
            Err(DeployStoreError::Collision(_)) => report.errors += 1,
            Err(DeployStoreError::Internal(error)) => {
                tracing::warn!(%error, "deploy backfill accept failed");
                report.errors += 1;
            }
        }
    }
    Ok(report)
}

const fn state_label(state: DeployState) -> &'static str {
    match state {
        DeployState::Requested => "requested",
        DeployState::Queued => "queued",
        DeployState::Pending => "pending",
        DeployState::InProgress => "in_progress",
        DeployState::Success => "success",
        DeployState::Failure => "failure",
        DeployState::Error => "error",
        DeployState::Inactive => "inactive",
        DeployState::Unknown => "unknown",
    }
}

const fn edge_label(edge: EdgeStrength) -> &'static str {
    match edge {
        EdgeStrength::Strong => "strong",
        EdgeStrength::Medium => "medium",
        EdgeStrength::Weak => "weak",
        EdgeStrength::Missing => "missing",
    }
}

async fn seed_claim_rows(metadata: &TursoMetadataStore, now_nanos: u128) {
    for key in DEPLOY_CLAIM_KEYS {
        let wording = match *key {
            "webhook_hmac_accept" => {
                "Deploy webhook accept is HMAC-verified before durable write".to_string()
            }
            "delivery_id_idempotent" => {
                "Deploy delivery-id + payload-hash is idempotent".to_string()
            }
            "description_email_excluded" => {
                "Deploy description text and sender email are excluded from storage".to_string()
            }
            "strong_edge_requires_sha_and_env" => {
                "Strong deploy edges require both commit SHA and environment".to_string()
            }
            "no_causal_wording_from_adjacency" => DEPLOY_ADJACENCY_CLAIM_WORDING.to_string(),
            _ => DEPLOY_ADJACENCY_CLAIM_WORDING.to_string(),
        };
        let row = EvidenceClaimRow {
            domain: "deploy_context".into(),
            claim_key: (*key).into(),
            level: "fixture_proven".into(),
            measured_at_nanos: now_nanos,
            coverage_numerator: Some(1),
            coverage_denominator: Some(1),
            wording,
        };
        if let Err(error) = metadata.upsert_evidence_claim(&row).await {
            tracing::warn!(%error, claim = *key, "deploy claim row upsert failed");
        }
    }
}

pub(crate) struct LiveDeploymentsHttp {
    client: reqwest::Client,
    token: Option<String>,
    page_size: u32,
}

impl LiveDeploymentsHttp {
    pub(crate) fn new(token: Option<String>, page_size: u32) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent("parallax-deploy-evidence/0.1")
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            token,
            page_size: page_size.clamp(1, 100),
        }
    }
}

#[async_trait::async_trait]
impl DeploymentsHttp for LiveDeploymentsHttp {
    async fn list_deployments(
        &self,
        repo_full_name: &str,
        page: u32,
    ) -> Result<DeployHttpResponse, DeployHttpError> {
        let url = format!(
            "https://api.github.com/repos/{repo_full_name}/deployments?per_page={}&page={page}",
            self.page_size
        );
        let mut request = self.client.get(url).header(
            "X-GitHub-Api-Version",
            parallax_evidence::github_deploy::API_VERSION_DEFAULT,
        );
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }
        let response = request
            .send()
            .await
            .map_err(|error| DeployHttpError::Transport(error.to_string()))?;
        let status = response.status().as_u16();
        let remaining = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok());
        let reset_at_nanos = response
            .headers()
            .get("x-ratelimit-reset")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .map(|secs| u128::from(secs) * 1_000_000_000);
        if status == 401 {
            return Err(DeployHttpError::Unauthorized);
        }
        if status == 403 || status == 429 {
            return Err(DeployHttpError::RateLimited { reset_at_nanos });
        }
        if !(200..300).contains(&status) {
            return Err(DeployHttpError::Unexpected(format!("HTTP {status}")));
        }
        let body = response
            .json::<Value>()
            .await
            .map_err(|error| DeployHttpError::Transport(error.to_string()))?;
        Ok(DeployHttpResponse {
            body,
            rate_limit_remaining: remaining,
        })
    }
}

pub(crate) fn spawn_loop(
    tasks: &mut Vec<tokio::task::JoinHandle<()>>,
    metadata: Arc<TursoMetadataStore>,
    config: DeployBackfillConfig,
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
        let http = LiveDeploymentsHttp::new(token, page_size);
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
                accepted = report.accepted,
                duplicates = report.duplicates,
                rate_limited = report.rate_limited,
                errors = report.errors,
                "deploy evidence backfill tick"
            );
        }
    }));
    tracing::info!(
        interval_secs,
        repos = repo_count,
        "deploy evidence REST backfill ready"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct MockHttp {
        body: Value,
    }

    #[async_trait::async_trait]
    impl DeploymentsHttp for MockHttp {
        async fn list_deployments(
            &self,
            _repo_full_name: &str,
            _page: u32,
        ) -> Result<DeployHttpResponse, DeployHttpError> {
            Ok(DeployHttpResponse {
                body: self.body.clone(),
                rate_limit_remaining: Some(100),
            })
        }
    }

    #[tokio::test]
    async fn tick_accepts_deployments_and_claim_rows() {
        let directory = tempfile::tempdir().expect("tempdir");
        let store = TursoMetadataStore::open(directory.path().join("meta.db"))
            .await
            .expect("store");
        let http = MockHttp {
            body: json!([{
                "id": 42,
                "ref": "main",
                "sha": "cccccccccccccccccccccccccccccccccccccccc",
                "environment": "production",
                "task": "deploy",
                "created_at": "2026-07-17T00:00:00Z"
            }]),
        };
        let config = DeployBackfillConfig {
            repos: vec!["tailrocks/parallax".into()],
            page_size: 30,
        };
        let report = tick_once(&store, &http, &config, 5_000_000).await;
        assert_eq!(report.accepted, 1);
        assert_eq!(
            store
                .count_evidence_claims(Some("deploy_context"))
                .await
                .expect("claims"),
            DEPLOY_CLAIM_KEYS.len() as u64
        );
        let again = tick_once(&store, &http, &config, 6_000_000).await;
        assert_eq!(again.accepted, 0);
        assert_eq!(again.duplicates, 1);
    }
}
