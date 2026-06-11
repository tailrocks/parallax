//! Asserts the properties this PoC exists to prove:
//! 1. Error derivation works from both exception encodings (span event and
//!    log record) plus plain ERROR logs — no fourth signal needed.
//! 2. Both exception encodings converge to the same fingerprint.
//! 3. Seeded secrets never reach the serialized bundle, and the redaction
//!    report records what was removed.
//! 4. Bundle output is deterministic: identical fixtures → identical
//!    canonical hash.
//! 5. Deploy adjacency escalates the trigger to deploy_adjacent_regression
//!    with a strong edge when the deployed SHA matches the service's
//!    vcs.ref.head.revision.
//! 6. The Reconciler recurrence kernel returns the right verdict for the
//!    recurred / silent / window-open cases.

use evidence_loop_poc::budget::{compute_budget, OutcomeRow, OutcomesData};
use evidence_loop_poc::deploy::{reconcile_recurrence, RecurrenceVerdict};
use evidence_loop_poc::derive::ErrorSource;
use evidence_loop_poc::dispatch::build_fix_candidate;
use evidence_loop_poc::learn::compute_edge_weights;
use evidence_loop_poc::run_pipeline;

const TRACE: &str = include_str!("../fixtures/otlp-trace.json");
const LOGS: &str = include_str!("../fixtures/otlp-logs.json");
const DEPLOYS: &str = include_str!("../fixtures/deploy-events.json");
const OUTCOMES: &str = include_str!("../fixtures/outcome-rows.json");

fn outcome_rows() -> Vec<OutcomeRow> {
    serde_json::from_str::<OutcomesData>(OUTCOMES).unwrap().outcomes
}

#[test]
fn derives_error_events_from_spans_and_logs() {
    let out = run_pipeline("proj_checkout", TRACE, LOGS, None).unwrap();
    assert_eq!(out.error_events.len(), 3, "span exception + ERROR log + exception-as-log");
    assert!(out.error_events.iter().any(|e| e.source == ErrorSource::SpanException));
    assert!(out.error_events.iter().any(|e| e.source == ErrorSource::LogRecord));
    assert!(out.error_events.iter().any(|e| e.source == ErrorSource::LogException));
}

#[test]
fn both_exception_encodings_share_one_fingerprint() {
    let out = run_pipeline("proj_checkout", TRACE, LOGS, None).unwrap();
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
    let out = run_pipeline("proj_checkout", TRACE, LOGS, None).unwrap();
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
    let out = run_pipeline("proj_checkout", TRACE, LOGS, None).unwrap();
    for b in &out.bundles {
        assert_eq!(b.schema_version, "bundle-v0-poc");
        assert!(!b.missing_evidence.is_empty(), "missing evidence must be explicit");
        assert!(b.canonical_hash.as_deref().unwrap_or("").starts_with("sha256:"));
        assert!(b.trigger.dispatch_eligible);
    }
}

#[test]
fn output_is_deterministic() {
    let a = run_pipeline("proj_checkout", TRACE, LOGS, None).unwrap();
    let b = run_pipeline("proj_checkout", TRACE, LOGS, None).unwrap();
    let hashes = |o: &evidence_loop_poc::PipelineOutput| {
        o.bundles.iter().map(|x| x.canonical_hash.clone().unwrap()).collect::<Vec<_>>()
    };
    assert_eq!(hashes(&a), hashes(&b), "identical input must produce identical canonical hashes");
}

