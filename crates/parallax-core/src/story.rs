#![expect(clippy::too_many_lines, reason = "measured legacy story projection")]

//! Deterministic story timeline projection over telemetry rows.

use crate::bundle::MetricWindow;
use crate::fingerprint::normalize_message;
use crate::semconv;
use parallax_storage::model::{LogRow, SpanRow};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoryBeat {
    pub ts_nanos: u128,
    pub lane: String,
    pub kind: String,
    pub title: String,
    pub trace_id: String,
    pub span_id: Option<String>,
    pub severity: Option<String>,
    pub duration_ns: Option<u128>,
}

struct SortableBeat {
    beat: StoryBeat,
    sort_ts: u128,
    phase: u8,
    depth_key: usize,
    span_order: usize,
}

pub fn project_story(
    trace_spans: &[SpanRow],
    trace_logs: &[LogRow],
    _metric_windows: &[MetricWindow],
) -> Vec<StoryBeat> {
    let by_id: HashMap<String, &SpanRow> = trace_spans
        .iter()
        .map(|span| (span.span_id.clone(), span))
        .collect();
    let mut ordered_spans: Vec<&SpanRow> = trace_spans.iter().collect();
    ordered_spans.sort_by(|a, b| {
        a.ts_nanos
            .cmp(&b.ts_nanos)
            .then_with(|| a.span_id.cmp(&b.span_id))
    });
    let span_order: HashMap<String, usize> = ordered_spans
        .iter()
        .enumerate()
        .map(|(index, span)| (span.span_id.clone(), index))
        .collect();
    let mut start_memo = HashMap::new();
    let mut depth_memo = HashMap::new();
    let mut beats = Vec::new();

    for span in ordered_spans {
        let start = adjusted_start(span, &by_id, &mut start_memo);
        let depth = span_depth(span, &by_id, &mut depth_memo);
        let order = *span_order.get(&span.span_id).unwrap_or(&usize::MAX);
        let lane = lane_for(&span.service, &span.resource);
        beats.push(SortableBeat {
            beat: StoryBeat {
                ts_nanos: span.ts_nanos,
                lane: lane.clone(),
                kind: "span.start".to_string(),
                title: span.name.clone(),
                trace_id: span.trace_id.clone(),
                span_id: Some(span.span_id.clone()),
                severity: None,
                duration_ns: None,
            },
            sort_ts: start,
            phase: 0,
            depth_key: depth,
            span_order: order,
        });

        for (event_ts, title) in span_events(span) {
            beats.push(SortableBeat {
                beat: StoryBeat {
                    ts_nanos: event_ts,
                    lane: lane.clone(),
                    kind: "event".to_string(),
                    title,
                    trace_id: span.trace_id.clone(),
                    span_id: Some(span.span_id.clone()),
                    severity: None,
                    duration_ns: None,
                },
                sort_ts: event_ts.max(start),
                phase: 1,
                depth_key: depth,
                span_order: order,
            });
        }

        let end = span.ts_nanos.saturating_add(span.duration_ns);
        if span.status_code == "STATUS_CODE_ERROR" {
            beats.push(SortableBeat {
                beat: StoryBeat {
                    ts_nanos: end,
                    lane: lane.clone(),
                    kind: "error".to_string(),
                    title: format!("{} error", span.name),
                    trace_id: span.trace_id.clone(),
                    span_id: Some(span.span_id.clone()),
                    severity: Some("ERROR".to_string()),
                    duration_ns: Some(span.duration_ns),
                },
                sort_ts: end.max(start),
                phase: 2,
                depth_key: usize::MAX.saturating_sub(depth),
                span_order: order,
            });
        }
        beats.push(SortableBeat {
            beat: StoryBeat {
                ts_nanos: end,
                lane,
                kind: "span.end".to_string(),
                title: span.name.clone(),
                trace_id: span.trace_id.clone(),
                span_id: Some(span.span_id.clone()),
                severity: None,
                duration_ns: Some(span.duration_ns),
            },
            sort_ts: end.max(start),
            phase: 3,
            depth_key: usize::MAX.saturating_sub(depth),
            span_order: order,
        });
    }

    for (index, log) in trace_logs.iter().enumerate() {
        let span = by_id.get(&log.span_id);
        let start = span.map_or(log.ts_nanos, |span| {
            adjusted_start(span, &by_id, &mut start_memo)
        });
        let depth = span.map_or(usize::MAX, |span| span_depth(span, &by_id, &mut depth_memo));
        let order = span
            .and_then(|span| span_order.get(&span.span_id).copied())
            .unwrap_or(usize::MAX.saturating_sub(index));
        let is_error = log.severity_num >= 17 || log.severity_text.eq_ignore_ascii_case("ERROR");
        beats.push(SortableBeat {
            beat: StoryBeat {
                ts_nanos: log.ts_nanos,
                lane: lane_for(&log.service, &log.resource),
                kind: if is_error { "error" } else { "log" }.to_string(),
                title: log_title(log),
                trace_id: log.trace_id.clone(),
                span_id: non_empty(log.span_id.clone()),
                severity: non_empty(log.severity_text.clone()),
                duration_ns: None,
            },
            sort_ts: log.ts_nanos.max(start),
            phase: 1,
            depth_key: depth,
            span_order: order,
        });
    }

    beats.sort_by(|a, b| {
        a.sort_ts
            .cmp(&b.sort_ts)
            .then_with(|| a.phase.cmp(&b.phase))
            .then_with(|| a.depth_key.cmp(&b.depth_key))
            .then_with(|| a.span_order.cmp(&b.span_order))
            .then_with(|| a.beat.kind.cmp(&b.beat.kind))
            .then_with(|| a.beat.title.cmp(&b.beat.title))
            .then_with(|| a.beat.span_id.cmp(&b.beat.span_id))
    });
    beats.into_iter().map(|beat| beat.beat).collect()
}

