use super::*;

#[test]
fn parses_functions_closures_and_comment_free_logical_lines() {
    let source = "// comment\nfn work() {\n  let f = || {\n    if true { 1 } else { 2 }\n  };\n}\n\n/* ignored */\n";
    let metric = analyze(source).expect("fixture should parse");
    assert_eq!(metric.logical_lines, 5);
    assert_eq!(metric.functions.len(), 2);
    assert_eq!(metric.functions[0].name, "work");
    assert!(metric.functions[1].cognitive > 0);
}

#[test]
fn malformed_rust_fails_closed() {
    analyze("fn {").unwrap_err();
}

#[test]
fn distinguishes_inline_bodies_from_external_markers_and_macros() {
    let inline = analyze("#[cfg(test)] mod tests { #[test] fn works() {} }")
        .expect("inline fixture should parse");
    assert_eq!(inline.inline_test_modules, 1);

    let external = analyze("#[cfg(test)] mod tests;").expect("external fixture should parse");
    assert_eq!(external.inline_test_modules, 0);

    let generated =
        analyze("macro_rules! generated_tests { () => { #[cfg(test)] mod tests { } }; }")
            .expect("macro fixture should parse");
    assert_eq!(generated.inline_test_modules, 0);
}

#[test]
fn detects_nondeterministic_test_harness_calls() {
    let metric = analyze(
        r#"fn harness() {
            std::env::set_var("K", "V");
            std::env::remove_var("K");
            std::thread::sleep(duration);
            tokio::time::sleep(duration);
            std::net::TcpListener::bind("127.0.0.1:4317");
            std::time::SystemTime::now();
            Instant::now();
            std::env::temp_dir();
        }"#,
    )
    .expect("determinism fixture should parse");
    assert_eq!(
        (
            metric.determinism.environment_mutations,
            metric.determinism.sleeps,
            metric.determinism.listener_binds,
            metric.determinism.wall_clocks,
            metric.determinism.temp_root_accesses,
        ),
        (2, 2, 1, 2, 1)
    );
}

#[test]
fn parses_reasoned_direct_and_cfg_attr_suppressions() {
    let metric = analyze(
        r#"
        #![cfg_attr(test, allow(clippy::unwrap_used, reason = "fixture assertion"))]
        #[expect(clippy::too_many_arguments, reason = "wire contract")]
        fn direct() {}
        "#,
    )
    .expect("suppression fixture should parse");
    let expected = [
        suppressions::Suppression {
            lint: "clippy::unwrap_used".into(),
            reason: Some("fixture assertion".into()),
        },
        suppressions::Suppression {
            lint: "clippy::too_many_arguments".into(),
            reason: Some("wire contract".into()),
        },
    ];
    if metric.suppression_details != expected {
        panic!(
            "suppression parsing drifted: {:?}",
            metric.suppression_details
        );
    }
}
