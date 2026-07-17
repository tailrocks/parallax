//! Plan-103 allocation instrumentation (standalone, workspace-excluded so a
//! bench-only counting allocator can exist without weakening the product
//! crates' `forbid(unsafe_code)`).
//!
//! Prints allocations/bytes per `normalize_metrics` call over a
//! representative 1k-point batch — the zero-copy ingest promise's
//! measurement hook. Consumed by the scheduled-measurement workflow.

#![expect(
    unsafe_code,
    reason = "counting allocator delegates verbatim to the system allocator"
)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicU64, Ordering};

use parallax_proto::collector_metrics::ExportMetricsServiceRequest;
use parallax_proto::common::any_value::Value as AnyValueEnum;
use parallax_proto::common::{AnyValue, KeyValue};

struct CountingAllocator;

static ALLOCATIONS: AtomicU64 = AtomicU64::new(0);
static ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);

// SAFETY: delegates directly to the system allocator; counters are
// side-effect-only atomics.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        ALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

fn kv(key: &str, value: &str) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue {
            value: Some(AnyValueEnum::StringValue(value.to_string())),
        }),
        key_strindex: 0,
    }
}

fn request(points_per_metric: usize) -> ExportMetricsServiceRequest {
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

fn main() {
    let request = request(1_000);
    // Warm once so lazy statics do not pollute the measured pass.
    black_box(parallax_ingest::normalize_metrics(&request));
    const REPEATS: u64 = 10;
    let allocs_before = ALLOCATIONS.load(Ordering::Relaxed);
    let bytes_before = ALLOCATED_BYTES.load(Ordering::Relaxed);
    for _ in 0..REPEATS {
        black_box(parallax_ingest::normalize_metrics(&request));
    }
    let allocs = (ALLOCATIONS.load(Ordering::Relaxed) - allocs_before) / REPEATS;
    let bytes = (ALLOCATED_BYTES.load(Ordering::Relaxed) - bytes_before) / REPEATS;
    println!(
        "normalize_metrics_1k_points allocation-profile: {allocs} allocations/call, {bytes} bytes/call"
    );
}
