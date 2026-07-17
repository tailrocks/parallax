//! Plan-103 fuzz boundary: evidence bundle JSON parsing + canonicalization.
//! Oracle: no panic; canonical form is stable under re-canonicalization.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(data) else {
        return;
    };
    let canonical = parallax_evidence::envelope::canonical_json(&value);
    if let Ok(reparsed) = serde_json::from_str::<serde_json::Value>(&canonical) {
        assert_eq!(
            parallax_evidence::envelope::canonical_json(&reparsed),
            canonical,
            "canonical_json must be a fixpoint"
        );
    }
    let _ = parallax_evidence::envelope::parse_bundle_json(value);
});
