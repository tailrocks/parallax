//! Plan 118 residual: Sentry envelope HTTP → spool → worker → Turso issue +
//! MemoryStore error_events, with ack ledger idempotency.

#![expect(clippy::expect_used, reason = "integration fixtures")]

use parallax_server::Config;
use std::time::Duration;

#[path = "support/harness.rs"]
mod support;

fn test_config(data_dir: &std::path::Path) -> Config {
    let mut config = Config::default();
    config.server.api_port = 0;
    config.server.otlp_grpc_port = 0;
    config.server.otlp_http_port = 0;
    config.storage.data_dir = data_dir.to_string_lossy().into_owned();
    config.sentry.enabled = true;
    config.sentry.project_id = "1".into();
    config.sentry.public_key = "test-public-key".into();
    config
}

fn fixture_envelope() -> Vec<u8> {
    std::fs::read(
        "crates/parallax-ingest/tests/fixtures/sentry/python-sdk-2.48-event.envelope",
    )
    .or_else(|_| {
        std::fs::read(
            "../parallax-ingest/tests/fixtures/sentry/python-sdk-2.48-event.envelope",
        )
    })
    .expect("python sdk fixture")
}

#[tokio::test(flavor = "multi_thread")]
async fn sentry_envelope_accepts_and_projects_issue() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = support::start(&test_config(tmp.path()))
        .await
        .expect("server starts");
    let client = reqwest::Client::new();
    let url = format!(
        "http://{}/api/1/envelope/",
        handle.api_addr
    );
    let body = fixture_envelope();
    let response = client
        .post(&url)
        .header(
            "x-sentry-auth",
            "Sentry sentry_version=7, sentry_client=sentry.python/2.48.0, sentry_key=test-public-key",
        )
        .header("content-type", "application/x-sentry-envelope")
        .body(body.clone())
        .send()
        .await
        .expect("post envelope");
    assert_eq!(response.status(), reqwest::StatusCode::OK, "body={}", response.text().await.unwrap_or_default());

    // Exact redelivery → 200 without replaying completed effects.
    let again = client
        .post(&url)
        .header(
            "x-sentry-auth",
            "Sentry sentry_version=7, sentry_client=sentry.python/2.48.0, sentry_key=test-public-key",
        )
        .header("content-type", "application/x-sentry-envelope")
        .body(body)
        .send()
        .await
        .expect("redelivery");
    assert_eq!(again.status(), reqwest::StatusCode::OK);

    // Wait for worker to project the issue.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let issues = handle
            .metadata
            .issues(10)
            .await
            .expect("list issues");
        if !issues.is_empty() {
            assert_eq!(issues.len(), 1, "exact redelivery must not invent a second issue");
            break;
        }
        assert!(
            tokio::time::Instant::now() <= deadline,
            "timed out waiting for Sentry-derived issue"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
