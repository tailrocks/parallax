//! Stable test-case and variant identity derivation.

use super::TestParameter;
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use std::str::FromStr;

macro_rules! test_key {
    ($name:ident, $prefix:literal) => {
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);
        impl $name {
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl FromStr for $name {
            type Err = TestKeyError;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let digest = value
                    .strip_prefix(concat!($prefix, ":"))
                    .ok_or(TestKeyError)?;
                if digest.len() != 64
                    || !digest
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                {
                    return Err(TestKeyError);
                }
                Ok(Self(value.to_string()))
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                String::deserialize(deserializer)?
                    .parse()
                    .map_err(serde::de::Error::custom)
            }
        }
    };
}
test_key!(TestCaseKey, "tc1");
test_key!(TestVariantKey, "tv1");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TestKeyError;
impl fmt::Display for TestKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("test key must have its version prefix and 64 lowercase hex digits")
    }
}
impl std::error::Error for TestKeyError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestCaseIdentityInput {
    pub explicit_id: Option<String>,
    pub code_reference: Option<String>,
    pub suite_path: Vec<String>,
    pub name: String,
}
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestCaseIdentitySource {
    Explicit,
    CodeReference,
    NamePath,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestCaseIdentity {
    key: TestCaseKey,
    source: TestCaseIdentitySource,
}

impl TestCaseIdentity {
    pub fn derive(input: &TestCaseIdentityInput) -> Result<Self, TestIdentityError> {
        let (source, parts) = if let Some(value) = input.explicit_id.as_deref() {
            (
                TestCaseIdentitySource::Explicit,
                vec![nonblank(value, TestIdentityError::BlankExplicitId)?],
            )
        } else if let Some(value) = input.code_reference.as_deref() {
            (
                TestCaseIdentitySource::CodeReference,
                vec![nonblank(value, TestIdentityError::BlankCodeReference)?],
            )
        } else {
            let mut parts = input
                .suite_path
                .iter()
                .map(|part| nonblank(part, TestIdentityError::BlankSuiteSegment))
                .collect::<Result<Vec<_>, _>>()?;
            parts.push(nonblank(&input.name, TestIdentityError::BlankName)?);
            (TestCaseIdentitySource::NamePath, parts)
        };
        let source_name = match source {
            TestCaseIdentitySource::Explicit => "explicit",
            TestCaseIdentitySource::CodeReference => "code_reference",
            TestCaseIdentitySource::NamePath => "name_path",
        };
        let mut framed = vec![source_name];
        framed.extend(parts);
        Ok(Self {
            key: TestCaseKey(format!(
                "tc1:{}",
                framed_sha256(b"parallax.test.case.v1", &framed)
            )),
            source,
        })
    }
    #[must_use]
    pub fn key(&self) -> &TestCaseKey {
        &self.key
    }
    #[must_use]
    pub fn source(&self) -> TestCaseIdentitySource {
        self.source
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestVariantIdentity {
    key: TestVariantKey,
    parameters: Vec<TestParameter>,
}
impl TestVariantIdentity {
    pub fn derive(
        case_key: &TestCaseKey,
        parameters: &[TestParameter],
    ) -> Result<Self, TestIdentityError> {
        let included = parameters
            .iter()
            .filter(|parameter| !parameter.excluded)
            .map(|parameter| {
                nonblank(&parameter.name, TestIdentityError::BlankParameterName)?;
                Ok(parameter.clone())
            })
            .collect::<Result<Vec<_>, TestIdentityError>>()?;
        let mut parts = vec![case_key.as_str()];
        for parameter in &included {
            parts.push(&parameter.name);
            parts.push(&parameter.value);
        }
        Ok(Self {
            key: TestVariantKey(format!(
                "tv1:{}",
                framed_sha256(b"parallax.test.variant.v1", &parts)
            )),
            parameters: included,
        })
    }
    #[must_use]
    pub fn key(&self) -> &TestVariantKey {
        &self.key
    }
    #[must_use]
    pub fn parameters(&self) -> &[TestParameter] {
        &self.parameters
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TestIdentityError {
    BlankExplicitId,
    BlankCodeReference,
    BlankSuiteSegment,
    BlankName,
    BlankParameterName,
}
impl fmt::Display for TestIdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BlankExplicitId => "explicit test ID cannot be blank",
            Self::BlankCodeReference => "test code reference cannot be blank",
            Self::BlankSuiteSegment => "test suite segment cannot be blank",
            Self::BlankName => "test name cannot be blank",
            Self::BlankParameterName => "included test parameter name cannot be blank",
        })
    }
}
impl std::error::Error for TestIdentityError {}
fn nonblank(value: &str, error: TestIdentityError) -> Result<&str, TestIdentityError> {
    let value = value.trim();
    if value.is_empty() {
        Err(error)
    } else {
        Ok(value)
    }
}
fn framed_sha256(domain: &[u8], parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    for part in parts {
        hasher.update(u64::try_from(part.len()).unwrap_or(u64::MAX).to_be_bytes());
        hasher.update(part.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}
