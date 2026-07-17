use super::*;
use rmcp::{
    ClientHandler,
    model::{CallToolRequestParams, GetPromptRequestParams, ReadResourceRequestParams},
};

struct VersionedClient(ProtocolVersion);

impl ClientHandler for VersionedClient {
    fn get_info(&self) -> rmcp::model::ClientInfo {
        let mut info = rmcp::model::ClientInfo::default();
        info.protocol_version = self.0.clone();
        info
    }
}

fn test_server() -> SpikeServer {
    SpikeServer::new(
        "http://127.0.0.1:4000".to_string(),
        LocalAuthorization::from_explicit_cli_trust(),
    )
    .expect("server")
}

fn valid_bundle_projection() -> gql::BundleProjection {
    let mut value = json!({
        "schema_version": "bundle-v2",
        "bundle_id": "bundle-test",
        "schema_ref": "parallax/evidence/bundle-v2",
        "generated_at": "2026-07-17T00:00:00Z",
        "generator": "parallax/test",
        "project": "test",
        "window": {
            "from": "2026-07-17T00:00:00Z",
            "to": "2026-07-17T00:01:00Z"
        },
        "access": { "policy": "local-operator" },
        "data": {
            "schema_version": "bundle-v1",
            "generator": "parallax/test",
            "anchor": { "kind": "issue", "id": "fp-test" },
            "issue": null,
            "invocation": null,
            "latest_event": null,
            "trace": null,
            "metric_windows": [],
            "logs": [],
            "hypotheses": [],
            "missing_evidence": [],
            "redaction": { "policy": "test", "redacted_counts": {} },
            "bounded": {
                "max_tokens": 4000,
                "estimated_tokens": 0,
                "dropped_log_lines": 0,
                "truncated_stacktrace": false
            },
            "canonical_hash": null
        },
        "canonical_hash": null
    });
    let hash = crate::check::recompute_canonical_hash(&value.to_string()).expect("hash");
    value["canonical_hash"] = json!(hash);
    gql::BundleProjection {
        json: value.to_string(),
        markdown: "# Evidence".to_string(),
        canonical_hash: hash,
    }
}

async fn assert_protocol_rejected(protocol_version: ProtocolVersion) {
    let (server_transport, client_transport) = tokio::io::duplex(4096);
    let server = test_server();
    let server_task = tokio::spawn(async move { server.serve(server_transport).await });
    let error = VersionedClient(protocol_version)
        .serve(client_transport)
        .await
        .err()
        .expect("unreviewed protocol must fail");

    assert!(
        error
            .to_string()
            .contains("unsupported MCP protocol version")
    );
    let _server_result = server_task.await.expect("join server");
}

async fn assert_protocol_negotiates(protocol_version: ProtocolVersion) {
    let (server_transport, client_transport) = tokio::io::duplex(4096);
    let server = test_server();
    let server_task = tokio::spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow::Ok(())
    });
    let client = VersionedClient(protocol_version.clone())
        .serve(client_transport)
        .await
        .expect("reviewed protocol must initialize");

    assert_eq!(
        client
            .peer_info()
            .expect("server initialization info")
            .protocol_version,
        protocol_version
    );

    client.cancel().await.expect("cancel client");
    server_task
        .await
        .expect("join server")
        .expect("stop server");
}

async fn discover_tools_on_fresh_transport(server: SpikeServer) -> Vec<String> {
    let (server_transport, client_transport) = tokio::io::duplex(64 * 1024);
    let server_task = tokio::spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow::Ok(())
    });
    let client = ().serve(client_transport).await.expect("initialize client");
    let names = client
        .peer()
        .list_tools(None)
        .await
        .expect("tools/list response")
        .tools
        .into_iter()
        .map(|tool| tool.name.into_owned())
        .collect();

    client.cancel().await.expect("cancel client");
    server_task
        .await
        .expect("join server")
        .expect("stop server");
    names
}

fn assert_method_not_found(error: &rmcp::ServiceError) {
    assert!(
        matches!(
            error,
            rmcp::ServiceError::McpError(data)
                if data.code == rmcp::model::ErrorCode::METHOD_NOT_FOUND
        ),
        "expected protocol method-not-found, got {error:?}"
    );
}

#[test]
fn advertises_tools_without_unapproved_capabilities() {
    let info = test_server().get_info();
    assert_eq!(info.protocol_version, ProtocolVersion::V_2025_11_25);
    let capabilities = serde_json::to_value(info.capabilities).expect("serialize capabilities");

    assert!(capabilities.get("tools").is_some());
    for denied in [
        "completions",
        "elicitation",
        "experimental",
        "logging",
        "prompts",
        "resources",
        "roots",
        "sampling",
        "tasks",
    ] {
        assert_eq!(
            capabilities.get(denied),
            None,
            "{denied} must stay disabled"
        );
    }
}

