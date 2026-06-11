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

use evidence_loop_poc::bound::{bound_bundle, BoundReport, DEFAULT_MAX_TOKENS};
use evidence_loop_poc::budget::{compute_budget, OutcomeRow, OutcomesData};
use evidence_loop_poc::bundle::Node;
use evidence_loop_poc::deploy::{reconcile_recurrence, RecurrenceVerdict};
use evidence_loop_poc::derive::ErrorSource;
use evidence_loop_poc::dispatch::build_fix_candidate;
use evidence_loop_poc::learn::{apply_edge_weights, compute_edge_weights};
use evidence_loop_poc::rollup::{spike_check, RollupStore, DEFAULT_BUCKET_NANOS};
use evidence_loop_poc::run_pipeline;
use evidence_loop_poc::spike::{
    frequency_spike, SpikeVerdict, DEFAULT_EWMA_ALPHA, DEFAULT_K, DEFAULT_MIN_BASELINE_BUCKETS,
    DEFAULT_MIN_COUNT,
};

fn synthetic_event(fingerprint: &str, time_unix_nano: u128) -> evidence_loop_poc::derive::ErrorEvent {
    evidence_loop_poc::derive::ErrorEvent {
        source: ErrorSource::LogRecord,
        error_type: "log_error".to_string(),
        message: "synthetic".to_string(),
        stacktrace: None,
        trace_id: "00000000000000000000000000000000".to_string(),
        span_id: "0000000000000000".to_string(),
        time_unix_nano: time_unix_nano.to_string(),
        service_name: "checkout".to_string(),
        fingerprint: fingerprint.to_string(),
    }
}

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

