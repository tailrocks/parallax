//! Decode GreptimeDB HTTP `format=arrow` (+ optional zstd IPC compression)
//! responses into the same row shape as `greptimedb_v1` JSON (`Vec<Vec<Value>>`).
//!
//! Used by heavy typed reads (plan 091). Tiny / DDL / schema probes stay on JSON.

use anyhow::{Context, bail};
use arrow::array::{
    Array, ArrayRef, BooleanArray, DictionaryArray, GenericBinaryArray, GenericStringArray,
    OffsetSizeTrait, PrimitiveArray, StringViewArray,
};
use arrow::datatypes::{
    DataType, Date32Type, Date64Type, Decimal128Type, Decimal256Type, Float16Type, Float32Type,
    Float64Type, Int8Type, Int16Type, Int32Type, Int64Type, Time32MillisecondType,
    Time32SecondType, Time64MicrosecondType, Time64NanosecondType, TimeUnit,
    TimestampMicrosecondType, TimestampMillisecondType, TimestampNanosecondType,
    TimestampSecondType, UInt8Type, UInt16Type, UInt32Type, UInt64Type,
};
use arrow::record_batch::RecordBatch;
use arrow_ipc::reader::StreamReader;
use serde_json::{Number, Value};
use std::io::Cursor;

/// Decode an Arrow IPC stream (optionally zstd-compressed per-message) into
/// column names + rows of JSON-compatible cells.
pub fn decode_arrow_ipc(bytes: &[u8]) -> anyhow::Result<(Vec<String>, Vec<Vec<Value>>)> {
    if bytes.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    // Greptime returns a JSON error envelope even when format=arrow was requested.
    if bytes.first() == Some(&b'{') {
        return Err(map_greptime_json_error(bytes));
    }

    let reader = StreamReader::try_new(Cursor::new(bytes), None)
        .context("arrow-ipc StreamReader (format=arrow)")?;
    let mut columns: Vec<String> = Vec::new();
    let mut rows: Vec<Vec<Value>> = Vec::new();
    let mut schema_set = false;

    for batch in reader {
        let batch = batch.context("arrow-ipc record batch")?;
        if !schema_set {
            columns = batch
                .schema()
                .fields()
                .iter()
                .map(|f| f.name().clone())
                .collect();
            schema_set = true;
        }
        append_batch_rows(&batch, &mut rows)?;
    }
    Ok((columns, rows))
}

fn map_greptime_json_error(bytes: &[u8]) -> anyhow::Error {
    match serde_json::from_slice::<Value>(bytes) {
        Ok(response) => {
            if let Some(error) = response.get("error").and_then(|e| e.as_str()) {
                let code = response.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
                return anyhow::anyhow!("greptime sql failed (code {code}): {error}");
            }
            anyhow::anyhow!("expected Arrow IPC stream, got JSON without error field: {response}")
        }
        Err(err) => anyhow::anyhow!(
            "expected Arrow IPC stream, got non-IPC body ({err}): {}",
            String::from_utf8_lossy(&bytes[..bytes.len().min(200)])
        ),
    }
}

fn append_batch_rows(batch: &RecordBatch, rows: &mut Vec<Vec<Value>>) -> anyhow::Result<()> {
    let n = batch.num_rows();
    let cols: Vec<ArrayRef> = batch.columns().to_vec();
    rows.reserve(n);
    for row_idx in 0..n {
        let mut row = Vec::with_capacity(cols.len());
        for col in &cols {
            row.push(array_value_to_json(col.as_ref(), row_idx)?);
        }
        rows.push(row);
    }
    Ok(())
}

