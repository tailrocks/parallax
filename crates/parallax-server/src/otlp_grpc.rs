//! OTLP/gRPC receivers (:4317): trace, logs, and metrics collector services.
//! Each accepted request is spooled (durability) then queued for the ingest
//! worker (processing) before acknowledgement.

use crate::serve::IngestState;
use crate::worker::IngestItem;
use parallax_proto::collector_logs::logs_service_server::{LogsService, LogsServiceServer};
use parallax_proto::collector_logs::{ExportLogsServiceRequest, ExportLogsServiceResponse};
use parallax_proto::collector_metrics::metrics_service_server::{
    MetricsService, MetricsServiceServer,
};
use parallax_proto::collector_metrics::{
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use parallax_proto::collector_trace::trace_service_server::{TraceService, TraceServiceServer};
use parallax_proto::collector_trace::{ExportTraceServiceRequest, ExportTraceServiceResponse};
use parallax_storage::spool::Signal;
use prost::Message;
use tonic::codec::CompressionEncoding;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct OtlpGrpc {
    state: IngestState,
    max_decoding_message_size: usize,
}

impl OtlpGrpc {
    #[must_use]
    pub fn new(state: IngestState, max_decoding_message_size: usize) -> Self {
        Self {
            state,
            max_decoding_message_size,
        }
    }

    #[must_use]
    pub fn trace_service(&self) -> TraceServiceServer<Self> {
        TraceServiceServer::new(self.clone())
            .accept_compressed(CompressionEncoding::Gzip)
            .send_compressed(CompressionEncoding::Gzip)
            .max_decoding_message_size(self.max_decoding_message_size)
    }

    #[must_use]
    pub fn logs_service(&self) -> LogsServiceServer<Self> {
        LogsServiceServer::new(self.clone())
            .accept_compressed(CompressionEncoding::Gzip)
            .send_compressed(CompressionEncoding::Gzip)
            .max_decoding_message_size(self.max_decoding_message_size)
    }

    #[must_use]
    pub fn metrics_service(&self) -> MetricsServiceServer<Self> {
        MetricsServiceServer::new(self.clone())
            .accept_compressed(CompressionEncoding::Gzip)
            .send_compressed(CompressionEncoding::Gzip)
            .max_decoding_message_size(self.max_decoding_message_size)
    }

    async fn spool_then_queue<T: Message>(
        &self,
        signal: Signal,
        request: T,
        to_item: impl FnOnce(T, bytes::Bytes) -> IngestItem,
    ) -> Result<(), Status> {
        let raw = bytes::Bytes::from(request.encode_to_vec());
        self.state
            .spool
            .append_raw(signal, &raw)
            .await
            .map_err(|e| Status::internal(format!("spool write failed: {e}")))?;
        self.state
            .senders
            .for_signal(signal)
            .send(to_item(request, raw))
            .await
            .map_err(|_| Status::internal("ingest worker unavailable"))
    }
}

#[tonic::async_trait]
impl TraceService for OtlpGrpc {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        let request = request.into_inner();
        self.spool_then_queue(Signal::Traces, request, IngestItem::Traces)
            .await?;
        Ok(Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl LogsService for OtlpGrpc {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        let request = request.into_inner();
        self.spool_then_queue(Signal::Logs, request, IngestItem::Logs)
            .await?;
        Ok(Response::new(ExportLogsServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl MetricsService for OtlpGrpc {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        let request = request.into_inner();
        self.spool_then_queue(Signal::Metrics, request, IngestItem::Metrics)
            .await?;
        Ok(Response::new(ExportMetricsServiceResponse {
            partial_success: None,
        }))
    }
}