#[test]
fn local_authorization_is_server_assigned_and_default_deny() {
    let trusted = LocalAuthorization::from_explicit_cli_trust();
    assert_eq!(trusted.principal, "local-operator");
    assert_eq!(trusted.scopes, [EVIDENCE_READ_SCOPE]);
    require_evidence_read(&trusted).expect("explicit local trust");

    let denied = LocalAuthorization {
        principal: "local-operator",
        scopes: &[],
    };
    let error = require_evidence_read(&denied).expect_err("missing scope must deny");
    assert_eq!(error.code, rmcp::model::ErrorCode::INVALID_REQUEST);
    let encoded = serde_json::to_string(&error).expect("serialize error");
    assert!(encoded.contains("authorization_denied"));
    assert!(!encoded.contains("evidence:read"));
}

#[test]
fn tool_catalog_is_exact_and_inputs_are_closed() {
    let server = test_server();
    let tools = server.tool_router.list_all();
    let names = tools
        .iter()
        .map(|tool| tool.name.as_ref())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        ["parallax_agent_session_show", "parallax_issue_context"]
    );
    for tool in tools {
        assert_eq!(
            tool.input_schema.get("additionalProperties"),
            Some(&Value::Bool(false)),
            "{} input must reject unknown fields",
            tool.name
        );
        assert_eq!(
            tool.input_schema
                .get("required")
                .and_then(Value::as_array)
                .map(Vec::len),
            Some(1),
            "{} input must require its anchor",
            tool.name
        );
        let properties = tool
            .input_schema
            .get("properties")
            .and_then(Value::as_object)
            .expect("input properties");
        let anchor = properties.values().next().expect("anchor property");
        assert_eq!(anchor.get("minLength"), Some(&json!(1)));
        assert_eq!(anchor.get("maxLength"), Some(&json!(256)));
        let annotations = tool.annotations.as_ref().expect("tool annotations");
        assert_eq!(annotations.read_only_hint, Some(true));
        assert_eq!(annotations.destructive_hint, Some(false));
        assert_eq!(annotations.idempotent_hint, Some(true));
        assert_eq!(annotations.open_world_hint, Some(false));
    }
}

#[test]
fn bundle_result_has_no_comparison_only_raw_metadata() {
    let projection = valid_bundle_projection();
    let expected: Value = serde_json::from_str(&projection.json).expect("bundle JSON");
    let result = bundle_tool_result(projection).expect("valid bundle result");
    let encoded = serde_json::to_value(result).expect("serialize result");

    assert_eq!(encoded.get("_meta"), None);
    assert_eq!(encoded.get("structuredContent"), Some(&expected));
}

#[test]
fn malformed_bundles_and_upstream_failures_return_stable_secret_free_errors() {
    let malformed = bundle_tool_result(gql::BundleProjection {
        json: r#"{"schema_version":"bundle-v1","secret":"seeded-secret"}"#.to_string(),
        markdown: "seeded-secret".to_string(),
        canonical_hash: "seeded-secret".to_string(),
    })
    .expect_err("wrong contract must fail");
    let upstream = safe_internal_error("bundle_unavailable");

    for error in [malformed, upstream] {
        let encoded = serde_json::to_string(&error).expect("serialize MCP error");
        assert!(!encoded.contains("seeded-secret"));
        assert!(encoded.contains("Parallax could not produce a safe MCP result"));
    }
}

#[test]
fn missing_evidence_maps_to_resource_not_found_without_echoing_anchor() {
    let error = map_fetch_error(gql::FetchError::NotFound("bundle"), "bundle_unavailable");
    let encoded = serde_json::to_string(&error).expect("serialize error");

    assert_eq!(error.code, rmcp::model::ErrorCode::RESOURCE_NOT_FOUND);
    assert!(encoded.contains("bundle_not_found"));
    assert!(!encoded.contains("seeded-secret-anchor"));
}

#[test]
fn bundle_hash_mismatch_fails_closed() {
    let mut projection = valid_bundle_projection();
    projection.canonical_hash = "sha256-jcs:projected".to_string();
    let error = bundle_tool_result(projection).expect_err("mismatched projection must fail");
    let encoded = serde_json::to_string(&error).expect("serialize MCP error");

    assert!(encoded.contains("bundle_hash_mismatch"));
    assert!(!encoded.contains("embedded"));
    assert!(!encoded.contains("projected"));
}

