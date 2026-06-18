//! `~/.parallax/config.toml` — keys and defaults per the implementation spec §4.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub retention: RetentionConfig,
    pub limits: LimitsConfig,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    /// managed | external | none
    pub mode: String,
    pub greptime_url: String,
    /// Pinned GreptimeDB version to install. Defaults to the v1.1.0 line, the
    /// floor that ships the native OTLP traces pipeline (`greptime_trace_v1`)
    /// Parallax's storage path depends on. `"latest"` resolves the newest
    /// GitHub stable release at install instead (see `resolve_version`).
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LimitsConfig {
    pub bundle_max_tokens: usize,
    pub graphql_max_depth: usize,
    pub graphql_max_complexity: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1".to_string(),
            api_port: 4000,
            otlp_grpc_port: 4317,
            otlp_http_port: 4318,
            ui_dist: String::new(),
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            mode: "managed".to_string(),
            greptime_url: String::new(),
            greptime_version: "1.1.0".to_string(),
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
        }
    }
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            bundle_max_tokens: 10_000,
            graphql_max_depth: 8,
            graphql_max_complexity: 1_000,
        }
    }
}

impl Config {
    /// Load from a config file if present, else defaults.
    pub fn load(path: Option<&Path>) -> anyhow::Result<Self> {
        match path {
            Some(p) if p.exists() => Ok(toml::from_str(&std::fs::read_to_string(p)?)?),
            _ => Ok(Self::default()),
        }
    }

    /// Expand `~` in `storage.data_dir` against the user's home directory.
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
