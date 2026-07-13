//! Stable, sanitized GraphQL error projection.

use juniper::{FieldError, graphql_value};
use parallax_storage::adapter::{StorageError, StorageErrorKind};
use parallax_storage::metadata::{MetadataError, MetadataErrorKind};
use std::error::Error;

const INVALID_INPUT: &str = "INVALID_INPUT";
const NOT_FOUND: &str = "NOT_FOUND";
const CONFLICT: &str = "CONFLICT";
const UNAVAILABLE: &str = "UNAVAILABLE";
const TIMEOUT: &str = "TIMEOUT";
const INTERNAL: &str = "INTERNAL";

pub(super) fn invalid(error: impl std::fmt::Display) -> FieldError {
    field(error.to_string(), INVALID_INPUT)
}

pub(super) fn internal(error: impl Error + 'static) -> FieldError {
    let error: &dyn Error = &error;
    let (message, code) = if let Some(storage) = error.downcast_ref::<StorageError>() {
        match storage.kind() {
            StorageErrorKind::Unavailable | StorageErrorKind::Transport => {
                ("telemetry store unavailable", UNAVAILABLE)
            }
            StorageErrorKind::Timeout => ("telemetry operation timed out", TIMEOUT),
            StorageErrorKind::Query | StorageErrorKind::Schema | StorageErrorKind::Internal => {
                ("internal server error", INTERNAL)
            }
        }
    } else if let Some(metadata) = error.downcast_ref::<MetadataError>() {
        match metadata.kind() {
            MetadataErrorKind::InvalidInput => ("invalid metadata input", INVALID_INPUT),
            MetadataErrorKind::NotFound => ("metadata record not found", NOT_FOUND),
            MetadataErrorKind::Conflict => ("metadata conflict", CONFLICT),
            MetadataErrorKind::Unavailable => ("metadata store unavailable", UNAVAILABLE),
            MetadataErrorKind::Timeout => ("metadata operation timed out", TIMEOUT),
            MetadataErrorKind::Schema | MetadataErrorKind::Internal => {
                ("internal server error", INTERNAL)
            }
        }
    } else {
        ("internal server error", INTERNAL)
    };
    tracing::error!(error = %error, code, "GraphQL resolver failed");
    field(message, code)
}

fn field(message: impl std::fmt::Display, code: &'static str) -> FieldError {
    FieldError::new(message, graphql_value!({ "code": code }))
}

#[cfg(test)]
mod tests;
