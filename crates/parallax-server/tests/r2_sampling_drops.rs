//! R2 live proof: sampling-policy readout plus a drop reason end-to-end.
//!
//! Boots an in-process server (real HTTP listeners, real OTLP + GraphQL),
//! asserts the declared policy over GraphQL, then drives a malformed OTLP
//! batch and asserts the `ingress_reject` drop reason is visible through the
//! same GraphQL path the pipeline page uses. No mocks on the new path.

#![allow(clippy::expect_used, clippy::panic, reason = "test fixture assertions")]

use parallax_server::Config;
use prost::Message;

fn test_config(data_dir: &std::path::Path) -> Config {
    let mut config = Config::default();
    config.server.api_port = 0;
    config.server.otlp_grpc_port = 0;
    config.server.otlp_http_port = 0;
    config.storage.data_dir = data_dir.to_string_lossy().into_owned();
    config
}

async fn graphql(client: &reqwest::Client, api_addr: &str, query: &str) -> serde_json::Value {
    client
        .post(format!("http://{api_addr}/graphql"))
        .json(&serde_json::json!({ "query": query }))
        .send()
        .await
        .expect("graphql request")
        .json()
        .await
        .expect("graphql json")
}

fn drop_count(drops: &serde_json::Value, signal: &str, reason: &str) -> u64 {
    drops
        .as_array()
        .expect("drops array")
        .iter()
        .find(|row| row["signal"] == signal && row["reason"] == reason)
        .unwrap_or_else(|| panic!("missing {signal}/{reason} in {drops}"))["count"]
        .as_str()
        .expect("count string")
        .parse()
        .expect("count parses")
}

#[tokio::test(flavor = "multi_thread")]
async fn sampling_policy_and_drop_reason_end_to_end() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = support::start(&test_config(tmp.path()))
        .await
        .expect("server starts");
    let client = reqwest::Client::new();

    // Policy readout: one row per signal, head keep-all by default.
    let policy = graphql(
        &client,
        &handle.api_addr.to_string(),
        "{ samplingPolicy { signal service rule rate enforcedBy description } }",
    )
    .await;
    assert_eq!(policy["errors"], serde_json::Value::Null, "{policy}");
    let rows = policy["data"]["samplingPolicy"].as_array().expect("rows");
    assert_eq!(rows.len(), 4, "{policy}");
    for signal in ["traces", "logs", "metrics", "sentry"] {
        let row = rows
            .iter()
            .find(|row| row["signal"] == signal)
            .unwrap_or_else(|| panic!("missing {signal} policy in {policy}"));
        assert_eq!(row["service"], serde_json::Value::Null, "{row}");
        assert_eq!(row["rule"], "head", "{row}");
        assert_eq!(row["rate"], 1.0, "{row}");
        assert_eq!(row["enforcedBy"], "server-ingest", "{row}");
    }

    // Queues start empty; a valid batch lands in accepted.
    let valid =
        parallax_proto::collector_trace::ExportTraceServiceRequest::default().encode_to_vec();
    let ok = client
        .post(format!("http://{}/v1/traces", handle.otlp_http_addr))
        .header("content-type", "application/x-protobuf")
        .body(valid)
        .send()
        .await
        .expect("valid post");
    assert_eq!(ok.status(), 200);
    let queues = graphql(
        &client,
        &handle.api_addr.to_string(),
        "{ ingestQueues { signal depth capacity highWater accepted } }",
    )
    .await;
    assert_eq!(queues["errors"], serde_json::Value::Null, "{queues}");
    let traces_queue = queues["data"]["ingestQueues"]
        .as_array()
        .expect("queues array")
        .iter()
        .find(|row| row["signal"] == "traces")
        .expect("traces queue");
    assert_eq!(traces_queue["capacity"], 256);
    let accepted: u64 = traces_queue["accepted"]
        .as_str()
        .expect("accepted string")
        .parse()
        .expect("accepted parses");
    assert!(accepted >= 1, "{traces_queue}");

    // A malformed batch is rejected (HTTP 400) and visible by reason.
    let before = graphql(
        &client,
        &handle.api_addr.to_string(),
        "{ ingestDrops { signal reason count detail } }",
    )
    .await;
    let before_rejects = drop_count(&before["data"]["ingestDrops"], "traces", "ingress_reject");
    let bad = client
        .post(format!("http://{}/v1/traces", handle.otlp_http_addr))
        .header("content-type", "application/x-protobuf")
        .body(vec![0xffu8, 0x01, 0x02])
        .send()
        .await
        .expect("bad post");
    assert_eq!(bad.status(), 400, "garbage protobuf must be rejected");
    let after = graphql(
        &client,
        &handle.api_addr.to_string(),
        "{ ingestDrops(signal: \"traces\") { signal reason count detail } }",
    )
    .await;
    assert_eq!(after["errors"], serde_json::Value::Null, "{after}");
    let after_rejects = drop_count(&after["data"]["ingestDrops"], "traces", "ingress_reject");
    assert_eq!(after_rejects, before_rejects + 1);
    // The signal filter keeps pipeline-global reasons alongside.
    assert!(
        after["data"]["ingestDrops"]
            .as_array()
            .expect("filtered array")
            .iter()
            .any(|row| row["signal"].is_null() && row["reason"] == "unsupported_metric"),
        "{after}"
    );

    handle.shutdown();
}

#[path = "support/harness.rs"]
mod support;
