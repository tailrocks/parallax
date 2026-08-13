use super::*;
use arrow::array::{
    ArrayRef, BinaryArray, BooleanArray, Decimal128Array, DictionaryArray, Float32Array,
    Float64Array, Int8Array, Int16Array, Int32Array, Int64Array, LargeBinaryArray,
    LargeStringArray, NullArray, StringArray, TimestampMillisecondArray, TimestampNanosecondArray,
    UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use arrow::datatypes::{Field, Int32Type, Schema};
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

fn decode_one_col(name: &str, array: ArrayRef) -> Value {
    let schema = Arc::new(Schema::new(vec![Field::new(
        name,
        array.data_type().clone(),
        true,
    )]));
    let batch = RecordBatch::try_new(schema.clone(), vec![array]).expect("batch");
    let mut buf = Vec::new();
    {
        let mut writer = StreamWriter::try_new(&mut buf, &schema).expect("writer");
        writer.write(&batch).expect("write");
        writer.finish().expect("finish");
    }
    let (columns, rows) = decode_arrow_ipc(&buf).expect("decode");
    assert_eq!(columns, vec![name]);
    assert_eq!(rows.len(), 1);
    rows[0][0].clone()
}

#[test]
fn array_value_to_json_covers_supported_data_types() {
    assert_eq!(
        decode_one_col(
            "ts_ns",
            Arc::new(TimestampNanosecondArray::from(vec![Some(
                1_700_000_000_000_000_000
            )]))
        ),
        json_i64(1_700_000_000_000_000_000)
    );
    assert_eq!(
        decode_one_col(
            "ts_ms",
            Arc::new(TimestampMillisecondArray::from(vec![Some(
                1_700_000_000_000
            )]))
        ),
        json_i64(1_700_000_000_000)
    );

    let dict_values = StringArray::from(vec!["alpha", "beta"]);
    let dict_keys = Int32Array::from(vec![1]);
    let dict = DictionaryArray::<Int32Type>::try_new(dict_keys, Arc::new(dict_values))
        .expect("dictionary");
    assert_eq!(
        decode_one_col("dict", Arc::new(dict)),
        Value::String("beta".into())
    );

    let decimal = Decimal128Array::from(vec![12_345])
        .with_precision_and_scale(10, 2)
        .expect("decimal");
    assert_eq!(decode_one_col("dec", Arc::new(decimal)), json_f64(123.45));

    assert_eq!(
        decode_one_col(
            "bin",
            Arc::new(BinaryArray::from(vec![Some(b"hi".as_ref())]))
        ),
        Value::String("hi".into())
    );
    assert_eq!(
        decode_one_col(
            "bin_hex",
            Arc::new(BinaryArray::from(vec![Some([0xff, 0x00].as_ref())]))
        ),
        Value::String("ff00".into())
    );
    assert_eq!(
        decode_one_col(
            "lbin",
            Arc::new(LargeBinaryArray::from(vec![Some(b"lg".as_ref())]))
        ),
        Value::String("lg".into())
    );
    assert_eq!(
        decode_one_col(
            "lutf8",
            Arc::new(LargeStringArray::from(vec![Some("wide")]))
        ),
        Value::String("wide".into())
    );

    assert_eq!(
        decode_one_col("u8", Arc::new(UInt8Array::from(vec![7u8]))),
        json_u64(7)
    );
    assert_eq!(
        decode_one_col("u16", Arc::new(UInt16Array::from(vec![700u16]))),
        json_u64(700)
    );
    assert_eq!(
        decode_one_col("u32", Arc::new(UInt32Array::from(vec![70_000u32]))),
        json_u64(70_000)
    );
    assert_eq!(
        decode_one_col("u64", Arc::new(UInt64Array::from(vec![7_000_000_000u64]))),
        json_u64(7_000_000_000)
    );
    assert_eq!(
        decode_one_col("i8", Arc::new(Int8Array::from(vec![-8]))),
        json_i64(-8)
    );
    assert_eq!(
        decode_one_col("i16", Arc::new(Int16Array::from(vec![-16]))),
        json_i64(-16)
    );
    assert_eq!(
        decode_one_col("i32", Arc::new(Int32Array::from(vec![-32]))),
        json_i64(-32)
    );
    assert_eq!(
        decode_one_col("f32", Arc::new(Float32Array::from(vec![1.25f32]))),
        json_f64(1.25)
    );
    assert_eq!(
        decode_one_col("bool", Arc::new(BooleanArray::from(vec![true]))),
        Value::Bool(true)
    );
    assert_eq!(
        decode_one_col("nulls", Arc::new(NullArray::new(1))),
        Value::Null
    );
}

#[test]
fn validate_ipc_frame_lengths_rejects_truncated_declared_metadata() {
    let mut bytes = 65_535u32.to_le_bytes().to_vec();
    bytes.extend_from_slice(&[1, 2, 3]);
    let err = validate_ipc_frame_lengths(&bytes).expect_err("truncated");
    let msg = format!("{err:#}");
    assert!(msg.contains("declares 65535 metadata bytes"), "{msg}");
}

#[test]
fn append_batch_rows_concatenates_two_record_batches() {
    let schema = Arc::new(Schema::new(vec![Field::new("n", DataType::Int64, true)]));
    let first = RecordBatch::try_new(schema.clone(), vec![Arc::new(Int64Array::from(vec![1i64]))])
        .expect("first");
    let second = RecordBatch::try_new(schema.clone(), vec![Arc::new(Int64Array::from(vec![2i64]))])
        .expect("second");
    let mut buf = Vec::new();
    {
        let mut writer = StreamWriter::try_new(&mut buf, &schema).expect("writer");
        writer.write(&first).expect("write first");
        writer.write(&second).expect("write second");
        writer.finish().expect("finish");
    }
    let (columns, rows) = decode_arrow_ipc(&buf).expect("decode");
    assert_eq!(columns, vec!["n"]);
    assert_eq!(rows, vec![vec![json_i64(1)], vec![json_i64(2)]]);
}
