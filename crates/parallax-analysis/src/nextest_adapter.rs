//! Strict cargo-nextest per-attempt environment normalization.
//!
//! cargo-nextest 0.9.116+ documents the identity variables normalized here:
//! <https://nexte.st/docs/configuration/env-vars/#environment-variables-nextest-sets>.

use parallax_model::TestAttempt;
use std::fmt;
use std::num::NonZeroU32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NextestAttemptContext {
    pub cli_invocation_id: String,
    pub nextest_run_id: String,
    pub binary_id: String,
    pub test_name: String,
    pub attempt: TestAttempt,
    pub total_attempts: NonZeroU32,
    pub attempt_id: String,
    pub traceparent: Option<String>,
}

impl NextestAttemptContext {
    /// Normalize one test process without reading global environment state.
    ///
    /// `CLI_INVOCATION_ID` is mandatory and never substituted with
    /// `NEXTEST_RUN_ID`: the latter identifies the runner execution, not the
    /// generic CLI invocation that owns Parallax correlation.
    pub fn from_lookup(
        mut lookup: impl FnMut(&str) -> Option<String>,
    ) -> Result<Self, NextestContextError> {
        if required(&mut lookup, "NEXTEST")? != "1" {
            return Err(NextestContextError::InvalidValue("NEXTEST"));
        }
        let cli_invocation_id = required(&mut lookup, "CLI_INVOCATION_ID")?;
        let nextest_run_id = required(&mut lookup, "NEXTEST_RUN_ID")?;
        let binary_id = required(&mut lookup, "NEXTEST_BINARY_ID")?;
        let test_name = required(&mut lookup, "NEXTEST_TEST_NAME")?;
        let attempt = parse_nonzero(&mut lookup, "NEXTEST_ATTEMPT")?;
        let total_attempts = parse_nonzero(&mut lookup, "NEXTEST_TOTAL_ATTEMPTS")?;
        if attempt > total_attempts {
            return Err(NextestContextError::AttemptExceedsTotal);
        }
        let attempt_id = required(&mut lookup, "NEXTEST_ATTEMPT_ID")?;
        let traceparent = lookup("TRACEPARENT")
            .map(|value| {
                let value = value.trim();
                if value.is_empty() {
                    Err(NextestContextError::Blank("TRACEPARENT"))
                } else {
                    Ok(value.to_owned())
                }
            })
            .transpose()?;
        Ok(Self {
            cli_invocation_id,
            nextest_run_id,
            binary_id,
            test_name,
            attempt: TestAttempt::new(attempt.get())
                .map_err(|_| NextestContextError::InvalidValue("NEXTEST_ATTEMPT"))?,
            total_attempts,
            attempt_id,
            traceparent,
        })
    }

    #[must_use]
    pub fn code_reference(&self) -> String {
        format!("{}::{}", self.binary_id, self.test_name)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NextestContextError {
    Missing(&'static str),
    Blank(&'static str),
    InvalidValue(&'static str),
    AttemptExceedsTotal,
}

impl fmt::Display for NextestContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(name) => write!(formatter, "missing nextest environment variable {name}"),
            Self::Blank(name) => write!(formatter, "blank nextest environment variable {name}"),
            Self::InvalidValue(name) => {
                write!(formatter, "invalid nextest environment variable {name}")
            }
            Self::AttemptExceedsTotal => {
                formatter.write_str("nextest attempt exceeds total attempts")
            }
        }
    }
}

impl std::error::Error for NextestContextError {}

fn required(
    lookup: &mut impl FnMut(&str) -> Option<String>,
    name: &'static str,
) -> Result<String, NextestContextError> {
    let value = lookup(name).ok_or(NextestContextError::Missing(name))?;
    let value = value.trim();
    if value.is_empty() {
        Err(NextestContextError::Blank(name))
    } else {
        Ok(value.to_owned())
    }
}

fn parse_nonzero(
    lookup: &mut impl FnMut(&str) -> Option<String>,
    name: &'static str,
) -> Result<NonZeroU32, NextestContextError> {
    required(lookup, name)?
        .parse()
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or(NextestContextError::InvalidValue(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn environment() -> BTreeMap<String, String> {
        BTreeMap::from([
            ("NEXTEST".into(), "1".into()),
            ("CLI_INVOCATION_ID".into(), "cli-42".into()),
            ("NEXTEST_RUN_ID".into(), "run-uuid".into()),
            ("NEXTEST_BINARY_ID".into(), "crate::integration".into()),
            ("NEXTEST_TEST_NAME".into(), "suite::retries".into()),
            ("NEXTEST_ATTEMPT".into(), "2".into()),
            ("NEXTEST_TOTAL_ATTEMPTS".into(), "3".into()),
            (
                "NEXTEST_ATTEMPT_ID".into(),
                "run-uuid:crate::integration$suite::retries#2".into(),
            ),
            (
                "TRACEPARENT".into(),
                "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".into(),
            ),
        ])
    }

    fn parse(
        environment: &BTreeMap<String, String>,
    ) -> Result<NextestAttemptContext, NextestContextError> {
        NextestAttemptContext::from_lookup(|name| environment.get(name).cloned())
    }

    #[test]
    fn normalizes_documented_attempt_identity_without_conflating_runner_and_cli() {
        let context = parse(&environment()).expect("context");
        assert_eq!(context.cli_invocation_id, "cli-42");
        assert_eq!(context.nextest_run_id, "run-uuid");
        assert_eq!(context.attempt.get(), 2);
        assert_eq!(context.total_attempts.get(), 3);
        assert_eq!(
            context.code_reference(),
            "crate::integration::suite::retries"
        );
        assert!(context.attempt_id.contains('$'));
        assert!(context.traceparent.is_some());
    }

    #[test]
    fn cli_invocation_is_required_and_never_falls_back_to_nextest_run() {
        let mut environment = environment();
        environment.remove("CLI_INVOCATION_ID");
        assert_eq!(
            parse(&environment),
            Err(NextestContextError::Missing("CLI_INVOCATION_ID"))
        );
    }

    #[test]
    fn attempt_bounds_fail_closed() {
        for (attempt, total, expected) in [
            (
                "0",
                "3",
                NextestContextError::InvalidValue("NEXTEST_ATTEMPT"),
            ),
            ("4", "3", NextestContextError::AttemptExceedsTotal),
            (
                "1",
                "0",
                NextestContextError::InvalidValue("NEXTEST_TOTAL_ATTEMPTS"),
            ),
        ] {
            let mut environment = environment();
            environment.insert("NEXTEST_ATTEMPT".into(), attempt.into());
            environment.insert("NEXTEST_TOTAL_ATTEMPTS".into(), total.into());
            assert_eq!(parse(&environment), Err(expected));
        }
    }

    #[test]
    fn optional_parent_may_be_absent_but_not_blank() {
        let mut environment = environment();
        environment.remove("TRACEPARENT");
        assert_eq!(parse(&environment).expect("context").traceparent, None);
        environment.insert("TRACEPARENT".into(), " ".into());
        assert_eq!(
            parse(&environment),
            Err(NextestContextError::Blank("TRACEPARENT"))
        );
    }
}
