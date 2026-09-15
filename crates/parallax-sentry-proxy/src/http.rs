//! Axum HTTP surface: envelope + legacy store ingest.

use axum::Router;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Path, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use parallax_ingest::{EnvelopeOutcome, RejectReason, parse_envelope};
use tower_http::cors::{Any, CorsLayer};
use tower_http::decompression::RequestDecompressionLayer;

use crate::config::IngressConfig;
use crate::dsn::{rewrite_envelope_dsn, rewrite_sentry_auth};
use crate::fanout::{DeliveryJob, FanOut};

pub const MAX_ENVELOPE_BODY: usize = 1_048_576;

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: std::sync::Arc<crate::config::Config>,
    pub fanout: std::sync::Arc<FanOut>,
}

pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::POST, Method::OPTIONS])
        .allow_headers(Any)
        .allow_origin(Any);

    Router::new()
        .route("/health", get(health))
        .route("/api/{project_id}/envelope/", post(envelope))
        .route("/api/{project_id}/envelope", post(envelope))
        .route("/api/{project_id}/store/", post(legacy_store))
        .layer(DefaultBodyLimit::max(MAX_ENVELOPE_BODY))
        .layer(RequestDecompressionLayer::new())
        .layer(cors)
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn envelope(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if body.is_empty() {
        return reject_response(RejectReason::EmptyInput);
    }
    if body.len() > MAX_ENVELOPE_BODY {
        return (StatusCode::PAYLOAD_TOO_LARGE, "envelope too large").into_response();
    }

    let Some(ingress) = authorize(&state, &project_id, &headers) else {
        return (StatusCode::UNAUTHORIZED, "unknown project").into_response();
    };

    let parsed = match parse_envelope(&body) {
        EnvelopeOutcome::Rejected { reason } => return reject_response(reason),
        EnvelopeOutcome::Accepted { event_id, .. } => event_id,
    };

    let auth_template = extract_sentry_auth(&headers).unwrap_or_default();
    let jobs = build_envelope_jobs(ingress, &auth_template, &body);
    state.fanout.dispatch(jobs);
    ok_event_id(&parsed)
}

async fn legacy_store(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if body.is_empty() {
        return (StatusCode::BAD_REQUEST, "empty store payload").into_response();
    }
    if body.len() > MAX_ENVELOPE_BODY {
        return (StatusCode::PAYLOAD_TOO_LARGE, "store payload too large").into_response();
    }

    let Some(ingress) = authorize(&state, &project_id, &headers) else {
        return (StatusCode::UNAUTHORIZED, "unknown project").into_response();
    };

    let event_id = match serde_json::from_slice::<serde_json::Value>(&body) {
        Ok(value) => value
            .get("event_id")
            .and_then(serde_json::Value::as_str)
            .map(normalize_event_id)
            .filter(|id| id.len() == 32 && id.chars().all(|c| c.is_ascii_hexdigit()))
            .unwrap_or_else(|| "00000000000000000000000000000000".to_string()),
        Err(_) => return (StatusCode::BAD_REQUEST, "malformed store JSON").into_response(),
    };

    let auth_template = extract_sentry_auth(&headers).unwrap_or_default();
    let jobs = build_store_jobs(ingress, &auth_template, &body);
    state.fanout.dispatch(jobs);
    ok_event_id(&event_id)
}

fn authorize<'a>(
    state: &'a AppState,
    project_id: &str,
    headers: &HeaderMap,
) -> Option<&'a IngressConfig> {
    let provided = extract_sentry_key(headers)?;
    state.config.resolve_ingress(project_id, &provided)
}

fn build_envelope_jobs(
    ingress: &IngressConfig,
    auth_template: &str,
    body: &Bytes,
) -> Vec<DeliveryJob> {
    ingress
        .destinations
        .enabled()
        .into_iter()
        .filter_map(|(kind, dest)| {
            let rewritten = rewrite_envelope_dsn(body, &dest).ok()?;
            Some(DeliveryJob {
                destination: kind,
                url: dest.envelope_url(),
                auth: rewrite_sentry_auth(auth_template, &dest.public_key),
                body: Bytes::from(rewritten),
                content_type: "application/x-sentry-envelope",
            })
        })
        .collect()
}

fn build_store_jobs(
    ingress: &IngressConfig,
    auth_template: &str,
    body: &Bytes,
) -> Vec<DeliveryJob> {
    ingress
        .destinations
        .enabled()
        .into_iter()
        .map(|(kind, dest)| DeliveryJob {
            destination: kind,
            url: dest.store_url(),
            auth: rewrite_sentry_auth(auth_template, &dest.public_key),
            body: body.clone(),
            content_type: "application/json",
        })
        .collect()
}

fn ok_event_id(event_id: &str) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        format!(r#"{{"id":"{event_id}"}}"#),
    )
        .into_response()
}

