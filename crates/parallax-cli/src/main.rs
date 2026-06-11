//! The installed `parallax` binary: thin client of the canonical API
//! (kubectl model — `--context` selects the server), plus the `serve`
//! subcommand embedding the server library.

mod client;
mod commands;
mod doctor;

use clap::{Parser, Subcommand};
use client::{Client, resolve_url};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "parallax",
    version,
    about = "Local-first observability for agent-assisted development"
)]
struct Cli {
    /// Named context from ~/.parallax/contexts.toml (default: local).
    #[arg(long, global = true)]
    context: Option<String>,
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
    /// Runs: bounded, inspectable execution units.
    Run {
        #[command(subcommand)]
        command: RunCommand,
    },
    /// Grouped errors.
    Issue {
        #[command(subcommand)]
        command: IssueCommand,
    },
    /// Traces.
    Trace {
        #[command(subcommand)]
        command: TraceCommand,
    },
    /// Correlated logs for a trace or a run.
    Logs {
        /// Trace id to fetch logs for.
        #[arg(long, conflicts_with = "run", required_unless_present = "run")]
        trace: Option<String>,
        /// Run id to fetch logs for.
        #[arg(long)]
        run: Option<String>,
        /// Only lines whose body contains this substring.
        #[arg(long)]
        grep: Option<String>,
    },
    /// Diagnose the local install (server, engine, spool, sizes).
    Doctor,
    /// Reclaim spool space now (telemetry TTLs are engine-managed).
    Prune,
    /// Remove the Parallax data directory.
    Uninstall {
        /// Actually delete the data directory.
        #[arg(long)]
        purge: bool,
        /// Skip the confirmation.
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum RunCommand {
    /// Start a run. With `-- <command…>`: wrapper mode (injects OTel env,
    /// captures the exit code). Without: prints exports to source.
    Start {
        /// Everything after `--` is the wrapped command.
        #[arg(last = true)]
        command: Vec<String>,
    },
    /// Close a bare-mode run.
    Finish { run_id: String, exit_code: i32 },
    /// Show one run's record.
    Inspect { run_id: String },
    /// List recent runs.
    List,
}

#[derive(Subcommand)]
enum IssueCommand {
    /// List grouped errors (newest activity first).
    List {
        #[arg(long)]
        status: Option<String>,
    },
    /// The agent handoff: Markdown evidence for one issue.
    Context { fingerprint: String },
    /// Mark an issue resolved.
    Resolve { fingerprint: String },
}

#[derive(Subcommand)]
enum TraceCommand {
    /// Show a trace's spans and correlated logs by trace id.
    Inspect { trace_id: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();
    let client =
        || -> anyhow::Result<Client> { Ok(Client::new(resolve_url(cli.context.as_deref())?)) };

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
        Command::Run { command } => match command {
            RunCommand::Start { command } => {
                let code = commands::run_start(&client()?, command).await?;
                std::process::exit(code);
            }
            RunCommand::Finish { run_id, exit_code } => {
                commands::run_finish(&client()?, &run_id, exit_code).await
            }
            RunCommand::List => commands::run_list(&client()?).await,
            RunCommand::Inspect { run_id } => commands::run_inspect(&client()?, &run_id).await,
        },
        Command::Issue { command } => match command {
            IssueCommand::List { status } => {
                commands::issue_list(&client()?, status.as_deref()).await
            }
            IssueCommand::Context { fingerprint } => {
                commands::issue_context(&client()?, &fingerprint).await
            }
            IssueCommand::Resolve { fingerprint } => {
                let client = client()?;
                client
                    .graphql(&format!(
                        r#"mutation {{ issueSetStatus(fingerprint: "{}", status: "resolved") }}"#,
                        client::gql_str(&fingerprint)
                    ))
                    .await?;
                println!("issue {fingerprint} resolved");
                Ok(())
            }
        },
        Command::Trace { command } => match command {
            TraceCommand::Inspect { trace_id } => {
                commands::trace_inspect(&client()?, &trace_id).await
            }
        },
        Command::Logs { trace, run, grep } => {
            commands::logs(
                &client()?,
                trace.as_deref(),
                run.as_deref(),
                grep.as_deref(),
            )
            .await
        }
        Command::Doctor => doctor::doctor().await,
        Command::Prune => doctor::prune(),
        Command::Uninstall { purge, yes } => doctor::uninstall(purge, yes),
    }
}
