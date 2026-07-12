use std::path::Path;

use anyhow::{Result, bail};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::cli::Output;
use crate::diagnostic::{Finding, Format, Severity, render};

#[derive(Debug, Default, Eq, PartialEq)]
struct Evidence {
    declared_tests: usize,
    cases: usize,
    failures: usize,
    errors: usize,
    retry_failures: usize,
}

pub(crate) fn run(root: &Path, profile: &str, output: Output) -> Result<()> {
    let relative = format!("target/nextest/{profile}/junit.xml");
    let path = root.join(&relative);
    let findings = match std::fs::read(&path) {
        Ok(xml) => validate(&xml, &relative),
        Err(error) => vec![finding(
            &relative,
            &format!("JUnit report is missing: {error}"),
        )],
    };
    let format = match output {
        Output::Human => Format::Human,
        Output::Json => Format::Json,
        Output::Github => Format::Github,
    };
    println!("{}", render(&findings, format)?);
    let errors = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .count();
    if errors > 0 {
        bail!("nextest evidence found {errors} violation(s)");
    }
    Ok(())
}

fn validate(xml: &[u8], file: &str) -> Vec<Finding> {
    match parse(xml) {
        Ok(evidence) => evidence_findings(&evidence, file),
        Err(error) => vec![finding(file, &format!("malformed JUnit XML: {error}"))],
    }
}

fn parse(xml: &[u8]) -> Result<Evidence> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut evidence = Evidence::default();
    let mut root_seen = false;
    loop {
        match reader.read_event()? {
            Event::Start(element) | Event::Empty(element) => match element.name().as_ref() {
                b"testsuites" if !root_seen => {
                    root_seen = true;
                    evidence.declared_tests = attribute_usize(&element, b"tests")?;
                    evidence.failures = attribute_usize(&element, b"failures")?;
                    evidence.errors = attribute_usize(&element, b"errors")?;
                }
                b"testcase" => evidence.cases += 1,
                b"failure" | b"error" => evidence.failures += 1,
                b"flakyFailure" | b"flakyError" | b"rerunFailure" | b"rerunError" => {
                    evidence.retry_failures += 1;
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }
    anyhow::ensure!(root_seen, "missing testsuites root");
    Ok(evidence)
}

fn attribute_usize(element: &BytesStart<'_>, name: &[u8]) -> Result<usize> {
    let value = element
        .attributes()
        .find_map(|attribute| {
            let attribute = attribute.ok()?;
            (attribute.key.as_ref() == name).then_some(attribute.value)
        })
        .ok_or_else(|| anyhow::anyhow!("missing `{}` attribute", String::from_utf8_lossy(name)))?;
    Ok(std::str::from_utf8(&value)?.parse()?)
}

fn evidence_findings(evidence: &Evidence, file: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    if evidence.declared_tests == 0 || evidence.cases == 0 {
        findings.push(finding(file, "JUnit report contains zero tests"));
    }
    if evidence.declared_tests != evidence.cases {
        findings.push(finding(
            file,
            &format!(
                "JUnit declared {} tests but contains {} cases",
                evidence.declared_tests, evidence.cases
            ),
        ));
    }
    if evidence.failures > 0 || evidence.errors > 0 {
        findings.push(finding(
            file,
            &format!(
                "JUnit contains {} failures and {} errors",
                evidence.failures, evidence.errors
            ),
        ));
    }
    if evidence.retry_failures > 0 {
        findings.push(finding(
            file,
            &format!(
                "JUnit contains {} retry/flaky failure elements",
                evidence.retry_failures
            ),
        ));
    }
    findings
}

fn finding(file: &str, reason: &str) -> Finding {
    Finding::error(
        "nextest.evidence",
        file,
        1,
        reason,
        "fix the test or structured report; never erase retry evidence",
        "cargo xtask nextest-evidence --profile <profile>",
    )
}

#[cfg(test)]
mod tests;