fn reject_response(reason: RejectReason) -> Response {
    let (status, message) = match reason {
        RejectReason::EnvelopeTooLarge
        | RejectReason::HeaderLineTooLarge
        | RejectReason::EventPayloadTooLarge
        | RejectReason::LengthOverflow
        | RejectReason::TooManyItems => (StatusCode::PAYLOAD_TOO_LARGE, reason.as_str()),
        RejectReason::NoEventItem => (StatusCode::UNSUPPORTED_MEDIA_TYPE, reason.as_str()),
        RejectReason::EmptyInput
        | RejectReason::MalformedEnvelopeHeader
        | RejectReason::MalformedItemHeader
        | RejectReason::PrematureEof
        | RejectReason::TrailingGarbageAfterPayload
        | RejectReason::DuplicateEventItem
        | RejectReason::EventPayloadNotJson => (StatusCode::BAD_REQUEST, reason.as_str()),
    };
    (status, message).into_response()
}

fn extract_sentry_key(headers: &HeaderMap) -> Option<String> {
    extract_sentry_auth(headers).and_then(|raw| parse_sentry_auth(&raw))
}

fn extract_sentry_auth(headers: &HeaderMap) -> Option<String> {
    if let Some(value) = headers.get("x-sentry-auth").and_then(|v| v.to_str().ok()) {
        return Some(value.to_string());
    }
    if let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        let rest = value
            .strip_prefix("Sentry ")
            .or_else(|| value.strip_prefix("sentry "))?;
        return Some(rest.to_string());
    }
    None
}

fn parse_sentry_auth(raw: &str) -> Option<String> {
    for part in raw.split(',') {
        let part = part.trim();
        if let Some(key) = part
            .strip_prefix("sentry_key=")
            .or_else(|| part.strip_prefix("Sentry sentry_key="))
        {
            let key = key.trim().trim_matches('"');
            if !key.is_empty() {
                return Some(key.to_string());
            }
        }
    }
    None
}

fn normalize_event_id(raw: &str) -> String {
    raw.chars()
        .filter(|c| *c != '-')
        .collect::<String>()
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fanout::DestinationKind;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt;

    use crate::config::Config;

    fn sample_envelope() -> Vec<u8> {
        let event =
            br#"{"event_id":"9ec79c33ec9942ab8353589fcb2e04dc","message":"hello","level":"error"}"#;
        let mut body = Vec::new();
        body.extend_from_slice(
            br#"{"event_id":"9ec79c33ec9942ab8353589fcb2e04dc","dsn":"https://proxy-public-key@proxy/1"}"#,
        );
        body.push(b'\n');
        body.extend_from_slice(
            format!(r#"{{"type":"event","length":{}}}"#, event.len()).as_bytes(),
        );
        body.push(b'\n');
        body.extend_from_slice(event);
        body.push(b'\n');
        body
    }

    const TEST_CONFIG: &str = r#"
listen = "127.0.0.1:0"

[[ingress]]
project_id = "1"
public_key = "proxy-public-key"

[ingress.destinations.sentry]
enabled = true
dsn = "https://sentry-key@dest-sentry/2"

[ingress.destinations.parallax]
enabled = true
dsn = "https://parallax-key@dest-parallax/1"
"#;

    fn test_config() -> Config {
        toml::from_str(TEST_CONFIG).expect("test config")
    }

    fn test_state() -> AppState {
        let config = test_config();
        AppState {
            config: Arc::new(config.clone()),
            fanout: Arc::new(FanOut::spawn(&config)),
        }
    }

    fn auth_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-sentry-auth",
            header::HeaderValue::from_static("Sentry sentry_key=proxy-public-key"),
        );
        headers
    }

    #[test]
    fn build_envelope_jobs_rewrite_destinations() {
        let config = test_config();
        let ingress = config.ingress.first().expect("ingress");
        let body = Bytes::from(sample_envelope());
        let jobs = build_envelope_jobs(ingress, "Sentry sentry_key=proxy-public-key", &body);
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].destination, DestinationKind::Sentry);
        assert_eq!(jobs[0].url, "https://dest-sentry/api/2/envelope/");
        assert!(jobs[0].auth.contains("sentry_key=sentry-key"));
        let header_line = jobs[0].body.split(|&b| b == b'\n').next().unwrap();
        let header: serde_json::Value = serde_json::from_slice(header_line).unwrap();
        assert_eq!(header["dsn"], "https://sentry-key@dest-sentry/2");
    }

    #[tokio::test]
    async fn envelope_endpoint_accepts_valid_payload() {
        let state = test_state();
        let app = router(state);
        let mut request = Request::post("/api/1/envelope/")
            .body(Body::from(sample_envelope()))
            .unwrap();
        *request.headers_mut() = auth_headers();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn envelope_rejects_unknown_key() {
        let state = test_state();
        let app = router(state);
        let mut request = Request::post("/api/1/envelope/")
            .body(Body::from(sample_envelope()))
            .unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-sentry-auth",
            header::HeaderValue::from_static("Sentry sentry_key=wrong"),
        );
        *request.headers_mut() = headers;
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn parse_envelope_integration_accepts_fixture() {
        let body = sample_envelope();
        match parse_envelope(&body) {
            EnvelopeOutcome::Accepted { event_id, .. } => {
                assert_eq!(event_id, "9ec79c33ec9942ab8353589fcb2e04dc");
            }
            other => panic!("expected accept, got {other:?}"),
        }
    }
}
