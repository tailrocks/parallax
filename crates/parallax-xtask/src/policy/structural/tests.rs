use super::*;

#[test]
fn scope_line_split_preserves_plain_files() {
    let finding = error("x", "ui/src/a.ts:42", "reason");
    let line_scope = (finding.file, finding.line);
    let finding = error("x", "Cargo.toml", "reason");
    assert_eq!(
        (
            line_scope,
            (finding.file, finding.line),
            is_legacy_module(Path::new("crates/a/src/mod.rs"), "crates/a/src/mod.rs"),
            is_legacy_module(Path::new("docs/mod.rs"), "docs/mod.rs"),
        ),
        (
            ("ui/src/a.ts".to_owned(), 42),
            ("Cargo.toml".to_owned(), 1),
            true,
            false,
        )
    );
}

#[test]
fn rejects_missing_growth_shrink_and_stale_rows() {
    let key = ("rust.file-lines".to_owned(), "a.rs".to_owned());
    let measured = BTreeMap::from([(key.clone(), 12)]);
    assert_eq!(
        evaluate(&measured, &BTreeMap::new())[0].rule_id,
        "structural.limit.missing"
    );
    assert_eq!(
        evaluate(&measured, &BTreeMap::from([(key.clone(), 11)]))[0].rule_id,
        "structural.limit.growth"
    );
    assert_eq!(
        evaluate(&measured, &BTreeMap::from([(key.clone(), 13)]))[0].rule_id,
        "structural.limit.stale"
    );
    assert_eq!(
        evaluate(&BTreeMap::new(), &BTreeMap::from([(key, 12)]))[0].rule_id,
        "structural.limit.stale"
    );
}
