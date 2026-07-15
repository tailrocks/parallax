use std::{collections::BTreeSet, fs, path::Path, process::Command};

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

const REGISTRY: &str = "telemetry/semconv/contract.yaml";
const RUST_OUTPUT: &str = "crates/parallax-semconv/src/lib.rs";
const TYPESCRIPT_OUTPUT: &str = "ui/src/shared/semconv.ts";
const JAVA_OUTPUT: &str = "telemetry/semconv/generated/java/io/tailrocks/semconv/Semconv.java";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Constant {
    id: String,
    rust: String,
    typescript: String,
    java: String,
    value: Option<String>,
    values: Option<Vec<String>>,
    owner: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct CheckReport {
    pub(crate) schema_version: u32,
    pub(crate) artifacts: Vec<String>,
}

pub(crate) fn check(root: &Path, playground_root: Option<&Path>) -> Result<CheckReport> {
    check_weaver(root)?;
    check_rust_ownership(root)?;
    check_generated_artifacts(root, playground_root)
}

fn check_rust_ownership(root: &Path) -> Result<()> {
    let compatibility = root.join("crates/parallax-proto/src/semconv.rs");
    if compatibility.exists() {
        bail!(
            "obsolete semantic-convention compatibility module `{}` must stay removed",
            compatibility.display()
        );
    }
    check_rust_sources(&root.join("crates"))
}

fn check_rust_sources(directory: &Path) -> Result<()> {
    for entry in fs::read_dir(directory)
        .with_context(|| format!("read Rust ownership directory `{}`", directory.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            check_rust_sources(&path)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let source = fs::read_to_string(&path)
                .with_context(|| format!("read Rust source `{}`", path.display()))?;
            if source.lines().any(|line| {
                let line = line.trim_start();
                (line.starts_with("use ") || line.starts_with("pub use "))
                    && line.contains("parallax_proto::semconv")
            }) {
                bail!(
                    "Rust source `{}` imports semantic conventions through parallax-proto; depend on parallax-semconv directly",
                    path.display()
                );
            }
        }
    }
    Ok(())
}

fn check_generated_artifacts(root: &Path, playground_root: Option<&Path>) -> Result<CheckReport> {
    let artifacts = artifacts(root, playground_root)?;
    let temporary = TempDir::new().context("create semantic-convention temporary directory")?;
    for artifact in &artifacts {
        let generated = temporary.path().join(&artifact.path);
        write(&generated, &artifact.contents)?;
        let checked_in = artifact.root.join(&artifact.path);
        let actual = fs::read_to_string(&checked_in)
            .with_context(|| format!("read checked-in `{}`", checked_in.display()))?;
        if actual != artifact.contents {
            bail!(
                "stale semantic-convention artifact `{}`; run `cargo xtask semconv generate`",
                artifact.path.display()
            );
        }
    }
    Ok(CheckReport {
        schema_version: 1,
        artifacts: artifacts
            .iter()
            .map(|artifact| artifact.path.display().to_string())
            .collect(),
    })
}

fn check_weaver(root: &Path) -> Result<()> {
    let registry = root.join("telemetry/semconv/registry");
    let output = run_weaver(root, &registry)?;
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Weaver rejected the semantic-convention registry ({status}):\nstdout:\n{stdout}\nstderr:\n{stderr}",
            status = output.status
        );
    }

    let fixtures_root = root.join("telemetry/semconv/fixtures");
    for name in [
        "invalid-stability",
        "invalid-type",
        "unknown-import",
        "unknown-reference",
    ] {
        let fixture = fixtures_root.join(name);
        ensure!(fixture.is_dir(), "missing Weaver negative fixture `{name}`");
        let output = run_weaver(root, &fixture)?;
        ensure!(
            !output.status.success(),
            "Weaver negative fixture `{}` unexpectedly passed",
            fixture.display()
        );
    }
    Ok(())
}

fn run_weaver(root: &Path, registry: &Path) -> Result<std::process::Output> {
    Command::new("weaver")
        .args(["registry", "check", "--registry"])
        .arg(registry)
        .arg("--future")
        .arg("--quiet")
        .current_dir(root)
        .output()
        .context("start pinned Weaver; run `mise install` to provision it")
}

pub(crate) fn generate(root: &Path) -> Result<()> {
    generate_at(root, None)
}

pub(crate) fn generate_with_playground(root: &Path, playground_root: &Path) -> Result<()> {
    generate_at(root, Some(playground_root))
}

fn generate_at(root: &Path, playground_root: Option<&Path>) -> Result<()> {
    for artifact in artifacts(root, playground_root)? {
        write(&artifact.root.join(&artifact.path), &artifact.contents)?;
        println!("generated {}", artifact.path.display());
    }
    Ok(())
}

struct Artifact {
    root: std::path::PathBuf,
    path: std::path::PathBuf,
    contents: String,
}

