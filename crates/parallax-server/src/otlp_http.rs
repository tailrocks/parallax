//! OTLP/HTTP receivers: `/v1/traces`, `/v1/logs`, `/v1/metrics`
//! (binary protobuf bodies, per the OTLP/HTTP spec). Spool, then queue for
//! the ingest worker, then acknowledge.

use crate::serve::IngestState;
use crate::worker::IngestItem;
use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::post;
use parallax_proto::collector_logs::ExportLogsServiceRequest;
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_storage::spool::Signal;
use prost::Message;

pub fn router(state: IngestState) -> Router {
    Router::new()
        .route("/v1/traces", post(traces))
        .route("/v1/logs", post(logs))
        .route("/v1/metrics", post(metrics))
        .with_state(state)
}

async fn ingest<R>(
    state: &IngestState,
    signal: Signal,
    body: Bytes,
    to_item: impl FnOnce(R) -> IngestItem,
) -> axum::response::Response
where
    R: Message + Default + serde::Serialize,
{
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
    if let Err(e) = state.spool.append(signal, &request).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("spool write failed: {e}"),
        )
            .into_response();
    }
    if state.sender.send(to_item(request)).await.is_err() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "ingest worker unavailable".to_string(),
        )
            .into_response();
    }
    // OTLP/HTTP success: 200 with an empty protobuf response message.
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/x-protobuf")],
        Vec::<u8>::new(),
    )
        .into_response()
}

async fn traces(State(state): State<IngestState>, body: Bytes) -> impl IntoResponse {
    ingest::<ExportTraceServiceRequest>(&state, Signal::Traces, body, IngestItem::Traces).await
}

async fn logs(State(state): State<IngestState>, body: Bytes) -> impl IntoResponse {
    ingest::<ExportLogsServiceRequest>(&state, Signal::Logs, body, IngestItem::Logs).await
}

async fn metrics(State(state): State<IngestState>, body: Bytes) -> impl IntoResponse {
    ingest::<ExportMetricsServiceRequest>(&state, Signal::Metrics, body, IngestItem::Metrics).await
}
