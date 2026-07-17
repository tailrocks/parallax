//! Plan-103 fuzz boundary: redaction text projection.
//! Oracle: no panic; sanitizing is idempotent (a second pass changes nothing).
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|text: &str| {
    let once = parallax_evidence::sanitize_text(text);
    let twice = parallax_evidence::sanitize_text(&once);
    assert_eq!(once, twice, "sanitize_text must be idempotent");
});
