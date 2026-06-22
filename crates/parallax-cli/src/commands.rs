//! CLI commands over the API: runs (with wrapper mode), issues, traces, logs.

use crate::client::{Client, gql_str};
use std::time::{SystemTime, UNIX_EPOCH};

/// Default destination for child telemetry: Parallax's own OTLP receiver.
const DEFAULT_PARALLAX_ENDPOINT: &str = "http://127.0.0.1:4317";
/// What `--otlp-forward rotel` (and `PARALLAX_OTLP_FORWARD=rotel`) resolves to —
/// the lab's Rotel collector published on the host.
const DEFAULT_ROTEL_ENDPOINT: &str = "http://localhost:4317";
const OTLP_GRPC_PROTOCOL: &str = "grpc";
const OTLP_HTTP_PROTOCOL: &str = "http";

/// Resolved compare-mode forwarding target for `run start`.
struct Forward {
    endpoint: String,
    protocol: &'static str,
    /// True when child telemetry is forwarded to a collector (Rotel) rather than
    /// straight to Parallax — i.e. fan-out comparison mode.
    compare: bool,
}

/// OTLP HTTP defaults to port 4318; treat anything else as gRPC.
fn protocol_for(endpoint: &str) -> &'static str {
    if endpoint.contains(":4318") {
        OTLP_HTTP_PROTOCOL
    } else {
        OTLP_GRPC_PROTOCOL
    }
}

/// Pure resolution of the compare-mode precedence (testable without touching the
/// process environment): flag > `PARALLAX_OTLP_FORWARD` > a pre-existing
/// `OTEL_EXPORTER_OTLP_ENDPOINT` (respected, not clobbered) > Parallax default.
fn resolve_forward_from(
    flag: Option<&str>,
    env_forward: Option<String>,
    env_otel: Option<String>,
) -> anyhow::Result<Forward> {
    if let Some(raw) = flag.map(str::to_owned).or(env_forward) {
        let value = raw.trim();
        let endpoint = match value.to_ascii_lowercase().as_str() {
            "off" | "parallax" => {
                return Ok(Forward {
                    endpoint: DEFAULT_PARALLAX_ENDPOINT.to_string(),
                    protocol: OTLP_GRPC_PROTOCOL,
                    compare: false,
                });
            }
            "rotel" | "1" | "true" | "on" => DEFAULT_ROTEL_ENDPOINT.to_string(),
            _ if value.starts_with("http://") || value.starts_with("https://") => value.to_string(),
            other => {
                anyhow::bail!("invalid --otlp-forward '{other}' (use a URL, 'rotel', or 'off')")
            }
        };
        let protocol = protocol_for(&endpoint);
        return Ok(Forward {
            endpoint,
            protocol,
            compare: true,
        });
    }
    // No explicit forward: respect a pre-existing OTEL endpoint if the user set
    // one in the environment (the idiomatic OTel escape hatch); else default.
    if let Some(existing) = env_otel.filter(|v| !v.is_empty()) {
        let protocol = protocol_for(&existing);
        let compare = existing != DEFAULT_PARALLAX_ENDPOINT;
        return Ok(Forward {
            endpoint: existing,
            protocol,
            compare,
        });
    }
    Ok(Forward {
        endpoint: DEFAULT_PARALLAX_ENDPOINT.to_string(),
        protocol: OTLP_GRPC_PROTOCOL,
        compare: false,
    })
}

fn resolve_forward(flag: Option<&str>) -> anyhow::Result<Forward> {
    let env_forward = std::env::var("PARALLAX_OTLP_FORWARD")
        .ok()
        .filter(|v| !v.is_empty());
    let env_otel = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();
    resolve_forward_from(flag, env_forward, env_otel)
}

/// Resource attributes injected into the child. `parallax.run.id` always; in
/// compare mode also `parallax.lab=1` + `deployment.environment.name` so the same
/// run is findable across every backend's UI.
fn forward_resource_attrs(run_id: &str, compare: bool) -> String {
    let mut attrs = format!("parallax.run.id={run_id}");
    if compare {
        let env = std::env::var("PARALLAX_ENV")
            .ok()
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "lab".to_string());
        attrs.push_str(&format!(
            ",parallax.lab=1,deployment.environment.name={env}"
        ));
    }
    attrs
}

