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

fn estimate_bundle_tokens(bundle: &Bundle) -> usize {
    estimate_tokens(&serde_json::to_string(bundle).unwrap_or_default())
}

fn retain_top_trace_spans(trace: &mut TraceSection, keep: usize) {
    if keep >= trace.spans.len() {
        return;
    }
    let mut ranked: Vec<usize> = (0..trace.spans.len()).collect();
    ranked.sort_by(|&a, &b| {
        let a_span = &trace.spans[a];
        let b_span = &trace.spans[b];
        let a_error = a_span.status_code.contains("ERROR");
        let b_error = b_span.status_code.contains("ERROR");
        b_error
            .cmp(&a_error)
            .then_with(|| b_span.duration_us.cmp(&a_span.duration_us))
            .then_with(|| a.cmp(&b))
    });
    let mut selected = vec![false; trace.spans.len()];
    for index in ranked.into_iter().take(keep.max(1)) {
        selected[index] = true;
    }
    let spans = std::mem::take(&mut trace.spans);
    trace.spans = spans
        .into_iter()
        .enumerate()
        .filter_map(|(index, span)| selected[index].then_some(span))
        .collect();
}

fn bound_trace_spans(bundle: &mut Bundle, max_tokens: usize) {
    let Some(original_len) = bundle.trace.as_ref().map(|trace| trace.spans.len()) else {
        return;
    };
    if original_len <= 1 {
        return;
    }
    let mut keep = original_len;
    while estimate_bundle_tokens(bundle) > max_tokens && keep > 1 {
        keep = keep.saturating_sub((keep / 4).max(1));
        if let Some(trace) = bundle.trace.as_mut() {
            retain_top_trace_spans(trace, keep);
        }
    }
    let final_len = bundle
        .trace
        .as_ref()
        .map(|trace| trace.spans.len())
        .unwrap_or(0);
    let dropped = original_len.saturating_sub(final_len);
    if dropped > 0 {
        bundle.missing_evidence.push(format!(
            "bounded: dropped {dropped} trace spans to fit budget"
        ));
    }
}

fn decimate_points(points: &mut Vec<MetricPointLine>, keep: usize) -> usize {
    if keep >= points.len() {
        return 0;
    }
    let original_len = points.len();
    let keep = keep.max(1);
    let mut selected = vec![false; original_len];
    if keep == 1 {
        selected[original_len - 1] = true;
    } else {
        for slot in 0..keep {
            selected[slot * (original_len - 1) / (keep - 1)] = true;
        }
    }
    let mut selected_count = selected.iter().filter(|&&keep| keep).count();
    for keep_slot in &mut selected {
        if selected_count >= keep {
            break;
        }
        if !*keep_slot {
            *keep_slot = true;
            selected_count += 1;
        }
    }
    let old = std::mem::take(points);
    *points = old
        .into_iter()
        .enumerate()
        .filter_map(|(index, point)| selected[index].then_some(point))
        .collect();
    original_len - points.len()
}

