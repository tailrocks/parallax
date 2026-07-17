//! Parsed-command dispatch, kept separate from clap declarations and runtime setup.

use crate::client::{Client, gql_str, resolve_url};
use crate::{
    Cli, Command, InvocationCommand, IssueCommand, TraceCommand, commands, doctor, runtime,
};

pub(crate) async fn execute(cli: Cli, runtime: runtime::Runtime) -> anyhow::Result<()> {
    let client =
        || -> anyhow::Result<Client> { Ok(Client::new(resolve_url(cli.context.as_deref())?)) };
    match cli.command {
        Command::Serve { .. } => runtime::serve(runtime).await,
        Command::Invocation { command } => invocation(command, &client).await,
        Command::Issue { command } => issue(command, &client).await,
        Command::Trace { command } => match command {
            TraceCommand::Inspect { trace_id } => {
                commands::trace_inspect(&client()?, &trace_id).await
            }
        },
        Command::Metrics {
            invocation,
            run,
            since,
            json,
        } => {
            if run.is_some() {
                anyhow::bail!(
                    "--run is retired: CLI runs became invocations — use --invocation <id>"
                );
            }
            let Some(invocation) = invocation else {
                anyhow::bail!("--invocation <id> is required (see `parallax invocation list`)");
            };
            commands::metrics_invocation(&client()?, &invocation, &since, json).await
        }
        Command::Logs {
            trace,
            invocation,
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
                invocation: invocation.as_deref(),
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
            invocation,
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
                invocation: invocation.as_deref(),
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
        Command::Prune { execute, yes, json } => doctor::prune(execute, yes, json).await,
        Command::Uninstall { purge, yes } => doctor::uninstall(purge, yes),
    }
}

async fn invocation(
    command: InvocationCommand,
    client: &impl Fn() -> anyhow::Result<Client>,
) -> anyhow::Result<()> {
    match command {
        InvocationCommand::Start {
            otlp_forward,
            print_env,
            command,
        } => {
            let code =
                commands::invocation_start(&client()?, command, otlp_forward, print_env).await?;
            std::process::exit(code);
        }
        InvocationCommand::Finish {
            invocation_id,
            exit_code,
        } => commands::invocation_finish(&client()?, &invocation_id, exit_code).await,
        InvocationCommand::List => commands::invocation_list(&client()?).await,
        InvocationCommand::Inspect { invocation_id } => {
            commands::invocation_inspect(&client()?, &invocation_id).await
        }
        InvocationCommand::Bundle {
            invocation_id,
            format,
        } => commands::invocation_bundle(&client()?, &invocation_id, format).await,
        InvocationCommand::Agent {
            invocation_id,
            format,
        } => commands::invocation_agent_session(&client()?, &invocation_id, format).await,
        InvocationCommand::Watch {
            invocation_id,
            level,
            grep,
            watch_for,
        } => {
            commands::invocation_watch(
                &client()?,
                &invocation_id,
                level.as_deref(),
                grep.as_deref(),
                watch_for.as_deref(),
            )
            .await
        }
    }
}

async fn issue(
    command: IssueCommand,
    client: &impl Fn() -> anyhow::Result<Client>,
) -> anyhow::Result<()> {
    match command {
        IssueCommand::List { status, invocation } => {
            commands::issue_list(&client()?, status.as_deref(), invocation.as_deref()).await
        }
        IssueCommand::Context {
            fingerprint,
            format,
        } => commands::issue_context(&client()?, &fingerprint, format).await,
        IssueCommand::Resolve { fingerprint } => {
            client()?.graphql(&format!(r#"mutation {{ issueSetStatus(fingerprint: "{}", status: "resolved") {{ status }} }}"#, gql_str(&fingerprint))).await?;
            println!("issue {fingerprint} resolved");
            Ok(())
        }
    }
}
