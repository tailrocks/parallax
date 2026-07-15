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
