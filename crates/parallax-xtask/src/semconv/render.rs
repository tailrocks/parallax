use anyhow::{Context, Result};
use serde::Serialize;

use super::Constant;

pub(super) fn render_rust(constants: &[Constant], include_freeze_test: bool) -> String {
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
        "\n#[must_use]\npub fn resource_json_path(attr: &str) -> String {\n    format!(r#\"$.\\\"{}\\\"\"#, attr.replace('\"', \"\\\\\\\"\"))\n}\n\n#[must_use]\npub fn resource_column(attr: &str) -> String {\n    format!(\"resource_attributes.{attr}\")\n}\n\n#[must_use]\npub fn span_column(attr: &str) -> String {\n    format!(\"span_attributes.{attr}\")\n}\n",
    );
    if include_freeze_test {
        output.push_str("\n#[cfg(test)]\nmod tests;\n");
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

pub(super) fn render_typescript(constants: &[Constant]) -> String {
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

pub(super) fn render_java(constants: &[Constant]) -> String {
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

pub(super) fn render_wire_fixture(constants: &[Constant]) -> Result<String> {
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
    format!("{value:?}")
}

fn json(value: &str) -> String {
    format!("{value:?}")
}
