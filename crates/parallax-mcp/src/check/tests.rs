use super::*;

#[test]
fn mismatch_diagnostic_does_not_disclose_evidence_or_slice_utf8() {
    let secret_a = "🦀seeded-secret-a";
    let secret_b = "🦀seeded-secret-b";
    let diagnostic = diff_err("MCP", secret_a, "CLI", secret_b).to_string();

    assert!(diagnostic.contains("byte mismatch MCP vs CLI"));
    assert!(diagnostic.contains("sha256 MCP="));
    assert!(!diagnostic.contains("seeded-secret"));
    assert!(!diagnostic.contains('🦀'));
}

#[test]
fn v2_hash_ignores_only_declared_envelope_and_data_fields() {
    let base = serde_json::json!({
        "schema_version": "bundle-v2",
        "generator": "build-a",
        "data": {
            "generator": "build-a",
            "canonical_hash": "sha256:old",
            "bounded": { "estimated_tokens": 1 },
            "evidence": "kept"
        },
        "canonical_hash": "sha256-jcs:old"
    });
    let mut excluded_changes = base.clone();
    excluded_changes["generator"] = serde_json::json!("build-b");
    excluded_changes["data"]["generator"] = serde_json::json!("build-b");
    excluded_changes["data"]["canonical_hash"] = serde_json::json!("sha256:new");
    excluded_changes["data"]["bounded"] = serde_json::json!({ "estimated_tokens": 999 });
    let mut evidence_change = base.clone();
    evidence_change["data"]["evidence"] = serde_json::json!("changed");

    let base_hash = recompute_canonical_hash(&base.to_string()).expect("base hash");
    assert!(base_hash.starts_with("sha256-jcs:"));
    assert_eq!(
        base_hash,
        recompute_canonical_hash(&excluded_changes.to_string()).expect("excluded fields")
    );
    assert_ne!(
        base_hash,
        recompute_canonical_hash(&evidence_change.to_string()).expect("evidence field")
    );
}