fn bound_metric_windows(bundle: &mut Bundle, max_tokens: usize) {
    let mut dropped = 0usize;
    loop {
        if estimate_bundle_tokens(bundle) <= max_tokens {
            break;
        }
        let mut changed = false;
        for window in &mut bundle.metric_windows {
            if window.points.len() <= 2 {
                continue;
            }
            let keep = window
                .points
                .len()
                .saturating_sub((window.points.len() / 4).max(1));
            dropped += decimate_points(&mut window.points, keep.max(2));
            changed = true;
        }
        if !changed {
            break;
        }
    }
    if dropped > 0 {
        bundle.missing_evidence.push(format!(
            "bounded: dropped {dropped} metric points to fit budget"
        ));
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

fn redaction_rules() -> &'static [(&'static str, Regex, &'static str)] {
    static CELL: OnceLock<Vec<(&'static str, Regex, &'static str)>> = OnceLock::new();
    CELL.get_or_init(|| {
        vec![
            (
                "dsn_userinfo",
                Regex::new(r"://[^/\s:@]+:[^/\s@]+@").expect("static regex"),
                "://[REDACTED:dsn_userinfo]@",
            ),
            (
                "private_key_block",
                Regex::new(
                    r"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----",
                )
                .expect("static regex"),
                "[REDACTED:private_key_block]",
            ),
            (
                "github_token",
                Regex::new(r"\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{20,}\b").expect("static regex"),
                "[REDACTED:github_token]",
            ),
            (
                "github_pat",
                Regex::new(r"\bgithub_pat_[A-Za-z0-9_]{20,}\b").expect("static regex"),
                "[REDACTED:github_pat]",
            ),
            (
                "slack_token",
                Regex::new(r"\bxox[baprs]-[A-Za-z0-9-]{10,}\b").expect("static regex"),
                "[REDACTED:slack_token]",
            ),
            (
                "jwt",
                Regex::new(r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b")
                    .expect("static regex"),
                "[REDACTED:jwt]",
            ),
            (
                "aws_access_key_id",
                Regex::new(r"\bAKIA[0-9A-Z]{16}\b").expect("static regex"),
                "[REDACTED:aws_access_key_id]",
            ),
            (
                "bearer_token",
                Regex::new(r"Bearer\s+[A-Za-z0-9._\-]{8,}").expect("static regex"),
                "[REDACTED:bearer_token]",
            ),
            (
                "password_assignment",
                Regex::new(r"(?i)password\s*[=:]\s*\S+").expect("static regex"),
                "[REDACTED:password_assignment]",
            ),
            (
                "email_address",
                Regex::new(r"\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}\b")
                    .expect("static regex"),
                "[REDACTED:email_address]",
            ),
        ]
    })
}

fn redact(text: &str, report: &mut RedactionReport) -> String {
    let mut out = text.to_string();
    for (name, rule, replacement) in redaction_rules() {
        let hits = rule.find_iter(&out).count() as u64;
        if hits > 0 {
            out = rule.replace_all(&out, *replacement).into_owned();
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

fn issue_summary(issue: &Issue, report: &mut RedactionReport) -> IssueSummary {
    IssueSummary {
        title: redact(&issue.title, report),
        error_type: issue.error_type.clone(),
        culprit: issue
            .culprit
            .as_deref()
            .map(|culprit| redact(culprit, report)),
        service: issue.service.clone(),
        status: issue.status.clone(),
        event_count: issue.event_count,
        first_seen_nanos: issue.first_seen_nanos.to_string(),
        last_seen_nanos: issue.last_seen_nanos.to_string(),
    }
}

pub fn assemble(inputs: BundleInputs, max_tokens: usize) -> Bundle {
    let mut redaction = RedactionReport {
        policy: "redaction-lite-v2",
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
                    command: run
                        .command
                        .as_deref()
                        .map(|command| redact(command, &mut redaction)),
                    status: run.status.clone(),
                    exit_code: run.exit_code,
                    started_at_nanos: run.started_at_nanos.to_string(),
                    ended_at_nanos: run.ended_at_nanos.map(|n| n.to_string()),
                    issues: issues
                        .iter()
                        .map(|issue| issue_summary(issue, &mut redaction))
                        .collect(),
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
        issue: primary_issue
            .as_ref()
            .map(|issue| issue_summary(issue, &mut redaction)),
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

    if estimate_bundle_tokens(&bundle) > max_tokens
        && let Some(event) = bundle.latest_event.as_mut()
        && let Some(stack) = event.stacktrace.as_mut()
    {
        let frames: Vec<&str> = stack.lines().take(3).collect();
        *stack = format!("{}\n[... truncated to fit token budget]", frames.join("\n"));
        bundle.bounded.truncated_stacktrace = true;
    }

    bound_trace_spans(&mut bundle, max_tokens);
    bound_metric_windows(&mut bundle, max_tokens);

    bundle.bounded.estimated_tokens = estimate_bundle_tokens(&bundle);
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

/// Sorted-key compact SHA-256 over evidence fields only.
///
/// Covered: schema_version, anchor, issue, run, latest_event, trace,
/// metric_windows, logs, hypotheses, missing_evidence, and redaction. Excluded:
/// generator (build environment), bounded (per-request budget accounting), and
/// canonical_hash itself.
fn canonical_hash(bundle: &Bundle) -> String {
    let mut value = serde_json::to_value(bundle).unwrap_or_default();
    if let serde_json::Value::Object(map) = &mut value {
        map.remove("canonical_hash");
        map.remove("generator");
        map.remove("bounded");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_issue() -> Issue {
        Issue {
            fingerprint: "fp".to_string(),
            title: "test::Boom: boom".to_string(),
            error_type: "test::Boom".to_string(),
            culprit: Some("top".to_string()),
            service: "checkout".to_string(),
            status: "open".to_string(),
            first_seen_nanos: 1,
            last_seen_nanos: 2,
            event_count: 1,
            last_trace_id: Some("trace".to_string()),
            tags: "{}".to_string(),
        }
    }

    fn test_event() -> ErrorEventRow {
        ErrorEventRow {
            ts_nanos: 2,
            service: "checkout".to_string(),
            fingerprint: "fp".to_string(),
            error_type: "test::Boom".to_string(),
            message: "boom".to_string(),
            stacktrace: Some("top\nmiddle\nbottom\nextra".to_string()),
            source: parallax_storage::model::ErrorSource::SpanException,
            trace_id: "trace".to_string(),
            span_id: "span-error".to_string(),
            attributes: serde_json::Value::Null,
        }
    }

    fn test_span(index: usize, error: bool, duration_us: u128) -> SpanRow {
        SpanRow {
            ts_nanos: index as u128,
            service: "checkout".to_string(),
            trace_id: "trace".to_string(),
            span_id: format!("span-{index}"),
            parent_span_id: None,
            name: format!("span-{index}"),
            kind: "SPAN_KIND_INTERNAL".to_string(),
            status_code: if error {
                "STATUS_CODE_ERROR".to_string()
            } else {
                "STATUS_CODE_UNSET".to_string()
            },
            status_message: String::new(),
            duration_ns: duration_us * 1_000,
            run_id: None,
            scope_name: "test".to_string(),
            events: None,
            links: serde_json::Value::Null,
            attributes: serde_json::Value::Null,
            resource: serde_json::Value::Null,
        }
    }

    fn test_inputs(spans: Vec<SpanRow>) -> BundleInputs {
        BundleInputs {
            anchor: BundleAnchor::Issue(Box::new(test_issue())),
            events: vec![test_event()],
            trace_spans: spans,
            trace_logs: Vec::new(),
            metric_windows: Vec::new(),
        }
    }

    #[test]
    fn canonical_hash_ignores_generator() {
        let mut left = assemble(test_inputs(vec![test_span(0, true, 10)]), 8_000);
        let mut right = assemble(test_inputs(vec![test_span(0, true, 10)]), 8_000);
        left.generator = "parallax/test-a";
        right.generator = "parallax/test-b";

        assert_eq!(canonical_hash(&left), canonical_hash(&right));
    }

    #[test]
    fn large_trace_is_bounded_and_keeps_error_span() {
        let spans = (0..400)
            .map(|index| test_span(index, index == 123, (index as u128 + 1) * 1_000))
            .collect();

        let bundle = assemble(test_inputs(spans), 500);

        let trace = bundle.trace.as_ref().expect("trace");
        assert!(
            trace
                .spans
                .iter()
                .any(|span| span.status_code == "STATUS_CODE_ERROR"),
            "error span survives trace bounding"
        );
        assert!(
            bundle
                .missing_evidence
                .iter()
                .any(|message| message.contains("dropped") && message.contains("trace spans")),
            "trace bounding records dropped spans: {:?}",
            bundle.missing_evidence
        );
        assert!(
            bundle.bounded.estimated_tokens <= bundle.bounded.max_tokens
                || bundle
                    .missing_evidence
                    .iter()
                    .any(|message| message.contains("dropped") && message.contains("trace spans")),
            "bundle either fits or reports trace drops"
        );
    }

    #[test]
    fn redact_masks_dsn_userinfo_and_preserves_context() {
        let mut report = RedactionReport::default();

        let out = redact("connect postgres://admin:s3cr3t@db:5432/app", &mut report);

        assert!(out.contains("postgres://"));
        assert!(out.contains("[REDACTED:dsn_userinfo]"));
        assert!(out.contains("@db:5432"));
        assert!(!out.contains("s3cr3t"));
        assert_eq!(report.redacted_counts.get("dsn_userinfo"), Some(&1));
    }

    #[test]
    fn redact_leaves_url_without_userinfo_unchanged() {
        let mut report = RedactionReport::default();

        let out = redact("fetch https://example.com/path", &mut report);

        assert_eq!(out, "fetch https://example.com/path");
        assert!(report.redacted_counts.is_empty());
    }

    #[test]
    fn redact_masks_private_key_blocks() {
        let mut report = RedactionReport::default();
        let input = "before\n-----BEGIN PRIVATE KEY-----\nabc123\n-----END PRIVATE KEY-----\nafter";

        let out = redact(input, &mut report);

        assert_eq!(out, "before\n[REDACTED:private_key_block]\nafter");
        assert!(!out.contains("BEGIN PRIVATE KEY"));
        assert!(!out.contains("abc123"));
        assert_eq!(report.redacted_counts.get("private_key_block"), Some(&1));
    }

    #[test]
    fn redact_masks_common_token_prefixes() {
        let mut report = RedactionReport::default();
        let input = concat!(
            "ghp_0123456789ABCDEFGHIJKLMNOPQRST ",
            "github_pat_0123456789abcdefghijklmnopqrst ",
            "xoxb-1234567890abcdef ",
            "eyJaaaaaaaaaa.bbbbbbbbbbbb.cccccccccccc"
        );

        let out = redact(input, &mut report);

        assert!(out.contains("[REDACTED:github_token]"));
        assert!(out.contains("[REDACTED:github_pat]"));
        assert!(out.contains("[REDACTED:slack_token]"));
        assert!(out.contains("[REDACTED:jwt]"));
        assert!(!out.contains("ghp_0123456789"));
        assert!(!out.contains("github_pat_0123456789"));
        assert!(!out.contains("xoxb-1234567890"));
        assert!(!out.contains("eyJaaaaaaaaaa"));
        assert_eq!(report.redacted_counts.get("github_token"), Some(&1));
        assert_eq!(report.redacted_counts.get("github_pat"), Some(&1));
        assert_eq!(report.redacted_counts.get("slack_token"), Some(&1));
        assert_eq!(report.redacted_counts.get("jwt"), Some(&1));
    }
}
