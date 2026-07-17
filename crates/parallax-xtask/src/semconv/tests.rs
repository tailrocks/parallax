use super::{
    Constant, check_generated_artifacts, check_playground_test_consumer_ownership,
    check_rust_ownership, generate_at, render_rust, render_typescript, validate,
};
use std::fs;
use tempfile::TempDir;

fn constant() -> Constant {
    Constant {
        id: "service.name".to_owned(),
        rust: "SERVICE_NAME".to_owned(),
        typescript: "SERVICE_NAME".to_owned(),
        java: "SERVICE_NAME".to_owned(),
        value: Some("service.name".to_owned()),
        values: None,
        owner: "shared".to_owned(),
    }
}

#[test]
fn rejects_duplicate_ids_and_invalid_contract_fields() -> Result<(), String> {
    let first = constant();
    let mut duplicate = constant();
    duplicate.rust = "SECOND_SERVICE_NAME".to_owned();
    if validate(&[first, duplicate]).is_ok() {
        return Err("duplicate semantic-convention ids were accepted".to_owned());
    }

    let mut invalid = constant();
    invalid.values = Some(vec!["service.name".to_owned()]);
    if validate(&[invalid]).is_ok() {
        return Err("scalar/list cardinality conflict was accepted".to_owned());
    }

    let mut duplicate_identifier = constant();
    duplicate_identifier.id = "event.name".to_owned();
    duplicate_identifier.value = Some("event.name".to_owned());
    if validate(&[constant(), duplicate_identifier]).is_ok() {
        return Err("duplicate generated language identifier was accepted".to_owned());
    }

    let mut empty_wire_value = constant();
    empty_wire_value.value = Some(" ".to_owned());
    if validate(&[empty_wire_value]).is_ok() {
        return Err("empty wire value was accepted".to_owned());
    }
    Ok(())
}

#[test]
fn generated_artifact_check_rejects_a_hand_edit() -> anyhow::Result<()> {
    let root = TempDir::new()?;
    let registry = root.path().join("telemetry/semconv/contract.yaml");
    fs::create_dir_all(registry.parent().expect("registry parent"))?;
    fs::write(
        registry,
        "constants:\n  - id: service.name\n    rust: SERVICE_NAME\n    typescript: SERVICE_NAME\n    java: SERVICE_NAME\n    value: service.name\n    owner: shared\n",
    )?;
    generate_at(root.path(), None)?;
    let report = check_generated_artifacts(root.path(), None)?;
    assert_eq!(report.artifacts.len(), 3);

    fs::write(
        root.path().join("ui/src/shared/semconv.ts"),
        "// hand edit\n",
    )?;
    let error = check_generated_artifacts(root.path(), None).expect_err("stale output fails");
    assert!(
        error
            .to_string()
            .contains("stale semantic-convention artifact")
    );
    Ok(())
}

#[test]
fn playground_test_consumer_ownership_rejects_wire_literals() -> anyhow::Result<()> {
    let root = TempDir::new()?;
    let playground = TempDir::new()?;
    let registry = root.path().join("telemetry/semconv/contract.yaml");
    fs::create_dir_all(registry.parent().expect("registry parent"))?;
    fs::write(
        &registry,
        "constants:\n  - id: test.attempt.ordinal\n    rust: TEST_ATTEMPT_ORDINAL\n    typescript: TEST_ATTEMPT_ORDINAL\n    java: TEST_ATTEMPT_ORDINAL\n    value: test.attempt.ordinal\n    owner: playground\n",
    )?;
    let consumer = playground.path().join("cli/src/test_report.rs");
    fs::create_dir_all(consumer.parent().expect("consumer parent"))?;
    fs::write(&consumer, "let key = semconv::TEST_ATTEMPT_ORDINAL;\n")?;
    check_playground_test_consumer_ownership(root.path(), playground.path())?;

    fs::write(&consumer, "let key = \"test.attempt.ordinal\";\n")?;
    let error = check_playground_test_consumer_ownership(root.path(), playground.path())
        .expect_err("runtime wire literal must fail");
    assert!(error.to_string().contains("duplicates generated"));

    fs::write(
        &consumer,
        "let key = semconv::TEST_ATTEMPT_ORDINAL;\n#[cfg(test)]\nmod tests { const FIXTURE: &str = \"test.attempt.ordinal\"; }\n",
    )?;
    check_playground_test_consumer_ownership(root.path(), playground.path())?;
    Ok(())
}

#[test]
fn rust_ownership_rejects_proto_bridge_and_indirect_imports() -> anyhow::Result<()> {
    let root = TempDir::new()?;
    let proto = root.path().join("crates/parallax-proto/src");
    let consumer = root.path().join("crates/consumer/src");
    fs::create_dir_all(&proto)?;
    fs::create_dir_all(&consumer)?;
    fs::write(
        consumer.join("lib.rs"),
        "use parallax_semconv as semconv;\n",
    )?;
    check_rust_ownership(root.path())?;

    fs::write(proto.join("semconv.rs"), "pub use parallax_semconv::*;\n")?;
    let bridge = check_rust_ownership(root.path()).expect_err("compatibility bridge fails");
    assert!(bridge.to_string().contains("must stay removed"));
    fs::remove_file(proto.join("semconv.rs"))?;

    fs::write(consumer.join("lib.rs"), "use parallax_proto::semconv;\n")?;
    let indirect = check_rust_ownership(root.path()).expect_err("indirect import fails");
    assert!(
        indirect
            .to_string()
            .contains("depend on parallax-semconv directly")
    );
    Ok(())
}

#[test]
fn typescript_renderer_emits_formatter_compatible_declarations() {
    let mut long = constant();
    long.typescript = "DEPLOYMENT_ENVIRONMENT_NAME".to_owned();
    long.value = Some("deployment.environment.name".to_owned());
    let mut list = constant();
    list.typescript = "REQUEST_DURATION_METRICS".to_owned();
    list.value = None;
    list.values = Some(vec![
        "http.server.request.duration".to_owned(),
        "rpc.server.duration".to_owned(),
    ]);

    let actual = render_typescript(&[constant(), long, list]);
    let expected = concat!(
        "// Generated from telemetry/semconv/contract.yaml.\n",
        "// Run `cargo xtask semconv generate`; do not edit by hand.\n\n",
        "export const SERVICE_NAME = \"service.name\" as const\n",
        "export const DEPLOYMENT_ENVIRONMENT_NAME =\n",
        "  \"deployment.environment.name\" as const\n",
        "export const REQUEST_DURATION_METRICS = [\n",
        "  \"http.server.request.duration\",\n",
        "  \"rpc.server.duration\",\n",
        "] as const\n",
    );
    assert_eq!(actual, expected);
}

#[test]
fn rust_renderer_keeps_short_lists_rustfmt_clean() {
    let mut list = constant();
    list.value = None;
    list.values = Some(vec![
        "claude".to_owned(),
        "codex".to_owned(),
        "amp".to_owned(),
    ]);
    let actual = render_rust(&[list], false);
    assert!(
        actual.contains("pub const SERVICE_NAME: &[&str] = &[\"claude\", \"codex\", \"amp\"];")
    );
}
