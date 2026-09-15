//! Typed TOML configuration: ingress project/key → destination DSNs.

use std::collections::HashSet;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::dsn::ParsedDsn;
use crate::fanout::DestinationKind;

const DEFAULT_LISTEN: &str = "0.0.0.0:8080";
const DEFAULT_CHANNEL_CAPACITY: usize = 256;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_listen")]
    pub listen: String,
    #[serde(default)]
    pub fanout: FanoutConfig,
    pub ingress: Vec<IngressConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FanoutConfig {
    #[serde(default = "default_channel_capacity")]
    pub channel_capacity: usize,
}

impl Default for FanoutConfig {
    fn default() -> Self {
        Self {
            channel_capacity: DEFAULT_CHANNEL_CAPACITY,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct IngressConfig {
    pub project_id: String,
    pub public_key: String,
    pub destinations: DestinationsConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DestinationsConfig {
    pub sentry: Option<DestinationConfig>,
    pub rustrak: Option<DestinationConfig>,
    pub parallax: Option<DestinationConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DestinationConfig {
    #[serde(default)]
    pub enabled: bool,
    pub dsn: String,
}

impl DestinationsConfig {
    /// Enabled destinations with parsed DSNs for one ingress route.
    #[must_use]
    pub fn enabled(&self) -> Vec<(DestinationKind, ParsedDsn)> {
        let mut out = Vec::new();
        if let Some(dest) = self.sentry.as_ref().filter(|d| d.enabled)
            && let Ok(parsed) = ParsedDsn::parse(&dest.dsn)
        {
            out.push((DestinationKind::Sentry, parsed));
        }
        if let Some(dest) = self.rustrak.as_ref().filter(|d| d.enabled)
            && let Ok(parsed) = ParsedDsn::parse(&dest.dsn)
        {
            out.push((DestinationKind::Rustrak, parsed));
        }
        if let Some(dest) = self.parallax.as_ref().filter(|d| d.enabled)
            && let Ok(parsed) = ParsedDsn::parse(&dest.dsn)
        {
            out.push((DestinationKind::Parallax, parsed));
        }
        out
    }
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("read config {}", path.display()))?;
        let config: Self = toml::from_str(&raw).context("parse config TOML")?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        anyhow::ensure!(
            !self.ingress.is_empty(),
            "at least one [[ingress]] entry is required"
        );
        for entry in &self.ingress {
            anyhow::ensure!(
                !entry.project_id.is_empty(),
                "ingress project_id must not be empty"
            );
            anyhow::ensure!(
                !entry.public_key.is_empty(),
                "ingress public_key must not be empty"
            );
            anyhow::ensure!(
                !entry.destinations.enabled().is_empty(),
                "ingress project {} must have at least one enabled destination",
                entry.project_id
            );
        }
        Ok(())
    }

    /// Lookup ingress by `(project_id, public_key)`.
    #[must_use]
    pub fn resolve_ingress(&self, project_id: &str, public_key: &str) -> Option<&IngressConfig> {
        self.ingress
            .iter()
            .find(|entry| entry.project_id == project_id && entry.public_key == public_key)
    }

    /// Union of destination kinds enabled on any ingress row.
    #[must_use]
    pub fn active_destinations(&self) -> HashSet<DestinationKind> {
        let mut kinds = HashSet::new();
        for entry in &self.ingress {
            for (kind, _) in entry.destinations.enabled() {
                kinds.insert(kind);
            }
        }
        kinds
    }
}

fn default_listen() -> String {
    DEFAULT_LISTEN.to_string()
}

fn default_channel_capacity() -> usize {
    DEFAULT_CHANNEL_CAPACITY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_example_config() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/config.toml.example");
        let config = Config::load(Path::new(path)).expect("example config");
        assert_eq!(config.listen, "0.0.0.0:8080");
        assert_eq!(config.ingress.len(), 1);
        assert_eq!(config.ingress[0].project_id, "1");
        let enabled = config.ingress[0].destinations.enabled();
        assert_eq!(enabled.len(), 3);
    }
}
