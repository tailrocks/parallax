use super::*;
use parallax_proto::common::{AnyValue, KeyValue as ProtoKeyValue};
use parallax_proto::metrics::ResourceMetrics;
use parallax_proto::resource::Resource;

fn metric_request(service: &str) -> ExportMetricsServiceRequest {
    ExportMetricsServiceRequest {
        resource_metrics: vec![ResourceMetrics {
            resource: Some(Resource {
                attributes: vec![ProtoKeyValue {
                    key: "service.name".into(),
                    key_strindex: 0,
                    value: Some(AnyValue {
                        value: Some(Value::StringValue(service.into())),
                    }),
                }],
                dropped_attributes_count: 0,
                entity_refs: Vec::new(),
            }),
            scope_metrics: Vec::new(),
            schema_url: String::new(),
        }],
    }
}

#[test]
fn queue_state_and_self_metric_filter_are_exact() -> Result<(), String> {
    let health = IngestHealth::new(2);
    health.enqueued(Signal::Traces, Duration::from_millis(2), true);
    health.enqueued(Signal::Traces, Duration::ZERO, true);
    let full = health.snapshot(Signal::Traces);
    let degraded = health.degradation();
    health.dequeued(Signal::Traces, Duration::from_millis(4), true);
    health.dequeued(Signal::Traces, Duration::from_millis(5), true);
    health.retry(Signal::Traces);
    health.terminal_drop(Signal::Traces);
    let snapshot = health.snapshot(Signal::Traces);
    let actual = (
        snapshot,
        health.degradation(),
        full,
        degraded,
        is_self_metrics(&metric_request("parallax")),
        is_self_metrics(&metric_request("checkout")),
        instrument_contract_is_bounded(),
    );
    let expected = (
        QueueSnapshot {
            depth: 0,
            capacity: 2,
            high_water: 2,
            retries: 1,
            drops: 1,
        },
        Some("ingest terminal drop (1)".to_string()),
        QueueSnapshot {
            depth: 2,
            capacity: 2,
            high_water: 2,
            retries: 0,
            drops: 0,
        },
        Some("ingest queue full (traces=2/2)".to_string()),
        true,
        false,
        true,
    );
    if actual != expected {
        return Err(format!("ingest health mismatch: {actual:?}"));
    }
    Ok(())
}

fn instrument_contract_is_bounded() -> bool {
    let source = include_str!("../ingest_health.rs");
    [
        "parallax.ingest.enqueue.outcomes",
        "parallax.ingest.enqueue.wait",
        "parallax.ingest.queue.age",
        "parallax.ingest.worker.retries",
        "parallax.ingest.worker.drops",
        "parallax.ingest.loss.ingress_reject",
        "parallax.ingest.loss.spool_write",
        "parallax.ingest.loss.unsupported_metric",
        "parallax.ingest.loss.live_tail_lag",
        "parallax.ingest.worker.drain",
        "parallax.ingest.queue.depth",
        "parallax.ingest.queue.capacity",
        "parallax.ingest.queue.high_water",
        "parallax.ingest.queue.oldest_age",
        "parallax.ingest.spool.bytes",
        "parallax.ingest.spool.oldest_age",
        "parallax.ingest.spool.reclaimed",
    ]
    .iter()
    .all(|name| source.contains(name))
        && !["tenant", "trace_id", "service.name", "error.message"]
            .iter()
            .any(|label| source.contains(&format!("KeyValue::new(\"{label}\"")))
}

#[test]
fn ingress_reject_increments_loss_json() {
    let health = IngestHealth::new(4);
    health.ingress_reject(Signal::Logs);
    assert!(health.loss_json().contains("\"ingress_reject\":1"));
    assert!(health.degradation().is_none());
}

#[test]
fn spool_write_fail_degrades_health() {
    let health = IngestHealth::new(4);
    health.spool_failed(Signal::Traces);
    assert!(health.loss_json().contains("\"spool_write\":1"));
    assert_eq!(
        health.degradation().as_deref(),
        Some("spool write failed (1)")
    );
}

#[test]
fn unsupported_metric_is_visible_and_does_not_degrade() {
    let health = IngestHealth::new(4);
    health.unsupported_metric(2);
    assert!(health.loss_json().contains("\"unsupported_metric\":2"));
    assert!(health.degradation().is_none());
}

#[test]
fn live_tail_lag_is_counted_and_does_not_degrade() {
    let health = IngestHealth::new(4);
    health.live_lagged(3);
    assert!(health.loss_json().contains("\"live_tail_lag\":3"));
    assert!(health.degradation().is_none());
}

#[test]
fn queue_unavailable_is_counted() {
    let health = IngestHealth::new(4);
    health.unavailable(Signal::Metrics, Duration::ZERO);
    assert!(health.loss_json().contains("\"queue_unavailable\":1"));
}
