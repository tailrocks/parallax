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
    pub telemetry: TelemetryConfig,
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
    /// Product storage mode: `managed` or `external`.
    pub mode: String,
    pub greptime_url: String,
    /// Pinned GreptimeDB version to install. Defaults to v1.1.2, the latest
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

impl Config {
    /// Load from a config file if present, else defaults.
    pub fn load(path: Option<&Path>) -> anyhow::Result<Self> {
        let config = match path {
            Some(p) if p.exists() => toml::from_str(&std::fs::read_to_string(p)?)?,
            _ => Self::default(),
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            matches!(self.storage.mode.as_str(), "managed" | "external"),
            "unsupported storage.mode {:?}; supported values are \"managed\" and \"external\"",
            self.storage.mode
        );
        anyhow::ensure!(
            self.storage.mode != "external" || !self.storage.greptime_url.trim().is_empty(),
            "storage.mode=external requires greptime_url"
        );
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn rejects_removed_none_storage_mode() {
        let config: Config = toml::from_str("[storage]\nmode = 'none'\n").expect("parse");
        let error = config.validate().expect_err("none must be rejected");
        assert_eq!(
            error.to_string(),
            "unsupported storage.mode \"none\"; supported values are \"managed\" and \"external\""
        );
    }

    #[test]
    fn external_storage_requires_url() {
        let mut config = Config::default();
        config.storage.mode = "external".to_string();
        let error = config.validate().expect_err("URL required");
        assert_eq!(
            error.to_string(),
            "storage.mode=external requires greptime_url"
        );
    }
}
