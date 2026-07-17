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
use std::sync::{Arc, LazyLock};

const MCP_RESULT_MAX_BYTES: usize = 128 * 1024;
const MCP_ANCHOR_MAX_BYTES: usize = 256;
const EVIDENCE_READ_SCOPE: &str = "evidence:read";
const LOCAL_OPERATOR_SCOPES: &[&str] = &[EVIDENCE_READ_SCOPE];
const SUPPORTED_PROTOCOL_VERSIONS: &[ProtocolVersion] = &[
    ProtocolVersion::V_2024_11_05,
    ProtocolVersion::V_2025_03_26,
    ProtocolVersion::V_2025_06_18,
    ProtocolVersion::V_2025_11_25,
];
static BUNDLE_V2_VALIDATOR: LazyLock<Result<jsonschema::Validator, ()>> = LazyLock::new(|| {
    compile_schema(include_str!(
        "../../../schema/evidence-bundle.v2.schema.json"
    ))
});
static BUNDLE_V1_VALIDATOR: LazyLock<Result<jsonschema::Validator, ()>> = LazyLock::new(|| {
    compile_schema(include_str!(
        "../../../schema/evidence-bundle.v1.schema.json"
    ))
});

fn evidence_bundle_output_schema() -> Arc<JsonObject> {
    Arc::new(parse_output_schema(include_str!(
        "../../../schema/evidence-bundle.v2.schema.json"
    )))
}

fn parse_output_schema(raw: &str) -> JsonObject {
    serde_json::from_str(raw).unwrap_or_else(|_| {
        // `{}` accepts every value. `{"not": {}}` accepts none, so corrupted
        // checked-in schema content cannot silently widen the tool contract.
        JsonObject::from_iter([("not".to_string(), json!({}))])
    })
}

fn compile_schema(raw: &str) -> Result<jsonschema::Validator, ()> {
    let schema = Value::Object(parse_output_schema(raw));
    jsonschema::draft202012::options()
        .should_validate_formats(true)
        .should_ignore_unknown_formats(false)
        .build(&schema)
        .map_err(|_| ())
}

fn validate_bundle_contract(bundle: &Value) -> Result<(), McpError> {
    let validator = BUNDLE_V2_VALIDATOR
        .as_ref()
        .map_err(|()| safe_internal_error("bundle_schema_invalid"))?;
    if !validator.is_valid(bundle) {
        return Err(safe_internal_error("bundle_contract_mismatch"));
    }
    let data = bundle
        .get("data")
        .ok_or_else(|| safe_internal_error("bundle_contract_mismatch"))?;
    let data_validator = BUNDLE_V1_VALIDATOR
        .as_ref()
        .map_err(|()| safe_internal_error("bundle_schema_invalid"))?;
    if !data_validator.is_valid(data) {
        return Err(safe_internal_error("bundle_contract_mismatch"));
    }
    Ok(())
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct LocalAuthorization {
    principal: &'static str,
    scopes: &'static [&'static str],
}

impl LocalAuthorization {
    fn from_explicit_cli_trust() -> Self {
        Self {
            principal: "local-operator",
            scopes: LOCAL_OPERATOR_SCOPES,
        }
    }
}

fn require_evidence_read(authorization: &LocalAuthorization) -> Result<(), McpError> {
    if authorization.principal != "local-operator" || authorization.scopes != LOCAL_OPERATOR_SCOPES
    {
        return Err(McpError::invalid_request(
            "local MCP authorization denied",
            Some(json!({ "code": "authorization_denied" })),
        ));
    }
    Ok(())
}

#[derive(Clone)]
pub(crate) struct SpikeServer {
    client: GraphqlClient,
    authorization: LocalAuthorization,
    #[allow(
        dead_code,
        reason = "tool_handler macro reads the generated router field"
    )]
    tool_router: ToolRouter<Self>,
}

