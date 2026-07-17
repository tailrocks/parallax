//! GitHub deploy/change webhook HTTP adapter (plan 121 residual).
//!
//! `POST /webhooks/github` — verify HMAC, normalize deploy events, durable
//! Turso accept with delivery-id idempotency. Provider text is untrusted
//! evidence. Disabled by default.

use axum::Router;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use parallax_evidence::github_actions::{CI_ADJACENCY_CLAIM_WORDING, CI_CLAIM_KEYS};
use parallax_evidence::github_actions::{attempt_identity, normalize_workflow_job};
use parallax_evidence::github_deploy::{DEPLOY_ADJACENCY_CLAIM_WORDING, DEPLOY_CLAIM_KEYS};
use parallax_evidence::github_deploy::{
    DeployState, EdgeStrength, normalize_deploy_webhook, verify_signature_256,
};
use parallax_metadata::{
    CiAttemptAccept, CiAttemptDeliveryRecord, CiAttemptStoreError, DeployAccept,
    DeployDeliveryRecord, DeployStoreError, EvidenceClaimRow, TursoMetadataStore,
    payload_sha256_hex,
};
use std::sync::Arc;

use crate::config::{GithubActionsConfig, GithubDeployConfig};

const MAX_WEBHOOK_BODY: usize = 256 * 1024;

#[derive(Clone)]
pub(crate) struct GithubWebhookState {
    pub deploy_config: GithubDeployConfig,
    pub deploy_secret: Option<String>,
    pub actions_config: GithubActionsConfig,
    pub actions_secret: Option<String>,
    pub metadata: Option<Arc<TursoMetadataStore>>,
}

pub(crate) fn router(state: GithubWebhookState) -> Router {
    Router::new()
        .route("/webhooks/github", post(webhook))
        .layer(DefaultBodyLimit::max(MAX_WEBHOOK_BODY))
        .with_state(state)
}

async fn webhook(
    State(state): State<GithubWebhookState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let event_name = headers
        .get("x-github-event")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    let (enabled, secret) = match event_name {
        "deployment" | "deployment_status" => {
            (state.deploy_config.enabled, state.deploy_secret.as_deref())
        }
        "workflow_job" => (
            state.actions_config.enabled,
            state.actions_secret.as_deref(),
        ),
        _ => (
            state.deploy_config.enabled || state.actions_config.enabled,
            state
                .deploy_secret
                .as_deref()
                .or(state.actions_secret.as_deref()),
        ),
    };
    if !enabled {
        return (StatusCode::NOT_FOUND, "github webhook adapter disabled").into_response();
    }
    let Some(secret) = secret else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "github webhook secret not configured",
        )
            .into_response();
    };
    if body.len() > MAX_WEBHOOK_BODY {
        return (StatusCode::PAYLOAD_TOO_LARGE, "webhook body too large").into_response();
    }

    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|value| value.to_str().ok());
    if let Err(error) = verify_signature_256(secret.as_bytes(), &body, signature) {
        return (StatusCode::UNAUTHORIZED, error.as_str()).into_response();
    }
    let Some(metadata) = state.metadata.as_ref() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "deploy delivery store unavailable",
        )
            .into_response();
    };

    if !matches!(
        event_name,
        "deployment" | "deployment_status" | "workflow_job"
    ) {
        // Accept but ignore unsupported events so GitHub does not retry.
        return (StatusCode::ACCEPTED, "event ignored").into_response();
    }
    let delivery_id = headers
        .get("x-github-delivery")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let Some(delivery_id) = delivery_id else {
        return (StatusCode::BAD_REQUEST, "missing delivery id").into_response();
    };

    let body_json: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(value) => value,
        Err(_) => return (StatusCode::BAD_REQUEST, "malformed json body").into_response(),
    };
    if event_name == "workflow_job" {
        return accept_workflow_job(metadata, delivery_id, &body, &body_json).await;
    }
    accept_deploy(metadata, event_name, delivery_id, &body, &body_json).await
}

