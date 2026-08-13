//! Delivery worker pass (plan 167 step 3, preliminary).
//!
//! `deliver_due_once` claims due outbox rows with a lease, builds the
//! webhook / Slack payload from the stored rule + incident + destination,
//! POSTs it (reqwest, repo native-TLS rule via workspace features), and marks
//! success, backed-off retry, or dead-letter. The tokio interval loop and the
//! `email` destination (deferred per plan STOP condition) are peer-owned.

use parallax_metadata::{AlertDeliveryEventRecord, TursoMetadataStore};

use super::{
    DeliveryEventType, NotificationContext, backoff_after_failure, is_dead_letter,
    slack_webhook_payload_json, webhook_payload_json,
};

/// Default lease for a claimed outbox row.
pub(crate) const DELIVERY_LEASE_SECS: u32 = 30;

/// Summary of one delivery pass.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct DeliveryReport {
    pub claimed: usize,
    pub delivered: usize,
    pub retried: usize,
    pub dead_lettered: usize,
}

fn destination_url(config: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(config)
        .ok()?
        .get("url")?
        .as_str()
        .map(str::to_string)
}

const NANOS_PER_SEC: u128 = 1_000_000_000;

/// Build the outbound body for one claimed event, or a reason it cannot be
/// built (missing rows / unsupported destination kind → permanent failure).
async fn build_payload(
    store: &TursoMetadataStore,
    event: &AlertDeliveryEventRecord,
    base_url: &str,
) -> anyhow::Result<Result<(String, String), String>> {
    let Some(event_type) = DeliveryEventType::parse(&event.event_type) else {
        return Ok(Err(format!("unknown event type: {}", event.event_type)));
    };
    let Some(incident) = store.alert_incident(&event.incident_id).await? else {
        return Ok(Err(format!("incident missing: {}", event.incident_id)));
    };
    let Some(rule) = store.alert_rule(&incident.rule_id).await? else {
        return Ok(Err(format!("rule missing: {}", incident.rule_id)));
    };
    let Some(destination) = store.alert_destination(&event.destination_id).await? else {
        return Ok(Err(format!(
            "destination missing: {}",
            event.destination_id
        )));
    };
    let Some(url) = destination_url(&destination.config) else {
        return Ok(Err("destination config has no url".to_string()));
    };
    let incident_url = format!("{base_url}/alerts/incidents/{}", incident.id);
    let investigate_url = if incident.group_key.is_empty() {
        format!("{base_url}/traces")
    } else {
        format!("{base_url}/traces?service={}", incident.group_key)
    };
    let bundle_url = format!("{base_url}/alerts?incident={}", incident.id);
    let adjacency: Vec<String> = incident
        .bundle_deploy_adjacency
        .as_deref()
        .and_then(|raw| serde_json::from_str(raw).ok())
        .unwrap_or_default();
    let ctx = NotificationContext {
        rule_id: &rule.id,
        rule_name: &rule.name,
        signal_type: &rule.signal_type,
        severity: &rule.severity,
        group_key: &incident.group_key,
        incident_id: &incident.id,
        event_type,
        observed_value: incident.last_value,
        threshold: rule.threshold,
        threshold_upper: rule.threshold_upper,
        window_minutes: rule.window_minutes,
        incident_url: &incident_url,
        investigate_url: &investigate_url,
        bundle_hash: incident.bundle_hash.as_deref(),
        bundle_url: incident.bundle_hash.as_ref().map(|_| bundle_url.as_str()),
        top_hypothesis: incident.bundle_top_hypothesis.as_deref(),
        deploy_adjacency: &adjacency,
        bundle_error: incident
            .bundle_error
            .as_deref()
            .or(if incident.bundle_hash.is_none() {
                Some("assembly unavailable")
            } else {
                None
            }),
    };
    let body = match destination.kind.as_str() {
        "webhook" => webhook_payload_json(&ctx),
        "slack_webhook" => slack_webhook_payload_json(&ctx),
        other => return Ok(Err(format!("unsupported destination kind: {other}"))),
    };
    Ok(Ok((url, body)))
}

async fn record_failure(
    store: &TursoMetadataStore,
    event: &AlertDeliveryEventRecord,
    error: &str,
    now_nanos: u128,
    permanent: bool,
    report: &mut DeliveryReport,
) -> anyhow::Result<()> {
    let attempt = event.attempt_count + 1;
    let dead = permanent || is_dead_letter(attempt);
    let next_attempt =
        now_nanos + u128::from(backoff_after_failure(attempt).as_secs()) * NANOS_PER_SEC;
    store
        .alert_delivery_mark_failed(&event.id, error, next_attempt, dead)
        .await?;
    if dead {
        report.dead_lettered += 1;
    } else {
        report.retried += 1;
    }
    Ok(())
}