impl SpikeServer {
    fn new(base_url: String, authorization: LocalAuthorization) -> anyhow::Result<Self> {
        Ok(Self {
            client: GraphqlClient::new(base_url)?,
            authorization,
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
        let guard = crate::audit::ToolCallGuard::start(
            crate::audit::AuditTool::IssueContext,
            self.authorization.principal,
            self.authorization.scopes,
        );
        if let Err(error) = require_evidence_read(&self.authorization) {
            guard.finish_err(&crate::audit::error_code(&error));
            return Err(error);
        }
        if let Err(error) = validate_anchor(&args.fingerprint) {
            guard.finish_err(&crate::audit::error_code(&error));
            return Err(error);
        }
        let bundle = match gql::fetch_bundle(&self.client, Some(&args.fingerprint), None).await {
            Ok(bundle) => bundle,
            Err(error) => {
                let mapped = map_fetch_error(error, "bundle_unavailable");
                guard.finish_err(&crate::audit::error_code(&mapped));
                return Err(mapped);
            }
        };
        match bundle_tool_result(bundle) {
            Ok(result) => {
                let bytes = serde_json::to_vec(&result).map(|v| v.len()).unwrap_or(0);
                guard.finish_ok(bytes);
                Ok(result)
            }
            Err(error) => {
                guard.finish_err(&crate::audit::error_code(&error));
                Err(error)
            }
        }
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
        ),
        output_schema = rmcp::handler::server::tool::schema_for_type::<gql::AgentSessionProjection>()
    )]
    async fn parallax_agent_session_show(
        &self,
        Parameters(args): Parameters<AgentSessionArgs>,
    ) -> Result<CallToolResult, McpError> {
        let guard = crate::audit::ToolCallGuard::start(
            crate::audit::AuditTool::AgentSessionShow,
            self.authorization.principal,
            self.authorization.scopes,
        );
        if let Err(error) = require_evidence_read(&self.authorization) {
            guard.finish_err(&crate::audit::error_code(&error));
            return Err(error);
        }
        if let Err(error) = validate_anchor(&args.invocation_id) {
            guard.finish_err(&crate::audit::error_code(&error));
            return Err(error);
        }
        let session = match gql::fetch_agent_session(&self.client, &args.invocation_id).await {
            Ok(session) => session,
            Err(error) => {
                let mapped = map_fetch_error(error, "agent_session_unavailable");
                guard.finish_err(&crate::audit::error_code(&mapped));
                return Err(mapped);
            }
        };
        // Mirror CLI JSON shape: compact re-serialize of the GraphQL object.
        let Ok(body) = serde_json::to_string(&session) else {
            let error = safe_internal_error("agent_session_invalid");
            guard.finish_err(&crate::audit::error_code(&error));
            return Err(error);
        };
        if let Err(error) = ensure_already_redacted(&[&body], "agent_session_redaction_mismatch") {
            guard.finish_err(&crate::audit::error_code(&error));
            return Err(error);
        }
        let Ok(structured) = serde_json::to_value(session) else {
            let error = safe_internal_error("agent_session_invalid");
            guard.finish_err(&crate::audit::error_code(&error));
            return Err(error);
        };
        let mut result = CallToolResult::structured(structured.clone());
        result.content = vec![ContentBlock::text(body)];
        let result = if ensure_result_budget(&result).is_ok() {
            result
        } else {
            let resource = format!(
                "parallax://evidence/agent-session/{id}",
                id = args.invocation_id
            );
            match bounded_summary_result(
                "agent_session",
                json!({
                    "invocation_id": args.invocation_id,
                    "truncated_fields": true,
                    "omitted": ["full_timeline_body"],
                }),
                &[resource],
            ) {
                Ok(summary) => summary,
                Err(error) => {
                    guard.finish_err(&crate::audit::error_code(&error));
                    return Err(error);
                }
            }
        };
        let bytes = serde_json::to_vec(&result).map(|v| v.len()).unwrap_or(0);
        guard.finish_ok(bytes);
        Ok(result)
    }
}

fn safe_internal_error(code: &'static str) -> McpError {
    McpError::internal_error(
        "Parallax could not produce a safe MCP result",
        Some(json!({ "code": code })),
    )
}

fn map_fetch_error(error: gql::FetchError, unavailable_code: &'static str) -> McpError {
    match error {
        gql::FetchError::NotFound(kind) => McpError::resource_not_found(
            "Parallax evidence was not found",
            Some(json!({ "code": format!("{kind}_not_found") })),
        ),
        gql::FetchError::Other(_) => safe_internal_error(unavailable_code),
    }
}

fn validate_anchor(anchor: &str) -> Result<(), McpError> {
    if anchor.is_empty()
        || anchor.len() > MCP_ANCHOR_MAX_BYTES
        || parallax_evidence::sanitize_text(anchor) != anchor
    {
        return Err(McpError::invalid_params(
            "anchor must contain 1 to 256 safe UTF-8 bytes",
            Some(json!({ "code": "invalid_anchor" })),
        ));
    }
    Ok(())
}

