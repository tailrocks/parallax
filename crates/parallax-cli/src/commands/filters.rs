//! Shared logs and traces filter parsing.

/// The CLI mirror of the UI Logs page filters — agents compose the same
/// scoping (trace/run/service/level/text/window) in one command.
pub(crate) struct LogsFilter<'a> {
    pub trace: Option<&'a str>,
    pub invocation: Option<&'a str>,
    pub service: Option<&'a str>,
    pub level: Option<&'a str>,
    pub grep: Option<&'a str>,
    pub since: &'a str,
    pub limit: u32,
}

pub(crate) fn severity_min(level: &str) -> anyhow::Result<i32> {
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

pub(crate) fn parse_since(since: &str) -> anyhow::Result<u128> {
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
/// The CLI mirror of the UI Traces page filters.
pub(crate) struct TracesFilter<'a> {
    pub service: Option<&'a str>,
    pub invocation: Option<&'a str>,
    pub min_duration: Option<&'a str>,
    pub errors_only: bool,
    pub grep: Option<&'a str>,
    pub since: &'a str,
    pub limit: u32,
}

/// "500ms" | "2s" | "1m" | bare millis ("250") → milliseconds.
pub(crate) fn parse_duration_ms(value: &str) -> anyhow::Result<f64> {
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
