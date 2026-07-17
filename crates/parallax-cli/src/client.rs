//! The CLI's API client: raw GraphQL over HTTP against a context's URL.
//! The CLI never touches storage — kubectl model, one canonical API.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const DEFAULT_LOCAL_URL: &str = "http://127.0.0.1:4000";
const RESERVED_LOCAL: &str = "local";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ContextsFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contexts: Vec<NamedContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NamedContext {
    pub name: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_env: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedContext {
    /// Context name; asserted in tests and kept for operator-facing output.
    #[cfg_attr(not(test), expect(dead_code, reason = "read by context tests"))]
    pub name: String,
    pub url: String,
    pub token: Option<String>,
}

fn contexts_path() -> Option<PathBuf> {
    std::env::home_dir().map(|home| home.join(".parallax/contexts.toml"))
}

fn load_contexts_at(path: &Path) -> anyhow::Result<ContextsFile> {
    if !path.exists() {
        return Ok(ContextsFile::default());
    }
    Ok(toml::from_str(&fs::read_to_string(path)?)?)
}

fn write_contexts_atomic(path: &Path, file: &ContextsFile) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            drop(fs::set_permissions(
                parent,
                fs::Permissions::from_mode(0o700),
            ));
        }
    }
    let text = toml::to_string_pretty(file)?;
    let tmp = path.with_extension("toml.tmp");
    {
        let mut out = fs::File::create(&tmp)?;
        out.write_all(text.as_bytes())?;
        out.sync_all()?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&tmp, fs::Permissions::from_mode(0o600))?;
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

pub(crate) fn resolve_context(context: Option<&str>) -> anyhow::Result<ResolvedContext> {
    resolve_context_at(
        context,
        contexts_path().as_deref(),
        std::env::var("PARALLAX_API_TOKEN").ok(),
    )
}

pub(crate) fn resolve_context_at(
    context: Option<&str>,
    path: Option<&Path>,
    env_token: Option<String>,
) -> anyhow::Result<ResolvedContext> {
    let file = match path {
        Some(path) => load_contexts_at(path)?,
        None => ContextsFile::default(),
    };
    let wanted = context
        .map(str::to_string)
        .or_else(|| file.current.clone())
        .unwrap_or_else(|| RESERVED_LOCAL.to_string());

    if wanted == RESERVED_LOCAL {
        return Ok(ResolvedContext {
            name: RESERVED_LOCAL.to_string(),
            url: DEFAULT_LOCAL_URL.to_string(),
            token: normalize_token(env_token),
        });
    }

    let entry = file
        .contexts
        .iter()
        .find(|entry| entry.name == wanted)
        .ok_or_else(|| anyhow::anyhow!("unknown context '{wanted}' in contexts.toml"))?;

    let token = match normalize_token(env_token) {
        Some(token) => Some(token),
        None => resolve_entry_token(entry)?,
    };

    Ok(ResolvedContext {
        name: entry.name.clone(),
        url: entry.url.trim_end_matches('/').to_string(),
        token,
    })
}

fn resolve_entry_token(entry: &NamedContext) -> anyhow::Result<Option<String>> {
    if let Some(var) = entry.token_env.as_deref() {
        let value = std::env::var(var).map_err(|_| {
            anyhow::anyhow!("context '{}' token_env '{var}' is not set", entry.name)
        })?;
        return Ok(normalize_token(Some(value)));
    }
    Ok(normalize_token(entry.token.clone()))
}

