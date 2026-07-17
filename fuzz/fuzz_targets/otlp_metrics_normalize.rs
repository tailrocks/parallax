//! Plan-103 fuzz boundary: OTLP metrics protobuf decode + normalization.
//! Oracle: no panic, no unbounded loop for arbitrary bytes.
#![no_main]

use libfuzzer_sys::fuzz_target;
use prost::Message;

fuzz_target!(|data: &[u8]| {
    if let Ok(request) =
        parallax_proto::collector_metrics::ExportMetricsServiceRequest::decode(data)
    {
        let _ = parallax_ingest::normalize_metrics(&request);
    }
});
