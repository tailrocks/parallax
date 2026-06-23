//! Server assembly: API listener (:4000 — GraphQL + health + OTLP/HTTP
//! routes), the dedicated OTLP/HTTP listener (:4318), the OTLP/gRPC listener
//! (:4317), and the ingest worker connecting receivers to storage.

use crate::config::Config;
use crate::otlp_grpc::OtlpGrpc;
use crate::otlp_http;
use crate::worker::{self, IngestSender, Worker};
use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::routing::{get, post};
use parallax_api::{ApiContext, Schema as ParallaxSchema};
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
    supervisor: Option<crate::greptime_supervisor::GreptimeSupervisor>,
    tasks: Vec<JoinHandle<()>>,
}

impl ServerHandle {
    /// Abort all listener tasks (test teardown / shutdown path). The managed
    /// engine child dies with its kill-on-drop handle.
    pub fn shutdown(&self) {
        if let Some(supervisor) = &self.supervisor {
            supervisor.stop();
        }
        for task in &self.tasks {
            task.abort();
        }
    }
}

async fn connect_greptime(url: &str, config: &Config) -> anyhow::Result<Arc<dyn TelemetryStore>> {
    // TTLs ride the `x-greptime-hints` on each native OTLP forward, so the
    // adapter keeps them; bootstrap only creates the extension tables.
    let store = parallax_storage::greptime::GreptimeStore::connect(
        url,
        &config.retention.traces_ttl,
        &config.retention.logs_ttl,
        &config.retention.metrics_ttl,
    )
    .await?;
    store
        .bootstrap(
            &config.retention.metrics_ttl,
            &config.retention.error_events_ttl,
        )
        .await?;
    Ok(Arc::new(store))
}

#[derive(Clone)]
struct GraphQlState {
    schema: Arc<ParallaxSchema>,
    context: Arc<ApiContext>,
}

