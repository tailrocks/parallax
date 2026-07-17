//! Versioned, deterministic browser scenario manifests.

use serde::{Deserialize, Serialize};

/// Stable dataset identity selected by Playwright fixtures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DatasetId {
    /// Empty shell: no telemetry, no investigations.
    ShellEmpty,
    /// Investigations pilot with one seeded case file + pin + note.
    InvestigationsPilot,
}

impl DatasetId {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ShellEmpty => "shell-empty",
            Self::InvestigationsPilot => "investigations-pilot",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "shell-empty" => Some(Self::ShellEmpty),
            "investigations-pilot" => Some(Self::InvestigationsPilot),
            _ => None,
        }
    }
}

impl std::fmt::Display for DatasetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Fixed investigation pilot identity (seed + postcondition).
pub const INVESTIGATION_PILOT_ID: &str = "inv-pilot-001";
pub const INVESTIGATION_PILOT_NAME: &str = "Checkout latency case";

/// Fixed UTC anchor for seeded telemetry rows (2026-01-15T12:00:00Z).
pub const ANCHOR_TS_NANOS: u128 = 1_768_478_400_000_000_000;

/// Typed scenario manifest: fixed IDs, timestamps, and expected postconditions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenarioManifest {
    pub schema_version: u32,
    pub dataset_id: DatasetId,
    pub investigation_ids: Vec<String>,
    pub expected_investigation_names: Vec<String>,
    pub span_count: usize,
    pub log_count: usize,
}

#[must_use]
pub fn catalog() -> Vec<ScenarioManifest> {
    dataset_ids().into_iter().map(manifest_for).collect()
}

#[must_use]
pub fn dataset_ids() -> Vec<DatasetId> {
    vec![DatasetId::ShellEmpty, DatasetId::InvestigationsPilot]
}

#[must_use]
pub fn manifest_for(dataset: DatasetId) -> ScenarioManifest {
    match dataset {
        DatasetId::ShellEmpty => ScenarioManifest {
            schema_version: 1,
            dataset_id: dataset,
            investigation_ids: Vec::new(),
            expected_investigation_names: Vec::new(),
            span_count: 0,
            log_count: 0,
        },
        DatasetId::InvestigationsPilot => ScenarioManifest {
            schema_version: 1,
            dataset_id: dataset,
            investigation_ids: vec![INVESTIGATION_PILOT_ID.into()],
            expected_investigation_names: vec![INVESTIGATION_PILOT_NAME.into()],
            span_count: 1,
            log_count: 0,
        },
    }
}

/// Pilot investigation state JSON (versioned pins + notes).
#[must_use]
pub fn pilot_investigation_state_json() -> String {
    serde_json::json!({
        "version": 1,
        "window": { "range": "24h" },
        "pins": [{
            "kind": "trace",
            "ref": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "label": "Checkout authorize",
            "note": "p95 spiked after deploy"
        }],
        "notes": "Initial case notes from fixture seed."
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dataset_ids_round_trip() {
        for id in dataset_ids() {
            assert_eq!(DatasetId::parse(id.as_str()), Some(id));
            assert_eq!(id.to_string(), id.as_str());
        }
        assert!(DatasetId::parse("unknown").is_none());
    }

    #[test]
    fn catalog_covers_all_datasets_once() {
        let catalog = catalog();
        assert_eq!(catalog.len(), dataset_ids().len());
        for manifest in &catalog {
            assert_eq!(manifest.schema_version, 1);
            assert_eq!(manifest, &manifest_for(manifest.dataset_id));
        }
    }

    #[test]
    fn pilot_state_is_valid_json() {
        let value: serde_json::Value =
            serde_json::from_str(&pilot_investigation_state_json()).expect("json");
        assert_eq!(value["version"], 1);
        assert_eq!(value["pins"][0]["kind"], "trace");
    }
}
