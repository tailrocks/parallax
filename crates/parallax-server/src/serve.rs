//! Server assembly: API listener (:4000 — GraphQL + health + OTLP/HTTP
//! routes), the dedicated OTLP/HTTP listener (:4318), the OTLP/gRPC listener
//! (:4317), and the ingest worker connecting receivers to storage.

use crate::config::{Config, LimitsConfig};
use crate::otlp_grpc::OtlpGrpc;
use crate::otlp_http;
use crate::worker::{self, IngestSenders, Worker};
use axum::extract::{Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use parallax_api::{ApiContext, Schema as ParallaxSchema};
use parallax_spool::{Spool, SpoolRetention};
use parallax_storage::adapter::TelemetryStore;
use parallax_storage::metadata::MetadataStore;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

#[expect(missing_debug_implementations, reason = "opaque runtime handles")]
pub struct ServerHandle {
    pub api_addr: SocketAddr,
    pub otlp_grpc_addr: SocketAddr,
    pub otlp_http_addr: SocketAddr,
    pub spool: Arc<Spool>,
    pub store: Arc<dyn TelemetryStore>,
    pub metadata: Arc<MetadataStore>,
    supervisor: Option<crate::greptime_supervisor::GreptimeSupervisor>,
    /// Listener/reaper tasks (aborted on shutdown).
    tasks: Vec<JoinHandle<()>>,
    /// Ingest worker tasks (one per signal) — drained on graceful shutdown after senders drop.
    worker_tasks: Vec<JoinHandle<()>>,
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
        for worker in &self.worker_tasks {
            worker.abort();
        }
    }

    /// Graceful shutdown: stop listeners first so no new ingest items are
    /// accepted, then wait for the worker to drain the channel (bounded).
    pub async fn shutdown_graceful(mut self) {
        if let Some(supervisor) = &self.supervisor {
            supervisor.stop();
        }
        for task in &self.tasks {
            task.abort();
        }
        let workers = std::mem::take(&mut self.worker_tasks);
        crate::outcomes::drain_workers(workers).await;
    }
}

async fn connect_greptime(url: &str, config: &Config) -> anyhow::Result<Arc<dyn TelemetryStore>> {
    // TTLs ride the `x-greptime-hints` on each native OTLP forward, so the
    // adapter keeps them; bootstrap only creates the extension tables.
    let store = parallax_greptime::GreptimeStore::connect(
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
    store: Arc<dyn TelemetryStore>,
    metadata: Arc<MetadataStore>,
    otlp_grpc_port: u16,
    limits: LimitsConfig,
}

#[derive(Clone)]
struct HostGuard {
    allowed_hosts: Arc<Vec<String>>,
}

impl HostGuard {
    fn for_listener(bind: &str, api_addr: SocketAddr) -> Self {
        let mut allowed = vec![
            "localhost".to_string(),
            "127.0.0.1".to_string(),
            "[::1]".to_string(),
        ];
        add_allowed_host(&mut allowed, bind);
        add_allowed_host(&mut allowed, &api_addr.ip().to_string());
        allowed.sort();
        allowed.dedup();
        Self {
            allowed_hosts: Arc::new(allowed),
        }
    }

    fn allows(&self, host: &str) -> bool {
        normalize_host_header(host)
            .is_some_and(|host| self.allowed_hosts.iter().any(|allowed| allowed == &host))
    }
}

fn add_allowed_host(allowed: &mut Vec<String>, host: &str) {
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
    if host.is_empty() {
        return;
    }
    if host.contains(':') && !host.starts_with('[') {
        allowed.push(format!("[{host}]"));
    } else {
        allowed.push(host);
    }
}

fn normalize_host_header(host: &str) -> Option<String> {
    let host = host.trim().trim_end_matches('.');
    if host.is_empty() {
        return None;
    }
    if host.starts_with('[') {
        let end = host.find(']')?;
        let rest = &host[end + 1..];
        if rest.is_empty()
            || rest
                .strip_prefix(':')
                .is_some_and(|port| !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()))
        {
            return Some(host[..=end].to_ascii_lowercase());
        }
        return None;
    }
    if host.matches(':').count() > 1 {
        return None;
    }
    let bare = match host.rsplit_once(':') {
        Some((name, port))
            if !name.is_empty() && !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) =>
        {
            name
        }
        Some(_) => return None,
        None => host,
    };
    Some(bare.to_ascii_lowercase())
}

async fn host_guard_middleware(
    State(guard): State<HostGuard>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let allowed = request
        .headers()
        .get(header::HOST)
        .and_then(|host| host.to_str().ok())
        .is_some_and(|host| guard.allows(host));
    if allowed {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
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
        // Fresh ApiContext per request so RequestMemo is request-scoped and
        // sibling resolvers share one spans_by_trace / logs_by_trace fetch.
        let context = ApiContext {
            store: state.store.clone(),
            metadata: state.metadata.clone(),
            otlp_grpc_port: state.otlp_grpc_port,
            memo: parallax_api::RequestMemo::default(),
        };
        let response = match parallax_api::check_query_limits(
            &state.schema,
            &request.query,
            request.operation_name.as_deref(),
            state.limits.graphql_max_depth,
            state.limits.graphql_max_complexity,
        ) {
            Ok(()) => request.execute(&state.schema, &context).await,
            Err(message) => juniper::http::GraphQLResponse::error(juniper::FieldError::new(
                message,
                juniper::Value::null(),
            )),
        };
        tracing::info!(ok = response.is_ok(), "graphql request");
        Json(response)
    }
    .instrument(tracing::info_span!("graphql.request", otel.name = %operation))
    .await
}

