//! Instrumentation suggestions: the machine-actionable half of the
//! reproduce-and-instrument loop (lifecycle 3,
//! docs/research/00-vision/problem-audience-product-shape.md §5).
//!
//! The agent's gap-closing move — "I see what evidence was missing, let me
//! add the instrumentation and ask you to reproduce once more" — needs the
//! gaps as structured TODOs, not prose. Rules read the bundle's
//! `missing_evidence` entries and the evidence graph itself, and emit
//! suggestions that point at the integration-contract convention closing each
//! gap. Unmapped gaps are surfaced, never silently dropped.

use crate::bundle::{Bundle, Node};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct InstrumentationSuggestion {
    pub id: String,
    /// The gap that triggered this suggestion (verbatim missing_evidence
    /// entry, or a graph-derived gap).
    pub gap: String,
    /// What to add, agent-actionable.
    pub suggestion: String,
    /// Where in the application/pipeline the change lands.
    pub where_hint: String,
    /// Which integration-contract section defines the convention.
    pub convention_ref: String,
}

#[derive(Debug, Serialize)]
pub struct InstrumentationReport {
    pub suggestions: Vec<InstrumentationSuggestion>,
    /// missing_evidence entries no rule understood — visible, not dropped.
    pub unmapped_gaps: Vec<String>,
}

/// Derive structured instrumentation TODOs from a bundle.
pub fn suggest_instrumentation(bundle: &Bundle) -> InstrumentationReport {
    let mut suggestions = Vec::new();
    let mut unmapped = Vec::new();
    let mut push = |gap: &str, suggestion: &str, where_hint: &str, convention_ref: &str,
                    suggestions: &mut Vec<InstrumentationSuggestion>| {
        suggestions.push(InstrumentationSuggestion {
            id: format!("ins_{}", suggestions.len() + 1),
            gap: gap.to_string(),
            suggestion: suggestion.to_string(),
            where_hint: where_hint.to_string(),
            convention_ref: convention_ref.to_string(),
        });
    };

    for gap in &bundle.missing_evidence {
        if gap.contains("no metric windows") {
            push(
                gap,
                "Initialize an OTLP metrics exporter beside traces/logs and export runtime and dependency metrics (e.g. connection-pool in-use/idle gauges, request rate); metric windows then join error evidence automatically.",
                "telemetry init (opentelemetry_sdk::metrics alongside the tracer/logger providers)",
                "integration-contract.md §1",
                &mut suggestions,
            );
        } else if gap.contains("no deploy event") {
            push(
                gap,
                "Emit a parallax.deploy.v0 event from the deploy pipeline (or enable the GitHub deployments webhook ingest) so deploy-adjacent regressions become detectable with a strong edge.",
                "CI/CD deploy step (one curl) or VCS webhook configuration",
                "integration-contract.md §2–3",
                &mut suggestions,
            );
        } else if gap.contains("bounded: dropped") {
            push(
                gap,
                "Evidence was trimmed to fit the token budget; prefer fewer, higher-signal structured log fields near the failure path over raw volume, or scope DEBUG logging to the failing component.",
                "logging configuration of the failing component",
                "agent-context-integration.md (bounded-bundle rule)",
                &mut suggestions,
            );
        } else if gap.contains("deploy adjacency not evaluated") {
            // Informational for run-anchored bundles; nothing to instrument.
            continue;
        } else {
            unmapped.push(gap.clone());
        }
    }

    // Graph-derived gaps, beyond missing_evidence:
    let has_log_window = bundle.nodes.iter().any(|n| matches!(n, Node::LogWindow { .. }));
    if !has_log_window {
        push(
            "no log records share the anchoring trace_id",
            "Attach trace context to logs so they correlate: bridge the log appender through the tracing/OpenTelemetry layer (tracing-opentelemetry + OTLP log exporter).",
            "logger initialization",
            "integration-contract.md §1",
            &mut suggestions,
        );
    }
    if bundle
        .nodes
        .iter()
        .any(|n| matches!(n, Node::ErrorEvent { stacktrace: None, source, .. } if *source != crate::derive::ErrorSource::LogRecord))
    {
        push(
            "an exception-derived error event carries no stacktrace",
            "Enable backtrace capture so exception events carry frames: set a panic hook with std::backtrace (RUST_BACKTRACE=1 in dev) and record exception.stacktrace on the exception event/log.",
            "panic/error-reporting setup",
            "capture/rust.md",
            &mut suggestions,
        );
    }
    if bundle
        .edges
        .iter()
        .any(|e| e.r#type == "deploy_preceded_issue" && e.strength == "medium")
    {
        push(
            "deploy edge is medium (time adjacency only; deployed SHA did not match the service revision)",
            "Stamp vcs.ref.head.revision and vcs.repository.url.full into the service's OTel resource at build time (e.g. vergen) so deploy edges upgrade to strong via SHA equality.",
            "build script + telemetry resource attributes",
            "integration-contract.md §1",
            &mut suggestions,
        );
    }

    InstrumentationReport { suggestions, unmapped_gaps: unmapped }
}
