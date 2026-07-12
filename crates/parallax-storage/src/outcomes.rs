use std::fmt::Display;

pub(crate) fn warn_error<T, E: Display>(result: Result<T, E>, operation: &str) {
    if let Err(error) = result {
        tracing::warn!(%error, operation, "best-effort storage operation failed");
    }
}
