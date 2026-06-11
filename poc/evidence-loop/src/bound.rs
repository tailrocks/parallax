//! Token-budget bounding (Context stage).
//!
//! The product definition is a *bounded*, redacted, citable bundle:
//! docs/research/architecture/agent-context-integration.md caps the default
//! agent-visible bundle at ~10K tokens (clients warn at 10K, truncate around
//! 25K). This kernel enforces the cap deterministically and honestly: every
//! trim is recorded in `missing_evidence`, anchor-critical nodes (error
//! events, spans, deploys) are never dropped, and the canonical hash is
//! recomputed over the bounded artifact.
//!
//! Token estimation is the chars/4 heuristic for the PoC; the real
//! implementation makes the tokenizer pluggable.

use crate::bundle::{canonical_hash, Bundle, Node};
use serde::Serialize;

pub const DEFAULT_MAX_TOKENS: usize = 10_000;
const STACKTRACE_KEEP_FRAMES: usize = 3;

pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

fn bundle_tokens(bundle: &Bundle) -> usize {
    estimate_tokens(&serde_json::to_string(bundle).expect("bundle serializes"))
}

#[derive(Debug, Default, Serialize, PartialEq, Eq)]
pub struct BoundReport {
    pub before_tokens: usize,
    pub after_tokens: usize,
    pub dropped_log_lines: usize,
    pub truncated_stacktraces: usize,
}

/// Bound a bundle to `max_tokens`. Trim order: oldest log lines first (the
/// bulk), then stacktrace tails. Idempotent: bounding an already-bounded
/// bundle drops nothing.
pub fn bound_bundle(bundle: &mut Bundle, max_tokens: usize) -> BoundReport {
    let before_tokens = bundle_tokens(bundle);
    let mut report = BoundReport { before_tokens, after_tokens: before_tokens, ..Default::default() };
    if before_tokens <= max_tokens {
        return report;
    }

    // Step 1: halve log windows oldest-first until the bundle fits (or one
    // line remains per window). Halving keeps the pass count logarithmic.
    while bundle_tokens(bundle) > max_tokens {
        let mut dropped_this_round = 0;
        for node in bundle.nodes.iter_mut() {
            if let Node::LogWindow { lines, .. } = node {
                if lines.len() > 1 {
                    let drop = (lines.len() / 2).max(1);
                    lines.drain(..drop);
                    dropped_this_round += drop;
                }
            }
        }
        report.dropped_log_lines += dropped_this_round;
        if dropped_this_round == 0 {
            break;
        }
    }

    // Step 2: truncate stacktraces to the top frames.
    if bundle_tokens(bundle) > max_tokens {
        for node in bundle.nodes.iter_mut() {
            if let Node::ErrorEvent { stacktrace: Some(trace), .. } = node {
                let frames: Vec<&str> = trace.lines().collect();
                if frames.len() > STACKTRACE_KEEP_FRAMES {
                    *trace = format!(
                        "{}\n[... {} frames bounded out]",
                        frames[..STACKTRACE_KEEP_FRAMES].join("\n"),
                        frames.len() - STACKTRACE_KEEP_FRAMES
                    );
                    report.truncated_stacktraces += 1;
                }
            }
        }
    }

    // Honesty rule: every trim is explicit evidence of absence.
    if report.dropped_log_lines > 0 {
        bundle.missing_evidence.push(format!(
            "bounded: dropped {} oldest log lines to fit the {max_tokens}-token budget",
            report.dropped_log_lines
        ));
    }
    if report.truncated_stacktraces > 0 {
        bundle.missing_evidence.push(format!(
            "bounded: truncated {} stacktrace(s) to top {STACKTRACE_KEEP_FRAMES} frames \
             to fit the {max_tokens}-token budget",
            report.truncated_stacktraces
        ));
    }

    let hash = canonical_hash(bundle);
    bundle.canonical_hash = Some(hash);
    report.after_tokens = bundle_tokens(bundle);
    report
}
