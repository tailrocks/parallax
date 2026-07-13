use super::*;

#[test]
fn maps_codes_and_never_exposes_internal_details() -> anyhow::Result<()> {
    let cases = [
        (
            invalid("bad range"),
            "bad range",
            graphql_value!({ "code": "INVALID_INPUT" }),
        ),
        (
            internal(StorageError::Query {
                source: anyhow::anyhow!("SELECT secret FROM private_table"),
            }),
            "internal server error",
            graphql_value!({ "code": "INTERNAL" }),
        ),
        (
            internal(StorageError::Unavailable {
                source: anyhow::anyhow!("https://credential@private-host"),
            }),
            "telemetry store unavailable",
            graphql_value!({ "code": "UNAVAILABLE" }),
        ),
        (
            internal(MetadataError::NotFound("private-id".into())),
            "metadata record not found",
            graphql_value!({ "code": "NOT_FOUND" }),
        ),
    ];
    for (error, message, extensions) in cases {
        anyhow::ensure!(error.message() == message && error.extensions() == &extensions);
        anyhow::ensure!(
            !error.message().contains("SELECT")
                && !error.message().contains("credential")
                && !error.message().contains("private")
        );
    }
    Ok(())
}