/// The full standard OTel env block (all signals + protocols + resource attrs),
/// pointed at `endpoint`. Used identically for wrapper, bare, and dry-run modes.
fn otel_env_pairs(endpoint: &str, protocol: &str, attrs: &str) -> Vec<(&'static str, String)> {
    vec![
        ("OTEL_EXPORTER_OTLP_ENDPOINT", endpoint.to_string()),
        ("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", endpoint.to_string()),
        ("OTEL_EXPORTER_OTLP_LOGS_ENDPOINT", endpoint.to_string()),
        ("OTEL_EXPORTER_OTLP_METRICS_ENDPOINT", endpoint.to_string()),
        ("OTEL_EXPORTER_OTLP_PROFILES_ENDPOINT", endpoint.to_string()),
        ("OTEL_EXPORTER_OTLP_PROTOCOL", protocol.to_string()),
        ("OTEL_EXPORTER_OTLP_TRACES_PROTOCOL", protocol.to_string()),
        ("OTEL_EXPORTER_OTLP_LOGS_PROTOCOL", protocol.to_string()),
        ("OTEL_EXPORTER_OTLP_METRICS_PROTOCOL", protocol.to_string()),
        ("OTEL_EXPORTER_OTLP_PROFILES_PROTOCOL", protocol.to_string()),
        ("OTEL_RESOURCE_ATTRIBUTES", attrs.to_string()),
    ]
}

/// Best-effort reachability check for compare mode: warn (never fail) if the
/// collector isn't accepting connections — a dead Rotel means nothing shows in
/// any backend, including Parallax.
async fn preflight_warn(endpoint: &str) {
    let host_port = endpoint
        .split("://")
        .nth(1)
        .and_then(|rest| rest.split('/').next())
        .unwrap_or(endpoint);
    let connect = tokio::net::TcpStream::connect(host_port.to_string());
    let reachable = matches!(
        tokio::time::timeout(std::time::Duration::from_millis(500), connect).await,
        Ok(Ok(_))
    );
    if !reachable {
        eprintln!("⚠ {endpoint} not reachable — telemetry may be dropped");
    }
}

fn now_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn new_run_id() -> String {
    // Time-based id is enough for a single-user local tool.
    format!("{:x}", now_nanos())
}

fn relative(nanos_str: &str) -> String {
    let nanos: u128 = nanos_str.parse().unwrap_or(0);
    let now = now_nanos();
    let secs = now.saturating_sub(nanos) / 1_000_000_000;
    match secs {
        0..=59 => format!("{secs}s ago"),
        60..=3599 => format!("{}m ago", secs / 60),
        3600..=86_399 => format!("{}h ago", secs / 3600),
        _ => format!("{}d ago", secs / 86_400),
    }
}

/// `parallax run start [--otlp-forward <target>] [--print-env] [-- <command…>]`
///
/// Default: child telemetry → Parallax's own receiver. Compare mode (forward set
/// via flag or `PARALLAX_OTLP_FORWARD`): child telemetry → the collector (Rotel),
/// which fans it out to every backend incl. Parallax for side-by-side comparison.
pub async fn run_start(
    client: &Client,
    command: Vec<String>,
    forward: Option<String>,
    print_env: bool,
) -> anyhow::Result<i32> {
    let run_id = new_run_id();
    let fwd = resolve_forward(forward.as_deref())?;
    let attrs = forward_resource_attrs(&run_id, fwd.compare);
    let pairs = otel_env_pairs(&fwd.endpoint, fwd.protocol, &attrs);

    // Dry-run: print the env we *would* inject, run nothing, record nothing.
    if print_env && !command.is_empty() {
        for (key, value) in &pairs {
            println!("export {key}={value}");
        }
        return Ok(0);
    }

    let command_str = (!command.is_empty()).then(|| command.join(" "));
    client
        .graphql(&format!(
            r#"mutation {{ runStart(runId: "{}", command: {}, startedAtNanos: "{}") }}"#,
            gql_str(&run_id),
            command_str
                .as_deref()
                .map(|c| format!("\"{}\"", gql_str(c)))
                .unwrap_or_else(|| "null".to_string()),
            now_nanos()
        ))
        .await?;

    if command.is_empty() {
        // Bare mode: print exports for the developer to source.
        for (key, value) in &pairs {
            println!("export {key}={value}");
        }
        println!("# run id: {run_id}  (finish with: parallax run finish {run_id} <exit-code>)");
        return Ok(0);
    }

    // Wrapper mode: inject env, run the child, capture the exit code.
    println!("Parallax run id: {run_id}");
    println!("command: {}", command.join(" "));
    if fwd.compare {
        println!(
            "telemetry → Rotel (fan-out) {}   [COMPARE MODE]",
            fwd.endpoint
        );
        println!("   ↳ parallax · maple · signoz · openobserve · sentry");
        preflight_warn(&fwd.endpoint).await;
    } else {
        println!("telemetry → Parallax {}", fwd.endpoint);
    }
    println!("live: parallax run watch {run_id}");
    let mut cmd = tokio::process::Command::new(&command[0]);
    cmd.args(&command[1..]);
    for (key, value) in &pairs {
        cmd.env(key, value);
    }
    let status = cmd.status().await?;
    let exit_code = status.code().unwrap_or(-1);

    client
        .graphql(&format!(
            r#"mutation {{ runFinish(runId: "{}", endedAtNanos: "{}", exitCode: {exit_code}) }}"#,
            gql_str(&run_id),
            now_nanos()
        ))
        .await?;
    println!("Parallax run {run_id} finished with exit code {exit_code}");
    println!("inspect: parallax run inspect {run_id}   issues: parallax issue list");
    Ok(exit_code)
}

