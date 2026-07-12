use super::*;

#[test]
fn scope_line_split_preserves_plain_files() {
    let finding = error("x", "ui/src/a.ts:42", "reason");
    assert_eq!((finding.file.as_str(), finding.line), ("ui/src/a.ts", 42));
    let finding = error("x", "Cargo.toml", "reason");
    assert_eq!((finding.file.as_str(), finding.line), ("Cargo.toml", 1));
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
