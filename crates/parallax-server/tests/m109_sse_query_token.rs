//! Plan 109 regression coverage for the SSE `access_token` query fallback
//! (defect #4, 2026-09-12 verification run): browsers cannot attach headers to
//! `EventSource`, so the live-tail routes accept the bearer token via the query
//! string — scoped to the stream routes only, never to GraphQL.

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

/// SSE responses start streaming; the caller only inspects response headers.
async fn stream_head(url: String) -> reqwest::Response {
    reqwest::Client::new()
        .get(url)
        .send()
        .await
        .expect("stream request")
}

#[tokio::test(flavor = "multi_thread")]
async fn stream_routes_accept_access_token_query_param() {
    if env_token_present() {
        eprintln!("skip query-token fixture: PARALLAX_API_TOKEN is set in the environment");
        return;
    }
    let tmp = tempfile::tempdir().expect("tempdir");
    let token = "sse-query-token-16ch";
    let handle = support::start(&test_config(tmp.path(), Some(token)))
        .await
        .expect("start");
    let base = format!("http://{}", handle.api_addr);

    // Without any credential the stream route answers 401 like every other
    // protected route.
    let denied = stream_head(format!("{base}/v1/logs/stream")).await;
    assert_eq!(denied.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert_eq!(
        denied
            .headers()
            .get(reqwest::header::WWW_AUTHENTICATE)
            .and_then(|value| value.to_str().ok()),
        Some("Bearer"),
        "401 must advertise the bearer scheme"
    );

    // The query fallback unlocks the stream route.
    let ok = stream_head(format!("{base}/v1/logs/stream?access_token={token}")).await;
    assert!(ok.status().is_success(), "status={}", ok.status());
    let ok_traces = stream_head(format!("{base}/v1/traces/stream?access_token={token}")).await;
    assert!(ok_traces.status().is_success(), "status={}", ok_traces.status());

    // A wrong query token is still a 401.
    let wrong = stream_head(format!("{base}/v1/logs/stream?access_token=wrong-token-value")).await;
    assert_eq!(wrong.status(), reqwest::StatusCode::UNAUTHORIZED);

    handle.shutdown();
}

#[tokio::test(flavor = "multi_thread")]
async fn query_token_is_percent_decoded_and_scoped_to_streams() {
    if env_token_present() {
        eprintln!("skip query-token fixture: PARALLAX_API_TOKEN is set in the environment");
        return;
    }
    let tmp = tempfile::tempdir().expect("tempdir");
    // The token contains characters that MUST be percent-encoded in a query
    // string; the server has to decode `a%26b%2Fc` back to `a&b/c`.
    let token = "a&b/c";
    let handle = support::start(&test_config(tmp.path(), Some(token)))
        .await
        .expect("start");
    let base = format!("http://{}", handle.api_addr);

    let ok = stream_head(format!("{base}/v1/logs/stream?access_token=a%26b%2Fc")).await;
    assert!(
        ok.status().is_success(),
        "percent-encoded query token must decode, status={}",
        ok.status()
    );

    // The fallback is opt-in per route: GraphQL must not accept it.
    let leaked = reqwest::Client::new()
        .post(format!("{base}/graphql?access_token={token}"))
        .json(&serde_json::json!({ "query": "{ __typename }" }))
        .send()
        .await
        .expect("graphql with query token");
    assert_eq!(
        leaked.status(),
        reqwest::StatusCode::UNAUTHORIZED,
        "query-token fallback must stay scoped to the stream routes"
    );

    handle.shutdown();
}

#[tokio::test(flavor = "multi_thread")]
async fn stream_routes_still_accept_bearer_header() {
    if env_token_present() {
        eprintln!("skip query-token fixture: PARALLAX_API_TOKEN is set in the environment");
        return;
    }
    let tmp = tempfile::tempdir().expect("tempdir");
    let token = "bearer-on-stream-ok";
    let handle = support::start(&test_config(tmp.path(), Some(token)))
        .await
        .expect("start");
    let base = format!("http://{}", handle.api_addr);

    let ok = reqwest::Client::new()
        .get(format!("{base}/v1/logs/stream"))
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
        .send()
        .await
        .expect("stream with bearer");
    assert!(ok.status().is_success(), "status={}", ok.status());

    handle.shutdown();
}

#[path = "support/harness.rs"]
mod support;
