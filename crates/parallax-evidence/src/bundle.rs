//! The evidence bundle: bounded, redacted, hypothesis-ranked context for one
//! issue — graduated from `poc/evidence-loop` (bundle/bound/redact/hypothesis
//! kernels) onto the live row model. The same JSON powers the GraphQL
//! `bundle` field, the CLI's `issue context`, and the UI's bundle preview.

use parallax_model::{ErrorEventRow, InvocationRecord, Issue, LogRow, SpanRow};
use regex::Regex;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::OnceLock};

mod assembly;
mod bounding;
mod hash;
mod markdown;
mod ranking;
mod redaction;

pub use assembly::{BundleAnchor, BundleInputs, assemble};
use bounding::*;
use hash::*;
pub use markdown::to_markdown;
use ranking::*;
use redaction::*;

pub const SCHEMA_VERSION: &str = "bundle-v1";

#[derive(Debug, Serialize)]
pub struct Bundle {
    pub schema_version: &'static str,
    pub generator: &'static str,
    pub anchor: Anchor,
    /// The primary grouped issue — always present for issue anchors; for
    /// invocation/trace anchors it is the issue behind the newest error
    /// event, when any error occurred at all.
    pub issue: Option<IssueSummary>,
    /// Invocation context for invocation anchors (spec §8 `bundle(invocationId:)`).
    pub invocation: Option<InvocationSection>,
    pub latest_event: Option<EventDetail>,
    pub trace: Option<TraceSection>,
    /// Correlated metric slices around the anchor (spec §8: trace + logs +
    /// metric windows together).
    pub metric_windows: Vec<MetricWindow>,
    pub logs: Vec<String>,
    pub hypotheses: Vec<Hypothesis>,
    pub missing_evidence: Vec<String>,
    pub redaction: RedactionReport,
    pub bounded: BoundReport,
    pub canonical_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Anchor {
    pub kind: &'static str,
    /// The anchoring identifier: issue fingerprint, invocation id, or trace id.
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct InvocationSection {
    pub invocation_id: String,
    pub command: Option<String>,
    pub app_mode: Option<String>,
    pub outcome: Option<String>,
    pub status: String,
    pub exit_code: Option<i32>,
    pub started_at_nanos: String,
    pub ended_at_nanos: Option<String>,
    /// Every grouped issue whose events fell inside this invocation's traces.
    pub issues: Vec<IssueSummary>,
}

#[derive(Debug, Serialize)]
pub struct IssueSummary {
    pub title: String,
    pub error_type: String,
    pub culprit: Option<String>,
    pub service: String,
    pub status: String,
    pub event_count: u64,
    pub first_seen_nanos: String,
    pub last_seen_nanos: String,
}

#[derive(Debug, Serialize)]
pub struct EventDetail {
    pub ts_nanos: String,
    pub message: String,
    pub stacktrace: Option<String>,
    pub source: String,
    pub trace_id: String,
}

#[derive(Debug, Serialize)]
pub struct TraceSection {
    pub trace_id: String,
    pub spans: Vec<SpanLine>,
}

/// One correlated metric slice around the anchor — the bundle's
/// trace+logs+**metric window** promise (spec §8 correlation sections).
#[derive(Debug, Serialize)]
pub struct MetricWindow {
    pub metric: String,
    /// "invocation" (points tagged with the anchor's invocation id) or "service".
    pub scope: &'static str,
    pub from_nanos: String,
    pub to_nanos: String,
    pub step_seconds: u32,
    pub points: Vec<MetricPointLine>,
    pub stats: MetricStats,
}

#[derive(Debug, Serialize)]
pub struct MetricPointLine {
    pub ts_nanos: String,
    pub value: f64,
}

#[derive(Debug, Serialize)]
pub struct MetricStats {
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub last: f64,
}

/// Cap per metric window — keeps the section bounded before token bounding.
pub const METRIC_WINDOW_MAX_POINTS: usize = 60;

impl MetricWindow {
    /// Build a window from raw points (nanos, value), computing stats and
    /// enforcing the point cap (oldest dropped first — the anchor sits at
    /// the window's end).
    pub fn from_points(
        metric: impl Into<String>,
        scope: &'static str,
        from_nanos: u128,
        to_nanos: u128,
        step_seconds: u32,
        mut points: Vec<(u128, f64)>,
    ) -> Option<Self> {
        if points.is_empty() {
            return None;
        }
        points.sort_by_key(|(ts, _)| *ts);
        if points.len() > METRIC_WINDOW_MAX_POINTS {
            points.drain(..points.len() - METRIC_WINDOW_MAX_POINTS);
        }
        let values: Vec<f64> = points.iter().map(|(_, v)| *v).collect();
        let min = values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let avg = values.iter().sum::<f64>() / values.len() as f64;
        let last = *values.last().unwrap_or(&0.0);
        Some(Self {
            metric: metric.into(),
            scope,
            from_nanos: from_nanos.to_string(),
            to_nanos: to_nanos.to_string(),
            step_seconds,
            points: points
                .into_iter()
                .map(|(ts, value)| MetricPointLine {
                    ts_nanos: ts.to_string(),
                    value,
                })
                .collect(),
            stats: MetricStats {
                min,
                max,
                avg,
                last,
            },
        })
    }
}

#[derive(Debug, Serialize)]
pub struct SpanLine {
    pub service: String,
    pub name: String,
    pub kind: String,
    pub status_code: String,
    pub duration_us: u128,
    pub db_query: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Hypothesis {
    pub kind: &'static str,
    pub statement: String,
    pub confidence: &'static str,
    pub evidence: Vec<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct RedactionReport {
    pub policy: &'static str,
    pub redacted_counts: BTreeMap<&'static str, u64>,
}

#[derive(Debug, Default, Serialize)]
pub struct BoundReport {
    pub max_tokens: usize,
    pub estimated_tokens: usize,
    pub dropped_log_lines: usize,
    pub truncated_stacktrace: bool,
}

#[cfg(test)]
mod tests;
