//! Browser full-stack harness (plan 145).
//!
//! Modes:
//! - `managed` — unique temp data dir + managed GreptimeDB (ports 24000–24003 must
//!   be free) + Turso metadata, then seed via public OTLP/HTTP.
//! - `attach` — reuse an already-running product stack (operator QA or CI service).
//!   Never kills foreign PIDs; only writes a runtime manifest after seed/readiness.
//!
//! Auto-select: when ports 24000–24003 are occupied and no mode is forced, attach
//! to `PARALLAX_FULL_STACK_BASE_URL` (default `http://127.0.0.1:4000`) so a live
//! QA stack can be reused without teardown.
//!
//! A private loopback control plane supports live follow-up OTLP seeds and issue
//! snapshots. Product assertions still use public GraphQL/UI only.

#![expect(clippy::expect_used, reason = "harness exits on setup failure")]
#![expect(
    clippy::print_stdout,
    reason = "progress narration for long-running serve"
)]
#![expect(clippy::too_many_lines, reason = "self-contained full-stack harness")]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, Response, StatusCode};
use axum::response::IntoResponse;
use axum::routing::any;
use parallax_server::{Config, ServerHandle};
use parallax_test_support::browser::{
    RealStackIds, live_followup_log, live_followup_logs, live_followup_span, live_followup_spans,
    logs_request, metrics_request, traces_request,
};
use prost::Message as _;
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tower_http::services::{ServeDir, ServeFile};

const GREPTIME_HTTP_PORT: u16 = 24000;
const GREPTIME_PORTS: [u16; 4] = [24000, 24001, 24002, 24003];

#[derive(Clone)]
struct ControlState {
    base_url: String,
    otlp_http: String,
    ids: RealStackIds,
    issue_fingerprint: Arc<Mutex<Option<String>>>,
}

#[derive(Debug, Deserialize)]
struct ControlRequest {
    op: String,
    body: Option<String>,
    /// Optional burst size for `seed-live-log-burst` (default 5, max 50).
    count: Option<u32>,
    /// Optional fixed timestamp for identity/duplicate cases.
    ts_nanos: Option<String>,
    /// Optional span name for `seed-live-span`.
    span_name: Option<String>,
    /// Optional span id hex for `seed-live-span` identity cases.
    span_id: Option<String>,
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
    let mode = resolve_mode().await?;
    println!("==> browser full-stack mode: {mode}");

