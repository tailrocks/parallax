//! `~/.parallax/config.toml` — keys and defaults per the implementation spec §4.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::errors::{ConfigError, ConfigResult};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub retention: RetentionConfig,
    pub limits: LimitsConfig,
    pub telemetry: TelemetryConfig,
    pub alerting: AlertingConfig,
    /// Sentry envelope migration adapter (plan 118). Disabled by default.
    pub sentry: SentryConfig,
    /// GitHub deploy/change webhook adapter (plan 121). Disabled by default.
    pub github_deploy: GithubDeployConfig,
    /// GitHub Actions workflow-job evidence adapter (plan 124). Disabled by default.
    pub github_actions: GithubActionsConfig,
}

/// Local project/public-key mapping for `POST /api/<project_id>/envelope/`.
///
/// A Sentry public key is a routing credential, not a user secret. Remote
/// exposure still requires plan 109 bearer auth at the API boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SentryConfig {
    /// When false, the envelope route returns 404.
    pub enabled: bool,
    /// Path project id clients put in the DSN path (e.g. `"1"`).
    pub project_id: String,
    /// Registered public key. Empty means no key is accepted (fail closed).
    /// Override with env `PARALLAX_SENTRY_PUBLIC_KEY`.
    pub public_key: String,
}

impl Default for SentryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            project_id: "1".to_string(),
            public_key: String::new(),
        }
    }
}

/// GitHub webhook receiver for `deployment` / `deployment_status` (plan 121).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GithubDeployConfig {
    /// When false, the webhook route returns 404.
    pub enabled: bool,
    /// HMAC secret for `X-Hub-Signature-256`. Prefer env
    /// `PARALLAX_GITHUB_WEBHOOK_SECRET`. Empty = fail closed when enabled.
    pub webhook_secret: String,
}

/// GitHub Actions `workflow_job` webhook receiver (plan 124).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GithubActionsConfig {
    /// When false, workflow-job events return 404.
    pub enabled: bool,
    /// HMAC secret for `X-Hub-Signature-256`. Prefer env
    /// `PARALLAX_GITHUB_ACTIONS_WEBHOOK_SECRET`.
    pub webhook_secret: String,
}

/// Alert evaluator + delivery worker (plan 167). Defaults keep alerting on
/// with 60s evaluation / 10s delivery ticks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AlertingConfig {
    /// When false, GraphQL CRUD still works but no background loops run.
    pub enabled: bool,
    /// Seconds between evaluator ticks (CAS claim + measure + state machine).
    pub evaluate_interval_secs: u64,
    /// Seconds between delivery-worker ticks (outbox claim + HTTP POST).
    pub deliver_interval_secs: u64,
    /// CAS claim interval written on rules (`last_scheduled_at` skip window).
    pub claim_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub bind: String,
    pub api_port: u16,
    pub otlp_grpc_port: u16,
    pub otlp_http_port: u16,
    /// Directory of the built UI (SPA shell + assets). Empty = autodetect
    /// (./ui/dist/client for dev checkouts); missing dir = API-only mode.
    pub ui_dist: String,
    /// Optional shared API bearer token (plan 109). Empty = auth disabled on
    /// loopback. Prefer env `PARALLAX_API_TOKEN` over committing secrets here.
    /// See `docs/research/decisions/v2-auth-and-context-contract.md`.
    #[serde(default)]
    pub api_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    /// Product storage mode: `managed` or `external`.
    pub mode: String,
    pub greptime_url: String,
    /// Pinned `GreptimeDB` version to install. Defaults to v1.1.2, the latest
    /// stable native-OTLP-capable release verified by Parallax. `"latest"`
    /// resolves the newest GitHub stable release at install instead (see
    /// `resolve_version`).
    pub greptime_version: String,
    pub data_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RetentionConfig {
    pub traces_ttl: String,
    pub logs_ttl: String,
    pub metrics_ttl: String,
    pub error_events_ttl: String,
    pub spool_max_segment_bytes: u64,
    pub spool_max_total_bytes: u64,
    pub spool_max_age_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LimitsConfig {
    pub bundle_max_tokens: usize,
    pub graphql_max_depth: usize,
    pub graphql_max_complexity: usize,
    pub otlp_max_body_bytes: usize,
    pub ingest_queue_batches: usize,
}

/// Self-telemetry: where `parallax serve` exports its **own** spans/logs. Empty
/// (the default) keeps Parallax silent about itself — it only receives. Set an
/// OTLP/gRPC endpoint (e.g. the lab's Rotel `http://localhost:4317`) and serve
/// emits its internal telemetry there; the env var `PARALLAX_SELF_OTLP`
/// overrides this key (`off` forces it off). The OTLP ingest receivers are
/// suppressed from this exporter so a sink pointed back at Parallax does not
/// re-export what it just received.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct TelemetryConfig {
    pub self_otlp_endpoint: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1".to_string(),
            api_port: 4000,
            otlp_grpc_port: 4317,
            otlp_http_port: 4318,
            ui_dist: String::new(),
            api_token: String::new(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            mode: "managed".to_string(),
            greptime_url: String::new(),
            greptime_version: "1.1.2".to_string(),
            data_dir: "~/.parallax".to_string(),
        }
    }
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            traces_ttl: "7d".to_string(),
            logs_ttl: "7d".to_string(),
            metrics_ttl: "14d".to_string(),
            error_events_ttl: "30d".to_string(),
            spool_max_segment_bytes: 64 * 1024 * 1024,
            spool_max_total_bytes: 512 * 1024 * 1024,
            spool_max_age_hours: 72,
        }
    }
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            bundle_max_tokens: 10_000,
            graphql_max_depth: 8,
            graphql_max_complexity: 1_000,
            otlp_max_body_bytes: 16 * 1024 * 1024,
            ingest_queue_batches: 256,
        }
    }
}

