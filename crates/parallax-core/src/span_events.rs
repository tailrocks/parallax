//! Trace-wide span event parsing over normalized span rows.

use parallax_storage::model::SpanRow;
use serde_json::Value;

/// One parsed span event, joined to its carrying span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEvent {
    pub span_id: String,
    pub span_name: String,
    pub service: String,
    pub name: String,
    pub time_unix_nano: u128,
    /// Flat string map; non-string values JSON-stringified.
    pub attributes: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEvents {
    pub events: Vec<TraceEvent>,
    pub skipped_spans: usize,
    pub total_matching: usize,
}

impl TraceEvents {
    #[must_use]
    pub fn truncated(&self) -> bool {
        self.total_matching > self.events.len()
    }
}

/// Parse every span's `events` JSON, filter, sort by time ascending, cap.
///
/// Malformed JSON or malformed event-array shape on one span is skipped and
/// counted; individual malformed events are skipped without failing the span.
pub fn trace_events(spans: &[SpanRow], name_prefix: Option<&str>, limit: usize) -> TraceEvents {
    let mut events = Vec::new();
    let mut skipped_spans = 0;

    for span in spans {
        let Some(raw_events) = span.events.as_deref() else {
            continue;
        };
        let parsed: Value = if let Ok(value) = serde_json::from_str(raw_events) {
            value
        } else {
            skipped_spans += 1;
            continue;
        };
        let Some(array) = parsed.as_array() else {
            skipped_spans += 1;
            continue;
        };

        for event in array {
            let Some(name) = event.get("name").and_then(Value::as_str) else {
                continue;
            };
            if name_prefix.is_some_and(|prefix| !name.starts_with(prefix)) {
                continue;
            }
            let Some(time_unix_nano) = event_time_unix_nano(event) else {
                continue;
            };
            let attributes = event
                .get("attributes")
                .and_then(Value::as_object)
                .map(|object| {
                    let mut attributes: Vec<_> = object
                        .iter()
                        .map(|(key, value)| {
                            (
                                key.clone(),
                                value
                                    .as_str()
                                    .map_or_else(|| value.to_string(), str::to_string),
                            )
                        })
                        .collect();
                    attributes.sort_by(|(a, _), (b, _)| a.cmp(b));
                    attributes
                })
                .unwrap_or_default();
            events.push(TraceEvent {
                span_id: span.span_id.clone(),
                span_name: span.name.clone(),
                service: span.service.clone(),
                name: name.to_string(),
                time_unix_nano,
                attributes,
            });
        }
    }

    events.sort_by(|left, right| {
        left.time_unix_nano
            .cmp(&right.time_unix_nano)
            .then_with(|| left.span_id.cmp(&right.span_id))
            .then_with(|| left.name.cmp(&right.name))
    });
    let total_matching = events.len();
    events.truncate(limit);

    TraceEvents {
        events,
        skipped_spans,
        total_matching,
    }
}

fn event_time_unix_nano(event: &Value) -> Option<u128> {
    event
        .get("time_unix_nano")
        .or_else(|| event.get("timeUnixNano"))
        .and_then(|value| {
            value
                .as_u64()
                .map(u128::from)
                .or_else(|| value.as_str().and_then(|text| text.parse().ok()))
        })
        .or_else(|| {
            event
                .get("time")
                .and_then(Value::as_str)
                .and_then(parse_greptime_event_time)
        })
}

fn parse_greptime_event_time(value: &str) -> Option<u128> {
    let bytes = value.as_bytes();
    if bytes.len() < 19 {
        return None;
    }
    let year: i32 = value.get(0..4)?.parse().ok()?;
    let month: u32 = value.get(5..7)?.parse().ok()?;
    let day: u32 = value.get(8..10)?.parse().ok()?;
    let hour: u32 = value.get(11..13)?.parse().ok()?;
    let minute: u32 = value.get(14..16)?.parse().ok()?;
    let second: u32 = value.get(17..19)?.parse().ok()?;
    if bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || !matches!(bytes.get(10), Some(b' ' | b'T'))
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
        || !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return None;
    }

    let mut index = 19;
    let mut nanos = 0u32;
    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        let digits = value.get(start..index)?;
        if !digits.is_empty() {
            let significant = &digits[..digits.len().min(9)];
            nanos = significant.parse::<u32>().ok()?
                * 10u32.pow(u32::try_from(9usize.saturating_sub(significant.len())).ok()?);
        }
    }

    let offset_seconds = match bytes.get(index) {
        Some(b'Z') => 0i64,
        Some(sign @ (b'+' | b'-')) => {
            let hours: i64 = value.get(index + 1..index + 3)?.parse().ok()?;
            let minutes: i64 = value.get(index + 3..index + 5)?.parse().ok()?;
            if hours > 23 || minutes > 59 {
                return None;
            }
            let seconds = hours * 3600 + minutes * 60;
            if *sign == b'+' { seconds } else { -seconds }
        }
        _ => 0,
    };

    let days = days_from_civil(year, month, day)?;
    let local_seconds = days
        .checked_mul(86_400)?
        .checked_add(i64::from(hour) * 3600 + i64::from(minute) * 60 + i64::from(second))?;
    let utc_seconds = local_seconds.checked_sub(offset_seconds)?;
    if utc_seconds < 0 {
        return None;
    }
    Some(u128::try_from(utc_seconds).ok()? * 1_000_000_000 + u128::from(nanos))
}

fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    let year = i64::from(year) - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month = i64::from(month);
    let day = i64::from(day);
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146_097 + day_of_era - 719_468;
    (days >= 0).then_some(days)
}

#[cfg(test)]
mod tests;
