use std::collections::{HashMap, HashSet};

use parallax_storage::model;

use super::InvocationStats;
use crate::MAX_ROWS;

pub(super) fn invocation_stats_from_spans(
    spans: &[model::SpanRow],
    events_by_trace: &HashMap<String, Vec<model::ErrorEventRow>>,
) -> InvocationStats {
    let mut trace_ids: Vec<String> = Vec::new();
    let mut seen_trace_ids = HashSet::new();
    let mut last_span_nanos = 0;
    for span in spans {
        last_span_nanos = last_span_nanos.max(span.ts_nanos);
        let trace_id = span.trace_id.clone();
        if seen_trace_ids.insert(trace_id.clone()) {
            trace_ids.push(trace_id);
        }
    }
    let mut events: Vec<model::ErrorEventRow> = Vec::new();
    for trace_id in &trace_ids {
        if let Some(trace_events) = events_by_trace.get(trace_id) {
            events.extend(trace_events.iter().cloned());
        }
    }
    events.sort_by_key(|event| std::cmp::Reverse(event.ts_nanos));
    events.truncate(MAX_ROWS);
    InvocationStats {
        trace_ids,
        events,
        last_span_nanos,
        command_completion: command_completion(spans),
    }
}

/// Latest completed root `cli.command` span carrying an `outcome` attribute:
/// the observed end of an external (non-wrapper-registered) invocation. The
/// outcome attribute is recorded at span completion, so its presence means
/// the command finished.
pub(super) fn command_completion(spans: &[model::SpanRow]) -> Option<(u128, String)> {
    // A daemon's lifecycle is not command-derived: its capsule children
    // complete root command spans while the daemon keeps running, so any
    // daemon-mode signal disables the derivation (wrapper registration or
    // staleness closes daemons).
    let daemon = spans.iter().any(|span| {
        span.attributes
            .get(parallax_analysis::semconv::APP_MODE)
            .and_then(|value| value.as_str())
            == Some("daemon")
    });
    if daemon {
        return None;
    }
    spans
        .iter()
        .filter(|span| {
            span.name == parallax_analysis::semconv::CLI_COMMAND_SPAN_NAME
                && span.parent_span_id.as_deref().is_none_or(str::is_empty)
        })
        .filter_map(|span| {
            // A capsule child is wrapped INSIDE another invocation; its
            // completion never closes the surrounding invocation.
            if span
                .attributes
                .get(parallax_analysis::semconv::APP_MODE)
                .and_then(|value| value.as_str())
                == Some("capsule")
            {
                return None;
            }
            let outcome = span
                .attributes
                .get(parallax_analysis::semconv::OUTCOME)
                .and_then(|value| value.as_str())?;
            Some((span.ts_nanos + span.duration_ns, outcome.to_string()))
        })
        .max_by_key(|(end, _)| *end)
}
