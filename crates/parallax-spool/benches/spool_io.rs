//! Plan-103 baseline: spool append + frame count on a temp dir.
//! Measurement only — no thresholds until variance is modeled.

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
                .expect("append")
        })
    });
    criterion.bench_function("spool_line_count", |b| {
        b.iter(|| black_box(spool.line_count(Signal::Metrics).expect("count")))
    });
}

criterion_group!(benches, bench_spool);
criterion_main!(benches);
