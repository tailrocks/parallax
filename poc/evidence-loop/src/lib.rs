//! Parallax evidence-loop PoC library.
//!
//! Proves the offline data plane of the autonomous fix loop's Detect→Context
//! stages (docs/research/architecture/autonomous-fix-loop.md): OTLP JSON in,
//! redacted evidence bundles with canonical hashes out. No network, no
//! database, no wall clock — byte-identical output from identical fixtures.

pub mod bundle;
pub mod derive;
pub mod fingerprint;
pub mod otlp;
pub mod redact;

use anyhow::Context;

pub struct PipelineOutput {
    pub error_events: Vec<derive::ErrorEvent>,
    pub bundles: Vec<bundle::Bundle>,
}

/// Run the whole pipeline on OTLP/JSON trace + logs payloads.
pub fn run_pipeline(project: &str, trace_json: &str, logs_json: &str) -> anyhow::Result<PipelineOutput> {
    let trace: otlp::TraceData =
        serde_json::from_str(trace_json).context("parsing OTLP trace JSON")?;
    let logs: otlp::LogsData = serde_json::from_str(logs_json).context("parsing OTLP logs JSON")?;

    let mut error_events = derive::derive_from_trace(&trace);
    error_events.extend(derive::derive_from_logs(&logs));

    let bundles = bundle::build_bundles(project, &trace, &logs, &error_events);
    Ok(PipelineOutput { error_events, bundles })
}