/// Shared state handed to both OTLP transports.
#[derive(Clone, Debug)]
pub struct IngestState {
    pub spool: Arc<Spool>,
    pub senders: IngestSenders,
}

fn spawn_spool_reaper(
    spool: Arc<Spool>,
    max_total_bytes: u64,
    max_age_hours: u64,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let retention = SpoolRetention {
            max_total_bytes,
            max_age: Duration::from_secs(max_age_hours.saturating_mul(60 * 60)),
        };
        let mut interval = tokio::time::interval(Duration::from_secs(10 * 60));
        loop {
            interval.tick().await;
            match spool.reap(retention, std::time::SystemTime::now()) {
                Ok(reclaimed) if reclaimed.removed_segments > 0 => {
                    tracing::info!(
                        segments = reclaimed.removed_segments,
                        bytes = reclaimed.reclaimed_bytes,
                        "reclaimed ingest spool segments"
                    );
                }
                Ok(_) => {}
                Err(error) => tracing::warn!("ingest spool reaper failed: {error}"),
            }
        }
    })
}

/// Start every listener plus the ingest worker. Port 0 means "pick a free
/// port" (tests); bound addresses are reported in the handle.
///
/// Storage mode (config `[storage] mode`): `managed` supervises a local
/// GreptimeDB standalone child on the shifted ports; `external` uses
/// `greptime_url`. No product fallback store exists.
pub async fn start(config: &Config) -> anyhow::Result<ServerHandle> {
    config.validate()?;
    let data_dir = config.data_dir();
    tokio::fs::create_dir_all(&data_dir).await?;

    let mut supervisor = None;
    let store: Arc<dyn TelemetryStore> = match config.storage.mode.as_str() {
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
        "managed" => {
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
        mode => anyhow::bail!(
            "unsupported storage.mode {mode:?}; supported values are \"managed\" and \"external\""
        ),
    };
    let metadata = Arc::new(MetadataStore::open(data_dir.join("meta.db")).await?);
    start_assembled(config, store, metadata, supervisor).await
}

/// Internal composition seam for integration tests. Product entry points use
/// [`start`] and therefore always construct GreptimeDB + Turso themselves.
#[doc(hidden)]
pub async fn start_with_capabilities(
    config: &Config,
    store: Arc<dyn TelemetryStore>,
    metadata: Arc<MetadataStore>,
) -> anyhow::Result<ServerHandle> {
    start_assembled(config, store, metadata, None).await
}

async fn start_assembled(
    config: &Config,
    store: Arc<dyn TelemetryStore>,
    metadata: Arc<MetadataStore>,
    supervisor: Option<crate::greptime_supervisor::GreptimeSupervisor>,
) -> anyhow::Result<ServerHandle> {
    let data_dir = config.data_dir();
    tokio::fs::create_dir_all(&data_dir).await?;
    let spool =
        crate::outcomes::open_spool(&data_dir, config.retention.spool_max_segment_bytes).await?;

    let queue_batches = config.limits.ingest_queue_batches.max(1);
    let (senders, receivers) = worker::channels(queue_batches);
    let ingest = IngestState {
        spool: spool.clone(),
        senders,
    };
    let live = crate::live::channels();
    let worker = Worker::new(store.clone(), metadata.clone(), live.clone());
    let worker_tasks = vec![
        tokio::spawn(worker.clone().run(receivers.traces)),
        tokio::spawn(worker.clone().run(receivers.logs)),
        tokio::spawn(worker.run(receivers.metrics)),
    ];
    tracing::info!(
        queue_batches_per_signal = queue_batches,
        "ingest workers ready (traces/logs/metrics)"
    );
    let mut tasks = Vec::new();
    tasks.push(spawn_spool_reaper(
        spool.clone(),
        config.retention.spool_max_total_bytes,
        config.retention.spool_max_age_hours,
    ));

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
        store: store.clone(),
        metadata: metadata.clone(),
        otlp_grpc_port: otlp_grpc_addr.port(),
        limits: config.limits.clone(),
    };
    let host_guard_state = HostGuard::for_listener(bind, api_addr);
    let api_router = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/version", get(|| async { env!("CARGO_PKG_VERSION") }))
        .merge(
            Router::new()
                .route("/v1/logs/stream", get(crate::live::stream_logs))
                .route("/v1/traces/stream", get(crate::live::stream_traces))
                .with_state(live)
                .layer(middleware::from_fn_with_state(
                    host_guard_state.clone(),
                    host_guard_middleware,
                )),
        )
        .merge(
            Router::new()
                .route("/graphql", post(graphql_handler))
                .with_state(graphql_state)
                .layer(middleware::from_fn_with_state(
                    host_guard_state.clone(),
                    host_guard_middleware,
                )),
        )
        // OTLP/HTTP stays unguarded here: exporters may legitimately use a
        // container or bridge hostname while the developer API remains local.
        .merge(otlp_http::router(
            ingest.clone(),
            config.limits.otlp_max_body_bytes,
        ));

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

    let otlp_http_router = otlp_http::router(ingest.clone(), config.limits.otlp_max_body_bytes);
    let grpc = OtlpGrpc::new(ingest, config.limits.otlp_max_body_bytes);
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
        worker_tasks,
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
            "Parallax API is running. UI build not found — run `bun run build` in ui/ \
             or set [server].ui_dist in config.toml."
        }))
    }
}
