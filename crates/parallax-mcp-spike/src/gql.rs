//! Thin GraphQL client against a running Parallax API.
//!
use serde_json::Value;

#[derive(Clone)]
pub(crate) struct GraphqlClient {
    base_url: String,
    http: reqwest::Client,
}

impl GraphqlClient {
    pub(crate) fn new(base_url: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http: reqwest::Client::new(),
        }
    }

    pub(crate) async fn graphql(&self, query: &str, variables: Value) -> anyhow::Result<Value> {
        let response: Value = self
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
            .json()
            .await?;
        if let Some(errors) = response.get("errors")
            && !errors.as_array().is_none_or(Vec::is_empty)
        {
            anyhow::bail!("graphql error: {errors}");
        }
        Ok(response)
    }
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
            "query Bundle($anchor: String!) { bundle(fingerprint: $anchor) { json markdown canonicalHash } }",
            serde_json::json!({ "anchor": fingerprint }),
        ),
        (None, Some(invocation_id)) => (
            "query Bundle($anchor: String!) { bundle(invocationId: $anchor) { json markdown canonicalHash } }",
            serde_json::json!({ "anchor": invocation_id }),
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
