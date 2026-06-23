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
    version = env!("PARALLAX_VERSION"),
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
    /// Browse logs — the same filters as the UI's Logs page.
    Logs {
        /// Trace id to scope to.
        #[arg(long, conflicts_with = "run")]
        trace: Option<String>,
        /// Run id to scope to.
        #[arg(long)]
        run: Option<String>,
        /// Service name to scope to.
        #[arg(long)]
        service: Option<String>,
        /// Minimum severity: trace | debug | info | warn | error | fatal.
        #[arg(long)]
        level: Option<String>,
        /// Only lines whose body contains this substring.
        #[arg(long, alias = "query")]
        grep: Option<String>,
        /// Time window, e.g. 15m, 2h, 7d (default 15m; ignored with --trace/--run).
        #[arg(long, default_value = "15m")]
        since: String,
        /// Max lines (newest first).
        #[arg(long, default_value_t = 100)]
        limit: u32,
        /// Live tail (kubectl-style): stream new matching logs as they arrive.
        #[arg(long, short = 'f')]
        follow: bool,
        /// With --follow: stop after this window and report the match count
        /// (agent verification: "does it still appear?"), e.g. 30s, 5m.
        #[arg(long = "for", requires = "follow")]
        follow_for: Option<String>,
    },
    /// Browse traces — the same filters as the UI's Traces page.
    Traces {
        /// Run id to scope to (anchored read; other filters ignored).
        #[arg(long)]
        run: Option<String>,
        /// Service name to scope to.
        #[arg(long)]
        service: Option<String>,
        /// Only traces whose root span is at least this long, e.g. 500ms, 2s.
        #[arg(long)]
        min_duration: Option<String>,
        /// Only traces containing an error span.
        #[arg(long)]
        errors: bool,
        /// Only root spans whose name contains this substring.
        #[arg(long, alias = "query")]
        grep: Option<String>,
        /// Time window, e.g. 15m, 2h, 7d.
        #[arg(long, default_value = "15m")]
        since: String,
        /// Max traces (newest first).
        #[arg(long, default_value_t = 50)]
        limit: u32,
        /// Live tail: stream finished spans matching the filters.
        #[arg(long, short = 'f')]
        follow: bool,
        /// With --follow: stop after this window and report the match count.
        #[arg(long = "for", requires = "follow")]
        follow_for: Option<String>,
    },
    /// Run a read-only SQL query against the telemetry engine (GreptimeDB).
    Sql {
        /// The SELECT-shaped statement, e.g.
        /// "SELECT * FROM opentelemetry_logs ORDER BY timestamp DESC LIMIT 10".
        query: String,
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
        /// Compare mode: forward child telemetry to a collector instead of
        /// Parallax. A URL, `rotel` (the configured hub), or `off`. Also settable
        /// ambiently via `PARALLAX_OTLP_FORWARD`.
        #[arg(long = "otlp-forward", value_name = "TARGET")]
        otlp_forward: Option<String>,
        /// Print the OTel env that would be injected, then exit (dry-run).
        #[arg(long = "print-env")]
        print_env: bool,
        /// Everything after `--` is the wrapped command.
        #[arg(last = true)]
        command: Vec<String>,
    },
    /// Close a bare-mode run.
    Finish { run_id: String, exit_code: i32 },
    /// Show one run's record (status, counts, issues).
    Inspect { run_id: String },
    /// The run-anchored evidence bundle (Markdown).
    Bundle { run_id: String },
    /// List recent runs.
    List,
    /// Live tail of one run: new logs + finished spans, interleaved.
    Watch {
        run_id: String,
        /// Minimum log severity: trace | debug | info | warn | error | fatal.
        #[arg(long)]
        level: Option<String>,
        /// Only log lines whose body contains this substring.
        #[arg(long, alias = "query")]
        grep: Option<String>,
        /// Stop after this window and report match counts, e.g. 30s, 5m.
        #[arg(long = "for")]
        watch_for: Option<String>,
    },
}

