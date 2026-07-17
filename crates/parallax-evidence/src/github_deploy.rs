//! GitHub deploy/change context adapter — signature verify + normalize
//! (plan 121, first slice).
//!
//! Pure: no HTTP, Turso, or network. Callers supply raw webhook body bytes and
//! headers. Provider text is untrusted evidence, never policy.

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub const PROVIDER: &str = "github";
pub const API_VERSION_DEFAULT: &str = "2022-11-28";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureError {
    MissingHeader,
    MalformedHeader,
    InvalidHex,
    Mismatch,
    EmptySecret,
}

impl SignatureError {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingHeader => "missing_signature",
            Self::MalformedHeader => "malformed_signature",
            Self::InvalidHex => "invalid_signature_hex",
            Self::Mismatch => "signature_mismatch",
            Self::EmptySecret => "empty_webhook_secret",
        }
    }
}

/// Verify GitHub `X-Hub-Signature-256: sha256=<hex>` over the raw body.
///
/// Constant-time compare of digests. Empty secret fails closed.
pub fn verify_signature_256(
    secret: &[u8],
    body: &[u8],
    signature_header: Option<&str>,
) -> Result<(), SignatureError> {
    if secret.is_empty() {
        return Err(SignatureError::EmptySecret);
    }
    let header = signature_header
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or(SignatureError::MissingHeader)?;
    let hex = header
        .strip_prefix("sha256=")
        .ok_or(SignatureError::MalformedHeader)?;
    if hex.len() != 64 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(SignatureError::InvalidHex);
    }
    let mut expected = [0u8; 32];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        expected[i] = parse_hex_byte(chunk).ok_or(SignatureError::InvalidHex)?;
    }
    let mut mac =
        HmacSha256::new_from_slice(secret).map_err(|_| SignatureError::EmptySecret)?;
    mac.update(body);
    let digest = mac.finalize().into_bytes();
    if !constant_time_eq(digest.as_slice(), &expected) {
        return Err(SignatureError::Mismatch);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeployState {
    Requested,
    Queued,
    Pending,
    InProgress,
    Success,
    Failure,
    Error,
    Inactive,
    Unknown,
}

impl DeployState {
    fn parse(raw: &str) -> Self {
        match raw {
            "requested" => Self::Requested,
            "queued" => Self::Queued,
            "pending" => Self::Pending,
            "in_progress" => Self::InProgress,
            "success" => Self::Success,
            "failure" => Self::Failure,
            "error" => Self::Error,
            "inactive" => Self::Inactive,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedDeploy {
    pub provider: String,
    pub delivery_event: String,
    pub deployment_id: i64,
    pub repo_full_name: Option<String>,
    pub ref_name: Option<String>,
    pub commit_sha: Option<String>,
    pub environment: Option<String>,
    pub state: DeployState,
    pub task: Option<String>,
    pub actor_login: Option<String>,
    pub description_present: bool,
    pub log_url_present: bool,
    pub created_at: Option<String>,
    pub edge_strength: EdgeStrength,
    pub lossiness: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeStrength {
    Strong,
    Medium,
    Weak,
    Missing,
}

/// Normalize a GitHub `deployment` or `deployment_status` webhook JSON body.
#[must_use]
pub fn normalize_deploy_webhook(event_name: &str, body: &Value) -> Option<NormalizedDeploy> {
    let object = body.as_object()?;
    let (delivery_event, deployment, status_state) = match event_name {
        "deployment" => {
            let deployment = object.get("deployment").and_then(Value::as_object)?;
            ("deployment", deployment, None)
        }
        "deployment_status" => {
            let deployment = object.get("deployment").and_then(Value::as_object)?;
            let status = object.get("deployment_status").and_then(Value::as_object);
            let state = status
                .and_then(|s| s.get("state"))
                .and_then(Value::as_str)
                .map(DeployState::parse);
            ("deployment_status", deployment, state)
        }
        _ => return None,
    };

    let deployment_id = deployment.get("id").and_then(Value::as_i64)?;
    let mut lossiness = Vec::new();
    let repo_full_name = object
        .get("repository")
        .and_then(|r| r.get("full_name"))
        .and_then(Value::as_str)
        .map(str::to_string);
    if repo_full_name.is_none() {
        lossiness.push("repo_missing".into());
    }

    let ref_name = deployment
        .get("ref")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let commit_sha = deployment
        .get("sha")
        .and_then(Value::as_str)
        .map(normalize_sha)
        .filter(|s| s.len() == 40);
    if commit_sha.is_none() {
        lossiness.push("commit_sha_missing".into());
    }

    let environment = deployment
        .get("environment")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    if environment.is_none() {
        lossiness.push("environment_missing".into());
    }

    // deployment.create has no final state; treat as requested.
    let state = status_state.unwrap_or(DeployState::Requested);
    if matches!(state, DeployState::Unknown) {
        lossiness.push("unknown_state".into());
    }

    let task = deployment
        .get("task")
        .and_then(Value::as_str)
        .map(str::to_string);
    let actor_login = object
        .get("sender")
        .and_then(|s| s.get("login"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let description_present = deployment
        .get("description")
        .and_then(Value::as_str)
        .is_some_and(|s| !s.is_empty());
    let log_url_present = object
        .get("deployment_status")
        .and_then(|s| s.get("log_url"))
        .and_then(Value::as_str)
        .is_some_and(|s| !s.is_empty());
    let created_at = deployment
        .get("created_at")
        .and_then(Value::as_str)
        .map(str::to_string);

    // Description text is untrusted — never copy into the normalized record.
    if description_present {
        lossiness.push("description_ref_only".into());
    }

    let edge_strength = classify_edge(commit_sha.as_deref(), environment.as_deref());

    Some(NormalizedDeploy {
        provider: PROVIDER.into(),
        delivery_event: delivery_event.into(),
        deployment_id,
        repo_full_name,
        ref_name,
        commit_sha,
        environment,
        state,
        task,
        actor_login,
        description_present,
        log_url_present,
        created_at,
        edge_strength,
        lossiness,
    })
}

fn classify_edge(commit_sha: Option<&str>, environment: Option<&str>) -> EdgeStrength {
    match (commit_sha, environment) {
        (Some(_), Some(_)) => EdgeStrength::Strong,
        (Some(_), None) | (None, Some(_)) => EdgeStrength::Medium,
        (None, None) => EdgeStrength::Missing,
    }
}

fn normalize_sha(raw: &str) -> String {
    raw.chars()
        .filter(|c| *c != '-')
        .flat_map(char::to_lowercase)
        .collect()
}

fn parse_hex_byte(chunk: &[u8]) -> Option<u8> {
    if chunk.len() != 2 {
        return None;
    }
    let hi = hex_nibble(chunk[0])?;
    let lo = hex_nibble(chunk[1])?;
    Some((hi << 4) | lo)
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn verifies_known_github_signature() {
        // GitHub docs example style: secret + body → sha256 header.
        let secret = b"It's a Secret to Everybody";
        let body = b"Hello, World!";
        let mut mac = HmacSha256::new_from_slice(secret).unwrap();
        mac.update(body);
        let hex: String = mac
            .finalize()
            .into_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let header = format!("sha256={hex}");
        verify_signature_256(secret, body, Some(&header)).expect("valid signature");
        assert_eq!(
            verify_signature_256(secret, body, Some("sha256=00")).unwrap_err(),
            SignatureError::InvalidHex
        );
        assert_eq!(
            verify_signature_256(secret, b"tampered", Some(&header)).unwrap_err(),
            SignatureError::Mismatch
        );
        assert_eq!(
            verify_signature_256(secret, body, None).unwrap_err(),
            SignatureError::MissingHeader
        );
        assert_eq!(
            verify_signature_256(b"", body, Some(&header)).unwrap_err(),
            SignatureError::EmptySecret
        );
    }

    #[test]
    fn normalizes_deployment_event() {
        let body = json!({
            "deployment": {
                "id": 42,
                "ref": "main",
                "sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "environment": "production",
                "task": "deploy",
                "description": "ship it with secrets",
                "created_at": "2026-07-17T00:00:00Z"
            },
            "repository": {"full_name": "tailrocks/parallax"},
            "sender": {"login": "octocat", "email": "octocat@example.com"}
        });
        let row = normalize_deploy_webhook("deployment", &body).expect("row");
        assert_eq!(row.deployment_id, 42);
        assert_eq!(row.repo_full_name.as_deref(), Some("tailrocks/parallax"));
        assert_eq!(
            row.commit_sha.as_deref(),
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
        assert_eq!(row.environment.as_deref(), Some("production"));
        assert_eq!(row.state, DeployState::Requested);
        assert_eq!(row.edge_strength, EdgeStrength::Strong);
        assert_eq!(row.actor_login.as_deref(), Some("octocat"));
        assert!(row.description_present);
        let encoded = serde_json::to_string(&row).expect("json");
        assert!(!encoded.contains("ship it"));
        assert!(!encoded.contains("octocat@example.com"));
    }

    #[test]
    fn deployment_status_sets_state_and_medium_without_sha() {
        let body = json!({
            "deployment": {
                "id": 7,
                "ref": "main",
                "environment": "staging"
            },
            "deployment_status": {
                "state": "success",
                "log_url": "https://example.invalid/log"
            },
            "repository": {"full_name": "tailrocks/parallax"}
        });
        let row = normalize_deploy_webhook("deployment_status", &body).expect("row");
        assert_eq!(row.state, DeployState::Success);
        assert!(row.log_url_present);
        assert_eq!(row.edge_strength, EdgeStrength::Medium);
        assert!(row.lossiness.iter().any(|r| r == "commit_sha_missing"));
    }

    #[test]
    fn rejects_unknown_event_name() {
        assert!(normalize_deploy_webhook("push", &json!({})).is_none());
    }
}
