//! Browser product-contract harness (plan 144).
//!
//! Starts Parallax with an injected in-memory telemetry adapter + Turso
//! metadata (test composition seam only — never a product storage mode),
//! serves the built UI, and exposes a private loopback control plane for
//! dataset reset/seed/snapshot. Playwright talks to public GraphQL/HTTP only
//! for product assertions; control is for the fixture harness.

#![expect(clippy::expect_used, reason = "harness exits on setup failure")]
#![expect(
    clippy::excessive_nesting,
    clippy::too_many_lines,
    reason = "self-contained browser-contract fixture server"
)]
#![expect(
    clippy::print_stdout,
    reason = "progress narration for long-running serve"
)]

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use anyhow::{Context, Result, bail};
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::any;
use axum::{Json, Router};
use parallax_metadata::TursoMetadataStore;
use parallax_server::{Config, start_with_capabilities};
use parallax_storage::metadata::MetadataStore;
use parallax_test_support::browser::{DatasetId, investigation_snapshot, reset_and_seed};
use parallax_test_support::builders::MemoryStore;
use serde::Deserialize;
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

#[derive(Clone)]
struct ControlState {
    store: Arc<MemoryStore>,
    metadata: Arc<dyn MetadataStore>,
    dataset: Arc<Mutex<Option<DatasetId>>>,
    fail_next_graphql: Arc<AtomicU32>,
    lock: Arc<Mutex<()>>,
}

#[derive(Debug, Deserialize)]
struct ControlRequest {
    op: String,
    dataset: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let root = workspace_root()?;
    let ui_dist = root.join("ui/dist/client");
    if !ui_dist.join("_shell.html").is_file() {
        bail!(
            "browser contracts require built UI at {} — run `cd ui && bun run build` first",
            ui_dist.display()
        );
    }

    let data_dir = std::env::temp_dir().join(format!(
        "parallax-browser-contracts-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&data_dir).context("create harness data dir")?;

    let mut config = Config::default();
    config.server.bind = "127.0.0.1".into();
    config.server.api_port = 0;
    config.server.otlp_grpc_port = 0;
    config.server.otlp_http_port = 0;
    config.server.ui_dist = ui_dist.display().to_string();
    config.storage.data_dir = data_dir.display().to_string();
    // Product modes remain managed/external only; this harness never sets a
    // memory mode — it injects MemoryStore at the composition seam.
    config.storage.mode = "managed".into();
    config.alerting.enabled = false;

    let store = Arc::new(MemoryStore::new().with_normalizers(
        Arc::new(parallax_ingest::normalize_traces),
        Arc::new(parallax_ingest::normalize_logs),
    ));
    let metadata_concrete = Arc::new(
        TursoMetadataStore::open(data_dir.join("meta.db"))
            .await
            .context("open turso metadata")?,
    );
    let metadata: Arc<dyn MetadataStore> = metadata_concrete.clone();

    let handle = start_with_capabilities(&config, store.clone(), metadata.clone())
        .await
        .context("start server with injected test adapter")?;

    let public_port: u16 = std::env::var("PARALLAX_BROWSER_CONTRACTS_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let public_listener = TcpListener::bind(("127.0.0.1", public_port))
        .await
        .context("bind public proxy")?;
    let public_addr = public_listener.local_addr()?;

    let control_port: u16 = std::env::var("PARALLAX_BROWSER_CONTRACTS_CONTROL_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let control_listener = TcpListener::bind(("127.0.0.1", control_port))
        .await
        .context("bind control plane")?;
    let control_addr = control_listener.local_addr()?;

    let control = ControlState {
        store: store.clone(),
        metadata: metadata.clone(),
        dataset: Arc::new(Mutex::new(None)),
        fail_next_graphql: Arc::new(AtomicU32::new(0)),
        lock: Arc::new(Mutex::new(())),
    };

    // Default seed so shell smoke and readiness work before first fixture reset.
    reset_and_seed(store.as_ref(), metadata.clone(), DatasetId::ShellEmpty)
        .await
        .context("default shell-empty seed")?;
    *control.dataset.lock().await = Some(DatasetId::ShellEmpty);

    let manifest_path = root.join("ui/test-results/browser-contracts-runtime.json");
    if let Some(parent) = manifest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let manifest = json!({
        "schema_version": 1,
        "bind": "127.0.0.1",
        "port": public_addr.port(),
        "base_url": format!("http://127.0.0.1:{}", public_addr.port()),
        "health_url": format!("http://127.0.0.1:{}/health", public_addr.port()),
        "control_url": format!("http://127.0.0.1:{}", control_addr.port()),
        "api_addr": handle.api_addr.to_string(),
        "ui_dist": ui_dist.display().to_string(),
        "data_dir": data_dir.display().to_string(),
        "dataset_id": "shell-empty",
        "pid": std::process::id(),
        "server_pid": std::process::id(),
    });
    std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)
        .with_context(|| format!("write {}", manifest_path.display()))?;

    println!("==> browser contracts server starting");
    println!("    public:  http://127.0.0.1:{}", public_addr.port());
    println!("    control: {}", control_addr);
    println!("    api:     {}", handle.api_addr);
    println!("    ui:      {}", ui_dist.display());
    println!("    manifest: {}", manifest_path.display());
    println!(
        "    health:  http://127.0.0.1:{}/health",
        public_addr.port()
    );
    println!("Parallax browser contracts ready — Ctrl-C / webServer stop to exit");

    let proxy_state = ProxyState {
        upstream: handle.api_addr,
        fail_next_graphql: control.fail_next_graphql.clone(),
    };
    let proxy = Router::new()
        .fallback(any(proxy_handler))
        .with_state(proxy_state);

    let control_task = tokio::spawn(control_loop(control_listener, control));
    let proxy_task = tokio::spawn(async move {
        axum::serve(public_listener, proxy)
            .await
            .expect("proxy serve");
    });

    drop(tokio::signal::ctrl_c().await);
    handle.shutdown();
    control_task.abort();
    proxy_task.abort();
    drop(std::fs::remove_dir_all(&data_dir));
    Ok(())
}

#[derive(Clone)]
struct ProxyState {
    upstream: SocketAddr,
    fail_next_graphql: Arc<AtomicU32>,
}

async fn proxy_handler(State(state): State<ProxyState>, request: Request<Body>) -> Response<Body> {
    let path = request.uri().path().to_string();
    if path == "/graphql"
        && state.fail_next_graphql.load(Ordering::SeqCst) > 0
        && state.fail_next_graphql.fetch_sub(1, Ordering::SeqCst) > 0
    {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "errors": [{ "message": "fixture-controlled API failure" }]
            })),
        )
            .into_response();
    }

    let client = reqwest::Client::new();
    let method = request.method().clone();
    let path_and_query = request
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");
    let url = format!("http://{}{path_and_query}", state.upstream);
    let headers = request.headers().clone();
    let body = axum::body::to_bytes(request.into_body(), 16 * 1024 * 1024)
        .await
        .unwrap_or_default();

    let mut builder = client.request(method, url);
    for (name, value) in headers.iter() {
        if name == axum::http::header::HOST {
            continue;
        }
        builder = builder.header(name, value);
    }
    match builder.body(body).send().await {
        Ok(upstream) => {
            let status =
                StatusCode::from_u16(upstream.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
            let mut response = Response::builder().status(status);
            if let Some(headers_mut) = response.headers_mut() {
                for (name, value) in upstream.headers().iter() {
                    if name == reqwest::header::TRANSFER_ENCODING {
                        continue;
                    }
                    if let (Ok(name), Ok(value)) = (
                        axum::http::HeaderName::from_bytes(name.as_str().as_bytes()),
                        axum::http::HeaderValue::from_bytes(value.as_bytes()),
                    ) {
                        headers_mut.append(name, value);
                    }
                }
            }
            let bytes = upstream.bytes().await.unwrap_or_default();
            response.body(Body::from(bytes)).unwrap_or_else(|_| {
                (StatusCode::BAD_GATEWAY, "upstream body error").into_response()
            })
        }
        Err(error) => (StatusCode::BAD_GATEWAY, format!("upstream: {error}")).into_response(),
    }
}

