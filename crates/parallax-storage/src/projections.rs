//! Pure query-time projections over normalized native-table rows.
//!
//! Sessions, screen visits, actions, cycles, jobs, and conversations are
//! derived — never stored (decision: new signal families are query-time
//! projections over native tables). Adapters fetch bounded row windows and
//! share this pairing/aggregation logic so the in-memory and GreptimeDB
//! adapters cannot diverge.

use crate::adapter::{
    BackgroundCycleSummary, ConversationSummary, InvocationSession, JobAttempt, JobSummary,
    ReleaseHealth, RumSession, RumSessionDetail, RumSessionError, RumSessionPageView,
    RumSessionVital, ScreenVisit, UiAction,
};
use parallax_model::{LogRow, SpanRow};
use parallax_semconv as semconv;
use std::collections::{BTreeMap, BTreeSet};

fn attr_str<'a>(attributes: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    attributes.get(key).and_then(|value| value.as_str())
}

fn attr_i64(attributes: &serde_json::Value, key: &str) -> Option<i64> {
    let value = attributes.get(key)?;
    value
        .as_i64()
        .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
}

fn attr_f64(attributes: &serde_json::Value, key: &str) -> Option<f64> {
    let value = attributes.get(key)?;
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|s| s.parse().ok()))
}

fn log_session_id(row: &LogRow) -> Option<String> {
    row.session_id
        .clone()
        .or_else(|| attr_str(&row.attributes, semconv::SESSION_ID).map(str::to_string))
        .filter(|value| !value.is_empty())
}

/// Pair `session.start` / `session.end` log events into sessions, oldest
/// first. An unmatched start stays open (`end_nanos = None`).
#[must_use]
pub fn pair_sessions(rows: &[LogRow], limit: usize) -> Vec<InvocationSession> {
    let mut ordered: Vec<&LogRow> = rows
        .iter()
        .filter(|row| {
            row.event_name == semconv::SESSION_START_EVENT_NAME
                || row.event_name == semconv::SESSION_END_EVENT_NAME
        })
        .collect();
    ordered.sort_by_key(|row| row.ts_nanos);
    let mut sessions: Vec<InvocationSession> = Vec::new();
    for row in ordered {
        let Some(session_id) = log_session_id(row) else {
            continue;
        };
        if row.event_name == semconv::SESSION_START_EVENT_NAME {
            sessions.push(InvocationSession {
                session_id,
                previous_session_id: attr_str(&row.attributes, semconv::SESSION_PREVIOUS_ID)
                    .map(str::to_string),
                start_nanos: row.ts_nanos,
                end_nanos: None,
            });
        } else if let Some(session) = sessions
            .iter_mut()
            .rev()
            .find(|session| session.session_id == session_id && session.end_nanos.is_none())
        {
            session.end_nanos = Some(row.ts_nanos);
        }
    }
    sessions.truncate(limit);
    sessions
}

/// Pair `ui.screen.entered` / `ui.screen.exited` events by
/// `ui.screen.visit.id` into visits, navigation order ascending.
#[must_use]
pub fn pair_screen_visits(
    rows: &[LogRow],
    session_id: Option<&str>,
    limit: usize,
) -> Vec<ScreenVisit> {
    let mut ordered: Vec<&LogRow> = rows
        .iter()
        .filter(|row| {
            row.event_name == semconv::UI_SCREEN_ENTERED_EVENT_NAME
                || row.event_name == semconv::UI_SCREEN_EXITED_EVENT_NAME
        })
        .filter(|row| {
            session_id.is_none_or(|session_id| log_session_id(row).as_deref() == Some(session_id))
        })
        .collect();
    ordered.sort_by_key(|row| row.ts_nanos);
    let mut visits: Vec<ScreenVisit> = Vec::new();
    let mut open: BTreeMap<String, usize> = BTreeMap::new();
    for row in ordered {
        let Some(visit_id) = attr_str(&row.attributes, semconv::UI_SCREEN_VISIT_ID) else {
            continue;
        };
        if row.event_name == semconv::UI_SCREEN_ENTERED_EVENT_NAME {
            open.insert(visit_id.to_string(), visits.len());
            visits.push(ScreenVisit {
                screen_id: attr_str(&row.attributes, semconv::APP_SCREEN_ID)
                    .unwrap_or_default()
                    .to_string(),
                visit_id: visit_id.to_string(),
                session_id: log_session_id(row),
                navigation_sequence: attr_i64(&row.attributes, semconv::UI_NAVIGATION_SEQUENCE),
                transition_reason: attr_str(&row.attributes, semconv::UI_TRANSITION_REASON)
                    .map(str::to_string),
                entered_nanos: row.ts_nanos,
                exited_nanos: None,
            });
        } else if let Some(index) = open.remove(visit_id) {
            visits[index].exited_nanos = Some(row.ts_nanos);
        }
    }
    visits.sort_by_key(|visit| {
        (
            visit.navigation_sequence.unwrap_or(i64::MAX),
            visit.entered_nanos,
        )
    });
    visits.truncate(limit);
    visits
}

