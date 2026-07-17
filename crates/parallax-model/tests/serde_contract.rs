use proptest::prelude::*;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::str::FromStr;

use parallax_model::{
    ErrorEventRow, FlakyEvidence, FlakyState, HistogramRow, LogRow, MetricExemplarRow,
    MetricPointRow, SpanRow, TestFlakyStateRecord, TestVariantKey,
};

fn round_trip<T: Serialize + DeserializeOwned>(expected: Value) -> anyhow::Result<()> {
    let row: T = serde_json::from_value(expected.clone())?;
    anyhow::ensure!(
        serde_json::to_value(row)? == expected,
        "serde shape drifted"
    );
    Ok(())
}

#[test]
fn normalized_telemetry_serde_shapes_are_stable() -> anyhow::Result<()> {
    round_trip::<SpanRow>(json!({
        "ts_nanos": 1, "service": "svc", "trace_id": "trace", "span_id": "span",
        "parent_span_id": null, "name": "GET /", "kind": "server",
        "status_code": "error", "status_message": "failed", "duration_ns": 2,
        "invocation_id": "inv", "session_id": null, "scope_name": "scope", "events": null, "links": [],
        "attributes": {"http.request.method": "GET"}, "resource": {"host.name": "dev"}
    }))?;
    round_trip::<LogRow>(json!({
        "ts_nanos": 3, "event_name": "exception", "observed_ts_nanos": 4,
        "service": "svc", "severity_num": 17, "severity_text": "ERROR", "body": "failed",
        "trace_id": "trace", "span_id": "span", "invocation_id": null, "session_id": "sess",
        "scope_name": "scope",
        "attributes": {}, "resource": {}
    }))?;
    round_trip::<MetricPointRow>(json!({
        "ts_nanos": 5, "service": "svc", "name": "requests", "value": 1.5,
        "is_monotonic": true, "invocation_id": null, "attributes": {"method": "GET"}
    }))?;
    round_trip::<MetricExemplarRow>(json!({
        "ts_nanos": 6, "service": "svc", "name": "latency", "value": 0.25,
        "trace_id": "trace", "span_id": "span", "invocation_id": "run", "attributes": {}
    }))?;
    round_trip::<HistogramRow>(json!({
        "ts_nanos": 7, "service": "svc", "name": "latency", "count": 2, "sum": 0.75,
        "bucket_counts": [1, 1], "bounds": [0.1, 1.0], "attributes": {}
    }))?;
    round_trip::<ErrorEventRow>(json!({
        "ts_nanos": 8, "service": "svc", "fingerprint": "fp", "error_type": "Error",
        "message": "failed", "stacktrace": null, "source": "span_exception",
        "trace_id": "trace", "span_id": "span", "attributes": {}
    }))?;
    Ok(())
}

// Plan 103 residual: serialization compatibility — JSON fixpoint for flaky
// state records (versioned keys + evidence flags) under arbitrary booleans.
proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(64))]
    #[test]
    fn flaky_state_record_json_is_a_fixpoint(
        same_commit in any::<bool>(),
        intra in any::<bool>(),
        transitions in 0_u32..8,
        passes in 0_u32..8,
        consistent in any::<bool>(),
        state_ix in 0_u8..4,
        updated in 0_u128..1_000_000_000_000,
    ) {
        let variant = TestVariantKey::from_str(&format!("tv1:{}", "a".repeat(64)))
            .expect("variant");
        let state = match state_ix {
            0 => FlakyState::Healthy,
            1 => FlakyState::Flaky,
            2 => FlakyState::Fixed,
            _ => FlakyState::Broken,
        };
        let record = TestFlakyStateRecord {
            variant_key: variant,
            state,
            evidence: FlakyEvidence {
                same_commit_divergence: same_commit,
                intra_invocation_mix: intra,
                window_transition_count: transitions,
                consecutive_passes: passes,
                consistently_failing: consistent,
            },
            updated_at_nanos: updated,
        };
        let once = serde_json::to_value(&record).expect("serialize");
        let decoded: TestFlakyStateRecord =
            serde_json::from_value(once.clone()).expect("deserialize");
        let twice = serde_json::to_value(&decoded).expect("re-serialize");
        prop_assert_eq!(once, twice);
        prop_assert_eq!(decoded, record);
    }
}
