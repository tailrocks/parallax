//! Thin GraphQL client against a running Parallax API.
//!
use rmcp::schemars;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// MCP returns both structured JSON and compatibility text, so use a tighter
/// canonical bundle budget than the HTTP API's 10,000-token default.
pub(crate) const MCP_BUNDLE_MAX_TOKENS: u32 = 4_000;
pub(crate) const MCP_GRAPHQL_MAX_BYTES: usize = 1024 * 1024;
const AGENT_SESSION_QUERY: &str = r#"query AgentSession($invocationId: String!) {
  agentSession(invocationId: $invocationId) {
    rootSpanId totalInputTokens totalOutputTokens errorCount truncated
    steps {
      spanId traceId kind name startNanos durationNs isError
      genAiOperation inputTokens outputTokens
    }
  }
}"#;

#[derive(Clone)]
pub(crate) struct GraphqlClient {
    base_url: String,
    http: reqwest::Client,
}

impl GraphqlClient {
    pub(crate) fn new(base_url: String) -> anyhow::Result<Self> {
        let base_url = normalize_local_base_url(&base_url)?;
        Ok(Self {
            base_url,
            http: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(5))
                .timeout(Duration::from_secs(30))
                .redirect(reqwest::redirect::Policy::none())
                .build()?,
        })
    }

    pub(crate) async fn graphql(&self, query: &str, variables: Value) -> anyhow::Result<Value> {
        let mut response = self
            .http
            .post(format!("{}/graphql", self.base_url))
            .header("Host", host_header_for(&self.base_url))
            .json(&serde_json::json!({ "query": query, "variables": variables }))
            .send()
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "cannot reach Parallax at {} ({e}); is `parallax serve` running?",
                    self.base_url
                )
            })?
            .error_for_status()?;
        let mut body = Vec::with_capacity(
            response
                .content_length()
                .and_then(|length| usize::try_from(length).ok())
                .unwrap_or_default()
                .min(MCP_GRAPHQL_MAX_BYTES),
        );
        while let Some(chunk) = response.chunk().await? {
            append_bounded(&mut body, &chunk)?;
        }
        let response: Value = serde_json::from_slice(&body)?;
        if let Some(errors) = response.get("errors")
            && !errors.as_array().is_none_or(Vec::is_empty)
        {
            anyhow::bail!("graphql error: {errors}");
        }
        Ok(response)
    }
}

pub(crate) fn normalize_local_base_url(raw: &str) -> anyhow::Result<String> {
    let url = reqwest::Url::parse(raw)
        .map_err(|error| anyhow::anyhow!("invalid MCP API URL: {error}"))?;
    let local_host = matches!(
        url.host_str(),
        Some("localhost" | "127.0.0.1" | "::1" | "[::1]")
    );
    if url.scheme() != "http"
        || !local_host
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        anyhow::bail!(
            "MCP API URL must be a credential-free loopback HTTP origin; remote transport is deferred to Plan 109"
        );
    }
    Ok(url.as_str().trim_end_matches('/').to_string())
}

fn append_bounded(body: &mut Vec<u8>, chunk: &[u8]) -> anyhow::Result<()> {
    if body.len().saturating_add(chunk.len()) > MCP_GRAPHQL_MAX_BYTES {
        anyhow::bail!("GraphQL response exceeds MCP byte budget");
    }
    body.extend_from_slice(chunk);
    Ok(())
}

/// Host header value matching the local-loopback guard on `/graphql`.
fn host_header_for(base_url: &str) -> String {
    base_url
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_end_matches('/')
        .to_string()
}

/// Raw canonical bundle fields from GraphQL `bundle(...)`.
#[derive(Debug, Clone)]
pub(crate) struct BundleProjection {
    /// Exact `json` field string from the API (do not re-serialize for compare).
    pub json: String,
    pub markdown: String,
    pub canonical_hash: String,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentSessionProjection {
    pub root_span_id: Option<String>,
    pub total_input_tokens: String,
    pub total_output_tokens: String,
    pub error_count: i32,
    pub truncated: bool,
    pub steps: Vec<AgentStepProjection>,
}

#[derive(Debug, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AgentStepProjection {
    pub span_id: String,
    pub trace_id: String,
    pub kind: String,
    pub name: String,
    pub start_nanos: String,
    pub duration_ns: String,
    pub is_error: bool,
    pub gen_ai_operation: Option<String>,
    pub input_tokens: Option<String>,
    pub output_tokens: Option<String>,
}

/// Fetch issue- or invocation-anchored bundle. Exactly one anchor is required.
pub(crate) async fn fetch_bundle(
    client: &GraphqlClient,
    fingerprint: Option<&str>,
    invocation_id: Option<&str>,
) -> anyhow::Result<BundleProjection> {
    let (query, variables) = match (fingerprint, invocation_id) {
        (Some(fingerprint), None) => (
            "query Bundle($anchor: String!, $maxTokens: Int!) { bundle(fingerprint: $anchor, maxTokens: $maxTokens) { json markdown canonicalHash } }",
            serde_json::json!({ "anchor": fingerprint, "maxTokens": MCP_BUNDLE_MAX_TOKENS }),
        ),
        (None, Some(invocation_id)) => (
            "query Bundle($anchor: String!, $maxTokens: Int!) { bundle(invocationId: $anchor, maxTokens: $maxTokens) { json markdown canonicalHash } }",
            serde_json::json!({ "anchor": invocation_id, "maxTokens": MCP_BUNDLE_MAX_TOKENS }),
        ),
        _ => anyhow::bail!("fetch_bundle requires exactly one of fingerprint or invocation_id"),
    };
    let response = client.graphql(query, variables).await?;
    let Some(bundle) = response.pointer("/data/bundle").filter(|v| !v.is_null()) else {
        anyhow::bail!("bundle not found for the given anchor");
    };
    Ok(BundleProjection {
        json: required_string(bundle, "json")?,
        markdown: required_string(bundle, "markdown")?,
        canonical_hash: required_string(bundle, "canonicalHash")?,
    })
}

fn required_string(object: &Value, field: &str) -> anyhow::Result<String> {
    object
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("GraphQL response has invalid `{field}` field"))
}

/// Agent-session projection (same GraphQL shape the CLI uses).
pub(crate) async fn fetch_agent_session(
    client: &GraphqlClient,
    invocation_id: &str,
) -> anyhow::Result<AgentSessionProjection> {
    let response = client
        .graphql(
            AGENT_SESSION_QUERY,
            serde_json::json!({ "invocationId": invocation_id }),
        )
        .await?;
    let Some(session) = response
        .pointer("/data/agentSession")
        .filter(|v| !v.is_null())
    else {
        anyhow::bail!("no agent session detected for invocation {invocation_id}");
    };
    Ok(serde_json::from_value(session.clone())?)
}

#[cfg(test)]
mod tests {
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
    fn client_constructor_enforces_loopback_origin() {
        GraphqlClient::new("http://127.0.0.1:4000".to_string()).expect("loopback");
        let _remote = GraphqlClient::new("http://example.com:4000".to_string())
            .err()
            .expect("remote host");
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
}
