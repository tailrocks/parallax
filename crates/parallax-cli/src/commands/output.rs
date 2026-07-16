//! Stable bundle and agent-session output rendering.

use crate::OutputFormat;

/// Pure render decision for bundle commands — markdown is byte-compatible
/// with the pre-`--format` CLI; json emits the canonical string alone
/// (hash lives inside the bundle JSON; no trailer on stdout).
pub(crate) fn render_bundle(format: OutputFormat, bundle: &serde_json::Value) -> (String, String) {
    match format {
        OutputFormat::Markdown => {
            let mut out = String::new();
            out.push_str(bundle["markdown"].as_str().unwrap_or(""));
            out.push('\n');
            if let Some(hash) = bundle["canonicalHash"].as_str() {
                out.push_str("\n---\nbundle: ");
                out.push_str(hash);
                out.push('\n');
            }
            (out, String::new())
        }
        OutputFormat::Json => {
            // Verbatim canonical JSON — do not re-serialize or pretty-print.
            let mut out = String::new();
            out.push_str(bundle["json"].as_str().unwrap_or(""));
            out.push('\n');
            (out, String::new())
        }
    }
}

/// Pure render decision for the agent-session projection.
pub(crate) fn render_agent_session(
    format: OutputFormat,
    invocation_id: &str,
    session: &serde_json::Value,
) -> (String, String) {
    match format {
        OutputFormat::Json => {
            let body = serde_json::to_string(session).unwrap_or_else(|_| "{}".into());
            (format!("{body}\n"), String::new())
        }
        OutputFormat::Markdown => {
            let mut out = String::new();
            out.push_str(&format!("agent session for run {invocation_id}\n"));
            out.push_str(&format!(
                "  root:      {}\n",
                session["rootSpanId"].as_str().unwrap_or("-")
            ));
            out.push_str(&format!(
                "  tokens:    in={} out={}\n",
                session["totalInputTokens"].as_str().unwrap_or("0"),
                session["totalOutputTokens"].as_str().unwrap_or("0"),
            ));
            out.push_str(&format!(
                "  errors:    {}\n",
                session["errorCount"]
                    .as_i64()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| session["errorCount"].to_string())
            ));
            if session["truncated"].as_bool() == Some(true) {
                out.push_str("  truncated: true\n");
            }
            if let Some(steps) = session["steps"].as_array() {
                render_steps(&mut out, steps);
            }
            (out, String::new())
        }
    }
}

fn render_steps(out: &mut String, steps: &[serde_json::Value]) {
    if steps.is_empty() {
        out.push_str("steps: (none)\n");
        return;
    }
    out.push_str("steps:\n");
    for step in steps {
        let error = if step["isError"].as_bool() == Some(true) {
            "  ERR"
        } else {
            ""
        };
        let tokens = match (step["inputTokens"].as_str(), step["outputTokens"].as_str()) {
            (Some(input), Some(output)) => format!("  tokens={input}/{output}"),
            (Some(input), None) => format!("  tokens_in={input}"),
            (None, Some(output)) => format!("  tokens_out={output}"),
            _ => String::new(),
        };
        let operation = step["genAiOperation"]
            .as_str()
            .map(|operation| format!("  op={operation}"))
            .unwrap_or_default();
        out.push_str(&format!(
            "  {:<14} {:<32} dur={}{}{}{}\n",
            step["kind"].as_str().unwrap_or("-"),
            step["name"].as_str().unwrap_or("-"),
            step["durationNs"].as_str().unwrap_or("-"),
            error,
            tokens,
            operation,
        ));
    }
}