impl Default for AlertingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            evaluate_interval_secs: 60,
            deliver_interval_secs: 10,
            claim_interval_secs: 30,
        }
    }
}

impl Config {
    /// Load from a config file if present, else defaults.
    pub fn load(path: Option<&Path>) -> ConfigResult<Self> {
        let config = match path {
            Some(p) if p.exists() => {
                let text = std::fs::read_to_string(p).map_err(|source| ConfigError::Read {
                    path: p.to_path_buf(),
                    source,
                })?;
                toml::from_str(&text)?
            }
            _ => Self::default(),
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> ConfigResult<()> {
        if !matches!(self.storage.mode.as_str(), "managed" | "external") {
            return Err(ConfigError::Invalid(format!(
                "unsupported storage.mode {:?}; supported values are \"managed\" and \"external\"",
                self.storage.mode
            )));
        }
        if self.storage.mode == "external" && self.storage.greptime_url.trim().is_empty() {
            return Err(ConfigError::Invalid(
                "storage.mode=external requires greptime_url".to_string(),
            ));
        }
        let token = self.resolved_api_token();
        if let Some(token) = token.as_deref() {
            let len = token.len();
            if !(16..=256).contains(&len) {
                return Err(ConfigError::Invalid(
                    "server API token must be 16–256 UTF-8 bytes after trim".to_string(),
                ));
            }
        } else if !is_loopback_bind(&self.server.bind) {
            return Err(ConfigError::Invalid(
                "non-loopback server.bind requires an API token                  (set PARALLAX_API_TOKEN or [server] api_token);                  see docs/research/decisions/v2-auth-and-context-contract.md"
                    .to_string(),
            ));
        }
        Ok(())
    }

    /// Resolve the active API token: env `PARALLAX_API_TOKEN` wins over config.
    /// Empty/whitespace values mean auth disabled.
    #[must_use]
    pub fn resolved_api_token(&self) -> Option<String> {
        resolve_api_token_from(
            std::env::var("PARALLAX_API_TOKEN").ok(),
            &self.server.api_token,
        )
    }

    /// Resolve the Sentry public key: env `PARALLAX_SENTRY_PUBLIC_KEY` wins.
    #[must_use]
    pub fn resolved_sentry_public_key(&self) -> Option<String> {
        let env = std::env::var("PARALLAX_SENTRY_PUBLIC_KEY").ok();
        env.as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
            .or_else(|| {
                let trimmed = self.sentry.public_key.trim();
                (!trimmed.is_empty()).then(|| trimmed.to_string())
            })
    }

    /// Resolve the GitHub webhook secret: env `PARALLAX_GITHUB_WEBHOOK_SECRET`
    /// wins over config.
    #[must_use]
    pub fn resolved_github_webhook_secret(&self) -> Option<String> {
        let env = std::env::var("PARALLAX_GITHUB_WEBHOOK_SECRET").ok();
        env.as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
            .or_else(|| {
                let trimmed = self.github_deploy.webhook_secret.trim();
                (!trimmed.is_empty()).then(|| trimmed.to_string())
            })
    }

    /// Resolve the Actions webhook secret without coupling it to deploy capture.
    #[must_use]
    pub fn resolved_github_actions_webhook_secret(&self) -> Option<String> {
        let env = std::env::var("PARALLAX_GITHUB_ACTIONS_WEBHOOK_SECRET").ok();
        env.as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .or_else(|| {
                let trimmed = self.github_actions.webhook_secret.trim();
                (!trimmed.is_empty()).then(|| trimmed.to_string())
            })
    }

    /// Ready-banner label that never includes the secret.
    #[must_use]
    pub fn auth_status_label(&self) -> &'static str {
        if self.resolved_api_token().is_some() {
            "bearer-token"
        } else {
            "off"
        }
    }

    /// Expand `~` in `storage.data_dir` against the user's home directory.
    #[must_use]
    pub fn data_dir(&self) -> PathBuf {
        let raw = &self.storage.data_dir;
        if let Some(rest) = raw.strip_prefix("~/")
            && let Some(home) = std::env::home_dir()
        {
            return home.join(rest);
        }
        PathBuf::from(raw)
    }
}

/// Env override (even empty/`off`) wins over the config key.
pub(crate) fn resolve_api_token_from(env: Option<String>, config_token: &str) -> Option<String> {
    match env {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("off") {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        None => {
            let trimmed = config_token.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
    }
}

/// True when the bind host is loopback-only (`127.0.0.1`, `::1`, localhost).
#[must_use]
pub(crate) fn is_loopback_bind(bind: &str) -> bool {
    let host = bind
        .split_once(':')
        .map_or(bind, |(host, _)| host)
        .trim()
        .trim_matches(|c| c == '[' || c == ']')
        .to_ascii_lowercase();
    matches!(host.as_str(), "127.0.0.1" | "::1" | "localhost" | "")
}

#[cfg(test)]
mod tests;
