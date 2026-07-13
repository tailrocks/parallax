use super::*;

#[test]
fn parses_structured_front_matter_and_rejects_touch_only_docs() {
    let valid = "+++\nschema_version=1\npackage='x'\nclass='aux'\ndependencies=[]\nfacade_roots=['main.rs']\n+++\n# x\n";
    assert_eq!(parse(valid).expect("valid doc").package, "x");
    parse("# prose changed").unwrap_err();
    parse("+++\nschema_version=1\n+++\n").unwrap_err();
}

#[test]
fn evidence_bundle_decision_gate_fails_closed() {
    let draft = include_str!("../../../../../docs/research/decisions/evidence-bundle-contract.md");
    approved_evidence_bundle_decision(draft).expect_err("draft decision rejected");

    let approved = r#"+++
status = "approved"
canonical_model = "bundle-v1"
contract_version = "bundle-v1"
compatibility_window = "permanent"
migration_behavior = "no migration"
approved_by = "operator"
approval_date = "2026-07-13"
+++
# Decision
"#;
    approved_evidence_bundle_decision(approved).expect("complete approval accepted");

    let incomplete = approved.replace("bundle-v1", "UNRESOLVED");
    approved_evidence_bundle_decision(&incomplete).expect_err("unresolved approval rejected");
}

#[test]
fn semantic_crate_body_checks_roots_links_and_gates() {
    let directory = tempfile::tempdir().expect("temp crate");
    fs::create_dir(directory.path().join("src")).expect("src");
    fs::write(directory.path().join("src/lib.rs"), "").expect("root");
    fs::write(directory.path().join("facade.toml"), "").expect("facade");
    let source = "+++\nschema_version=1\npackage='x'\nclass='aux'\ndependencies=[]\nfacade_roots=['lib.rs']\n+++\n# x\n\n## Owned concerns\n\nPolicy.\n\n## Source map\n\n- [root](src/lib.rs)\n- [facade](facade.toml)\n\n## Public surface\n\nSee [facade](facade.toml).\n\n## Verification\n\n`cargo test -p x` and `cargo xtask facade check`.\n";
    let doc = parse(source).expect("crate doc");
    let facade = Facade {
        roots: BTreeMap::from([("lib.rs".to_string(), Vec::new())]),
    };
    check_semantic_body(directory.path(), &doc, &facade).expect("semantic body");

    let missing_root = source.replace("- [root](src/lib.rs)\n", "");
    let doc = parse(&missing_root).expect("crate doc");
    check_semantic_body(directory.path(), &doc, &facade).unwrap_err();
}

#[test]
fn handoff_schema_rejects_placeholders_and_malformed_rows() {
    let valid = "## Incoming Handoff From 127\n\n| Stable ID | Owner | Consumers | Target | Status |\n|---|---|---|---|---|\n| `127-store` | current | users | target; Plan 097 | OWNED |\n";
    assert_eq!(parse_handoff(valid).expect("valid handoff").len(), 1);

    let pending = valid.replace("127-store", "127-pending");
    parse_handoff(&pending).unwrap_err();
    let malformed = valid.replace(" | target; Plan 097", "");
    parse_handoff(&malformed).unwrap_err();
}
