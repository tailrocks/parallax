//! OTLP/gRPC receivers (:4317): trace, logs, and metrics collector services.
//! Each accepted request is spooled (durability) then queued for the ingest
//! worker (processing) before acknowledgement.

use crate::ingest_runtime::IngestState;
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
use parallax_spool::Signal;
use prost::Message;
use tonic::codec::CompressionEncoding;
use tonic::{Request, Response, Status};

#[derive(Clone, Debug)]
pub(crate) struct OtlpGrpc {
    state: IngestState,
    max_decoding_message_size: usize,
}

impl OtlpGrpc {
    #[must_use]
    pub(crate) fn new(state: IngestState, max_decoding_message_size: usize) -> Self {
        Self {
            state,
            max_decoding_message_size,
        }
    }

    #[must_use]
    pub(crate) fn trace_service(&self) -> TraceServiceServer<Self> {
        TraceServiceServer::new(self.clone())
            .accept_compressed(CompressionEncoding::Gzip)
            .send_compressed(CompressionEncoding::Gzip)
            .max_decoding_message_size(self.max_decoding_message_size)
    }

    #[must_use]
    pub(crate) fn logs_service(&self) -> LogsServiceServer<Self> {
        LogsServiceServer::new(self.clone())
            .accept_compressed(CompressionEncoding::Gzip)
            .send_compressed(CompressionEncoding::Gzip)
            .max_decoding_message_size(self.max_decoding_message_size)
    }

    #[must_use]
    pub(crate) fn metrics_service(&self) -> MetricsServiceServer<Self> {
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
        observed: bool,
    ) -> Result<(), Status> {
        let raw = bytes::Bytes::from(request.encode_to_vec());
        self.state
            .spool
            .append_raw(signal, &raw)
            .await
            .map_err(|e| Status::internal(format!("spool write failed: {e}")))?;
        self.state
            .enqueue(signal, to_item(request, raw), observed)
            .await
            .map_err(|()| Status::internal("ingest worker unavailable"))
    }
}

#[tonic::async_trait]
impl TraceService for OtlpGrpc {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        let request = request.into_inner();
        crate::otlp_validation::trace_ids(&request).map_err(Status::invalid_argument)?;
        self.spool_then_queue(Signal::Traces, request, IngestItem::Traces, true)
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
        crate::otlp_validation::log_trace_ids(&request).map_err(Status::invalid_argument)?;
        self.spool_then_queue(Signal::Logs, request, IngestItem::Logs, true)
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
        crate::otlp_validation::metric_trace_ids(&request).map_err(Status::invalid_argument)?;
        let observed = !crate::ingest_health::is_self_metrics(&request);
        self.spool_then_queue(Signal::Metrics, request, IngestItem::Metrics, observed)
            .await?;
        Ok(Response::new(ExportMetricsServiceResponse {
            partial_success: None,
        }))
    }
}
