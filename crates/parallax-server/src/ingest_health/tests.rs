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
        None,
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