fn is_root(span: &SpanRow) -> bool {
    span.parent_span_id.as_deref().is_none_or(str::is_empty)
}

fn span_has_error(span: &SpanRow) -> bool {
    span.status_code == "STATUS_CODE_ERROR"
}

/// `ui.action` root spans, newest first.
#[must_use]
pub fn project_ui_actions(spans: &[SpanRow], limit: usize) -> Vec<UiAction> {
    let mut actions: Vec<UiAction> = spans
        .iter()
        .filter(|span| span.name == semconv::UI_ACTION_SPAN_NAME && is_root(span))
        .map(|span| UiAction {
            name: attr_str(&span.attributes, semconv::UI_ACTION_NAME)
                .unwrap_or(&span.name)
                .to_string(),
            screen_id: attr_str(&span.attributes, semconv::APP_SCREEN_ID).map(str::to_string),
            widget_name: attr_str(&span.attributes, semconv::APP_WIDGET_NAME).map(str::to_string),
            session_id: span
                .session_id
                .clone()
                .or_else(|| attr_str(&span.attributes, semconv::SESSION_ID).map(str::to_string)),
            trace_id: span.trace_id.clone(),
            start_nanos: span.ts_nanos,
            duration_ns: span.duration_ns,
            outcome: attr_str(&span.attributes, semconv::OUTCOME).map(str::to_string),
            has_error: span_has_error(span),
        })
        .collect();
    actions.sort_by_key(|action| std::cmp::Reverse(action.start_nanos));
    actions.truncate(limit);
    actions
}

#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "bounded display aggregate over MAX_ROWS-sized windows"
)]
fn percentile(sorted_ns: &[u128], q: f64) -> Option<f64> {
    if sorted_ns.is_empty() {
        return None;
    }
    // Standard nearest-rank: ceil(q * N) - 1 (1-indexed rank to 0-index).
    let rank = ((q * sorted_ns.len() as f64).ceil() as usize).max(1) - 1;
    sorted_ns
        .get(rank.min(sorted_ns.len() - 1))
        .map(|v| *v as f64)
}

/// `background.cycle` spans grouped by `background.cycle.name`, most recent
/// activity first.
#[must_use]
pub fn summarize_background_cycles(spans: &[SpanRow], limit: usize) -> Vec<BackgroundCycleSummary> {
    let mut groups: BTreeMap<String, Vec<&SpanRow>> = BTreeMap::new();
    for span in spans
        .iter()
        .filter(|span| span.name == semconv::BACKGROUND_CYCLE_SPAN_NAME)
    {
        let name = attr_str(&span.attributes, semconv::BACKGROUND_CYCLE_NAME)
            .unwrap_or("(unnamed)")
            .to_string();
        groups.entry(name).or_default().push(span);
    }
    let mut cycles: Vec<BackgroundCycleSummary> = groups
        .into_iter()
        .filter_map(|(name, spans)| {
            let mut durations: Vec<u128> = spans.iter().map(|span| span.duration_ns).collect();
            durations.sort_unstable();
            let last = spans.iter().max_by_key(|span| span.ts_nanos)?;
            Some(BackgroundCycleSummary {
                name,
                count: spans.len() as u64,
                error_count: spans.iter().filter(|span| span_has_error(span)).count() as u64,
                p50_ns: percentile(&durations, 0.50),
                p95_ns: percentile(&durations, 0.95),
                last_nanos: last.ts_nanos,
                last_trace_id: last.trace_id.clone(),
            })
        })
        .collect();
    cycles.sort_by_key(|cycle| std::cmp::Reverse(cycle.last_nanos));
    cycles.truncate(limit);
    cycles
}

