//! Shared semantic-convention names for Parallax callers.

pub use parallax_proto::semconv::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_wire_names_are_frozen() {
        assert_eq!(PARALLAX_RUN_ID, "parallax.run.id");
        assert_eq!(SERVICE_NAME, "service.name");
        assert_eq!(SERVICE_VERSION, "service.version");
        assert_eq!(DEPLOYMENT_ENVIRONMENT_NAME, "deployment.environment.name");
        assert_eq!(EVENT_NAME, "event.name");
        assert_eq!(LOG_OBSERVED_TS_NANOS, "observed_ts_nanos");
        assert_eq!(EXCEPTION_TYPE, "exception.type");
        assert_eq!(EXCEPTION_MESSAGE, "exception.message");
        assert_eq!(EXCEPTION_STACKTRACE, "exception.stacktrace");
        assert_eq!(
            REQUEST_DURATION_METRICS,
            &["http.server.request.duration", "rpc.server.duration"]
        );
    }
}
