//! SPIKE binary: stdio MCP read-only context adapter + projection-equivalence check.
//!
//! Not a product surface. See crate README and
//! `docs/research/validation/2026-07-11-mcp-spike-projection-equivalence.md`.

mod check;
mod gql;
mod server;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "parallax-mcp-spike",
    about = "SPIKE: read-only stdio MCP adapter over Parallax GraphQL (not product)"
)]
struct Cli {
    /// Base URL of the Parallax API (default local serve).
    /// Override with env `PARALLAX_URL` if desired (read manually for clap without env feature).
    #[arg(long, default_value = "http://127.0.0.1:4000", global = true)]
    url: String,

    /// Explicitly trust and start the local stdio MCP server.
    #[arg(long, global = true)]
    allow_local_stdio: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Run the stdio MCP server (default when no subcommand is given).
    Serve,
    /// Prove CLI ≡ HTTP ≡ MCP raw canonical JSON for the given anchors.
    Check {
        /// Issue fingerprint for `parallax issue context` / `parallax_issue_context`.
        #[arg(long)]
        fingerprint: Option<String>,
        /// Invocation id for `parallax invocation bundle` (second anchor when available).
        #[arg(long)]
        invocation_id: Option<String>,
        /// Path to the `parallax` CLI binary.
        #[arg(long, default_value = "parallax")]
        parallax_bin: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logging to stderr only — stdout is the MCP JSON-RPC wire.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let mut cli = Cli::parse();
    if let Ok(url) = std::env::var("PARALLAX_URL")
        && !url.is_empty()
    {
        // Spike: env wins when set (clap workspace lacks the env feature).
        cli.url = url;
    }
    cli.url = gql::normalize_local_base_url(&cli.url)?;
    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => {
            if !cli.allow_local_stdio {
                anyhow::bail!(
                    "local stdio MCP is disabled; re-run with --allow-local-stdio after reviewing the command and configuration"
                );
            }
            server::run_stdio(cli.url).await
        }
        Command::Check {
            fingerprint,
            invocation_id,
            parallax_bin,
        } => {
            check::run(check::CheckArgs {
                base_url: cli.url,
                fingerprint,
                invocation_id,
                parallax_bin,
            })
            .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_stdio_requires_explicit_cli_opt_in() {
        let default = Cli::try_parse_from(["parallax-mcp-spike"]).expect("parse default");
        let trusted = Cli::try_parse_from(["parallax-mcp-spike", "--allow-local-stdio"])
            .expect("parse opt-in");

        assert!(!default.allow_local_stdio);
        assert!(trusted.allow_local_stdio);
    }

    #[test]
    fn api_url_is_loopback_only_until_remote_auth_lands() {
        for accepted in [
            "http://localhost:4000",
            "http://127.0.0.1:4000/",
            "http://[::1]:4000",
        ] {
            assert!(
                gql::normalize_local_base_url(accepted).is_ok(),
                "{accepted}"
            );
        }
        for denied in [
            "https://localhost:4000",
            "http://example.com:4000",
            "http://user:secret@localhost:4000",
            "http://localhost:4000/graphql",
            "http://localhost:4000?token=secret",
            "not-a-url",
        ] {
            assert!(gql::normalize_local_base_url(denied).is_err(), "{denied}");
        }
    }
}
