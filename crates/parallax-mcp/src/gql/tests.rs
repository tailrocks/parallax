use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[test]
fn response_byte_budget_accepts_boundary_and_rejects_overflow() {
    let mut body = vec![0; MCP_GRAPHQL_MAX_BYTES - 1];
    append_bounded(&mut body, &[1]).expect("exact boundary");
    assert_eq!(body.len(), MCP_GRAPHQL_MAX_BYTES);
    let before = body.len();

    assert!(append_bounded(&mut body, &[2]).is_err());
    assert_eq!(body.len(), before, "overflow must not partially append");
}

#[test]
fn declared_response_budget_rejects_oversize_before_streaming() {
    ensure_declared_response_budget(None).expect("chunked response");
    ensure_declared_response_budget(Some(MCP_GRAPHQL_MAX_BYTES as u64)).expect("exact boundary");
    assert!(ensure_declared_response_budget(Some((MCP_GRAPHQL_MAX_BYTES as u64) + 1)).is_err());
}

#[test]
fn client_constructor_enforces_loopback_origin() {
    GraphqlClient::new("http://127.0.0.1:4000".to_string(), None).expect("loopback");
    GraphqlClient::new("http://127.42.0.9:4000".to_string(), None).expect("loopback range");
    let _remote = GraphqlClient::new("http://example.com:4000".to_string(), None)
        .err()
        .expect("remote host");
    let _localhost = GraphqlClient::new("http://localhost:4000".to_string(), None)
        .err()
        .expect("DNS names are not literal loopback");
}

#[test]
fn empty_token_is_normalized_to_auth_disabled() {
    let with_value =
        GraphqlClient::new("http://127.0.0.1:4000".to_string(), Some("secret".to_string()))
            .expect("loopback");
    assert_eq!(with_value.api_token.as_deref(), Some("secret"));
    let empty =
        GraphqlClient::new("http://127.0.0.1:4000".to_string(), Some(String::new()))
            .expect("loopback");
    assert_eq!(empty.api_token, None, "empty token must match no-auth server config");
}

/// One-shot loopback HTTP stub: captures the request's `Authorization` header,
/// answers with a minimal GraphQL response, then closes.
async fn capture_authorization_header(
    config_token: Option<String>,
) -> anyhow::Result<Option<String>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await?;
        let mut buffer = Vec::new();
        let mut chunk = [0u8; 1024];
        loop {
            let read = socket.read(&mut chunk).await?;
            buffer.extend_from_slice(&chunk[..read]);
            if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
            if read == 0 {
                break;
            }
        }
        let text = String::from_utf8_lossy(&buffer);
        let authorization = text
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("authorization")
                    .then(|| value.trim().to_string())
            });
        let body = br#"{"data":{"__typename":"Query"}}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
            body.len()
        );
        socket.write_all(response.as_bytes()).await?;
        socket.write_all(body).await?;
        socket.shutdown().await?;
        anyhow::Ok(authorization)
    });

    let client = GraphqlClient::new(format!("http://{addr}"), config_token)?;
    client.graphql("{ __typename }", serde_json::json!({})).await?;
    server.await?
}

/// Defect #2 (2026-09-12 verification run): with a configured API token every
/// MCP-originated GraphQL request must present the bearer credential, and with
/// none configured it must send no `Authorization` header at all (matching the
/// server's open local-first mode).
#[tokio::test]
async fn graphql_requests_attach_bearer_exactly_when_configured() {
    let presented = capture_authorization_header(Some("secret".to_string()))
        .await
        .expect("configured client round-trip");
    assert_eq!(presented.as_deref(), Some("Bearer secret"));

    let absent = capture_authorization_header(None)
        .await
        .expect("open-mode client round-trip");
    assert_eq!(absent, None, "open mode must not send Authorization");
}

#[test]
fn agent_session_query_uses_real_graphql_braces() {
    assert!(AGENT_SESSION_QUERY.contains("steps {"));
    assert!(!AGENT_SESSION_QUERY.contains("{{"));
    assert!(!AGENT_SESSION_QUERY.contains("}}"));
}

#[test]
fn bundle_projection_requires_all_string_fields() {
    let valid = serde_json::json!({ "json": "{}", "markdown": "#", "canonicalHash": "h" });
    assert_eq!(required_string(&valid, "json").expect("json"), "{}");
    let _missing = required_string(&valid, "missing").expect_err("missing field");
    let _null =
        required_string(&serde_json::json!({ "json": null }), "json").expect_err("null field");
}

#[test]
fn agent_session_projection_rejects_unknown_fields() {
    let value = serde_json::json!({
        "rootSpanId": null,
        "totalInputTokens": "0",
        "totalOutputTokens": "0",
        "errorCount": 0,
        "truncated": false,
        "steps": [],
        "unexpected": "denied"
    });
    let _error =
        serde_json::from_value::<AgentSessionProjection>(value).expect_err("unknown field");
}