fn adjusted_start(
    span: &SpanRow,
    by_id: &HashMap<String, &SpanRow>,
    memo: &mut HashMap<String, u128>,
) -> u128 {
    if let Some(start) = memo.get(&span.span_id) {
        return *start;
    }
    let parent_start = span
        .parent_span_id
        .as_deref()
        .and_then(|parent_id| by_id.get(parent_id))
        .map(|parent| adjusted_start(parent, by_id, memo))
        .unwrap_or(span.ts_nanos);
    let start = span.ts_nanos.max(parent_start);
    memo.insert(span.span_id.clone(), start);
    start
}

fn span_depth(
    span: &SpanRow,
    by_id: &HashMap<String, &SpanRow>,
    memo: &mut HashMap<String, usize>,
) -> usize {
    if let Some(depth) = memo.get(&span.span_id) {
        return *depth;
    }
    let depth = span
        .parent_span_id
        .as_deref()
        .and_then(|parent_id| by_id.get(parent_id))
        .map(|parent| span_depth(parent, by_id, memo).saturating_add(1))
        .unwrap_or(0);
    memo.insert(span.span_id.clone(), depth);
    depth
}

fn span_events(span: &SpanRow) -> Vec<(u128, String)> {
    let Some(raw) = span.events.as_deref() else {
        return Vec::new();
    };
    let Ok(Value::Array(events)) = serde_json::from_str::<Value>(raw) else {
        return Vec::new();
    };
    events
        .iter()
        .filter_map(|event| {
            let name = event.get("name")?.as_str()?.trim();
            if name.is_empty() {
                return None;
            }
            let ts = event
                .get("timeUnixNano")
                .or_else(|| event.get("time_unix_nano"))
                .and_then(nanos_value)
                .unwrap_or(span.ts_nanos);
            Some((ts, name.to_string()))
        })
        .collect()
}

fn nanos_value(value: &Value) -> Option<u128> {
    match value {
        Value::String(s) => s.parse().ok(),
        Value::Number(n) => n.as_u64().map(u128::from),
        _ => None,
    }
}

fn lane_for(service: &str, resource: &Value) -> String {
    for key in [semconv::PARALLAX_SOURCE, semconv::SERVICE_NAMESPACE] {
        if let Some(value) = resource
            .get(key)
            .and_then(Value::as_str)
            .filter(|value| matches!(*value, "browser" | "cli"))
        {
            return value.to_string();
        }
    }
    if !service.trim().is_empty() {
        return service.to_string();
    }
    resource
        .get(semconv::SERVICE_NAME)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("unknown")
        .to_string()
}

fn log_title(log: &LogRow) -> String {
    let severity = log
        .severity_text
        .trim()
        .to_ascii_uppercase()
        .if_empty("LOG");
    let first_line = log.body.lines().next().unwrap_or("").trim();
    if first_line.is_empty() {
        return format!("{severity} log");
    }
    let normalized = truncate_chars(&normalize_message(first_line), 96);
    format!("{severity} {normalized}")
}

fn truncate_chars(value: &str, limit: usize) -> String {
    let mut chars = value.chars();
    let truncated: String = chars.by_ref().take(limit).collect();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

fn non_empty(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

trait EmptyFallback {
    fn if_empty(self, fallback: &'static str) -> String;
}

impl EmptyFallback for String {
    fn if_empty(self, fallback: &'static str) -> String {
        if self.is_empty() {
            fallback.to_string()
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests;
