//! Pure derivation of test-reporting registry records from normalized spans.

use parallax_model::{
    ErrorEventRow, SpanRow, TestAttempt, TestCaseIdentity, TestCaseIdentityInput, TestCaseRecord,
    TestConfiguration, TestIdentityError, TestParameter, TestResultKey, TestResultRecord,
    TestStatus, TestVariantIdentity, TestVariantRecord, TraceId,
};
use parallax_semconv as semconv;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivedTestSpan {
    pub case: TestCaseRecord,
    pub variant: TestVariantRecord,
    pub result: TestResultRecord,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TestSpanDerivationError {
    MissingInvocationId,
    InvalidTraceId,
    MissingSpanId,
    EndTimeOverflow,
    InvalidIdentity(TestIdentityError),
    InvalidParameters,
    InvalidAttempt,
    InvalidConfiguration,
}

impl fmt::Display for TestSpanDerivationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingInvocationId => "test span has no CLI invocation identity",
            Self::InvalidTraceId => "test span has an invalid trace ID",
            Self::MissingSpanId => "test span has no span ID",
            Self::EndTimeOverflow => "test span end time overflows",
            Self::InvalidIdentity(_) => "test case identity is malformed",
            Self::InvalidParameters => "test case parameters are malformed",
            Self::InvalidAttempt => "test attempt ordinal is invalid",
            Self::InvalidConfiguration => "test configuration is malformed",
        })
    }
}

impl std::error::Error for TestSpanDerivationError {}

/// Derive registry and result references for a span carrying `test.case.name`.
/// The optional error must be the normalized error derived from the same span;
/// mismatched errors are ignored so fingerprint identity cannot cross spans.
pub fn derive_test_span(
    span: &SpanRow,
    failure: Option<&ErrorEventRow>,
) -> Result<Option<DerivedTestSpan>, TestSpanDerivationError> {
    let Some(attributes) = span.attributes.as_object() else {
        return Ok(None);
    };
    let Some(name) = nonblank_string(attributes, semconv::TEST_CASE_NAME) else {
        return Ok(None);
    };
    let failure = failure
        .filter(|failure| failure.trace_id == span.trace_id && failure.span_id == span.span_id);
    let ended_at_nanos = span
        .ts_nanos
        .checked_add(span.duration_ns)
        .ok_or(TestSpanDerivationError::EndTimeOverflow)?;
    let identity_input = TestCaseIdentityInput {
        explicit_id: owned_nonblank(attributes, semconv::TEST_CASE_ID),
        code_reference: owned_nonblank(attributes, semconv::TEST_CODE_REFERENCE),
        suite_path: nonblank_string(attributes, semconv::TEST_SUITE_NAME)
            .map(|suite| vec![suite.to_string()])
            .unwrap_or_default(),
        name: name.to_string(),
    };
    let identity = TestCaseIdentity::derive(&identity_input)
        .map_err(TestSpanDerivationError::InvalidIdentity)?;
    let parameters = parameters(attributes.get(semconv::TEST_CASE_PARAMETERS))?;
    let variant = TestVariantIdentity::derive(identity.key(), &parameters)
        .map_err(TestSpanDerivationError::InvalidIdentity)?;
    let invocation_id = span
        .invocation_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(TestSpanDerivationError::MissingInvocationId)?;
    if span.span_id.trim().is_empty() {
        return Err(TestSpanDerivationError::MissingSpanId);
    }
    let status = classify_test_status(span, failure);
    let case_key = identity.key().clone();
    let variant_key = variant.key().clone();
    Ok(Some(DerivedTestSpan {
        case: TestCaseRecord {
            key: case_key.clone(),
            identity_source: identity.source(),
            explicit_id: identity_input.explicit_id,
            code_reference: identity_input.code_reference,
            suite_path: identity_input.suite_path,
            name: identity_input.name,
            first_seen_nanos: ended_at_nanos,
            last_seen_nanos: ended_at_nanos,
        },
        variant: TestVariantRecord {
            key: variant_key.clone(),
            case_key,
            parameters: variant.parameters().to_vec(),
            first_seen_nanos: ended_at_nanos,
            last_seen_nanos: ended_at_nanos,
        },
        result: TestResultRecord {
            key: TestResultKey {
                variant_key,
                invocation_id: invocation_id.to_string(),
                attempt: attempt(attributes.get(semconv::TEST_ATTEMPT_ORDINAL))?,
            },
            status,
            trace_id: span
                .trace_id
                .parse::<TraceId>()
                .map_err(|_| TestSpanDerivationError::InvalidTraceId)?,
            span_id: span.span_id.clone(),
            started_at_nanos: span.ts_nanos,
            ended_at_nanos,
            service: span.service.clone(),
            service_version: dimension(span, semconv::SERVICE_VERSION),
            vcs_head_revision: dimension(span, semconv::VCS_REF_HEAD_REVISION),
            configuration: configuration(span)?,
            failure_fingerprint: matches!(status, TestStatus::Failed | TestStatus::Broken)
                .then(|| failure.map(|failure| failure.fingerprint.clone()))
                .flatten(),
        },
    }))
}