pub async fn run_finish(client: &Client, run_id: &str, exit_code: i32) -> anyhow::Result<()> {
    client
        .graphql(&format!(
            r#"mutation {{ runFinish(runId: "{}", endedAtNanos: "{}", exitCode: {exit_code}) }}"#,
            gql_str(run_id),
            now_nanos()
        ))
        .await?;
    println!("run {run_id} finished ({exit_code})");
    Ok(())
}

pub async fn run_list(client: &Client) -> anyhow::Result<()> {
    let response = client
        .graphql(r#"{ runs { runId command status exitCode startedAtNanos } }"#)
        .await?;
    let runs = response
        .pointer("/data/runs")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if runs.is_empty() {
        println!("no runs yet — start one with: parallax run start -- <command>");
        return Ok(());
    }
    println!(
        "{:<24} {:<10} {:>5}  {:<10} command",
        "RUN", "STATUS", "EXIT", "STARTED"
    );
    for run in runs {
        println!(
            "{:<24} {:<10} {:>5}  {:<10} {}",
            run["runId"].as_str().unwrap_or("-"),
            run["status"].as_str().unwrap_or("-"),
            run["exitCode"]
                .as_i64()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "-".into()),
            relative(run["startedAtNanos"].as_str().unwrap_or("0")),
            run["command"].as_str().unwrap_or("-"),
        );
    }
    Ok(())
}

/// `parallax run inspect <run_id>` — the run's record plus its derived
/// counts and grouped issues.
pub async fn run_inspect(client: &Client, run_id: &str) -> anyhow::Result<()> {
    let response = client
        .graphql(&format!(
            r#"{{ run(runId: "{}") {{ runId command status exitCode startedAtNanos endedAtNanos
                 errorCount traceCount issues {{ fingerprint title }} }} }}"#,
            gql_str(run_id)
        ))
        .await?;
    let Some(run) = response.pointer("/data/run").filter(|v| !v.is_null()) else {
        anyhow::bail!("run {run_id} not found");
    };
    println!("run {run_id}");
    println!("  status:  {}", run["status"].as_str().unwrap_or("-"));
    println!("  command: {}", run["command"].as_str().unwrap_or("-"));
    println!(
        "  started: {}",
        relative(run["startedAtNanos"].as_str().unwrap_or("0"))
    );
    if let Some(code) = run["exitCode"].as_i64() {
        println!("  exit:    {code}");
    }
    println!("  traces:  {}", run["traceCount"].as_i64().unwrap_or(0));
    println!("  errors:  {}", run["errorCount"].as_i64().unwrap_or(0));
    if let Some(issues) = run["issues"].as_array()
        && !issues.is_empty()
    {
        println!("issues in this run:");
        for issue in issues {
            println!(
                "  {}  {}",
                issue["fingerprint"].as_str().unwrap_or("-"),
                issue["title"].as_str().unwrap_or("-"),
            );
        }
        println!("context: parallax issue context <fingerprint>");
    }
    println!("bundle: parallax run bundle {run_id}   traces: parallax trace inspect <trace_id>");
    Ok(())
}

