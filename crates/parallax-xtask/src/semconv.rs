use std::{collections::BTreeSet, fs, path::Path, process::Command};

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use tempfile::TempDir;

const REGISTRY: &str = "telemetry/semconv/contract.yaml";
const RUST_OUTPUT: &str = "crates/parallax-semconv/src/lib.rs";
const TYPESCRIPT_OUTPUT: &str = "ui/src/shared/semconv.ts";
const JAVA_OUTPUT: &str = "telemetry/semconv/generated/java/io/tailrocks/semconv/Semconv.java";

#[derive(Clone, Debug, Deserialize)]
struct Constant {
    id: String,
    rust: String,
    typescript: String,
    java: String,
    value: Option<String>,
    values: Option<Vec<String>>,
    owner: String,
}

pub(crate) fn check(root: &Path, playground_root: Option<&Path>) -> Result<()> {
    check_weaver(root)?;
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
    println!("semantic-convention artifacts are deterministic and current");
    Ok(())
}

fn check_weaver(root: &Path) -> Result<()> {
    let status = Command::new("weaver")
        .args([
            "registry",
            "check",
            "--registry",
            "telemetry/semconv/registry",
            "--future",
        ])
        .current_dir(root)
        .status()
        .context("start pinned Weaver; run `mise install` to provision it")?;
    if !status.success() {
        bail!("Weaver rejected the semantic-convention registry: {status}");
    }
    Ok(())
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
        artifacts.extend(playground_artifacts(playground_root, &playground));
    }
    Ok(artifacts)
}

fn playground_artifacts(root: &Path, constants: &[&Constant]) -> Vec<Artifact> {
    let constants = constants
        .iter()
        .map(|constant| (*constant).clone())
        .collect::<Vec<_>>();
    vec![
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
    ]
}

fn validate(constants: &[Constant]) -> Result<()> {
    let mut ids = BTreeSet::new();
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
            if values.len() <= 2 {
                output.push_str(&format!("pub const {}: &[&str] =\n    &[", constant.rust));
                for (index, value) in values.iter().enumerate() {
                    if index > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&rust(value));
                }
                output.push_str("];\n");
            } else {
                output.push_str(&format!("pub const {}: &[&str] = &[\n", constant.rust));
                for value in values {
                    output.push_str(&format!("    {},\n", rust(value)));
                }
                output.push_str("];\n");
            }
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

fn render_typescript(constants: &[Constant]) -> String {
    let mut output = String::from(
        "// Generated from telemetry/semconv/contract.yaml.\n// Run `cargo xtask semconv generate`; do not edit by hand.\n\n",
    );
    for constant in constants {
        if let Some(value) = &constant.value {
            output.push_str(&format!(
                "export const {} = {} as const;\n",
                constant.typescript,
                json(value)
            ));
        } else if let Some(values) = &constant.values {
            output.push_str(&format!("export const {} = [", constant.typescript));
            for value in values {
                output.push_str(&format!("{}, ", json(value)));
            }
            output.push_str("] as const;\n");
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
    use super::{Constant, validate};

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
        Ok(())
    }
}
