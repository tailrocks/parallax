//! OTLP/HTTP receivers: `/v1/traces`, `/v1/logs`, `/v1/metrics`
//! (binary protobuf bodies, per the OTLP/HTTP spec).

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::post;
use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_storage::spool::{Signal, Spool};
use prost::Message;
use std::sync::Arc;

pub fn router(spool: Arc<Spool>) -> Router {
    Router::new()
        .route("/v1/traces", post(traces))
        .route("/v1/logs", post(logs))
        .route("/v1/metrics", post(metrics))
        .with_state(spool)
}

async fn ingest<R: Message + Default + serde::Serialize>(
    spool: &Spool,
    signal: Signal,
    body: Bytes,
) -> impl IntoResponse + use<R> {
    let request = match R::decode(body) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("invalid OTLP protobuf body: {e}"),
            )
                .into_response();
        }
    };
    match spool.append(signal, &request).await {
        // OTLP/HTTP success: 200 with an empty protobuf response message.
        Ok(()) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/x-protobuf")],
            Vec::<u8>::new(),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("spool write failed: {e}"),
        )
            .into_response(),
    }
}

async fn traces(State(spool): State<Arc<Spool>>, body: Bytes) -> impl IntoResponse {
    ingest::<ExportTraceServiceRequest>(&spool, Signal::Traces, body).await
}

async fn logs(State(spool): State<Arc<Spool>>, body: Bytes) -> impl IntoResponse {
    ingest::<ExportLogsServiceRequest>(&spool, Signal::Logs, body).await
}

async fn metrics(State(spool): State<Arc<Spool>>, body: Bytes) -> impl IntoResponse {
    ingest::<ExportMetricsServiceRequest>(&spool, Signal::Metrics, body).await
}
