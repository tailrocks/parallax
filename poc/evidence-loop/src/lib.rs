//! Parallax evidence-loop PoC library.
//!
//! Proves the offline data plane of the autonomous fix loop's Detect→Context
//! stages (docs/research/architecture/autonomous-fix-loop.md): OTLP JSON in,
//! redacted evidence bundles with canonical hashes out. No network, no
//! database, no wall clock — byte-identical output from identical fixtures.

pub mod bound;
pub mod budget;
pub mod bundle;
pub mod deploy;
pub mod derive;
pub mod dispatch;
pub mod fingerprint;
pub mod learn;
pub mod otlp;
pub mod redact;
pub mod rollup;
pub mod spike;

use anyhow::Context;

pub struct PipelineOutput {
    pub error_events: Vec<derive::ErrorEvent>,
    pub bundles: Vec<bundle::Bundle>,
}

/// Run the whole pipeline on OTLP/JSON trace + logs payloads, optionally with
/// `parallax.deploy.v0` deploy events for trigger escalation.
pub fn run_pipeline(
    project: &str,
    trace_json: &str,
    logs_json: &str,
    deploys_json: Option<&str>,
) -> anyhow::Result<PipelineOutput> {
    let trace: otlp::TraceData =
        serde_json::from_str(trace_json).context("parsing OTLP trace JSON")?;
    let logs: otlp::LogsData = serde_json::from_str(logs_json).context("parsing OTLP logs JSON")?;
    let deploys = match deploys_json {
        Some(json) => {
            serde_json::from_str::<deploy::DeploysData>(json)
                .context("parsing deploy events JSON")?
                .deploys
        }
        None => Vec::new(),
    };

    let mut error_events = derive::derive_from_trace(&trace);
    error_events.extend(derive::derive_from_logs(&logs));

    let bundles = bundle::build_bundles(project, &trace, &logs, &deploys, &error_events);
    Ok(PipelineOutput { error_events, bundles })
}
