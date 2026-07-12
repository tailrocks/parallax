use parallax_server::Config;
use reqwest::header::HOST;

fn test_config(data_dir: &std::path::Path) -> Config {
    let mut config = Config::default();
    config.server.api_port = 0;
    config.server.otlp_grpc_port = 0;
    config.server.otlp_http_port = 0;
    config.storage.data_dir = data_dir.to_string_lossy().into_owned();
    config.limits.graphql_max_depth = 8;
    config.limits.graphql_max_complexity = 4;
    config
}

async fn graphql(
    client: &reqwest::Client,
    api_addr: std::net::SocketAddr,
    host: Option<&str>,
    query: &str,
) -> (reqwest::StatusCode, serde_json::Value) {
    let mut request = client
        .post(format!("http://{api_addr}/graphql"))
        .json(&serde_json::json!({ "query": query }));
    if let Some(host) = host {
        request = request.header(HOST, host);
    }
    let response = request.send().await.expect("graphql request");
    let status = response.status();
    let body = response.text().await.expect("graphql body");
    let json = serde_json::from_str(&body).unwrap_or_else(|_| serde_json::json!({ "body": body }));
    (status, json)
}

fn error_messages(json: &serde_json::Value) -> Vec<&str> {
    json.pointer("/errors")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|error| error.get("message").and_then(|message| message.as_str()))
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn m5_gates_limits_enforce_graphql_depth_complexity_and_host_guard() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let handle = support::start(&test_config(tmp.path()))
        .await
        .expect("server starts");
    let client = reqwest::Client::new();

    let (status, json) = graphql(&client, handle.api_addr, None, "{ version }").await;
    assert_eq!(status, reqwest::StatusCode::OK);
    assert_eq!(
        json.pointer("/data/version")
            .and_then(|value| value.as_str()),
        Some(env!("CARGO_PKG_VERSION"))
    );

    let deep_query = r"
        {
          __schema {
            types {
              fields {
                type {
                  ofType {
                    ofType {
                      ofType {
                        ofType {
                          ofType {
                            kind
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
    ";
    let (status, json) = graphql(&client, handle.api_addr, None, deep_query).await;
    assert_eq!(status, reqwest::StatusCode::OK);
    assert!(
        error_messages(&json)
            .iter()
            .any(|message| message.contains("depth")),
        "deep query rejected with depth error: {json}"
    );

    let wide_query = "{ a: version b: version c: version d: version e: version }";
    let (status, json) = graphql(&client, handle.api_addr, None, wide_query).await;
    assert_eq!(status, reqwest::StatusCode::OK);
    assert!(
        error_messages(&json)
            .iter()
            .any(|message| message.contains("field count")),
        "wide query rejected with complexity error: {json}"
    );

    let (status, _) = graphql(
        &client,
        handle.api_addr,
        Some("evil.example.com"),
        "{ version }",
    )
    .await;
    assert_eq!(status, reqwest::StatusCode::FORBIDDEN);

    let (status, json) = graphql(
        &client,
        handle.api_addr,
        Some("127.0.0.1:4000"),
        "{ version }",
    )
    .await;
    assert_eq!(status, reqwest::StatusCode::OK);
    assert!(
        error_messages(&json).is_empty(),
        "allowed host reaches GraphQL: {json}"
    );

    handle.shutdown();
}
#[path = "support/harness.rs"]
mod support;
