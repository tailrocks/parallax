//! Grouped-issue listing and evidence-context commands.

use super::forwarding::relative;
use super::output::render_bundle;
use crate::OutputFormat;
use crate::client::{Client, gql_str};

pub(crate) async fn issue_list(
    client: &Client,
    status: Option<&str>,
    run: Option<&str>,
) -> anyhow::Result<()> {
    // Run scoping reads the run's issues; otherwise the filtered issue list.
    let (pointer, query) = match run {
        Some(run_id) => (
            "/data/run/issues",
            format!(
                r#"{{ run(runId: "{}") {{ issues {{ fingerprint title service status eventCount lastSeenNanos }} }} }}"#,
                gql_str(run_id)
            ),
        ),
        None => (
            "/data/issues/items",
            format!(
                r#"{{ issues{} {{ items {{ fingerprint title service status eventCount lastSeenNanos }} }} }}"#,
                status
                    .map(|s| format!(r#"(status: "{}")"#, gql_str(s)))
                    .unwrap_or_default()
            ),
        ),
    };
    let response = client.graphql(&query).await?;
    if run.is_some() && response.pointer("/data/run").is_some_and(|v| v.is_null()) {
        anyhow::bail!("run {} not found", run.unwrap_or_default());
    }
    let mut issues = response
        .pointer(pointer)
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if let Some(status) = status {
        // The run path has no server-side status filter; apply it here.
        issues.retain(|i| i["status"].as_str() == Some(status));
    }
    if issues.is_empty() {
        println!("no issues — either your code is perfect or nothing is sending telemetry yet");
        return Ok(());
    }
    println!(
        "{:<18} {:<8} {:>6}  {:<10} {:<12} title",
        "FINGERPRINT", "STATUS", "EVENTS", "LAST SEEN", "SERVICE"
    );
    for issue in issues {
        println!(
            "{:<18} {:<8} {:>6}  {:<10} {:<12} {}",
            issue["fingerprint"].as_str().unwrap_or("-"),
            issue["status"].as_str().unwrap_or("-"),
            issue["eventCount"].as_u64().unwrap_or(0),
            relative(issue["lastSeenNanos"].as_str().unwrap_or("0")),
            issue["service"].as_str().unwrap_or("-"),
            issue["title"].as_str().unwrap_or("-"),
        );
    }
    Ok(())
}

/// `parallax issue context <fingerprint>` — the agent handoff: the bounded,
/// redacted, hypothesis-ranked evidence bundle, rendered by the server.
pub(crate) async fn issue_context(
    client: &Client,
    fingerprint: &str,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let query = match format {
        OutputFormat::Markdown => format!(
            r#"{{ bundle(fingerprint: "{}") {{ markdown canonicalHash }} }}"#,
            gql_str(fingerprint)
        ),
        OutputFormat::Json => format!(
            r#"{{ bundle(fingerprint: "{}") {{ json canonicalHash }} }}"#,
            gql_str(fingerprint)
        ),
    };
    let response = client.graphql(&query).await?;
    let Some(bundle) = response.pointer("/data/bundle").filter(|v| !v.is_null()) else {
        anyhow::bail!("issue {fingerprint} not found");
    };
    let (stdout, stderr) = render_bundle(format, bundle);
    print!("{stdout}");
    eprint!("{stderr}");
    Ok(())
}
