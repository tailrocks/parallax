//! Bounded JUnit XML authority-layer normalization for killed/retried tests.

use quick_xml::Reader;
use quick_xml::XmlVersion;
use quick_xml::events::{BytesStart, Event};
use std::collections::BTreeSet;
use std::fmt;

const MAX_XML_BYTES: usize = 16 * 1024 * 1024;
const MAX_CASES: usize = 100_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JUnitOutcome {
    Passed,
    Failed,
    Broken,
    Skipped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JUnitCaseEvidence {
    pub suite_id: String,
    pub class_name: Option<String>,
    pub name: String,
    pub outcome: JUnitOutcome,
    pub prior_assertion_failures: u32,
    pub prior_harness_errors: u32,
}

impl JUnitCaseEvidence {
    #[must_use]
    pub fn attempt_count(&self) -> u32 {
        self.prior_assertion_failures
            .saturating_add(self.prior_harness_errors)
            .saturating_add(1)
    }

    #[must_use]
    pub fn missing_attempts(&self, observed: &BTreeSet<u32>) -> Vec<u32> {
        (1..=self.attempt_count())
            .filter(|attempt| !observed.contains(attempt))
            .collect()
    }

    #[must_use]
    pub fn code_reference(&self) -> String {
        self.class_name.as_ref().map_or_else(
            || format!("{}::{}", self.suite_id, self.name),
            |class_name| format!("{}::{class_name}::{}", self.suite_id, self.name),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JUnitParseError {
    TooLarge,
    TooManyCases,
    Malformed,
    MissingSuite,
    MissingCaseName,
}

impl fmt::Display for JUnitParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::TooLarge => "JUnit XML exceeds the byte limit",
            Self::TooManyCases => "JUnit XML exceeds the test-case limit",
            Self::Malformed => "JUnit XML is malformed",
            Self::MissingSuite => "JUnit testcase has no containing suite identity",
            Self::MissingCaseName => "JUnit testcase has no name",
        })
    }
}

impl std::error::Error for JUnitParseError {}

#[derive(Default)]
struct PendingCase {
    suite_id: String,
    class_name: Option<String>,
    name: String,
    outcome: Option<JUnitOutcome>,
    prior_assertion_failures: u32,
    prior_harness_errors: u32,
}

pub fn parse_junit_cases(xml: &[u8]) -> Result<Vec<JUnitCaseEvidence>, JUnitParseError> {
    if xml.len() > MAX_XML_BYTES {
        return Err(JUnitParseError::TooLarge);
    }
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut suites = Vec::<String>::new();
    let mut pending = None;
    let mut cases = Vec::new();
    loop {
        let event = reader
            .read_event()
            .map_err(|_| JUnitParseError::Malformed)?;
        match event {
            Event::Start(element) => match element.name().as_ref() {
                b"testsuite" => suites.push(required_attribute(&reader, &element, b"name")?),
                b"testcase" => {
                    pending = Some(start_case(&reader, &element, suites.last())?);
                }
                name => mark_case(name, pending.as_mut()),
            },
            Event::Empty(element) => match element.name().as_ref() {
                b"testcase" => {
                    let case = start_case(&reader, &element, suites.last())?;
                    push_case(&mut cases, case)?;
                }
                name => mark_case(name, pending.as_mut()),
            },
            Event::End(element) => match element.name().as_ref() {
                b"testcase" => {
                    let case = pending.take().ok_or(JUnitParseError::Malformed)?;
                    push_case(&mut cases, case)?;
                }
                b"testsuite" => {
                    suites.pop().ok_or(JUnitParseError::Malformed)?;
                }
                _ => {}
            },
            Event::Eof => break,
            _ => {}
        }
    }
    if pending.is_some() || !suites.is_empty() {
        return Err(JUnitParseError::Malformed);
    }
    Ok(cases)
}

fn start_case(
    reader: &Reader<&[u8]>,
    element: &BytesStart<'_>,
    suite: Option<&String>,
) -> Result<PendingCase, JUnitParseError> {
    let suite_id = suite.cloned().ok_or(JUnitParseError::MissingSuite)?;
    let name = required_attribute(reader, element, b"name")?;
    let class_name = optional_attribute(reader, element, b"classname")?;
    Ok(PendingCase {
        suite_id,
        class_name,
        name,
        ..PendingCase::default()
    })
}