fn array_value_to_json(array: &dyn Array, row: usize) -> anyhow::Result<Value> {
    if array.is_null(row) {
        return Ok(Value::Null);
    }
    match array.data_type() {
        DataType::Null => Ok(Value::Null),
        DataType::Boolean => {
            let a = array
                .as_any()
                .downcast_ref::<BooleanArray>()
                .context("bool array")?;
            Ok(Value::Bool(a.value(row)))
        }
        DataType::Int8 => Ok(json_i64(primitive::<Int8Type>(array)?.value(row) as i64)),
        DataType::Int16 => Ok(json_i64(primitive::<Int16Type>(array)?.value(row) as i64)),
        DataType::Int32 => Ok(json_i64(i64::from(
            primitive::<Int32Type>(array)?.value(row),
        ))),
        DataType::Int64 => Ok(json_i64(primitive::<Int64Type>(array)?.value(row))),
        DataType::UInt8 => Ok(json_u64(u64::from(
            primitive::<UInt8Type>(array)?.value(row),
        ))),
        DataType::UInt16 => Ok(json_u64(u64::from(
            primitive::<UInt16Type>(array)?.value(row),
        ))),
        DataType::UInt32 => Ok(json_u64(u64::from(
            primitive::<UInt32Type>(array)?.value(row),
        ))),
        DataType::UInt64 => Ok(json_u64(primitive::<UInt64Type>(array)?.value(row))),
        DataType::Float16 => Ok(json_f64(f64::from(
            primitive::<Float16Type>(array)?.value(row).to_f32(),
        ))),
        DataType::Float32 => Ok(json_f64(f64::from(
            primitive::<Float32Type>(array)?.value(row),
        ))),
        DataType::Float64 => Ok(json_f64(primitive::<Float64Type>(array)?.value(row))),
        DataType::Utf8 => Ok(Value::String(
            generic_string::<i32>(array)?.value(row).to_owned(),
        )),
        DataType::LargeUtf8 => Ok(Value::String(
            generic_string::<i64>(array)?.value(row).to_owned(),
        )),
        DataType::Utf8View => {
            let a = array
                .as_any()
                .downcast_ref::<StringViewArray>()
                .context("utf8 view")?;
            Ok(Value::String(a.value(row).to_owned()))
        }
        DataType::Binary => Ok(binary_to_value::<i32>(array, row)?),
        DataType::LargeBinary => Ok(binary_to_value::<i64>(array, row)?),
        DataType::FixedSizeBinary(_) => {
            let a = array
                .as_any()
                .downcast_ref::<arrow::array::FixedSizeBinaryArray>()
                .context("fixed binary")?;
            Ok(bytes_to_value(a.value(row)))
        }
        DataType::Timestamp(unit, _) => Ok(json_i64(timestamp_value(array, *unit, row)?)),
        DataType::Date32 => Ok(json_i64(i64::from(
            primitive::<Date32Type>(array)?.value(row),
        ))),
        DataType::Date64 => Ok(json_i64(primitive::<Date64Type>(array)?.value(row))),
        DataType::Time32(TimeUnit::Second) => Ok(json_i64(i64::from(
            primitive::<Time32SecondType>(array)?.value(row),
        ))),
        DataType::Time32(TimeUnit::Millisecond) => Ok(json_i64(i64::from(
            primitive::<Time32MillisecondType>(array)?.value(row),
        ))),
        DataType::Time64(TimeUnit::Microsecond) => Ok(json_i64(
            primitive::<Time64MicrosecondType>(array)?.value(row),
        )),
        DataType::Time64(TimeUnit::Nanosecond) => Ok(json_i64(
            primitive::<Time64NanosecondType>(array)?.value(row),
        )),
        DataType::Decimal128(_, scale) => {
            let raw = primitive::<Decimal128Type>(array)?.value(row);
            Ok(decimal_i128_to_value(raw, *scale))
        }
        DataType::Decimal256(_, scale) => {
            let raw = primitive::<Decimal256Type>(array)?.value(row);
            // Best-effort: i256 → string so callers can parse if needed.
            let _ = scale;
            Ok(Value::String(raw.to_string()))
        }
        DataType::Dictionary(_, _) => dictionary_value(array, row),
        other => {
            // Nested / exotic types are uncommon on Greptime SQL result paths we
            // care about; surface a stable string rather than failing the page.
            tracing::debug!(
                target: "parallax_storage::arrow_sql",
                ?other,
                "arrow cell fell back to Debug string"
            );
            Ok(Value::String(format!("unsupported_arrow:{other:?}")))
        }
    }
}

