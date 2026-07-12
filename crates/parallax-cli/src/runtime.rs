//! Subscriber setup and embedded-server lifecycle for the CLI.

use crate::Command;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub(crate) struct Runtime {
    pub(crate) serve_config: Option<parallax_server::Config>,
    telemetry_guard: Option<parallax_server::SelfTelemetry>,
    telemetry_endpoint: Option<String>,
}

pub(crate) fn prepare(command: &Command) -> anyhow::Result<Runtime> {
    let mut serve_config = None;
    let mut self_telemetry = None;
    if let Command::Serve { config } = command {
        let default_path = std::env::home_dir().map(|home| home.join(".parallax/config.toml"));
        let config = parallax_server::Config::load(config.clone().or(default_path).as_deref())?;
        if let Some(endpoint) = parallax_server::resolve_self_telemetry_endpoint(&config) {
            self_telemetry = Some(parallax_server::install_self_telemetry(&endpoint)?);
        }
        serve_config = Some(config);
    }
    let (layers, telemetry_guard, telemetry_endpoint) = match self_telemetry {
        Some(parallax_server::InstalledSelfTelemetry {
            layers,
            guard,
            endpoint,
        }) => (layers, Some(guard), Some(endpoint)),
        None => (Vec::new(), None, None),
    };
    let filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    tracing_subscriber::registry()
        .with(layers)
        .with(tracing_subscriber::fmt::layer().with_filter(filter))
        .init();
    Ok(Runtime {
        serve_config,
        telemetry_guard,
        telemetry_endpoint,
    })
}

pub(crate) async fn serve(runtime: Runtime) -> anyhow::Result<()> {
    let config = runtime
        .serve_config
        .ok_or_else(|| anyhow::anyhow!("serve config missing"))?;
    let handle = parallax_server::start(&config).await?;
    let storage = match config.storage.mode.as_str() {
        "external" => format!("external GreptimeDB at {}", config.storage.greptime_url),
        "managed" => "managed GreptimeDB on 127.0.0.1:24000".to_string(),
        mode => anyhow::bail!("unsupported validated storage mode {mode:?}"),
    };
    println!();
    println!("  Parallax ready — Ctrl-C to stop");
    println!();
    println!("    UI         http://{}", handle.api_addr);
    println!("    GraphQL    http://{}/graphql", handle.api_addr);
    println!("    OTLP/gRPC  {}", handle.otlp_grpc_addr);
    println!("    OTLP/HTTP  {}", handle.otlp_http_addr);
    println!("    storage    {storage}");
    println!(
        "    metadata   Turso at {}",
        config.data_dir().join("meta.db").display()
    );
    println!("    data       {}", config.data_dir().display());
    match &runtime.telemetry_endpoint {
        Some(endpoint) => {
            println!("    self-otlp   parallax → {endpoint} (ingest path suppressed)");
        }
        None => println!("    self-otlp   off (set PARALLAX_SELF_OTLP to export)"),
    }
    println!();
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = sigterm.recv() => {},
    }
    handle.shutdown();
    if let Some(guard) = &runtime.telemetry_guard {
        guard.shutdown();
    }
    Ok(())
}
