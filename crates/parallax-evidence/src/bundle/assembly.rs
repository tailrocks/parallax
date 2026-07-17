use super::*;

/// What the bundle is anchored to (spec §8: exactly one of issue fingerprint,
/// invocation id, trace id).
#[derive(Debug)]
pub enum BundleAnchor {
    Issue(Box<Issue>),
    Invocation {
        invocation: Box<InvocationRecord>,
        /// Grouped issues whose events fell inside the invocation's traces.
        issues: Vec<Issue>,
    },
    Trace {
        trace_id: String,
        issues: Vec<Issue>,
    },
}

/// Inputs for assembly — the caller (API layer) fetches these through the
/// storage adapters; assembly itself is pure and deterministic.
#[derive(Debug)]
pub struct BundleInputs {
    pub anchor: BundleAnchor,
    pub events: Vec<ErrorEventRow>,
    pub trace_spans: Vec<SpanRow>,
    pub trace_logs: Vec<LogRow>,
    /// Pre-fetched, pre-bounded metric windows (the API layer queries the
    /// adapter; assembly stays pure).
    pub metric_windows: Vec<MetricWindow>,
}

use crate::redaction_policy::{EvidenceField, project_text};

fn project_required(field: EvidenceField, value: &str, report: &mut RedactionReport) -> String {
    project_text(field, value, report).unwrap_or_else(|| "[REDACTED:stripped]".to_string())
}

fn issue_summary(issue: &Issue, report: &mut RedactionReport) -> IssueSummary {
    IssueSummary {
        title: project_required(EvidenceField::IssueTitle, &issue.title, report),
        error_type: project_required(EvidenceField::IssueErrorType, &issue.error_type, report),
        culprit: issue
            .culprit
            .as_deref()
            .and_then(|culprit| project_text(EvidenceField::IssueCulprit, culprit, report)),
        service: project_required(EvidenceField::ServiceName, &issue.service, report),
        status: issue.status.clone(),
        event_count: issue.event_count,
        first_seen_nanos: issue.first_seen_nanos.to_string(),
        last_seen_nanos: issue.last_seen_nanos.to_string(),
    }
}

pub fn assemble(inputs: BundleInputs, max_tokens: usize) -> Bundle {
    let mut redaction = RedactionReport {
        policy: REDACTION_POLICY_V1,
        ..Default::default()
    };
    let mut missing = Vec::new();

    // Resolve the anchor into its sections and the primary issue.
    let (anchor, invocation_section, primary_issue) = match &inputs.anchor {
        BundleAnchor::Issue(issue) => (
            Anchor {
                kind: "issue",
                id: project_required(EvidenceField::AnchorId, &issue.fingerprint, &mut redaction),
            },
            None,
            Some(issue.as_ref().clone()),
        ),
        BundleAnchor::Invocation { invocation, issues } => {
            let primary = inputs
                .events
                .first()
                .and_then(|e| issues.iter().find(|i| i.fingerprint == e.fingerprint))
                .or_else(|| issues.first())
                .cloned();
            (
                Anchor {
                    kind: "invocation",
                    id: project_required(
                        EvidenceField::AnchorId,
                        &invocation.invocation_id,
                        &mut redaction,
                    ),
                },
                Some(InvocationSection {
                    invocation_id: project_required(
                        EvidenceField::AnchorId,
                        &invocation.invocation_id,
                        &mut redaction,
                    ),
                    command: invocation.command.as_deref().and_then(|command| {
                        project_text(EvidenceField::InvocationCommand, command, &mut redaction)
                    }),
                    app_mode: invocation.app_mode.as_deref().map(|mode| {
                        project_required(EvidenceField::InvocationMode, mode, &mut redaction)
                    }),
                    outcome: invocation.outcome.as_deref().map(|outcome| {
                        project_required(EvidenceField::InvocationOutcome, outcome, &mut redaction)
                    }),
                    status: invocation.status.clone(),
                    exit_code: invocation.exit_code,
                    started_at_nanos: invocation.started_at_nanos.to_string(),
                    ended_at_nanos: invocation.ended_at_nanos.map(|n| n.to_string()),
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
                    id: project_required(EvidenceField::TraceId, trace_id, &mut redaction),
                },
                None,
                primary,
            )
        }
    };

    let latest_event = inputs.events.first().map(|event| EventDetail {
        ts_nanos: event.ts_nanos.to_string(),
        message: project_required(EvidenceField::EventMessage, &event.message, &mut redaction),
        stacktrace: event
            .stacktrace
            .as_deref()
            .and_then(|s| project_text(EvidenceField::EventStacktrace, s, &mut redaction)),
        source: serde_json::to_string(&event.source)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string(),
        trace_id: project_required(EvidenceField::TraceId, &event.trace_id, &mut redaction),
    });
    if inputs.events.is_empty() {
        missing.push(match anchor.kind {
            "invocation" => "no error events inside this invocation's traces".into(),
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
            trace_id: project_required(
                EvidenceField::TraceId,
                &inputs.trace_spans[0].trace_id,
                &mut redaction,
            ),
            spans: inputs
                .trace_spans
                .iter()
                .map(|span| SpanLine {
                    service: project_required(
                        EvidenceField::ServiceName,
                        &span.service,
                        &mut redaction,
                    ),
                    name: project_required(EvidenceField::SpanName, &span.name, &mut redaction),
                    kind: project_required(EvidenceField::SpanKind, &span.kind, &mut redaction),
                    status_code: project_required(
                        EvidenceField::SpanStatus,
                        &span.status_code,
                        &mut redaction,
                    ),
                    duration_us: span.duration_ns / 1_000,
                    db_query: span
                        .attributes
                        .get("db.query.text")
                        .and_then(|v| v.as_str())
                        .and_then(|q| {
                            project_text(EvidenceField::DatabaseQueryText, q, &mut redaction)
                        }),
                })
                .collect(),
        })
    };

    let mut logs: Vec<String> = inputs
        .trace_logs
        .iter()
        .map(|log| {
            let body = project_required(EvidenceField::LogBody, &log.body, &mut redaction);
            format!(
                "{} {} [{}] {body}",
                log.ts_nanos, log.severity_text, log.service
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
             gauges (invocation-tagged under the wrapper) for the cross-signal view"
                .into(),
        );
    }

    for gap in crate::gaps::detect_gaps(&inputs.trace_spans, &inputs.trace_logs) {
        let line = format!("{}: {}", gap.kind, gap.detail);
        if !missing.contains(&line) {
            missing.push(line);
        }
    }

    let issue = primary_issue
        .as_ref()
        .map(|issue| issue_summary(issue, &mut redaction));
    let hypotheses = rank_hypotheses(
        issue.as_ref(),
        latest_event.as_ref(),
        trace.as_ref(),
        &anchor,
    );

    let mut bundle = Bundle {
        schema_version: SCHEMA_VERSION,
        generator: concat!("parallax/", env!("CARGO_PKG_VERSION")),
        anchor,
        issue,
        invocation: invocation_section,
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
