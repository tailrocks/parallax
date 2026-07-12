use super::parse_greptime_version_output;

#[test]
fn parse_version_from_multiline_1_1_output() {
    let output = "GreptimeDB \nbranch: \ncommit: abc\nclean: true\nversion: 1.1.2\n";
    assert_eq!(
        parse_greptime_version_output(output).as_deref(),
        Some("1.1.2")
    );
}

#[test]
fn parse_version_strips_leading_v() {
    assert_eq!(
        parse_greptime_version_output("version: v1.0.0").as_deref(),
        Some("1.0.0")
    );
}

#[test]
fn parse_version_fallback_token() {
    assert_eq!(
        parse_greptime_version_output("greptime 1.1.0").as_deref(),
        Some("1.1.0")
    );
}

#[test]
fn parse_version_empty_is_none() {
    assert_eq!(parse_greptime_version_output(""), None);
}