fn normalize_token(raw: Option<String>) -> Option<String> {
    raw.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("off") {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(crate) struct Client {
    base_url: String,
    token: Option<String>,
    http: reqwest::Client,
}

impl Client {
    pub(crate) fn from_resolved(resolved: ResolvedContext) -> Self {
        Self::with_token(resolved.url, resolved.token)
    }

    pub(crate) fn with_token(base_url: String, token: Option<String>) -> Self {
        Self {
            base_url,
            token,
            http: reqwest::Client::new(),
        }
    }

    pub(crate) fn base_url(&self) -> &str {
        &self.base_url
    }

    fn apply_auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.token {
            Some(token) => {
                request.header(reqwest::header::AUTHORIZATION, format!("Bearer {token}"))
            }
            None => request,
        }
    }

    pub(crate) async fn graphql(&self, query: &str) -> anyhow::Result<serde_json::Value> {
        let response: serde_json::Value = self
            .apply_auth(
                self.http
                    .post(format!("{}/graphql", self.base_url))
                    .json(&serde_json::json!({ "query": query })),
            )
            .send()
            .await
            .map_err(|error| {
                anyhow::anyhow!(
                    "cannot reach Parallax at {} ({error}); is `parallax serve` running?",
                    self.base_url
                )
            })?
            .error_for_status()
            .map_err(|error| {
                if error.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
                    anyhow::anyhow!(
                        "Parallax at {} rejected the request (unauthorized); \
                         set PARALLAX_API_TOKEN or a context token",
                        self.base_url
                    )
                } else {
                    anyhow::anyhow!("request failed: {error}")
                }
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

    pub(crate) async fn sse(&self, path_and_query: &str) -> anyhow::Result<reqwest::Response> {
        let response = self
            .apply_auth(
                self.http
                    .get(format!("{}{}", self.base_url, path_and_query))
                    .header("accept", "text/event-stream"),
            )
            .send()
            .await
            .map_err(|error| {
                anyhow::anyhow!(
                    "cannot reach Parallax at {} ({error}); is `parallax serve` running?",
                    self.base_url
                )
            })?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            anyhow::bail!(
                "Parallax at {} rejected the stream (unauthorized); \
                 set PARALLAX_API_TOKEN or a context token",
                self.base_url
            );
        }
        anyhow::ensure!(
            response.status().is_success(),
            "stream request failed: {}",
            response.status()
        );
        Ok(response)
    }
}

pub(crate) fn gql_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub(crate) fn context_add(
    name: &str,
    url: &str,
    token: Option<&str>,
    token_env: Option<&str>,
) -> anyhow::Result<()> {
    context_add_at(
        contexts_path().ok_or_else(|| anyhow::anyhow!("cannot resolve home directory"))?,
        name,
        url,
        token,
        token_env,
    )
}

pub(crate) fn context_add_at(
    path: impl AsRef<Path>,
    name: &str,
    url: &str,
    token: Option<&str>,
    token_env: Option<&str>,
) -> anyhow::Result<()> {
    let name = name.trim();
    anyhow::ensure!(!name.is_empty(), "context name must not be empty");
    anyhow::ensure!(
        name != RESERVED_LOCAL,
        "context name '{RESERVED_LOCAL}' is reserved for the implicit loopback context"
    );
    anyhow::ensure!(
        !(token.is_some() && token_env.is_some()),
        "pass either --token or --token-env, not both"
    );
    let url = url.trim().trim_end_matches('/');
    anyhow::ensure!(!url.is_empty(), "context url must not be empty");
    let path = path.as_ref();
    let mut file = load_contexts_at(path)?;
    if file.contexts.iter().any(|entry| entry.name == name) {
        anyhow::bail!("context '{name}' already exists; remove it first");
    }
    file.contexts.push(NamedContext {
        name: name.to_string(),
        url: url.to_string(),
        token: token
            .map(str::to_string)
            .and_then(|value| normalize_token(Some(value))),
        token_env: token_env
            .map(str::to_string)
            .filter(|value| !value.is_empty()),
    });
    write_contexts_atomic(path, &file)?;
    println!("context '{name}' added");
    Ok(())
}

pub(crate) fn context_list() -> anyhow::Result<()> {
    context_list_at(
        contexts_path().ok_or_else(|| anyhow::anyhow!("cannot resolve home directory"))?,
    )
}

pub(crate) fn context_list_at(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let file = load_contexts_at(path.as_ref())?;
    let current = file.current.as_deref().unwrap_or(RESERVED_LOCAL);
    print_row(
        current == RESERVED_LOCAL,
        RESERVED_LOCAL,
        DEFAULT_LOCAL_URL,
        false,
    );
    for entry in &file.contexts {
        print_row(
            current == entry.name,
            &entry.name,
            entry.url.trim_end_matches('/'),
            entry.token.is_some() || entry.token_env.is_some(),
        );
    }
    Ok(())
}

fn print_row(current: bool, name: &str, url: &str, has_secret: bool) {
    let mark = if current { "*" } else { " " };
    let secret = if has_secret { "  token=set" } else { "" };
    println!("{mark}  {name:<16}  {url}{secret}");
}

pub(crate) fn context_use(name: &str) -> anyhow::Result<()> {
    context_use_at(
        contexts_path().ok_or_else(|| anyhow::anyhow!("cannot resolve home directory"))?,
        name,
    )
}

pub(crate) fn context_use_at(path: impl AsRef<Path>, name: &str) -> anyhow::Result<()> {
    let name = name.trim();
    anyhow::ensure!(!name.is_empty(), "context name must not be empty");
    let path = path.as_ref();
    let mut file = load_contexts_at(path)?;
    if name != RESERVED_LOCAL && !file.contexts.iter().any(|entry| entry.name == name) {
        anyhow::bail!("unknown context '{name}'");
    }
    file.current = if name == RESERVED_LOCAL {
        None
    } else {
        Some(name.to_string())
    };
    write_contexts_atomic(path, &file)?;
    println!("using context '{name}'");
    Ok(())
}

pub(crate) fn context_show(name: Option<&str>) -> anyhow::Result<()> {
    context_show_at(
        contexts_path().ok_or_else(|| anyhow::anyhow!("cannot resolve home directory"))?,
        name,
    )
}

pub(crate) fn context_show_at(path: impl AsRef<Path>, name: Option<&str>) -> anyhow::Result<()> {
    let file = load_contexts_at(path.as_ref())?;
    let wanted = name
        .map(str::to_string)
        .or_else(|| file.current.clone())
        .unwrap_or_else(|| RESERVED_LOCAL.to_string());
    if wanted == RESERVED_LOCAL {
        println!("name:  {RESERVED_LOCAL}");
        println!("url:   {DEFAULT_LOCAL_URL}");
        println!("token: (none in file; PARALLAX_API_TOKEN may still apply)");
        return Ok(());
    }
    let entry = file
        .contexts
        .iter()
        .find(|entry| entry.name == wanted)
        .ok_or_else(|| anyhow::anyhow!("unknown context '{wanted}'"))?;
    println!("name:  {}", entry.name);
    println!("url:   {}", entry.url.trim_end_matches('/'));
    match (&entry.token, &entry.token_env) {
        (_, Some(var)) => println!("token: env:{var}"),
        (Some(_), None) => println!("token: ********"),
        (None, None) => println!("token: (none)"),
    }
    Ok(())
}

pub(crate) fn context_remove(name: &str) -> anyhow::Result<()> {
    context_remove_at(
        contexts_path().ok_or_else(|| anyhow::anyhow!("cannot resolve home directory"))?,
        name,
    )
}

pub(crate) fn context_remove_at(path: impl AsRef<Path>, name: &str) -> anyhow::Result<()> {
    let name = name.trim();
    anyhow::ensure!(
        name != RESERVED_LOCAL,
        "cannot remove reserved context 'local'"
    );
    let path = path.as_ref();
    let mut file = load_contexts_at(path)?;
    let before = file.contexts.len();
    file.contexts.retain(|entry| entry.name != name);
    anyhow::ensure!(file.contexts.len() < before, "unknown context '{name}'");
    if file.current.as_deref() == Some(name) {
        file.current = None;
    }
    write_contexts_atomic(path, &file)?;
    println!("context '{name}' removed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn context_lifecycle_is_atomic_and_masks_token() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("contexts.toml");
        context_add_at(
            &path,
            "prod",
            "https://parallax.internal/",
            Some("super-secret-token"),
            None,
        )
        .expect("add");
        let loaded = load_contexts_at(&path).expect("load");
        assert_eq!(loaded.contexts[0].url, "https://parallax.internal");
        assert_eq!(
            loaded.contexts[0].token.as_deref(),
            Some("super-secret-token")
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = fs::metadata(&path).expect("meta").permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
        context_use_at(&path, "prod").expect("use");
        let resolved = resolve_context_at(None, Some(&path), None).expect("resolve");
        assert_eq!(resolved.token.as_deref(), Some("super-secret-token"));
        context_remove_at(&path, "prod").expect("remove");
        assert!(load_contexts_at(&path).expect("load").contexts.is_empty());
    }

    #[test]
    fn env_token_overrides_context_token() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("contexts.toml");
        context_add_at(
            &path,
            "prod",
            "https://example.test",
            Some("file-token-value"),
            None,
        )
        .expect("add");
        let resolved = resolve_context_at(
            Some("prod"),
            Some(&path),
            Some("env-token-value-16".to_string()),
        )
        .expect("resolve");
        assert_eq!(resolved.token.as_deref(), Some("env-token-value-16"));
    }

    #[test]
    fn local_context_stays_open_without_file() {
        let resolved = resolve_context_at(None, None, None).expect("local");
        assert_eq!(resolved.name, "local");
        assert!(resolved.token.is_none());
    }

    #[test]
    fn reserved_local_name_cannot_be_added() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("contexts.toml");
        let err = context_add_at(&path, "local", "http://example", None, None).unwrap_err();
        assert!(err.to_string().contains("reserved"));
    }
}
