//! Default-deny-lite redaction pass for the PoC.
//!
//! The real design is a six-stage pipeline (see docs/research/capture/redaction.md);
//! this PoC proves the load-bearing property end to end: secrets seeded into
//! telemetry never reach the serialized bundle, and the bundle carries a
//! machine-readable redaction report saying what was removed.

use regex::Regex;
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub const POLICY_VERSION: &str = "poc-default-deny-lite-v0";

struct Rule {
    name: &'static str,
    pattern: Regex,
}

fn rules() -> &'static [Rule] {
    static CELL: OnceLock<Vec<Rule>> = OnceLock::new();
    CELL.get_or_init(|| {
        vec![
            Rule {
                name: "aws_access_key_id",
                pattern: Regex::new(r"\bAKIA[0-9A-Z]{16}\b").unwrap(),
            },
            Rule {
                name: "bearer_token",
                pattern: Regex::new(r"Bearer\s+[A-Za-z0-9._\-]{8,}").unwrap(),
            },
            Rule {
                name: "password_assignment",
                pattern: Regex::new(r#"(?i)password\s*[=:]\s*\S+"#).unwrap(),
            },
            Rule {
                name: "email_address",
                pattern: Regex::new(r"\b[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}\b").unwrap(),
            },
        ]
    })
}

#[derive(Debug, Default, Serialize)]
pub struct RedactionReport {
    pub policy_version: String,
    /// rule name -> number of replacements made across the whole bundle.
    pub redacted_counts: BTreeMap<String, u64>,
}

impl RedactionReport {
    pub fn new() -> Self {
        Self {
            policy_version: POLICY_VERSION.to_string(),
            redacted_counts: BTreeMap::new(),
        }
    }

    pub fn total(&self) -> u64 {
        self.redacted_counts.values().sum()
    }
}

/// Redact one string in place, accumulating counts into the report.
pub fn redact_string(input: &str, report: &mut RedactionReport) -> String {
    let mut out = input.to_string();
    for rule in rules() {
        let count = rule.pattern.find_iter(&out).count() as u64;
        if count > 0 {
            out = rule
                .pattern
                .replace_all(&out, format!("[REDACTED:{}]", rule.name))
                .into_owned();
            *report.redacted_counts.entry(rule.name.to_string()).or_insert(0) += count;
        }
    }
    out
}
