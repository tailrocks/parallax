use super::*;

#[test]
fn parses_structured_front_matter_and_rejects_touch_only_docs() {
    let valid = "+++\nschema_version=1\npackage='x'\nclass='aux'\ndependencies=[]\nfacade_roots=['main.rs']\n+++\n# x\n";
    assert_eq!(parse(valid).expect("valid doc").package, "x");
    assert!(parse("# prose changed").is_err());
    assert!(parse("+++\nschema_version=1\n+++\n").is_err());
}

#[test]
fn handoff_schema_rejects_placeholders_and_malformed_rows() {
    let valid = "## Incoming Handoff From 127\n\n| Stable ID | Owner | Consumers | Target | Status |\n|---|---|---|---|---|\n| `127-store` | current | users | target; Plan 097 | OWNED |\n";
    assert_eq!(parse_handoff(valid).expect("valid handoff").len(), 1);

    let pending = valid.replace("127-store", "127-pending");
    assert!(parse_handoff(&pending).is_err());
    let malformed = valid.replace(" | target; Plan 097", "");
    assert!(parse_handoff(&malformed).is_err());
}