/// Claim and attempt up to `limit` due deliveries. Safe to call repeatedly;
/// lease-claimed rows are invisible to other workers until expiry.
/// `base_url` is the operator-facing UI origin used in payload links.
pub(crate) async fn deliver_due_once(
    store: &TursoMetadataStore,
    client: &reqwest::Client,
    claimer: &str,
    base_url: &str,
    now_nanos: u128,
    limit: usize,
) -> anyhow::Result<DeliveryReport> {
    let mut report = DeliveryReport::default();
    let events = store
        .alert_deliveries_claim(claimer, now_nanos, DELIVERY_LEASE_SECS, limit)
        .await?;
    report.claimed = events.len();
    for event in events {
        let (url, body) = match build_payload(store, &event, base_url).await? {
            Ok(built) => built,
            Err(reason) => {
                record_failure(store, &event, &reason, now_nanos, true, &mut report).await?;
                continue;
            }
        };
        let response = client
            .post(&url)
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await;
        match response {
            Ok(response) if response.status().is_success() => {
                store
                    .alert_delivery_mark_delivered(&event.id, now_nanos)
                    .await?;
                report.delivered += 1;
            }
            Ok(response) => {
                let error = format!("HTTP {}", response.status().as_u16());
                record_failure(store, &event, &error, now_nanos, false, &mut report).await?;
            }
            Err(error) => {
                record_failure(
                    store,
                    &event,
                    &error.to_string(),
                    now_nanos,
                    false,
                    &mut report,
                )
                .await?;
            }
        }
    }
    Ok(report)
}

#[cfg(test)]
#[expect(
    clippy::excessive_nesting,
    reason = "inline HTTP fixture server reads clearest as one loop"
)]
mod tests {
    use super::*;
    use parallax_metadata::{AlertDestinationRecord, AlertIncidentRecord, AlertRuleRecord};
    use std::io::{Read, Write};

    const MIN: u128 = 60 * 1_000_000_000;