#[test]
fn deploy_adjacency_escalates_trigger_with_strong_sha_edge() {
    let out = run_pipeline("proj_checkout", TRACE, LOGS, Some(DEPLOYS)).unwrap();
    for b in &out.bundles {
        // Fixture deploy 1.42.0 finished ~9 minutes before the first error and
        // its vcs_sha matches the service's vcs.ref.head.revision.
        assert_eq!(b.trigger.r#type, "deploy_adjacent_regression");
        let deploy_edge = b
            .edges
            .iter()
            .find(|e| e.r#type == "deploy_preceded_issue")
            .expect("deploy edge present");
        assert_eq!(deploy_edge.strength, "strong", "matching deployed SHA must upgrade to strong");
        assert!(
            serde_json::to_string(b).unwrap().contains("\"release\":\"1.42.0\""),
            "the adjacent deploy (not the older control deploy) must be in the bundle"
        );
    }
}

#[test]
fn old_deploy_alone_does_not_escalate_trigger() {
    let only_old_deploy = r#"{ "deploys": [ {
        "schema": "parallax.deploy.v0", "project_id": "proj_checkout",
        "release": "1.41.9", "vcs_sha": "0000aaaa1111bbbb2222cccc3333dddd4444eeee",
        "environment": "production", "status": "succeeded",
        "finished_at_unix_nano": "1781340000000000000" } ] }"#;
    let out = run_pipeline("proj_checkout", TRACE, LOGS, Some(only_old_deploy)).unwrap();
    for b in &out.bundles {
        assert_eq!(
            b.trigger.r#type, "new_fingerprint",
            "a deploy outside the 30-minute window must not escalate the trigger"
        );
        assert!(
            b.missing_evidence.iter().any(|m| m.contains("no deploy event within adjacency window")),
            "missing deploy adjacency must be explicit"
        );
    }
}

#[test]
fn recurrence_kernel_returns_all_three_verdicts() {
    let fix_deploy = 1_000_000u128;
    let window = 700u128; // watch window in nanos (scale-free for the kernel)

    // Recurred: events after the fix deploy, inside the window.
    let verdict = reconcile_recurrence(fix_deploy, &[1_000_500, 1_000_600], window, 1_001_000);
    assert_eq!(verdict, RecurrenceVerdict::Recurred { events_in_window: 2 });

    // Silent: window fully elapsed, no events after the deploy.
    let verdict = reconcile_recurrence(fix_deploy, &[999_000], window, 1_001_000);
    assert_eq!(verdict, RecurrenceVerdict::Silent);

    // WindowOpen: no recurrence yet, but the horizon has not covered the window.
    let verdict = reconcile_recurrence(fix_deploy, &[999_000], window, 1_000_300);
    assert_eq!(verdict, RecurrenceVerdict::WindowOpen);

    // Pre-deploy events never count as recurrence.
    let verdict = reconcile_recurrence(fix_deploy, &[999_999, 1_000_000], window, 1_001_000);
    assert_eq!(verdict, RecurrenceVerdict::Silent, "events at or before the deploy are the original bug");
}

#[test]
fn autonomy_budget_is_earned_from_outcomes() {
    let rows = outcome_rows();

    // backend_error: 12 rows, accepted_rate (8 + 0.5*2)/12 = 0.75, zero
    // reverts — earns L2. One recurrence (rate 0.083 > 0.05) blocks L3:
    // draft-PR autonomy must be earned with clean recurrence windows.
    let backend = compute_budget(&rows, "backend_error");
    assert_eq!(backend.max_level, "L2_propose_patch");
    assert_eq!(backend.basis.runs, 12);
    assert_eq!(backend.basis.accepted_rate, "0.750");

    // frontend_error: only 2 rows — below the L2 sample floor, stays L1.
    let frontend = compute_budget(&rows, "frontend_error");
    assert_eq!(frontend.max_level, "L1_diagnose");

    // Unknown class: no history, starts at L1.
    let unknown = compute_budget(&rows, "deploy_regression");
    assert_eq!(unknown.max_level, "L1_diagnose");
}

