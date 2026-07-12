//! Parsed-command dispatch, kept separate from clap declarations and runtime setup.

use crate::client::{Client, gql_str, resolve_url};
use crate::{Cli, Command, IssueCommand, RunCommand, TraceCommand, commands, doctor, runtime};

pub(crate) async fn execute(cli: Cli, runtime: runtime::Runtime) -> anyhow::Result<()> {
    let client =
        || -> anyhow::Result<Client> { Ok(Client::new(resolve_url(cli.context.as_deref())?)) };
    match cli.command {
        Command::Serve { .. } => runtime::serve(runtime).await,
        Command::Run { command } => run(command, &client).await,
        Command::Issue { command } => issue(command, &client).await,
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

async fn run(
    command: RunCommand,
    client: &impl Fn() -> anyhow::Result<Client>,
) -> anyhow::Result<()> {
    match command {
        RunCommand::Start {
            otlp_forward,
            print_env,
            command,
        } => {
            let code = commands::run_start(&client()?, command, otlp_forward, print_env).await?;
            std::process::exit(code);
        }
        RunCommand::Finish { run_id, exit_code } => {
            commands::run_finish(&client()?, &run_id, exit_code).await
        }
        RunCommand::List => commands::run_list(&client()?).await,
        RunCommand::Inspect { run_id } => commands::run_inspect(&client()?, &run_id).await,
        RunCommand::Bundle { run_id, format } => {
            commands::run_bundle(&client()?, &run_id, format).await
        }
        RunCommand::Agent { run_id, format } => {
            commands::run_agent_session(&client()?, &run_id, format).await
        }
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
    }
}

async fn issue(
    command: IssueCommand,
    client: &impl Fn() -> anyhow::Result<Client>,
) -> anyhow::Result<()> {
    match command {
        IssueCommand::List { status, run } => {
            commands::issue_list(&client()?, status.as_deref(), run.as_deref()).await
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
