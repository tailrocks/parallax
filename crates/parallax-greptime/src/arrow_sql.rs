#![expect(
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    reason = "Arrow schema conversions preserve source representation"
)]

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
/// Public for the plan-103 fuzz boundary (`fuzz/fuzz_targets/arrow_decode.rs`):
/// arbitrary bytes must never panic or allocate unboundedly here.
pub fn decode_arrow_ipc(bytes: &[u8]) -> anyhow::Result<(Vec<String>, Vec<Vec<Value>>)> {
    if bytes.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    // Greptime returns a JSON error envelope even when format=arrow was requested.
    if bytes.first() == Some(&b'{') {
        return Err(map_greptime_json_error(bytes));
    }

    // The IPC stream's length prefixes drive buffer allocations inside the
    // arrow reader; a corrupt/hostile prefix can demand gigabytes from a
    // few bytes of input. Walk the message framing first and reject any
    // declared length that exceeds the bytes we actually hold.
    validate_ipc_frame_lengths(bytes)?;
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

/// Reject a stream whose first message declares more metadata than the
/// payload holds — the cheap hostile-prefix case where a few bytes demand
/// gigabytes from the arrow reader. Deeper messages are framed by lengths
/// inside verified flatbuffer metadata; walking them without full parsing
/// is not possible here, so this guards the first-allocation class only.
fn validate_ipc_frame_lengths(bytes: &[u8]) -> anyhow::Result<()> {
    let mut offset = 0usize;
    let mut word = |offset: &mut usize| -> Option<u32> {
        let end = *offset + 4;
        let value = bytes
            .get(*offset..end)
            .map(|b| u32::from_le_bytes(b.try_into().expect("4 bytes")));
        *offset = end;
        value
    };
    let Some(first) = word(&mut offset) else {
        return Ok(());
    };
    let declared = if first == u32::MAX {
        match word(&mut offset) {
            Some(len) => len,
            None => return Ok(()),
        }
    } else {
        first
    };
    anyhow::ensure!(
        (declared as usize) <= bytes.len().saturating_sub(offset),
        "arrow-ipc stream declares {declared} metadata bytes but only {} remain",
        bytes.len().saturating_sub(offset)
    );
    Ok(())
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
        DataType::Int8 => primitive::<Int8Type>(array).map(|a| json_i64(a.value(row).into())),
        DataType::Int16 => primitive::<Int16Type>(array).map(|a| json_i64(a.value(row).into())),
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
    pub(super) fn encode(bytes: &[u8]) -> String {
        let mut out = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 0xf) as usize] as char);
        }
        out
    }
}

#[cfg(test)]
mod tests;