async fn control_loop(listener: TcpListener, state: ControlState) {
    loop {
        let Ok((stream, _)) = listener.accept().await else {
            continue;
        };
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(error) = handle_control(stream, state).await {
                eprintln!("browser contracts control error: {error:#}");
            }
        });
    }
}

async fn handle_control(stream: TcpStream, state: ControlState) -> Result<()> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let request: ControlRequest = serde_json::from_str(line.trim())
        .with_context(|| format!("parse control request: {line}"))?;
    let stream = reader.into_inner();
    let mut stream = stream;

    let response = match request.op.as_str() {
        "reset" => {
            let dataset_raw = request
                .dataset
                .as_deref()
                .context("reset requires dataset")?;
            let dataset = DatasetId::parse(dataset_raw)
                .with_context(|| format!("unknown dataset {dataset_raw}"))?;
            let _guard = state.lock.lock().await;
            let manifest = reset_and_seed(state.store.as_ref(), state.metadata.clone(), dataset)
                .await
                .context("reset_and_seed")?;
            *state.dataset.lock().await = Some(dataset);
            json!({
                "ok": true,
                "dataset_id": manifest.dataset_id.as_str(),
                "manifest": manifest,
            })
        }
        "snapshot" => {
            let _guard = state.lock.lock().await;
            let dataset = *state.dataset.lock().await;
            let investigations = investigation_snapshot(state.metadata.as_ref()).await?;
            let counts = state.store.counts();
            json!({
                "ok": true,
                "dataset_id": dataset.map(|d| d.as_str()),
                "investigations": investigations,
                "counts": {
                    "spans": counts.0,
                    "logs": counts.1,
                    "metrics": counts.2,
                    "error_events": counts.3,
                }
            })
        }
        "fail-next-graphql" => {
            state.fail_next_graphql.store(1, Ordering::SeqCst);
            json!({ "ok": true, "fail_next_graphql": 1 })
        }
        "ping" => json!({ "ok": true }),
        other => json!({ "ok": false, "error": format!("unknown op {other}") }),
    };

    let payload = serde_json::to_vec(&response)?;
    stream.write_all(&payload).await?;
    stream.write_all(b"\n").await?;
    drop(stream.shutdown().await);
    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest
        .parent()
        .and_then(Path::parent)
        .context("workspace root")?
        .to_path_buf())
}