/// `parallax run bundle <run_id>` — the run-anchored evidence bundle
/// (scope §2.4: the run model's bundle).
pub async fn run_bundle(client: &Client, run_id: &str) -> anyhow::Result<()> {
    let response = client
        .graphql(&format!(
            r#"{{ bundle(runId: "{}") {{ markdown canonicalHash }} }}"#,
            gql_str(run_id)
        ))
        .await?;
    let Some(bundle) = response.pointer("/data/bundle").filter(|v| !v.is_null()) else {
        anyhow::bail!("run {run_id} not found");
    };
    println!("{}", bundle["markdown"].as_str().unwrap_or(""));
    if let Some(hash) = bundle["canonicalHash"].as_str() {
        println!("\n---\nbundle: {hash}");
    }
    Ok(())
}

pub async fn issue_list(
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
pub async fn issue_context(client: &Client, fingerprint: &str) -> anyhow::Result<()> {
    let response = client
        .graphql(&format!(
            r#"{{ bundle(fingerprint: "{}") {{ markdown canonicalHash }} }}"#,
            gql_str(fingerprint)
        ))
        .await?;
    let Some(bundle) = response.pointer("/data/bundle").filter(|v| !v.is_null()) else {
        anyhow::bail!("issue {fingerprint} not found");
    };
    println!("{}", bundle["markdown"].as_str().unwrap_or(""));
    if let Some(hash) = bundle["canonicalHash"].as_str() {
        println!("\n---\nbundle: {hash}");
    }
    Ok(())
}

/// The CLI mirror of the UI Logs page filters — agents compose the same
/// scoping (trace/run/service/level/text/window) in one command.
pub struct LogsFilter<'a> {
    pub trace: Option<&'a str>,
    pub run: Option<&'a str>,
    pub service: Option<&'a str>,
    pub level: Option<&'a str>,
    pub grep: Option<&'a str>,
    pub since: &'a str,
    pub limit: u32,
}

fn severity_min(level: &str) -> anyhow::Result<i32> {
    // OTel severity number floors per level.
    Ok(match level.to_ascii_lowercase().as_str() {
        "trace" => 1,
        "debug" => 5,
        "info" => 9,
        "warn" | "warning" => 13,
        "error" => 17,
        "fatal" => 21,
        other => anyhow::bail!("unknown level '{other}' (trace|debug|info|warn|error|fatal)"),
    })
}

fn parse_since(since: &str) -> anyhow::Result<u128> {
    let (digits, unit) = since.split_at(since.len().saturating_sub(1));
    let n: u128 = digits
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid --since '{since}' (e.g. 15m, 2h, 7d)"))?;
    let seconds = match unit {
        "s" => n,
        "m" => n * 60,
        "h" => n * 3600,
        "d" => n * 86_400,
        _ => anyhow::bail!("invalid --since unit '{unit}' (s|m|h|d)"),
    };
    Ok(seconds * 1_000_000_000)
}

