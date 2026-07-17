use crate::resolvers::test_support::{context_with_memory, error_messages};
use crate::{build_schema, execute};
use parallax_storage::model::{
    TestAttempt, TestCaseIdentitySource, TestCaseKey, TestCaseRecord, TestConfiguration,
    TestResultKey, TestResultRecord, TestStatus, TestVariantKey, TestVariantRecord, TraceId,
};
use parallax_test_support::builders::MemoryStore;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::test]
async fn test_explorer_exposes_rollup_and_native_span_links() {
    let context = context_with_memory(Arc::new(MemoryStore::new())).await;
    let case_key = TestCaseKey::from_str(&format!("tc1:{}", "a".repeat(64))).expect("case");
    let variant_key =
        TestVariantKey::from_str(&format!("tv1:{}", "b".repeat(64))).expect("variant");
    context
        .metadata
        .upsert_test_case(&TestCaseRecord {
            key: case_key.clone(),
            identity_source: TestCaseIdentitySource::Explicit,
            explicit_id: Some("stable".into()),
            code_reference: Some("suite::test".into()),
            suite_path: vec!["checkout".into()],
            name: "charges card".into(),
            first_seen_nanos: 1_000_000,
            last_seen_nanos: 2_000_000,
        })
        .await
        .expect("case");
    context
        .metadata
        .upsert_test_variant(&TestVariantRecord {
            key: variant_key.clone(),
            case_key: case_key.clone(),
            parameters: Vec::new(),
            first_seen_nanos: 1_000_000,
            last_seen_nanos: 2_000_000,
        })
        .await
        .expect("variant");
    for (attempt, status, trace) in [
        (1, TestStatus::Failed, "abababababababababababababababab"),
        (2, TestStatus::Passed, "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd"),
    ] {
        context
            .metadata
            .upsert_test_result(&TestResultRecord {
                key: TestResultKey {
                    variant_key: variant_key.clone(),
                    invocation_id: "inv-1".into(),
                    attempt: TestAttempt::new(attempt).expect("attempt"),
                },
                status,
                trace_id: TraceId::from_str(trace).expect("trace"),
                span_id: format!("{attempt:016x}"),
                started_at_nanos: u128::from(attempt) * 1_000_000,
                ended_at_nanos: u128::from(attempt + 1) * 1_000_000,
                service: "checkout".into(),
                service_version: Some("1.2.3".into()),
                vcs_head_revision: Some("deadbeef".into()),
                configuration: TestConfiguration::default(),
                failure_fingerprint: (status == TestStatus::Failed).then(|| "fp".into()),
            })
            .await
            .expect("result");
    }
    let request = juniper::http::GraphQLRequest::new(
        r#"{ testCases(service: "checkout", status: FLAKY_PASS, limit: 5000) { hasMore items { caseKey variantKey name invocationId rollup attemptCount lastResult { attempt status traceId spanId serviceVersion failureFingerprint } } } }"#.into(),
        None, None,
    );
    let response = execute(&build_schema(), &context, request).await;
    let json = serde_json::to_value(response).expect("json");
    assert!(error_messages(&json).is_empty(), "test explorer: {json}");
    let item = json.pointer("/data/testCases/items/0").expect("item");
    assert_eq!(item["rollup"], "FLAKY_PASS");
    assert_eq!(item["attemptCount"], 2);
    assert_eq!(item["lastResult"]["attempt"], 2);
    assert_eq!(
        item["lastResult"]["traceId"],
        "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd"
    );

    let detail_request = juniper::http::GraphQLRequest::new(
        format!(
            r#"{{ testCase(caseKey: "{}", variantLimit: 5000, resultLimit: 5000) {{ caseKey identitySource name variants {{ variantKey history {{ invocationId attempt status traceId failureFingerprint }} }} }} }}"#,
            case_key.as_str()
        ),
        None,
        None,
    );
    let detail = serde_json::to_value(execute(&build_schema(), &context, detail_request).await)
        .expect("detail json");
    assert!(error_messages(&detail).is_empty(), "test detail: {detail}");
    let detail_case = detail.pointer("/data/testCase").expect("detail case");
    assert_eq!(detail_case["caseKey"], case_key.as_str());
    assert_eq!(
        detail_case["variants"][0]["variantKey"],
        variant_key.as_str()
    );
    assert_eq!(detail_case["identitySource"], "EXPLICIT");
    assert_eq!(detail_case["variants"][0]["history"][0]["attempt"], 2);
    assert_eq!(detail_case["variants"][0]["history"][1]["attempt"], 1);
}

#[tokio::test]
async fn test_explorer_rejects_invalid_configuration_filter() {
    let context = context_with_memory(Arc::new(MemoryStore::new())).await;
    let request = juniper::http::GraphQLRequest::new(
        r#"{ testCases(configuration: {key: "unsafe", value: "linux"}) { hasMore } }"#.into(),
        None,
        None,
    );
    let json =
        serde_json::to_value(execute(&build_schema(), &context, request).await).expect("json");
    let messages = error_messages(&json);
    assert!(
        messages
            .iter()
            .any(|message| message.contains("test configuration filter is invalid")),
        "expected configuration filter rejection, got {messages:?}"
    );

    let invalid_detail = juniper::http::GraphQLRequest::new(
        r#"{ testCase(caseKey: "not-versioned") { caseKey } }"#.into(),
        None,
        None,
    );
    let invalid_detail =
        serde_json::to_value(execute(&build_schema(), &context, invalid_detail).await)
            .expect("detail json");
    assert!(
        error_messages(&invalid_detail)
            .iter()
            .any(|message| message == "invalid test case key"),
        "expected invalid case key rejection: {invalid_detail}"
    );
}