/// Spans carrying `job.id`, grouped into producer time + consumer attempts,
/// newest activity first.
#[must_use]
pub fn summarize_jobs(spans: &[SpanRow], limit: usize) -> Vec<JobSummary> {
    let mut groups: BTreeMap<String, Vec<&SpanRow>> = BTreeMap::new();
    for span in spans {
        if let Some(job_id) = attr_str(&span.attributes, semconv::JOB_ID) {
            groups.entry(job_id.to_string()).or_default().push(span);
        }
    }
    let mut jobs: Vec<JobSummary> = groups
        .into_iter()
        .filter_map(|(job_id, mut spans)| {
            spans.sort_by_key(|span| span.ts_nanos);
            let last = spans.last()?;
            Some(JobSummary {
                job_id,
                job_type: spans
                    .iter()
                    .find_map(|span| attr_str(&span.attributes, semconv::JOB_TYPE))
                    .map(str::to_string),
                produced_nanos: spans
                    .iter()
                    .find(|span| span.kind == "SPAN_KIND_PRODUCER")
                    .map(|span| span.ts_nanos),
                attempts: spans
                    .iter()
                    .filter(|span| span.kind == "SPAN_KIND_CONSUMER")
                    .map(|span| JobAttempt {
                        start_nanos: span.ts_nanos,
                        duration_ns: span.duration_ns,
                        outcome: attr_str(&span.attributes, semconv::OUTCOME).map(str::to_string),
                        has_error: span_has_error(span),
                        trace_id: span.trace_id.clone(),
                    })
                    .collect(),
                last_trace_id: last.trace_id.clone(),
            })
        })
        .collect();
    jobs.sort_by_key(|job| {
        std::cmp::Reverse(
            job.attempts
                .iter()
                .map(|attempt| attempt.start_nanos)
                .max()
                .or(job.produced_nanos)
                .unwrap_or(0),
        )
    });
    jobs.truncate(limit);
    jobs
}

/// Spans carrying `gen_ai.conversation.id`, summarized per conversation,
/// newest activity first.
#[must_use]
pub fn summarize_conversations(spans: &[SpanRow], limit: usize) -> Vec<ConversationSummary> {
    let mut groups: BTreeMap<String, Vec<&SpanRow>> = BTreeMap::new();
    for span in spans {
        if let Some(id) = attr_str(&span.attributes, semconv::GEN_AI_CONVERSATION_ID) {
            groups.entry(id.to_string()).or_default().push(span);
        }
    }
    let mut conversations: Vec<ConversationSummary> = groups
        .into_iter()
        .map(|(conversation_id, spans)| {
            let sum_tokens = |key: &str| {
                let values: Vec<f64> = spans
                    .iter()
                    .filter_map(|span| attr_f64(&span.attributes, key))
                    .collect();
                (!values.is_empty()).then(|| values.iter().sum())
            };
            ConversationSummary {
                conversation_id,
                agent_name: spans
                    .iter()
                    .find_map(|span| attr_str(&span.attributes, semconv::GEN_AI_AGENT_NAME))
                    .map(str::to_string),
                provider_name: spans
                    .iter()
                    .find_map(|span| attr_str(&span.attributes, semconv::GEN_AI_PROVIDER_NAME))
                    .map(str::to_string),
                first_nanos: spans.iter().map(|span| span.ts_nanos).min().unwrap_or(0),
                last_nanos: spans.iter().map(|span| span.ts_nanos).max().unwrap_or(0),
                span_count: spans.len() as u64,
                input_tokens: sum_tokens(semconv::GEN_AI_USAGE_INPUT_TOKENS),
                output_tokens: sum_tokens(semconv::GEN_AI_USAGE_OUTPUT_TOKENS),
            }
        })
        .collect();
    conversations.sort_by_key(|conversation| std::cmp::Reverse(conversation.last_nanos));
    conversations.truncate(limit);
    conversations
}

/// Flag suspect releases: row `i > 0` is suspect when its session crash
/// rate strictly regresses versus the previous release. Rows must arrive
/// ordered oldest-first (both engines sort by `(first_seen, version)`,
/// the `release_windows` order); the first release is never suspect.
/// Shared so the in-memory and GreptimeDB adapters cannot diverge.
pub fn flag_suspect_releases(rows: &mut [ReleaseHealth]) {
    let mut previous_rate = 0.0f64;
    let mut first = true;
    for row in rows {
        let rate = row.crash_rate();
        row.suspect_release = !first && rate > previous_rate;
        previous_rate = rate;
        first = false;
    }
}

fn rum_session_id(span: &SpanRow) -> Option<&str> {
    span.session_id
        .as_deref()
        .or_else(|| attr_str(&span.attributes, semconv::SESSION_ID))
        .filter(|value| !value.is_empty())
}