    let suffix = std::env::var("PARALLAX_FULL_STACK_DATASET_SUFFIX").unwrap_or_else(|_| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
            .to_string()
    });
    let start_nanos = u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
    )
    .unwrap_or(u64::MAX / 2);
    let ids = RealStackIds::new(&suffix, start_nanos);

    let ui_dist = root.join("ui/dist/client");
    if !ui_dist.join("_shell.html").is_file() {
        bail!(
            "full-stack requires built UI at {} — run `cd ui && bun run build`",
            ui_dist.display()
        );
    }

    let (api_base, otlp_http, owns, data_dir, server, public_base) = match mode.as_str() {
        "attach" => {
            let api = std::env::var("PARALLAX_FULL_STACK_BASE_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:4000".into());
            let otlp = std::env::var("PARALLAX_FULL_STACK_OTLP_HTTP")
                .unwrap_or_else(|_| "http://127.0.0.1:4318".into());
            wait_http_ok(&format!("{api}/health"), Duration::from_secs(30)).await?;
            // Local UI+API proxy — attach backend often has no ui_dist configured.
            let public_port: u16 = std::env::var("PARALLAX_BROWSER_FULL_STACK_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(4175);
            let public = start_ui_proxy(&ui_dist, &api, public_port).await?;
            (api, otlp, false, None, None::<ServerHandle>, public)
        }
        "managed" => {
            ensure_ports_free().await?;
            let data_dir = std::env::temp_dir().join(format!(
                "parallax-browser-full-stack-{}-{start_nanos}",
                std::process::id()
            ));
            std::fs::create_dir_all(&data_dir).context("create full-stack data dir")?;
            seed_engine_binary(&data_dir)?;

            let mut config = Config::default();
            config.server.bind = "127.0.0.1".into();
            config.server.api_port = std::env::var("PARALLAX_BROWSER_FULL_STACK_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            config.server.otlp_grpc_port = 0;
            config.server.otlp_http_port = 0;
            config.server.ui_dist = ui_dist.display().to_string();
            config.storage.mode = "managed".into();
            config.storage.data_dir = data_dir.display().to_string();
            config.alerting.enabled = false;

            println!(
                "==> starting managed Parallax (Greptime {GREPTIME_HTTP_PORT}–24003, data {})",
                data_dir.display()
            );
            let handle = parallax_server::start(&config)
                .await
                .context("start managed full-stack server")?;
            let base = format!("http://{}", handle.api_addr);
            let otlp = format!("http://{}", handle.otlp_http_addr);
            wait_http_ok(&format!("{base}/health"), Duration::from_secs(120)).await?;
            (base.clone(), otlp, true, Some(data_dir), Some(handle), base)
        }
        other => bail!("unknown PARALLAX_FULL_STACK_MODE `{other}` (attach|managed)"),
    };

    println!("==> seeding public OTLP dataset {}", ids.dataset_id);
    seed_otlp(&otlp_http, &ids).await?;
    println!("==> waiting for Greptime + Turso visibility");
    let issue = wait_visibility(&api_base, &ids, Duration::from_secs(90)).await?;
    let fingerprint = issue
        .get("fingerprint")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let control_port: u16 = std::env::var("PARALLAX_BROWSER_FULL_STACK_CONTROL_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let control_listener = TcpListener::bind(("127.0.0.1", control_port))
        .await
        .context("bind full-stack control plane")?;
    let control_addr = control_listener.local_addr()?;

    // Post-seed readiness for Playwright webServer (distinct from product /health).
    let ready_port: u16 = std::env::var("PARALLAX_BROWSER_FULL_STACK_READY_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(4176);
    let ready_listener = TcpListener::bind(("127.0.0.1", ready_port))
        .await
        .with_context(|| format!("bind full-stack ready port {ready_port}"))?;
    let ready_addr = ready_listener.local_addr()?;

    let control = ControlState {
        base_url: api_base.clone(),
        otlp_http: otlp_http.clone(),
        ids: ids.clone(),
        issue_fingerprint: Arc::new(Mutex::new(Some(fingerprint.clone()))),
    };

    let manifest_path = root.join("ui/test-results/browser-full-stack-runtime.json");
    if let Some(parent) = manifest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let manifest = json!({
        "schema_version": 1,
        "mode": mode,
        "storage": "managed-greptime+turso",
        "base_url": public_base,
        "api_base_url": api_base,
        "health_url": format!("{public_base}/health"),
        "ready_url": format!("http://{ready_addr}/health"),
        "graphql_url": format!("{public_base}/graphql"),
        "otlp_http_url": otlp_http,
        "control_url": format!("tcp://{control_addr}"),
        "dataset_id": ids.dataset_id,
        "service": ids.service,
        "trace_id": ids.trace_id_hex,
        "span_id": ids.span_id_hex,
        "invocation_id": ids.invocation_id,
        "session_id": ids.session_id,
        "error_type": ids.error_type,
        "error_message": ids.error_message,
        "log_body": ids.log_body,
        "metric_name": ids.metric_name,
        "issue_fingerprint": fingerprint,
        "issue_status": issue.get("status").and_then(Value::as_str),
        "start_nanos": ids.start_nanos.to_string(),
        "pid": std::process::id(),
        "owns_process": owns,
        "data_dir": data_dir.as_ref().map(|p| p.display().to_string()),
    });
    std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)
        .with_context(|| format!("write {}", manifest_path.display()))?;

    println!("==> browser full-stack ready");
    println!("    mode:     {mode}");
    println!("    public:   {public_base}");
    println!("    api:      {api_base}");
    println!("    otlp:     {otlp_http}");
    println!("    control:  tcp://{control_addr}");
    println!("    ready:    http://{ready_addr}/health");
    println!("    dataset:  {}", ids.dataset_id);
    println!("    service:  {}", ids.service);
    println!("    trace:    {}", ids.trace_id_hex);
    println!("    issue:    {fingerprint}");
    println!("    manifest: {}", manifest_path.display());
    println!("Parallax browser full-stack ready — Ctrl-C / webServer stop to exit");

    let control_task = tokio::spawn(control_loop(control_listener, control));
    let ready_task = tokio::spawn(ready_loop(ready_listener));
    drop(tokio::signal::ctrl_c().await);
    control_task.abort();
    ready_task.abort();

    if let Some(handle) = server {
        println!("==> shutting down owned managed stack");
        handle.shutdown();
    } else {
        println!("==> attach mode: leaving foreign stack running");
    }
    if owns
        && let Some(dir) = data_dir
        && std::env::var("PARALLAX_FULL_STACK_KEEP_DATA")
            .ok()
            .as_deref()
            != Some("1")
    {
        drop(std::fs::remove_dir_all(dir));
    }
    Ok(())
}

async fn start_ui_proxy(ui_dist: &Path, api_base: &str, port: u16) -> Result<String> {
    let upstream = api_base.trim_end_matches('/').to_string();
    let shell = ServeFile::new(ui_dist.join("_shell.html"));
    let static_files = ServeDir::new(ui_dist).fallback(shell);
    let proxy_state = ProxyState { upstream };
    let app = Router::new()
        .route("/health", any(proxy_api))
        .route("/graphql", any(proxy_api))
        .route("/v1/{*rest}", any(proxy_api))
        .fallback_service(static_files)
        .with_state(proxy_state);
    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .with_context(|| format!("bind UI proxy port {port}"))?;
    let addr = listener.local_addr()?;
    println!("==> attach UI proxy on http://{addr} → {api_base}");
    tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, app).await {
            eprintln!("full-stack UI proxy error: {err}");
        }
    });
    Ok(format!("http://{addr}"))
}