fn ensure_result_budget(result: &CallToolResult) -> Result<(), McpError> {
    let serialized =
        serde_json::to_vec(result).map_err(|_| safe_internal_error("result_invalid"))?;
    if serialized.len() > MCP_RESULT_MAX_BYTES {
        return Err(safe_internal_error("result_too_large"));
    }
    Ok(())
}

/// When a full tool payload exceeds the wire budget, return a bounded summary
/// plus approved resource references (plan 112 residual) instead of fail-closed
/// empty output. Secrets stay out of text; only structural ids/hashes/refs.
fn bounded_summary_result(
    kind: &'static str,
    summary: Value,
    resource_uris: &[String],
) -> Result<CallToolResult, McpError> {
    let structured = json!({
        "truncated": true,
        "kind": kind,
        "byte_budget": MCP_RESULT_MAX_BYTES,
        "summary": summary,
        "resources": resource_uris.iter().map(|uri| json!({
            "uri": uri,
            "mimeType": "application/json",
        })).collect::<Vec<_>>(),
    });
    let text =
        serde_json::to_string(&structured).map_err(|_| safe_internal_error("summary_invalid"))?;
    ensure_already_redacted(&[&text], "summary_redaction_mismatch")?;
    let mut result = CallToolResult::structured(structured);
    result.content = vec![ContentBlock::text(text)];
    ensure_result_budget(&result)?;
    Ok(result)
}

fn ensure_already_redacted(parts: &[&str], code: &'static str) -> Result<(), McpError> {
    if parts
        .iter()
        .any(|part| parallax_evidence::sanitize_text(part) != *part)
    {
        return Err(safe_internal_error(code));
    }
    Ok(())
}

fn bundle_tool_result(bundle: gql::BundleProjection) -> Result<CallToolResult, McpError> {
    // Parse exactly once for structuredContent. Keep the raw string for
    // comparison outside this function (check subcommand); do not re-serialize
    // the parsed value when comparing hashes.
    ensure_already_redacted(
        &[&bundle.json, &bundle.markdown],
        "bundle_redaction_mismatch",
    )?;
    let parsed: Value =
        serde_json::from_str(&bundle.json).map_err(|_| safe_internal_error("bundle_invalid"))?;
    validate_bundle_contract(&parsed)?;
    let embedded_hash = parsed.get("canonical_hash").and_then(Value::as_str);
    let recomputed_hash = crate::check::recompute_canonical_hash(&bundle.json)
        .map_err(|_| safe_internal_error("bundle_hash_invalid"))?;
    if embedded_hash.is_none()
        || embedded_hash != Some(bundle.canonical_hash.as_str())
        || embedded_hash != Some(recomputed_hash.as_str())
    {
        return Err(safe_internal_error("bundle_hash_mismatch"));
    }
    let schema = parsed.get("schema").cloned().unwrap_or(Value::Null);
    let contract_version = parsed
        .get("contract_version")
        .cloned()
        .unwrap_or(Value::Null);
    let mut result = CallToolResult::structured(parsed);
    result.content = vec![ContentBlock::text(bundle.markdown)];
    if ensure_result_budget(&result).is_ok() {
        return Ok(result);
    }
    // Oversized path: keep only hash + approved resource refs, never the full
    // markdown/JSON body (plan 112 residual ship gate).
    let resource = format!(
        "parallax://evidence/bundle/{hash}",
        hash = bundle.canonical_hash
    );
    bounded_summary_result(
        "evidence_bundle",
        json!({
            "canonical_hash": bundle.canonical_hash,
            "schema": schema,
            "contract_version": contract_version,
            "omitted": ["markdown", "full_json"],
        }),
        &[resource],
    )
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

    async fn initialize(
        &self,
        request: rmcp::model::InitializeRequestParams,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<rmcp::model::InitializeResult, McpError> {
        if !SUPPORTED_PROTOCOL_VERSIONS.contains(&request.protocol_version) {
            return Err(McpError::invalid_request(
                "unsupported MCP protocol version",
                Some(json!({ "code": "unsupported_protocol_version" })),
            ));
        }
        context.peer.set_peer_info(request.clone());
        Ok(self
            .get_info()
            .with_protocol_version(request.protocol_version))
    }
}

/// Run the stdio MCP server until the client disconnects.
pub(crate) async fn run_stdio(base_url: String) -> anyhow::Result<()> {
    let server = SpikeServer::new(base_url, LocalAuthorization::from_explicit_cli_trust())?;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests;
