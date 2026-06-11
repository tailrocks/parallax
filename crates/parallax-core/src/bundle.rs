//! The evidence bundle: bounded, redacted, hypothesis-ranked context for one
//! issue — graduated from `poc/evidence-loop` (bundle/bound/redact/hypothesis
//! kernels) onto the live row model. The same JSON powers the GraphQL
//! `bundle` field, the CLI's `issue context`, and the UI's bundle preview.

use parallax_storage::model::{ErrorEventRow, Issue, LogRow, SpanRow};
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
    pub issue: IssueSummary,
    pub latest_event: Option<EventDetail>,
    pub trace: Option<TraceSection>,
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
    pub fingerprint: String,
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

/// Inputs for assembly — the caller (API layer) fetches these through the
/// storage adapters; assembly itself is pure and deterministic.
pub struct BundleInputs {
    pub issue: Issue,
    pub events: Vec<ErrorEventRow>,
    pub trace_spans: Vec<SpanRow>,
    pub trace_logs: Vec<LogRow>,
}

pub fn assemble(inputs: BundleInputs, max_tokens: usize) -> Bundle {
    let mut redaction = RedactionReport {
        policy: "redaction-lite-v1 (pre-A6)",
        ..Default::default()
    };
    let mut missing = Vec::new();

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
        missing.push("no stored error events for this fingerprint (check retention)".into());
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

    let hypotheses = rank_hypotheses(&inputs, trace.as_ref());

    let mut bundle = Bundle {
        schema_version: SCHEMA_VERSION,
        generator: concat!("parallax/", env!("CARGO_PKG_VERSION")),
        anchor: Anchor {
            kind: "issue",
            fingerprint: inputs.issue.fingerprint.clone(),
        },
        issue: IssueSummary {
            title: inputs.issue.title.clone(),
            error_type: inputs.issue.error_type.clone(),
            culprit: inputs.issue.culprit.clone(),
            service: inputs.issue.service.clone(),
            status: inputs.issue.status.clone(),
            event_count: inputs.issue.event_count,
            first_seen_nanos: inputs.issue.first_seen_nanos.to_string(),
            last_seen_nanos: inputs.issue.last_seen_nanos.to_string(),
        },
        latest_event,
        trace,
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

fn rank_hypotheses(inputs: &BundleInputs, trace: Option<&TraceSection>) -> Vec<Hypothesis> {
    let mut hypotheses = Vec::new();
    let message = inputs
        .events
        .first()
        .map(|e| e.message.to_lowercase())
        .unwrap_or_default();

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
                "{} points at a downstream dependency timing out or saturated; check that \
                 dependency's capacity and latency in this window.",
                inputs.issue.error_type
            ),
            confidence: "medium",
            evidence: vec![
                format!("latest event message"),
                format!("issue {}", inputs.issue.fingerprint),
            ],
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
            evidence: vec![format!("issue {}", inputs.issue.fingerprint)],
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
    out.push_str(&format!("# {}\n\n", bundle.issue.title));
    out.push_str(&format!(
        "- fingerprint: `{}`\n- service: {}\n- status: {}\n- occurrences: {}\n",
        bundle.anchor.fingerprint,
        bundle.issue.service,
        bundle.issue.status,
        bundle.issue.event_count
    ));
    if let Some(culprit) = &bundle.issue.culprit {
        out.push_str(&format!("- culprit: `{culprit}`\n"));
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
