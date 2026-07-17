use super::*;
use parallax_model::ErrorSource;
use serde_json::json;

fn span(attributes: Value) -> SpanRow {
    SpanRow {
        ts_nanos: 100,
        service: "checkout".into(),
        trace_id: "abababababababababababababababab".into(),
        span_id: "cdcdcdcdcdcdcdcd".into(),
        parent_span_id: Some("external-wrapper".into()),
        name: "ignored span name".into(),
        kind: "SPAN_KIND_INTERNAL".into(),
        status_code: "STATUS_CODE_UNSET".into(),
        status_message: String::new(),
        duration_ns: 50,
        invocation_id: Some("inv-1".into()),
        session_id: None,
        scope_name: "tests".into(),
        events: None,
        links: Value::Null,
        attributes,
        resource: json!({
            "service.version": "1.2.3",
            "vcs.ref.head.revision": "deadbeef",
            "test.configuration.os": "linux"
        }),
    }
}

fn failure(span: &SpanRow, error_type: &str, fingerprint: &str) -> ErrorEventRow {
    ErrorEventRow {
        ts_nanos: span.ts_nanos,
        service: span.service.clone(),
        fingerprint: fingerprint.into(),
        error_type: error_type.into(),
        message: "boom".into(),
        stacktrace: Some("top".into()),
        source: ErrorSource::SpanException,
        trace_id: span.trace_id.clone(),
        span_id: span.span_id.clone(),
        attributes: Value::Null,
    }
}

#[test]
fn only_named_test_markers_are_recognized_and_parenting_is_allowed() {
    assert_eq!(
        derive_test_span(&span(json!({})), None).expect("derive"),
        None
    );
    assert!(
        derive_test_span(&span(json!({"test.case.name": "works"})), None)
            .expect("derive")
            .is_some()
    );
}

#[test]
fn derives_identity_result_and_resource_dimensions() {
    let derived = derive_test_span(
        &span(json!({
            "test.case.id": "stable-42",
            "test.code_reference": "suite::charges_card",
            "test.case.name": "charges card",
            "test.suite.name": "checkout",
            "test.case.parameters": [
                {"name": "browser", "value": "webkit", "excluded": false},
                {"name": "seed", "value": "42", "excluded": true}
            ],
            "test.attempt.ordinal": 2,
            "test.case.result.status": "pass",
            "test.configuration.browser": "webkit"
        })),
        None,
    )
    .expect("derive")
    .expect("test span");
    assert_eq!(derived.case.explicit_id.as_deref(), Some("stable-42"));
    assert_eq!(derived.case.suite_path, vec!["checkout"]);
    assert_eq!(derived.variant.parameters.len(), 1);
    assert_eq!(derived.result.key.invocation_id, "inv-1");
    assert_eq!(derived.result.key.attempt.get(), 2);
    assert_eq!(derived.result.status, TestStatus::Passed);
    assert_eq!(derived.result.ended_at_nanos, 150);
    assert_eq!(derived.case.first_seen_nanos, 150);
    assert_eq!(derived.result.service_version.as_deref(), Some("1.2.3"));
    assert_eq!(
        derived.result.vcs_head_revision.as_deref(),
        Some("deadbeef")
    );
    assert_eq!(derived.result.configuration.dimensions.len(), 2);
}

#[test]
fn string_and_array_parameter_encodings_match_but_maps_fail() {
    let parameters = json!([{"name": "browser", "value": "webkit", "excluded": false}]);
    let direct = derive_test_span(
        &span(json!({"test.case.name": "test", "test.case.parameters": parameters})),
        None,
    )
    .expect("direct")
    .expect("test");
    let encoded = derive_test_span(
        &span(json!({
            "test.case.name": "test",
            "test.case.parameters": parameters.to_string()
        })),
        None,
    )
    .expect("encoded")
    .expect("test");
    assert_eq!(direct.variant.key, encoded.variant.key);
    assert_eq!(
        derive_test_span(
            &span(json!({"test.case.name": "test", "test.case.parameters": {"x": 1}})),
            None
        ),
        Err(TestSpanDerivationError::InvalidParameters)
    );
}

#[test]
fn attempt_must_be_a_positive_json_integer() {
    for value in [json!(0), json!(-1), json!(1.5), json!("2"), json!(u64::MAX)] {
        assert_eq!(
            derive_test_span(
                &span(json!({"test.case.name": "test", "test.attempt.ordinal": value})),
                None
            ),
            Err(TestSpanDerivationError::InvalidAttempt)
        );
    }
}

#[test]
fn status_taxonomy_is_explicit_and_fail_closed() {
    for (result, kind, expected) in [
        ("pass", None, TestStatus::Passed),
        ("skipped", None, TestStatus::Skipped),
        ("fail", Some("assertion_failure"), TestStatus::Failed),
        ("fail", Some("harness_error"), TestStatus::Broken),
        ("mystery", None, TestStatus::Unknown),
        ("fail", Some("mystery"), TestStatus::Unknown),
    ] {
        let mut attributes = json!({
            "test.case.name": "test",
            "test.case.result.status": result
        });
        if let Some(kind) = kind {
            attributes["test.case.failure.kind"] = json!(kind);
        }
        assert_eq!(classify_test_status(&span(attributes), None), expected);
    }
    let row = span(json!({"test.case.name": "test", "test.case.result.status": "fail"}));
    assert_eq!(
        classify_test_status(&row, Some(&failure(&row, "AssertionFailedError", "fp"))),
        TestStatus::Failed
    );
    assert_eq!(
        classify_test_status(&row, Some(&failure(&row, "RuntimeError", "fp"))),
        TestStatus::Broken
    );
}

#[test]
fn copies_only_the_same_span_production_fingerprint() {
    let row = span(json!({
        "test.case.name": "test",
        "test.case.result.status": "fail",
        "test.case.failure.kind": "assertion_failure"
    }));
    let error = failure(&row, "AssertionError", "production-fp");
    let linked = derive_test_span(&row, Some(&error))
        .expect("derive")
        .expect("test");
    assert_eq!(
        linked.result.failure_fingerprint.as_deref(),
        Some("production-fp")
    );

    let mut mismatched = error;
    mismatched.span_id = "other".into();
    let unlinked = derive_test_span(&row, Some(&mismatched))
        .expect("derive")
        .expect("test");
    assert_eq!(unlinked.result.failure_fingerprint, None);
}

#[test]
fn recognized_tests_reject_missing_result_identity_and_overflow() {
    let mut row = span(json!({"test.case.name": "test"}));
    row.invocation_id = None;
    assert_eq!(
        derive_test_span(&row, None),
        Err(TestSpanDerivationError::MissingInvocationId)
    );
    row.invocation_id = Some("inv".into());
    row.ts_nanos = u128::MAX;
    row.duration_ns = 1;
    assert_eq!(
        derive_test_span(&row, None),
        Err(TestSpanDerivationError::EndTimeOverflow)
    );
}
