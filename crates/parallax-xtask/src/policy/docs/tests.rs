use super::*;

#[test]
fn parses_structured_front_matter_and_rejects_touch_only_docs() {
    let valid = "+++\nschema_version=1\npackage='x'\nclass='aux'\ndependencies=[]\nfacade_roots=['main.rs']\n+++\n# x\n";
    assert_eq!(parse(valid).expect("valid doc").package, "x");
    assert!(parse("# prose changed").is_err());
    assert!(parse("+++\nschema_version=1\n+++\n").is_err());
}
