//! The installed `parallax` binary: thin client of the local API, plus the
//! `serve` subcommand embedding the server library (one binary, per the
//! implementation spec).

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "parallax",
    version,
    about = "Local-first observability for agent-assisted development"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start the Parallax server (OTLP ingest + API + UI).
    Serve {
        /// Path to config.toml (default: ~/.parallax/config.toml when present).
        #[arg(long)]
        config: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::Serve { config } => {
            let default_path = std::env::home_dir().map(|h| h.join(".parallax/config.toml"));
            let path = config.or(default_path);
            let config = parallax_server::Config::load(path.as_deref())?;
            let handle = parallax_server::start(&config).await?;
            tracing::info!(
                api = %handle.api_addr,
                otlp_grpc = %handle.otlp_grpc_addr,
                otlp_http = %handle.otlp_http_addr,
                "parallax serve running; Ctrl-C to stop"
            );
            tokio::signal::ctrl_c().await?;
            handle.shutdown();
            Ok(())
        }
    }
}
