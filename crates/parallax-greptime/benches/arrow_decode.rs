//! Plan-103 baseline: Arrow IPC response decode (uncompressed + zstd).
//! Measurement only — no thresholds until variance is modeled.

use arrow::array::{Float64Array, Int64Array, RecordBatch, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow_ipc::CompressionType;
use arrow_ipc::writer::{IpcWriteOptions, StreamWriter};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::sync::Arc;

fn fixture(zstd: bool, rows: usize) -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("bucket_ns", DataType::Int64, true),
        Field::new("service", DataType::Utf8, true),
        Field::new("value", DataType::Float64, true),
    ]));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from(
                (0..rows).map(|i| Some(i as i64)).collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                (0..rows).map(|i| Some(format!("svc-{}", i % 8))).collect::<Vec<_>>(),
            )),
            Arc::new(Float64Array::from(
                (0..rows).map(|i| Some(i as f64)).collect::<Vec<_>>(),
            )),
        ],
    )
    .expect("batch");
    let options = if zstd {
        IpcWriteOptions::default()
            .try_with_compression(Some(CompressionType::ZSTD))
            .expect("zstd options")
    } else {
        IpcWriteOptions::default()
    };
    let mut buf = Vec::new();
    {
        let mut writer =
            StreamWriter::try_new_with_options(&mut buf, &schema, options).expect("writer");
        writer.write(&batch).expect("write");
        writer.finish().expect("finish");
    }
    buf
}

fn bench_decode(criterion: &mut Criterion) {
    let plain = fixture(false, 10_000);
    let compressed = fixture(true, 10_000);
    criterion.bench_function("arrow_decode_10k_rows", |b| {
        b.iter(|| black_box(parallax_greptime::arrow_sql::decode_arrow_ipc(black_box(&plain))))
    });
    criterion.bench_function("arrow_decode_10k_rows_zstd", |b| {
        b.iter(|| {
            black_box(parallax_greptime::arrow_sql::decode_arrow_ipc(black_box(
                &compressed,
            )))
        })
    });
}

criterion_group!(benches, bench_decode);
criterion_main!(benches);