#[test]
fn matching_forged_bundle_hashes_fail_closed() {
    let forged = "sha256-jcs:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let mut projection = valid_bundle_projection();
    let mut value: Value = serde_json::from_str(&projection.json).expect("bundle JSON");
    value["canonical_hash"] = json!(forged);
    projection.json = value.to_string();
    projection.canonical_hash = forged.to_string();

    let error = bundle_tool_result(projection).expect_err("forged hash must fail");
    let encoded = serde_json::to_string(&error).expect("serialize MCP error");
    assert!(encoded.contains("bundle_hash_mismatch"));
    assert!(!encoded.contains(forged));
}

#[test]
fn correctly_hashed_nonconforming_bundle_fails_closed() {
    let mut projection = valid_bundle_projection();
    let mut value: Value = serde_json::from_str(&projection.json).expect("bundle JSON");
    value["unexpected"] = json!("seeded-secret");
    let hash = crate::check::recompute_canonical_hash(&value.to_string()).expect("hash");
    value["canonical_hash"] = json!(hash);
    projection.json = value.to_string();
    projection.canonical_hash = hash;

    let error = bundle_tool_result(projection).expect_err("schema violation must fail");
    let encoded = serde_json::to_string(&error).expect("serialize MCP error");
    assert!(encoded.contains("bundle_contract_mismatch"));
    assert!(!encoded.contains("seeded-secret"));
}

#[test]
fn correctly_hashed_nonconforming_v1_data_fails_closed() {
    let mut projection = valid_bundle_projection();
    let mut value: Value = serde_json::from_str(&projection.json).expect("bundle JSON");
    value["data"]
        .as_object_mut()
        .expect("data object")
        .remove("logs");
    let hash = crate::check::recompute_canonical_hash(&value.to_string()).expect("hash");
    value["canonical_hash"] = json!(hash);
    projection.json = value.to_string();
    projection.canonical_hash = hash;

    let error = bundle_tool_result(projection).expect_err("v1 schema violation must fail");
    assert!(
        serde_json::to_string(&error)
            .expect("serialize MCP error")
            .contains("bundle_contract_mismatch")
    );
}

#[test]
fn correctly_hashed_seeded_secret_fails_closed() {
    let mut projection = valid_bundle_projection();
    let mut value: Value = serde_json::from_str(&projection.json).expect("bundle JSON");
    let canary = "ghp_0123456789ABCDEFGHIJKLMNOPQRST";
    value["data"]["logs"] = json!([format!("token={canary}")]);
    let hash = crate::check::recompute_canonical_hash(&value.to_string()).expect("hash");
    value["canonical_hash"] = json!(hash);
    projection.json = value.to_string();
    projection.canonical_hash = hash;

    let error = bundle_tool_result(projection).expect_err("seeded secret must fail");
    let encoded = serde_json::to_string(&error).expect("serialize MCP error");
    assert!(encoded.contains("bundle_redaction_mismatch"));
    assert!(!encoded.contains(canary));
}

#[test]
fn agent_session_redaction_verifier_rejects_secret_patterns() {
    let canary = "sk-ant-0123456789ABCDEFGHIJKLMN";
    let error = ensure_already_redacted(&[canary], "agent_session_redaction_mismatch")
        .expect_err("seeded secret must fail");
    let encoded = serde_json::to_string(&error).expect("serialize MCP error");
    assert!(encoded.contains("agent_session_redaction_mismatch"));
    assert!(!encoded.contains(canary));
}

#[test]
fn result_budget_counts_json_escaping_on_the_wire() {
    let mut small = CallToolResult::structured(json!({}));
    small.content = vec![ContentBlock::text("bounded")];
    ensure_result_budget(&small).expect("small result");

    let escaping_text = "\n".repeat((MCP_RESULT_MAX_BYTES / 2) + 1);
    assert!(escaping_text.len() < MCP_RESULT_MAX_BYTES);
    let mut expanded = CallToolResult::structured(json!({}));
    expanded.content = vec![ContentBlock::text(escaping_text)];
    let error = ensure_result_budget(&expanded).expect_err("escaped wire result is over budget");
    assert!(
        serde_json::to_string(&error)
            .expect("serialize error")
            .contains("result_too_large")
    );
}

#[test]
fn anchor_validation_is_bounded_and_does_not_echo_input() {
    validate_anchor("a").expect("one byte");
    validate_anchor(&"a".repeat(MCP_ANCHOR_MAX_BYTES)).expect("exact boundary");
    for denied in [
        String::new(),
        "seeded-secret".repeat(30),
        "ghp_0123456789ABCDEFGHIJKLMNOPQRST".to_string(),
        "fp\u{1b}[31m".to_string(),
    ] {
        let error = validate_anchor(&denied).expect_err("invalid anchor");
        let encoded = serde_json::to_string(&error).expect("serialize error");
        assert!(encoded.contains("invalid_anchor"));
        if !denied.is_empty() {
            assert!(!encoded.contains(&denied));
        }
    }
}

