//! Secret-free per-call audit rows for the product MCP spike (plan 112).
//!
//! Deliberately does **not** install a tracing subscriber (dependency MCP
//! protocol logging of anchors/evidence stays impossible). Audit rows are
//! in-process, structured, and free of anchors, evidence bodies, tokens, and
//! raw GraphQL/CLI diagnostics.

use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// Stable tool names the catalog may emit. Unknown tools must not be audited
/// as success — only the closed two-tool catalog produces rows in product mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuditTool {
    IssueContext,
    AgentSessionShow,
}

impl AuditTool {
    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::IssueContext => "parallax_issue_context",
            Self::AgentSessionShow => "parallax_agent_session_show",
        }
    }
}

/// One finished tool invocation. Fields are intentionally coarse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AuditRow {
    pub tool: &'static str,
    pub principal: &'static str,
    pub scopes: Vec<&'static str>,
    /// `"ok"` or a stable error code (`invalid_anchor`, `bundle_unavailable`, …).
    pub status: String,
    pub result_bytes: usize,
    pub duration_ms: u64,
}

static AUDIT_LOG: OnceLock<Mutex<Vec<AuditRow>>> = OnceLock::new();

fn audit_log() -> &'static Mutex<Vec<AuditRow>> {
    AUDIT_LOG.get_or_init(|| Mutex::new(Vec::new()))
}

/// Append a secret-free audit row. Never takes evidence or anchor content.
pub(crate) fn record(row: AuditRow) {
    if let Ok(mut guard) = audit_log().lock() {
        // Bound in-process retention so a long-lived stdio session cannot grow
        // without limit. Oldest rows drop first.
        const MAX_ROWS: usize = 1_024;
        if guard.len() >= MAX_ROWS {
            let drop_n = guard.len() - MAX_ROWS + 1;
            guard.drain(0..drop_n);
        }
        guard.push(row);
    }
}

/// Snapshot of recorded rows (tests / doctor only).
#[must_use]
#[cfg(test)]
pub(crate) fn snapshot() -> Vec<AuditRow> {
    audit_log().lock().map(|g| g.clone()).unwrap_or_default()
}

/// Clear the in-process log (tests).
#[cfg(test)]
pub(crate) fn clear() {
    if let Ok(mut guard) = audit_log().lock() {
        guard.clear();
    }
}

/// Measures a tool call and records one audit row without capturing inputs.
pub(crate) struct ToolCallGuard {
    tool: AuditTool,
    principal: &'static str,
    scopes: Vec<&'static str>,
    started: Instant,
    span: tracing::Span,
    completed: bool,
}

impl ToolCallGuard {
    #[must_use]
    pub(crate) fn start(tool: AuditTool, principal: &'static str, scopes: &[&'static str]) -> Self {
        let scope_names = scopes.join(",");
        Self {
            tool,
            principal,
            scopes: scopes.to_vec(),
            started: Instant::now(),
            span: tracing::info_span!(
                target: "parallax.mcp.audit",
                "mcp.tool_call",
                tool = tool.as_str(),
                principal,
                scopes = scope_names,
                status = tracing::field::Empty,
                result_bytes = tracing::field::Empty,
                duration_ms = tracing::field::Empty,
            ),
            completed: false,
        }
    }

    pub(crate) fn finish_ok(mut self, result_bytes: usize) {
        self.finish("ok", result_bytes);
    }

    pub(crate) fn finish_err(mut self, code: &str) {
        self.finish(code, 0);
    }

    fn finish(&mut self, status: &str, result_bytes: usize) {
        let duration_ms = u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX);
        self.span.record("status", status);
        self.span.record("result_bytes", result_bytes);
        self.span.record("duration_ms", duration_ms);
        record(AuditRow {
            tool: self.tool.as_str(),
            principal: self.principal,
            scopes: std::mem::take(&mut self.scopes),
            status: status.to_string(),
            result_bytes,
            duration_ms,
        });
        self.completed = true;
    }
}

impl Drop for ToolCallGuard {
    fn drop(&mut self) {
        if !self.completed {
            self.finish("cancelled", 0);
        }
    }
}

