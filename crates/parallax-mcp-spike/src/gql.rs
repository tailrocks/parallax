//! Thin GraphQL client against a running Parallax API.
//!
//! Escape semantics match `crates/parallax-cli/src/client.rs` `gql_str`
//! (backslash + double-quote only).

use serde_json::Value;

/// Escape a string for inclusion inside a GraphQL double-quoted literal.
///
/// Copied from `parallax-cli` (`gql_str`) so MCP and CLI embed arguments the
/// same way. The CLI only escapes `\` and `"` — not newline/tab.
pub(crate) fn gql_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

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

    pub(crate) async fn graphql(&self, query: &str) -> anyhow::Result<Value> {
        let response: Value = self
            .http
            .post(format!("{}/graphql", self.base_url))
            .header("Host", host_header_for(&self.base_url))
            .json(&serde_json::json!({ "query": query }))
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

/// Fetch issue- or run-anchored bundle. Exactly one of `fingerprint` / `run_id`.
pub(crate) async fn fetch_bundle(
    client: &GraphqlClient,
    fingerprint: Option<&str>,
    run_id: Option<&str>,
) -> anyhow::Result<BundleProjection> {
    let query = match (fingerprint, run_id) {
        (Some(fp), None) => format!(
            r#"{{ bundle(fingerprint: "{}") {{ json markdown canonicalHash }} }}"#,
            gql_str(fp)
        ),
        (None, Some(rid)) => format!(
            r#"{{ bundle(runId: "{}") {{ json markdown canonicalHash }} }}"#,
            gql_str(rid)
        ),
        _ => anyhow::bail!("fetch_bundle requires exactly one of fingerprint or run_id"),
    };
    let response = client.graphql(&query).await?;
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
    run_id: &str,
) -> anyhow::Result<Value> {
    let response = client
        .graphql(&format!(
            r#"{{ agentSession(runId: "{}") {{
                rootSpanId totalInputTokens totalOutputTokens errorCount truncated
                steps {{
                  spanId traceId kind name startNanos durationNs isError
                  genAiOperation inputTokens outputTokens
                }}
            }} }}"#,
            gql_str(run_id)
        ))
        .await?;
    let Some(session) = response
        .pointer("/data/agentSession")
        .filter(|v| !v.is_null())
    else {
        anyhow::bail!("no agent session detected for run {run_id}");
    };
    Ok(session.clone())
}
