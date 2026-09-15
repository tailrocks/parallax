//! Sentry envelope fan-out proxy — Selene §11 scaffold binary.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::serve;
use clap::Parser;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use parallax_sentry_proxy::{Config, FanOut, http};

#[derive(Parser, Debug)]
#[command(
    name = "parallax-sentry-proxy",
    about = "Sentry-compatible envelope fan-out proxy for Selene observability"
)]
struct Cli {
    /// Path to TOML configuration file.
    #[arg(long, default_value = "/etc/parallax-sentry-proxy/config.toml")]
    config: PathBuf,

    /// Override listen address from config (e.g. 0.0.0.0:8080).
    #[arg(long)]
    listen: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let cli = Cli::parse();
    let config = Config::load(&cli.config)?;
    let listen = cli.listen.unwrap_or(config.listen.clone());
    let fanout = FanOut::spawn(&config);
    let state = http::AppState {
        config: Arc::new(config),
        fanout: Arc::new(fanout),
    };
    let app = http::router(state);

    let addr: SocketAddr = listen.parse()?;
    let listener = TcpListener::bind(addr).await?;
    tracing::info!(%addr, "parallax-sentry-proxy listening");
    serve(listener, app).await?;
    Ok(())
}
