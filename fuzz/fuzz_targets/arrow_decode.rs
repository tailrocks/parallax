//! Plan-103 fuzz boundary: GreptimeDB Arrow IPC response decode.
//! Oracle: no panic and no unbounded allocation for arbitrary bytes.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = parallax_greptime::arrow_sql::decode_arrow_ipc(data);
});
