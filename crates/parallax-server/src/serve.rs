//! Server assembly: API listener (:4000 — GraphQL + health + OTLP/HTTP
//! routes), the dedicated OTLP/HTTP listener (:4318), the OTLP/gRPC listener
//! (:4317), and the ingest worker connecting receivers to storage.

use crate::config::Config;
use crate::otlp_grpc::OtlpGrpc;
use crate::otlp_http;
use crate::worker::{self, IngestSender, Worker};
use async_graphql_axum::GraphQL;
use axum::Router;
use axum::routing::{get, post_service};
use parallax_storage::adapter::TelemetryStore;
use parallax_storage::memory::MemoryStore;
use parallax_storage::metadata::MetadataStore;
use parallax_storage::spool::Spool;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub struct ServerHandle {
    pub api_addr: SocketAddr,
    pub otlp_grpc_addr: SocketAddr,
    pub otlp_http_addr: SocketAddr,
    pub spool: Arc<Spool>,
    pub store: Arc<dyn TelemetryStore>,
    pub metadata: Arc<MetadataStore>,
    tasks: Vec<JoinHandle<()>>,
}

impl ServerHandle {
    /// Abort all listener tasks (test teardown / shutdown path).
    pub fn shutdown(&self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}

/// Shared state handed to both OTLP transports.
#[derive(Clone)]
pub struct IngestState {
    pub spool: Arc<Spool>,
    pub sender: IngestSender,
}

/// Start every listener plus the ingest worker. Port 0 means "pick a free
/// port" (tests); bound addresses are reported in the handle.
///
/// M1 storage: in-memory telemetry store + Turso metadata. The GreptimeDB
/// adapter replaces the memory store behind the same trait in the next slice.
pub async fn start(config: &Config) -> anyhow::Result<ServerHandle> {
    let data_dir = config.data_dir();
    std::fs::create_dir_all(&data_dir)?;
    let spool = Arc::new(Spool::open(data_dir.join("spool"))?);
    let store: Arc<dyn TelemetryStore> = Arc::new(MemoryStore::new());
    let metadata = Arc::new(MetadataStore::open(data_dir.join("meta.db")).await?);

    let (sender, receiver) = worker::channel(1024);
    let ingest = IngestState {
        spool: spool.clone(),
        sender,
    };
    let worker = Worker::new(store.clone(), metadata.clone());
    let mut tasks = Vec::new();
    tasks.push(tokio::spawn(worker.run(receiver)));

    let schema = parallax_api::build_schema();
    let api_router = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/version", get(|| async { env!("CARGO_PKG_VERSION") }))
        .route_service("/graphql", post_service(GraphQL::new(schema)))
        .merge(otlp_http::router(ingest.clone()));

    let bind = &config.server.bind;
    let api_listener = TcpListener::bind((bind.as_str(), config.server.api_port)).await?;
    let api_addr = api_listener.local_addr()?;

    let otlp_http_listener =
        TcpListener::bind((bind.as_str(), config.server.otlp_http_port)).await?;
    let otlp_http_addr = otlp_http_listener.local_addr()?;
    let otlp_http_router = otlp_http::router(ingest.clone());

    let otlp_grpc_listener =
        TcpListener::bind((bind.as_str(), config.server.otlp_grpc_port)).await?;
    let otlp_grpc_addr = otlp_grpc_listener.local_addr()?;
    let grpc = OtlpGrpc::new(ingest);
    let grpc_server = tonic::transport::Server::builder()
        .add_service(grpc.trace_service())
        .add_service(grpc.logs_service())
        .add_service(grpc.metrics_service());

    tasks.push(tokio::spawn(async move {
        if let Err(e) = axum::serve(api_listener, api_router).await {
            tracing::error!("api listener failed: {e}");
        }
    }));
    tasks.push(tokio::spawn(async move {
        if let Err(e) = axum::serve(otlp_http_listener, otlp_http_router).await {
            tracing::error!("otlp/http listener failed: {e}");
        }
    }));
    tasks.push(tokio::spawn(async move {
        if let Err(e) = grpc_server
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(
                otlp_grpc_listener,
            ))
            .await
        {
            tracing::error!("otlp/grpc listener failed: {e}");
        }
    }));

    tracing::info!(%api_addr, %otlp_grpc_addr, %otlp_http_addr, "parallax listening");
    Ok(ServerHandle {
        api_addr,
        otlp_grpc_addr,
        otlp_http_addr,
        spool,
        store,
        metadata,
        tasks,
    })
}