/// Browser RUM sessions: spans grouped by `session.id`, newest activity
/// first. `service` matches spans of that service before grouping;
/// `error_only` keeps sessions with at least one error span.
#[must_use]
pub fn summarize_rum_sessions(
    spans: &[SpanRow],
    service: Option<&str>,
    error_only: bool,
    limit: usize,
) -> Vec<RumSession> {
    let mut groups: BTreeMap<&str, Vec<&SpanRow>> = BTreeMap::new();
    for span in spans {
        if service.is_some_and(|service| span.service != service) {
            continue;
        }
        if let Some(session_id) = rum_session_id(span) {
            groups.entry(session_id).or_default().push(span);
        }
    }
    let mut sessions: Vec<RumSession> = groups
        .into_iter()
        .map(|(session_id, spans)| {
            let mut traces: BTreeSet<&str> = BTreeSet::new();
            let mut service_name = String::new();
            let mut start_nanos = u128::MAX;
            let mut end_nanos = 0;
            let mut view_count = 0u64;
            let mut vital_count = 0u64;
            let mut error_count = 0u64;
            for span in &spans {
                traces.insert(span.trace_id.as_str());
                if span.service > service_name {
                    service_name.clone_from(&span.service);
                }
                start_nanos = start_nanos.min(span.ts_nanos);
                end_nanos = end_nanos.max(span.ts_nanos);
                if span.name == semconv::APP_SCREEN_NAME {
                    view_count += 1;
                }
                if span.name == semconv::BROWSER_WEB_VITAL {
                    vital_count += 1;
                }
                if span_has_error(span) {
                    error_count += 1;
                }
            }
            RumSession {
                session_id: session_id.to_string(),
                service: service_name,
                start_nanos,
                end_nanos,
                span_count: spans.len() as u64,
                trace_count: traces.len() as u64,
                view_count,
                vital_count,
                error_count,
                has_error: error_count > 0,
            }
        })
        .filter(|session| !error_only || session.has_error)
        .collect();
    sessions.sort_by_key(|session| std::cmp::Reverse(session.end_nanos));
    sessions.truncate(limit);
    sessions
}

/// One RUM session's timeline from its spans: page views (`app.screen.name`
/// spans), vitals (`browser.web_vital` spans), and errors (ERROR-status
/// spans), each time ascending and truncated to `limit`. `None` when no
/// span carries the session id.
#[must_use]
pub fn project_rum_session_detail(
    spans: &[SpanRow],
    session_id: &str,
    limit: usize,
) -> Option<RumSessionDetail> {
    let scoped: Vec<&SpanRow> = spans
        .iter()
        .filter(|span| rum_session_id(span) == Some(session_id))
        .collect();
    if scoped.is_empty() {
        return None;
    }
    let mut views: Vec<RumSessionPageView> = Vec::new();
    let mut vitals: Vec<RumSessionVital> = Vec::new();
    let mut errors: Vec<RumSessionError> = Vec::new();
    for span in &scoped {
        if span.name == semconv::APP_SCREEN_NAME {
            views.push(RumSessionPageView {
                ts_nanos: span.ts_nanos,
                screen: attr_str(&span.attributes, semconv::APP_SCREEN_NAME)
                    .unwrap_or_default()
                    .to_string(),
                path: attr_str(&span.attributes, semconv::URL_PATH).map(str::to_string),
                trace_id: span.trace_id.clone(),
                span_id: span.span_id.clone(),
            });
        } else if span.name == semconv::BROWSER_WEB_VITAL {
            vitals.push(RumSessionVital {
                ts_nanos: span.ts_nanos,
                name: attr_str(&span.attributes, semconv::WEB_VITAL_NAME)
                    .unwrap_or_default()
                    .to_string(),
                value: attr_f64(&span.attributes, semconv::WEB_VITAL_VALUE).unwrap_or(0.0),
                rating: attr_str(&span.attributes, semconv::WEB_VITAL_RATING).map(str::to_string),
                trace_id: span.trace_id.clone(),
                span_id: span.span_id.clone(),
            });
        }
        if span_has_error(span) {
            errors.push(RumSessionError {
                ts_nanos: span.ts_nanos,
                name: span.name.clone(),
                error_type: attr_str(&span.attributes, semconv::ERROR_TYPE)
                    .or_else(|| attr_str(&span.attributes, semconv::EXCEPTION_TYPE))
                    .map(str::to_string),
                message: span.status_message.clone(),
                trace_id: span.trace_id.clone(),
                span_id: span.span_id.clone(),
            });
        }
    }
    views.sort_by_key(|view| view.ts_nanos);
    views.truncate(limit);
    vitals.sort_by_key(|vital| vital.ts_nanos);
    vitals.truncate(limit);
    errors.sort_by_key(|error| error.ts_nanos);
    errors.truncate(limit);
    let summary = summarize_rum_sessions(spans, None, false, usize::MAX)
        .into_iter()
        .find(|session| session.session_id == session_id)?;
    Some(RumSessionDetail {
        session: summary,
        views,
        vitals,
        errors,
    })
}

#[cfg(test)]
mod tests;
