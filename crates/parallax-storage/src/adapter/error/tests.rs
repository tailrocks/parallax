use super::*;
use std::error::Error as _;

#[test]
fn storage_error_taxonomy_preserves_operator_context_and_sources() -> anyhow::Result<()> {
    let errors = [
        (
            StorageError::Transport {
                source: anyhow::anyhow!("https://credential@private-host"),
            },
            StorageErrorKind::Transport,
            "telemetry transport failed",
        ),
        (
            StorageError::Query {
                source: anyhow::anyhow!("SELECT secret FROM private_table"),
            },
            StorageErrorKind::Query,
            "telemetry query failed",
        ),
        (
            StorageError::Schema {
                source: anyhow::anyhow!("ALTER private_table"),
            },
            StorageErrorKind::Schema,
            "telemetry schema operation failed",
        ),
        (
            StorageError::Unavailable {
                source: anyhow::anyhow!("socket detail"),
            },
            StorageErrorKind::Unavailable,
            "telemetry store unavailable",
        ),
        (
            StorageError::Timeout {
                source: anyhow::anyhow!("deadline detail"),
            },
            StorageErrorKind::Timeout,
            "telemetry operation timed out",
        ),
        (
            StorageError::internal(anyhow::anyhow!("internal detail")),
            StorageErrorKind::Internal,
            "telemetry operation failed",
        ),
    ];
    for (error, kind, display) in errors {
        anyhow::ensure!(
            error.kind() == kind
                && error.to_string().starts_with(display)
                && error.source().is_some()
        );
    }
    Ok(())
}