#[derive(Subcommand)]
enum IssueCommand {
    /// List grouped errors (newest activity first).
    List {
        /// Filter by workflow status (open | resolved).
        #[arg(long)]
        status: Option<String>,
        /// Only issues whose events fell inside this run's traces.
        #[arg(long)]
        run: Option<String>,
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
    let cli = Cli::parse();

    // Self-telemetry (serve only): resolve + build the OTLP export pipeline
    // before the subscriber is installed, then attach its layers alongside the
    // console `fmt` layer. The serve config loaded here is reused by the arm.
    let mut serve_config: Option<parallax_server::Config> = None;
    let mut self_telemetry: Option<parallax_server::self_telemetry::Installed> = None;
    if let Command::Serve { config } = &cli.command {
        let default_path = std::env::home_dir().map(|h| h.join(".parallax/config.toml"));
        let path = config.clone().or(default_path);
        let cfg = parallax_server::Config::load(path.as_deref())?;
        if let Some(endpoint) = parallax_server::self_telemetry::resolve_endpoint(&cfg) {
            self_telemetry = Some(parallax_server::self_telemetry::install(&endpoint)?);
        }
        serve_config = Some(cfg);
    }

    let (otel_layers, telemetry_guard, telemetry_endpoint) = match self_telemetry {
        Some(parallax_server::self_telemetry::Installed {
            layers,
            guard,
            endpoint,
        }) => (layers, Some(guard), Some(endpoint)),
        None => (Vec::new(), None, None),
    };

    {
        use tracing_subscriber::Layer;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;
        let env =
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
        tracing_subscriber::registry()
            .with(otel_layers)
            .with(tracing_subscriber::fmt::layer().with_filter(env))
            .init();
    }
    let client =
        || -> anyhow::Result<Client> { Ok(Client::new(resolve_url(cli.context.as_deref())?)) };

    match cli.command {
        Command::Serve { .. } => {
            // Config was loaded above (to resolve self-telemetry); reuse it.
            let config = serve_config.expect("serve config loaded above");
            let handle = parallax_server::start(&config).await?;
            let storage = match config.storage.mode.as_str() {
                "none" => "in-memory (degraded; data lost on exit)".to_string(),
                "external" => format!("external GreptimeDB at {}", config.storage.greptime_url),
                _ => "managed GreptimeDB on 127.0.0.1:24000".to_string(),
            };
            println!();
            println!("  Parallax ready — Ctrl-C to stop");
            println!();
            println!("    UI         http://{}", handle.api_addr);
            println!("    GraphQL    http://{}/graphql", handle.api_addr);
            println!("    OTLP/gRPC  {}", handle.otlp_grpc_addr);
            println!("    OTLP/HTTP  {}", handle.otlp_http_addr);
            println!("    storage    {storage}");
            println!("    data       {}", config.data_dir().display());
            match &telemetry_endpoint {
                Some(endpoint) => {
                    println!("    self-otlp   parallax → {endpoint} (ingest path suppressed)")
                }
                None => println!("    self-otlp   off (set PARALLAX_SELF_OTLP to export)"),
            }
            println!();
            // SIGTERM must also shut down cleanly — dying without cleanup
            // orphans the managed engine child on its ports.
            let mut sigterm =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {},
                _ = sigterm.recv() => {},
            }
            handle.shutdown();
            // Flush buffered self-telemetry before exit.
            if let Some(guard) = &telemetry_guard {
                guard.shutdown();
            }
            Ok(())
        }
        Command::Run { command } => match command {
            RunCommand::Start {
                otlp_forward,
                print_env,
                command,
            } => {
                let code =
                    commands::run_start(&client()?, command, otlp_forward, print_env).await?;
                std::process::exit(code);
            }
            RunCommand::Finish { run_id, exit_code } => {
                commands::run_finish(&client()?, &run_id, exit_code).await
            }
            RunCommand::List => commands::run_list(&client()?).await,
            RunCommand::Inspect { run_id } => commands::run_inspect(&client()?, &run_id).await,
            RunCommand::Bundle { run_id } => commands::run_bundle(&client()?, &run_id).await,
            RunCommand::Watch {
                run_id,
                level,
                grep,
                watch_for,
            } => {
                commands::run_watch(
                    &client()?,
                    &run_id,
                    level.as_deref(),
                    grep.as_deref(),
                    watch_for.as_deref(),
                )
                .await
            }
        },
        Command::Issue { command } => match command {
            IssueCommand::List { status, run } => {
                commands::issue_list(&client()?, status.as_deref(), run.as_deref()).await
            }
            IssueCommand::Context { fingerprint } => {
                commands::issue_context(&client()?, &fingerprint).await
            }
            IssueCommand::Resolve { fingerprint } => {
                let client = client()?;
                client
                    .graphql(&format!(
                        r#"mutation {{ issueSetStatus(fingerprint: "{}", status: "resolved") {{ status }} }}"#,
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
        Command::Logs {
            trace,
            run,
            service,
            level,
            grep,
            since,
            limit,
            follow,
            follow_for,
        } => {
            let filter = commands::LogsFilter {
                trace: trace.as_deref(),
                run: run.as_deref(),
                service: service.as_deref(),
                level: level.as_deref(),
                grep: grep.as_deref(),
                since: &since,
                limit,
            };
            if follow {
                commands::logs_follow(&client()?, filter, follow_for.as_deref()).await
            } else {
                commands::logs(&client()?, filter).await
            }
        }
        Command::Traces {
            run,
            service,
            min_duration,
            errors,
            grep,
            since,
            limit,
            follow,
            follow_for,
        } => {
            let filter = commands::TracesFilter {
                service: service.as_deref(),
                run: run.as_deref(),
                min_duration: min_duration.as_deref(),
                errors_only: errors,
                grep: grep.as_deref(),
                since: &since,
                limit,
            };
            if follow {
                commands::traces_follow(&client()?, filter, follow_for.as_deref()).await
            } else {
                commands::traces(&client()?, filter).await
            }
        }
        Command::Sql { query } => commands::sql(&client()?, &query).await,
        Command::Doctor => doctor::doctor().await,
        Command::Prune => doctor::prune(),
        Command::Uninstall { purge, yes } => doctor::uninstall(purge, yes),
    }
}
