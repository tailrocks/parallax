use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use parallax_model::{
    ErrorEventRow, HistogramRow, LogRow, MetricExemplarRow, MetricPointRow, SpanRow,
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
        "run_id": "run", "scope_name": "scope", "events": null, "links": [],
        "attributes": {"http.request.method": "GET"}, "resource": {"host.name": "dev"}
    }))?;
    round_trip::<LogRow>(json!({
        "ts_nanos": 3, "event_name": "exception", "observed_ts_nanos": 4,
        "service": "svc", "severity_num": 17, "severity_text": "ERROR", "body": "failed",
        "trace_id": "trace", "span_id": "span", "run_id": null, "scope_name": "scope",
        "attributes": {}, "resource": {}
    }))?;
    round_trip::<MetricPointRow>(json!({
        "ts_nanos": 5, "service": "svc", "name": "requests", "value": 1.5,
        "is_monotonic": true, "run_id": null, "attributes": {"method": "GET"}
    }))?;
    round_trip::<MetricExemplarRow>(json!({
        "ts_nanos": 6, "service": "svc", "name": "latency", "value": 0.25,
        "trace_id": "trace", "span_id": "span", "run_id": "run", "attributes": {}
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
