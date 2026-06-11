//! OTLP/gRPC receivers (:4317): trace, logs, and metrics collector services.

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
use parallax_storage::spool::{Signal, Spool};
use std::sync::Arc;
use tonic::{Request, Response, Status};

#[derive(Clone)]
pub struct OtlpGrpc {
    spool: Arc<Spool>,
}

impl OtlpGrpc {
    pub fn new(spool: Arc<Spool>) -> Self {
        Self { spool }
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

    async fn spool_or_status<T: serde::Serialize>(
        &self,
        signal: Signal,
        request: &T,
    ) -> Result<(), Status> {
        self.spool
            .append(signal, request)
            .await
            .map_err(|e| Status::internal(format!("spool write failed: {e}")))
    }
}

#[tonic::async_trait]
impl TraceService for OtlpGrpc {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        self.spool_or_status(Signal::Traces, request.get_ref())
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
        self.spool_or_status(Signal::Logs, request.get_ref())
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
        self.spool_or_status(Signal::Metrics, request.get_ref())
            .await?;
        Ok(Response::new(ExportMetricsServiceResponse {
            partial_success: None,
        }))
    }
}
