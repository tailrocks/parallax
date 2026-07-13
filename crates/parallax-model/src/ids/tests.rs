use super::TraceId;
use std::str::FromStr;

#[test]
fn trace_id_parse_bytes_and_serde_are_wire_compatible() {
    let upper = TraceId::from_str("ABABABABABABABABABABABABABABABAB").expect("valid text");
    let bytes = TraceId::from_otlp_bytes(&[0xab; 16]).expect("valid OTLP bytes");
    assert_eq!(upper, bytes);
    assert_eq!(upper.as_str(), "abababababababababababababababab");
    assert_eq!(
        serde_json::to_string(&upper).expect("serialize"),
        format!("\"{upper}\"")
    );
    TraceId::from_str("not-a-trace").expect_err("short text rejected");
    TraceId::from_str("00000000000000000000000000000000").expect_err("zero text rejected");
    TraceId::from_otlp_bytes(&[0; 16]).expect_err("zero bytes rejected");
}
