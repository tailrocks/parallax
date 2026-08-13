//! Pure delivery helpers for alerting v1 (plan 167 step 3).
//!
//! Webhook/Slack payload shapes, unique delivery keys, exponential backoff,
//! and claim-lease due checks. No I/O — the delivery worker (peer) owns
//! reqwest + outbox claim loops. Unit-tested below.

use std::time::Duration;

/// Outbox event kinds enqueued by the evaluator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum DeliveryEventType {
    Triggered,
    Resolved,
    Renotify,
}

impl DeliveryEventType {
    #[must_use]
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Triggered => "triggered",
            Self::Resolved => "resolved",
            Self::Renotify => "renotify",
        }
    }

    #[must_use]
    pub(crate) fn parse(s: &str) -> Option<Self> {
        match s {
            "triggered" => Some(Self::Triggered),
            "resolved" => Some(Self::Resolved),
            "renotify" => Some(Self::Renotify),
            _ => None,
        }
    }
}

/// Inputs for a single notification payload (rule + incident snapshot).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NotificationContext<'a> {
    pub rule_id: &'a str,
    pub rule_name: &'a str,
    pub signal_type: &'a str,
    pub severity: &'a str,
    pub group_key: &'a str,
    pub incident_id: &'a str,
    pub event_type: DeliveryEventType,
    pub observed_value: Option<f64>,
    pub threshold: f64,
    pub threshold_upper: Option<f64>,
    pub window_minutes: u32,
    /// Absolute UI link to the incident (or rule detail).
    pub incident_url: &'a str,
    /// Absolute UI link to scoped traces/logs for investigation.
    pub investigate_url: &'a str,
}

/// Stable unique delivery key: one successful delivery per
/// (incident, destination, event type) — prevents double-delivery.
#[must_use]
pub(crate) fn unique_delivery_key(
    incident_id: &str,
    destination_id: &str,
    event_type: DeliveryEventType,
) -> String {
    format!("{incident_id}|{destination_id}|{}", event_type.as_str())
}

/// Generic webhook JSON body (documented plan-167 API shape).
/// Keys are stable for external integrators.
#[must_use]
pub(crate) fn webhook_payload_json(ctx: &NotificationContext<'_>) -> String {
    let value = match ctx.observed_value {
        Some(v) => format!("{v}"),
        None => "null".to_string(),
    };
    let upper = match ctx.threshold_upper {
        Some(u) => format!("{u}"),
        None => "null".to_string(),
    };
    // Hand-built JSON keeps this module free of serde dependency choices;
    // strings are escaped for quotes/backslashes only (V1 local-operator URLs).
    format!(
        concat!(
            "{{",
            "\"event\":\"{}\",",
            "\"rule\":{{\"id\":\"{}\",\"name\":\"{}\",\"signal_type\":\"{}\",\"severity\":\"{}\"}},",
            "\"incident\":{{\"id\":\"{}\",\"group_key\":\"{}\"}},",
            "\"value\":{},",
            "\"threshold\":{},",
            "\"threshold_upper\":{},",
            "\"window_minutes\":{},",
            "\"links\":{{\"incident\":\"{}\",\"investigate\":\"{}\"}}",
            "}}"
        ),
        ctx.event_type.as_str(),
        escape_json(ctx.rule_id),
        escape_json(ctx.rule_name),
        escape_json(ctx.signal_type),
        escape_json(ctx.severity),
        escape_json(ctx.incident_id),
        escape_json(ctx.group_key),
        value,
        ctx.threshold,
        upper,
        ctx.window_minutes,
        escape_json(ctx.incident_url),
        escape_json(ctx.investigate_url),
    )
}

/// Slack incoming-webhook payload (`text` + attachment-style fields).
#[must_use]
pub(crate) fn slack_webhook_payload_json(ctx: &NotificationContext<'_>) -> String {
    let verb = match ctx.event_type {
        DeliveryEventType::Triggered => "FIRING",
        DeliveryEventType::Resolved => "RESOLVED",
        DeliveryEventType::Renotify => "STILL FIRING",
    };
    let value = ctx
        .observed_value
        .map(|v| format!("{v}"))
        .unwrap_or_else(|| "n/a".to_string());
    let text = format!(
        "[{verb}] {name} ({sev}) group={group} value={value} threshold={thr} — {url}",
        name = ctx.rule_name,
        sev = ctx.severity,
        group = ctx.group_key,
        thr = ctx.threshold,
        url = ctx.incident_url,
    );
    format!("{{\"text\":\"{}\"}}", escape_json(&text))
}

fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out
}

/// Backoff schedule: attempt 1 → 1m, 2 → 5m, 3 → 30m, then cap at 30m.
/// `attempt` is 1-based (first failure → attempt 1).
#[must_use]
pub(crate) fn backoff_after_failure(attempt: u32) -> Duration {
    match attempt.max(1) {
        1 => Duration::from_secs(60),
        2 => Duration::from_secs(5 * 60),
        _ => Duration::from_secs(30 * 60),
    }
}

/// Max delivery attempts before dead-letter (plan 167: 5).
pub(crate) const MAX_DELIVERY_ATTEMPTS: u32 = 5;

#[must_use]
pub(crate) fn is_dead_letter(attempt_count: u32) -> bool {
    attempt_count >= MAX_DELIVERY_ATTEMPTS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> NotificationContext<'static> {
        NotificationContext {
            rule_id: "rule-1",
            rule_name: "High error rate",
            signal_type: "error_rate",
            severity: "critical",
            group_key: "checkout",
            incident_id: "inc-9",
            event_type: DeliveryEventType::Triggered,
            observed_value: Some(0.42),
            threshold: 0.2,
            threshold_upper: None,
            window_minutes: 5,
            incident_url: "http://localhost:3000/alerts/incidents/inc-9",
            investigate_url: "http://localhost:3000/traces?service=checkout",
        }
    }

    #[test]
    fn unique_delivery_key_is_stable_and_event_scoped() {
        let a = unique_delivery_key("inc-1", "dest-a", DeliveryEventType::Triggered);
        let b = unique_delivery_key("inc-1", "dest-a", DeliveryEventType::Triggered);
        let c = unique_delivery_key("inc-1", "dest-a", DeliveryEventType::Resolved);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(a.contains("triggered"));
        assert!(c.contains("resolved"));
    }

    #[test]
    fn webhook_payload_contains_required_fields() {
        let body = webhook_payload_json(&ctx());
        assert!(body.contains("\"event\":\"triggered\""));
        assert!(body.contains("\"id\":\"rule-1\""));
        assert!(body.contains("High error rate"));
        assert!(body.contains("\"value\":0.42"));
        assert!(body.contains("\"threshold\":0.2"));
        assert!(body.contains("inc-9"));
        assert!(body.contains("investigate"));
        // Escaping: quotes in names must not break JSON structure for simple parse.
        let mut quoted = ctx();
        quoted.rule_name = r#"say "hi""#;
        let escaped = webhook_payload_json(&quoted);
        assert!(escaped.contains(r#"say \"hi\""#));
    }

    #[test]
    fn slack_payload_wraps_text() {
        let body = slack_webhook_payload_json(&ctx());
        assert!(body.starts_with("{\"text\":\""));
        assert!(body.contains("FIRING"));
        assert!(body.contains("High error rate"));
    }

    #[test]
    fn backoff_schedule_matches_plan() {
        assert_eq!(backoff_after_failure(1), Duration::from_secs(60));
        assert_eq!(backoff_after_failure(2), Duration::from_secs(300));
        assert_eq!(backoff_after_failure(3), Duration::from_secs(1800));
        assert_eq!(backoff_after_failure(10), Duration::from_secs(1800));
    }

    #[test]
    fn dead_letter_at_max_attempts() {
        assert!(!is_dead_letter(4));
        assert!(is_dead_letter(5));
        assert!(is_dead_letter(6));
    }

    #[test]
    fn event_type_round_trip() {
        for t in [
            DeliveryEventType::Triggered,
            DeliveryEventType::Resolved,
            DeliveryEventType::Renotify,
        ] {
            assert_eq!(DeliveryEventType::parse(t.as_str()), Some(t));
        }
        assert_eq!(DeliveryEventType::parse("nope"), None);
    }
}
