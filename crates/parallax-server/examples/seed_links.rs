//! Seed a linked-trace pair: trace A holds a source operation; trace B's
//! span carries an OTel span link back to A (the batch/async sub-operation
//! pattern). Demo data for the trace page's "Linked traces" section.

use parallax_proto::collector_trace::ExportTraceServiceRequest;
use parallax_proto::common::{AnyValue, KeyValue, any_value};
use parallax_proto::resource::Resource;
use parallax_proto::trace::span::Link;
use parallax_proto::trace::{ResourceSpans, ScopeSpans, Span, Status};
use prost::Message as _;

fn str_attr(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.into(),
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(value.into())),
        }),
        ..Default::default()
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos() as u64;
    let trace_a: Vec<u8> = (0..16).map(|i| 0xa0 + i as u8).collect();
    let span_a: Vec<u8> = (0..8).map(|i| 0xb0 + i as u8).collect();
    let trace_b: Vec<u8> = (0..16).map(|i| 0xc0 + i as u8).collect();
    let span_b: Vec<u8> = (0..8).map(|i| 0xd0 + i as u8).collect();

    let spans = vec![
        Span {
            trace_id: trace_a.clone(),
            span_id: span_a.clone(),
            name: "source operation".into(),
            kind: 2,
            start_time_unix_nano: now,
            end_time_unix_nano: now + 5_000_000,
            status: Some(Status {
                code: 1,
                ..Default::default()
            }),
            ..Default::default()
        },
        Span {
            trace_id: trace_b,
            span_id: span_b,
            name: "batch process (links source)".into(),
            kind: 1,
            start_time_unix_nano: now + 10_000_000,
            end_time_unix_nano: now + 30_000_000,
            status: Some(Status {
                code: 1,
                ..Default::default()
            }),
            links: vec![Link {
                trace_id: trace_a,
                span_id: span_a,
                attributes: vec![str_attr("link.kind", "batch-source")],
                ..Default::default()
            }],
            ..Default::default()
        },
    ];
    let request = ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(Resource {
                attributes: vec![str_attr("service.name", "batcher")],
                ..Default::default()
            }),
            scope_spans: vec![ScopeSpans {
                spans,
                ..Default::default()
            }],
            ..Default::default()
        }],
    };

    let response = reqwest::Client::new()
        .post("http://127.0.0.1:4318/v1/traces")
        .header("content-type", "application/x-protobuf")
        .body(request.encode_to_vec())
        .send()
        .await?;
    println!("seeded linked traces: {}", response.status());
    println!("source trace:  {}", hex(&[0xa0u8; 0][..]));
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
