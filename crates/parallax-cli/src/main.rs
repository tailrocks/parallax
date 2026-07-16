//! Installed thin API client (`--context` selects the server) plus the `serve`
//! subcommand that embeds the server library.
mod client;
mod commands;
mod dispatch;
mod doctor;
mod runtime;

use clap::{Parser, Subcommand, ValueEnum};

const RELEASE_IDENTITY: &str = concat!("parallax-release-identity:", env!("PARALLAX_VERSION"));

/// Output shape for agent-facing projections (bundles, agent sessions).
/// Markdown is the human default; JSON is the machine/agent contract.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Markdown,
    Json,
}

#[derive(Parser)]
#[command(
    name = "parallax",
    version = env!("PARALLAX_VERSION"),
    about = "Local-first observability for agent-assisted development"
)]
pub(crate) struct Cli {
    /// Named context from ~/.parallax/contexts.toml (default: local).
    #[arg(long, global = true)]
    context: Option<String>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Start the Parallax server (OTLP ingest + API + UI).
    Serve {
        /// Path to config.toml (default: ~/.parallax/config.toml when present).
        #[arg(long)]
        config: Option<std::path::PathBuf>,
    },
    /// CLI invocations: bounded, inspectable execution units.
    Invocation {
        #[command(subcommand)]
        command: InvocationCommand,
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
        #[arg(long, conflicts_with = "invocation")]
        trace: Option<String>,
        /// Invocation id to scope to.
        #[arg(long)]
        invocation: Option<String>,
        /// Service name to scope to.
        #[arg(long)]
        service: Option<String>,
        /// Minimum severity: trace | debug | info | warn | error | fatal.
        #[arg(long)]
        level: Option<String>,
        /// Only lines whose body contains this substring.
        #[arg(long, alias = "query")]
        grep: Option<String>,
        /// Time window, e.g. 15m, 2h, 7d (default 15m; ignored with --trace/--invocation).
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
        /// Invocation id to scope to (anchored read; other filters ignored).
        #[arg(long)]
        invocation: Option<String>,
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
    /// Run a read-only SQL query against the telemetry engine (`GreptimeDB`).
    Sql {
        /// The SELECT-shaped statement, e.g.
        /// "SELECT * FROM `opentelemetry_logs` ORDER BY timestamp DESC LIMIT 10".
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
pub(crate) enum InvocationCommand {
    /// Start an invocation. With `-- <command…>`: wrapper mode (injects `OTel`
    /// env, captures the exit code). Without: prints exports to source.
    Start {
        /// Compare mode: forward child telemetry to a collector instead of
        /// Parallax. A URL, `rotel` (the configured hub), or `off`. Also settable
        /// ambiently via `PARALLAX_OTLP_FORWARD`.
        #[arg(long = "otlp-forward", value_name = "TARGET")]
        otlp_forward: Option<String>,
        /// Print the `OTel` env that would be injected, then exit (dry-run).
        #[arg(long = "print-env")]
        print_env: bool,
        /// Everything after `--` is the wrapped command.
        #[arg(last = true)]
        command: Vec<String>,
    },
    /// Close a bare-mode invocation.
    Finish {
        invocation_id: String,
        exit_code: i32,
    },
    /// Show one invocation's record (status, counts, issues).
    Inspect { invocation_id: String },
    /// The invocation-anchored evidence bundle (Markdown by default; `--format json` for canonical JSON).
    Bundle {
        invocation_id: String,
        #[arg(long = "format", value_enum, default_value = "markdown")]
        format: OutputFormat,
    },
    /// Agent-session projection for an invocation (tool steps, token totals).
    Agent {
        invocation_id: String,
        #[arg(long = "format", value_enum, default_value = "markdown")]
        format: OutputFormat,
    },
    /// List recent invocations.
    List,
    /// Live tail of one invocation: new logs + finished spans, interleaved.
    Watch {
        invocation_id: String,
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
pub(crate) enum IssueCommand {
    /// List grouped errors (newest activity first).
    List {
        /// Filter by workflow status (open | resolved).
        #[arg(long)]
        status: Option<String>,
        /// Only issues whose events fell inside this invocation's traces.
        #[arg(long)]
        invocation: Option<String>,
    },
    /// The agent handoff: Markdown evidence for one issue (`--format json` for canonical JSON).
    Context {
        fingerprint: String,
        #[arg(long = "format", value_enum, default_value = "markdown")]
        format: OutputFormat,
    },
    /// Mark an issue resolved.
    Resolve { fingerprint: String },
}

#[derive(Subcommand)]
pub(crate) enum TraceCommand {
    /// Show a trace's spans and correlated logs by trace id.
    Inspect { trace_id: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::hint::black_box(RELEASE_IDENTITY);
    let cli = Cli::parse();
    let runtime = runtime::prepare(&cli.command)?;
    dispatch::execute(cli, runtime).await
}
