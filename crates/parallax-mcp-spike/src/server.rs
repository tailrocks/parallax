//! Read-only stdio MCP server: two tools over GraphQL, nothing else.

use crate::gql::{self, GraphqlClient};
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ContentBlock, Meta, ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
    transport::stdio,
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct IssueContextArgs {
    /// Issue fingerprint (canonical issue anchor).
    pub fingerprint: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AgentSessionArgs {
    /// Run id whose agent-session projection to show.
    pub run_id: String,
}

#[derive(Clone)]
#[allow(dead_code)] // tool_router is used by #[tool_handler] macro
pub struct SpikeServer {
    client: GraphqlClient,
    tool_router: ToolRouter<Self>,
}

impl SpikeServer {
    pub fn new(base_url: String) -> Self {
        Self {
            client: GraphqlClient::new(base_url),
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl SpikeServer {
    #[tool(
        name = "parallax_issue_context",
        description = "Canonical evidence bundle for an issue fingerprint. Returns bounded Markdown in text content and the parsed canonical JSON in structuredContent (already redacted by the Parallax API)."
    )]
    async fn parallax_issue_context(
        &self,
        Parameters(args): Parameters<IssueContextArgs>,
    ) -> Result<CallToolResult, McpError> {
        let bundle = gql::fetch_bundle(&self.client, Some(&args.fingerprint), None)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(bundle_tool_result(bundle))
    }

    #[tool(
        name = "parallax_agent_session_show",
        description = "Sanitized agent-session timeline for a run id (tool steps, token totals). Null/error when no agent spans were detected."
    )]
    async fn parallax_agent_session_show(
        &self,
        Parameters(args): Parameters<AgentSessionArgs>,
    ) -> Result<CallToolResult, McpError> {
        let session = gql::fetch_agent_session(&self.client, &args.run_id)
            .await
            .map_err(|e| McpError::internal_error(e.to_string(), None))?;
        // Mirror CLI JSON shape: compact re-serialize of the GraphQL object.
        let body = serde_json::to_string(&session).unwrap_or_else(|_| "{}".into());
        let parsed: Value = serde_json::from_str(&body).unwrap_or(json!({}));
        let mut result = CallToolResult::structured(parsed);
        result.content = vec![ContentBlock::text(body)];
        Ok(result)
    }
}

fn bundle_tool_result(bundle: gql::BundleProjection) -> CallToolResult {
    // Parse exactly once for structuredContent. Keep the raw string for
    // comparison outside this function (check subcommand); do not re-serialize
    // the parsed value when comparing hashes.
    let parsed: Value = serde_json::from_str(&bundle.json).unwrap_or(json!({}));
    let mut meta = Meta::new();
    meta.0.insert(
        "canonicalHash".to_string(),
        Value::String(bundle.canonical_hash.clone()),
    );
    // Spike-only: expose the raw canonical JSON string so a client/check can
    // compare byte-identity without relying on structuredContent re-serialization.
    meta.0
        .insert("rawJson".to_string(), Value::String(bundle.json));
    let mut result = CallToolResult::structured(parsed);
    result.content = vec![ContentBlock::text(bundle.markdown)];
    result.with_meta(Some(meta))
}

#[tool_handler]
impl ServerHandler for SpikeServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "Parallax MCP SPIKE — read-only context adapter. \
                 Two tools only. Not a product surface. \
                 Calls http://127.0.0.1:4000/graphql (or PARALLAX_URL).",
        )
    }
}

/// Run the stdio MCP server until the client disconnects.
pub async fn run_stdio(base_url: String) -> anyhow::Result<()> {
    let server = SpikeServer::new(base_url);
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
