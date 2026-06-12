//! The evidence bundle: bounded, redacted, hypothesis-ranked context for one
//! issue — graduated from `poc/evidence-loop` (bundle/bound/redact/hypothesis
//! kernels) onto the live row model. The same JSON powers the GraphQL
//! `bundle` field, the CLI's `issue context`, and the UI's bundle preview.

use parallax_storage::model::{ErrorEventRow, Issue, LogRow, RunRecord, SpanRow};
use regex::Regex;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub const SCHEMA_VERSION: &str = "bundle-v1";

#[derive(Debug, Serialize)]
pub struct Bundle {
    pub schema_version: &'static str,
    pub generator: &'static str,
    pub anchor: Anchor,
    /// The primary grouped issue — always present for issue anchors; for
    /// run/trace anchors it is the issue behind the newest error event, when
    /// any error occurred at all.
    pub issue: Option<IssueSummary>,
    /// Run context for run anchors (spec §8 `bundle(runId:)`).
    pub run: Option<RunSection>,
    pub latest_event: Option<EventDetail>,
    pub trace: Option<TraceSection>,
    /// Correlated metric slices around the anchor (spec §8: trace + logs +
    /// metric windows together).
    pub metric_windows: Vec<MetricWindow>,
    pub logs: Vec<String>,
    pub hypotheses: Vec<Hypothesis>,
    pub missing_evidence: Vec<String>,
    pub redaction: RedactionReport,
    pub bounded: BoundReport,
    pub canonical_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Anchor {
    pub kind: &'static str,
    /// The anchoring identifier: issue fingerprint, run id, or trace id.
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct RunSection {
    pub run_id: String,
    pub command: Option<String>,
    pub status: String,
    pub exit_code: Option<i32>,
    pub started_at_nanos: String,
    pub ended_at_nanos: Option<String>,
    /// Every grouped issue whose events fell inside this run's traces.
    pub issues: Vec<IssueSummary>,
}

#[derive(Debug, Serialize)]
pub struct IssueSummary {
    pub title: String,
    pub error_type: String,
    pub culprit: Option<String>,
    pub service: String,
    pub status: String,
    pub event_count: u64,
    pub first_seen_nanos: String,
    pub last_seen_nanos: String,
}

#[derive(Debug, Serialize)]
pub struct EventDetail {
    pub ts_nanos: String,
    pub message: String,
    pub stacktrace: Option<String>,
    pub source: String,
    pub trace_id: String,
}

#[derive(Debug, Serialize)]
pub struct TraceSection {
    pub trace_id: String,
    pub spans: Vec<SpanLine>,
}

/// One correlated metric slice around the anchor — the bundle's
/// trace+logs+**metric window** promise (spec §8 correlation sections).
#[derive(Debug, Serialize)]
pub struct MetricWindow {
    pub metric: String,
    /// "run" (points tagged with the anchor's run id) or "service".
    pub scope: &'static str,
    pub from_nanos: String,
    pub to_nanos: String,
    pub step_seconds: u32,
    pub points: Vec<MetricPointLine>,
    pub stats: MetricStats,
}

#[derive(Debug, Serialize)]
pub struct MetricPointLine {
    pub ts_nanos: String,
    pub value: f64,
}

#[derive(Debug, Serialize)]
pub struct MetricStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub last: f64,
}

/// Cap per metric window — keeps the section bounded before token bounding.
pub const METRIC_WINDOW_MAX_POINTS: usize = 60;

impl MetricWindow {
    /// Build a window from raw points (nanos, value), computing stats and
    /// enforcing the point cap (oldest dropped first — the anchor sits at
    /// the window's end).
    pub fn from_points(
        metric: impl Into<String>,
        scope: &'static str,
        from_nanos: u128,
        to_nanos: u128,
        step_seconds: u32,
        mut points: Vec<(u128, f64)>,
    ) -> Option<Self> {
        if points.is_empty() {
            return None;
        }
        points.sort_by_key(|(ts, _)| *ts);
        if points.len() > METRIC_WINDOW_MAX_POINTS {
            points.drain(..points.len() - METRIC_WINDOW_MAX_POINTS);
        }
        let values: Vec<f64> = points.iter().map(|(_, v)| *v).collect();
        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let avg = values.iter().sum::<f64>() / values.len() as f64;
        let last = *values.last().unwrap_or(&0.0);
        Some(Self {
            metric: metric.into(),
            scope,
            from_nanos: from_nanos.to_string(),
            to_nanos: to_nanos.to_string(),
            step_seconds,
            points: points
                .into_iter()
                .map(|(ts, value)| MetricPointLine {
                    ts_nanos: ts.to_string(),
                    value,
                })
                .collect(),
            stats: MetricStats {
                min,
                max,
                avg,
                last,
            },
        })
    }
}

#[derive(Debug, Serialize)]
pub struct SpanLine {
    pub service: String,
    pub name: String,
    pub kind: String,
    pub status_code: String,
    pub duration_us: u128,
    pub db_query: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Hypothesis {
    pub kind: &'static str,
    pub statement: String,
    pub confidence: &'static str,
    pub evidence: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct RedactionReport {
    pub policy: &'static str,
    pub redacted_counts: BTreeMap<&'static str, u64>,
}

#[derive(Debug, Default, Serialize)]
pub struct BoundReport {
    pub max_tokens: usize,
    pub estimated_tokens: usize,
    pub dropped_log_lines: usize,
    pub truncated_stacktrace: bool,
}

fn redaction_rules() -> &'static [(&'static str, Regex)] {
    static CELL: OnceLock<Vec<(&'static str, Regex)>> = OnceLock::new();
    CELL.get_or_init(|| {
        vec![
            (
                "aws_access_key_id",
                Regex::new(r"\bAKIA[0-9A-Z]{16}\b").expect("static regex"),
            ),
            (
                "bearer_token",
                Regex::new(r"Bearer\s+[A-Za-z0-9._\-]{8,}").expect("static regex"),
            ),
            (
                "password_assignment",
                Regex::new(r"(?i)password\s*[=:]\s*\S+").expect("static regex"),
            ),
            (
                "email_address",
                Regex::new(r"\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}\b")
                    .expect("static regex"),
            ),
        ]
    })
}

fn redact(text: &str, report: &mut RedactionReport) -> String {
    let mut out = text.to_string();
    for (name, rule) in redaction_rules() {
        let hits = rule.find_iter(&out).count() as u64;
        if hits > 0 {
            out = rule
                .replace_all(&out, format!("[REDACTED:{name}]"))
                .into_owned();
            *report.redacted_counts.entry(name).or_insert(0) += hits;
        }
    }
    out
}

fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

/// What the bundle is anchored to (spec §8: exactly one of issue fingerprint,
/// run id, trace id).
pub enum BundleAnchor {
    Issue(Box<Issue>),
    Run {
        run: Box<RunRecord>,
        /// Grouped issues whose events fell inside the run's traces.
        issues: Vec<Issue>,
    },
    Trace {
        trace_id: String,
        issues: Vec<Issue>,
    },
}

/// Inputs for assembly — the caller (API layer) fetches these through the
/// storage adapters; assembly itself is pure and deterministic.
pub struct BundleInputs {
    pub anchor: BundleAnchor,
    pub events: Vec<ErrorEventRow>,
    pub trace_spans: Vec<SpanRow>,
    pub trace_logs: Vec<LogRow>,
    /// Pre-fetched, pre-bounded metric windows (the API layer queries the
    /// adapter; assembly stays pure).
    pub metric_windows: Vec<MetricWindow>,
}

fn issue_summary(issue: &Issue) -> IssueSummary {
    IssueSummary {
        title: issue.title.clone(),
        error_type: issue.error_type.clone(),
        culprit: issue.culprit.clone(),
        service: issue.service.clone(),
        status: issue.status.clone(),
        event_count: issue.event_count,
        first_seen_nanos: issue.first_seen_nanos.to_string(),
        last_seen_nanos: issue.last_seen_nanos.to_string(),
    }
}

pub fn assemble(inputs: BundleInputs, max_tokens: usize) -> Bundle {
    let mut redaction = RedactionReport {
        policy: "redaction-lite-v1 (pre-A6)",
        ..Default::default()
    };
    let mut missing = Vec::new();

    // Resolve the anchor into its sections and the primary issue.
    let (anchor, run_section, primary_issue) = match &inputs.anchor {
        BundleAnchor::Issue(issue) => (
            Anchor {
                kind: "issue",
                id: issue.fingerprint.clone(),
            },
            None,
            Some(issue.as_ref().clone()),
        ),
        BundleAnchor::Run { run, issues } => {
            let primary = inputs
                .events
                .first()
                .and_then(|e| issues.iter().find(|i| i.fingerprint == e.fingerprint))
                .or_else(|| issues.first())
                .cloned();
            (
                Anchor {
                    kind: "run",
                    id: run.run_id.clone(),
                },
                Some(RunSection {
                    run_id: run.run_id.clone(),
                    command: run.command.clone(),
                    status: run.status.clone(),
                    exit_code: run.exit_code,
                    started_at_nanos: run.started_at_nanos.to_string(),
                    ended_at_nanos: run.ended_at_nanos.map(|n| n.to_string()),
                    issues: issues.iter().map(issue_summary).collect(),
                }),
                primary,
            )
        }
        BundleAnchor::Trace { trace_id, issues } => {
            let primary = inputs
                .events
                .first()
                .and_then(|e| issues.iter().find(|i| i.fingerprint == e.fingerprint))
                .or_else(|| issues.first())
                .cloned();
            (
                Anchor {
                    kind: "trace",
                    id: trace_id.clone(),
                },
                None,
                primary,
            )
        }
    };

    let latest_event = inputs.events.first().map(|event| EventDetail {
        ts_nanos: event.ts_nanos.to_string(),
        message: redact(&event.message, &mut redaction),
        stacktrace: event
            .stacktrace
            .as_deref()
            .map(|s| redact(s, &mut redaction)),
        source: serde_json::to_string(&event.source)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string(),
        trace_id: event.trace_id.clone(),
    });
    if inputs.events.is_empty() {
        missing.push(match anchor.kind {
            "run" => "no error events inside this run's traces".into(),
            "trace" => "no error events on this trace".into(),
            _ => "no stored error events for this fingerprint (check retention)".to_string(),
        });
    }

    let trace = if inputs.trace_spans.is_empty() {
        missing.push(
            "no trace correlated to the latest event — propagate W3C context end to end".into(),
        );
        None
    } else {
        Some(TraceSection {
            trace_id: inputs.trace_spans[0].trace_id.clone(),
            spans: inputs
                .trace_spans
                .iter()
                .map(|span| SpanLine {
                    service: span.service.clone(),
                    name: span.name.clone(),
                    kind: span.kind.clone(),
                    status_code: span.status_code.clone(),
                    duration_us: span.duration_ns / 1_000,
                    db_query: span
                        .attributes
                        .get("db.query.text")
                        .and_then(|v| v.as_str())
                        .map(|q| redact(q, &mut redaction)),
                })
                .collect(),
        })
    };

    let mut logs: Vec<String> = inputs
        .trace_logs
        .iter()
        .map(|log| {
            redact(
                &format!(
                    "{} {} [{}] {}",
                    log.ts_nanos, log.severity_text, log.service, log.body
                ),
                &mut redaction,
            )
        })
        .collect();
    if logs.is_empty() {
        missing.push(
            "no logs correlated to the trace — bridge the log appender through \
             tracing-opentelemetry"
                .into(),
        );
    }

    if inputs.metric_windows.is_empty() {
        missing.push(
            "no process metrics in the anchor window — export process.cpu/process.memory \
             gauges (run-tagged under the wrapper) for the cross-signal view"
                .into(),
        );
    }

    let hypotheses = rank_hypotheses(
        primary_issue.as_ref(),
        &inputs.events,
        trace.as_ref(),
        &anchor,
    );

    let mut bundle = Bundle {
        schema_version: SCHEMA_VERSION,
        generator: concat!("parallax/", env!("CARGO_PKG_VERSION")),
        anchor,
        issue: primary_issue.as_ref().map(issue_summary),
        run: run_section,
        latest_event,
        trace,
        metric_windows: inputs.metric_windows,
        logs: Vec::new(),
        hypotheses,
        missing_evidence: missing,
        redaction,
        bounded: BoundReport {
            max_tokens,
            ..Default::default()
        },
        canonical_hash: None,
    };

    // Bound: drop oldest log lines first, then truncate the stacktrace tail.
    let base_tokens = estimate_tokens(&serde_json::to_string(&bundle).unwrap_or_default());
    let mut used = base_tokens;
    let mut kept = Vec::new();
    for line in logs.iter().rev() {
        let cost = estimate_tokens(line) + 2;
        if used + cost > max_tokens {
            break;
        }
        used += cost;
        kept.push(line.clone());
    }
    kept.reverse();
    bundle.bounded.dropped_log_lines = logs.len() - kept.len();
    if bundle.bounded.dropped_log_lines > 0 {
        bundle.missing_evidence.push(format!(
            "bounded: dropped {} oldest log lines to fit the {max_tokens}-token budget",
            bundle.bounded.dropped_log_lines
        ));
    }
    logs = kept;
    bundle.logs = logs;

    if used > max_tokens
        && let Some(event) = bundle.latest_event.as_mut()
        && let Some(stack) = event.stacktrace.as_mut()
    {
        let frames: Vec<&str> = stack.lines().take(3).collect();
        *stack = format!("{}\n[... truncated to fit token budget]", frames.join("\n"));
        bundle.bounded.truncated_stacktrace = true;
    }

    let serialized = serde_json::to_string(&bundle).unwrap_or_default();
    bundle.bounded.estimated_tokens = estimate_tokens(&serialized);
    bundle.canonical_hash = Some(canonical_hash(&bundle));
    bundle
}

fn rank_hypotheses(
    primary_issue: Option<&Issue>,
    events: &[ErrorEventRow],
    trace: Option<&TraceSection>,
    anchor: &Anchor,
) -> Vec<Hypothesis> {
    let mut hypotheses = Vec::new();
    let message = events
        .first()
        .map(|e| e.message.to_lowercase())
        .unwrap_or_default();
    let anchor_evidence = format!("{} {}", anchor.kind, anchor.id);
    let error_type = primary_issue
        .map(|i| i.error_type.as_str())
        .or_else(|| events.first().map(|e| e.error_type.as_str()))
        .unwrap_or("The error");

    if [
        "timed out",
        "timeout",
        "pool",
        "connection refused",
        "connection reset",
    ]
    .iter()
    .any(|p| message.contains(p))
    {
        hypotheses.push(Hypothesis {
            kind: "dependency_failure",
            statement: format!(
                "{error_type} points at a downstream dependency timing out or saturated; check \
                 that dependency's capacity and latency in this window."
            ),
            confidence: "medium",
            evidence: vec!["latest event message".to_string(), anchor_evidence.clone()],
        });
    }

    if let Some(trace) = trace {
        if let Some(slowest) = trace.spans.iter().max_by_key(|s| s.duration_us)
            && slowest.duration_us > 1_000_000
        {
            hypotheses.push(Hypothesis {
                kind: "slow_span",
                statement: format!(
                    "Span `{}` in {} took {}ms — the dominant cost in the failing trace.",
                    slowest.name,
                    slowest.service,
                    slowest.duration_us / 1_000
                ),
                confidence: "medium",
                evidence: vec![format!("trace {}", trace.trace_id)],
            });
        }
        if let Some(db) = trace.spans.iter().find(|s| s.db_query.is_some()) {
            hypotheses.push(Hypothesis {
                kind: "database_involved",
                statement: format!(
                    "The failing trace touches the database in `{}` — inspect the captured \
                     query and its plan.",
                    db.name
                ),
                confidence: "low",
                evidence: vec![format!("trace {}", trace.trace_id)],
            });
        }
    }

    if hypotheses.is_empty() {
        hypotheses.push(Hypothesis {
            kind: "insufficient_evidence",
            statement: "The evidence does not support a root-cause hypothesis; see \
                        missing_evidence for what to instrument next."
                .into(),
            confidence: "low",
            evidence: vec![anchor_evidence],
        });
    }
    hypotheses
}

/// Sorted-key compact JSON, SHA-256, hash field excluded (PoC semantics).
fn canonical_hash(bundle: &Bundle) -> String {
    let mut value = serde_json::to_value(bundle).unwrap_or_default();
    if let serde_json::Value::Object(map) = &mut value {
        map.remove("canonical_hash");
    }
    fn canonical(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Object(map) => {
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                let inner: Vec<String> = keys
                    .into_iter()
                    .map(|k| {
                        format!(
                            "{}:{}",
                            serde_json::to_string(k).unwrap_or_default(),
                            canonical(&map[k])
                        )
                    })
                    .collect();
                format!("{{{}}}", inner.join(","))
            }
            serde_json::Value::Array(items) => {
                format!(
                    "[{}]",
                    items.iter().map(canonical).collect::<Vec<_>>().join(",")
                )
            }
            leaf => serde_json::to_string(leaf).unwrap_or_default(),
        }
    }
    let digest = Sha256::digest(canonical(&value).as_bytes());
    format!(
        "sha256:{}",
        digest
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    )
}

/// The agent-facing Markdown projection of the same bundle.
pub fn to_markdown(bundle: &Bundle) -> String {
    let mut out = String::new();
    match (&bundle.issue, bundle.anchor.kind) {
        (Some(issue), "issue") => {
            out.push_str(&format!("# {}\n\n", issue.title));
            out.push_str(&format!(
                "- fingerprint: `{}`\n- service: {}\n- status: {}\n- occurrences: {}\n",
                bundle.anchor.id, issue.service, issue.status, issue.event_count
            ));
            if let Some(culprit) = &issue.culprit {
                out.push_str(&format!("- culprit: `{culprit}`\n"));
            }
        }
        _ => {
            out.push_str(&format!(
                "# {} `{}`\n\n",
                match bundle.anchor.kind {
                    "run" => "Run",
                    "trace" => "Trace",
                    other => other,
                },
                bundle.anchor.id
            ));
        }
    }
    if let Some(run) = &bundle.run {
        if let Some(command) = &run.command {
            out.push_str(&format!("- command: `{command}`\n"));
        }
        out.push_str(&format!("- status: {}\n", run.status));
        if let Some(code) = run.exit_code {
            out.push_str(&format!("- exit code: {code}\n"));
        }
        if run.issues.is_empty() {
            out.push_str("\nNo grouped issues inside this run.\n");
        } else {
            out.push_str("\n## Issues in this run\n\n");
            for issue in &run.issues {
                out.push_str(&format!(
                    "- {} — {} ({} occurrences, {})\n",
                    issue.error_type, issue.title, issue.event_count, issue.status
                ));
            }
        }
    }
    if bundle.anchor.kind != "issue"
        && let Some(issue) = &bundle.issue
    {
        out.push_str(&format!(
            "\n## Primary issue\n\n{} — {} ({} occurrences, service {})\n",
            issue.error_type, issue.title, issue.event_count, issue.service
        ));
    }
    if let Some(event) = &bundle.latest_event {
        out.push_str(&format!("\n## Latest event\n\n{}\n", event.message));
        if let Some(stack) = &event.stacktrace {
            out.push_str(&format!("\n```\n{stack}\n```\n"));
        }
    }
    if let Some(trace) = &bundle.trace {
        out.push_str(&format!("\n## Trace `{}`\n\n", trace.trace_id));
        for span in &trace.spans {
            out.push_str(&format!(
                "- [{}] {} — {} ({}µs)\n",
                span.service, span.name, span.status_code, span.duration_us
            ));
            if let Some(query) = &span.db_query {
                out.push_str(&format!("  - query: `{query}`\n"));
            }
        }
    }
    if !bundle.metric_windows.is_empty() {
        out.push_str("\n## Metric windows\n\n");
        for window in &bundle.metric_windows {
            out.push_str(&format!(
                "- {} ({}-scoped, {} points @ {}s): avg {:.4}, min {:.4}, max {:.4}, last {:.4}\n",
                window.metric,
                window.scope,
                window.points.len(),
                window.step_seconds,
                window.stats.avg,
                window.stats.min,
                window.stats.max,
                window.stats.last,
            ));
        }
    }
    if !bundle.logs.is_empty() {
        out.push_str("\n## Correlated logs\n\n");
        for line in &bundle.logs {
            out.push_str(&format!("- {line}\n"));
        }
    }
    out.push_str("\n## Hypotheses\n\n");
    for h in &bundle.hypotheses {
        out.push_str(&format!(
            "- [{}] ({}) {}\n",
            h.kind, h.confidence, h.statement
        ));
    }
    if !bundle.missing_evidence.is_empty() {
        out.push_str("\n## Missing evidence\n\n");
        for m in &bundle.missing_evidence {
            out.push_str(&format!("- {m}\n"));
        }
    }
    out
}
