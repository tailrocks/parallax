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
        "\n#[must_use]\npub fn resource_json_path(attr: &str) -> String {\n    // GreptimeDB's json_get_string wants a plainly quoted member —\n    // `$.\"a.b\"` — NOT backslash-escaped quotes (`$.\\\"a.b\\\"` matches\n    // nothing on the live engine). Embedded quotes stay escaped per the\n    // JSON-path grammar.\n    format!(r#\"$.\"{}\"\"#, attr.replace('\"', \"\\\\\\\"\"))\n}\n\n/// Prometheus-style native metric table base name: every non\n/// `[A-Za-z0-9_]` byte becomes `_` — the same normalization GreptimeDB's\n/// OTLP ingest applies when it creates per-metric tables.\n#[must_use]\npub fn native_metric_table_base(name: &str) -> String {\n    name.chars()\n        .map(|ch| {\n            if ch.is_ascii_alphanumeric() || ch == '_' {\n                ch\n            } else {\n                '_'\n            }\n        })\n        .collect()\n}\n\n#[must_use]\npub fn resource_column(attr: &str) -> String {\n    format!(\"resource_attributes.{attr}\")\n}\n\n#[must_use]\npub fn span_column(attr: &str) -> String {\n    format!(\"span_attributes.{attr}\")\n}\n",
    );
    if include_freeze_test {
        output.push_str("\n#[cfg(test)]\nmod tests;\n");
    }
    output
}

fn render_rust_values(identifier: &str, values: &[String]) -> String {
    let joined = values
        .iter()
        .map(|value| rust(value))
        .collect::<Vec<_>>()
        .join(", ");
    let compact = format!("pub const {identifier}: &[&str] = &[{joined}];");
    if compact.len() <= 100 {
        return compact + "\n";
    }
    let wrapped = format!("pub const {identifier}: &[&str] =\n    &[{joined}];");
    if values.len() <= 2 && joined.len() + 8 <= 100 {
        return wrapped + "\n";
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
            if declaration.len() > 100 {
                output.push_str(&format!(
                    "export const {} =\n  {} as const\n",
                    constant.typescript,
                    json(value)
                ));
            } else {
                output.push_str(&format!("{declaration}\n"));
            }
        } else if let Some(values) = &constant.values {
            let rendered = values.iter().map(|value| json(value)).collect::<Vec<_>>();
            let declaration = format!(
                "export const {} = [{}] as const",
                constant.typescript,
                rendered.join(", ")
            );
            if declaration.len() <= 100 {
                output.push_str(&format!("{declaration}\n"));
            } else {
                output.push_str(&format!("export const {} = [\n", constant.typescript));
                for value in rendered {
                    output.push_str(&format!("  {value},\n"));
                }
                output.push_str("] as const\n");
            }
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