#[derive(Clone)]
struct ProxyState {
    upstream: String,
}

async fn proxy_api(State(state): State<ProxyState>, request: Request<Body>) -> Response<Body> {
    let client = reqwest::Client::new();
    let method = request.method().clone();
    let path_and_query = request
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/")
        .to_string();
    let url = format!("{}{path_and_query}", state.upstream);
    let stream_path = path_and_query.contains("/stream");
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
            if stream_path {
                // SSE/live streams must not be buffered; forward the byte stream.
                let stream = upstream.bytes_stream();
                response
                    .body(Body::from_stream(stream))
                    .unwrap_or_else(|_| {
                        (StatusCode::BAD_GATEWAY, "upstream stream error").into_response()
                    })
            } else {
                let bytes = upstream.bytes().await.unwrap_or_default();
                response.body(Body::from(bytes)).unwrap_or_else(|_| {
                    (StatusCode::BAD_GATEWAY, "upstream body error").into_response()
                })
            }
        }
        Err(error) => (StatusCode::BAD_GATEWAY, format!("upstream: {error}")).into_response(),
    }
}

async fn ready_loop(listener: TcpListener) {
    loop {
        let Ok((mut stream, _)) = listener.accept().await else {
            continue;
        };
        let mut buf = [0u8; 1024];
        drop(tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await);
        let body = b"ok";
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: text/plain\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            std::str::from_utf8(body).unwrap_or("ok")
        );
        drop(stream.write_all(response.as_bytes()).await);
        drop(stream.shutdown().await);
    }
}

async fn control_loop(listener: TcpListener, state: ControlState) {
    loop {
        let Ok((stream, _)) = listener.accept().await else {
            continue;
        };
        let state = state.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_control(stream, state).await {
                eprintln!("full-stack control error: {err:#}");
            }
        });
    }
}

