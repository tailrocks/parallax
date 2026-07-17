//! Nextest + JUnit export lifecycle for plan 155 D9 adapters.
//!
//! Composes pure nextest attempt identity and JUnit reconciliation into a
//! durable export report. Missing ordinals stay gap evidence only — this module
//! never fabricates raw test results or statuses for unobserved attempts.

use crate::junit_reconcile::{
    JUnitAttemptReconciliation, JUnitCaseEvidence, JUnitParseError, JUnitReconcileError,
    parse_junit_cases, reconcile_junit_attempts,
};
use crate::nextest_adapter::{NextestAttemptContext, NextestContextError};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Observed telemetry attempt ordinals keyed by JUnit code-reference identity.
pub type ObservedAttempts = BTreeMap<String, BTreeSet<u32>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NextestExportSession {
    pub context: NextestAttemptContext,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JUnitExportReport {
    pub cases: Vec<JUnitCaseEvidence>,
    pub reconciliations: Vec<JUnitAttemptReconciliation>,
    /// Gap rows only — never invent status; adapters surface missing ordinals.
    pub missing_attempt_gaps: Vec<MissingAttemptGap>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissingAttemptGap {
    pub code_reference: String,
    pub attempt: u32,
    pub attempt_count: u32,
    pub terminal_outcome: crate::junit_reconcile::JUnitOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExportError {
    Nextest(NextestContextError),
    JUnitParse(JUnitParseError),
    JUnitReconcile(JUnitReconcileError),
}

impl fmt::Display for ExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nextest(error) => write!(formatter, "nextest export: {error}"),
            Self::JUnitParse(error) => write!(formatter, "junit parse: {error}"),
            Self::JUnitReconcile(error) => write!(formatter, "junit reconcile: {error}"),
        }
    }
}

impl std::error::Error for ExportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Nextest(error) => Some(error),
            Self::JUnitParse(error) => Some(error),
            Self::JUnitReconcile(error) => Some(error),
        }
    }
}

/// Begin a nextest per-attempt export session from process environment.
pub fn nextest_export_session_from_env<F>(lookup: F) -> Result<NextestExportSession, ExportError>
where
    F: FnMut(&str) -> Option<String>,
{
    let context = NextestAttemptContext::from_lookup(lookup).map_err(ExportError::Nextest)?;
    Ok(NextestExportSession { context })
}

/// Parse JUnit authority XML and reconcile against observed attempt ordinals.
///
/// Persistence guidance: write only observed telemetry rows (normal OTLP path).
/// Emit [`MissingAttemptGap`] to operator evidence / UI badges — do not upsert
/// fabricated `TestResultRecord` rows for missing ordinals.
pub fn junit_export_report(
    xml: &[u8],
    observed: &ObservedAttempts,
) -> Result<JUnitExportReport, ExportError> {
    let cases = parse_junit_cases(xml).map_err(ExportError::JUnitParse)?;
    let reconciliations =
        reconcile_junit_attempts(&cases, observed).map_err(ExportError::JUnitReconcile)?;
    let mut missing_attempt_gaps = Vec::new();
    for row in &reconciliations {
        for attempt in &row.missing_attempts {
            missing_attempt_gaps.push(MissingAttemptGap {
                code_reference: row.code_reference.clone(),
                attempt: *attempt,
                attempt_count: row.attempt_count,
                terminal_outcome: row.terminal_outcome,
            });
        }
    }
    Ok(JUnitExportReport {
        cases,
        reconciliations,
        missing_attempt_gaps,
    })
}

/// Full adapter lifecycle: nextest session identity + JUnit gap report.
pub fn export_lifecycle_report<F>(
    lookup: F,
    junit_xml: Option<&[u8]>,
    observed: &ObservedAttempts,
) -> Result<(NextestExportSession, Option<JUnitExportReport>), ExportError>
where
    F: FnMut(&str) -> Option<String>,
{
    let session = nextest_export_session_from_env(lookup)?;
    let report = match junit_xml {
        Some(xml) => Some(junit_export_report(xml, observed)?),
        None => None,
    };
    Ok((session, report))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::junit_reconcile::JUnitOutcome;

    fn env(map: BTreeMap<&'static str, &'static str>) -> impl FnMut(&str) -> Option<String> {
        move |name| map.get(name).map(|value| (*value).to_string())
    }

    fn base_env() -> BTreeMap<&'static str, &'static str> {
        BTreeMap::from([
            ("NEXTEST", "1"),
            ("NEXTEST_RUN_ID", "run-uuid"),
            ("NEXTEST_BINARY_ID", "crate"),
            ("NEXTEST_TEST_NAME", "suite::case"),
            ("NEXTEST_ATTEMPT", "1"),
            ("NEXTEST_TOTAL_ATTEMPTS", "2"),
            ("NEXTEST_ATTEMPT_ID", "attempt-1"),
            ("CLI_INVOCATION_ID", "inv-1"),
        ])
    }

    #[test]
    fn nextest_session_requires_cli_invocation_not_run_id() {
        let mut vars = base_env();
        let session = nextest_export_session_from_env(env(vars.clone())).expect("session");
        assert_eq!(session.context.cli_invocation_id, "inv-1");
        assert_eq!(session.context.nextest_run_id, "run-uuid");
        vars.remove("CLI_INVOCATION_ID");
        nextest_export_session_from_env(env(vars)).expect_err("cli invocation required");
    }

    #[test]
    fn junit_report_surfaces_missing_gaps_without_fabricating_results() {
        // Two flakyFailure priors + terminal pass => attempt_count 3; observe 1+3.
        let xml = br#"<testsuites><testsuite name="crate::integration">
          <testcase classname="suite" name="retry"><flakyFailure/><flakyFailure/></testcase>
        </testsuite></testsuites>"#;
        let mut observed = ObservedAttempts::new();
        observed.insert(
            "crate::integration::suite::retry".into(),
            BTreeSet::from([1, 3]),
        );
        let report = junit_export_report(xml, &observed).expect("report");
        assert_eq!(report.cases.len(), 1);
        assert_eq!(report.missing_attempt_gaps.len(), 1);
        assert_eq!(report.missing_attempt_gaps[0].attempt, 2);
        assert_eq!(
            report.reconciliations[0].terminal_outcome,
            JUnitOutcome::Passed
        );
    }

    #[test]
    fn lifecycle_composes_session_and_optional_junit() {
        let (session, report) =
            export_lifecycle_report(env(base_env()), None, &ObservedAttempts::new())
                .expect("lifecycle");
        assert_eq!(session.context.cli_invocation_id, "inv-1");
        assert!(report.is_none());
    }
}
