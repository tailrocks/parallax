use super::*;

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
    GraphqlClient::new("http://127.0.0.1:4000".to_string()).expect("loopback");
    GraphqlClient::new("http://127.42.0.9:4000".to_string()).expect("loopback range");
    let _remote = GraphqlClient::new("http://example.com:4000".to_string())
        .err()
        .expect("remote host");
    let _localhost = GraphqlClient::new("http://localhost:4000".to_string())
        .err()
        .expect("DNS names are not literal loopback");
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