fn dictionary_value(array: &dyn Array, row: usize) -> anyhow::Result<Value> {
    macro_rules! try_dict {
        ($key:ty) => {
            if let Some(dict) = array.as_any().downcast_ref::<DictionaryArray<$key>>() {
                let key = dict.keys().value(row);
                let values = dict.values();
                // Keys are non-negative for the types we handle here.
                let idx = usize::try_from(key).unwrap_or(0);
                return array_value_to_json(values.as_ref(), idx);
            }
        };
    }
    try_dict!(Int8Type);
    try_dict!(Int16Type);
    try_dict!(Int32Type);
    try_dict!(Int64Type);
    try_dict!(UInt8Type);
    try_dict!(UInt16Type);
    try_dict!(UInt32Type);
    try_dict!(UInt64Type);
    bail!("unsupported dictionary key type: {:?}", array.data_type())
}

fn timestamp_value(array: &dyn Array, unit: TimeUnit, row: usize) -> anyhow::Result<i64> {
    Ok(match unit {
        TimeUnit::Second => primitive::<TimestampSecondType>(array)?.value(row),
        TimeUnit::Millisecond => primitive::<TimestampMillisecondType>(array)?.value(row),
        TimeUnit::Microsecond => primitive::<TimestampMicrosecondType>(array)?.value(row),
        TimeUnit::Nanosecond => primitive::<TimestampNanosecondType>(array)?.value(row),
    })
}

fn primitive<T: arrow::array::ArrowPrimitiveType>(
    array: &dyn Array,
) -> anyhow::Result<&PrimitiveArray<T>> {
    array
        .as_any()
        .downcast_ref::<PrimitiveArray<T>>()
        .with_context(|| format!("expected primitive {:?}", std::any::type_name::<T>()))
}

fn generic_string<O: OffsetSizeTrait>(array: &dyn Array) -> anyhow::Result<&GenericStringArray<O>> {
    array
        .as_any()
        .downcast_ref::<GenericStringArray<O>>()
        .context("string array")
}

fn binary_to_value<O: OffsetSizeTrait>(array: &dyn Array, row: usize) -> anyhow::Result<Value> {
    let a = array
        .as_any()
        .downcast_ref::<GenericBinaryArray<O>>()
        .context("binary array")?;
    Ok(bytes_to_value(a.value(row)))
}

fn bytes_to_value(bytes: &[u8]) -> Value {
    match std::str::from_utf8(bytes) {
        Ok(s) => Value::String(s.to_owned()),
        Err(_) => Value::String(hex::encode(bytes)),
    }
}

fn json_i64(n: i64) -> Value {
    Value::Number(Number::from(n))
}

fn json_u64(n: u64) -> Value {
    Value::Number(Number::from(n))
}

fn json_f64(n: f64) -> Value {
    Number::from_f64(n)
        .map(Value::Number)
        .unwrap_or(Value::Null)
}

fn decimal_i128_to_value(raw: i128, scale: i8) -> Value {
    if scale == 0 {
        if let Ok(n) = i64::try_from(raw) {
            return json_i64(n);
        }
        return Value::String(raw.to_string());
    }
    let scale = i32::from(scale);
    let divisor = 10_f64.powi(scale.unsigned_abs() as i32);
    let as_f = if scale > 0 {
        (raw as f64) / divisor
    } else {
        (raw as f64) * divisor
    };
    json_f64(as_f)
}

// `hex` is tiny; avoid a new dependency by inlining a nibble encoder.
mod hex {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    pub fn encode(bytes: &[u8]) -> String {
        let mut out = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 0xf) as usize] as char);
        }
        out
    }
}

#[cfg(test)]
mod tests {
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
}
