//! Validated identifiers with transparent wire and persistence representations.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A W3C/OpenTelemetry 16-byte trace identifier, represented as 32 lowercase
/// hexadecimal characters at text boundaries.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TraceId(String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TraceIdError;

impl TraceId {
    pub const HEX_LEN: usize = 32;
    pub const BYTE_LEN: usize = 16;

    pub fn from_otlp_bytes(bytes: &[u8]) -> Result<Self, TraceIdError> {
        if bytes.len() != Self::BYTE_LEN || bytes.iter().all(|byte| *byte == 0) {
            return Err(TraceIdError);
        }
        Ok(Self(
            bytes.iter().map(|byte| format!("{byte:02x}")).collect(),
        ))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for TraceId {
    type Err = TraceIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != Self::HEX_LEN
            || value.bytes().all(|byte| byte == b'0')
            || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(TraceIdError);
        }
        Ok(Self(value.to_ascii_lowercase()))
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl fmt::Display for TraceIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("trace ID must be 32 non-zero hexadecimal characters")
    }
}

impl std::error::Error for TraceIdError {}

#[cfg(test)]
mod tests;
