//! Asserts the four properties this PoC exists to prove:
//! 1. Error derivation works from both exception encodings (span event and
//!    log record) plus plain ERROR logs — no fourth signal needed.
//! 2. Both exception encodings converge to the same fingerprint.
//! 3. Seeded secrets never reach the serialized bundle, and the redaction
//!    report records what was removed.
//! 4. Bundle output is deterministic: identical fixtures → identical
//!    canonical hash.

use evidence_loop_poc::derive::ErrorSource;
use evidence_loop_poc::run_pipeline;

const TRACE: &str = include_str!("../fixtures/otlp-trace.json");
const LOGS: &str = include_str!("../fixtures/otlp-logs.json");

#[test]
fn derives_error_events_from_spans_and_logs() {
    let out = run_pipeline("proj_checkout", TRACE, LOGS).unwrap();
    assert_eq!(out.error_events.len(), 3, "span exception + ERROR log + exception-as-log");
    assert!(out.error_events.iter().any(|e| e.source == ErrorSource::SpanException));
    assert!(out.error_events.iter().any(|e| e.source == ErrorSource::LogRecord));
    assert!(out.error_events.iter().any(|e| e.source == ErrorSource::LogException));
}

#[test]
fn both_exception_encodings_share_one_fingerprint() {
    let out = run_pipeline("proj_checkout", TRACE, LOGS).unwrap();
    let span_fp = out
        .error_events
        .iter()
        .find(|e| e.source == ErrorSource::SpanException)
        .unwrap()
        .fingerprint
        .clone();
    let log_fp = out
        .error_events
        .iter()
        .find(|e| e.source == ErrorSource::LogException)
        .unwrap()
        .fingerprint
        .clone();
    assert_eq!(span_fp, log_fp, "old and new OTel exception encodings must group together");

    // Two distinct fingerprints overall: the converged exception group plus
    // the plain ERROR log line.
    assert_eq!(out.bundles.len(), 2);
    let exception_bundle = out
        .bundles
        .iter()
        .find(|b| b.anchor.fingerprint == span_fp)
        .expect("bundle anchored on the exception fingerprint");
    assert!(
        exception_bundle.edges.iter().any(|e| e.r#type == "same_fingerprint"),
        "converged events must be linked by a same_fingerprint edge"
    );
    assert!(
        exception_bundle.edges.iter().any(|e| e.r#type == "error_in_span" && e.strength == "strong"),
        "error must link to its span with a strong edge"
    );
}

#[test]
fn seeded_secrets_never_reach_the_bundle() {
    let out = run_pipeline("proj_checkout", TRACE, LOGS).unwrap();
    for b in &out.bundles {
        let serialized = serde_json::to_string(b).unwrap();
        assert!(!serialized.contains("AKIAIOSFODNN7EXAMPLE"), "AWS key canary leaked");
        assert!(!serialized.contains("abc123secrettoken"), "bearer token canary leaked");
        assert!(!serialized.contains("jane.doe@example.com"), "email canary leaked");
    }
    let total: u64 = out.bundles.iter().map(|b| b.redaction_report.total()).sum();
    assert!(total >= 3, "redaction report must record the removals, got {total}");
}

#[test]
fn bundles_have_required_contract_fields() {
    let out = run_pipeline("proj_checkout", TRACE, LOGS).unwrap();
    for b in &out.bundles {
        assert_eq!(b.schema_version, "bundle-v0-poc");
        assert!(!b.missing_evidence.is_empty(), "missing evidence must be explicit");
        assert!(b.canonical_hash.as_deref().unwrap_or("").starts_with("sha256:"));
        assert!(b.trigger.dispatch_eligible);
    }
}

#[test]
fn output_is_deterministic() {
    let a = run_pipeline("proj_checkout", TRACE, LOGS).unwrap();
    let b = run_pipeline("proj_checkout", TRACE, LOGS).unwrap();
    let hashes = |o: &evidence_loop_poc::PipelineOutput| {
        o.bundles.iter().map(|x| x.canonical_hash.clone().unwrap()).collect::<Vec<_>>()
    };
    assert_eq!(hashes(&a), hashes(&b), "identical input must produce identical canonical hashes");
}
