use super::*;

fn report(tests: usize, body: &str) -> String {
    format!(
        r#"<testsuites name="x" tests="{tests}" failures="0" errors="0"><testsuite name="s" tests="{tests}" disabled="0" failures="0" errors="0">{body}</testsuite></testsuites>"#
    )
}

#[test]
fn accepts_nonempty_passing_report() {
    assert!(
        validate(
            report(1, r#"<testcase name="ok"/>"#).as_bytes(),
            "junit.xml"
        )
        .is_empty()
    );
}

#[test]
fn rejects_zero_malformed_failure_and_retry_pass_reports() {
    for xml in [
        report(0, ""),
        "not xml".to_string(),
        report(1, r#"<testcase name="bad"><failure/></testcase>"#),
        report(1, r#"<testcase name="flaky"><flakyFailure/></testcase>"#),
    ] {
        assert!(!validate(xml.as_bytes(), "junit.xml").is_empty(), "{xml}");
    }
}