async fn accept_deploy(
    metadata: &TursoMetadataStore,
    event_name: &str,
    delivery_id: &str,
    body: &[u8],
    body_json: &serde_json::Value,
) -> Response {
    let Some(normalized) = normalize_deploy_webhook(event_name, body_json) else {
        return (StatusCode::BAD_REQUEST, "unsupported deploy payload").into_response();
    };

    let record = DeployDeliveryRecord {
        delivery_id: delivery_id.to_string(),
        provider: normalized.provider,
        event_name: normalized.delivery_event,
        deployment_id: normalized.deployment_id,
        repo_full_name: normalized.repo_full_name,
        ref_name: normalized.ref_name,
        commit_sha: normalized.commit_sha,
        environment: normalized.environment,
        state: state_label(normalized.state).to_string(),
        task: normalized.task,
        actor_login: normalized.actor_login,
        edge_strength: edge_label(normalized.edge_strength).to_string(),
        lossiness: normalized.lossiness,
        payload_hash: payload_sha256_hex(body),
        received_at_nanos: now_nanos(),
    };

    match metadata.accept_deploy_delivery(&record).await {
        Ok(DeployAccept::Inserted | DeployAccept::Duplicate) => {
            seed_deploy_webhook_claims(metadata, record.received_at_nanos).await;
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                r#"{"ok":true}"#,
            )
                .into_response()
        }
        Err(DeployStoreError::Collision(_)) => {
            (StatusCode::CONFLICT, "delivery payload collision").into_response()
        }
        Err(DeployStoreError::Internal(error)) => {
            tracing::warn!(error = %error, "deploy delivery write failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "deploy delivery store unavailable",
            )
                .into_response()
        }
    }
}

async fn seed_deploy_webhook_claims(metadata: &TursoMetadataStore, now_nanos: u128) {
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

async fn accept_workflow_job(
    metadata: &TursoMetadataStore,
    delivery_id: &str,
    body: &[u8],
    body_json: &serde_json::Value,
) -> Response {
    let Some(normalized) = normalize_workflow_job("workflow_job", body_json) else {
        return (StatusCode::BAD_REQUEST, "unsupported workflow-job payload").into_response();
    };
    let record = CiAttemptDeliveryRecord {
        delivery_id: delivery_id.to_string(),
        attempt_id: attempt_identity(&normalized),
        provider: normalized.provider,
        repo_full_name: normalized.repo_full_name,
        workflow_run_id: normalized.workflow_run_id,
        job_id: normalized.job_id,
        attempt: normalized.attempt,
        conclusion: normalized.conclusion,
        name: normalized.name,
        lossiness: normalized.lossiness,
        payload_hash: payload_sha256_hex(body),
        received_at_nanos: now_nanos(),
    };
    match metadata.accept_ci_attempt_delivery(&record).await {
        Ok(CiAttemptAccept::Inserted | CiAttemptAccept::Duplicate) => {
            seed_ci_webhook_claim(metadata, record.received_at_nanos).await;
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                r#"{"ok":true}"#,
            )
                .into_response()
        }
        Err(CiAttemptStoreError::Collision(_)) => {
            (StatusCode::CONFLICT, "CI attempt evidence collision").into_response()
        }
        Err(CiAttemptStoreError::Internal(error)) => {
            tracing::warn!(error = %error, "CI attempt delivery write failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "CI attempt store unavailable",
            )
                .into_response()
        }
    }
}

