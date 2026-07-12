use super::{Finding, Format, render};

#[test]
fn renderers_preserve_every_field() {
    let finding = Finding::error(
        "arch.edge",
        "Cargo.toml",
        7,
        "bad edge",
        "remove it",
        "cargo xtask arch",
    );
    for format in [Format::Human, Format::Json, Format::Github] {
        let rendered =
            render(std::slice::from_ref(&finding), format).expect("finding should render");
        let (schema, severity) = match format {
            Format::Json => ("\"schema_version\": 1", "\"severity\": \"error\""),
            Format::Human | Format::Github => ("schema=1", "severity=Error"),
        };
        for value in [
            "arch.edge",
            schema,
            severity,
            "Cargo.toml",
            "7",
            "bad edge",
            "remove it",
            "cargo xtask arch",
        ] {
            assert!(rendered.contains(value), "missing {value} from {rendered}");
        }
    }
}

#[test]
fn json_round_trips_schema() {
    let findings = vec![Finding::error(
        "policy.test",
        "x.rs",
        1,
        "reason",
        "fix",
        "rerun",
    )];
    let json = render(&findings, Format::Json).expect("finding should render as JSON");
    assert_eq!(
        serde_json::from_str::<Vec<Finding>>(&json).expect("rendered JSON should parse"),
        findings
    );
}
