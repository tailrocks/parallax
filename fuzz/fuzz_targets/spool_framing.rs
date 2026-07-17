//! Plan-103 fuzz boundary: spool PSPL frame counting over arbitrary
//! on-disk bytes. Oracle: no panic and no unbounded allocation (a hostile
//! length prefix must not allocate; defect class fixed 2026-07-17).
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let dir = std::env::temp_dir().join(format!("parallax-fuzz-spool-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    std::fs::write(dir.join("traces.pspl"), data).expect("write corpus file");
    let spool = parallax_spool::Spool::open(&dir).expect("open spool");
    let _ = spool.line_count(parallax_spool::Signal::Traces);
});
