//! Sentry envelope HTTP adapter (plan 118).
//!
//! `POST /api/<project_id>/envelope/` — parse, project/public-key map,
//! normalize to `ErrorEventRow`, durable-spool the **normalized** record,
//! enqueue issue recording, then acknowledge.
//!
//! OTLP remains primary. No Greptime raw Sentry table.

use axum::Router;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use parallax_analysis::sentry::derive_from_sentry_event;
use parallax_ingest::{EnvelopeOutcome, RejectReason, parse_envelope};
use parallax_spool::Signal;
use tower_http::decompression::RequestDecompressionLayer;

use crate::config::SentryConfig;
use crate::ingest_runtime::IngestState;
use crate::worker::IngestItem;
use parallax_metadata::{SentryAck, SentryAckError, TursoMetadataStore};
use sha2::{Digest, Sha256};
use std::sync::Arc;

const MAX_ENVELOPE_BODY: usize = 1_048_576;

#[derive(Clone)]
pub(crate) struct SentryHttpState {
    pub ingest: IngestState,
    pub config: SentryConfig,
    pub public_key: Option<String>,
    /// Optional Turso ledger for event-id idempotency (plan 118 residual).
    pub metadata: Option<Arc<TursoMetadataStore>>,
}

pub(crate) fn router(state: SentryHttpState) -> Router {
    Router::new()
        .route("/api/{project_id}/envelope/", post(envelope))
        // Some SDKs omit the trailing slash.
        .route("/api/{project_id}/envelope", post(envelope))
        .layer(DefaultBodyLimit::max(MAX_ENVELOPE_BODY))
        .layer(RequestDecompressionLayer::new())
        .with_state(state)
}

async fn envelope(
    State(state): State<SentryHttpState>,
    Path(project_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if !state.config.enabled {
        return (StatusCode::NOT_FOUND, "sentry adapter disabled").into_response();
    }

    let encoding = headers
        .get(header::CONTENT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    // RequestDecompressionLayer already handled gzip. Reject other encodings
    // that would leave compressed frames for the pure parser.
    if !encoding.is_empty()
        && !encoding.eq_ignore_ascii_case("identity")
        && !encoding.eq_ignore_ascii_case("gzip")
    {
        return (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported content encoding",
        )
            .into_response();
    }

    if project_id != state.config.project_id {
        return (StatusCode::UNAUTHORIZED, "unknown project").into_response();
    }
    let Some(expected_key) = state.public_key.as_deref() else {
        return (StatusCode::UNAUTHORIZED, "sentry public key not configured").into_response();
    };
    let Some(provided) = extract_sentry_key(&headers) else {
        return (StatusCode::UNAUTHORIZED, "missing sentry public key").into_response();
    };
    if provided != expected_key {
        return (StatusCode::UNAUTHORIZED, "unknown project").into_response();
    }

    if body.len() > MAX_ENVELOPE_BODY {
        return (StatusCode::PAYLOAD_TOO_LARGE, "envelope too large").into_response();
    }

    match parse_envelope(&body) {
        EnvelopeOutcome::Rejected { reason } => reject_response(reason),
        EnvelopeOutcome::Accepted {
            event_id,
            event_json,
            unsupported_items: _,
        } => accept_event(&state, &project_id, event_id, event_json).await,
    }
}

async fn accept_event(
    state: &SentryHttpState,
    project_id: &str,
    event_id: String,
    event_json: serde_json::Value,
) -> Response {
    if event_id.is_empty() || !is_32_hex(&event_id) {
        return (
            StatusCode::BAD_REQUEST,
            "malformed event: missing or invalid event_id",
        )
            .into_response();
    }
    let Some(row) = derive_from_sentry_event(&event_json) else {
        return (StatusCode::BAD_REQUEST, "malformed event payload").into_response();
    };
    let durable = match serde_json::to_vec(&row) {
        Ok(bytes) => Bytes::from(bytes),
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to encode durable accept record",
            )
                .into_response();
        }
    };
    let payload_hash = sha256_hex(&durable);
    if let Some(early) = check_event_ack(state, project_id, &event_id, &payload_hash).await {
        return early;
    }
    if let Err(e) = state
        .ingest
        .spool
        .append_raw(Signal::Sentry, &durable)
        .await
    {
        tracing::warn!(error = %e, "sentry spool write failed");
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            [(header::RETRY_AFTER, "1")],
            "spool unavailable",
        )
            .into_response();
    }
    if state
        .ingest
        .enqueue(Signal::Sentry, IngestItem::Sentry(Box::new(row)), true)
        .await
        .is_err()
    {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            [(header::RETRY_AFTER, "1")],
            "ingest worker unavailable",
        )
            .into_response();
    }
    ok_event_id(&event_id)
}

async fn check_event_ack(
    state: &SentryHttpState,
    project_id: &str,
    event_id: &str,
    payload_hash: &str,
) -> Option<Response> {
    let metadata = state.metadata.as_ref()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    match metadata
        .accept_sentry_event_ack(project_id, event_id, payload_hash, now)
        .await
    {
        Ok(SentryAck::Duplicate) => Some(ok_event_id(event_id)),
        Err(SentryAckError::Collision { .. }) => {
            Some((StatusCode::CONFLICT, "event_id payload collision").into_response())
        }
        Err(SentryAckError::Internal(error)) => {
            tracing::warn!(error = %error, "sentry event ack write failed");
            Some(
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    [(header::RETRY_AFTER, "1")],
                    "metadata unavailable",
                )
                    .into_response(),
            )
        }
        Ok(SentryAck::Inserted) => None,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
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

/// Pull `sentry_key` from `X-Sentry-Auth` or `Authorization: Sentry …`.
fn extract_sentry_key(headers: &HeaderMap) -> Option<String> {
    if let Some(value) = headers
        .get("x-sentry-auth")
        .and_then(|v| v.to_str().ok())
        .and_then(parse_sentry_auth)
    {
        return Some(value);
    }
    if let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        let rest = value
            .strip_prefix("Sentry ")
            .or_else(|| value.strip_prefix("sentry "))?;
        return parse_sentry_auth(rest);
    }
    // Query-string style is rare on envelope POST; not accepted here.
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

fn is_32_hex(value: &str) -> bool {
    value.len() == 32 && value.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn parses_x_sentry_auth_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-sentry-auth",
            HeaderValue::from_static(
                "Sentry sentry_version=7, sentry_client=sentry.rust/0.48.5, sentry_key=abc123",
            ),
        );
        assert_eq!(extract_sentry_key(&headers).as_deref(), Some("abc123"));
    }

    #[test]
    fn rejects_missing_key() {
        let headers = HeaderMap::new();
        assert!(extract_sentry_key(&headers).is_none());
    }
}