    /// Minimal one-shot HTTP server: accepts `hits` connections, replies with
    /// `status`, records each request body.
    fn test_server(status: u16, hits: usize) -> (String, std::sync::mpsc::Receiver<String>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for _ in 0..hits {
                let Ok((mut stream, _)) = listener.accept() else {
                    return;
                };
                let mut buf = vec![0_u8; 65536];
                let mut read_total = 0;
                let body = loop {
                    let n = stream.read(&mut buf[read_total..]).unwrap_or(0);
                    if n == 0 {
                        break String::new();
                    }
                    read_total += n;
                    let text = String::from_utf8_lossy(&buf[..read_total]).to_string();
                    if let Some(split) = text.find("\r\n\r\n") {
                        let header = &text[..split];
                        let length = header
                            .lines()
                            .find_map(|l| {
                                l.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .map(|v| v.trim().parse::<usize>().unwrap_or(0))
                            })
                            .unwrap_or(0);
                        let have = read_total - split - 4;
                        if have >= length {
                            break text[split + 4..].to_string();
                        }
                    }
                };
                drop(tx.send(body));
                let reply = format!(
                    "HTTP/1.1 {status} X\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"
                );
                drop(stream.write_all(reply.as_bytes()));
            }
        });
        (format!("http://{addr}/hook"), rx)
    }

    fn temp_store() -> (tempfile::TempDir, std::path::PathBuf) {
        let directory = tempfile::tempdir().expect("temporary directory");
        let path = directory.path().join("metadata.db");
        (directory, path)
    }

    async fn seed(store: &TursoMetadataStore, url: &str, kind: &str) {
        store
            .alert_rule_save(&AlertRuleRecord {
                id: "r1".to_string(),
                name: "High error rate".to_string(),
                enabled: true,
                signal_type: "error_rate".to_string(),
                services: "[]".to_string(),
                exclude_services: "[]".to_string(),
                attribute_filters: "[]".to_string(),
                group_by: None,
                comparator: "gt".to_string(),
                threshold: 0.2,
                threshold_upper: None,
                window_minutes: 5,
                minimum_sample_count: 1,
                consecutive_breaches_required: 2,
                consecutive_healthy_required: 2,
                no_data_behavior: "skip".to_string(),
                severity: "critical".to_string(),
                renotify_interval_minutes: 30,
                destination_ids: "[\"d1\"]".to_string(),
                metric_name: None,
                metric_aggregation: None,
                created_at_nanos: MIN,
                updated_at_nanos: MIN,
            })
            .await
            .expect("rule");
        store
            .alert_destination_save(&AlertDestinationRecord {
                id: "d1".to_string(),
                name: "hook".to_string(),
                kind: kind.to_string(),
                config: format!("{{\"url\":\"{url}\"}}"),
                created_at_nanos: MIN,
                updated_at_nanos: MIN,
            })
            .await
            .expect("destination");
        store
            .alert_incident_open(&AlertIncidentRecord {
                id: "i1".to_string(),
                rule_id: "r1".to_string(),
                group_key: "checkout".to_string(),
                status: "open".to_string(),
                severity: "critical".to_string(),
                first_triggered_at_nanos: MIN,
                last_triggered_at_nanos: MIN,
                resolved_at_nanos: None,
                last_value: Some(0.4),
                last_notified_at_nanos: Some(MIN),
                bundle_hash: None,
                bundle_assembled_at_nanos: None,
                bundle_top_hypothesis: None,
                bundle_deploy_adjacency: None,
                bundle_error: None,
            })
            .await
            .expect("incident");
        store
            .alert_delivery_enqueue(&AlertDeliveryEventRecord {
                id: "e1".to_string(),
                incident_id: "i1".to_string(),
                destination_id: "d1".to_string(),
                event_type: "triggered".to_string(),
                status: "pending".to_string(),
                attempt_count: 0,
                next_attempt_at_nanos: MIN,
                claimed_by: None,
                claim_expires_at_nanos: None,
                delivered_at_nanos: None,
                last_error: None,
                delivery_key: "i1|d1|triggered".to_string(),
                created_at_nanos: MIN,
            })
            .await
            .expect("enqueue");
    }

    #[tokio::test]
    async fn success_delivers_webhook_payload() {
        let (_dir, path) = temp_store();
        let store = TursoMetadataStore::open(path).await.expect("open");
        let (url, rx) = test_server(200, 1);
        seed(&store, &url, "webhook").await;
        let client = reqwest::Client::new();
        let report = deliver_due_once(&store, &client, "w1", "http://localhost:3000", 2 * MIN, 10)
            .await
            .expect("pass");
        assert_eq!(report.claimed, 1);
        assert_eq!(report.delivered, 1);
        let body = rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("body");
        assert!(body.contains("\"event\":\"triggered\""));
        assert!(body.contains("High error rate"));
        assert!(body.contains("/alerts/incidents/i1"));
        let events = store
            .alert_deliveries_for_incident("i1")
            .await
            .expect("list");
        assert_eq!(events[0].status, "delivered");
        // Nothing left to claim.
        let again = deliver_due_once(&store, &client, "w1", "http://localhost:3000", 3 * MIN, 10)
            .await
            .expect("pass");
        assert_eq!(again.claimed, 0);
    }

    #[tokio::test]
    async fn http_500_backs_off_then_next_pass_is_not_due_yet() {
        let (_dir, path) = temp_store();
        let store = TursoMetadataStore::open(path).await.expect("open");
        let (url, _rx) = test_server(500, 1);
        seed(&store, &url, "webhook").await;
        let client = reqwest::Client::new();
        let now = 2 * MIN;
        let report = deliver_due_once(&store, &client, "w1", "http://localhost:3000", now, 10)
            .await
            .expect("pass");
        assert_eq!(report.retried, 1);
        let events = store
            .alert_deliveries_for_incident("i1")
            .await
            .expect("list");
        assert_eq!(events[0].status, "pending");
        assert_eq!(events[0].attempt_count, 1);
        assert_eq!(events[0].last_error.as_deref(), Some("HTTP 500"));
        // Backoff after attempt 1 is 60s: not due 30s later, due 61s later.
        let soon = deliver_due_once(
            &store,
            &client,
            "w1",
            "http://localhost:3000",
            now + MIN / 2,
            10,
        )
        .await
        .expect("pass");
        assert_eq!(soon.claimed, 0);
    }

    #[tokio::test]
    async fn broken_destination_config_dead_letters_permanently() {
        let (_dir, path) = temp_store();
        let store = TursoMetadataStore::open(path).await.expect("open");
        seed(&store, "http://127.0.0.1:1/hook", "webhook").await;
        store
            .alert_destination_save(&AlertDestinationRecord {
                id: "d1".to_string(),
                name: "hook".to_string(),
                kind: "webhook".to_string(),
                config: "{\"nope\":true}".to_string(),
                created_at_nanos: MIN,
                updated_at_nanos: MIN,
            })
            .await
            .expect("destination");
        let client = reqwest::Client::new();
        let report = deliver_due_once(&store, &client, "w1", "http://localhost:3000", 2 * MIN, 10)
            .await
            .expect("pass");
        assert_eq!(report.dead_lettered, 1);
        let events = store
            .alert_deliveries_for_incident("i1")
            .await
            .expect("list");
        assert_eq!(events[0].status, "dead");
        assert!(
            events[0]
                .last_error
                .as_deref()
                .unwrap_or("")
                .contains("no url")
        );
    }

    #[tokio::test]
    async fn slack_destination_sends_text_payload() {
        let (_dir, path) = temp_store();
        let store = TursoMetadataStore::open(path).await.expect("open");
        let (url, rx) = test_server(200, 1);
        seed(&store, &url, "slack_webhook").await;
        let client = reqwest::Client::new();
        let report = deliver_due_once(&store, &client, "w1", "http://localhost:3000", 2 * MIN, 10)
            .await
            .expect("pass");
        assert_eq!(report.delivered, 1);
        let body = rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("body");
        assert!(body.starts_with("{\"text\":\""));
        assert!(body.contains("FIRING"));
    }
}