fn mark_case(name: &[u8], pending: Option<&mut PendingCase>) {
    let Some(case) = pending else { return };
    match name {
        b"failure" => case.outcome = Some(JUnitOutcome::Failed),
        b"error" => case.outcome = Some(JUnitOutcome::Broken),
        b"skipped" => case.outcome = Some(JUnitOutcome::Skipped),
        b"flakyFailure" | b"rerunFailure" => {
            case.prior_assertion_failures = case.prior_assertion_failures.saturating_add(1);
        }
        b"flakyError" | b"rerunError" => {
            case.prior_harness_errors = case.prior_harness_errors.saturating_add(1);
        }
        _ => {}
    }
}

fn push_case(cases: &mut Vec<JUnitCaseEvidence>, case: PendingCase) -> Result<(), JUnitParseError> {
    if cases.len() >= MAX_CASES {
        return Err(JUnitParseError::TooManyCases);
    }
    cases.push(JUnitCaseEvidence {
        suite_id: case.suite_id,
        class_name: case.class_name,
        name: case.name,
        outcome: case.outcome.unwrap_or(JUnitOutcome::Passed),
        prior_assertion_failures: case.prior_assertion_failures,
        prior_harness_errors: case.prior_harness_errors,
    });
    Ok(())
}

fn required_attribute(
    reader: &Reader<&[u8]>,
    element: &BytesStart<'_>,
    name: &[u8],
) -> Result<String, JUnitParseError> {
    optional_attribute(reader, element, name)?
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            if name == b"name" && element.name().as_ref() == b"testcase" {
                JUnitParseError::MissingCaseName
            } else {
                JUnitParseError::MissingSuite
            }
        })
}

fn optional_attribute(
    reader: &Reader<&[u8]>,
    element: &BytesStart<'_>,
    name: &[u8],
) -> Result<Option<String>, JUnitParseError> {
    for attribute in element.attributes() {
        let attribute = attribute.map_err(|_| JUnitParseError::Malformed)?;
        if attribute.key.as_ref() == name {
            return attribute
                .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
                .map(|value| Some(value.trim().to_owned()))
                .map_err(|_| JUnitParseError::Malformed);
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_nextest_retries_and_killed_test_gap() {
        let xml = br#"<testsuites><testsuite name="crate::integration">
          <testcase classname="suite" name="retry"><flakyFailure/><rerunError/></testcase>
          <testcase classname="suite" name="killed"><error/></testcase>
          <testcase classname="suite" name="skipped"><skipped/></testcase>
        </testsuite></testsuites>"#;
        let cases = parse_junit_cases(xml).expect("cases");
        assert_eq!(cases.len(), 3);
        assert_eq!(cases[0].outcome, JUnitOutcome::Passed);
        assert_eq!(cases[0].attempt_count(), 3);
        assert_eq!(cases[0].missing_attempts(&BTreeSet::from([1, 3])), vec![2]);
        assert_eq!(
            cases[0].code_reference(),
            "crate::integration::suite::retry"
        );
        assert_eq!(cases[1].outcome, JUnitOutcome::Broken);
        assert_eq!(cases[2].outcome, JUnitOutcome::Skipped);
    }

    #[test]
    fn supports_empty_passing_cases_and_unescapes_identity() {
        let cases = parse_junit_cases(
            br#"<testsuite name="crate&amp;bin"><testcase name="a&lt;b"/></testsuite>"#,
        )
        .expect("cases");
        assert_eq!(cases[0].suite_id, "crate&bin");
        assert_eq!(cases[0].name, "a<b");
        assert_eq!(cases[0].outcome, JUnitOutcome::Passed);
    }

    #[test]
    fn malformed_and_missing_identity_fail_closed() {
        for (xml, expected) in [
            (
                b"<testsuite name='x'><testcase/></testsuite>".as_slice(),
                JUnitParseError::MissingCaseName,
            ),
            (
                b"<testcase name='x'/>".as_slice(),
                JUnitParseError::MissingSuite,
            ),
            (
                b"<testsuite name='x'>".as_slice(),
                JUnitParseError::Malformed,
            ),
        ] {
            assert_eq!(parse_junit_cases(xml), Err(expected));
        }
    }

    #[test]
    fn input_byte_bound_is_fail_closed() {
        let xml = vec![b'x'; MAX_XML_BYTES + 1];
        assert_eq!(parse_junit_cases(&xml), Err(JUnitParseError::TooLarge));
    }
}
