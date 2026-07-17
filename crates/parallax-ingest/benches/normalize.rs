//! Plan-103 baseline: OTLP metrics normalization over a representative
//! batch. Measurement only — no thresholds until variance is modeled.

use criterion::{Criterion, criterion_group, criterion_main};
use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::common::any_value::Value as AnyValueEnum;
use parallax_proto::common::{AnyValue, KeyValue};
use std::hint::black_box;

fn request(points_per_metric: usize) -> ExportMetricsServiceRequest {
    let kv = |key: &str, value: &str| KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(AnyValueEnum::StringValue(value.to_string())),
        }),
        key_strindex: 0,
    };
    let points = (0..points_per_metric)
        .map(|index| parallax_proto::metrics::NumberDataPoint {
            attributes: vec![kv("region", if index % 3 == 0 { "eu" } else { "us" })],
            start_time_unix_nano: 1,
            time_unix_nano: 1 + index as u64,
            exemplars: Vec::new(),
            flags: 0,
            value: Some(parallax_proto::metrics::number_data_point::Value::AsDouble(
                index as f64,
            )),
        })
        .collect::<Vec<_>>();
    ExportMetricsServiceRequest {
        resource_metrics: vec![parallax_proto::metrics::ResourceMetrics {
            resource: Some(parallax_proto::resource::Resource {
                attributes: vec![
                    kv("service.name", "bench"),
                    kv("cli.invocation.id", "bench-invocation"),
                ],
                dropped_attributes_count: 0,
                entity_refs: Vec::new(),
            }),
            scope_metrics: vec![parallax_proto::metrics::ScopeMetrics {
                scope: None,
                metrics: vec![parallax_proto::metrics::Metric {
                    name: "bench.gauge".to_string(),
                    description: String::new(),
                    unit: "1".to_string(),
                    metadata: Vec::new(),
                    data: Some(parallax_proto::metrics::metric::Data::Gauge(
                        parallax_proto::metrics::Gauge {
                            data_points: points,
                        },
                    )),
                }],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    }
}

fn bench_normalize(criterion: &mut Criterion) {
    let request = request(1_000);
    criterion.bench_function("normalize_metrics_1k_points", |b| {
        b.iter(|| black_box(parallax_ingest::normalize_metrics(black_box(&request))))
    });
}

criterion_group!(benches, bench_normalize);
criterion_main!(benches);
