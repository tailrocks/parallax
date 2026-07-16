use super::build_schema;

#[test]
fn schema_sdl_snapshot() {
    let sdl = build_schema().as_sdl();
    for needle in [
        "type Query",
        "type Mutation",
        "issues(",
        "trace(",
        "logsAround(",
        "serviceMap(",
        "metricSeries(",
        "bundle(",
        "story(",
        "sql(",
        "invocation(",
        "invocations(",
        "observedInvocations(",
        "tracesByInvocation(",
        "logsByInvocation(",
        "sessions(",
        "screenVisits(",
        "uiActions(",
        "backgroundCycles(",
        "jobs(",
        "conversations(",
        "invocationStart(",
        "invocationFinish(",
    ] {
        assert!(
            sdl.contains(needle),
            "schema SDL missing sentinel {needle:?}"
        );
    }
    // The runs vocabulary is fully retired (operator, 2026-07-17).
    for forbidden in [
        "observedRuns",
        "tracesByRun",
        "logsByRun",
        "runStart",
        "runFinish",
        "runId",
    ] {
        assert!(
            !sdl.contains(forbidden),
            "schema SDL still exposes retired field {forbidden:?}"
        );
    }
}
