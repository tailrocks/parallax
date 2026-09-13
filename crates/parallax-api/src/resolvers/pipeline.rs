//! R2 sampling policy + drop-reason visibility (GOAL P0-next).
//!
//! The server ingests head-first and keeps every batch it receives; these
//! resolvers surface the *declared* effective policy per signal plus the
//! live drop-reason counters from the ingest pipeline, so a volume gap is
//! attributable (policy rate vs named drop reason) instead of silent.
//!
//! Data comes from [`PipelineSnapshot`], implemented server-side over
//! `IngestHealth` + `[sampling]` config. Unit harnesses leave
//! `ApiContext::pipeline` as `None` and read back empty lists.

use juniper::{FieldResult, graphql_object};

use crate::{ApiContext, field_err, saturate_i32};

/// Live pipeline readout provider (server implements this; api owns the types
/// so no server→api dependency cycle forms).
pub trait PipelineSnapshot: std::fmt::Debug + Send + Sync {
    /// Declared sampling policy rows, one per ingest signal.
    fn sampling_policies(&self) -> Vec<SamplingPolicy>;
    /// Dropped/batch-loss counts by reason (`signal` is `None` for
    /// pipeline-global reasons).
    fn ingest_drops(&self) -> Vec<IngestDrop>;
    /// Per-signal queue depth/capacity plus accepted batch counts.
    fn ingest_queues(&self) -> Vec<IngestQueue>;
}

/// Declared effective sampling policy for one ingest signal. `service` is
/// `None` (applies to all services): the server leg keeps every received
/// batch, so there are no per-service server overrides — a rate below 1.0 is
/// operator-declared producer-side sampling documented for attribution.
#[derive(Debug, Clone)]
pub struct SamplingPolicy {
    pub signal: String,
    pub service: Option<String>,
    pub rule: String,
    pub rate: f64,
    pub enforced_by: String,
    pub description: String,
}

#[graphql_object(context = ApiContext)]
impl SamplingPolicy {
    fn signal(&self) -> &str {
        &self.signal
    }
    fn service(&self) -> Option<&str> {
        self.service.as_deref()
    }
    fn rule(&self) -> &str {
        &self.rule
    }
    fn rate(&self) -> f64 {
        self.rate
    }
    fn enforced_by(&self) -> &str {
        &self.enforced_by
    }
    fn description(&self) -> &str {
        &self.description
    }
}

/// One drop-reason counter. Count is a string so large volumes never saturate
/// GraphQL Int.
#[derive(Debug, Clone)]
pub struct IngestDrop {
    pub signal: Option<String>,
    pub reason: String,
    pub count: u64,
    pub detail: String,
}

#[graphql_object(context = ApiContext)]
impl IngestDrop {
    fn signal(&self) -> Option<&str> {
        self.signal.as_deref()
    }
    fn reason(&self) -> &str {
        &self.reason
    }
    fn count(&self) -> String {
        self.count.to_string()
    }
    fn detail(&self) -> &str {
        &self.detail
    }
}

/// Per-signal ingest queue watermark plus accepted batch count (the kept leg
/// of rate attribution: accepted vs dropped-by-reason).
#[derive(Debug, Clone)]
pub struct IngestQueue {
    pub signal: String,
    pub depth: usize,
    pub capacity: usize,
    pub high_water: usize,
    pub accepted: u64,
}

#[graphql_object(context = ApiContext)]
impl IngestQueue {
    fn signal(&self) -> &str {
        &self.signal
    }
    fn depth(&self) -> i32 {
        saturate_i32(self.depth as u64)
    }
    fn capacity(&self) -> i32 {
        saturate_i32(self.capacity as u64)
    }
    fn high_water(&self) -> i32 {
        saturate_i32(self.high_water as u64)
    }
    fn accepted(&self) -> String {
        self.accepted.to_string()
    }
}

/// Ingest signals in pipeline order.
pub(crate) const SIGNALS: [&str; 4] = ["traces", "logs", "metrics", "sentry"];

fn validate_signal(signal: Option<String>) -> FieldResult<Option<String>> {
    signal
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            if SIGNALS.contains(&normalized.as_str()) {
                Ok(normalized)
            } else {
                Err(field_err(format!(
                    "unknown signal {value:?}; expected one of traces, logs, metrics, sentry"
                )))
            }
        })
        .transpose()
}

pub(crate) async fn sampling_policy(
    context: &ApiContext,
    service: Option<String>,
    signal: Option<String>,
) -> FieldResult<Vec<SamplingPolicy>> {
    let Some(pipeline) = &context.pipeline else {
        return Ok(Vec::new());
    };
    let signal = validate_signal(signal)?;
    // Policy rows are global (service None = applies to all services); a
    // service filter keeps the rows that apply to the requested service.
    Ok(pipeline
        .sampling_policies()
        .into_iter()
        .filter(|policy| signal.as_ref().is_none_or(|want| &policy.signal == want))
        .filter(|policy| {
            service
                .as_ref()
                .is_none_or(|want| policy.service.as_ref().is_none_or(|have| have == want))
        })
        .collect())
}

pub(crate) async fn ingest_drops(
    context: &ApiContext,
    signal: Option<String>,
) -> FieldResult<Vec<IngestDrop>> {
    let Some(pipeline) = &context.pipeline else {
        return Ok(Vec::new());
    };
    let signal = validate_signal(signal)?;
    Ok(pipeline
        .ingest_drops()
        .into_iter()
        .filter(|drop| {
            signal
                .as_ref()
                .is_none_or(|want| drop.signal.as_ref().is_none_or(|have| have == want))
        })
        .collect())
}

pub(crate) async fn ingest_queues(context: &ApiContext) -> FieldResult<Vec<IngestQueue>> {
    Ok(context
        .pipeline
        .as_ref()
        .map_or(Vec::new(), |pipeline| pipeline.ingest_queues()))
}

#[cfg(test)]
mod tests;