#[derive(Debug, Deserialize)]
struct RegistryDocument {
    constants: Vec<Constant>,
}

fn artifacts(root: &Path, playground_root: Option<&Path>) -> Result<Vec<Artifact>> {
    let registry_path = root.join(REGISTRY);
    let source = fs::read_to_string(&registry_path).with_context(|| {
        format!(
            "read semantic-convention registry `{}`",
            registry_path.display()
        )
    })?;
    let document: RegistryDocument = serde_yml::from_str(&source).with_context(|| {
        format!(
            "parse semantic-convention registry `{}`",
            registry_path.display()
        )
    })?;
    let constants = document.constants;
    validate(&constants)?;
    let mut artifacts = vec![
        Artifact {
            root: root.to_owned(),
            path: RUST_OUTPUT.into(),
            contents: render_rust(&constants, true),
        },
        Artifact {
            root: root.to_owned(),
            path: TYPESCRIPT_OUTPUT.into(),
            contents: render_typescript(&constants),
        },
        Artifact {
            root: root.to_owned(),
            path: JAVA_OUTPUT.into(),
            contents: render_java(&constants),
        },
    ];
    if let Some(playground_root) = playground_root {
        let playground = constants
            .iter()
            .filter(|constant| constant.owner != "parallax")
            .collect::<Vec<_>>();
        artifacts.extend(playground_artifacts(playground_root, &playground)?);
    }
    Ok(artifacts)
}

fn playground_artifacts(root: &Path, constants: &[&Constant]) -> Result<Vec<Artifact>> {
    let constants = constants
        .iter()
        .map(|constant| (*constant).clone())
        .collect::<Vec<_>>();
    Ok(vec![
        Artifact {
            root: root.to_owned(),
            path: "libs/playground-telemetry/src/semconv.rs".into(),
            contents: render_rust(&constants, false),
        },
        Artifact {
            root: root.to_owned(),
            path: "web/src/semconv.ts".into(),
            contents: render_typescript(&constants),
        },
        Artifact {
            root: root.to_owned(),
            path: "services/semconv/src/main/java/io/tailrocks/semconv/Semconv.java".into(),
            contents: render_java(&constants),
        },
        Artifact {
            root: root.to_owned(),
            path: "fixtures/semconv-wire-contract.json".into(),
            contents: render_wire_fixture(&constants)?,
        },
    ])
}

fn validate(constants: &[Constant]) -> Result<()> {
    let mut ids = BTreeSet::new();
    let mut rust_identifiers = BTreeSet::new();
    let mut typescript_identifiers = BTreeSet::new();
    let mut java_identifiers = BTreeSet::new();
    for constant in constants {
        if !ids.insert(&constant.id) {
            bail!("duplicate semantic-convention id `{}`", constant.id);
        }
        let has_value = constant.value.is_some();
        let has_values = constant.values.is_some();
        if has_value == has_values {
            bail!(
                "semantic-convention `{}` must have exactly one of `value` or `values`",
                constant.id
            );
        }
        if constant
            .value
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
            || constant.values.as_ref().is_some_and(|values| {
                values.is_empty() || values.iter().any(|value| value.trim().is_empty())
            })
        {
            bail!(
                "semantic-convention `{}` has an empty wire value",
                constant.id
            );
        }
        for (language, identifier) in [
            ("rust", &constant.rust),
            ("typescript", &constant.typescript),
            ("java", &constant.java),
        ] {
            if !identifier
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte == b'_' || byte.is_ascii_digit())
            {
                bail!(
                    "semantic-convention `{}` has invalid {language} identifier `{identifier}`",
                    constant.id
                );
            }
        }
        for (language, identifiers, identifier) in [
            ("rust", &mut rust_identifiers, &constant.rust),
            (
                "typescript",
                &mut typescript_identifiers,
                &constant.typescript,
            ),
            ("java", &mut java_identifiers, &constant.java),
        ] {
            if !identifiers.insert(identifier) {
                bail!(
                    "semantic-convention `{}` duplicates {language} identifier `{identifier}`",
                    constant.id
                );
            }
        }
        if !matches!(
            constant.owner.as_str(),
            "shared" | "parallax" | "playground"
        ) {
            bail!(
                "semantic-convention `{}` has invalid owner `{}`",
                constant.id,
                constant.owner
            );
        }
    }
    Ok(())
}