async fn handle_control(stream: TcpStream, state: ControlState) -> Result<()> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let request: ControlRequest = serde_json::from_str(line.trim()).context("control json")?;
    let response = match request.op.as_str() {
        "ping" => json!({ "ok": true }),
        "snapshot" => {
            let fp = state.issue_fingerprint.lock().await.clone();
            let issue = if let Some(fp) = fp {
                issue_by_fingerprint(&state.base_url, &fp).await.ok()
            } else {
                None
            };
            json!({
                "ok": true,
                "dataset_id": state.ids.dataset_id,
                "service": state.ids.service,
                "trace_id": state.ids.trace_id_hex,
                "issue": issue,
            })
        }
        "seed-live-log" => {
            let body = request
                .body
                .unwrap_or_else(|| format!("pw-live-{}", state.ids.dataset_id));
            let ts = parse_ts_or_now(request.ts_nanos.as_deref(), state.ids.start_nanos);
            let req = live_followup_log(&state.ids, &body, ts);
            post_proto(
                &reqwest::Client::new(),
                &format!("{}/v1/logs", state.otlp_http),
                req.encode_to_vec(),
            )
            .await?;
            json!({ "ok": true, "body": body, "ts_nanos": ts.to_string() })
        }
        "seed-live-log-burst" => {
            let count = request.count.unwrap_or(5).clamp(1, 50) as usize;
            let prefix = request
                .body
                .unwrap_or_else(|| format!("pw-live-burst-{}", state.ids.dataset_id));
            let base_ts = parse_ts_or_now(request.ts_nanos.as_deref(), state.ids.start_nanos);
            let mut rows: Vec<(String, u64)> = Vec::with_capacity(count);
            for i in 0..count {
                rows.push((format!("{prefix}-{i}"), base_ts.saturating_add(i as u64)));
            }
            let refs: Vec<(&str, u64)> = rows.iter().map(|(b, t)| (b.as_str(), *t)).collect();
            let req = live_followup_logs(&state.ids, &refs);
            post_proto(
                &reqwest::Client::new(),
                &format!("{}/v1/logs", state.otlp_http),
                req.encode_to_vec(),
            )
            .await?;
            json!({
                "ok": true,
                "prefix": prefix,
                "count": count,
                "bodies": rows.iter().map(|(b, _)| b.clone()).collect::<Vec<_>>(),
                "ts_nanos": base_ts.to_string(),
            })
        }
        "seed-live-log-duplicate-pair" => {
            // Two identical (body, ts) rows in one export — merge must keep one.
            let body = request
                .body
                .unwrap_or_else(|| format!("pw-live-dup-{}", state.ids.dataset_id));
            let ts = parse_ts_or_now(request.ts_nanos.as_deref(), state.ids.start_nanos);
            let req = live_followup_logs(&state.ids, &[(body.as_str(), ts), (body.as_str(), ts)]);
            post_proto(
                &reqwest::Client::new(),
                &format!("{}/v1/logs", state.otlp_http),
                req.encode_to_vec(),
            )
            .await?;
            json!({ "ok": true, "body": body, "ts_nanos": ts.to_string(), "count": 2 })
        }
        "seed-live-span" => {
            let name = request
                .span_name
                .or(request.body)
                .unwrap_or_else(|| format!("pw.live.span.{}", state.ids.dataset_id));
            let span_id = request.span_id.unwrap_or_else(|| {
                let nanos = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("clock")
                    .as_nanos();
                format!("{nanos:032x}")
                    .chars()
                    .rev()
                    .take(16)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect()
            });
            let ts = parse_ts_or_now(request.ts_nanos.as_deref(), state.ids.start_nanos);
            let req = live_followup_span(&state.ids, &span_id, &name, ts);
            post_proto(
                &reqwest::Client::new(),
                &format!("{}/v1/traces", state.otlp_http),
                req.encode_to_vec(),
            )
            .await?;
            json!({
                "ok": true,
                "span_name": name,
                "span_id": span_id,
                "ts_nanos": ts.to_string(),
            })
        }
        "seed-live-span-duplicate-pair" => {
            let name = request
                .span_name
                .or(request.body)
                .unwrap_or_else(|| format!("pw.live.span.dup.{}", state.ids.dataset_id));
            let span_id = request.span_id.unwrap_or_else(|| {
                let nanos = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("clock")
                    .as_nanos();
                format!("{nanos:032x}")
                    .chars()
                    .rev()
                    .take(16)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect()
            });
            let ts = parse_ts_or_now(request.ts_nanos.as_deref(), state.ids.start_nanos);
            let req = live_followup_spans(
                &state.ids,
                &[
                    (span_id.as_str(), name.as_str(), ts),
                    (span_id.as_str(), name.as_str(), ts),
                ],
            );
            post_proto(
                &reqwest::Client::new(),
                &format!("{}/v1/traces", state.otlp_http),
                req.encode_to_vec(),
            )
            .await?;
            json!({
                "ok": true,
                "span_name": name,
                "span_id": span_id,
                "ts_nanos": ts.to_string(),
                "count": 2,
            })
        }
        other => json!({ "ok": false, "error": format!("unknown op {other}") }),
    };
    let mut stream = reader.into_inner();
    stream
        .write_all(format!("{}\n", serde_json::to_string(&response)?).as_bytes())
        .await?;
    stream.shutdown().await?;
    Ok(())
}

