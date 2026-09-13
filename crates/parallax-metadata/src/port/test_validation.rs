use parallax_model::TestExplorerQuery;
use parallax_storage::metadata::{MetadataError, MetadataResult};

pub(super) fn validate_test_explorer_query(query: &TestExplorerQuery) -> MetadataResult<()> {
    if query
        .from_nanos
        .zip(query.to_nanos)
        .is_some_and(|(from, to)| from > to)
    {
        return Err(MetadataError::InvalidInput(
            "test explorer time range is reversed".into(),
        ));
    }
    for value in [
        query.query.as_deref(),
        query.suite.as_deref(),
        query.service.as_deref(),
        query.service_version.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        if value.trim().is_empty() || value.len() > 256 {
            return Err(MetadataError::InvalidInput(
                "test explorer filter must be nonblank and at most 256 bytes".into(),
            ));
        }
    }
    if let Some(configuration) = &query.configuration
        && (!configuration.key.starts_with("test.configuration.")
            || configuration.key.len() > 256
            || configuration.value.len() > 256)
    {
        return Err(MetadataError::InvalidInput(
            "test configuration filter is invalid".into(),
        ));
    }
    Ok(())
}

pub(super) fn validate_test_window(from_nanos: u128, to_nanos: u128) -> MetadataResult<()> {
    if from_nanos > to_nanos {
        return Err(MetadataError::InvalidInput(
            "test result time range is reversed".into(),
        ));
    }
    Ok(())
}