fn render_rust(constants: &[Constant], include_freeze_test: bool) -> String {
    let mut output = String::from(
        "//! Generated semantic-convention names shared by Parallax producers and consumers.\n//!\n//! Source: `telemetry/semconv/contract.yaml`. Do not edit by hand;\n//! run `cargo xtask semconv generate`. Product builds depend only on this\n//! dependency-free crate, never on the generator or Weaver.\n\n",
    );
    for constant in constants {
        if let Some(value) = &constant.value {
            output.push_str(&format!(
                "pub const {}: &str = {};\n",
                constant.rust,
                rust(value)
            ));
        } else if let Some(values) = &constant.values {
            output.push_str(&render_rust_values(&constant.rust, values));
        }
    }
    output.push_str(
        "\n#[must_use]\npub fn resource_json_path(attr: &str) -> String {\n    format!(r#\"$.\\\"{}\\\"\"#, attr.replace('\"', \"\\\\\\\"\"))\n}\n\n#[must_use]\npub fn resource_column(attr: &str) -> String {\n    format!(\"resource_attributes.{attr}\")\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn preserves_load_bearing_wire_names() -> Result<(), String> {\n        let actual = (\n            SERVICE_NAME,\n            EVENT_NAME,\n            PARALLAX_RUN_ID,\n            BUNDLE_WINDOW_METRICS,\n        );\n        let expected = (\n            \"service.name\",\n            \"event.name\",\n            \"parallax.run.id\",\n            &[\n                \"process.cpu.utilization\",\n                \"process.memory.usage\",\n                \"tokio.runtime.alive_tasks\",\n            ][..],\n        );\n        if actual != expected {\n            return Err(format!(\"semantic-convention wire-name drift: {actual:?}\"));\n        }\n        Ok(())\n    }\n}\n",
    );
    if !include_freeze_test {
        let test_start = output.find("\n#[cfg(test)]").unwrap_or(output.len());
        output.truncate(test_start);
    }
    output
}

fn render_rust_values(identifier: &str, values: &[String]) -> String {
    if values.len() <= 2 {
        let values = values
            .iter()
            .map(|value| rust(value))
            .collect::<Vec<_>>()
            .join(", ");
        return format!("pub const {identifier}: &[&str] =\n    &[{values}];\n");
    }
    let values = values
        .iter()
        .map(|value| format!("    {},\n", rust(value)))
        .collect::<String>();
    format!("pub const {identifier}: &[&str] = &[\n{values}];\n")
}

fn render_typescript(constants: &[Constant]) -> String {
    let mut output = String::from(
        "// Generated from telemetry/semconv/contract.yaml.\n// Run `cargo xtask semconv generate`; do not edit by hand.\n\n",
    );
    for constant in constants {
        if let Some(value) = &constant.value {
            let declaration = format!(
                "export const {} = {} as const",
                constant.typescript,
                json(value)
            );
            if declaration.len() > 80 {
                output.push_str(&format!(
                    "export const {} =\n  {} as const\n",
                    constant.typescript,
                    json(value)
                ));
            } else {
                output.push_str(&format!("{declaration}\n"));
            }
        } else if let Some(values) = &constant.values {
            output.push_str(&format!("export const {} = [\n", constant.typescript));
            for value in values {
                output.push_str(&format!("  {},\n", json(value)));
            }
            output.push_str("] as const\n");
        }
    }
    output
}

fn render_java(constants: &[Constant]) -> String {
    let mut output = String::from(
        "// Generated from telemetry/semconv/contract.yaml.\n// Run `cargo xtask semconv generate`; do not edit by hand.\npackage io.tailrocks.semconv;\n\npublic final class Semconv {\n    private Semconv() {}\n\n",
    );
    for constant in constants {
        if let Some(value) = &constant.value {
            output.push_str(&format!(
                "    public static final String {} = {};\n",
                constant.java,
                json(value)
            ));
        } else if let Some(values) = &constant.values {
            output.push_str(&format!(
                "    public static final String[] {} = {{",
                constant.java
            ));
            for value in values {
                output.push_str(&format!("{}, ", json(value)));
            }
            output.push_str("};\n");
        }
    }
    output.push_str("}\n");
    output
}

fn render_wire_fixture(constants: &[Constant]) -> Result<String> {
    #[derive(Serialize)]
    struct Fixture<'a> {
        schema_version: u32,
        constants: &'a [Constant],
    }

    let rendered = serde_json::to_string_pretty(&Fixture {
        schema_version: 1,
        constants,
    })
    .context("serialize semantic-convention wire fixture")?;
    Ok(rendered + "\n")
}

fn rust(value: &str) -> String {
    format!("{:?}", value)
}

fn json(value: &str) -> String {
    format!("{value:?}")
}

fn write(path: &Path, contents: &str) -> Result<()> {
    let parent = path
        .parent()
        .context("semantic-convention artifact has no parent directory")?;
    fs::create_dir_all(parent).with_context(|| {
        format!(
            "create semantic-convention directory `{}`",
            parent.display()
        )
    })?;
    fs::write(path, contents)
        .with_context(|| format!("write semantic-convention artifact `{}`", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{
        Constant, check_generated_artifacts, check_rust_ownership, generate_at, render_typescript,
        validate,
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
}
