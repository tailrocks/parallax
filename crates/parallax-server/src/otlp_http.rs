//! OTLP/HTTP receivers: `/v1/traces`, `/v1/logs`, `/v1/metrics`
//! (binary protobuf bodies, per the OTLP/HTTP spec). Spool, then queue for
//! the ingest worker, then acknowledge.
//!
//! Accepts `Content-Encoding: gzip` (OTLP/HTTP interop) via tower-http's
//! request decompression layer so spool + forward always see decompressed
//! protobuf bytes.

use crate::ingest_runtime::IngestState;
use crate::worker::IngestItem;
use axum::Router;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_spool::Signal;
use prost::Message;
use tower_http::decompression::RequestDecompressionLayer;

/// Build the OTLP/HTTP router with gzip request decompression and an explicit
/// body-size limit (`[limits] otlp_max_body_bytes`).
pub(crate) fn router(state: IngestState, max_body_bytes: usize) -> Router {
    Router::new()
        .route("/v1/traces", post(traces))
        .route("/v1/logs", post(logs))
        .route("/v1/metrics", post(metrics))
        .layer(middleware::from_fn(move |req, next| {
            body_limit_warn(max_body_bytes, req, next)
        }))
        .layer(DefaultBodyLimit::max(max_body_bytes))
        // Decompress after the body is collected so handlers see plaintext
        // protobuf; spool and Greptime forward must never see gzip frames.
        .layer(RequestDecompressionLayer::new())
        .with_state(state)
}

/// Log when Content-Length already exceeds the configured limit (progress
/// visibility for operator-facing rejections). Axum still enforces the limit.
async fn body_limit_warn(max_body_bytes: usize, request: Request, next: Next) -> Response {
    if let Some(len) = request
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
        && len > max_body_bytes
    {
        tracing::warn!(
            payload_size = len,
            limit = max_body_bytes,
            "OTLP/HTTP request rejected: body exceeds otlp_max_body_bytes"
        );
    }
    next.run(request).await
}

async fn ingest<R>(
    state: &IngestState,
    signal: Signal,
    body: Bytes,
    to_item: impl FnOnce(R, Bytes) -> IngestItem,
    validate: impl FnOnce(&R) -> Result<(), &'static str>,
    observe: impl FnOnce(&R) -> bool,
) -> Response
where
    R: Message + Default,
{
    // Body is decompressed protobuf (RequestDecompressionLayer). Forward
    // verbatim (zero-copy Bytes clone) while decoding for the in-process tee.
    let raw = body.clone();
    let request = match R::decode(body) {
        Ok(r) => r,
        Err(e) => {
            state.health.ingress_reject(signal);
            return (
                StatusCode::BAD_REQUEST,
                format!("invalid OTLP protobuf body: {e}"),
            )
                .into_response();
        }
    };
    if let Err(error) = validate(&request) {
        state.health.ingress_reject(signal);
        return (StatusCode::BAD_REQUEST, error.to_string()).into_response();
    }
    if let Err(e) = state.spool.append_raw(signal, &raw).await {
        state.health.spool_failed(signal);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("spool write failed: {e}"),
        )
            .into_response();
    }
    let observed = observe(&request);
    if state
        .enqueue(signal, to_item(request, raw), observed)
        .await
        .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "ingest worker unavailable".to_string(),
        )
            .into_response();
    }
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/x-protobuf")],
        Vec::<u8>::new(),
    )
        .into_response()
}

async fn traces(State(state): State<IngestState>, body: Bytes) -> impl IntoResponse {
    ingest::<ExportTraceServiceRequest>(
        &state,
        Signal::Traces,
        body,
        IngestItem::Traces,
        crate::otlp_validation::trace_ids,
        |_| true,
    )
    .await
}

async fn logs(State(state): State<IngestState>, body: Bytes) -> impl IntoResponse {
    ingest::<ExportLogsServiceRequest>(
        &state,
        Signal::Logs,
        body,
        IngestItem::Logs,
        crate::otlp_validation::log_trace_ids,
        |_| true,
    )
    .await
}

async fn metrics(State(state): State<IngestState>, body: Bytes) -> impl IntoResponse {
    ingest::<ExportMetricsServiceRequest>(
        &state,
        Signal::Metrics,
        body,
        IngestItem::Metrics,
        crate::otlp_validation::metric_trace_ids,
        |request| !crate::ingest_health::is_self_metrics(request),
    )
    .await
}