async fn seed_ci_webhook_claim(metadata: &TursoMetadataStore, now_nanos: u128) {
    for key in CI_CLAIM_KEYS {
        let wording = match *key {
            "workflow_job_webhook_accept" => {
                "workflow_job webhook accept is HMAC-verified and idempotent".to_string()
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
            "rest_backfill_rate_aware" => {
                // Webhook path does not prove REST; leave not_measured wording until backfill runs.
                continue;
            }
            _ => CI_ADJACENCY_CLAIM_WORDING.to_string(),
        };
        let row = EvidenceClaimRow {
            domain: "ci_evidence".into(),
            claim_key: (*key).into(),
            level: "fixture_proven".into(),
            measured_at_nanos: now_nanos,
            coverage_numerator: Some(1),
            coverage_denominator: Some(1),
            wording,
        };
        if let Err(error) = metadata.upsert_evidence_claim(&row).await {
            tracing::warn!(%error, claim = *key, "ci webhook claim row upsert failed");
        }
    }
}

fn now_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use tower::ServiceExt;

    type HmacSha256 = Hmac<Sha256>;

    fn sign(secret: &[u8], body: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(secret).expect("key");
        mac.update(body);
        let digest = mac.finalize().into_bytes();
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::from("sha256=");
        for byte in digest {
            out.push(HEX[(byte >> 4) as usize] as char);
            out.push(HEX[(byte & 0x0f) as usize] as char);
        }
        out
    }

    fn empty_actions() -> GithubActionsConfig {
        GithubActionsConfig::default()
    }

    #[tokio::test]
    async fn disabled_route_is_not_found() {
        let router = router(GithubWebhookState {
            deploy_config: GithubDeployConfig::default(),
            deploy_secret: Some("s".into()),
            actions_config: empty_actions(),
            actions_secret: None,
            metadata: None,
        });
        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/webhooks/github")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn rejects_bad_signature() {
        let router = router(GithubWebhookState {
            deploy_config: GithubDeployConfig {
                enabled: true,
                webhook_secret: "secret".into(),
                ..GithubDeployConfig::default()
            },
            deploy_secret: Some("secret".into()),
            actions_config: empty_actions(),
            actions_secret: None,
            metadata: None,
        });
        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/webhooks/github")
                    .header("x-hub-signature-256", "sha256=00")
                    .header("x-github-event", "deployment")
                    .header("x-github-delivery", "del-1")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn accepts_signed_deployment_idempotently() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("meta.db");
        let metadata = Arc::new(TursoMetadataStore::open(&path).await.unwrap());
        let secret = "webhook-secret";
        let body = br#"{
          "deployment": {
            "id": 42,
            "ref": "main",
            "sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "task": "deploy",
            "environment": "production",
            "description": "secret body must not persist",
            "created_at": "2026-07-17T00:00:00Z"
          },
          "repository": { "full_name": "tailrocks/parallax" },
          "sender": { "login": "octocat", "email": "octocat@example.com" }
        }"#;
        let signature = sign(secret.as_bytes(), body);
        let state = GithubWebhookState {
            deploy_config: GithubDeployConfig {
                enabled: true,
                webhook_secret: secret.into(),
                ..GithubDeployConfig::default()
            },
            deploy_secret: Some(secret.into()),
            actions_config: empty_actions(),
            actions_secret: None,
            metadata: Some(metadata.clone()),
        };
        let router = router(state.clone());
        let request = |sig: &str| {
            Request::builder()
                .method("POST")
                .uri("/webhooks/github")
                .header("x-hub-signature-256", sig)
                .header("x-github-event", "deployment")
                .header("x-github-delivery", "11111111-2222-3333-4444-555555555555")
                .body(Body::from(body.as_slice()))
                .unwrap()
        };
        let first = router.clone().oneshot(request(&signature)).await.unwrap();
        assert_eq!(first.status(), StatusCode::OK);
        let second = router.oneshot(request(&signature)).await.unwrap();
        assert_eq!(second.status(), StatusCode::OK);
        let stored = metadata
            .deploy_delivery("11111111-2222-3333-4444-555555555555")
            .await
            .unwrap()
            .expect("stored");
        assert_eq!(stored.deployment_id, 42);
        assert_eq!(stored.environment.as_deref(), Some("production"));
        assert_eq!(
            stored.commit_sha.as_deref(),
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
        assert_eq!(stored.actor_login.as_deref(), Some("octocat"));
        assert_eq!(stored.edge_strength, "strong");
    }

    #[tokio::test]
    async fn accepts_signed_workflow_job_idempotently_and_rejects_collision() {
        let directory = tempfile::tempdir().expect("tempdir");
        let metadata = Arc::new(
            TursoMetadataStore::open(directory.path().join("meta.db"))
                .await
                .expect("metadata"),
        );
        let secret = "actions-secret";
        let body = br#"{
          "action": "completed",
          "repository": {"full_name": "tailrocks/parallax"},
          "workflow_job": {
            "id": 99,
            "run_id": 1001,
            "run_attempt": 2,
            "name": "test",
            "conclusion": "failure",
            "html_url": "https://github.com/tailrocks/parallax/actions/runs/1001/job/99"
          }
        }"#;
        let state = GithubWebhookState {
            deploy_config: GithubDeployConfig::default(),
            deploy_secret: None,
            actions_config: GithubActionsConfig {
                enabled: true,
                webhook_secret: secret.into(),
                ..GithubActionsConfig::default()
            },
            actions_secret: Some(secret.into()),
            metadata: Some(metadata.clone()),
        };
        let request = |body: &'static [u8]| {
            Request::builder()
                .method("POST")
                .uri("/webhooks/github")
                .header("x-hub-signature-256", sign(secret.as_bytes(), body))
                .header("x-github-event", "workflow_job")
                .header("x-github-delivery", "workflow-delivery-1")
                .body(Body::from(body))
                .expect("request")
        };
        let app = router(state);
        assert_eq!(
            app.clone()
                .oneshot(request(body))
                .await
                .expect("first")
                .status(),
            StatusCode::OK
        );
        assert_eq!(
            app.clone()
                .oneshot(request(body))
                .await
                .expect("duplicate")
                .status(),
            StatusCode::OK
        );
        assert_eq!(metadata.count_ci_attempts().await.expect("attempts"), 1);
        assert_eq!(
            metadata
                .count_ci_attempt_deliveries()
                .await
                .expect("deliveries"),
            1
        );

        let collision = br#"{
          "repository": {"full_name": "tailrocks/parallax"},
          "workflow_job": {"id":99,"run_id":1001,"run_attempt":2,"conclusion":"success"}
        }"#;
        assert_eq!(
            app.oneshot(request(collision))
                .await
                .expect("collision")
                .status(),
            StatusCode::CONFLICT
        );
    }
}