#[test]
fn clean_history_earns_l3_and_redaction_failure_caps_at_l1() {
    let clean: Vec<OutcomeRow> = (0..12)
        .map(|i| OutcomeRow {
            outcome_id: format!("fixout_clean_{i}"),
            failure_class: "backend_error".to_string(),
            autonomy_level: "L2".to_string(),
            classification: "accepted".to_string(),
            merged: true,
            recurrence: "no".to_string(),
            redaction_failure: false,
            cited_edge_types: vec!["error_in_span".to_string()],
        })
        .collect();
    assert_eq!(compute_budget(&clean, "backend_error").max_level, "L3_draft_pr");

    // One redaction failure in the window caps the whole class at L1
    // regardless of accept rate.
    let mut tainted = clean;
    tainted[0].redaction_failure = true;
    assert_eq!(compute_budget(&tainted, "backend_error").max_level, "L1_diagnose");
}

#[test]
fn fix_candidate_payload_is_deterministic_and_carries_budget() {
    let out = run_pipeline("proj_checkout", TRACE, LOGS, Some(DEPLOYS)).unwrap();
    let rows = outcome_rows();
    let bundle = &out.bundles[0];

    let make = || {
        build_fix_candidate(
            bundle,
            compute_budget(&rows, "backend_error"),
            vec!["cargo test -p checkout".to_string()],
        )
    };
    let a = serde_json::to_string(&make()).unwrap();
    let b = serde_json::to_string(&make()).unwrap();
    assert_eq!(a, b, "identical bundle + history must produce identical payloads");

    let candidate = make();
    assert_eq!(candidate.event_type, "parallax.fix_candidate.v0");
    assert_eq!(candidate.trigger, "deploy_adjacent_regression");
    assert_eq!(candidate.autonomy_budget.max_level, "L2_propose_patch");
    assert_eq!(
        candidate.idempotency_key,
        format!("iss_{}:{}", bundle.anchor.fingerprint, bundle.bundle_id)
    );
    assert!(candidate.canonical_bundle_hash.starts_with("sha256:"));
    assert!(
        !a.contains("\"nodes\""),
        "payload must reference the bundle, never inline its nodes"
    );
}

#[test]
fn learner_weights_edges_by_outcome_citations() {
    let report = compute_edge_weights(&outcome_rows());

    let deploy = &report.weights["deploy_preceded_issue"];
    let temporal = &report.weights["temporal_proximity"];
    let deploy_lift: f64 = deploy.lift.parse().unwrap();
    let temporal_lift: f64 = temporal.lift.parse().unwrap();

    assert!(
        deploy_lift > 1.0,
        "edge type cited by accepted fixes must gain weight, got {deploy_lift}"
    );
    assert!(
        temporal_lift < 1.0,
        "edge type cited only by rejected/inconclusive fixes must lose weight, got {temporal_lift}"
    );
    assert!(deploy_lift > temporal_lift);

    // Dated-row rule: the adjustment references the outcomes that caused it.
    assert_eq!(report.basis_outcome_ids.len(), 14);
    assert!(report.basis_outcome_ids.contains(&"fixout_001".to_string()));

    // Deterministic.
    let a = serde_json::to_string(&compute_edge_weights(&outcome_rows())).unwrap();
    let b = serde_json::to_string(&compute_edge_weights(&outcome_rows())).unwrap();
    assert_eq!(a, b);
}

#[test]
fn appending_an_outcome_row_changes_the_policy() {
    // The loop-closure property: outcome rows demonstrably alter a policy
    // decision through the same public API — no special learning path.
    let mut rows = outcome_rows();
    assert_eq!(compute_budget(&rows, "backend_error").max_level, "L2_propose_patch");

    rows.push(OutcomeRow {
        outcome_id: "fixout_015".to_string(),
        failure_class: "backend_error".to_string(),
        autonomy_level: "L2".to_string(),
        classification: "reverted".to_string(),
        merged: true,
        recurrence: "yes".to_string(),
        redaction_failure: false,
        cited_edge_types: vec!["error_in_span".to_string()],
    });

    assert_eq!(
        compute_budget(&rows, "backend_error").max_level,
        "L1_diagnose",
        "one reverted fix must demote the class until trust is re-earned"
    );
}