/// The hand-rolled Juniper-over-axum handler (spec §2 note). Wrapped in a
/// `graphql.request` span so self-telemetry (when enabled) emits Parallax's own
/// API activity — this is the recurring signal that fans out to the lab.
async fn graphql_handler(
    State(state): State<GraphQlState>,
    Json(request): Json<juniper::http::GraphQLRequest>,
) -> Json<juniper::http::GraphQLResponse> {
    use tracing::Instrument;
    let operation = request
        .operation_name
        .clone()
        .unwrap_or_else(|| "anonymous".to_string());
    async move {
        let response = request.execute(&state.schema, &state.context).await;
        tracing::info!(ok = response.is_ok(), "graphql request");
        Json(response)
    }
    .instrument(tracing::info_span!("graphql.request", otel.name = %operation))
    .await
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
/// Storage mode (config `[storage] mode`): `managed` supervises a local
/// GreptimeDB standalone child on the shifted ports; `external` uses
/// `greptime_url`; `none` keeps telemetry in the bounded in-memory store.
pub async fn start(config: &Config) -> anyhow::Result<ServerHandle> {
    let data_dir = config.data_dir();
    std::fs::create_dir_all(&data_dir)?;
    let spool = Arc::new(Spool::open(data_dir.join("spool"))?);

    let mut supervisor = None;
    let store: Arc<dyn TelemetryStore> = match config.storage.mode.as_str() {
        "none" => Arc::new(MemoryStore::new()),
        "external" => {
            let url = &config.storage.greptime_url;
            anyhow::ensure!(
                !url.is_empty(),
                "storage.mode=external requires greptime_url"
            );
            tracing::info!("connecting to external GreptimeDB at {url}");
            let store = connect_greptime(url, config).await?;
            tracing::info!("storage ready (external engine)");
            store
        }
        _ => {
            let binary = crate::greptime_supervisor::ensure_binary(
                &data_dir.join("bin"),
                &config.storage.greptime_version,
                true,
            )
            .await?;
            let started =
                crate::greptime_supervisor::GreptimeSupervisor::start(binary, &data_dir).await?;
            let url = started.http_url.clone();
            supervisor = Some(started);
            tracing::info!("bootstrapping telemetry tables");
            let store = connect_greptime(&url, config).await?;
            tracing::info!("storage ready (managed engine)");
            store
        }
    };
    let metadata = Arc::new(MetadataStore::open(data_dir.join("meta.db")).await?);

    let (sender, receiver) = worker::channel(1024);
    let ingest = IngestState {
        spool: spool.clone(),
        sender,
    };
    let live = crate::live::channels();
    let worker = Worker::new(store.clone(), metadata.clone(), live.clone());
    let mut tasks = Vec::new();
    tasks.push(tokio::spawn(worker.run(receiver)));

    let bind = &config.server.bind;
    let api_listener = TcpListener::bind((bind.as_str(), config.server.api_port)).await?;
    let api_addr = api_listener.local_addr()?;

    let otlp_http_listener =
        TcpListener::bind((bind.as_str(), config.server.otlp_http_port)).await?;
    let otlp_http_addr = otlp_http_listener.local_addr()?;

    let otlp_grpc_listener =
        TcpListener::bind((bind.as_str(), config.server.otlp_grpc_port)).await?;
    let otlp_grpc_addr = otlp_grpc_listener.local_addr()?;

    let graphql_state = GraphQlState {
        schema: Arc::new(parallax_api::build_schema()),
        context: Arc::new(ApiContext {
            store: store.clone(),
            metadata: metadata.clone(),
            otlp_grpc_port: otlp_grpc_addr.port(),
        }),
    };
    let api_router = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/version", get(|| async { env!("CARGO_PKG_VERSION") }))
        .merge(
            Router::new()
                .route("/v1/logs/stream", get(crate::live::stream_logs))
                .route("/v1/traces/stream", get(crate::live::stream_traces))
                .with_state(live),
        )
        .merge(
            Router::new()
                .route("/graphql", post(graphql_handler))
                .with_state(graphql_state),
        )
        .merge(otlp_http::router(ingest.clone()));

    // The UI, by preference: an on-disk SPA build (assets + _shell.html
    // fallback), then assets embedded at compile time (release builds with
    // the `embed-ui` feature), then API-only with a hint.
    let ui_dist = if config.server.ui_dist.is_empty() {
        ["ui/dist/client", "../ui/dist/client"]
            .iter()
            .map(std::path::PathBuf::from)
            .find(|p| p.join("_shell.html").exists())
    } else {
        Some(std::path::PathBuf::from(&config.server.ui_dist))
    };
    let api_router = match ui_dist {
        Some(dist) if dist.join("_shell.html").exists() => {
            tracing::info!("serving UI from {}", dist.display());
            let shell = tower_http::services::ServeFile::new(dist.join("_shell.html"));
            let files = tower_http::services::ServeDir::new(&dist).fallback(shell);
            api_router.fallback_service(files)
        }
        _ => embedded_ui::fallback(api_router),
    };

    let otlp_http_router = otlp_http::router(ingest.clone());
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
        supervisor,
        tasks,
    })
}

/// The compile-time-embedded UI (release builds with `embed-ui`). Without
/// the feature this degrades to the API-only hint.
mod embedded_ui {
    use axum::Router;

    #[cfg(feature = "embed-ui")]
    #[derive(rust_embed::RustEmbed)]
    #[folder = "../../ui/dist/client"]
    struct Assets;

    #[cfg(feature = "embed-ui")]
    pub(super) fn fallback(router: Router) -> Router {
        use axum::http::{StatusCode, header};
        use axum::response::IntoResponse;
        tracing::info!("serving UI embedded in the binary");
        router.fallback(axum::routing::get(|uri: axum::http::Uri| async move {
            // Unmatched paths fall through to the SPA shell (client routing).
            let mut path = uri.path().trim_start_matches('/');
            let mut asset = Assets::get(path);
            if asset.is_none() {
                path = "_shell.html";
                asset = Assets::get(path);
            }
            match asset {
                Some(content) => {
                    let mime = mime_guess::from_path(path).first_or_octet_stream();
                    (
                        [(header::CONTENT_TYPE, mime.as_ref().to_string())],
                        content.data.into_owned(),
                    )
                        .into_response()
                }
                None => (StatusCode::NOT_FOUND, "not found").into_response(),
            }
        }))
    }

    #[cfg(not(feature = "embed-ui"))]
    pub(super) fn fallback(router: Router) -> Router {
        router.fallback(axum::routing::get(|| async {
            "Parallax API is running. UI build not found — run `pnpm build` in ui/ \
             or set [server].ui_dist in config.toml."
        }))
    }
}
