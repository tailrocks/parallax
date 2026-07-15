use std::{collections::BTreeSet, fs, path::Path, process::Command};

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

mod render;

use render::{render_java, render_rust, render_typescript, render_wire_fixture};

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
    let report = check_generated_artifacts(root, playground_root)?;
    if let Some(playground_root) = playground_root {
        check_playground_test_consumer_ownership(root, playground_root)?;
    }
    Ok(report)
}

fn check_playground_test_consumer_ownership(root: &Path, playground_root: &Path) -> Result<()> {
    let document: RegistryDocument =
        serde_norway::from_str(&fs::read_to_string(root.join(REGISTRY))?)?;
    let guarded = document
        .constants
        .iter()
        .filter_map(|constant| constant.value.as_deref())
        .filter(|value| {
            value.starts_with("test.")
                || *value == "vcs.ref.head.revision"
                || matches!(*value, "assertion_failure" | "harness_error")
        })
        .collect::<Vec<_>>();
    for relative in [
        "cli/src/test_report.rs",
        "cli/src/test_verify.rs",
        "libs/playground-telemetry/src/lib.rs",
        "services/payment/src/test/java/dev/tailrocks/payment/TestTelemetryAcceptanceTest.java",
        "services/semconv/src/main/java/io/tailrocks/testsupport/OpenTelemetryTestExtension.java",
        "web/e2e/telemetry-reporter.ts",
    ] {
        let path = playground_root.join(relative);
        if !path.is_file() {
            continue;
        }
        let source = fs::read_to_string(&path)?;
        let production = source.split("#[cfg(test)]").next().unwrap_or(&source);
        for value in &guarded {
            if production.contains(&format!("\"{value}\"")) {
                bail!(
                    "playground runtime source `{relative}` duplicates generated semantic-convention value `{value}`"
                );
            }
        }
    }
    Ok(())
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
    let document: RegistryDocument = serde_norway::from_str(&source).with_context(|| {
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
mod tests;