async fn issue_by_fingerprint(base_url: &str, fingerprint: &str) -> Result<Value> {
    let client = reqwest::Client::new();
    let q = format!(
        r#"{{ issue(fingerprint: "{}") {{ fingerprint title status service errorType }} }}"#,
        fingerprint.replace('"', "")
    );
    let body = gql(&client, &format!("{base_url}/graphql"), &q).await?;
    body.pointer("/data/issue")
        .cloned()
        .context("issue missing")
}

async fn resolve_mode() -> Result<String> {
    if let Ok(mode) = std::env::var("PARALLAX_FULL_STACK_MODE") {
        return Ok(mode);
    }
    if ports_busy().await {
        println!(
            "==> Greptime ports {GREPTIME_PORTS:?} occupied — defaulting to attach \
             (set PARALLAX_FULL_STACK_MODE=managed after freeing ports)"
        );
        return Ok("attach".into());
    }
    Ok("managed".into())
}

async fn ports_busy() -> bool {
    for port in GREPTIME_PORTS {
        if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            return true;
        }
    }
    false
}

async fn ensure_ports_free() -> Result<()> {
    for port in GREPTIME_PORTS {
        if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            bail!(
                "managed full-stack requires free Greptime port {port}; \
                 stop the foreign process or use PARALLAX_FULL_STACK_MODE=attach"
            );
        }
    }
    Ok(())
}

fn seed_engine_binary(data_dir: &Path) -> Result<()> {
    let dest_dir = data_dir.join("bin");
    let dest = dest_dir.join("greptime");
    if dest.exists() {
        return Ok(());
    }
    let candidates = [
        PathBuf::from("/tmp/parallax-qa/data/bin/greptime"),
        std::env::home_dir()
            .map(|h| h.join(".parallax/bin/greptime"))
            .unwrap_or_default(),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/greptime-test-bin/greptime"),
    ];
    for src in candidates {
        if src.is_file() {
            std::fs::create_dir_all(&dest_dir)?;
            std::fs::copy(&src, &dest)
                .with_context(|| format!("copy greptime from {}", src.display()))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt as _;
                let mut perms = std::fs::metadata(&dest)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&dest, perms)?;
            }
            println!("==> seeded greptime binary from {}", src.display());
            return Ok(());
        }
    }
    Ok(())
}

async fn seed_otlp(otlp_http: &str, ids: &RealStackIds) -> Result<()> {
    let client = reqwest::Client::new();
    post_proto(
        &client,
        &format!("{otlp_http}/v1/traces"),
        traces_request(ids).encode_to_vec(),
    )
    .await?;
    post_proto(
        &client,
        &format!("{otlp_http}/v1/logs"),
        logs_request(ids).encode_to_vec(),
    )
    .await?;
    post_proto(
        &client,
        &format!("{otlp_http}/v1/metrics"),
        metrics_request(ids).encode_to_vec(),
    )
    .await?;
    Ok(())
}

