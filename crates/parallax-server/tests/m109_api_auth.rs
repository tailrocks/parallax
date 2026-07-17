//! Plan 109: optional API bearer token on GraphQL; health stays open.

use parallax_server::Config;

fn test_config(data_dir: &std::path::Path, token: Option<&str>) -> Config {
    let mut config = Config::default();
    config.server.api_port = 0;
    config.server.otlp_grpc_port = 0;
    config.server.otlp_http_port = 0;
    config.storage.data_dir = data_dir.to_string_lossy().into_owned();
    if let Some(token) = token {
        config.server.api_token = token.to_string();
    }
    config
}

fn env_token_present() -> bool {
    std::env::var("PARALLAX_API_TOKEN")
        .ok()
        .is_some_and(|value| {
            let trimmed = value.trim();
            !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("off")
        })
}

#[tokio::test(flavor = "multi_thread")]
async fn open_mode_allows_graphql_without_bearer() {
    if env_token_present() {
        eprintln!("skip open-mode auth fixture: PARALLAX_API_TOKEN is set in the environment");
        return;
    }
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = support::start(&test_config(tmp.path(), None))
        .await
        .expect("start");
    let client = reqwest::Client::new();
    let health = client
        .get(format!("http://{}/health", handle.api_addr))
        .send()
        .await
        .expect("health");
    assert!(health.status().is_success());
    let response = client
        .post(format!("http://{}/graphql", handle.api_addr))
        .json(&serde_json::json!({ "query": "{ __typename }" }))
        .send()
        .await
        .expect("graphql");
    assert!(response.status().is_success());
    handle.shutdown();
}

#[tokio::test(flavor = "multi_thread")]
async fn bearer_token_required_when_configured() {
    if env_token_present() {
        eprintln!("skip bearer auth fixture: PARALLAX_API_TOKEN is set in the environment");
        return;
    }
    let tmp = tempfile::tempdir().expect("tempdir");
    let token = "test-api-token-16chars";
    let handle = support::start(&test_config(tmp.path(), Some(token)))
        .await
        .expect("start");
    let client = reqwest::Client::new();
    let base = format!("http://{}", handle.api_addr);

    let health = client
        .get(format!("{base}/health"))
        .send()
        .await
        .expect("health");
    assert!(health.status().is_success(), "health stays open");

    let denied = client
        .post(format!("{base}/graphql"))
        .json(&serde_json::json!({ "query": "{ __typename }" }))
        .send()
        .await
        .expect("graphql without token");
    assert_eq!(denied.status(), reqwest::StatusCode::UNAUTHORIZED);
    let body = denied.text().await.expect("body");
    assert!(!body.contains(token), "token must not leak in 401 body");
    assert_eq!(body, "unauthorized");

    let wrong = client
        .post(format!("{base}/graphql"))
        .header(reqwest::header::AUTHORIZATION, "Bearer wrong-token-value!!")
        .json(&serde_json::json!({ "query": "{ __typename }" }))
        .send()
        .await
        .expect("graphql wrong token");
    assert_eq!(wrong.status(), reqwest::StatusCode::UNAUTHORIZED);

    let ok = client
        .post(format!("{base}/graphql"))
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
        .json(&serde_json::json!({ "query": "{ __typename }" }))
        .send()
        .await
        .expect("graphql with token");
    assert!(ok.status().is_success(), "status={}", ok.status());
    handle.shutdown();
}

#[path = "support/harness.rs"]
mod support;
