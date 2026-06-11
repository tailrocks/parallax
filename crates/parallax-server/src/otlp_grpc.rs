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
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct OtlpGrpc {
    state: IngestState,
}

impl OtlpGrpc {
    pub fn new(state: IngestState) -> Self {
        Self { state }
    }

    pub fn trace_service(&self) -> TraceServiceServer<Self> {
        TraceServiceServer::new(self.clone())
    }

    pub fn logs_service(&self) -> LogsServiceServer<Self> {
        LogsServiceServer::new(self.clone())
    }

    pub fn metrics_service(&self) -> MetricsServiceServer<Self> {
        MetricsServiceServer::new(self.clone())
    }

    async fn accept<T: serde::Serialize>(
        &self,
        signal: Signal,
        request: &T,
        item: IngestItem,
    ) -> Result<(), Status> {
        self.state
            .spool
            .append(signal, request)
            .await
            .map_err(|e| Status::internal(format!("spool write failed: {e}")))?;
        self.state
            .sender
            .send(item)
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
        self.accept(
            Signal::Traces,
            &request,
            IngestItem::Traces(request.clone()),
        )
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
        self.accept(Signal::Logs, &request, IngestItem::Logs(request.clone()))
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
        self.accept(
            Signal::Metrics,
            &request,
            IngestItem::Metrics(request.clone()),
        )
        .await?;
        Ok(Response::new(ExportMetricsServiceResponse {
            partial_success: None,
        }))
    }
}
