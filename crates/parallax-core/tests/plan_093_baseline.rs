use sha2::{Digest, Sha256};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_json(path: &str) -> serde_json::Value {
    let path = repo_root().join(path);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

#[test]
fn plan_093_baseline_artifacts_validate() {
    const ROOT: &str = "docs/research/validation/2026-07-12-plan-093-baseline";
    for (document, schema) in [
        ("baseline.json", "baseline.schema.json"),
        ("defect-ledger.json", "defect-ledger.schema.json"),
    ] {
        let instance = read_json(&format!("{ROOT}/{document}"));
        let schema = read_json(&format!("{ROOT}/{schema}"));
        let validator = jsonschema::validator_for(&schema).expect("valid JSON Schema");
        let errors = validator
            .iter_errors(&instance)
            .map(|error| error.to_string())
            .collect::<Vec<_>>();
        assert!(errors.is_empty(), "{document} schema errors: {errors:#?}");
    }

    let baseline = read_json(&format!("{ROOT}/baseline.json"));
    let sdl_path = repo_root().join(format!("{ROOT}/graphql-schema.graphql"));
    let sdl = std::fs::read(&sdl_path).expect("read GraphQL SDL baseline");
    assert_eq!(
        format!("{:x}", Sha256::digest(&sdl)),
        baseline["graphql"]["sha256"].as_str().expect("SDL hash")
    );
    assert_eq!(sdl.len() as u64, baseline["graphql"]["bytes"]);
}