/// Extract a stable error code from MCP error data for the audit row.
#[must_use]
pub(crate) fn error_code(error: &rmcp::ErrorData) -> String {
    error
        .data
        .as_ref()
        .and_then(|value| value.get("code"))
        .and_then(|code| code.as_str())
        .unwrap_or("mcp_error")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;
    use std::sync::{Arc, Mutex};
    use tracing::field::{Field, Visit};
    use tracing::span::{Attributes, Record};
    use tracing::{Id, Subscriber};
    use tracing_subscriber::layer::{Context, SubscriberExt};
    use tracing_subscriber::registry::LookupSpan;
    use tracing_subscriber::{Layer, Registry};

    /// Global audit log is process-wide; serialize tests that assert exact length.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[derive(Clone, Default)]
    struct CaptureLayer(Arc<Mutex<Vec<String>>>);

    struct FieldVisitor<'a>(&'a Mutex<Vec<String>>);

    impl Visit for FieldVisitor<'_> {
        fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
            self.0
                .lock()
                .expect("span capture lock")
                .push(format!("{}={value:?}", field.name()));
        }
    }

    impl<S> Layer<S> for CaptureLayer
    where
        S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    {
        fn on_new_span(&self, attrs: &Attributes<'_>, _id: &Id, _ctx: Context<'_, S>) {
            assert_eq!(attrs.metadata().name(), "mcp.tool_call");
            attrs.record(&mut FieldVisitor(&self.0));
        }

        fn on_record(&self, _span: &Id, values: &Record<'_>, _ctx: Context<'_, S>) {
            values.record(&mut FieldVisitor(&self.0));
        }
    }

    fn captured_span(run: impl FnOnce()) -> String {
        let capture = CaptureLayer::default();
        let fields = Arc::clone(&capture.0);
        let subscriber = Registry::default().with(capture);
        tracing::subscriber::with_default(subscriber, run);
        fields.lock().expect("span capture lock").join("\n")
    }

    #[test]
    fn audit_rows_never_embed_anchors_or_evidence() {
        let _lock = TEST_LOCK.lock().expect("audit test lock");
        clear();
        let fields = captured_span(|| {
            let guard = ToolCallGuard::start(
                AuditTool::IssueContext,
                "local-operator",
                &["evidence:read"],
            );
            // Deliberately pass a secret-shaped byte count only — no fingerprint.
            guard.finish_ok(42);
        });
        let rows = snapshot();
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(row.tool, "parallax_issue_context");
        assert_eq!(row.principal, "local-operator");
        assert_eq!(row.scopes, vec!["evidence:read"]);
        assert_eq!(row.status, "ok");
        assert_eq!(row.result_bytes, 42);
        let rendered = format!("{row:?}");
        assert!(
            !rendered.contains("ghp_")
                && !rendered.contains("Bearer ")
                && !rendered.contains("sk_live"),
            "audit debug form leaked secret-shaped text: {rendered}"
        );
        assert!(fields.contains("tool=\"parallax_issue_context\""));
        assert!(fields.contains("principal=\"local-operator\""));
        assert!(fields.contains("scopes=\"evidence:read\""));
        assert!(fields.contains("status=\"ok\""));
        assert!(fields.contains("result_bytes=42"));
        assert!(fields.contains("duration_ms="));
        assert!(!fields.contains("ghp_") && !fields.contains("sk_live"));
    }

    #[test]
    fn error_path_records_stable_code_without_payload() {
        let _lock = TEST_LOCK.lock().expect("audit test lock");
        clear();
        let fields = captured_span(|| {
            let guard = ToolCallGuard::start(
                AuditTool::AgentSessionShow,
                "local-operator",
                &["evidence:read"],
            );
            guard.finish_err("invalid_anchor");
        });
        let rows = snapshot();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].status, "invalid_anchor");
        assert_eq!(rows[0].result_bytes, 0);
        assert_eq!(rows[0].tool, "parallax_agent_session_show");
        assert!(fields.contains("status=\"invalid_anchor\""));
        assert!(fields.contains("result_bytes=0"));
    }

    #[test]
    fn dropped_guard_records_one_cancelled_call() {
        let _lock = TEST_LOCK.lock().expect("audit test lock");
        clear();
        let fields = captured_span(|| {
            let _guard = ToolCallGuard::start(
                AuditTool::IssueContext,
                "local-operator",
                &["evidence:read"],
            );
        });
        let rows = snapshot();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].status, "cancelled");
        assert_eq!(rows[0].result_bytes, 0);
        assert!(fields.contains("status=\"cancelled\""));
    }
}
