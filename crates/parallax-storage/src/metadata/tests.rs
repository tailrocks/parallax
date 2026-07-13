use super::*;
use std::error::Error as _;

#[test]
fn metadata_error_kinds_are_stable() -> anyhow::Result<()> {
    let cases = [
        (
            MetadataError::InvalidInput("bad".into()),
            MetadataErrorKind::InvalidInput,
        ),
        (
            MetadataError::NotFound("missing".into()),
            MetadataErrorKind::NotFound,
        ),
        (
            MetadataError::Conflict("stale".into()),
            MetadataErrorKind::Conflict,
        ),
        (
            MetadataError::Unavailable {
                source: anyhow::anyhow!("connection detail"),
            },
            MetadataErrorKind::Unavailable,
        ),
        (
            MetadataError::Timeout {
                source: anyhow::anyhow!("timer detail"),
            },
            MetadataErrorKind::Timeout,
        ),
        (
            MetadataError::Schema {
                source: anyhow::anyhow!("schema detail"),
            },
            MetadataErrorKind::Schema,
        ),
        (
            MetadataError::internal(anyhow::anyhow!("internal detail")),
            MetadataErrorKind::Internal,
        ),
    ];
    for (error, kind) in cases {
        anyhow::ensure!(
            error.kind() == kind
                && (error.source().is_some()
                    || matches!(
                        kind,
                        MetadataErrorKind::InvalidInput
                            | MetadataErrorKind::NotFound
                            | MetadataErrorKind::Conflict
                    ))
        );
    }
    Ok(())
}

#[test]
fn internal_display_is_sanitized_but_source_chain_is_preserved() -> anyhow::Result<()> {
    let error = MetadataError::internal(anyhow::anyhow!(
        "SELECT secret FROM private_path WHERE token='credential'"
    ));
    anyhow::ensure!(
        error.to_string() == "metadata operation failed"
            && error
                .source()
                .is_some_and(|source| source.to_string().contains("private_path"))
    );
    Ok(())
}
