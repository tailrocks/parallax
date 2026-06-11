//! Server assembly: API listener (:4000 — GraphQL + health + OTLP/HTTP
//! routes), the dedicated OTLP/HTTP listener (:4318), and the OTLP/gRPC
//! listener (:4317).

use crate::config::Config;
use crate::otlp_grpc::OtlpGrpc;
use crate::otlp_http;
use async_graphql_axum::GraphQL;
use axum::Router;
use axum::routing::{get, post_service};
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

/// Start every listener. Port 0 in the config means "pick a free port"
/// (used by tests); the bound addresses are reported in the handle.
pub async fn start(config: &Config) -> anyhow::Result<ServerHandle> {
    let data_dir = config.data_dir();
    let spool = Arc::new(Spool::open(data_dir.join("spool"))?);

    let schema = parallax_api::build_schema();
    let api_router = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/version", get(|| async { env!("CARGO_PKG_VERSION") }))
        .route_service("/graphql", post_service(GraphQL::new(schema)))
        .merge(otlp_http::router(spool.clone()));

    let bind = &config.server.bind;
    let api_listener = TcpListener::bind((bind.as_str(), config.server.api_port)).await?;
    let api_addr = api_listener.local_addr()?;

    let otlp_http_listener =
        TcpListener::bind((bind.as_str(), config.server.otlp_http_port)).await?;
    let otlp_http_addr = otlp_http_listener.local_addr()?;
    let otlp_http_router = otlp_http::router(spool.clone());

    let otlp_grpc_listener =
        TcpListener::bind((bind.as_str(), config.server.otlp_grpc_port)).await?;
    let otlp_grpc_addr = otlp_grpc_listener.local_addr()?;
    let grpc = OtlpGrpc::new(spool.clone());
    let grpc_server = tonic::transport::Server::builder()
        .add_service(grpc.trace_service())
        .add_service(grpc.logs_service())
        .add_service(grpc.metrics_service());

    let mut tasks = Vec::new();
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
        tasks,
    })
}
