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
    /// Logs across two services and three severities.
    LogsPilot,
    /// One named trace with children and one error span.
    TracesPilot,
    /// One dashboard with one widget.
    DashboardsPilot,
    /// Minimal telemetry for `SELECT count(*)`.
    SqlPilot,
    /// One alert rule, destination, and resolved incident.
    AlertsPilot,
    /// Gauge + histogram with known series.
    MetricsPilot,
}

impl DatasetId {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ShellEmpty => "shell-empty",
            Self::InvestigationsPilot => "investigations-pilot",
            Self::LogsPilot => "logs-pilot",
            Self::TracesPilot => "traces-pilot",
            Self::DashboardsPilot => "dashboards-pilot",
            Self::SqlPilot => "sql-pilot",
            Self::AlertsPilot => "alerts-pilot",
            Self::MetricsPilot => "metrics-pilot",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "shell-empty" => Some(Self::ShellEmpty),
            "investigations-pilot" => Some(Self::InvestigationsPilot),
            "logs-pilot" => Some(Self::LogsPilot),
            "traces-pilot" => Some(Self::TracesPilot),
            "dashboards-pilot" => Some(Self::DashboardsPilot),
            "sql-pilot" => Some(Self::SqlPilot),
            "alerts-pilot" => Some(Self::AlertsPilot),
            "metrics-pilot" => Some(Self::MetricsPilot),
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

pub const LOGS_PILOT_BODY: &str = "checkout authorize failed";
pub const LOGS_PILOT_SERVICE_A: &str = "checkout";
pub const LOGS_PILOT_SERVICE_B: &str = "billing";
pub const LOGS_PILOT_COUNT: usize = 6;

pub const TRACES_PILOT_TRACE_ID: &str = "cccccccccccccccccccccccccccccccc";
pub const TRACES_PILOT_ROOT_NAME: &str = "checkout.authorize";
pub const TRACES_PILOT_CHILD_NAME: &str = "checkout.db";
pub const TRACES_PILOT_ERROR_NAME: &str = "checkout.pay";

pub const DASHBOARD_PILOT_ID: &str = "dash-pilot-001";
pub const DASHBOARD_PILOT_NAME: &str = "Checkout RED";
pub const DASHBOARD_PILOT_WIDGET: &str = "p95 checkout";

pub const ALERT_DEST_PILOT_ID: &str = "dest-pilot-001";
pub const ALERT_DEST_PILOT_NAME: &str = "Ops webhook";
pub const ALERT_RULE_PILOT_ID: &str = "rule-pilot-001";
pub const ALERT_RULE_PILOT_NAME: &str = "High checkout errors";
pub const ALERT_INCIDENT_PILOT_ID: &str = "inc-pilot-001";

pub const METRICS_PILOT_GAUGE: &str = "checkout.queue.depth";
pub const METRICS_PILOT_HISTOGRAM: &str = "http.server.duration";

/// Fixed UTC anchor for seeded investigation rows (2026-01-15T12:00:00Z).
pub const ANCHOR_TS_NANOS: u128 = 1_768_478_400_000_000_000;

/// Telemetry timestamp inside the contracts clock window
/// (`page.clock` = 2026-07-18T00:00Z, default range last 24h).
pub const CONTRACTS_TS_NANOS: u128 = 1_784_329_200_000_000_000;

/// Typed scenario manifest: fixed IDs, timestamps, and expected postconditions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenarioManifest {
    pub schema_version: u32,
    pub dataset_id: DatasetId,
    pub investigation_ids: Vec<String>,
    pub expected_investigation_names: Vec<String>,
    pub span_count: usize,
    pub log_count: usize,
    pub metric_count: usize,
}

#[must_use]
pub fn catalog() -> Vec<ScenarioManifest> {
    dataset_ids().into_iter().map(manifest_for).collect()
}

#[must_use]
pub fn dataset_ids() -> Vec<DatasetId> {
    vec![
        DatasetId::ShellEmpty,
        DatasetId::InvestigationsPilot,
        DatasetId::LogsPilot,
        DatasetId::TracesPilot,
        DatasetId::DashboardsPilot,
        DatasetId::SqlPilot,
        DatasetId::AlertsPilot,
        DatasetId::MetricsPilot,
    ]
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
            metric_count: 0,
        },
        DatasetId::InvestigationsPilot => ScenarioManifest {
            schema_version: 1,
            dataset_id: dataset,
            investigation_ids: vec![INVESTIGATION_PILOT_ID.into()],
            expected_investigation_names: vec![INVESTIGATION_PILOT_NAME.into()],
            span_count: 1,
            log_count: 0,
            metric_count: 0,
        },
        DatasetId::LogsPilot => ScenarioManifest {
            schema_version: 1,
            dataset_id: dataset,
            investigation_ids: Vec::new(),
            expected_investigation_names: Vec::new(),
            span_count: 0,
            log_count: LOGS_PILOT_COUNT,
            metric_count: 0,
        },
        DatasetId::TracesPilot => ScenarioManifest {
            schema_version: 1,
            dataset_id: dataset,
            investigation_ids: Vec::new(),
            expected_investigation_names: Vec::new(),
            span_count: 3,
            log_count: 0,
            metric_count: 0,
        },
        DatasetId::DashboardsPilot => ScenarioManifest {
            schema_version: 1,
            dataset_id: dataset,
            investigation_ids: Vec::new(),
            expected_investigation_names: Vec::new(),
            span_count: 0,
            log_count: 0,
            metric_count: 2,
        },
        DatasetId::SqlPilot => ScenarioManifest {
            schema_version: 1,
            dataset_id: dataset,
            investigation_ids: Vec::new(),
            expected_investigation_names: Vec::new(),
            span_count: 0,
            log_count: 2,
            metric_count: 0,
        },
        DatasetId::AlertsPilot => ScenarioManifest {
            schema_version: 1,
            dataset_id: dataset,
            investigation_ids: Vec::new(),
            expected_investigation_names: Vec::new(),
            span_count: 0,
            log_count: 0,
            metric_count: 0,
        },
        DatasetId::MetricsPilot => ScenarioManifest {
            schema_version: 1,
            dataset_id: dataset,
            investigation_ids: Vec::new(),
            expected_investigation_names: Vec::new(),
            span_count: 0,
            log_count: 0,
            metric_count: 2,
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
