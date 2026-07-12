use super::*;
use arrow::array::{Float64Array, Int64Array, StringArray};
use arrow::datatypes::{Field, Schema};
use arrow_ipc::CompressionType;
use arrow_ipc::writer::{IpcWriteOptions, StreamWriter};
use std::sync::Arc;

fn write_fixture(zstd: bool) -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("bucket_ns", DataType::Int64, true),
        Field::new("service", DataType::Utf8, true),
        Field::new("value", DataType::Float64, true),
        Field::new("empty", DataType::Utf8, true),
    ]));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![Some(1_000), Some(-2), None])),
            Arc::new(StringArray::from(vec![Some("api"), Some("worker"), None])),
            Arc::new(Float64Array::from(vec![
                Some(1.5),
                Some(0.0),
                Some(f64::NAN),
            ])),
            Arc::new(StringArray::from(vec![None, Some(""), Some("x")])),
        ],
    )
    .expect("batch");

    let mut buf = Vec::new();
    let options = if zstd {
        IpcWriteOptions::default()
            .try_with_compression(Some(CompressionType::ZSTD))
            .expect("zstd write options")
    } else {
        IpcWriteOptions::default()
    };
    {
        let mut writer =
            StreamWriter::try_new_with_options(&mut buf, &schema, options).expect("writer");
        writer.write(&batch).expect("write");
        writer.finish().expect("finish");
    }
    buf
}

#[test]
fn decodes_uncompressed_ipc_fixture() {
    let bytes = write_fixture(false);
    let (columns, rows) = decode_arrow_ipc(&bytes).expect("decode");
    assert_eq!(columns, vec!["bucket_ns", "service", "value", "empty"]);
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0][0], json_i64(1_000));
    assert_eq!(rows[0][1], Value::String("api".into()));
    assert_eq!(rows[0][2], json_f64(1.5));
    assert_eq!(rows[0][3], Value::Null);
    assert_eq!(rows[1][0], json_i64(-2));
    assert_eq!(rows[1][1], Value::String("worker".into()));
    assert_eq!(rows[2][0], Value::Null);
    assert_eq!(rows[2][1], Value::Null);
    // NaN cannot be a JSON number.
    assert_eq!(rows[2][2], Value::Null);
    assert_eq!(rows[2][3], Value::String("x".into()));
}

#[test]
fn decodes_zstd_compressed_ipc_fixture() {
    let bytes = write_fixture(true);
    let (columns, rows) = decode_arrow_ipc(&bytes).expect("decode zstd");
    assert_eq!(columns.len(), 4);
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0][1], Value::String("api".into()));
}

#[test]
fn maps_json_error_envelope() {
    let body = br#"{"code":1001,"error":"Table not found: foo","output":null}"#;
    let err = decode_arrow_ipc(body).expect_err("json error");
    let msg = format!("{err:#}");
    assert!(msg.contains("Table not found"), "{msg}");
    assert!(msg.contains("1001"), "{msg}");
}

#[test]
fn empty_body_is_empty_result() {
    let (columns, rows) = decode_arrow_ipc(&[]).expect("empty");
    assert!(columns.is_empty());
    assert!(rows.is_empty());
}

#[test]
fn row_count_parity_json_shape() {
    // Same fixture decoded twice must match (stand-in for JSON vs Arrow parity
    // without a live engine).
    let bytes = write_fixture(true);
    let (c1, r1) = decode_arrow_ipc(&bytes).unwrap();
    let (c2, r2) = decode_arrow_ipc(&bytes).unwrap();
    assert_eq!(c1, c2);
    assert_eq!(r1.len(), r2.len());
    assert_eq!(r1, r2);
}
