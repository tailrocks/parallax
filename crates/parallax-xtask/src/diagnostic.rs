use std::fmt::Write;

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Finding {
    pub schema_version: u32,
    pub rule_id: String,
    pub severity: Severity,
    pub file: String,
    pub line: usize,
    pub reason: String,
    pub remediation: String,
    pub rerun: String,
}

impl Finding {
    #[must_use]
    pub fn error(
        rule_id: &str,
        file: &str,
        line: usize,
        reason: &str,
        remediation: &str,
        rerun: &str,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            rule_id: rule_id.into(),
            severity: Severity::Error,
            file: file.into(),
            line,
            reason: reason.into(),
            remediation: remediation.into(),
            rerun: rerun.into(),
        }
    }

    #[must_use]
    pub fn warning(
        rule_id: &str,
        file: &str,
        line: usize,
        reason: &str,
        remediation: &str,
        rerun: &str,
    ) -> Self {
        let mut finding = Self::error(rule_id, file, line, reason, remediation, rerun);
        finding.severity = Severity::Warning;
        finding
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Format {
    Human,
    Json,
    Github,
}

pub fn render(findings: &[Finding], format: Format) -> Result<String> {
    match format {
        Format::Json => Ok(serde_json::to_string_pretty(findings)?),
        Format::Human => {
            let mut output = String::new();
            for finding in findings {
                writeln!(
                    output,
                    "{}:{} [schema={} severity={:?} rule={}] {}",
                    finding.file,
                    finding.line,
                    finding.schema_version,
                    finding.severity,
                    finding.rule_id,
                    finding.reason
                )?;
                writeln!(output, "  fix: {}", finding.remediation)?;
                writeln!(output, "  rerun: {}", finding.rerun)?;
            }
            Ok(output)
        }
        Format::Github => {
            let mut output = String::new();
            for finding in findings {
                let level = match finding.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                };
                writeln!(
                    output,
                    "::{level} file={},line={},title={}::schema={} severity={:?} {} Fix: {} Rerun: {}",
                    finding.file,
                    finding.line,
                    finding.rule_id,
                    finding.schema_version,
                    finding.severity,
                    finding.reason,
                    finding.remediation,
                    finding.rerun
                )?;
            }
            Ok(output)
        }
    }
}

#[cfg(test)]
mod tests {
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
}
