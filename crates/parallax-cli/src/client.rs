//! The CLI's API client: raw GraphQL over HTTP against a context's URL.
//! The CLI never touches storage — kubectl model, one canonical API.

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct ContextsFile {
    #[serde(default)]
    pub current: Option<String>,
    #[serde(default)]
    pub contexts: Vec<NamedContext>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NamedContext {
    pub name: String,
    pub url: String,
    /// API token for remote contexts — read when auth lands with the V2
    /// server profiles; accepted in the file now so contexts are forward
    /// compatible.
    #[serde(default)]
    #[allow(dead_code)]
    pub token: Option<String>,
}

fn contexts_path() -> Option<PathBuf> {
    std::env::home_dir().map(|h| h.join(".parallax/contexts.toml"))
}

/// Resolve the API base URL: --context name → contexts file; default →
/// the file's `current`, else the implicit local context.
pub fn resolve_url(context: Option<&str>) -> anyhow::Result<String> {
    let file: Option<ContextsFile> = contexts_path()
        .filter(|p| p.exists())
        .map(|p| -> anyhow::Result<ContextsFile> {
            Ok(toml::from_str(&std::fs::read_to_string(p)?)?)
        })
        .transpose()?;
    let wanted = context
        .map(str::to_string)
        .or_else(|| file.as_ref().and_then(|f| f.current.clone()));
    match (wanted, file) {
        (Some(name), Some(file)) if name != "local" => file
            .contexts
            .iter()
            .find(|c| c.name == name)
            .map(|c| c.url.trim_end_matches('/').to_string())
            .ok_or_else(|| anyhow::anyhow!("unknown context '{name}' in contexts.toml")),
        _ => Ok("http://127.0.0.1:4000".to_string()),
    }
}

pub struct Client {
    base_url: String,
    http: reqwest::Client,
}

impl Client {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            http: reqwest::Client::new(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn graphql(&self, query: &str) -> anyhow::Result<serde_json::Value> {
        let response: serde_json::Value = self
            .http
            .post(format!("{}/graphql", self.base_url))
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
            && !errors.as_array().map(Vec::is_empty).unwrap_or(true)
        {
            anyhow::bail!("graphql error: {errors}");
        }
        Ok(response)
    }

    /// Open a Server-Sent Events stream (live tail endpoints).
    pub async fn sse(&self, path_and_query: &str) -> anyhow::Result<reqwest::Response> {
        let response = self
            .http
            .get(format!("{}{}", self.base_url, path_and_query))
            .header("accept", "text/event-stream")
            .send()
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "cannot reach Parallax at {} ({e}); is `parallax serve` running?",
                    self.base_url
                )
            })?;
        anyhow::ensure!(
            response.status().is_success(),
            "stream request failed: {}",
            response.status()
        );
        Ok(response)
    }
}

/// Escape a string for inclusion inside a GraphQL double-quoted literal.
pub fn gql_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
