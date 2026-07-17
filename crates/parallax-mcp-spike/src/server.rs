//! Read-only stdio MCP server: two tools over GraphQL, nothing else.

use crate::gql::{self, GraphqlClient};
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, ContentBlock, JsonObject, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

const MCP_RESULT_MAX_BYTES: usize = 128 * 1024;
const MCP_ANCHOR_MAX_BYTES: usize = 256;

fn evidence_bundle_output_schema() -> Arc<JsonObject> {
    let schema = serde_json::from_str(include_str!(
        "../../../schema/evidence-bundle.v2.schema.json"
    ))
    .unwrap_or_default();
    Arc::new(schema)
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct IssueContextArgs {
    /// Issue fingerprint (canonical issue anchor).
    #[schemars(length(min = 1, max = 256))]
    pub fingerprint: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentSessionArgs {
    /// Invocation id whose agent-session projection to show.
    #[schemars(length(min = 1, max = 256))]
    pub invocation_id: String,
}

#[derive(Clone)]
pub(crate) struct SpikeServer {
    client: GraphqlClient,
    #[allow(
        dead_code,
        reason = "tool_handler macro reads the generated router field"
    )]
    tool_router: ToolRouter<Self>,
}

impl SpikeServer {
    pub(crate) fn new(base_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            client: GraphqlClient::new(base_url)?,
            tool_router: Self::tool_router(),
        })
    }
}

#[tool_router]
impl SpikeServer {
    #[tool(
        name = "parallax_issue_context",
        description = "Canonical evidence bundle for an issue fingerprint. Returns bounded Markdown in text content and the parsed canonical JSON in structuredContent (already redacted by the Parallax API).",
        annotations(
            title = "Read Parallax issue context",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        ),
        output_schema = evidence_bundle_output_schema()
    )]
    async fn parallax_issue_context(
        &self,
        Parameters(args): Parameters<IssueContextArgs>,
    ) -> Result<CallToolResult, McpError> {
        validate_anchor(&args.fingerprint)?;
        let bundle = gql::fetch_bundle(&self.client, Some(&args.fingerprint), None)
            .await
            .map_err(|_| safe_internal_error("bundle_unavailable"))?;
        bundle_tool_result(bundle)
    }

    #[tool(
        name = "parallax_agent_session_show",
        description = "Sanitized agent-session timeline for an invocation id (tool steps, token totals). Null/error when no agent spans were detected.",
        annotations(
            title = "Read Parallax agent session",
            read_only_hint = true,
            destructive_hint = false,
            idempotent_hint = true,
            open_world_hint = false
        )
    )]
    async fn parallax_agent_session_show(
        &self,
        Parameters(args): Parameters<AgentSessionArgs>,
    ) -> Result<CallToolResult, McpError> {
        validate_anchor(&args.invocation_id)?;
        let session = gql::fetch_agent_session(&self.client, &args.invocation_id)
            .await
            .map_err(|_| safe_internal_error("agent_session_unavailable"))?;
        // Mirror CLI JSON shape: compact re-serialize of the GraphQL object.
        let body = serde_json::to_string(&session)
            .map_err(|_| safe_internal_error("agent_session_invalid"))?;
        ensure_result_budget(&[body.len(), body.len()])?;
        let mut result = CallToolResult::structured(session);
        result.content = vec![ContentBlock::text(body)];
        Ok(result)
    }
}

fn safe_internal_error(code: &'static str) -> McpError {
    McpError::internal_error(
        "Parallax could not produce a safe MCP result",
        Some(json!({ "code": code })),
    )
}

fn validate_anchor(anchor: &str) -> Result<(), McpError> {
    if anchor.is_empty() || anchor.len() > MCP_ANCHOR_MAX_BYTES {
        return Err(McpError::invalid_params(
            "anchor must contain 1 to 256 UTF-8 bytes",
            Some(json!({ "code": "invalid_anchor" })),
        ));
    }
    Ok(())
}

fn ensure_result_budget(part_lengths: &[usize]) -> Result<(), McpError> {
    let total = part_lengths
        .iter()
        .fold(0usize, |total, length| total.saturating_add(*length));
    if total > MCP_RESULT_MAX_BYTES {
        return Err(safe_internal_error("result_too_large"));
    }
    Ok(())
}

