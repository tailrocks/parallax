use super::*;

#[test]
fn dry_run_accepts_valid_and_rejects_tampered_contracts() -> Result<()> {
    dry_run_fixtures()
}

#[test]
fn parser_rejects_duplicate_or_missing_fields() {
    let invalid = "auditor=a;c0=0;c1=0;tree=0;result=pass";
    parse_attestation(invalid).unwrap_err();
}

#[test]
fn evidence_paths_accept_any_single_valid_date() -> Result<()> {
    let paths = ["a.json", "a.md", "b.json", "b.md"]
        .map(|suffix| format!("docs/research/validation/2031-04-09-active-plans-closure-{suffix}"));
    ensure!(validate_evidence_paths(&paths)? == "2031-04-09");
    Ok(())
}

#[test]
fn evidence_paths_reject_mixed_dates() {
    let paths = vec![
        "docs/research/validation/2031-04-09-active-plans-closure-a.json".to_owned(),
        "docs/research/validation/2031-04-09-active-plans-closure-a.md".to_owned(),
        "docs/research/validation/2031-04-10-active-plans-closure-b.json".to_owned(),
        "docs/research/validation/2031-04-09-active-plans-closure-b.md".to_owned(),
    ];
    validate_evidence_paths(&paths).unwrap_err();
}

#[test]
fn artifact_hashes_bind_repository_bytes() -> Result<()> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("report.txt"), "passing evidence")?;
    let digest = format!("{:x}", Sha256::digest(b"passing evidence"));
    let mut packet = serde_json::json!({"artifact_hashes": {"report.txt": digest}});
    packet::validate_artifacts(root.path(), &packet)?;
    packet["artifact_hashes"]["report.txt"] =
        serde_json::json!("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
    ensure!(packet::validate_artifacts(root.path(), &packet).is_err());
    Ok(())
}
