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
use parallax_evidence::github_deploy::{
    DeployState, EdgeStrength, normalize_deploy_webhook, verify_signature_256,
};
use parallax_metadata::{
    DeployAccept, DeployDeliveryRecord, DeployStoreError, TursoMetadataStore, payload_sha256_hex,
};
use std::sync::Arc;

use crate::config::GithubDeployConfig;

const MAX_WEBHOOK_BODY: usize = 256 * 1024;

#[derive(Clone)]
pub(crate) struct GithubWebhookState {
    pub config: GithubDeployConfig,
    pub secret: Option<String>,
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
    if !state.config.enabled {
        return (StatusCode::NOT_FOUND, "github deploy webhook disabled").into_response();
    }
    let Some(secret) = state.secret.as_deref() else {
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

    let event_name = headers
        .get("x-github-event")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    if !matches!(event_name, "deployment" | "deployment_status") {
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
    let Some(normalized) = normalize_deploy_webhook(event_name, &body_json) else {
        return (StatusCode::BAD_REQUEST, "unsupported deploy payload").into_response();
    };

    let received_at_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
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
        payload_hash: payload_sha256_hex(&body),
        received_at_nanos,
    };

    match metadata.accept_deploy_delivery(&record).await {
        Ok(DeployAccept::Inserted | DeployAccept::Duplicate) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            r#"{"ok":true}"#,
        )
            .into_response(),
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

    #[tokio::test]
    async fn disabled_route_is_not_found() {
        let router = router(GithubWebhookState {
            config: GithubDeployConfig::default(),
            secret: Some("s".into()),
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
            config: GithubDeployConfig {
                enabled: true,
                webhook_secret: "secret".into(),
            },
            secret: Some("secret".into()),
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
            config: GithubDeployConfig {
                enabled: true,
                webhook_secret: secret.into(),
            },
            secret: Some(secret.into()),
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
}