fn bundle_tool_result(bundle: gql::BundleProjection) -> Result<CallToolResult, McpError> {
    // Parse exactly once for structuredContent. Keep the raw string for
    // comparison outside this function (check subcommand); do not re-serialize
    // the parsed value when comparing hashes.
    ensure_result_budget(&[bundle.json.len(), bundle.markdown.len()])?;
    let parsed: Value =
        serde_json::from_str(&bundle.json).map_err(|_| safe_internal_error("bundle_invalid"))?;
    if parsed.get("schema_version").and_then(Value::as_str) != Some("bundle-v2") {
        return Err(safe_internal_error("bundle_contract_mismatch"));
    }
    let embedded_hash = parsed.get("canonical_hash").and_then(Value::as_str);
    if embedded_hash.is_none() || embedded_hash != Some(bundle.canonical_hash.as_str()) {
        return Err(safe_internal_error("bundle_hash_mismatch"));
    }
    let mut result = CallToolResult::structured(parsed);
    result.content = vec![ContentBlock::text(bundle.markdown)];
    Ok(result)
}

#[tool_handler]
impl ServerHandler for SpikeServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_protocol_version(ProtocolVersion::V_2025_11_25)
            .with_instructions(
                "Parallax MCP SPIKE — read-only context adapter. \
                 Two tools only. Not a product surface. \
                 Calls http://127.0.0.1:4000/graphql (or PARALLAX_URL).",
            )
    }
}

/// Run the stdio MCP server until the client disconnects.
pub(crate) async fn run_stdio(base_url: String) -> anyhow::Result<()> {
    let server = SpikeServer::new(base_url)?;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advertises_tools_without_unapproved_capabilities() {
        let info = SpikeServer::new("http://127.0.0.1:4000".to_string())
            .expect("server")
            .get_info();
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
    fn tool_catalog_is_exact_and_inputs_are_closed() {
        let server = SpikeServer::new("http://127.0.0.1:4000".to_string()).expect("server");
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
        let result = bundle_tool_result(gql::BundleProjection {
            json: r#"{"schema_version":"bundle-v2","canonical_hash":"sha256-jcs:test"}"#
                .to_string(),
            markdown: "# Evidence".to_string(),
            canonical_hash: "sha256-jcs:test".to_string(),
        })
        .expect("valid bundle result");
        let encoded = serde_json::to_value(result).expect("serialize result");

        assert_eq!(encoded.get("_meta"), None);
        assert_eq!(
            encoded.get("structuredContent"),
            Some(&json!({
                "schema_version": "bundle-v2",
                "canonical_hash": "sha256-jcs:test"
            }))
        );
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
    fn bundle_hash_mismatch_fails_closed() {
        let error = bundle_tool_result(gql::BundleProjection {
            json: r#"{"schema_version":"bundle-v2","canonical_hash":"sha256-jcs:embedded"}"#
                .to_string(),
            markdown: "# Evidence".to_string(),
            canonical_hash: "sha256-jcs:projected".to_string(),
        })
        .expect_err("mismatched projection must fail");
        let encoded = serde_json::to_string(&error).expect("serialize MCP error");

        assert!(encoded.contains("bundle_hash_mismatch"));
        assert!(!encoded.contains("embedded"));
        assert!(!encoded.contains("projected"));
    }

    #[test]
    fn combined_result_budget_rejects_overflow_without_arithmetic_wrap() {
        ensure_result_budget(&[MCP_RESULT_MAX_BYTES]).expect("exact boundary");
        let error =
            ensure_result_budget(&[MCP_RESULT_MAX_BYTES, 1]).expect_err("over budget must fail");
        assert!(
            serde_json::to_string(&error)
                .expect("serialize error")
                .contains("result_too_large")
        );
        assert!(ensure_result_budget(&[usize::MAX, usize::MAX]).is_err());
    }

    #[test]
    fn anchor_validation_is_bounded_and_does_not_echo_input() {
        validate_anchor("a").expect("one byte");
        validate_anchor(&"a".repeat(MCP_ANCHOR_MAX_BYTES)).expect("exact boundary");
        for denied in [String::new(), "seeded-secret".repeat(30)] {
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
        let server = SpikeServer::new("http://127.0.0.1:4000".to_string()).expect("server");
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
    }
}