#[test]
fn issue_context_advertises_the_canonical_bundle_v2_schema() {
    let server = test_server();
    let tools = server.tool_router.list_all();
    let tool = tools
        .iter()
        .find(|tool| tool.name == "parallax_issue_context")
        .expect("issue context tool");
    let schema = tool.output_schema.as_ref().expect("output schema");

    assert_eq!(
        schema.get("$id").and_then(Value::as_str),
        Some("https://github.com/tailrocks/parallax/schema/evidence-bundle.v2.schema.json")
    );
    assert_eq!(schema.get("type").and_then(Value::as_str), Some("object"));
    assert_eq!(
        schema.get("additionalProperties"),
        Some(&json!(false)),
        "canonical output discovery must stay closed-schema"
    );
}

#[test]
fn malformed_output_schema_falls_back_to_deny_all() {
    assert_eq!(
        Value::Object(parse_output_schema("not-json")),
        json!({ "not": {} })
    );
}

#[test]
fn agent_session_advertises_a_closed_output_schema() {
    let server = test_server();
    let tools = server.tool_router.list_all();
    let tool = tools
        .iter()
        .find(|tool| tool.name == "parallax_agent_session_show")
        .expect("agent session tool");
    let schema = tool.output_schema.as_ref().expect("output schema");

    assert_eq!(schema.get("type"), Some(&json!("object")));
    assert_eq!(schema.get("additionalProperties"), Some(&json!(false)));
    assert!(schema.get("required").and_then(Value::as_array).is_some());
}

#[tokio::test]
async fn wire_initialization_and_tools_list_match_the_locked_catalog() {
    let (server_transport, client_transport) = tokio::io::duplex(64 * 1024);
    let server = test_server();
    let server_task = tokio::spawn(async move {
        server.serve(server_transport).await?.waiting().await?;
        anyhow::Ok(())
    });
    let client = ().serve(client_transport).await.expect("initialize client");
    let peer = client.peer_info().expect("server initialization info");
    let listed = client
        .peer()
        .list_tools(None)
        .await
        .expect("tools/list response");

    assert_eq!(peer.protocol_version, ProtocolVersion::V_2025_11_25);
    assert_eq!(
        listed
            .tools
            .iter()
            .map(|tool| tool.name.as_ref())
            .collect::<Vec<_>>(),
        ["parallax_agent_session_show", "parallax_issue_context"]
    );
    assert_eq!(listed.next_cursor, None);
    let prompts = client
        .peer()
        .list_prompts(None)
        .await
        .expect("prompts/list response");
    assert!(prompts.prompts.is_empty());
    assert_eq!(prompts.next_cursor, None);
    let prompt_error = client
        .peer()
        .get_prompt(GetPromptRequestParams::new("forbidden"))
        .await
        .expect_err("prompt reads must remain unavailable");
    assert_method_not_found(&prompt_error);

    let resources = client
        .peer()
        .list_resources(None)
        .await
        .expect("resources/list response");
    assert!(resources.resources.is_empty());
    assert_eq!(resources.next_cursor, None);
    let resource_templates = client
        .peer()
        .list_resource_templates(None)
        .await
        .expect("resources/templates/list response");
    assert!(resource_templates.resource_templates.is_empty());
    assert_eq!(resource_templates.next_cursor, None);
    let resource_error = client
        .peer()
        .read_resource(ReadResourceRequestParams::new("parallax://forbidden"))
        .await
        .expect_err("resource reads must remain unavailable");
    assert_method_not_found(&resource_error);
    for denied in ["run_shell", "dashboard_create"] {
        let error = client
            .call_tool(CallToolRequestParams::new(denied))
            .await
            .expect_err("forbidden tool must not resolve");
        assert!(error.to_string().contains("tool not found"));
    }

    client.cancel().await.expect("cancel client");
    server_task
        .await
        .expect("join server")
        .expect("stop server");
}

#[tokio::test]
async fn wire_initialization_rejects_unreviewed_protocols() {
    let unknown = serde_json::from_value(json!("2099-01-01")).expect("protocol string");
    for protocol_version in [ProtocolVersion::V_2026_07_28, unknown] {
        assert_protocol_rejected(protocol_version).await;
    }
}

#[tokio::test]
async fn wire_initialization_negotiates_every_reviewed_protocol() {
    for protocol_version in SUPPORTED_PROTOCOL_VERSIONS {
        assert_protocol_negotiates(protocol_version.clone()).await;
    }
}

#[tokio::test]
async fn wire_discovery_does_not_depend_on_prior_session_state() {
    let server = test_server();
    let first = discover_tools_on_fresh_transport(server.clone()).await;
    let second = discover_tools_on_fresh_transport(server).await;

    assert_eq!(first, second);
    assert_eq!(
        first,
        ["parallax_agent_session_show", "parallax_issue_context"]
    );
}