#[must_use]
pub fn classify_test_status(span: &SpanRow, failure: Option<&ErrorEventRow>) -> TestStatus {
    let attributes = span.attributes.as_object();
    let declared = string(attributes, semconv::TEST_CASE_RESULT_STATUS);
    match declared {
        Some("pass" | "passed") => return TestStatus::Passed,
        Some("skip" | "skipped") => return TestStatus::Skipped,
        Some("fail" | "failed") => {}
        Some(_) => return TestStatus::Unknown,
        None if span.status_code == "STATUS_CODE_OK" => return TestStatus::Passed,
        None if span.status_code != "STATUS_CODE_ERROR" && failure.is_none() => {
            return TestStatus::Unknown;
        }
        None => {}
    }
    match string(attributes, semconv::TEST_CASE_FAILURE_KIND) {
        Some(semconv::TEST_FAILURE_KIND_ASSERTION) => TestStatus::Failed,
        Some(semconv::TEST_FAILURE_KIND_HARNESS) => TestStatus::Broken,
        Some(_) => TestStatus::Unknown,
        None if failure.is_some_and(|failure| assertion_family(&failure.error_type)) => {
            TestStatus::Failed
        }
        None => TestStatus::Broken,
    }
}

fn assertion_family(error_type: &str) -> bool {
    let leaf = error_type
        .rsplit([':', '.', '$'])
        .find(|part| !part.is_empty())
        .unwrap_or(error_type);
    matches!(
        leaf,
        "AssertionError"
            | "AssertionFailedError"
            | "MultipleFailuresError"
            | "ComparisonFailure"
            | "AssertionFailed"
    )
}

fn parameters(value: Option<&Value>) -> Result<Vec<TestParameter>, TestSpanDerivationError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    if let Some(encoded) = value.as_str() {
        return serde_json::from_str(encoded)
            .map_err(|_| TestSpanDerivationError::InvalidParameters);
    }
    serde_json::from_value(value.clone()).map_err(|_| TestSpanDerivationError::InvalidParameters)
}

fn attempt(value: Option<&Value>) -> Result<TestAttempt, TestSpanDerivationError> {
    let ordinal = match value {
        None => 1,
        Some(value) => value
            .as_u64()
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(TestSpanDerivationError::InvalidAttempt)?,
    };
    TestAttempt::new(ordinal).map_err(|_| TestSpanDerivationError::InvalidAttempt)
}

fn configuration(span: &SpanRow) -> Result<TestConfiguration, TestSpanDerivationError> {
    let mut dimensions = BTreeMap::new();
    for source in [span.resource.as_object(), span.attributes.as_object()] {
        let Some(source) = source else { continue };
        for (key, value) in source
            .iter()
            .filter(|(key, _)| key.starts_with("test.configuration."))
        {
            let value = value
                .as_str()
                .ok_or(TestSpanDerivationError::InvalidConfiguration)?;
            dimensions.insert(key.clone(), value.to_string());
        }
    }
    Ok(TestConfiguration { dimensions })
}

fn dimension(span: &SpanRow, key: &str) -> Option<String> {
    string(span.attributes.as_object(), key)
        .or_else(|| string(span.resource.as_object(), key))
        .map(str::to_string)
}

fn owned_nonblank(attributes: &Map<String, Value>, key: &str) -> Option<String> {
    nonblank_string(attributes, key).map(str::to_string)
}

fn nonblank_string<'a>(attributes: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    string(Some(attributes), key)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn string<'a>(attributes: Option<&'a Map<String, Value>>, key: &str) -> Option<&'a str> {
    attributes?.get(key)?.as_str()
}

#[cfg(test)]
mod tests;