fn parse_ts_or_now(raw: Option<&str>, start_nanos: u64) -> u64 {
    if let Some(raw) = raw
        && let Ok(parsed) = raw.parse::<u64>()
    {
        return parsed;
    }
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
    )
    .unwrap_or(start_nanos.saturating_add(10_000_000))
}

async fn post_proto(client: &reqwest::Client, url: &str, body: Vec<u8>) -> Result<()> {
    let response = client
        .post(url)
        .header("content-type", "application/x-protobuf")
        .body(body)
        .send()
        .await
        .with_context(|| format!("POST {url}"))?;
    if !response.status().is_success() {
        bail!("OTLP seed {url} status {}", response.status());
    }
    Ok(())
}

async fn wait_visibility(base_url: &str, ids: &RealStackIds, deadline: Duration) -> Result<Value> {
    let client = reqwest::Client::new();
    let graphql = format!("{base_url}/graphql");
    let started = Instant::now();
    let mut last = String::new();
    let mut issue: Option<Value> = None;
    let mut saw_trace = false;
    let mut saw_service = false;

    while started.elapsed() < deadline {
        let from = ids.start_nanos.saturating_sub(60_000_000_000);
        let to = ids.start_nanos.saturating_add(3_600_000_000_000);
        let traces_q = format!(
            r#"{{ recentTraces(limit: 50) {{ traceId service }} serviceList(fromNanos: "{from}", toNanos: "{to}") {{ name }} }}"#
        );
        if let Ok(body) = gql(&client, &graphql, &traces_q).await {
            last = body.to_string();
            if let Some(arr) = body.pointer("/data/recentTraces").and_then(Value::as_array) {
                saw_trace = arr.iter().any(|row| {
                    row.get("traceId").and_then(Value::as_str) == Some(ids.trace_id_hex.as_str())
                });
            }
            if let Some(arr) = body.pointer("/data/serviceList").and_then(Value::as_array) {
                saw_service = arr.iter().any(|row| {
                    row.get("name").and_then(Value::as_str) == Some(ids.service.as_str())
                });
            }
        }

        let issues_q =
            r#"{ issues(limit: 100) { items { fingerprint title errorType status service } } }"#;
        if let Ok(body) = gql(&client, &graphql, issues_q).await
            && let Some(arr) = body.pointer("/data/issues/items").and_then(Value::as_array)
            && let Some(found) = arr.iter().find(|row| {
                row.get("errorType").and_then(Value::as_str) == Some(ids.error_type.as_str())
                    || row.get("title").and_then(Value::as_str).is_some_and(|t| {
                        t.contains(&ids.error_type) || t.contains(&ids.error_message)
                    })
            })
        {
            issue = Some(found.clone());
        }

        if saw_trace && saw_service && issue.is_some() {
            return Ok(issue.expect("issue"));
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }

    bail!(
        "visibility timeout after {deadline:?}: saw_trace={saw_trace} saw_service={saw_service} \
         issue={} last={last}",
        issue.is_some()
    );
}

async fn gql(client: &reqwest::Client, url: &str, query: &str) -> Result<Value> {
    let response = client
        .post(url)
        .json(&json!({ "query": query }))
        .send()
        .await
        .context("graphql request")?;
    let body: Value = response.json().await.context("graphql json")?;
    Ok(body)
}

async fn wait_http_ok(url: &str, deadline: Duration) -> Result<()> {
    let client = reqwest::Client::new();
    let started = Instant::now();
    let mut last = String::new();
    while started.elapsed() < deadline {
        match client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => return Ok(()),
            Ok(resp) => last = format!("status {}", resp.status()),
            Err(err) => last = err.to_string(),
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    bail!("health wait failed for {url}: {last}");
}

fn workspace_root() -> Result<PathBuf> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for _ in 0..4 {
        if dir.join("Cargo.toml").is_file() && dir.join("ui").is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    bail!(
        "could not locate workspace root from {}",
        env!("CARGO_MANIFEST_DIR")
    );
}