/// `parallax sql "<SELECT …>"` — the engine's raw query power for agents
/// and ad-hoc digging; read-only, same guard as the API.
pub async fn sql(client: &Client, query: &str) -> anyhow::Result<()> {
    let response = client
        .graphql(&format!(
            r#"{{ sql(query: "{}") {{ columns rows rowCount }} }}"#,
            gql_str(query)
        ))
        .await?;
    let Some(result) = response.pointer("/data/sql").filter(|v| !v.is_null()) else {
        anyhow::bail!("sql query failed");
    };
    let columns: Vec<String> = result["columns"]
        .as_array()
        .map(|cols| {
            cols.iter()
                .filter_map(|c| c.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    println!("{}", columns.join("\t"));
    for row in result["rows"].as_array().into_iter().flatten() {
        let cells: Vec<String> = row
            .as_str()
            .and_then(|s| serde_json::from_str::<Vec<serde_json::Value>>(s).ok())
            .map(|values| {
                values
                    .iter()
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        println!("{}", cells.join("\t"));
    }
    println!(
        "-- {} row(s)",
        result["rowCount"].as_i64().unwrap_or_default()
    );
    Ok(())
}

/// `parallax logs [--trace|--run] [--service] [--level] [--grep] [--since] [--limit]`.
pub async fn logs(client: &Client, filter: LogsFilter<'_>) -> anyhow::Result<()> {
    let mut args: Vec<String> = Vec::new();
    if let Some(trace_id) = filter.trace {
        args.push(format!(r#"traceId: "{}""#, gql_str(trace_id)));
    }
    if let Some(run_id) = filter.run {
        args.push(format!(r#"runId: "{}""#, gql_str(run_id)));
    }
    if let Some(service) = filter.service {
        args.push(format!(r#"service: "{}""#, gql_str(service)));
    }
    if let Some(level) = filter.level {
        args.push(format!("severityMin: {}", severity_min(level)?));
    }
    if let Some(needle) = filter.grep {
        args.push(format!(r#"query: "{}""#, gql_str(needle)));
    }
    if filter.trace.is_none() && filter.run.is_none() {
        let from = now_nanos().saturating_sub(parse_since(filter.since)?);
        args.push(format!(r#"fromNanos: "{from}""#));
    }
    args.push(format!("limit: {}", filter.limit));
    let response = client
        .graphql(&format!(
            r#"{{ logs({}) {{ tsNanos service severityText body }} }}"#,
            args.join(", ")
        ))
        .await?;
    let logs = response
        .pointer("/data/logs")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if logs.is_empty() {
        println!("no matching logs");
        return Ok(());
    }
    for log in &logs {
        println!(
            "{:<10} [{}] {} {}",
            relative(log["tsNanos"].as_str().unwrap_or("0")),
            log["service"].as_str().unwrap_or("-"),
            log["severityText"].as_str().unwrap_or("-"),
            log["body"].as_str().unwrap_or(""),
        );
    }
    Ok(())
}

/// The CLI mirror of the UI Traces page filters.
pub struct TracesFilter<'a> {
    pub service: Option<&'a str>,
    pub run: Option<&'a str>,
    pub min_duration: Option<&'a str>,
    pub errors_only: bool,
    pub grep: Option<&'a str>,
    pub since: &'a str,
    pub limit: u32,
}

/// "500ms" | "2s" | "1m" | bare millis ("250") → milliseconds.
fn parse_duration_ms(value: &str) -> anyhow::Result<f64> {
    let parse = |digits: &str, scale: f64| -> anyhow::Result<f64> {
        digits
            .parse::<f64>()
            .map(|n| n * scale)
            .map_err(|_| anyhow::anyhow!("invalid duration '{value}' (e.g. 500ms, 2s, 1m)"))
    };
    if let Some(digits) = value.strip_suffix("ms") {
        parse(digits, 1.0)
    } else if let Some(digits) = value.strip_suffix('s') {
        parse(digits, 1_000.0)
    } else if let Some(digits) = value.strip_suffix('m') {
        parse(digits, 60_000.0)
    } else {
        parse(value, 1.0)
    }
}

/// `parallax traces [--run] [--service] [--min-duration] [--errors] [--grep] [--since] [--limit]`.
pub async fn traces(client: &Client, filter: TracesFilter<'_>) -> anyhow::Result<()> {
    // --run anchors on the run's traces (tracesByRun); other filters are
    // the browse query.
    let (pointer, query) = match filter.run {
        Some(run_id) => (
            "/data/tracesByRun",
            format!(
                r#"{{ tracesByRun(runId: "{}", limit: {}) {{ traceId rootName service startNanos durationNs spanCount hasError }} }}"#,
                gql_str(run_id),
                filter.limit
            ),
        ),
        None => {
            let mut args: Vec<String> = Vec::new();
            if let Some(service) = filter.service {
                args.push(format!(r#"service: "{}""#, gql_str(service)));
            }
            if let Some(min) = filter.min_duration {
                args.push(format!("minDurationMs: {}", parse_duration_ms(min)?));
            }
            if filter.errors_only {
                args.push("errorOnly: true".into());
            }
            if let Some(needle) = filter.grep {
                args.push(format!(r#"query: "{}""#, gql_str(needle)));
            }
            let from = now_nanos().saturating_sub(parse_since(filter.since)?);
            args.push(format!(r#"fromNanos: "{from}""#));
            args.push(format!("limit: {}", filter.limit));
            (
                "/data/traces",
                format!(
                    r#"{{ traces({}) {{ traceId rootName service startNanos durationNs spanCount hasError }} }}"#,
                    args.join(", ")
                ),
            )
        }
    };
    let response = client.graphql(&query).await?;
    let traces = response
        .pointer(pointer)
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if traces.is_empty() {
        println!("no matching traces");
        return Ok(());
    }
    for trace in &traces {
        let millis = trace["durationNs"]
            .as_str()
            .and_then(|d| d.parse::<u128>().ok())
            .unwrap_or(0) as f64
            / 1e6;
        println!(
            "{:<10} {} [{}] {} — {} span(s), {millis:.1}ms{}",
            relative(trace["startNanos"].as_str().unwrap_or("0")),
            trace["traceId"].as_str().unwrap_or("-"),
            trace["service"].as_str().unwrap_or("-"),
            trace["rootName"].as_str().unwrap_or("-"),
            trace["spanCount"].as_i64().unwrap_or(0),
            if trace["hasError"].as_bool().unwrap_or(false) {
                ", ERROR"
            } else {
                ""
            },
        );
    }
    Ok(())
}

/// Tail an SSE endpoint, printing each row via `print`; `for_window`
/// (e.g. 30s) stops after that long and reports the match count — the
/// agent-verification mode ("watch whether it still appears").
async fn tail_sse(
    client: &Client,
    path_and_query: &str,
    for_window: Option<&str>,
    label: &str,
    print: impl Fn(&serde_json::Value),
) -> anyhow::Result<()> {
    use tokio_stream::StreamExt as _;
    let deadline = for_window
        .map(|w| parse_since(w).map(|nanos| (nanos / 1_000_000) as u64))
        .transpose()?
        .map(|millis| tokio::time::Instant::now() + std::time::Duration::from_millis(millis));
    let response = client.sse(path_and_query).await?;
    let mut stream = response.bytes_stream();
    let mut pending = String::new();
    let mut matched: u64 = 0;
    loop {
        let chunk = match deadline {
            Some(deadline) => {
                match tokio::time::timeout_at(deadline, stream.next()).await {
                    Ok(chunk) => chunk,
                    Err(_) => break, // window elapsed
                }
            }
            None => stream.next().await,
        };
        let Some(chunk) = chunk else { break };
        pending.push_str(&String::from_utf8_lossy(&chunk?));
        // SSE frames: "data: <json>\n"; keep-alives and partial lines skipped.
        while let Some(newline) = pending.find('\n') {
            let line = pending[..newline].trim().to_string();
            pending.drain(..=newline);
            let Some(payload) = line.strip_prefix("data: ") else {
                continue;
            };
            if let Ok(serde_json::Value::Array(rows)) = serde_json::from_str(payload) {
                for row in &rows {
                    matched += 1;
                    print(row);
                }
            }
        }
    }
    if let Some(window) = for_window {
        println!("-- watched {window}: {matched} matching {label}(s)");
    }
    Ok(())
}

/// `parallax logs --follow` — kubectl-style live tail over SSE.
pub async fn logs_follow(
    client: &Client,
    filter: LogsFilter<'_>,
    for_window: Option<&str>,
) -> anyhow::Result<()> {
    let mut params: Vec<(&str, String)> = Vec::new();
    if let Some(service) = filter.service {
        params.push(("service", service.into()));
    }
    if let Some(level) = filter.level {
        params.push(("severity_min", severity_min(level)?.to_string()));
    }
    if let Some(needle) = filter.grep {
        params.push(("q", needle.into()));
    }
    if let Some(trace_id) = filter.trace {
        params.push(("trace_id", trace_id.into()));
    }
    if let Some(run_id) = filter.run {
        params.push(("run_id", run_id.into()));
    }
    let query = encode_query(&params);
    tail_sse(
        client,
        &format!("/v1/logs/stream{query}"),
        for_window,
        "log event",
        |log| {
            println!(
                "{:<10} [{}] {} {}",
                relative(log["tsNanos"].as_str().unwrap_or("0")),
                log["service"].as_str().unwrap_or("-"),
                log["severityText"].as_str().unwrap_or("-"),
                log["body"].as_str().unwrap_or(""),
            );
        },
    )
    .await
}

/// `parallax traces --follow` — live finished-span feed over SSE.
pub async fn traces_follow(
    client: &Client,
    filter: TracesFilter<'_>,
    for_window: Option<&str>,
) -> anyhow::Result<()> {
    let mut params: Vec<(&str, String)> = Vec::new();
    if let Some(service) = filter.service {
        params.push(("service", service.into()));
    }
    if let Some(min) = filter.min_duration {
        params.push(("min_duration_ms", parse_duration_ms(min)?.to_string()));
    }
    if filter.errors_only {
        params.push(("errors_only", "true".into()));
    }
    if let Some(needle) = filter.grep {
        params.push(("q", needle.into()));
    }
    if let Some(run_id) = filter.run {
        params.push(("run_id", run_id.into()));
    }
    let query = encode_query(&params);
    tail_sse(
        client,
        &format!("/v1/traces/stream{query}"),
        for_window,
        "span",
        print_span_line,
    )
    .await
}

fn print_span_line(span: &serde_json::Value) {
    let millis = span["durationNs"]
        .as_str()
        .and_then(|d| d.parse::<u128>().ok())
        .unwrap_or(0) as f64
        / 1e6;
    println!(
        "{:<10} {} [{}] {} — {millis:.1}ms {}",
        relative(span["tsNanos"].as_str().unwrap_or("0")),
        span["traceId"].as_str().unwrap_or("-"),
        span["service"].as_str().unwrap_or("-"),
        span["name"].as_str().unwrap_or("-"),
        span["statusCode"]
            .as_str()
            .map(|s| s.trim_start_matches("STATUS_CODE_"))
            .unwrap_or("-"),
    );
}

/// `parallax run watch <run_id>` — the run-scoped combined live tail: new
/// log records and finished spans for one run id, interleaved as they
/// arrive (the CLI mirror of the run page's Live mode). `--for 30s` watches
/// a fixed window and reports per-stream match counts — the agent
/// verification loop for a specific run.
pub async fn run_watch(
    client: &Client,
    run_id: &str,
    level: Option<&str>,
    grep: Option<&str>,
    for_window: Option<&str>,
) -> anyhow::Result<()> {
    println!(
        "watching run {run_id} — live logs + spans{}",
        for_window
            .map(|w| format!(" for {w}"))
            .unwrap_or_else(|| " (Ctrl-C to stop)".into())
    );
    let mut log_params: Vec<(&str, String)> = vec![("run_id", run_id.into())];
    if let Some(level) = level {
        log_params.push(("severity_min", severity_min(level)?.to_string()));
    }
    if let Some(needle) = grep {
        log_params.push(("q", needle.into()));
    }
    let span_params: Vec<(&str, String)> = vec![("run_id", run_id.into())];
    let logs_path = format!("/v1/logs/stream{}", encode_query(&log_params));
    let spans_path = format!("/v1/traces/stream{}", encode_query(&span_params));
    let logs = tail_sse(client, &logs_path, for_window, "log event", |log| {
        println!(
            "[log]  {:<10} {} {}",
            relative(log["tsNanos"].as_str().unwrap_or("0")),
            log["severityText"].as_str().unwrap_or("-"),
            log["body"].as_str().unwrap_or(""),
        );
    });
    let spans = tail_sse(client, &spans_path, for_window, "span", |span| {
        print!("[span] ");
        print_span_line(span);
    });
    let (logs, spans) = tokio::join!(logs, spans);
    logs?;
    spans?;
    Ok(())
}

fn encode_query(params: &[(&str, String)]) -> String {
    if params.is_empty() {
        return String::new();
    }
    let encoded: Vec<String> = params
        .iter()
        .map(|(key, value)| format!("{key}={}", urlencoding::encode(value)))
        .collect();
    format!("?{}", encoded.join("&"))
}

pub async fn trace_inspect(client: &Client, trace_id: &str) -> anyhow::Result<()> {
    let response = client
        .graphql(&format!(
            r#"{{ trace(traceId: "{0}") {{ spans {{ name service kind statusCode durationNs spanId parentSpanId }} }}
                 logsByTrace(traceId: "{0}") {{ severityText body }} }}"#,
            gql_str(trace_id)
        ))
        .await?;
    let Some(spans) = response
        .pointer("/data/trace/spans")
        .and_then(|v| v.as_array())
    else {
        anyhow::bail!("trace {trace_id} not found");
    };
    println!("trace {trace_id} — {} span(s)", spans.len());
    for span in spans {
        let micros = span["durationNs"]
            .as_str()
            .and_then(|d| d.parse::<u128>().ok())
            .unwrap_or(0)
            / 1_000;
        println!(
            "  [{}] {} — {} {} ({micros}µs)",
            span["service"].as_str().unwrap_or("-"),
            span["name"].as_str().unwrap_or("-"),
            span["kind"].as_str().unwrap_or("-"),
            span["statusCode"].as_str().unwrap_or("-"),
        );
    }
    if let Some(logs) = response
        .pointer("/data/logsByTrace")
        .and_then(|v| v.as_array())
        && !logs.is_empty()
    {
        println!("logs:");
        for log in logs {
            println!(
                "  {} {}",
                log["severityText"].as_str().unwrap_or("-"),
                log["body"].as_str().unwrap_or(""),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_follows_port() {
        assert_eq!(protocol_for("http://localhost:4317"), OTLP_GRPC_PROTOCOL);
        assert_eq!(protocol_for("http://localhost:4318"), OTLP_HTTP_PROTOCOL);
        assert_eq!(
            protocol_for("http://host.docker.internal:14317"),
            OTLP_GRPC_PROTOCOL
        );
    }

    #[test]
    fn flag_beats_env() {
        let fwd =
            resolve_forward_from(Some("http://localhost:4317"), Some("off".to_string()), None)
                .unwrap();
        assert_eq!(fwd.endpoint, "http://localhost:4317");
        assert!(fwd.compare);
    }

    #[test]
    fn flag_off_forces_default() {
        let fwd = resolve_forward_from(Some("off"), Some("rotel".to_string()), None).unwrap();
        assert_eq!(fwd.endpoint, DEFAULT_PARALLAX_ENDPOINT);
        assert!(!fwd.compare);
    }

    #[test]
    fn rotel_alias_resolves() {
        let fwd = resolve_forward_from(None, Some("rotel".to_string()), None).unwrap();
        assert_eq!(fwd.endpoint, DEFAULT_ROTEL_ENDPOINT);
        assert!(fwd.compare);
    }

    #[test]
    fn explicit_url_from_env() {
        let fwd =
            resolve_forward_from(None, Some("http://collector:4318".to_string()), None).unwrap();
        assert_eq!(fwd.endpoint, "http://collector:4318");
        assert_eq!(fwd.protocol, OTLP_HTTP_PROTOCOL);
        assert!(fwd.compare);
    }

    #[test]
    fn respects_preexisting_otel_endpoint() {
        let fwd =
            resolve_forward_from(None, None, Some("http://localhost:4317".to_string())).unwrap();
        assert_eq!(fwd.endpoint, "http://localhost:4317");
        assert!(fwd.compare);
    }

    #[test]
    fn default_when_nothing_set() {
        let fwd = resolve_forward_from(None, None, None).unwrap();
        assert_eq!(fwd.endpoint, DEFAULT_PARALLAX_ENDPOINT);
        assert!(!fwd.compare);
    }

    #[test]
    fn preexisting_parallax_endpoint_is_not_compare() {
        let fwd =
            resolve_forward_from(None, None, Some(DEFAULT_PARALLAX_ENDPOINT.to_string())).unwrap();
        assert!(!fwd.compare);
    }

    #[test]
    fn invalid_target_errors() {
        assert!(resolve_forward_from(Some("nonsense"), None, None).is_err());
    }

    #[test]
    fn compare_adds_lab_attrs() {
        let attrs = forward_resource_attrs("abc123", true);
        assert!(attrs.contains("parallax.run.id=abc123"));
        assert!(attrs.contains("parallax.lab=1"));
        assert!(attrs.contains("deployment.environment.name="));
    }

    #[test]
    fn default_mode_run_id_only() {
        let attrs = forward_resource_attrs("abc123", false);
        assert_eq!(attrs, "parallax.run.id=abc123");
    }
}
