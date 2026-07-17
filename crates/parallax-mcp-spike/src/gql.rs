//! Thin GraphQL client against a running Parallax API.
//!
use serde_json::Value;
use std::time::Duration;

/// MCP returns both structured JSON and compatibility text, so use a tighter
/// canonical bundle budget than the HTTP API's 10,000-token default.
pub(crate) const MCP_BUNDLE_MAX_TOKENS: u32 = 4_000;
pub(crate) const MCP_GRAPHQL_MAX_BYTES: usize = 1024 * 1024;

#[derive(Clone)]
pub(crate) struct GraphqlClient {
    base_url: String,
    http: reqwest::Client,
}

impl GraphqlClient {
    pub(crate) fn new(base_url: String) -> anyhow::Result<Self> {
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
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

/// Fetch issue- or run-anchored bundle. Exactly one of `fingerprint` / `invocation_id`.
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
        json: bundle["json"].as_str().unwrap_or("").to_string(),
        markdown: bundle["markdown"].as_str().unwrap_or("").to_string(),
        canonical_hash: bundle["canonicalHash"].as_str().unwrap_or("").to_string(),
    })
}

/// Agent-session projection (same GraphQL shape the CLI uses).
pub(crate) async fn fetch_agent_session(
    client: &GraphqlClient,
    invocation_id: &str,
) -> anyhow::Result<Value> {
    let response = client
        .graphql(
            r#"query AgentSession($invocationId: String!) {
              agentSession(invocationId: $invocationId) {
                rootSpanId totalInputTokens totalOutputTokens errorCount truncated
                steps {{
                  spanId traceId kind name startNanos durationNs isError
                  genAiOperation inputTokens outputTokens
                }}
              }
            }"#,
            serde_json::json!({ "invocationId": invocation_id }),
        )
        .await?;
    let Some(session) = response
        .pointer("/data/agentSession")
        .filter(|v| !v.is_null())
    else {
        anyhow::bail!("no agent session detected for run {invocation_id}");
    };
    Ok(session.clone())
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
}
