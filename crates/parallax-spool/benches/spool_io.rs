//! Plan-103 baseline: spool append + frame count on a temp dir.
//! Measurement only — no thresholds until variance is modeled.

#![expect(clippy::expect_used, reason = "bench fixture construction")]

use criterion::{Criterion, criterion_group, criterion_main};
use parallax_spool::{Signal, Spool};
use std::hint::black_box;

fn bench_spool(criterion: &mut Criterion) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    let dir = tempfile::tempdir().expect("tempdir");
    let spool = Spool::open(dir.path()).expect("open");
    let payload = bytes::Bytes::from(vec![7u8; 4096]);
    criterion.bench_function("spool_append_4k", |b| {
        b.iter(|| {
            runtime
                .block_on(spool.append_raw(Signal::Metrics, black_box(&payload)))
                .expect("append");
        });
    });
    // Fixed-size fixture: counting the append bench's ever-growing file made
    // this measurement grow monotonically across repeats (observed 24% CV on
    // CI). Count a dedicated 1k-frame segment instead.
    let count_dir = tempfile::tempdir().expect("count tempdir");
    let count_spool = Spool::open(count_dir.path()).expect("open count spool");
    for _ in 0..1_000 {
        runtime
            .block_on(count_spool.append_raw(Signal::Metrics, &payload))
            .expect("seed frame");
    }
    criterion.bench_function("spool_line_count_1k", |b| {
        b.iter(|| black_box(count_spool.line_count(Signal::Metrics).expect("count")));
    });
}

criterion_group!(benches, bench_spool);
criterion_main!(benches);