#[test]
fn learned_weights_reorder_bundle_edges() {
    let out = run_pipeline("proj_checkout", TRACE, LOGS, Some(DEPLOYS)).unwrap();
    let report = compute_edge_weights(&outcome_rows());
    let mut bundle = out
        .bundles
        .into_iter()
        .find(|b| b.edges.iter().any(|e| e.r#type == "same_fingerprint"))
        .expect("the exception bundle");

    apply_edge_weights(&mut bundle, &report);

    // Fixture lifts: error_in_span 1.231 > deploy_preceded_issue 1.190 >
    // log_in_trace 1.164 > uncited types at the default 1.0
    // (same_fingerprint, span_child_of — alphabetical tie-break puts
    // span_child_of last).
    assert_eq!(bundle.edges.first().unwrap().r#type, "error_in_span");
    assert_eq!(bundle.edges.last().unwrap().r#type, "span_child_of");
    let positions: Vec<&str> = bundle.edges.iter().map(|e| e.r#type.as_str()).collect();
    let pos_of = |t: &str| positions.iter().position(|x| *x == t).unwrap();
    assert!(pos_of("deploy_preceded_issue") < pos_of("log_in_trace"));
    assert!(pos_of("log_in_trace") < pos_of("same_fingerprint"));

    // Hash is recomputed over the reordered artifact and stays deterministic.
    let h1 = bundle.canonical_hash.clone().unwrap();
    apply_edge_weights(&mut bundle, &report);
    assert_eq!(bundle.canonical_hash.unwrap(), h1);
    assert!(h1.starts_with("sha256:"));
}

#[test]
fn emitted_artifacts_validate_against_published_schemas() {
    let bundle_schema: serde_json::Value =
        serde_json::from_str(include_str!("../schema/evidence-bundle.v0-poc.schema.json")).unwrap();
    let candidate_schema: serde_json::Value =
        serde_json::from_str(include_str!("../schema/fix-candidate.v0.schema.json")).unwrap();
    let bundle_validator = jsonschema::validator_for(&bundle_schema).unwrap();
    let candidate_validator = jsonschema::validator_for(&candidate_schema).unwrap();

    let out = run_pipeline("proj_checkout", TRACE, LOGS, Some(DEPLOYS)).unwrap();
    let rows = outcome_rows();
    for bundle in &out.bundles {
        let bundle_json = serde_json::to_value(bundle).unwrap();
        assert!(
            bundle_validator.is_valid(&bundle_json),
            "bundle must validate: {:?}",
            bundle_validator.iter_errors(&bundle_json).map(|e| e.to_string()).collect::<Vec<_>>()
        );

        let candidate = build_fix_candidate(
            bundle,
            compute_budget(&rows, "backend_error"),
            vec!["cargo test -p checkout".to_string()],
        );
        let candidate_json = serde_json::to_value(&candidate).unwrap();
        assert!(
            candidate_validator.is_valid(&candidate_json),
            "fix_candidate must validate: {:?}",
            candidate_validator
                .iter_errors(&candidate_json)
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
        );

        // Negative control: dropping a safety-critical field must fail
        // validation — the schema actually guards the contract.
        let mut crippled = bundle_json.clone();
        crippled.as_object_mut().unwrap().remove("redaction_report");
        assert!(!bundle_validator.is_valid(&crippled), "schema must require redaction_report");
    }
}

#[test]
fn frequency_spike_kernel_verdicts() {
    let check = |counts: &[u64]| {
        frequency_spike(counts, DEFAULT_K, DEFAULT_EWMA_ALPHA, DEFAULT_MIN_BASELINE_BUCKETS, DEFAULT_MIN_COUNT)
    };

    // Flat traffic: never a spike.
    assert_eq!(check(&[10, 10, 10, 10, 10, 10, 10]), SpikeVerdict::NoSpike);

    // 10x jump over a ~2.5/bucket baseline: spike.
    match check(&[2, 3, 2, 3, 2, 3, 30]) {
        SpikeVerdict::Spike { latest, baseline_ewma_milli } => {
            assert_eq!(latest, 30);
            assert!(baseline_ewma_milli < 4_000, "baseline must stay near the quiet rate");
        }
        other => panic!("expected spike, got {other:?}"),
    }

    // Near-zero baseline + tiny absolute count: the min-count floor holds
    // (4 events after a quiet stretch is noise, not an incident).
    assert_eq!(check(&[0, 0, 0, 0, 0, 0, 4]), SpikeVerdict::NoSpike);

    // Cold start: not enough history to claim a baseline.
    assert_eq!(check(&[5, 50]), SpikeVerdict::InsufficientBaseline);
}

#[test]
fn one_bundle_spans_frontend_and_backend_tiers() {
    // The cross-tier claim: a user-facing browser error and the backend
    // failure that caused it share one trace, so one bundle reconstructs the
    // whole path. Browser span (web-frontend) -> backend span (checkout),
    // exception in the backend, logs from both tiers.
    let trace = include_str!("../fixtures/crosstier/otlp-trace.json");
    let logs = include_str!("../fixtures/crosstier/otlp-logs.json");
    let out = run_pipeline("proj_checkout", trace, logs, None).unwrap();

    let backend_fp = out
        .error_events
        .iter()
        .find(|e| e.error_type == "sqlx::PoolTimedOut")
        .expect("backend exception derived")
        .fingerprint
        .clone();
    let bundle = out
        .bundles
        .iter()
        .find(|b| b.anchor.fingerprint == backend_fp)
        .expect("bundle anchored on the backend exception");

    // Span nodes from BOTH services in one bundle.
    let services: std::collections::BTreeSet<&str> = bundle
        .nodes
        .iter()
        .filter_map(|n| match n {
            Node::Span { service_name, .. } => Some(service_name.as_str()),
            _ => None,
        })
        .collect();
    assert!(services.contains("web-frontend") && services.contains("checkout"));

    // The cross-tier topology edge: backend SERVER span is a child of the
    // browser CLIENT span, linked strong via W3C trace context.
    assert!(
        bundle.edges.iter().any(|e| e.r#type == "span_child_of"
            && e.from == "span_b8c9d0e1f2030415"
            && e.to == "span_a1b2c3d4e5f60718"
            && e.strength == "strong"),
        "backend span must link to its browser parent span"
    );

    // One log window interleaves both tiers, service-tagged.
    let log_lines = bundle
        .nodes
        .iter()
        .find_map(|n| match n {
            Node::LogWindow { lines, .. } => Some(lines),
            _ => None,
        })
        .expect("log window present");
    assert!(log_lines.iter().any(|l| l.contains("[web-frontend]")));
    assert!(log_lines.iter().any(|l| l.contains("[checkout]")));

    // Cross-tier bundles still validate against the published schema.
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../schema/evidence-bundle.v0-poc.schema.json")).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let bundle_json = serde_json::to_value(bundle).unwrap();
    assert!(validator.is_valid(&bundle_json));
}

#[test]
fn rollup_buckets_fixture_events_and_feeds_the_spike_chain() {
    // Fixture events: all three land in the same 1-minute bucket; two
    // fingerprints (converged exception group of 2, plain ERROR log of 1).
    let out = run_pipeline("proj_checkout", TRACE, LOGS, None).unwrap();
    let store = RollupStore::from_events(&out.error_events, DEFAULT_BUCKET_NANOS);
    assert_eq!(store.counts.len(), 2);
    let totals: Vec<u64> = store.counts.keys().map(|fp| store.total(fp)).collect();
    assert_eq!(totals.iter().sum::<u64>(), 3);

    // Full Detect chain on a synthetic history: 6 quiet 1-minute buckets
    // (~3/min) then a 30-event burst → spike. Zero-filled gaps included.
    let base: u128 = 1_781_430_000_000_000_000;
    let mut events = Vec::new();
    for bucket in 0..6u128 {
        for i in 0..3u128 {
            events.push(synthetic_event("fp_burst", base + bucket * DEFAULT_BUCKET_NANOS + i));
        }
    }
    for i in 0..30u128 {
        events.push(synthetic_event("fp_burst", base + 6 * DEFAULT_BUCKET_NANOS + i));
    }
    let store = RollupStore::from_events(&events, DEFAULT_BUCKET_NANOS);
    assert_eq!(store.dense_series("fp_burst"), vec![3, 3, 3, 3, 3, 3, 30]);
    match spike_check(&store, "fp_burst", DEFAULT_K, DEFAULT_EWMA_ALPHA, DEFAULT_MIN_BASELINE_BUCKETS, DEFAULT_MIN_COUNT) {
        SpikeVerdict::Spike { latest, .. } => assert_eq!(latest, 30),
        other => panic!("expected spike from the rollup chain, got {other:?}"),
    }

    // Zero-fill: events only in buckets 0 and 5 → six-element series.
    let sparse = vec![
        synthetic_event("fp_sparse", base),
        synthetic_event("fp_sparse", base + 5 * DEFAULT_BUCKET_NANOS),
    ];
    let store = RollupStore::from_events(&sparse, DEFAULT_BUCKET_NANOS);
    assert_eq!(store.dense_series("fp_sparse"), vec![1, 0, 0, 0, 0, 1]);
}

#[test]
fn rollup_is_the_cost_vertex_in_miniature() {
    // 5,000 raw events vs their rollup: the Detector reads the aggregate, so
    // detection cost stays flat while raw volume grows. >100x size ratio.
    let base: u128 = 1_781_430_000_000_000_000;
    let events: Vec<_> = (0..5_000u128)
        .map(|i| synthetic_event("fp_volume", base + (i % 8) * DEFAULT_BUCKET_NANOS + i))
        .collect();
    let store = RollupStore::from_events(&events, DEFAULT_BUCKET_NANOS);

    let raw_bytes = serde_json::to_string(&events).unwrap().len();
    let rollup_bytes = serde_json::to_string(&store).unwrap().len();
    assert!(
        raw_bytes > rollup_bytes * 100,
        "rollup must compress detection input >100x (raw {raw_bytes}B vs rollup {rollup_bytes}B)"
    );
    assert_eq!(store.total("fp_volume"), 5_000);
}

#[test]
fn bundles_are_bounded_to_the_token_budget() {
    let out = run_pipeline("proj_checkout", TRACE, LOGS, Some(DEPLOYS)).unwrap();
    let mut bundle = out
        .bundles
        .into_iter()
        .find(|b| b.edges.iter().any(|e| e.r#type == "same_fingerprint"))
        .unwrap();

    // Inflate the log window far past the budget.
    let original_lines = bundle
        .nodes
        .iter()
        .find_map(|n| match n {
            Node::LogWindow { lines, .. } => Some(lines.len()),
            _ => None,
        })
        .unwrap();
    for node in bundle.nodes.iter_mut() {
        if let Node::LogWindow { lines, .. } = node {
            for i in 0..3000 {
                lines.push(format!(
                    "1781430543{i:06} INFO synthetic breadcrumb line {i} for token inflation"
                ));
            }
        }
    }

    let report = bound_bundle(&mut bundle, DEFAULT_MAX_TOKENS);
    assert!(report.before_tokens > DEFAULT_MAX_TOKENS);
    assert!(report.after_tokens <= DEFAULT_MAX_TOKENS, "bundle must fit the budget");
    assert!(report.dropped_log_lines > 0);
    assert!(
        bundle.missing_evidence.iter().any(|m| m.contains("token budget")),
        "every trim must be explicit missing evidence"
    );

    // Anchor-critical nodes survive: error events, spans, and the deploy are
    // never bounded out.
    let error_events =
        bundle.nodes.iter().filter(|n| matches!(n, Node::ErrorEvent { .. })).count();
    let deploys = bundle.nodes.iter().filter(|n| matches!(n, Node::Deploy { .. })).count();
    assert_eq!(error_events, 2);
    assert_eq!(deploys, 1);
    assert_eq!(original_lines, 4, "fixture sanity: log window starts with four lines");

    // Idempotent: bounding a bounded bundle drops nothing further.
    let second = bound_bundle(&mut bundle, DEFAULT_MAX_TOKENS);
    assert_eq!(
        second,
        BoundReport {
            before_tokens: second.before_tokens,
            after_tokens: second.after_tokens,
            dropped_log_lines: 0,
            truncated_stacktraces: 0
        }
    );
    assert_eq!(second.before_tokens, second.after_tokens);
}

#[test]
fn small_bundles_pass_through_bounding_unchanged() {
    let out = run_pipeline("proj_checkout", TRACE, LOGS, Some(DEPLOYS)).unwrap();
    let mut bundle = out.bundles.into_iter().next().unwrap();
    let hash_before = bundle.canonical_hash.clone();
    let report = bound_bundle(&mut bundle, DEFAULT_MAX_TOKENS);
    assert_eq!(report.dropped_log_lines, 0);
    assert_eq!(report.truncated_stacktraces, 0);
    assert_eq!(bundle.canonical_hash, hash_before, "under-budget bundle must be untouched");
}
